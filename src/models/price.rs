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
    pub width: Vec<f64>,
    pub pile_composition: String,
    pub pile_height: f64,
    pub pile_weight: f64,
    pub total_height: f64,
    pub total_weight: f64,
    pub durability_class: String,
    pub fire_certificate: String,
    pub purchase_roll_price: f64,
    pub purchase_coupon_price: f64,
    pub recommended_roll_price: f64,
    pub recommended_coupon_price: f64,
    pub updated: DateTime<Utc>,
}
