use crate::{
    models::Currency,
    Result,
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

// url для получения курсов валют
const CBR_URI: &str = "https://www.cbr-xml-daily.ru/daily_json.js";

#[derive(Clone)]
/// Служба для работы с курсами валют
pub struct CurrencyService {
    /// клиент для отправки запросов
    client: reqwest::Client,
    /// база данных, имплементирующая необходимые методы
    storage: crate::storage::CurrencyStorage,
}
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

impl CurrencyService {
    /// создание нового экземпляра слуюбы валют
    pub fn new(storage: crate::storage::CurrencyStorage) -> Self {
        let client = reqwest::Client::builder().gzip(true).build().unwrap();
        Self { client, storage }
    }
    /// обновление курсов валют в базе данных
    pub async fn update_currencies(&self) -> Result<()> {
        let response: serde_json::Value = self
            .client
            .get(CBR_URI)
            .send()
            .await?
            .json()
            .await?;
        let valutes: HashMap<String, CurrencyInput> = response.get("Valute").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
        let currencies: Vec<Currency> = valutes.into_values().map(Currency::from).collect();
        let updated = self.storage.update(currencies).await?;
        info!("Обновлено {updated} курсов валют");
        Ok(())
    }
    pub async fn get(&self) -> Result<Vec<Currency>> {
        let res = self.storage.get_all().await?;
        Ok(res)
    }
    pub async fn find(&self, char_code: &str) -> Result<Option<Currency>> {
        self.storage.get_by_char_code(char_code).await
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