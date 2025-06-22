use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::models::{AppState, MsEvent, WebhookRequest};

pub fn init(state: AppState) -> Router {
    Router::new()
        .route("/ms", post(ms_webhook))
        .with_state(state)
}

async fn ms_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookRequest>,
) -> StatusCode {
    let events = payload
        .events
        .into_iter()
        .flat_map(|e| {
            MsEvent::try_from(e)
                .map_err(|e| tracing::error!("{e:?}"))
                .ok()
        })
        .collect::<Vec<_>>();
    if !events.is_empty() {
        match state.events_storage.save(events).await {
            Ok(_) => tracing::info!("События из Мой Склад добавлены в очередь"),
            Err(e) => tracing::error!("События из Мой Склад не добавлены в очередь: {e:?}"),
        }
    } else {
        tracing::error!("Ошибка при десериализации запроса с событиями");
    }
    StatusCode::OK
}
