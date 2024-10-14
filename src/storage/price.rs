use bson::doc;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::Collection;

use crate::models::{Currency, PriceDTO, PriceItem};

use super::{Storage, PRICE_COLLECTION};

impl Storage {
    pub async fn update_prices(&self, items: Vec<crate::models::PriceItem>) -> crate::Result<()> {
        for item in items {
            self.update_price(item).await?;
        }
        Ok(())
    }

    pub async fn update_price(&self, item: crate::models::PriceItem) -> crate::Result<()> {
        let collection: Collection<PriceDTO> = self.database.collection(PRICE_COLLECTION);
        let currencies = self.get_all_currencies().await?;
        let item = PriceDTO::from(item, currencies);
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
        let collection: Collection<PriceDTO> = self.database.collection(PRICE_COLLECTION);
        let mut cursor = collection.find(doc! {}).await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item.into())
        }
        Ok(result)
    }

    pub async fn find_price(
        &self,
        search_string: String,
    ) -> crate::Result<Vec<crate::models::Price>> {
        let slice_search = search_string.split_whitespace().collect::<Vec<_>>();
        let filter = if slice_search.len() > 2 {
            let brand = slice_search[0];
            let nobrand = slice_search[1..].join(" ");
            doc! {
                "brand": doc!{"$regex": format!("^.*{}.*$", brand), "$options": "i"},
                "name": doc! {"$regex": format!("^.*{}.*$", nobrand), "$options": "i"},
            }
        } else {
            let r = slice_search
                .iter()
                .map(|w| format!(".*{w}.*"))
                .collect::<Vec<_>>()
                .join("");
            doc! {
                "name": doc! {"$regex": format!("^{r}$"), "$options": "i"},
            }
        };
        let mut cursor = self.prices.find(filter).await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item.into())
        }
        Ok(result)
    }
}

impl PriceDTO {
    fn from(value: PriceItem, currencies: Vec<Currency>) -> Self {
        let updated = Utc::now();
        let purchase_price_currency = currencies
            .iter()
            .find(|c| c.char_code.to_lowercase() == value.purchase_price_currency.to_lowercase())
            .map(|c| c.id)
            .unwrap_or_default();
        let recommended_price_currency = value.recommended_price_currency.map(|r| {
            currencies
                .iter()
                .find(|c| c.char_code == r.to_uppercase())
                .map(|c| c.id)
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
            updated: updated.into(),
        }
    }
}
