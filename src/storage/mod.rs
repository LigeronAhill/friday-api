mod currency;
mod stock;
use std::sync::Arc;

use crate::Result;
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
        let storage = Storage { database };
        Ok(Arc::new(storage))
    }
}
