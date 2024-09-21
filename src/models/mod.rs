mod currency;
use std::sync::Arc;

pub use currency::*;

use crate::{currency_service::CurrencyStorage, stock_service::StockStorage};
/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_storage: Arc<dyn CurrencyStorage>,
    pub stock_storage: Arc<dyn StockStorage>,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(
        currency_storage: Arc<dyn CurrencyStorage>,
        stock_storage: Arc<dyn StockStorage>,
    ) -> Self {
        Self {
            currency_storage,
            stock_storage,
        }
    }
}
