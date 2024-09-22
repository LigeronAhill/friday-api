use std::collections::HashSet;

use bson::doc;
use bson::oid::ObjectId;
use futures::TryStreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::stock_service::{StockItem, StockStorage};
use crate::Result;

use super::{Storage, STOCK_COLLECTION};

#[async_trait::async_trait]
impl StockStorage for Storage {
    async fn update_stock(&self, items: Vec<StockItem>) -> Result<()> {
        let collection: Collection<StockItemDTO> = self.database.collection(STOCK_COLLECTION);
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
        let converted = items
            .into_iter()
            .map(StockItemDTO::from)
            .collect::<Vec<_>>();
        let result = collection.insert_many(converted).await?;
        let inserted = result.inserted_ids.len();
        tracing::info!("Обновила {inserted} позиций остатков в базе данных");
        Ok(())
    }
    async fn get_stock(&self, limit: i64, offset: u64) -> Result<Vec<StockItem>> {
        let collection: Collection<StockItemDTO> = self.database.collection(STOCK_COLLECTION);
        let mut cursor = collection.find(doc! {}).limit(limit).skip(offset).await?;
        let mut result = Vec::new();
        while let Some(item) = cursor.try_next().await? {
            result.push(item.into())
        }
        Ok(result)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StockItemDTO {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    supplier: String,
    name: String,
    stock: f64,
    updated: mongodb::bson::DateTime,
}
impl From<StockItem> for StockItemDTO {
    fn from(value: StockItem) -> Self {
        Self {
            id: None,
            supplier: value.supplier,
            name: value.name,
            stock: value.stock,
            updated: value.updated.into(),
        }
    }
}
impl From<StockItemDTO> for StockItem {
    fn from(value: StockItemDTO) -> Self {
        Self {
            supplier: value.supplier,
            name: value.name,
            stock: value.stock,
            updated: value.updated.into(),
        }
    }
}
// pub struct StockItem {
//     pub supplier: String,
//     pub name: String,
//     pub stock: f64,
// }
