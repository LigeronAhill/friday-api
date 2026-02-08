use crate::{models::Stock, Result};
use std::collections::HashSet;

#[derive(Clone)]
pub struct StockStorage {
    pool: sqlx::PgPool,
}

impl StockStorage {
    pub fn new(pool: sqlx::PgPool) -> StockStorage {
        StockStorage { pool }
    }
    pub async fn update(&self, input: &[Stock]) -> Result<(u64, u64)> {
        let mut suppliers = HashSet::new();
        for supplier in input.iter().map(|s| s.supplier.clone()) {
            suppliers.insert(supplier);
        }
        let mut tx = self.pool.begin().await?;
        let mut deleted = 0;
        for supplier in suppliers {
            let query = "DELETE FROM stock WHERE supplier=$1";
            let qr = sqlx::query(query).bind(supplier).execute(&mut *tx).await?;
            deleted += qr.rows_affected();
        }
        let query_string = "INSERT INTO stock(supplier, name, stock, updated) ";
        let mut query_builder = sqlx::QueryBuilder::new(query_string);
        query_builder.push_values(input, |mut b, stock| {
            b.push_bind(&stock.supplier)
                .push_bind(&stock.name)
                .push_bind(stock.stock)
                .push_bind(stock.updated);
        });
        let query = query_builder.build();
        let results = query.execute(&mut *tx).await?;
        let inserted = results.rows_affected();
        tx.commit().await?;
        Ok((deleted, inserted))
    }
    pub async fn get(&self, limit: i32, offset: i32) -> Result<Vec<Stock>> {
        let query = "SELECT * FROM stock LIMIT $1 OFFSET $2";
        let results = sqlx::query_as::<_, Stock>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
    }
}
