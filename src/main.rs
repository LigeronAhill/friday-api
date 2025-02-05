mod currency_service;
mod error;
use std::sync::Arc;

pub use error::{AppError, Result};
use storage::{CurrencyStorage, StockStorage};
use tracing::info;
mod event_processor;
mod models;
mod price_service;
mod routes;
mod stock_service;
mod storage;
mod synchronizer;
pub mod utils;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PG_PASS}@localhost:5432/friday-api"
    )]
    pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    info!("Инициализирую базу данных валют");
    let currency_storage = Arc::new(CurrencyStorage::new(pool.clone()).await?);
    info!("База данных валют готова к использованию");
    info!("Запускаю службу обновления курсов валют");
    tokio::spawn(currency_service::run(currency_storage.clone()));
    info!("Инициализирую базу данных остатков");
    let stock_storage = Arc::new(StockStorage::new(pool.clone()));
    info!("Запускаю службу обновления остатков");
    tokio::spawn(stock_service::run(secrets.clone(), stock_storage.clone()));
    let state = models::AppState::new(currency_storage.clone(), stock_storage.clone());
    let events_storage = Arc::new(storage::EventsStorage::new(pool.clone()));
    let api_clients = models::ApiClients::new(secrets.clone())?;
    tokio::spawn(event_processor::run(
        api_clients.clone(),
        events_storage.clone(),
    ));
    tokio::spawn(synchronizer::run(api_clients));
    // TODO: telegram bot
    // TODO: price service (input from telegram and API, currencies from MS)
    // TODO: update prices in MS
    let router = routes::init(state, events_storage.clone());
    Ok(router.into())
}
