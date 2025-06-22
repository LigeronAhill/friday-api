use crate::models::MsEvent;

#[derive(Clone)]
pub struct EventsStorage {
    pool: sqlx::PgPool,
}
impl EventsStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    pub async fn save(&self, events: Vec<MsEvent>) -> crate::Result<u64> {
        let query_string = "INSERT INTO ms_events(product_id, action, fields) ";
        let mut query_builder = sqlx::QueryBuilder::new(query_string);
        query_builder.push_values(events, |mut b, event| {
            b.push_bind(event.product_id)
                .push_bind(event.action.to_string())
                .push_bind(event.fields);
        });
        let query = query_builder.build();
        let results = query.execute(&self.pool).await?;
        let rows_affected = results.rows_affected();
        Ok(rows_affected)
    }
    pub async fn get(&self) -> crate::Result<Vec<MsEvent>> {
        let query = "SELECT * FROM ms_events WHERE processed = false";
        let results = sqlx::query_as::<_, MsEvent>(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(results)
    }
    pub async fn remove_processed(&self) -> crate::Result<()> {
        let query = "DELETE FROM ms_events WHERE processed = true";
        sqlx::query(query).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn process(&self, id: uuid::Uuid) -> crate::Result<()> {
        let query = "UPDATE ms_events SET processed = true WHERE id = $1";
        sqlx::query(query).bind(id).execute(&self.pool).await?;
        Ok(())
    }
}
