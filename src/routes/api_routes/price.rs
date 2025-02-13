use super::stock::StockQuery;
use crate::models::{AppState, PriceDTO, PriceItem};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;

pub(super) async fn get_price(
    State(state): State<AppState>,
    Query(query): Query<StockQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);
    let result = if let Some(search_string) = query.search {
        state
            .price_storage
            .find(&search_string)
            .await
            .map(|r| r.iter().map(PriceDTO::from).collect::<Vec<_>>())
    } else {
        state
            .price_storage
            .get(limit, offset)
            .await
            .map(|r| r.iter().map(PriceDTO::from).collect::<Vec<_>>())
    };
    match result {
        Ok(r) => (StatusCode::OK, Json(r)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response(),
    }
}
pub(super) async fn prices(
    State(state): State<AppState>,
    Json(payload): Json<Vec<PriceItem>>,
) -> StatusCode {
    match state.price_storage.update(payload).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
pub(super) async fn prices_by_supplier(
    State(state): State<AppState>,
    Path(supplier): Path<String>,
) -> impl IntoResponse {
    match state.price_storage.get_by_supplier(&supplier).await {
        Ok(r) => (
            StatusCode::OK,
            Json(r.iter().map(PriceDTO::from).collect::<Vec<_>>()),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
