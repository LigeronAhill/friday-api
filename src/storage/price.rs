use bson::doc;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::Collection;

use crate::models::{Currency, Price, PriceItem};

use super::{Storage, PRICE_COLLECTION};

impl Storage {
    pub async fn update_prices(&self, items: Vec<crate::models::PriceItem>) -> crate::Result<()> {
        for item in items {
            self.update_price(item).await?;
        }
        Ok(())
    }

    pub async fn update_price(&self, item: crate::models::PriceItem) -> crate::Result<()> {
        let collection: Collection<Price> = self.database.collection(PRICE_COLLECTION);
        let currencies = self.get_latest_currency_rates().await?;
        let item = Price::from(item, currencies);
        let filter = doc! {
            "supplier": &item.supplier,
            "product_type": &item.product_type,
            "brand": &item.brand,
            "name": &item.name,
        };
        let update = doc! {
            "$set": doc! {
            "supplier": &item.supplier,
            "product_type": &item.product_type,
            "brand": &item.brand,
            "name": &item.name,
            "purchase_price": item.purchase_price,
            "purchase_price_currency": &item.purchase_price_currency,
            "recommended_price": item.recommended_price,
            "recommended_price_currency": &item.recommended_price_currency,
            "colors": item.colors,
            "widths": item.widths,
            "updated": item.updated,
            }
        };
        collection.update_one(filter, update).upsert(true).await?;
        Ok(())
    }

    pub async fn read_all_price_items(&self) -> crate::Result<Vec<crate::models::Price>> {
        let collection: Collection<Price> = self.database.collection(PRICE_COLLECTION);
        let mut cursor = collection.find(doc! {}).await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item)
        }
        Ok(result)
    }

    pub async fn find_price_item(
        &self,
        search_string: String,
    ) -> crate::Result<Vec<crate::models::Price>> {
        let collection: Collection<Price> = self.database.collection(PRICE_COLLECTION);
        let mut cursor = collection.find(doc! {}).await?;
        let mut all_prices = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            all_prices.push(item)
        }
        Ok(search(all_prices, search_string))
    }
}

fn search(haystack: Vec<Price>, search_string: String) -> Vec<Price> {
    if search_string.is_empty() {
        return haystack;
    }
    let mut result = Vec::new();
    let search_slice = search_string
        .split_whitespace()
        .map(|w| w.trim().to_lowercase())
        .collect::<Vec<_>>();

    for word in search_slice {
        if result.is_empty() {
            result = haystack
                .clone()
                .into_iter()
                .filter(|item| {
                    let name = get_name(item);
                    name.contains(&word)
                })
                .collect();
        } else {
            result = result
                .clone()
                .into_iter()
                .filter(|item| {
                    let name = get_name(item);
                    name.contains(&word)
                })
                .collect();
        }
    }
    result
}
fn get_name(item: &Price) -> String {
    format!(
        "{} {} {} {}",
        item.supplier, item.product_type, item.brand, item.name
    )
    .to_lowercase()
}

impl Price {
    fn from(value: PriceItem, currencies: Vec<Currency>) -> Self {
        let updated = Utc::now();
        let purchase_price_currency = currencies
            .iter()
            .find(|c| c.char_code.to_lowercase() == value.purchase_price_currency.to_lowercase())
            .and_then(|c| c.id)
            .unwrap_or_default();
        let recommended_price_currency = value.recommended_price_currency.map(|r| {
            currencies
                .iter()
                .find(|c| c.char_code == r.to_uppercase())
                .and_then(|c| c.id)
                .unwrap_or_default()
        });
        Self {
            id: None,
            supplier: value.supplier.to_uppercase(),
            product_type: value.product_type.to_uppercase(),
            brand: value.brand.to_uppercase(),
            name: value.name.to_uppercase(),
            purchase_price: value.purchase_price,
            purchase_price_currency,
            recommended_price: value.recommended_price,
            recommended_price_currency,
            colors: value.colors,
            widths: value.widths,
            updated,
        }
    }
}
