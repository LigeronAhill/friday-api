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

pub struct Service {
    pool: sqlx::PgPool,
    secrets: shuttle_runtime::SecretStore,
}
impl Service {
    pub fn new(pool: sqlx::PgPool, secrets: shuttle_runtime::SecretStore) -> Self {
        Self { pool, secrets }
    }
    pub async fn run(&self, addr: std::net::SocketAddr) -> anyhow::Result<()> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .expect("Failed to run migrations");
        info!("Инициализирую базу данных валют");
        let currency_storage = Arc::new(CurrencyStorage::new(self.pool.clone()).await?);
        info!("База данных валют готова к использованию");
        info!("Запускаю службу обновления курсов валют");
        tokio::spawn(currency_service::run(currency_storage.clone()));
        info!("Инициализирую базу данных остатков");
        let stock_storage = Arc::new(StockStorage::new(self.pool.clone()));
        info!("Запускаю службу обновления остатков");
        tokio::spawn(stock_service::run(
            self.secrets.clone(),
            stock_storage.clone(),
        ));
        let state = models::AppState::new(currency_storage.clone(), stock_storage.clone());
        let events_storage = Arc::new(storage::EventsStorage::new(self.pool.clone()));
        let api_clients = models::ApiClients::new(self.secrets.clone())?;
        tokio::spawn(event_processor::run(
            api_clients.clone(),
            events_storage.clone(),
        ));
        tokio::spawn(synchronizer::run(api_clients));
        // TODO: telegram bot
        // TODO: price service (input from telegram and API, currencies from MS)
        // TODO: update prices in MS
        let router = routes::init(state, events_storage.clone());
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;
        Ok(())
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Service {
    async fn bind(
        mut self,
        addr: std::net::SocketAddr,
    ) -> std::result::Result<(), shuttle_runtime::Error> {
        self.run(addr).await?;
        Ok(())
    }
}
