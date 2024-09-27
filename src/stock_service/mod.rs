mod mail_client;
mod parser;
mod web_spider;
use std::sync::Arc;

use crate::{models::AppState, Result};
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Utc};
use mail_client::MailClient;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{error, info};
use web_spider::Spider;

pub type FetchMap =
    std::collections::HashMap<String, (Vec<Vec<u8>>, chrono::DateTime<chrono::Utc>)>;
#[async_trait::async_trait]
pub trait StockStorage: Send + Sync {
    async fn update_stock(&self, items: Vec<StockItem>) -> Result<()>;
    async fn get_stock(&self, limit: i64, offset: u64) -> Result<Vec<StockItem>>;
    async fn find_stock(&self, search: String) -> Result<Vec<StockItem>>;
}
pub struct StockService {
    mail_client: MailClient,
    spider: Spider,
    storage: Arc<dyn StockStorage>,
}

impl StockService {
    pub fn new(
        secrets: shuttle_runtime::SecretStore,
        storage: Arc<dyn StockStorage>,
    ) -> Result<Self> {
        let ort_user = secrets
            .get("ORTGRAPH_USERNAME")
            .expect("не нашла ORTHGRAPH_USER в Secrets.toml");
        let ort_pass = secrets
            .get("ORTGRAPH_PASSWORD")
            .expect("не нашла ORTHGRAPH_PASSWORD в Secrets.toml");
        let mail_host = secrets
            .get("MAIL_HOST")
            .expect("не нашла MAIL_HOST в Secrets.toml");
        let mail_user = secrets
            .get("MAIL_USER")
            .expect("не нашла MAIL_USER в Secrets.toml");
        let mail_pass = secrets
            .get("MAIL_PASS")
            .expect("не нашла MAIL_PASS в Secrets.toml");
        let mail_client = MailClient::new(mail_user, mail_pass, mail_host)?;
        let spider = Spider::new(ort_user, ort_pass)?;
        Ok(Self {
            mail_client,
            spider,
            storage,
        })
    }
    pub async fn run(&self) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut mc = self.mail_client.clone();
        let sender = tx.clone();
        tokio::spawn(async move {
            loop {
                match mc.fetch() {
                    Ok(mails) => {
                        let items = parser::parse(mails).await;
                        if sender.send(items).is_err() {
                            error!("Ошибка при отправке почтовых вложений в канал, попробую еще раз через час");
                            sleep(tokio::time::Duration::from_secs(60 * 60)).await;
                        } else {
                            sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
                        }
                    }
                    Err(e) => {
                        error!("Ошибка получения почты:\n{e:?}\n\nПопробую еще раз через час");
                        sleep(tokio::time::Duration::from_secs(60 * 60)).await;
                    }
                }
            }
        });
        let spider = self.spider.clone();
        tokio::spawn(async move {
            loop {
                match spider.get_web().await {
                    Ok(f) => {
                        let webs = parser::parse(f).await;
                        if tx.send(webs).is_err() {
                            error!("Ошибка при отправке веб остатков в канал, попробую еще раз через час");
                            sleep(tokio::time::Duration::from_secs(60 * 60)).await;
                        } else {
                            sleep(tokio::time::Duration::from_secs(60 * 60 * 24)).await;
                        }
                    }
                    Err(e) => {
                        error!("Ошибка получения почты:\n{e:?}\n\nПопробую еще раз через час");
                        sleep(tokio::time::Duration::from_secs(60 * 60)).await;
                    }
                }
            }
        });
        while let Some(f) = rx.recv().await {
            match self.storage.update_stock(f).await {
                Ok(_) => info!("Сток обновлен"),
                Err(e) => error!("Ошибка обновления стока в базе данных:\n{e:?}"),
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StockItem {
    pub supplier: String,
    pub name: String,
    pub stock: f64,
    pub updated: DateTime<Utc>,
}

#[derive(Deserialize)]
struct Query {
    limit: Option<String>,
    offset: Option<String>,
    search: Option<String>,
}

#[get("/api/stock")]
pub async fn stock(state: web::Data<AppState>, query: Option<web::Query<Query>>) -> HttpResponse {
    match query {
        Some(q) => {
            if let Some(search) = q.search.to_owned() {
                match state.stock_storage.find_stock(search).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            } else if let Some(limit) = q.limit.to_owned().and_then(|l| l.parse().ok()) {
                let offset = q
                    .offset
                    .to_owned()
                    .and_then(|o| o.parse().ok())
                    .unwrap_or_default();
                match state.stock_storage.get_stock(limit, offset).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            } else {
                match state.stock_storage.get_stock(100, 0).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            }
        }
        None => match state.stock_storage.get_stock(100, 0).await {
            Ok(r) => HttpResponse::Ok().json(r),
            Err(e) => HttpResponse::InternalServerError().json(e),
        },
    }
    // let (limit, offset) = match query {
    //     Some(q) => (
    //         q.limit.clone().and_then(|l| l.parse().ok()).unwrap_or(20),
    //         q.offset.clone().and_then(|o| o.parse().ok()).unwrap_or(0),
    //     ),
    //     None => (20, 0),
    // };
    // match state.stock_storage.get_stock(limit, offset).await {
    //     Ok(r) => HttpResponse::Ok().json(r),
    //     Err(e) => HttpResponse::InternalServerError().json(e),
    // }
}
