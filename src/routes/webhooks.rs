use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

use crate::{
    models::{MsEvent, WebhookRequest},
    storage::EventsStorage,
};

pub fn init(storage: Arc<EventsStorage>) -> Router {
    Router::new()
        .route("/ms", post(ms_webhook))
        .with_state(storage)
}

async fn ms_webhook(
    State(storage): State<Arc<EventsStorage>>,
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
        .filter(|e| !(e.fields.len() == 1 && e.fields.first().is_some_and(|f| f == "Наличие")))
        .collect::<Vec<_>>();
    if !events.is_empty() {
        match storage.save(events).await {
            Ok(_) => tracing::info!("События из Мой Склад добавлены в очередь"),
            Err(e) => tracing::error!("События из Мой Склад не добавлены в очередь: {e:?}"),
        }
    } else {
        tracing::error!("Ошибка при десериализации запроса с событиями");
    }
    StatusCode::OK
}
