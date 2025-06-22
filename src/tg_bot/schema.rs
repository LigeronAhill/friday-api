use crate::storage::{PriceStorage, StockStorage};
use crate::tg_bot::calculator::calculate_coupon;
use crate::tg_bot::price_parser::file_router;
use anyhow::Result;
use dptree::case;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{dialogue, DpHandlerDescription};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;
use teloxide::utils::command::BotCommands;
use tracing::{info, instrument};

const STOCK: &str = "📦 Остатки";
const PRICES: &str = "🏷️ Цены";
const CALCULATOR: &str = "🧮 Калькулятор";
const ADMINS: [u64; 2] = [337581254, 456660297];

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
    let kb = make_keyboard();
    bot.send_message(msg.chat.id, "Запрос отменен.")
        .reply_markup(kb)
        .await?;
    dialogue.exit().await?;
    Ok(())
}

#[instrument(
    name = "text handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
)]
async fn text_handler(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
    price_storage: Arc<PriceStorage>,
) -> Result<()> {
    let user = msg.from.clone().map(|u| u.id.0).unwrap_or_default();
    let kb = make_keyboard();
    if let Some(f) = msg.document() {
        if !is_admin(&msg) {
            bot.send_message(msg.chat.id, "От вас не могу получать файлы")
                .reply_markup(kb)
                .await?;
        } else {
            let file = f.file_name.clone().unwrap_or_default();
            info!("Получен файл '{file}'");
            let id = f.file.id.clone();
            let r = bot.get_file(id).await?;
            let uri = r.path;
            let token = bot.token();
            let url = format!("https://api.telegram.org/file/bot{token}/{uri}");
            let text = file_router(&url, price_storage.clone())
                .await
                .map_err(|e| tracing::error!("{e:?}"))
                .unwrap_or(String::from("Неизвестный формат файла"));
            info!("Отправляю {text}");
            bot.send_message(msg.chat.id, text).reply_markup(kb).await?;
        }
    } else if let Some(text) = msg.text() {
        info!("Получен текст '{text}' от '{user}'");
        let first_name = msg
            .from
            .clone()
            .map(|u| u.first_name.clone())
            .unwrap_or(String::from("Господин"));
        let name = format!("Уважаемый {first_name}");
        match text {
            STOCK => {
                let text = format!("{name}, введите строку поиска остатков");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Stock).await?;
            }
            PRICES => {
                let text = format!("{name}, введите строку поиска цен");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Price).await?;
            }
            CALCULATOR => {
                let text = format!("{name}, введите через пробел максимальную длину и ширину помещения, а также ширину рулона (все в метрах)");
                info!("Отправляю {text}");
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Calculate).await?;
            }
            _ => {
                let answer =
                    "Не могу обработать сообщение. Введите /help для инструкций по использованию.";
                info!("Отправляю {answer}");
                let kb = make_keyboard();
                bot.send_message(dialogue.chat_id(), answer)
                    .reply_markup(kb)
                    .await?;
                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

fn make_keyboard() -> teloxide::types::KeyboardMarkup {
    let mut keyboard: Vec<Vec<teloxide::types::KeyboardButton>> = vec![];
    let top_buttons = vec![
        teloxide::types::KeyboardButton::new(STOCK),
        teloxide::types::KeyboardButton::new(PRICES),
    ];
    let bottom_buttons = vec![teloxide::types::KeyboardButton::new(CALCULATOR)];
    keyboard.push(top_buttons);
    keyboard.push(bottom_buttons);
    teloxide::types::KeyboardMarkup::new(keyboard).resize_keyboard()
}

#[instrument(
    name = "start handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
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
    let kb = make_keyboard();
    bot.send_message(dialogue.chat_id(), text)
        .reply_markup(kb)
        .await?;
    dialogue.update(State::Selected).await?;
    Ok(())
}

#[instrument(
    name = "help handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
)]
async fn help(bot: Bot, msg: Message) -> Result<()> {
    info!(
        "Отправляю: {answer}",
        answer = Command::descriptions().to_string()
    );
    let kb = make_keyboard();
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .reply_markup(kb)
        .await?;
    Ok(())
}
#[instrument(
    name = "callback handler",
    skip_all,
    fields(
        from = %q.from.clone().full_name(),
        id = %q.from.clone().id.0,
    )
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
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Stock).await?;
            }
            "price" => {
                let text = format!("{name}, введите строку поиска цен");
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Price).await?;
            }
            "calculate" => {
                let text = format!("{name}, введите через пробел максимальную длину и ширину помещения, а также ширину рулона (все в метрах)");
                bot.send_message(dialogue.chat_id(), text)
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                dialogue.update(State::Calculate).await?;
            }
            _ => {
                let kb = make_keyboard();
                bot.send_message(dialogue.chat_id(), "Неизвестный вариант")
                    .reply_markup(kb)
                    .await?;
                dialogue.exit().await?;
            }
        }
    }
    Ok(())
}

fn is_admin(msg: &Message) -> bool {
    let Some(id) = msg.from.clone().map(|u| u.id.0) else {
        return false;
    };
    ADMINS.contains(&id)
}
#[instrument(
    name = "stock handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
)]
async fn stock(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
    stock_storage: Arc<StockStorage>,
) -> Result<()> {
    if let Some(search_string) = msg.text() {
        info!("Получено: {search_string}");
        let mut result = stock_storage.find(search_string.to_string()).await?;
        let mut answer = String::new();
        let kb = make_keyboard();
        if result.is_empty() {
            answer = String::from("Остатки не найдены");
            bot.send_message(msg.chat.id, answer)
                .reply_markup(kb)
                .await?;
        } else {
            if result.len() > 20 {
                result = result.drain(..20).collect::<Vec<_>>();
                answer = String::from("ВЫВЕДУ ТОЛЬКО ПЕРВЫЕ 20 ПОЗИЦИЙ\n\n\n");
            }

            for item in result {
                answer.push_str("\n----------------\n");
                let p = if is_admin(&msg) {
                    format!("{item}")
                } else {
                    item.safe_print()
                };
                answer.push_str(&p);
                answer.push_str("\n----------------\n");
            }
            info!("Отправляю: {answer}");
            bot.send_message(msg.chat.id, answer)
                .reply_markup(kb)
                .await?;
        }
    }
    dialogue.exit().await?;
    Ok(())
}
#[instrument(
    name = "price handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
)]
async fn price(
    bot: Bot,
    dialogue: Dialogue<State, InMemStorage<State>>,
    msg: Message,
    price_storage: Arc<PriceStorage>,
) -> Result<()> {
    if let Some(search_string) = msg.text() {
        let user = msg
            .from
            .clone()
            .map(|u| u.id.to_string())
            .unwrap_or_default();
        info!("Получено: {search_string} от '{user}'");
        let kb = make_keyboard();
        let mut founded = price_storage.find(search_string).await?;
        let mut answer = String::new();
        if founded.is_empty() {
            answer = String::from("Ничего не найдено");
            bot.send_message(msg.chat.id, answer)
                .reply_markup(kb)
                .await?;
            dialogue.exit().await?;
        } else {
            if founded.len() > 20 {
                founded = founded.drain(..20).collect::<Vec<_>>();
                answer = String::from("ВЫВЕДУ ТОЛЬКО ПЕРВЫЕ 20 ПОЗИЦИЙ\n\n\n");
            }
            for item in founded {
                answer.push_str("\n----------------\n");
                let p = if is_admin(&msg) {
                    format!("{item}")
                } else {
                    item.safe_print()
                };
                answer.push_str(&p);
                answer.push_str("\n----------------\n");
            }
            info!("Отправляю: {answer}");
            bot.send_message(msg.chat.id, answer)
                .reply_markup(kb)
                .await?;
            dialogue.exit().await?;
        }
    }
    Ok(())
}
#[instrument(
    name = "calculate handler",
    skip_all,
    fields(
        from = %msg.from.clone().map(|u| u.full_name()).unwrap_or_default(),
        id = %msg.from.clone().map(|u| u.id.0).unwrap_or_default(),
    )
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
        let kb = make_keyboard();
        bot.send_message(msg.chat.id, answer)
            .reply_markup(kb)
            .await?;
    }
    dialogue.exit().await?;
    Ok(())
}
