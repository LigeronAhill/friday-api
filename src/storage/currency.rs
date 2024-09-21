use bson::doc;
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::{bson::oid::ObjectId, Collection};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    currency_service::CurrencyStorage,
    models::{CurrenciesFromCbr, Currency, ValuteDTO},
    storage::CURRENCY_COLLECTION,
};

use super::Storage;

#[async_trait::async_trait]
impl CurrencyStorage for Storage {
    /// обновляет курсы валют в MongoDB
    async fn update_currencies(
        &self,
        input: crate::models::CurrenciesFromCbr,
    ) -> crate::Result<()> {
        let today = chrono::Utc::now();
        let date = today.date_naive();
        let latest = self.get_latest_currency_rates().await?;
        let collection: Collection<CurrencyDTO> = self.database.collection(CURRENCY_COLLECTION);
        if latest.is_empty() {
            let rub = CurrencyDTO {
                name: "Российский рубль".to_string(),
                char_code: "RUB".to_string(),
                rate: 1.0,
                updated: mongodb::bson::DateTime::now(),
                id: None,
            };
            let inserted = collection.insert_one(rub).await?;
            tracing::info!(
                "Добавила 'рубль' в курсы валют с id: {:?}",
                inserted.inserted_id
            );
        }
        if latest.is_empty()
            || latest
                .first()
                .is_some_and(|c| c.updated.date_naive() < date)
        {
            let currencies = convert(input);
            let inserted = collection.insert_many(currencies).await?;
            tracing::info!("Обновлено валют: {}", inserted.inserted_ids.len());
        }
        let month_ago = chrono::Utc::now()
            .checked_sub_months(chrono::Months::new(1))
            .unwrap_or_default();
        let old = bson::DateTime::from_chrono(month_ago);
        let filter = doc! {"updated": doc! {"$lt": old}};
        let deleted = collection.delete_many(filter).await?.deleted_count;
        if deleted > 0 {
            info!("Удалила {deleted} старых курсов валют");
        }
        Ok(())
    }
    /// получает последние курсы валют из MongoDB
    async fn get_latest_currency_rates(&self) -> crate::Result<Vec<crate::models::Currency>> {
        let collection: Collection<CurrencyDTO> = self.database.collection("currencies");
        let today = chrono::Utc::now();
        let yesterday = today
            .checked_sub_days(chrono::Days::new(1))
            .unwrap_or_default();
        let today_bson = bson::DateTime::from_chrono(today);
        let yesterday_bson = bson::DateTime::from_chrono(yesterday);
        let filter = doc! { "updated": doc! {"$gt": yesterday_bson, "$lt": today_bson}};
        let mut cursor = collection.find(filter).await?;
        let mut result = Vec::new();
        while let Some(cur) = cursor.try_next().await? {
            result.push(cur.into())
        }
        Ok(result)
    }
    /// получает курсы валют за месяц из MongoDB
    async fn get_monthly_currency_rates(&self) -> crate::Result<Vec<crate::models::Currency>> {
        let collection: Collection<CurrencyDTO> = self.database.collection("currencies");
        let mut cursor = collection.find(doc! {}).sort(doc! {"updated": -1}).await?;
        let mut result: Vec<Currency> = Vec::new();
        while let Some(cur) = cursor.try_next().await? {
            result.push(cur.into())
        }
        // result.sort_by(|a, b| a.updated.cmp(&b.updated));
        Ok(result)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrencyDTO {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    char_code: String,
    rate: f64,
    updated: mongodb::bson::DateTime,
}
impl From<CurrencyDTO> for Currency {
    fn from(value: CurrencyDTO) -> Self {
        let updated: chrono::DateTime<Utc> = value.updated.into();
        Currency {
            name: value.name,
            char_code: value.char_code,
            rate: value.rate,
            updated,
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
