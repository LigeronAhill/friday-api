use crate::{models::Currency, Result};

#[derive(Clone)]
pub struct CurrencyStorage {
    pool: sqlx::PgPool,
}

impl CurrencyStorage {
    pub async fn new(pool: sqlx::PgPool) -> Result<Self> {
        let query = "INSERT INTO currencies(name, char_code, rate) VALUES ('Российский рубль', 'RUB', 1.0) ON CONFLICT (char_code) DO NOTHING;";
        sqlx::query(query).execute(&pool).await?;
        Ok(Self { pool })
    }
    pub async fn get_all(&self) -> Result<Vec<Currency>> {
        let query = "SELECT * FROM currencies ORDER BY name";
        let results = sqlx::query_as::<_, Currency>(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
    }
    pub async fn get_by_char_code(&self, code: &str) -> Result<Option<Currency>> {
        let query = "SELECT * FROM currencies WHERE char_code = $1";
        let results = sqlx::query_as::<_, Currency>(query)
            .bind(code.to_uppercase())
            .fetch_optional(&self.pool)
            .await?;
        Ok(results)
    }
    pub async fn update(&self, input: Vec<Currency>) -> Result<u64> {
        let query_string = "INSERT INTO currencies(name, char_code, rate) ";
        let mut query_builder = sqlx::QueryBuilder::new(query_string);
        query_builder.push_values(input, |mut b, currency| {
            b.push_bind(currency.name)
                .push_bind(currency.char_code)
                .push_bind(currency.rate);
        });
        query_builder
            .push(" ON CONFLICT(char_code) DO UPDATE SET rate = EXCLUDED.rate, updated = now();");
        let query = query_builder.build();
        let results = query.execute(&self.pool).await?;
        let rows_affected = results.rows_affected();
        Ok(rows_affected)
    }
}
