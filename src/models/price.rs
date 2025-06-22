use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::fmt::Display;
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
impl Price {
    pub fn safe_print(&self) -> String {
        let widths_str = self
            .widths
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("🏷 Производитель: {}\n📦 Коллекция: {}\n📛 Название: {}\n📏 Ширины (м): {}\n🧶 Состав ворса: {}\n📏 Высота ворса: {:.1} мм\n📐 Общая высота: {:.1} мм\n⚖️ Вес ворса: {} г/м²\n🏋️ Общий вес: {} г/м²\n🛡️ Класс износостойкости: {}\n🔥 Сертификат пожарной безопасности: {}\n🏷️ Рекомендуемая цена (рулон): {:.2} ₽/м²\n🏷️ Рекомендуемая цена (купон): {:.2} ₽/м²\n🕒 Обновлено: {}", self.manufacturer, self.collection, self.name, widths_str, self.pile_composition, self.pile_height, self.total_height, self.pile_weight, self.total_weight, self.durability_class, self.fire_certificate, self.recommended_roll_price, self.recommended_coupon_price, self.updated.format("%d.%m.%Y %H:%M"))
    }
}
impl Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let widths_str = self
            .widths
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        writeln!(f, "🏭 Поставщик: {}", self.supplier)?;
        writeln!(f, "🏷 Производитель: {}", self.manufacturer)?;
        writeln!(f, "📦 Коллекция: {}", self.collection)?;
        writeln!(f, "📛 Название: {}", self.name)?;
        writeln!(f, "📏 Ширины (м): {}", widths_str)?;
        writeln!(f, "🧶 Состав ворса: {}", self.pile_composition)?;
        writeln!(f, "📏 Высота ворса: {:.1} мм", self.pile_height)?;
        writeln!(f, "📐 Общая высота: {:.1} мм", self.total_height)?;
        writeln!(f, "⚖️ Вес ворса: {} г/м²", self.pile_weight)?;
        writeln!(f, "🏋️ Общий вес: {} г/м²", self.total_weight)?;
        writeln!(f, "🛡️ Класс износостойкости: {}", self.durability_class)?;
        writeln!(
            f,
            "🔥 Сертификат пожарной безопасности: {}",
            self.fire_certificate
        )?;
        writeln!(
            f,
            "💰 Закупочная цена (рулон): {:.2}",
            self.purchase_roll_price
        )?;
        writeln!(
            f,
            "💵 Закупочная цена (купон): {:.2}",
            self.purchase_coupon_price
        )?;
        writeln!(
            f,
            "🏷️ Рекомендуемая цена (рулон): {:.2}",
            self.recommended_roll_price
        )?;
        writeln!(
            f,
            "🏷️ Рекомендуемая цена (купон): {:.2}",
            self.recommended_coupon_price
        )?;
        write!(f, "🕒 Обновлено: {}", self.updated.format("%d.%m.%Y %H:%M"))
    }
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
