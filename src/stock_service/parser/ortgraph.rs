use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::models::Stock;

pub async fn parser(files: Vec<Vec<u8>>, received: DateTime<Utc>) -> Vec<Stock> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        for file in files {
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
                    error!("Ошибка при открытии книги из сети от 'Ортграф': {e:?}");
                    continue;
                }
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
        if let Some(stock) = row
            .get(3)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
            let name = row.first().map(|d| d.to_string()).unwrap_or_default();
            let item = Stock {
                supplier: "ortgraph".to_string(),
                name: name.clone(),
                stock,
                updated: received,
                id: None,
            };
            if tx.send(item).is_err() {
                error!("Ошибка отправки строки в канал...")
            }
        }
    }
}
