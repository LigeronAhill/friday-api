mod currency;
mod price;
mod stock;
pub use currency::*;
pub use price::*;
pub use stock::*;

use crate::currency_service::CurrencyService;
use crate::price_service::PriceService;
use crate::stock_service::StockService;

/// Общие данные для обработчиков
#[derive(Clone)]
pub struct AppState {
    pub currency_service: CurrencyService,
    pub stock_service: StockService,
    pub price_service: PriceService,
}
impl AppState {
    /// Создать новый экземпляр общих данных
    pub fn new(currency_service: CurrencyService, stock_service: StockService, price_service: PriceService) -> Self {
        Self {
            currency_service,
            stock_service,
            price_service,
        }
    }
}