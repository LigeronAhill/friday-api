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
pub struct PriceDTO {
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
    pub updated: bson::DateTime,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Price {
    pub id: ObjectId,
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
    pub updated: chrono::DateTime<chrono::Utc>,
}
impl From<PriceDTO> for Price {
    fn from(value: PriceDTO) -> Self {
        Self {
            id: value.id.unwrap(),
            supplier: value.supplier,
            product_type: value.product_type,
            brand: value.brand,
            name: value.name,
            purchase_price: value.purchase_price,
            purchase_price_currency: value.purchase_price_currency,
            recommended_price: value.recommended_price,
            recommended_price_currency: value.recommended_price_currency,
            colors: value.colors,
            widths: value.widths,
            updated: value.updated.into(),
        }
    }
}
