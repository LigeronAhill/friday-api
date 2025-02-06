use crate::models::AppState;
use axum::routing::get;
use axum::Router;
mod currency;
mod stock;

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/currencies", get(currency::currencies))
        .route("/currencies/{char_code}", get(currency::currency))
        .route("/stock", get(stock::stock))
        .with_state(state)
}
