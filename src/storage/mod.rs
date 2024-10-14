mod currency;
mod price;
mod stock;
use std::sync::Arc;

use bson::doc;
use mongodb::IndexModel;

use crate::{
    models::{Price, Stock},
    Result,
};
const DATABASE: &str = "friday";
pub const CURRENCY_COLLECTION: &str = "currencies";
pub const STOCK_COLLECTION: &str = "stock";
pub const PRICE_COLLECTION: &str = "prices";

#[derive(Clone)]
pub struct Storage {
    database: mongodb::Database,
}
impl Storage {
    pub async fn new(uri: &str) -> Result<Arc<Self>> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let database = client.database(DATABASE);
        let index = IndexModel::builder().keys(doc! { "name": "text" }).build();
        let stock_collection: mongodb::Collection<Stock> = database.collection(STOCK_COLLECTION);
        match stock_collection.create_index(index).await {
            Ok(r) => {
                tracing::info!("Создала индекс по имени в коллекции остатков: {:?}", r);
            }
            Err(e) => tracing::error!("{e:?}"),
        }
        let name_index = IndexModel::builder().keys(doc! { "name": "text" }).build();
        let price_collection: mongodb::Collection<Price> = database.collection(PRICE_COLLECTION);
        match price_collection.create_index(name_index).await {
            Ok(r) => {
                tracing::info!("Создала индекс по имени в коллекции прайсов: {:?}", r);
            }
            Err(e) => tracing::error!("{e:?}"),
        }
        let storage = Storage { database };
        Ok(Arc::new(storage))
    }
}
