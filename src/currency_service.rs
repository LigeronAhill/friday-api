use crate::{models::Currency, storage::CurrencyStorage, utils::pause};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

// url для получения курсов валют
const CBR_URI: &str = "https://www.cbr-xml-daily.ru/daily_json.js";

#[derive(Deserialize)]
struct CurrencyInput {
    #[serde(rename = "CharCode")]
    char_code: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: f64,
}
impl From<CurrencyInput> for Currency {
    fn from(input: CurrencyInput) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: input.name,
            char_code: input.char_code,
            rate: input.value,
            updated: chrono::Utc::now(),
        }
    }
}

pub async fn run(storage: Arc<CurrencyStorage>) {
    let (tx, rx) = unbounded_channel::<Vec<Currency>>();
    tokio::spawn(async move {
        generator(tx).await;
    });
    saver(rx, storage).await;
}
async fn generator(tx: UnboundedSender<Vec<Currency>>) {
    let client = reqwest::Client::builder().gzip(true).build().unwrap();
    loop {
        tracing::info!("Начинаю запрос курсов валют");
        match client.get(CBR_URI).send().await {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(v) => {
                    let valutes: HashMap<String, CurrencyInput> = v
                        .get("Valute")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let currencies: Vec<Currency> =
                        valutes.into_values().map(Currency::from).collect();
                    let quantity = currencies.len();
                    if !currencies.is_empty() {
                        if tx.send(currencies).is_err() {
                            tracing::error!("Не удалось отправить данные о курсах валют в канал");
                        } else {
                            tracing::info!("Получено {quantity} курсов валют, пауза на 4 часа");
                            pause(4).await;
                        }
                    } else {
                        tracing::error!("Получен пустой ответ на запрос курсов валют, попробую еще раз через час");
                        pause(1).await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Ошибка чтения ответа на запрос курсов валют: {e:?}\n Пауза на 1 час"
                    );
                    pause(1).await;
                }
            },
            Err(e) => {
                tracing::error!("Ошибка получения курсов валют: {e:?}\n Пауза на 1 час");
                pause(1).await
            }
        }
    }
}

async fn saver(mut rx: UnboundedReceiver<Vec<Currency>>, storage: Arc<CurrencyStorage>) {
    while let Some(result) = rx.recv().await {
        match storage.update(result).await {
            Ok(updated) => {
                tracing::info!("Обновлено {updated} курсов валют");
            }
            Err(e) => {
                tracing::error!("Ошибка сохранения курсов валют: {e:?}");
            }
        }
    }
}
