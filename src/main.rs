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
mod price_service;

#[get("/")]
async fn hello_world() -> &'static str {
    "Hello World!"
}
// TODO: auth -> JWT/Clerk/Cookie ???

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.PG_PASS}@localhost:5432/friday-api"
    )] pool: sqlx::PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations;").execute(&pool).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS prices;").execute(&pool).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS stock;").execute(&pool).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS currencies;").execute(&pool).await.unwrap();
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
    let state = web::Data::new(models::AppState::new(cs, ss, ps));
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