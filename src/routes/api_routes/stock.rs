use crate::models::AppState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct StockQuery {
    limit: Option<i32>,
    offset: Option<i32>,
    search: Option<String>,
}
pub(super) async fn stock(
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
