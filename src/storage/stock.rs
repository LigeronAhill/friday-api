use std::collections::HashSet;

use bson::doc;
use futures::TryStreamExt;
use mongodb::Collection;
use tracing::info;

use crate::models::Stock;
use crate::Result;

use super::{Storage, STOCK_COLLECTION};

impl Storage {
    pub async fn update_stock(&self, items: Vec<Stock>) -> Result<()> {
        let collection: Collection<Stock> = self.database.collection(STOCK_COLLECTION);
        let mut set = HashSet::new();
        items.iter().map(|i| i.supplier.clone()).for_each(|s| {
            set.insert(s);
        });
        for supplier in set {
            let filter = doc! {"supplier": &supplier};
            let dr = collection.delete_many(filter).await?;
            let count = dr.deleted_count;
            info!("Удалила {count} позиций из остатков '{supplier}'");
        }
        let result = collection.insert_many(items).await?;
        let inserted = result.inserted_ids.len();
        tracing::info!("Обновила {inserted} позиций остатков в базе данных");
        Ok(())
    }
    pub async fn get_stock(&self, limit: i64, offset: u64) -> Result<Vec<Stock>> {
        let collection: Collection<Stock> = self.database.collection(STOCK_COLLECTION);
        let mut cursor = collection.find(doc! {}).limit(limit).skip(offset).await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item.into())
        }
        Ok(result)
    }
    pub async fn find_stock(&self, search: String) -> Result<Vec<Stock>> {
        let collection: Collection<Stock> = self.database.collection(STOCK_COLLECTION);
        let mut cursor = collection
            .find(doc! {"name": {"$regex": search, "$options": "i"}})
            .await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item.into())
        }
        Ok(result)
    }
}
