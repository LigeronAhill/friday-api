// #[shuttle_runtime::main]
// async fn main(
//     #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
//     #[shuttle_shared_db::Postgres(
//         local_uri = "postgres://postgres:postgres@localhost:5432/friday_api"
//     )]
//     pool: sqlx::PgPool,
// ) -> Result<friday_api::Service, shuttle_runtime::Error> {
//     let service = friday_api::Service::new(pool, secrets);
//     Ok(service)
// }
//
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set!");
    tracing::debug!("DATABASE_URL={db_url}");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    let service = friday_api::LocalService::new(pool);
    service.run().await;
    Ok(())
}
