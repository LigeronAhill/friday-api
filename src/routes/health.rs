use axum::response::IntoResponse;
use http::StatusCode;

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "My health is fine, thanks").into_response()
}
