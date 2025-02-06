mod db;
mod mail_client;
mod ms;
mod parser;
mod web_spider;
mod woo;

use std::sync::Arc;

use crate::storage::StockStorage;
use crate::Result;
use crate::{models::Stock, utils::pause};
use mail_client::MailClient;
use rust_moysklad::MoySkladApiClient;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};
use web_spider::Spider;

pub type FetchMap =
    std::collections::HashMap<String, (Vec<Vec<u8>>, chrono::DateTime<chrono::Utc>)>;

async fn mail_generator(tx: UnboundedSender<Vec<Stock>>, mut mc: MailClient) {
    loop {
        match mc.fetch() {
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
                    info!("Нет новых остатков в почте, паауза на 1 час");
                }
            }
            Err(e) => {
                error!("Ошибка получения почты:\n{e:?}\n\nПопробую еще раз через час");
            }
        }
        pause(1).await;
    }
}

async fn web_generator(tx: UnboundedSender<Vec<Stock>>, spider: Spider) {
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
async fn router(
    mut rx: UnboundedReceiver<Vec<Stock>>,
    storage: Arc<StockStorage>,
    ms_client: Arc<MoySkladApiClient>,
    safira_woo_client: Arc<rust_woocommerce::ApiClient>,
) -> Result<()> {
    let (db_sender, db_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (ms_sender, ms_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (woo_sender, woo_receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(db::saver(db_receiver, storage.clone()));
    tokio::spawn(ms::saver(ms_receiver, ms_client.clone()));
    tokio::spawn(woo::saver(woo_receiver, safira_woo_client));
    while let Some(stock) = rx.recv().await {
        let db_sender = db_sender.clone();
        tokio::spawn(async move {
            if db_sender.send(stock).is_err() {
                tracing::error!("Ошибка отправки остатков в канал для сохранения в БД");
            }
        });
        let limit = 1000;
        let mut offset = 0;
        let mut current_stock = Vec::new();
        loop {
            let temp_current_stock = storage.get(limit, offset).await?;
            if temp_current_stock.is_empty() {
                break;
            } else {
                current_stock.extend(temp_current_stock);
                offset += limit;
            }
        }
        let ms_sender = ms_sender.clone();
        let stock_for_ms = current_stock.clone();
        tokio::spawn(async move {
            if ms_sender.send(stock_for_ms).is_err() {
                tracing::error!("Ошибка отправки остатков в канал для сохранения в Мой Склад");
            }
        });
        let woo_sender = woo_sender.clone();
        tokio::spawn(async move {
            if woo_sender.send(current_stock).is_err() {
                tracing::error!("Ошибка отправки остатков в канал для сохранения в WooCommerce");
            }
        });
    }
    drop(db_sender);
    drop(ms_sender);
    drop(woo_sender);
    Ok(())
}

pub async fn run(secrets: shuttle_runtime::SecretStore, storage: Arc<StockStorage>) -> Result<()> {
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
    let ms_token = secrets.get("MS_TOKEN").expect("MS_TOKEN not set");
    let ms_client = Arc::new(
        rust_moysklad::MoySkladApiClient::new(ms_token).map_err(|e| {
            let error = format!("{e:?}");
            tracing::error!(error);
            crate::error::AppError::Custom(error)
        })?,
    );
    let safira_ck = secrets.get("SAFIRA_CK").expect("SAFIRA_CK not set");
    let safira_cs = secrets.get("SAFIRA_CS").expect("SAFIRA_CS not set");
    let safira_host = secrets.get("SAFIRA_HOST").expect("SAFIRA_HOST not set");
    let safira_woo_client = Arc::new(
        rust_woocommerce::ApiClient::init(safira_host, safira_ck, safira_cs)
            .expect("safira_woo_client init error"),
    );
    let mail_client = MailClient::new(mail_user, mail_pass, mail_host)?;
    let spider = Spider::new(ort_user, ort_pass)?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(mail_generator(tx.clone(), mail_client));
    tokio::spawn(web_generator(tx, spider));
    router(rx, storage, ms_client, safira_woo_client).await?;
    Ok(())
}
