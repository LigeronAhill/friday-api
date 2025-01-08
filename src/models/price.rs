use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PriceItem {
    pub supplier: String,
    pub product_type: String,
    pub brand: String,
    pub name: String,
    pub purchase_price: f64,
    pub purchase_price_currency: String,
    pub recommended_price: Option<f64>,
    pub recommended_price_currency: Option<String>,
    pub colors: Vec<String>,
    pub widths: Vec<f64>,
}
#[derive(Serialize, Deserialize, Clone, Debug, FromRow)]
pub struct Price {
    pub id: uuid::Uuid,
    pub supplier: String,
    pub product_type: String,
    pub brand: String,
    pub name: String,
    pub purchase_price: f64,
    pub purchase_price_currency: uuid::Uuid,
    pub recommended_price: Option<f64>,
    pub recommended_price_currency: Option<uuid::Uuid>,
    pub colors: Vec<String>,
    pub widths: Vec<f64>,
    pub updated: chrono::DateTime<chrono::Utc>,
}