use crate::models::AppState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use bytes::Bytes;
use http::{HeaderMap, Request, Response, StatusCode};
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::Any;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;

mod api_routes;

pub fn init(state: AppState) -> Router {
    let cors = tower_http::cors::CorsLayer::new().allow_methods(Any).allow_origin(Any);
    let trace = TraceLayer::new_for_http()
        .make_span_with(|_request: &Request<axum::body::Body>| {
            tracing::debug_span!("http-request")
        })
        .on_request(|request: &Request<axum::body::Body>, _span: &Span| {
            tracing::debug!("started {} {}", request.method(), request.uri().path())
        })
        .on_response(|_response: &Response<axum::body::Body>, latency: Duration, _span: &Span| {
            tracing::debug!("response generated in {:?}", latency)
        })
        .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
            tracing::debug!("sending {} bytes", chunk.len())
        })
        .on_eos(|_trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
            tracing::debug!("stream closed after {:?}", stream_duration)
        })
        .on_failure(|error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
            tracing::error!("something went wrong: {error:?} latency: {latency:?}")
        });
    Router::new()
        .route("/health", get(health_check))
        .nest("/api/v1", api_routes::init(state))
        .layer(trace)
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(cors)
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "My health is fine").into_response()
}