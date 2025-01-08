use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Currency {
    pub id: uuid::Uuid,
    pub name: String,
    pub char_code: String,
    pub rate: f64,
    pub updated: DateTime<Utc>,
}
