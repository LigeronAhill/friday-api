mod mail_client;
mod parser;
mod web_spider;
use std::sync::Arc;

use crate::{storage::Storage, Result};
use mail_client::MailClient;
use tokio::time::sleep;
use tracing::{error, info};
use web_spider::Spider;

pub type FetchMap =
    std::collections::HashMap<String, (Vec<Vec<u8>>, chrono::DateTime<chrono::Utc>)>;

pub struct StockService {
    mail_client: MailClient,
    spider: Spider,
    storage: Arc<Storage>,
}

impl StockService {
    pub fn new(secrets: shuttle_runtime::SecretStore, storage: Arc<Storage>) -> Result<Self> {
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
