use crate::tg_bot::calculator::calculate_coupon;
use crate::tg_bot::price_parser::{file_router, PriceDTO};
use anyhow::Result;
use dptree::case;
use serde::Deserialize;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, DpHandlerDescription};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use tracing::{info, instrument};

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    Selected,
    Stock,
    Price,
    Calculate,
}

/// Поддерживаются следующие команды
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    /// Показать этот текст
    Help,
    /// Начать работу
    Start,
    /// Отменить запрос
    Cancel,
}

#[instrument(name = "router")]
pub fn router() -> Handler<'static, Result<()>, DpHandlerDescription> {
    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(
            case![State::Selected]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel));

    let callback_query_handler =
        Update::filter_callback_query().branch(case![State::Selected].endpoint(cb_handler));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Stock].endpoint(stock))
        .branch(case![State::Price].endpoint(price))
        .branch(case![State::Calculate].endpoint(calculate))
        .branch(dptree::endpoint(text_handler));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}
#[instrument(name = "cancel handler", skip_all)]
async fn cancel(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    bot.send_message(msg.chat.id, "Запрос отменен.").await?;
    dialogue.exit().await?;
    Ok(())
}

#[instrument(
    name = "text handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn text_handler(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    if let Some(f) = msg.document() {
        let file = f.file_name.clone().unwrap_or_default();
        info!("Получен файл '{file}'");
        let id = f.file.id.clone();
        let r = bot.get_file(id).await?;
        let uri = r.path;
        let token = bot.token();
        let url = format!("https://api.telegram.org/file/bot{token}/{uri}");
        let text = file_router(&url)
            .await
            .map_err(|e| tracing::error!("{e:?}"))
            .unwrap_or(String::from("Неизвестный формат файла"));
        info!("Отправляю {text}");
        bot.send_message(msg.chat.id, text).await?;
    } else if let Some(text) = msg.text() {
        info!("Получен текст '{text}'");
        let first_name = msg
            .from
            .clone()
            .map(|u| u.first_name.clone())
            .unwrap_or(String::from("Господин"));
        let name = format!("Уважаемый {first_name}");
        match text {
            "Остатки" => {
                let text = format!("{name}, введите строку поиска остатков");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Stock).await?;
            }
            "Цены" => {
                let text = format!("{name}, введите строку поиска цен");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Price).await?;
            }
            "Калькулятор" => {
                let text = format!("{name}, введите через пробел максимальную длину и ширину помещения, а также ширину рулона (все в метрах)");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Calculate).await?;
            }
            _ => {
                let answer =
                    "Не могу обработать сообщение. Введите /help для инструкций по использованию.";
                info!("Отправляю {answer}");
                bot.send_message(dialogue.chat_id(), answer).await?;
                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

fn make_keyboard() -> teloxide::types::KeyboardMarkup {
    let mut keyboard: Vec<Vec<teloxide::types::KeyboardButton>> = vec![];
    let top_buttons = vec![
        teloxide::types::KeyboardButton::new("Остатки"),
        teloxide::types::KeyboardButton::new("Цены"),
    ];
    let bottom_buttons = vec![teloxide::types::KeyboardButton::new("Калькулятор")];
    keyboard.push(top_buttons);
    keyboard.push(bottom_buttons);
    teloxide::types::KeyboardMarkup::new(keyboard).resize_keyboard()
}

#[instrument(name = "sending keyboard", skip_all)]
async fn send_keyboard<T: Into<teloxide::types::Recipient>>(
    bot: Bot,
    chat_id: T,
    text: &str,
) -> Result<()> {
    let kb = make_keyboard();
    bot.send_message(chat_id, text).reply_markup(kb).await?;
    Ok(())
}
#[instrument(
    name = "start handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn start(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    let name = msg
        .from
        .map(|u| format!("уважаемый {name}", name = u.first_name))
        .unwrap_or(String::from("Господин"));
    let text = format!("Рад снова вас видеть, {name}. Выберите пункт меню:");
    info!("Отправляю: {text}");
    send_keyboard(bot, msg.chat.id, &text).await?;
    dialogue.update(State::Selected).await?;
    Ok(())
}

#[instrument(
    name = "help handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn help(bot: Bot, msg: Message) -> Result<()> {
    info!(
        "Отправляю: {answer}",
        answer = Command::descriptions().to_string()
    );
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    // send_keyboard(bot, msg.chat.id, "Выберите пункт меню").await?;
    Ok(())
}
#[instrument(
    name = "callback handler",
    skip_all,
    fields(from = %q.from.clone().full_name())
)]
async fn cb_handler(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    q: CallbackQuery,
) -> Result<()> {
    let name = format!("Уважаемый {name}", name = q.from.first_name);
    if let Some(variant) = &q.data {
        info!("Получено: {variant}");
        match variant.as_str() {
            "stock" => {
                let text = format!("{name}, введите строку поиска остатков");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Stock).await?;
            }
            "price" => {
                let text = format!("{name}, введите строку поиска цен");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Price).await?;
            }
            "calculate" => {
                let text = format!("{name}, введите через пробел максимальную длину и ширину помещения, а также ширину рулона (все в метрах)");
                bot.send_message(dialogue.chat_id(), text).await?;
                dialogue.update(State::Calculate).await?;
            }
            _ => {
                bot.send_message(dialogue.chat_id(), "Неизвестный вариант")
                    .await?;
                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct StockItem {
    name: String,
    stock: f64,
    updated: String,
}
#[instrument(
    name = "stock handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn stock(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    if let Some(search_string) = msg.text() {
        info!("Получено: {search_string}");
        let uri =
            format!("https://friday-api-vqkh.shuttle.app/api/v1/stock?search={search_string}");
        let response = reqwest::get(uri).await?;
        let mut body = response.json::<Vec<StockItem>>().await?;
        let mut answer = String::new();
        if body.is_empty() {
            answer = String::from("Остатки не найдены")
        } else {
            if body.len() > 20 {
                body = body.drain(..20).collect::<Vec<_>>();
                answer = String::from("ВЫВЕДУ ТОЛЬКО ПЕРВЫЕ 20 ПОЗИЦИЙ\n\n\n");
            }
            for item in body {
                let updated = item
                    .updated
                    .clone()
                    .split('T')
                    .collect::<Vec<_>>()
                    .first()
                    .map(|w| w.to_string())
                    .unwrap_or_default();
                answer = format!(
                    "{answer}\nНазвание: {name}\nВ наличии: {stock}\nДата: {updated}\n-----------",
                    name = item.name,
                    stock = item.stock
                );
            }
        }
        info!("Отправляю: {answer}");
        bot.send_message(msg.chat.id, answer).await?;
    }
    dialogue.exit().await?;
    // send_keyboard(bot, msg.chat.id, "Выберите пункт меню").await?;
    // dialogue.update(State::Selected).await?;
    Ok(())
}
#[instrument(
    name = "price handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn price(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    if let Some(search_string) = msg.text() {
        info!("Получено: {search_string}");
        let uri =
            format!("https://friday-api-vqkh.shuttle.app/api/v1/prices?search={search_string}");
        // let uri = format!("http://localhost:8000/api/v1/prices?search={search_string}");
        let response = reqwest::get(uri).await?;
        let value = response.json::<serde_json::Value>().await?;
        let mut answer = String::new();
        if let Ok(mut body) = serde_json::from_value::<Vec<PriceDTO>>(value.clone()) {
            if body.is_empty() {
                answer = String::from("Цены не найдены")
            } else {
                if body.len() > 20 {
                    body = body.drain(..20).collect::<Vec<_>>();
                    answer = String::from("ВЫВЕДУ ТОЛЬКО ПЕРВЫЕ 20 ПОЗИЦИЙ\n\n\n");
                }
                for item in body {
                    let updated = item
                        .updated
                        .clone()
                        .split('T')
                        .collect::<Vec<_>>()
                        .first()
                        .map(|w| w.to_string())
                        .unwrap_or_default();
                    answer = format!(
                        "{answer}\nНазвание: {name}\nЦена рулон: {rrp}\nЦена купон: {rcp}\nДата: {updated}\n-----------",
                        name = item.name,
                        rrp = item.recommended_roll_price,
                        rcp = item.recommended_coupon_price
                    );
                }
            }
        } else {
            answer = serde_json::to_string_pretty(&value)?;
        }
        info!("Отправляю: {answer}");
        bot.send_message(msg.chat.id, answer).await?;
    }
    dialogue.exit().await?;
    // send_keyboard(bot, msg.chat.id, "Выберите пункт меню").await?;
    // dialogue.update(State::Selected).await?;
    Ok(())
}
#[instrument(
    name = "calculate handler",
    skip_all,
    fields(from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default())
)]
async fn calculate(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
) -> Result<()> {
    if let Some(measures) = msg.text() {
        info!("Получено: {measures}");
        let answer = calculate_coupon(measures);
        info!("Отправляю: {answer}");
        bot.send_message(msg.chat.id, answer).await?;
    }
    dialogue.exit().await?;
    // send_keyboard(bot, msg.chat.id, "Выберите пункт меню").await?;
    // dialogue.update(State::Selected).await?;
    Ok(())
}
