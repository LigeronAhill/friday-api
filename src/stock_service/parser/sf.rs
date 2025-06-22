use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::models::Stock;

use super::clear_string;

pub async fn parser(file: Vec<u8>, received: DateTime<Utc>) -> Vec<Stock> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        let cursor = Cursor::new(file);
        match open_workbook_auto_from_rs(cursor) {
            Ok(mut wb) => {
                let sheets = wb.worksheets();
                for (_, table) in sheets {
                    let tx = tx.clone();
                    tokio::spawn(parse(table, received, tx));
                }
            }
            Err(e) => {
                error!("Ошибка при чтении книги из вложений 'Интерьерные решения': {e:?}");
            }
        }
    });
    let mut result = Vec::new();
    while let Some(pr) = rx.recv().await {
        result.push(pr)
    }
    result
}

async fn parse(table: Range<Data>, received: DateTime<Utc>, tx: UnboundedSender<Stock>) {
    for row in table.rows() {
        if let Some(stock) = row.get(8).and_then(|d| {
            d.to_string()
                .replace(" шт.", "")
                .replace(" уп.", "")
                .trim()
                .parse::<f64>()
                .ok()
        }) {
            if let Some(name) = row.get(3).map(|w| {
                w.to_string()
                    .split_whitespace()
                    .map(|d| d.trim())
                    .collect::<Vec<_>>()
                    .join(" ")
            }) {
                let item = Stock {
                    name: clear_string(&name),
                    stock,
                    supplier: "sportflooring".to_string(),
                    updated: received,
                    id: uuid::Uuid::new_v4(),
                };
                if tx.send(item).is_err() {
                    error!("Ошибка при отправке строки из файла в канал...")
                }
            }
        }
    }
}
