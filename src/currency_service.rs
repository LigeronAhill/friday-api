use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use tracing::info;

use crate::{
    models::{AppState, CurrenciesFromCbr},
    Result,
};

// url для получения курсов валют
const CBR_URI: &str = "https://www.cbr-xml-daily.ru/daily_json.js";

#[async_trait::async_trait]
pub trait CurrencyStorage: Send + Sync {
    async fn update_currencies(&self, input: CurrenciesFromCbr) -> Result<()>;
    async fn get_latest_currency_rates(&self) -> Result<Vec<crate::models::Currency>>;
    async fn get_monthly_currency_rates(&self) -> crate::Result<Vec<crate::models::Currency>>;
}
/// Служба для работы с курсами валют
pub struct CurrencyService {
    /// клиент для отправки запросов
    client: reqwest::Client,
    /// база данных, имплементирующая необходимые методы
    storage: Arc<dyn CurrencyStorage>,
}
impl CurrencyService {
    /// создание нового экземпляра слуюбы валют
    pub fn new(storage: Arc<dyn CurrencyStorage>) -> Self {
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
#[get("/api/currencies")]
pub async fn currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.currency_storage.get_latest_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
#[get("/api/currencies/month")]
pub async fn monthly_currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.currency_storage.get_monthly_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
