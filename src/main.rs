#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set!");
    tracing::debug!("DATABASE_URL={db_url}");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    let service = friday_api::Service::new(pool);
    service.run().await;
    Ok(())
}
