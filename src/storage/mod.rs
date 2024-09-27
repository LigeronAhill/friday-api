mod currency;
mod stock;
use std::sync::Arc;

use bson::doc;
use mongodb::IndexModel;

use crate::{stock_service::StockItem, Result};
const DATABASE: &str = "friday";
pub const CURRENCY_COLLECTION: &str = "currencies";
pub const STOCK_COLLECTION: &str = "stock";

#[derive(Clone)]
pub struct Storage {
    database: mongodb::Database,
}
impl Storage {
    pub async fn new(uri: &str) -> Result<Arc<Self>> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let database = client.database(DATABASE);
        let index = IndexModel::builder().keys(doc! { "name": "text" }).build();
        let stock_collection: mongodb::Collection<StockItem> =
            database.collection(STOCK_COLLECTION);
        let idx_res = stock_collection.create_index(index).await?;
        tracing::info!(
            "Создала индекс по имени в коллекции остатков: {:?}",
            idx_res
        );
        let storage = Storage { database };
        Ok(Arc::new(storage))
    }
}
