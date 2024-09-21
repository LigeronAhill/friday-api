mod currency;
use std::sync::Arc;

pub use currency::*;

use crate::currency_service::CurrencyStorage;
/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_storage: Arc<dyn CurrencyStorage>,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(currency_storage: Arc<dyn CurrencyStorage>) -> Self {
        Self { currency_storage }
    }
}
