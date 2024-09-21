use super::{FetchMap, StockItem};
use tracing::error;

mod carpetland;
mod fancy;
mod fenix;
mod fox;
mod opus;
mod ortgraph;
mod vvk;
mod zefir;

pub async fn parse(fetches: FetchMap) -> Vec<StockItem> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        for (supplier, (files, received)) in fetches {
            match supplier.as_str() {
                "opus" => {
                    let tx = tx.clone();
                    let res = opus::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "fox" => {
                    let tx = tx.clone();
                    let res = fox::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "fancy" => {
                    let tx = tx.clone();
                    let res = fancy::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "carpetland" => {
                    let tx = tx.clone();
                    let res = carpetland::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "zefir" => {
                    let tx = tx.clone();
                    let res = zefir::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "fenix" => {
                    let tx = tx.clone();
                    let res = fenix::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "vvk" => {
                    let tx = tx.clone();
                    let res = vvk::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                "ortgraph" => {
                    let tx = tx.clone();
                    let res = ortgraph::parser(files.clone(), received).await;
                    if tx.send(res).is_err() {
                        error!("Ошибка при отправке результата парсинга в канал...");
                    }
                }
                _ => continue,
            }
        }
    });
    let mut result = Vec::new();
    while let Some(m) = rx.recv().await {
        result.extend(m)
    }
    result
}
