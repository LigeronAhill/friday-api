use std::sync::Arc;

use actix_web::{
    get,
    web::{self, ServiceConfig},
};
use shuttle_actix_web::ShuttleActixWeb;
mod currency_service;
mod error;
pub use error::{AppError, Result};
mod models;
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
    let mongo_uri = secrets.get("MONGODB_URL").expect("MONGODB_URL not found");
    let storage = storage::Storage::new(&mongo_uri)
        .await
        .expect("Error initializing mongo DB");
    let cs = currency_service::CurrencyService::new(Arc::new(storage.clone()));
    cs.update_currencies().await?;
    let _currencies = cs.get_currencies().await?;
    let state = web::Data::new(models::AppState::new(storage));
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(hello_world).app_data(state);
    };

    Ok(config.into())
}
