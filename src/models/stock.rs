use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Stock {
    pub id: uuid::Uuid,
    pub supplier: String,
    pub name: String,
    pub stock: f64,
    pub updated: DateTime<Utc>,
}
