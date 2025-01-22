use crate::models::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/currencies", get(currencies))
        .with_state(state)
}

async fn currencies(State(state): State<AppState>) -> impl IntoResponse {
    match state.currency_service.get().await {
        Ok(currencies) => (StatusCode::OK, Json(currencies)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}
