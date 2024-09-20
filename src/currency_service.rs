use std::sync::Arc;

use actix_web::{get, web, HttpResponse};

use crate::{
    models::{AppState, CurrenciesFromCbr, Currency},
    Result,
};

const CBR_URI: &str = "https://www.cbr-xml-daily.ru/daily_json.js";

#[async_trait::async_trait]
pub trait CurrencyStorage: Send + Sync + 'static {
    async fn update_currencies(&self, input: CurrenciesFromCbr) -> Result<()>;
    async fn get_currencies(&self) -> Result<Vec<crate::models::Currency>>;
}
pub struct CurrencyService {
    client: reqwest::Client,
    storage: Arc<dyn CurrencyStorage>,
}
impl CurrencyService {
    pub fn new(storage: Arc<dyn CurrencyStorage>) -> Self {
        let client = reqwest::Client::builder().gzip(true).build().unwrap();
        Self { client, storage }
    }
    pub async fn update_currencies(&self) -> Result<()> {
        let response: CurrenciesFromCbr = self.client.get(CBR_URI).send().await?.json().await?;
        self.storage.update_currencies(response).await?;
        Ok(())
    }
    pub async fn get_currencies(&self) -> Result<Vec<Currency>> {
        let res = self.storage.get_currencies().await?;
        Ok(res)
    }
}
#[get("/api/currencies")]
pub async fn currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.get_currencies().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
