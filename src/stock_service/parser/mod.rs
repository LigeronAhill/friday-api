use super::FetchMap;
use crate::models::Stock;
use tracing::error;

mod carpetland;
mod fancy;
mod fenix;
mod fox;
mod opus;
mod ortgraph;
mod sf;
mod vvk;
mod zefir;

pub async fn parse(fetches: FetchMap) -> Vec<Stock> {
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
                "sportflooring" => {
                    let tx = tx.clone();
                    let res = sf::parser(
                        files
                            .clone()
                            .first()
                            .map(|s| s.to_owned())
                            .unwrap_or_default(),
                        received,
                    )
                    .await;
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

pub fn clear_string(input: impl AsRef<str>) -> String {
    input
        .as_ref()
        .split_whitespace()
        .map(|w| w.trim().to_uppercase())
        .collect::<Vec<_>>()
        .join(" ")
}
