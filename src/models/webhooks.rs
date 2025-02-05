use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::AppError;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRequest {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub meta: Meta,
    pub updated_fields: Option<Vec<String>>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct MsEvent {
    pub id: uuid::Uuid,
    pub product_id: uuid::Uuid,
    pub action: String,
    pub fields: Vec<String>,
    pub processed: bool,
    pub received: DateTime<Utc>,
}

impl TryFrom<Event> for MsEvent {
    type Error = AppError;

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        let url = value.meta.href.clone();
        let raw_product_id = url
            .split('/')
            .last()
            .ok_or(AppError::Custom("Invalid url".to_string()))?;
        let product_id = uuid::Uuid::parse_str(raw_product_id)
            .map_err(|e| AppError::Custom(format!("Invalid uuid in url: {e:?}")))?;
        let action = value.action.clone();
        let fields = value.updated_fields.unwrap_or_default();
        Ok(MsEvent {
            id: uuid::Uuid::new_v4(),
            product_id,
            action,
            fields,
            processed: false,
            received: Utc::now(),
        })
    }
}
