use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Price {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub supplier: String,
    pub product_type: String,
    pub brand: String,
    pub name: String,
    pub purchase_price: f64,
    pub purchase_price_currency: ObjectId,
    pub recommended_price: Option<f64>,
    pub recommended_price_currency: Option<ObjectId>,
    pub colors: Vec<String>,
    pub widths: Vec<f64>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated: chrono::DateTime<chrono::Utc>,
}
