#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:postgres@localhost:5432/friday_api"
    )]
    pool: sqlx::PgPool,
) -> Result<friday_api::Service, shuttle_runtime::Error> {
    let service = friday_api::Service::new(pool, secrets);
    Ok(service)
}
