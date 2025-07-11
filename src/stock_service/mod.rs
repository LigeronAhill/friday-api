mod mail_client;
mod parser;
mod web_spider;

use std::ops::Deref;
use std::sync::Arc;

use crate::storage::StockStorage;
use crate::{models::Stock, utils::pause};
use mail_client::MailClient;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{error, info};
use web_spider::Spider;

pub type FetchMap =
    std::collections::HashMap<String, (Vec<Vec<u8>>, chrono::DateTime<chrono::Utc>)>;

pub struct Stocker {
    mail_client: Arc<MailClient>,
    spider: Arc<Spider>,
    stock_storage: Arc<StockStorage>,
}
impl Stocker {
    pub fn new(
        secrets: shuttle_runtime::SecretStore,
        stock_storage: Arc<StockStorage>,
    ) -> Arc<Self> {
        let ort_user = secrets
            .get("ORTGRAPH_USERNAME")
            .expect("не нашла ORTGRAPH_USER в Secrets.toml");
        let ort_pass = secrets
            .get("ORTGRAPH_PASSWORD")
            .expect("не нашла ORTGRAPH_PASSWORD в Secrets.toml");
        let mail_host = secrets
            .get("MAIL_HOST")
            .expect("не нашла MAIL_HOST в Secrets.toml");
        let mail_user = secrets
            .get("MAIL_USER")
            .expect("не нашла MAIL_USER в Secrets.toml");
        let mail_pass = secrets
            .get("MAIL_PASS")
            .expect("не нашла MAIL_PASS в Secrets.toml");
        let mail_client = Arc::new(
            MailClient::new(mail_user, mail_pass, mail_host).expect("Error init mail_client"),
        );
        let spider = Arc::new(Spider::new(ort_user, ort_pass).expect("Error init spider"));
        Arc::new(Self {
            mail_client,
            spider,
            stock_storage,
        })
    }
    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let (mail_sender, mail_receiver) = unbounded_channel();
        tokio::spawn(mail_generator(
            mail_sender,
            self.clone().mail_client.clone(),
        ));
        let (web_sender, web_receiver) = unbounded_channel();
        tokio::spawn(web_generator(web_sender, self.clone().spider.clone()));
        let channels = vec![mail_receiver, web_receiver];
        self.clone().saver(channels).await?;
        Ok(())
    }
    async fn saver(
        self: Arc<Self>,
        channels: Vec<UnboundedReceiver<Vec<Stock>>>,
    ) -> anyhow::Result<()> {
        let (tx, mut rx) = unbounded_channel();
        for mut receiver in channels {
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(s) = receiver.recv().await {
                    if let Err(e) = tx.send(s) {
                        error!("{e:?}");
                    }
                }
            });
        }
        while let Some(s) = rx.recv().await {
            let (deleted, inserted) = self.clone().stock_storage.clone().update(&s).await?;
            info!("Удалено {deleted}, добавлено {inserted} строк остатков");
        }
        Ok(())
    }
}
async fn mail_generator(tx: UnboundedSender<Vec<Stock>>, mc: Arc<MailClient>) {
    loop {
        let mut client = mc.clone().deref().clone();
        match client.fetch() {
            Ok(mails) => {
                let items = parser::parse(mails).await;
                let quantity = items.len();
                if !items.is_empty() {
                    if tx.send(items).is_err() {
                        error!(
                        "Ошибка при отправке почтовых вложений в канал, попробую еще раз через час"
                    );
                    } else {
                        info!("Получено {quantity} строк остатков из почты, пауза на 1 час");
                    }
                } else {
                    info!("Нет новых остатков в почте, пауза на 1 час");
                }
            }
            Err(e) => {
                error!("Ошибка получения почты:\n{e:?}\n\nПопробую еще раз через час");
            }
        }
        pause(1).await;
    }
}

async fn web_generator(tx: UnboundedSender<Vec<Stock>>, spider: Arc<Spider>) {
    loop {
        match spider.get_web().await {
            Ok(f) => {
                let webs = parser::parse(f).await;
                let quantity = webs.len();
                if !webs.is_empty() {
                    if tx.send(webs).is_err() {
                        error!(
                            "Ошибка при отправке веб остатков в канал, попробую еще раз через час"
                        );
                        pause(1).await;
                    } else {
                        info!("Получено {quantity} строк остатков из сети, пауза на 24 часа");
                        pause(24).await;
                    }
                } else {
                    error!("Пустой ответ сети на запрос остатков, попробую еще раз через час");
                    pause(1).await;
                }
            }
            Err(e) => {
                error!("Ошибка получения почты:\n{e:?}\n\nПопробую еще раз через час");
                pause(1).await;
            }
        }
    }
}
// async fn router(
//     mut rx: UnboundedReceiver<Vec<Stock>>,
//     storage: Arc<StockStorage>,
//     ms_client: Arc<rust_moysklad::MoySkladApiClient>,
//     safira_woo_client: Arc<rust_woocommerce::ApiClient>,
// ) {
//     let (db_sender, db_receiver) = tokio::sync::mpsc::unbounded_channel();
//     let (ms_sender, ms_receiver) = tokio::sync::mpsc::unbounded_channel();
//     let (woo_sender, woo_receiver) = tokio::sync::mpsc::unbounded_channel();
//     let storage_instance = storage.clone();
//     tokio::spawn(async move {
//         db::saver(db_receiver, storage_instance).await;
//     });
//     tokio::spawn(async move {
//         ms::saver(ms_receiver, ms_client.clone()).await;
//     });
//     tokio::spawn(async move {
//         woo::saver(woo_receiver, safira_woo_client).await;
//     });
//     while let Some(stock) = rx.recv().await {
//         let db_sender = db_sender.clone();
//         tokio::spawn(async move {
//             if db_sender.send(stock).is_err() {
//                 error!("Ошибка отправки остатков в канал для сохранения в БД");
//             }
//         });
//         let limit = 500;
//         let mut offset = 0;
//         let mut current_stock = Vec::new();
//         loop {
//             let temp_current_stock = storage.get(limit, offset).await.unwrap_or_default();
//             if temp_current_stock.is_empty() {
//                 break;
//             } else {
//                 current_stock.extend(temp_current_stock);
//                 offset += limit;
//             }
//         }
//         let ms_sender = ms_sender.clone();
//         let stock_for_ms = current_stock.clone();
//         tokio::spawn(async move {
//             if ms_sender.send(stock_for_ms).is_err() {
//                 error!("Ошибка отправки остатков в канал для сохранения в Мой Склад");
//             }
//         });
//         let woo_sender = woo_sender.clone();
//         tokio::spawn(async move {
//             if woo_sender.send(current_stock).is_err() {
//                 error!("Ошибка отправки остатков в канал для сохранения в WooCommerce");
//             }
//         });
//     }
//     drop(db_sender);
//     drop(ms_sender);
//     drop(woo_sender);
// }
//
// pub async fn run(secrets: shuttle_runtime::SecretStore, storage: Arc<StockStorage>) {
//     let ort_user = secrets
//         .get("ORTGRAPH_USERNAME")
//         .expect("не нашла ORTGRAPH_USER в Secrets.toml");
//     let ort_pass = secrets
//         .get("ORTGRAPH_PASSWORD")
//         .expect("не нашла ORTGRAPH_PASSWORD в Secrets.toml");
//     let mail_host = secrets
//         .get("MAIL_HOST")
//         .expect("не нашла MAIL_HOST в Secrets.toml");
//     let mail_user = secrets
//         .get("MAIL_USER")
//         .expect("не нашла MAIL_USER в Secrets.toml");
//     let mail_pass = secrets
//         .get("MAIL_PASS")
//         .expect("не нашла MAIL_PASS в Secrets.toml");
//     let ms_token = secrets.get("MS_TOKEN").expect("MS_TOKEN not set");
//     let ms_client = Arc::new(
//         rust_moysklad::MoySkladApiClient::new(ms_token)
//             .map_err(|e| {
//                 let error = format!("{e:?}");
//                 tracing::error!(error);
//                 crate::error::AppError::Custom(error)
//             })
//             .expect("ms_client init error"),
//     );
//     let safira_ck = secrets.get("SAFIRA_CK").expect("SAFIRA_CK not set");
//     let safira_cs = secrets.get("SAFIRA_CS").expect("SAFIRA_CS not set");
//     let safira_host = secrets.get("SAFIRA_HOST").expect("SAFIRA_HOST not set");
//     let safira_woo_client = Arc::new(
//         rust_woocommerce::ApiClient::init(safira_host, safira_ck, safira_cs)
//             .expect("safira_woo_client init error"),
//     );
//     let mail_client =
//         MailClient::new(mail_user, mail_pass, mail_host).expect("Error init mail_client");
//     let spider = Spider::new(ort_user, ort_pass).expect("Error init spider");
//     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//     let mail_tx = tx.clone();
//     tokio::spawn(async move {
//         mail_generator(mail_tx, mail_client).await;
//     });
//     tokio::spawn(async move {
//         web_generator(tx, spider).await;
//     });
//     router(rx, storage, ms_client, safira_woo_client).await;
// }
