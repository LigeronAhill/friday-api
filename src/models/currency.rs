use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
/// Ответ от сервера с курсами валют
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrenciesFromCbr {
    #[serde(rename = "Valute")]
    pub valute: Valute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Valute {
    pub gbp: ValuteDTO,
    pub usd: ValuteDTO,
    pub eur: ValuteDTO,
    pub cny: ValuteDTO,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ValuteDTO {
    pub char_code: String,
    pub name: String,
    pub value: f64,
}
/// Ответ на API запрос /api/currencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub name: String,
    pub char_code: String,
    pub rate: f64,
    pub updated: DateTime<Utc>,
}
