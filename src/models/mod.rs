mod currency;
mod price;
mod stock;
mod webhooks;
use std::sync::Arc;
pub use webhooks::*;

pub use currency::*;
pub use stock::*;

use crate::storage::{CurrencyStorage, StockStorage};

/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_storage: Arc<CurrencyStorage>,
    pub stock_storage: Arc<StockStorage>,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(currency_storage: Arc<CurrencyStorage>, stock_storage: Arc<StockStorage>) -> Self {
        Self {
            currency_storage,
            stock_storage,
        }
    }
}
#[derive(Clone)]
pub struct ApiClients {
    pub ms_client: Arc<rust_moysklad::MoySkladApiClient>,
    pub safira_woo_client: Arc<rust_woocommerce::ApiClient>,
    pub lc_woo_client: Arc<rust_woocommerce::ApiClient>,
}
impl ApiClients {
    pub fn new(secrets: shuttle_runtime::SecretStore) -> crate::Result<Self> {
        let ms_token = secrets.get("MS_TOKEN").expect("MS_TOKEN not set");
        let ms_client = Arc::new(
            rust_moysklad::MoySkladApiClient::new(ms_token).map_err(|e| {
                let error = format!("{e:?}");
                tracing::error!(error);
                crate::error::AppError::Custom(error)
            })?,
        );
        let safira_ck = secrets.get("SAFIRA_CK").expect("SAFIRA_CK not set");
        let safira_cs = secrets.get("SAFIRA_CS").expect("SAFIRA_CS not set");
        let safira_host = secrets.get("SAFIRA_HOST").expect("SAFIRA_HOST not set");
        let safira_woo_client = Arc::new(
            rust_woocommerce::ApiClient::init(safira_host, safira_ck, safira_cs)
                .expect("safira_woo_client init error"),
        );
        let lc_ck = secrets.get("LC_CK").expect("LC_CK not set");
        let lc_cs = secrets.get("LC_CS").expect("LC_CS not set");
        let lc_host = secrets.get("LC_HOST").expect("LC_HOST not set");
        let lc_woo_client = Arc::new(
            rust_woocommerce::ApiClient::init(lc_host, lc_ck, lc_cs)
                .expect("lc_woo_client init error"),
        );
        Ok(Self {
            ms_client,
            safira_woo_client,
            lc_woo_client,
        })
    }
}
