mod currency;
mod price;
mod stock;
mod webhooks;
use std::sync::Arc;
pub use webhooks::*;

pub use currency::*;
pub use stock::*;

use crate::storage::{CurrencyStorage, EventsStorage, StockStorage};

/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_storage: Arc<CurrencyStorage>,
    pub stock_storage: Arc<StockStorage>,
    pub events_storage: Arc<EventsStorage>,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(
        currency_storage: Arc<CurrencyStorage>,
        stock_storage: Arc<StockStorage>,
        events_storage: Arc<EventsStorage>,
    ) -> Self {
        Self {
            currency_storage,
            stock_storage,
            events_storage,
        }
    }
}
