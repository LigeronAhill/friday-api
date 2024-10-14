use bson::doc;
use futures::TryStreamExt;
use tracing::info;

use crate::{
    models::{Currency, CurrencyDTO},
    Result,
};

use super::Storage;

impl Storage {
    pub async fn insert_currency(&self, currency: CurrencyDTO) -> Result<()> {
        let filter = doc! {
            "char_code": &currency.char_code,
            "name": &currency.name,
        };
        let update = doc! {
            "$set": doc! {
            "name": &currency.name,
            "char_code": &currency.char_code,
            "rate": &currency.rate,
            "updated": &currency.updated,
            },
        };
        let name = currency.name;
        self.currencies
            .update_one(filter, update)
            .upsert(true)
            .await?;
        info!("Обновила валюту {name}");
        Ok(())
    }
    pub async fn get_currency_by_char_code(&self, char_code: &str) -> Result<Option<Currency>> {
        let filter = doc! {"char_code": char_code};
        let doc = self.currencies.find_one(filter).await?.map(Currency::from);
        Ok(doc)
    }
    pub async fn get_all_currencies(&self) -> Result<Vec<Currency>> {
        let mut cursor = self.currencies.find(doc! {}).await?;
        let mut currencies = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            currencies.push(doc.into());
        }
        Ok(currencies)
    }
}
