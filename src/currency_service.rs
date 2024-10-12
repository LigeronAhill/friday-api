use std::sync::Arc;

use tracing::info;

use crate::{models::CurrenciesFromCbr, storage::Storage, Result};

// url для получения курсов валют
const CBR_URI: &str = "https://www.cbr-xml-daily.ru/daily_json.js";

/// Служба для работы с курсами валют
pub struct CurrencyService {
    /// клиент для отправки запросов
    client: reqwest::Client,
    /// база данных, имплементирующая необходимые методы
    storage: Arc<Storage>,
}
impl CurrencyService {
    /// создание нового экземпляра слуюбы валют
    pub fn new(storage: Arc<Storage>) -> Self {
        let client = reqwest::Client::builder().gzip(true).build().unwrap();
        Self { client, storage }
    }
    /// обновление курсов валют в базе данных
    pub async fn update_currencies(&self) -> Result<()> {
        let response: CurrenciesFromCbr = self.client.get(CBR_URI).send().await?.json().await?;
        self.storage.update_currencies(response).await?;
        Ok(())
    }
    pub async fn run(&self) {
        loop {
            info!("Обновляю курсы валют");
            match self.update_currencies().await {
                Ok(_) => {
                    let now = chrono::Local::now();
                    let tomorrow = now
                        .checked_add_days(chrono::Days::new(1))
                        .unwrap_or_default();
                    info!(
                        "Курсы валют обновлены {}. Пауза на 24 часа до {}",
                        now.format("%d.%m.%Y в %T"),
                        tomorrow.format("%d.%m.%Y в %T"),
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
                }
                Err(e) => {
                    info!("Возникла ошибка приобновлении курсов валют: {e:?}");
                    info!("Попробую еще раз через 10 минут");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10 * 60)).await;
                }
            }
        }
    }
}
