use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, FromRow, Default)]
pub struct Price {
    pub id: Uuid,
    pub supplier: String,
    pub manufacturer: String,
    pub collection: String,
    pub name: String,
    pub widths: Vec<f64>,
    pub pile_composition: String,
    pub pile_height: f64,
    pub total_height: f64,
    pub pile_weight: i32,
    pub total_weight: i32,
    pub durability_class: i32,
    pub fire_certificate: String,
    pub purchase_roll_price: f64,
    pub purchase_coupon_price: f64,
    pub recommended_roll_price: f64,
    pub recommended_coupon_price: f64,
    pub updated: DateTime<Utc>,
}
#[derive(Serialize, Deserialize, Debug, FromRow, Default)]
pub struct PriceItem {
    pub supplier: String,
    pub manufacturer: String,
    pub collection: String,
    pub name: String,
    pub widths: Vec<f64>,
    pub pile_composition: String,
    pub pile_height: f64,
    pub total_height: f64,
    pub pile_weight: i32,
    pub total_weight: i32,
    pub durability_class: i32,
    pub fire_certificate: String,
    pub purchase_roll_price: f64,
    pub purchase_coupon_price: f64,
    pub recommended_roll_price: f64,
    pub recommended_coupon_price: f64,
}
#[derive(Serialize, Deserialize, Debug, FromRow, Default)]
pub struct PriceDTO {
    pub manufacturer: String,
    pub collection: String,
    pub name: String,
    pub widths: Vec<f64>,
    pub pile_composition: String,
    pub pile_height: f64,
    pub total_height: f64,
    pub pile_weight: i32,
    pub total_weight: i32,
    pub durability_class: i32,
    pub fire_certificate: String,
    pub recommended_roll_price: f64,
    pub recommended_coupon_price: f64,
    pub updated: DateTime<Utc>,
}
impl From<&Price> for PriceDTO {
    fn from(price: &Price) -> Self {
        Self {
            manufacturer: price.manufacturer.clone(),
            collection: price.collection.clone(),
            name: price.name.clone(),
            widths: price.widths.clone(),
            pile_composition: price.pile_composition.clone(),
            pile_height: price.pile_height,
            total_height: price.total_height,
            pile_weight: price.pile_weight,
            total_weight: price.total_weight,
            durability_class: price.durability_class,
            fire_certificate: price.fire_certificate.clone(),
            recommended_roll_price: price.recommended_roll_price,
            recommended_coupon_price: price.recommended_coupon_price,
            updated: price.updated,
        }
    }
}
