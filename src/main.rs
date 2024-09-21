use actix_web::{
    get,
    web::{self, ServiceConfig},
};
use shuttle_actix_web::ShuttleActixWeb;
mod currency_service;
mod error;
pub use error::{AppError, Result};
use tracing::info;
mod models;
mod stock_service;
mod storage;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}
// TODO: mail parser
// TODO: price parser

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let mongo_uri = secrets
        .get("MONGODB_URL")
        .expect("MONGODB_URL не найдено в Secrets.toml");
    info!("MONGODB_URL прочитана");
    info!("Инициализирую базу данных");
    let storage = storage::Storage::new(&mongo_uri)
        .await
        .expect("Error initializing mongo DB");
    info!("База данных готова к использованию");
    let state = web::Data::new(models::AppState::new(storage.clone(), storage.clone()));
    info!("Инициализирую службу валют");
    let cs = currency_service::CurrencyService::new(storage.clone());
    info!("Служба валют готова к использованию, запускаю");
    tokio::spawn(async move { cs.run().await });
    info!("Инициализирую службу остатков");
    let ss = stock_service::StockService::new(secrets, storage.clone())?;
    info!("Служба остатков готова к использованию, запускаю");
    tokio::spawn(async move { ss.run().await });
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world)
            .service(currency_service::currencies)
            .service(currency_service::monthly_currencies)
            .service(stock_service::stock)
            .app_data(state);
    };

    Ok(config.into())
}
