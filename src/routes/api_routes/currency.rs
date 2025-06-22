use crate::models::AppState;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;

pub async fn currencies(State(state): State<AppState>) -> impl IntoResponse {
    match state.currency_storage.get_all().await {
        Ok(currencies) => (StatusCode::OK, Json(currencies)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}
pub async fn currency(
    State(state): State<AppState>,
    Path(char_code): Path<String>,
) -> impl IntoResponse {
    match state.currency_storage.get_by_char_code(&char_code).await {
        Ok(result) => match result {
            Some(currency) => (StatusCode::OK, Json(currency)).into_response(),
            None => (StatusCode::NOT_FOUND, Json("currency not found")).into_response(),
        },
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}
