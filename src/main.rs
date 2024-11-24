use actix_web::{
    get,
    web::{self, ServiceConfig},
};
use shuttle_actix_web::ShuttleActixWeb;
mod currency_service;
mod error;
pub use error::{AppError, Result};
use tracing::info;
mod handlers;
mod models;
mod stock_service;
mod storage;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}
// TODO: auth -> JWT/Clerk/Cookie ???

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
        .expect("Ошибка при инициализации базы данных");
    info!("База данных готова к использованию");
    info!("Инициализирую службу валют");
    let cs = currency_service::CurrencyService::new(storage.clone());
    let cs_to_run = cs.clone();
    info!("Служба валют готова к использованию, запускаю");
    tokio::spawn(async move { cs_to_run.run().await });
    info!("Инициализирую службу остатков");
    let ss = stock_service::StockService::new(secrets, storage.clone())?;
    info!("Служба остатков готова к использованию, запускаю");
    tokio::spawn(async move { ss.run().await });
    let state = web::Data::new(models::AppState::new(storage.clone(), cs));
    let config = move |cfg: &mut ServiceConfig| {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        cfg.service(hello_world)
            .service(
                web::scope("/api/v1")
                    .wrap(cors)
                    .service(handlers::currencies)
                    .service(handlers::currency)
                    .service(handlers::stock)
                    .service(handlers::get_price)
                    // .service(handlers::update_prices)
                    .service(
                        web::resource("/pricestest")
                            .route(web::get().to(index))
                            .route(web::post().to(handlers::update_prices)),
                    ),
            )
            .app_data(state);
    };

    Ok(config.into())
}
async fn index() -> actix_web::HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    actix_web::HttpResponse::Ok().body(html)
}
