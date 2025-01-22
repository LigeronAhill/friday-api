mod currency_service;
mod error;
pub use error::{AppError, Result};
use tracing::info;
mod models;
mod stock_service;
mod storage;
mod price_service;
mod routes;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PG_PASS}@localhost:5432/friday-api"
    )] pool: sqlx::PgPool,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");
    info!("Инициализирую базу данных валют");
    let currency_storage = storage::CurrencyStorage::new(pool.clone());
    info!("База данных валют готова к использованию");
    info!("Инициализирую службу валют");
    let cs = currency_service::CurrencyService::new(currency_storage);
    let cs_to_run = cs.clone();
    info!("Служба валют готова к использованию, запускаю");
    tokio::spawn(async move { cs_to_run.run().await });
    info!("Инициализирую службу остатков");
    let stock_storage = storage::StockStorage::new(pool.clone());
    let ss = stock_service::StockService::new(secrets, stock_storage)?;
    info!("Служба остатков готова к использованию, запускаю");
    let ss_to_run = ss.clone();
    tokio::spawn(async move { ss_to_run.run().await });
    let price_storage = storage::PriceStorage::new(pool.clone());
    let ps = price_service::PriceService::new(price_storage);
    let state = models::AppState::new(cs, ss, ps);
    let router = routes::init(state);
    Ok(router.into())
}