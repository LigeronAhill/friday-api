mod fancy;
mod fox;
mod supplier;
use std::sync::Arc;
use std::{collections::HashMap, io::Cursor};
pub use supplier::Supplier;

use crate::storage::PriceStorage;
use anyhow::Result;
use calamine::open_workbook_auto_from_rs;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::unbounded_channel;

pub async fn file_router(url: &str, price_storage: Arc<PriceStorage>) -> Result<String> {
    let body = reqwest::get(url).await?.bytes().await?;
    let cursor = Cursor::new(body.clone());
    let workbook = open_workbook_auto_from_rs(cursor.clone())?;
    let supplier = Supplier::try_from(workbook)?;
    let (tx, mut rx) = unbounded_channel();
    match supplier {
        Supplier::Fancy => {
            tokio::spawn(async move { fancy::parse(cursor, tx).await });
        }
        Supplier::Fox => {
            tokio::spawn(fox::parse(cursor, tx));
        }
    }
    let mut prices = Vec::new();
    while let Some(p) = rx.recv().await {
        prices.push(p);
    }
    let prices = deduplicate(prices)
        .into_iter()
        .map(|i| i.convert())
        .collect::<Vec<_>>();
    let updated = price_storage.update(prices).await?;
    let answer =
        format!("Получен файл прайс-листов от поставщика '{supplier}', доступен по ссылке: {url}, добавлено строк в БД: {updated}.");
    Ok(answer)
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct ParsedPriceItem {
    #[builder(setter(into))]
    pub supplier: String,
    #[builder(setter(into))]
    pub manufacturer: String,
    #[builder(setter(into))]
    pub collection: String,
    #[builder(field(build = "self.name_build()"))]
    pub name: String,
    #[builder(setter(custom), default)]
    pub widths: Vec<f64>,
    #[builder(setter(into))]
    pub pile_composition: String,
    #[builder(default)]
    pub pile_height: f64,
    #[builder(default)]
    pub total_height: f64,
    #[builder(default)]
    pub pile_weight: i32,
    #[builder(default)]
    pub total_weight: i32,
    #[builder(default)]
    pub durability_class: i32,
    #[builder(setter(into), default)]
    pub fire_certificate: String,
    #[builder(default)]
    pub purchase_roll_price: f64,
    #[builder(default)]
    pub purchase_coupon_price: f64,
    #[builder(default)]
    pub recommended_roll_price: f64,
    #[builder(default)]
    pub recommended_coupon_price: f64,
}

impl ParsedPriceItem {
    pub fn builder() -> ParsedPriceItemBuilder {
        ParsedPriceItemBuilder::default()
    }
    pub fn convert(self) -> crate::models::PriceItem {
        crate::models::PriceItem {
            supplier: self.supplier,
            manufacturer: self.manufacturer,
            collection: self.collection,
            name: self.name,
            widths: self.widths,
            pile_composition: self.pile_composition,
            pile_height: self.pile_height,
            total_height: self.total_height,
            pile_weight: self.pile_weight,
            total_weight: self.total_weight,
            durability_class: self.durability_class,
            fire_certificate: self.fire_certificate,
            purchase_roll_price: self.purchase_roll_price,
            purchase_coupon_price: self.purchase_coupon_price,
            recommended_roll_price: self.recommended_roll_price,
            recommended_coupon_price: self.recommended_coupon_price,
        }
    }
}

impl ParsedPriceItemBuilder {
    fn name_build(&self) -> String {
        let raw_name = format!(
            "{manufacturer} {collection}",
            manufacturer = self.manufacturer.clone().unwrap_or_default(),
            collection = self.collection.clone().unwrap_or_default()
        );
        let sl = raw_name
            .split_whitespace()
            .map(|w| w.trim())
            .collect::<Vec<_>>()
            .join(" ");
        sl
    }
    fn widths(&mut self, w: f64) -> &mut Self {
        self.widths.get_or_insert(vec![]).push(w);
        self
    }
    fn validate(&self) -> Result<(), String> {
        if self.purchase_roll_price.is_none()
            && self.purchase_coupon_price.is_none()
            && self.recommended_roll_price.is_none()
            && self.recommended_coupon_price.is_none()
        {
            Err(String::from("None of price fields is defined"))
        } else {
            Ok(())
        }
    }
}

fn deduplicate(input: Vec<ParsedPriceItem>) -> Vec<ParsedPriceItem> {
    let mut price_map = HashMap::new();
    for item in input {
        price_map.insert(
            (
                item.supplier.clone(),
                item.manufacturer.clone(),
                item.collection.clone(),
            ),
            item,
        );
    }
    let mut result = Vec::new();
    for (_, item) in price_map {
        result.push(item);
    }
    result
}
#[derive(Serialize, Deserialize, Debug)]
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
    pub updated: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    #[test]
    fn test_price_item_builder_success() -> Result<()> {
        let price = ParsedPriceItem::builder()
            .supplier("test supplier")
            .manufacturer("test manufacturer")
            .collection("test collection")
            .widths(1.0)
            .widths(2.0)
            .pile_composition("test pile composition")
            .purchase_coupon_price(100.)
            .build()?;
        assert_eq!(price.purchase_coupon_price, 100.);
        Ok(())
    }
    #[test]
    fn test_price_item_builder_failed() -> Result<()> {
        let price_without_manufacturer = ParsedPriceItem::builder()
            .supplier("test supplier")
            .collection("test collection")
            .widths(1.0)
            .widths(2.0)
            .pile_composition("test pile composition")
            .purchase_coupon_price(100.)
            .build();
        assert!(price_without_manufacturer.is_err());
        let price_without_supplier = ParsedPriceItem::builder()
            .manufacturer("test manufacturer")
            .collection("test collection")
            .widths(1.0)
            .widths(2.0)
            .pile_composition("test pile composition")
            .purchase_coupon_price(100.)
            .build();
        assert!(price_without_supplier.is_err());
        let price_without_prices = ParsedPriceItem::builder()
            .supplier("test supplier")
            .manufacturer("test manufacturer")
            .collection("test collection")
            .widths(1.0)
            .widths(2.0)
            .pile_composition("test pile composition")
            .build();
        assert!(price_without_prices.is_err());
        let price_without_width = ParsedPriceItem::builder()
            .supplier("test supplier")
            .manufacturer("test manufacturer")
            .collection("test collection")
            .pile_composition("test pile composition")
            .purchase_coupon_price(100.)
            .build();
        assert!(price_without_width.is_ok());
        Ok(())
    }
}
