use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockDTO {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub supplier: String,
    pub name: String,
    pub stock: f64,
    pub updated: bson::DateTime,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stock {
    pub id: ObjectId,
    pub supplier: String,
    pub name: String,
    pub stock: f64,
    pub updated: DateTime<Utc>,
}
impl From<StockDTO> for Stock {
    fn from(value: StockDTO) -> Self {
        Self {
            id: value.id.unwrap(),
            supplier: value.supplier,
            name: value.name,
            stock: value.stock,
            updated: value.updated.into(),
        }
    }
}
