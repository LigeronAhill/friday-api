mod error;
use std::sync::Arc;

pub use error::{AppError, Result};
use storage::StockStorage;
mod models;
mod stock_service;
mod storage;
mod synchronizer;
pub mod utils;

pub struct LocalService {
    pool: sqlx::PgPool,
}
impl LocalService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    pub async fn run(&self) {
        if let Err(e) = sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
            .execute(&self.pool)
            .await
        {
            tracing::error!("{e:?}");
        }
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .expect("Failed to migrate");
        let ms_token = std::env::var("MS_TOKEN").expect("MS_TOKEN not set");
        let ms_client = Arc::new(
            rust_moysklad::MoySkladApiClient::new(ms_token)
                .expect("Не получилось создать клиент Мой Склад"),
        );
        let safira_ck = std::env::var("SAFIRA_CK").expect("SAFIRA_CK not set");
        let safira_cs = std::env::var("SAFIRA_CS").expect("SAFIRA_CS not set");
        let safira_host = std::env::var("SAFIRA_HOST").expect("SAFIRA_HOST not set");
        let safira_client = Arc::new(
            rust_woocommerce::ApiClient::init(safira_host, safira_ck, safira_cs)
                .expect("safira_woo_client init error"),
        );
        let stock_storage = Arc::new(StockStorage::new(self.pool.clone()));
        let syncer = synchronizer::Synchronizer::new(
            ms_client.clone(),
            safira_client.clone(),
            stock_storage.clone(),
        );
        tokio::spawn(syncer.run());
        let stocker = stock_service::Stocker::new(stock_storage.clone());
        if let Err(e) = stocker.run().await {
            tracing::error!("{e:?}")
        }
    }
}
