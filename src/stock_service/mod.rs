mod mail_client;
mod parser;
mod web_spider;

use std::sync::Arc;

use crate::storage::StockStorage;
use mail_client::MailClient;
use tracing::{error, info};
use web_spider::Spider;

pub type FetchMap =
    std::collections::HashMap<String, (Vec<Vec<u8>>, chrono::DateTime<chrono::Utc>)>;

pub struct Stocker {
    mail_client: MailClient,
    spider: Spider,
    stock_storage: Arc<StockStorage>,
}
impl Stocker {
    pub fn new(stock_storage: Arc<StockStorage>) -> Self {
        let ort_user =
            std::env::var("ORTGRAPH_USERNAME").expect("не нашла ORTGRAPH_USER в Secrets.toml");
        let ort_pass =
            std::env::var("ORTGRAPH_PASSWORD").expect("не нашла ORTGRAPH_PASSWORD в Secrets.toml");
        let mail_host = std::env::var("MAIL_HOST").expect("не нашла MAIL_HOST в Secrets.toml");
        let mail_user = std::env::var("MAIL_USER").expect("не нашла MAIL_USER в Secrets.toml");
        let mail_pass = std::env::var("MAIL_PASS").expect("не нашла MAIL_PASS в Secrets.toml");
        let mail_client =
            MailClient::new(mail_user, mail_pass, mail_host).expect("Error init mail_client");
        let spider = Spider::new(ort_user, ort_pass).expect("Error init spider");
        Self {
            mail_client,
            spider,
            stock_storage,
        }
    }
    pub async fn run(&self) {
        tokio::spawn(mailer(self.mail_client.clone(), self.stock_storage.clone()));
        loop {
            match self.spider.get_web().await {
                Ok(fetched) => {
                    let parsed = parser::parse(fetched).await;
                    match self.stock_storage.update(&parsed).await {
                        Ok((deleted, inserted)) => {
                            info!(
                                "Удалено {deleted}, добавлено {inserted} строк остатков из сети, следующее обновление через 12 часов"
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(12 * 60 * 60))
                                .await;
                        }
                        Err(e) => {
                            error!(
                                "Ошибка обновления остатков из сети:\n{e:?}\nПовтор через 30 секунд"
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                        }
                    };
                }
                Err(e) => {
                    error!("Ошибка получения остатков из сети:\n{e:?}\nПовтор через 30 минут");
                    tokio::time::sleep(tokio::time::Duration::from_secs(30 * 60)).await;
                }
            }
        }
    }
}
async fn mailer(mut mc: MailClient, ss: Arc<StockStorage>) {
    loop {
        if let Ok(fetched) = mc.fetch()
            && !fetched.is_empty()
        {
            let parsed = parser::parse(fetched).await;
            match ss.update(&parsed).await {
                Ok((deleted, inserted)) => {
                    info!("Удалено {deleted}, добавлено {inserted} строк остатков из почты");
                }
                Err(e) => {
                    error!("Ошибка одновления остатков из почты {e:?}");
                }
            };
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;
    }
}
