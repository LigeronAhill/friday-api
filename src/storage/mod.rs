mod currency;
mod price;
mod stock;
use std::sync::Arc;

use bson::doc;
use mongodb::{options::IndexOptions, Collection, IndexModel};
use tracing::{error, info};

use crate::{
    models::{CurrencyDTO, PriceDTO, StockDTO},
    Result,
};
const DATABASE: &str = "friday";
pub const CURRENCY_COLLECTION: &str = "currencies";
pub const STOCK_COLLECTION: &str = "stock";
pub const PRICE_COLLECTION: &str = "prices";

#[derive(Clone)]
pub struct Storage {
    database: mongodb::Database,
    currencies: Collection<CurrencyDTO>,
    // stock: Collection<StockDTO>,
    prices: Collection<PriceDTO>,
}
impl Storage {
    pub async fn new(uri: &str) -> Result<Arc<Self>> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        let database = client.database(DATABASE);
        let currencies: Collection<CurrencyDTO> = database.collection(CURRENCY_COLLECTION);
        let stock: Collection<StockDTO> = database.collection(STOCK_COLLECTION);
        let prices: Collection<PriceDTO> = database.collection(PRICE_COLLECTION);
        let opts = IndexOptions::builder().unique(true).build();
        let char_code_index = IndexModel::builder()
            .keys(doc! { "char_code": "text" })
            .options(opts)
            .build();
        match currencies.create_index(char_code_index).await {
            Ok(r) => {
                info!("Создала индекс в коллекции валют: {}", r.index_name);
            }
            Err(e) => {
                error!("Не удалось создать индекс в коллекции валют: {e:?}")
            }
        }
        let index = IndexModel::builder().keys(doc! { "name": "text" }).build();
        match stock.create_index(index.clone()).await {
            Ok(res) => {
                info!("Создала индекс в коллекции остатков: {}", res.index_name);
            }
            Err(e) => {
                error!("Не удалось создать индекс в коллекции остатков: {e:?}")
            }
        }
        match prices.create_index(index).await {
            Ok(res) => {
                info!(
                    "Создала индекс в коллекции прайс-листов: {}",
                    res.index_name
                );
            }
            Err(e) => {
                error!("Не удалось создать индекс в коллекции прайс-листов: {e:?}")
            }
        }

        let storage = Storage {
            database,
            currencies,
            // stock,
            prices,
        };
        Ok(Arc::new(storage))
    }
}
