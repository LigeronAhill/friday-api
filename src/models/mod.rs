mod currency;
mod price;
mod stock;
pub use currency::*;
pub use price::*;
use std::sync::Arc;
pub use stock::*;

use crate::{currency_service::CurrencyService, storage::Storage};

/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_service: CurrencyService,
    pub storage: Arc<Storage>,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(storage: Arc<Storage>, currency_service: CurrencyService) -> Self {
        Self {
            storage,
            currency_service,
        }
    }
}
