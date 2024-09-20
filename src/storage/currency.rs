use bson::doc;
use futures::TryStreamExt;
use mongodb::{bson::oid::ObjectId, Collection};
use serde::{Deserialize, Serialize};

use crate::{
    currency_service::CurrencyStorage,
    models::{CurrenciesFromCbr, Currency, ValuteDTO},
};

use super::Storage;

const COLLECTION: &str = "currencies";

#[async_trait::async_trait]
impl CurrencyStorage for Storage {
    async fn update_currencies(
        &self,
        input: crate::models::CurrenciesFromCbr,
    ) -> crate::Result<()> {
        let collection: Collection<CurrencyDTO> = self.database.collection(COLLECTION);
        collection.drop().await?;

        let rub = CurrencyDTO {
            name: "Российский рубль".to_string(),
            char_code: "RUB".to_string(),
            rate: 1.0,
            updated: mongodb::bson::DateTime::now(),
            id: None,
        };
        let mut currencies = convert(input);
        currencies.push(rub);
        let inserted = collection.insert_many(currencies).await?;
        tracing::info!("Inserted {} currencies", inserted.inserted_ids.len());
        Ok(())
    }

    async fn get_currencies(&self) -> crate::Result<Vec<crate::models::Currency>> {
        let collection: Collection<CurrencyDTO> = self.database.collection("currencies");
        let mut cursor = collection.find(doc! {}).await?;
        let mut result = Vec::new();
        while let Some(cur) = cursor.try_next().await? {
            result.push(cur.into())
        }
        result.iter().for_each(|r| tracing::info!("{r:#?}"));
        Ok(result)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyDTO {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub char_code: String,
    pub rate: f64,
    pub updated: mongodb::bson::DateTime,
}
impl From<CurrencyDTO> for Currency {
    fn from(value: CurrencyDTO) -> Self {
        Currency {
            name: value.name,
            char_code: value.char_code,
            rate: value.rate,
            updated: value.updated.into(),
        }
    }
}
impl From<ValuteDTO> for CurrencyDTO {
    fn from(value: ValuteDTO) -> Self {
        Self {
            id: None,
            name: value.name,
            char_code: value.char_code,
            rate: value.value,
            updated: mongodb::bson::DateTime::now(),
        }
    }
}
fn convert(input: CurrenciesFromCbr) -> Vec<CurrencyDTO> {
    let gbp = CurrencyDTO::from(input.valute.gbp);
    let usd = CurrencyDTO::from(input.valute.usd);
    let eur = CurrencyDTO::from(input.valute.eur);
    let cny = CurrencyDTO::from(input.valute.cny);
    vec![gbp, usd, eur, cny]
}
