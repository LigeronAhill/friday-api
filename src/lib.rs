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
        sqlx::migrate!().run(&self.pool).await?;
        info!("Инициализирую базу данных валют");
        let currency_storage = Arc::new(CurrencyStorage::new(self.pool.clone()).await?);
        info!("База данных валют готова к использованию");
        info!("Запускаю службу обновления курсов валют");
        let currency_storage_instance = currency_storage.clone();
        tokio::spawn(async move { currency_service::run(currency_storage_instance).await });
        info!("Инициализирую базу данных остатков");
        let stock_storage = Arc::new(StockStorage::new(self.pool.clone()));
        info!("Запускаю службу обновления остатков");
        let stock_storage_instance = Arc::new(StockStorage::new(self.pool.clone()));
        let secrets_instance = self.secrets.clone();
        tokio::spawn(async move {
            stock_service::run(secrets_instance, stock_storage_instance).await;
        });
        let state = models::AppState::new(currency_storage.clone(), stock_storage.clone());
        let events_storage = Arc::new(storage::EventsStorage::new(self.pool.clone()));
        let api_clients = models::ApiClients::new(self.secrets.clone())?;
        let api_clients_instance = api_clients.clone();
        let events_storage_instance = events_storage.clone();
        tokio::spawn(async move {
            event_processor::run(api_clients_instance, events_storage_instance).await;
        });
        tokio::spawn(async move {
            synchronizer::run(api_clients).await;
        });
        let router = routes::init(state, events_storage);
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
