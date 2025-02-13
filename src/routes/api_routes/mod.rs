use crate::models::AppState;
use axum::routing::get;
use axum::Router;
mod currency;
mod price;
mod stock;

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/currencies", get(currency::currencies))
        .route("/currencies/{char_code}", get(currency::currency))
        .route("/stock", get(stock::stock))
        .route("/prices", get(price::get_price).post(price::prices))
        .route("/prices/{supplier}", get(price::prices_by_supplier))
        .with_state(state)
}
