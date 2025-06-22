mod currency_service;
mod tg_bot;
mod error;
use std::sync::Arc;

pub use error::{AppError, Result};
use event_processor::Eventer;
use storage::{CurrencyStorage, PriceStorage, StockStorage};
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
    pub async fn run(&self, addr: std::net::SocketAddr) {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .expect("Failed to migrate");
        let ms_token = self.secrets.get("MS_TOKEN").expect("MS_TOKEN not set");
        let ms_client = Arc::new(
            rust_moysklad::MoySkladApiClient::new(ms_token)
                .expect("Не получилось создать клиент Мой Склад"),
        );
        let safira_ck = self.secrets.get("SAFIRA_CK").expect("SAFIRA_CK not set");
        let safira_cs = self.secrets.get("SAFIRA_CS").expect("SAFIRA_CS not set");
        let safira_host = self
            .secrets
            .get("SAFIRA_HOST")
            .expect("SAFIRA_HOST not set");
        let safira_client = Arc::new(
            rust_woocommerce::ApiClient::init(safira_host, safira_ck, safira_cs)
                .expect("safira_woo_client init error"),
        );
        let lc_ck = self.secrets.get("LC_CK").expect("LC_CK not set");
        let lc_cs = self.secrets.get("LC_CS").expect("LC_CS not set");
        let lc_host = self.secrets.get("LC_HOST").expect("LC_HOST not set");
        let lc_client = Arc::new(
            rust_woocommerce::ApiClient::init(lc_host, lc_ck, lc_cs)
                .expect("lc_woo_client init error"),
        );
        let currency_storage = Arc::new(
            CurrencyStorage::new(self.pool.clone())
                .await
                .expect("Failed to init currency storage"),
        );
        let price_storage = Arc::new(PriceStorage::new(self.pool.clone()));
        let stock_storage = Arc::new(StockStorage::new(self.pool.clone()));
        let events_storage = Arc::new(storage::EventsStorage::new(self.pool.clone()));
        tokio::spawn(currency_service::run(currency_storage.clone()));
        let syncer = synchronizer::Synchronizer::new(
            ms_client.clone(),
            safira_client.clone(),
            lc_client.clone(),
            stock_storage.clone(),
        );
        tokio::spawn(syncer.run());
        let stocker = stock_service::Stocker::new(self.secrets.clone(), stock_storage.clone());
        tokio::spawn(stocker.run());
        let eventer = Eventer::new(
            ms_client.clone(),
            safira_client.clone(),
            events_storage.clone(),
            stock_storage.clone(),
        );
        tokio::spawn(eventer.run());
        let state = models::AppState::new(
            currency_storage.clone(),
            stock_storage.clone(),
            events_storage.clone(),
            price_storage.clone(),
        );
        let router = routes::init(state);
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind address to listener");
        let tg_token = self.secrets.get("WINSTON_TOKEN").expect("WINSTON_TOKEN not set");
        let bot = tg_bot::TGBot::new(&tg_token);
        tokio::spawn(async move { axum::serve(listener, router).await });
        bot.run(price_storage, stock_storage, currency_storage).await;
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Service {
    async fn bind(
        mut self,
        addr: std::net::SocketAddr,
    ) -> std::result::Result<(), shuttle_runtime::Error> {
        self.run(addr).await;
        Ok(())
    }
}
