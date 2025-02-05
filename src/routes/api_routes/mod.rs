use crate::models::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/currencies", get(currencies))
        .route("/currencies/{char_code}", get(currency))
        .route("/stock", get(stock))
        .with_state(state)
}

async fn currencies(State(state): State<AppState>) -> impl IntoResponse {
    match state.currency_storage.get_all().await {
        Ok(currencies) => (StatusCode::OK, Json(currencies)).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
    }
}
async fn currency(
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

#[derive(Deserialize)]
struct StockQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    search: Option<String>,
}
async fn stock(
    State(state): State<AppState>,
    Query(query): Query<StockQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    if let Some(search) = query.search {
        match state.stock_storage.find(search).await {
            Ok(r) => (StatusCode::OK, Json(r)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
        }
    } else {
        match state.stock_storage.get(limit, offset).await {
            Ok(stock) => (StatusCode::OK, Json(stock)).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, Json(err.to_string())).into_response(),
        }
    }
}
