use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::models::StockDTO;

pub async fn parser(files: Vec<Vec<u8>>, received: DateTime<Utc>) -> Vec<StockDTO> {
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
                    error!("Ошибка при открытии книги из вложения от 'Зефир': {e:?}");
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

async fn parse(table: Range<Data>, received: DateTime<Utc>, tx: UnboundedSender<StockDTO>) {
    for row in table.rows() {
        if let Some(stock) = row
            .get(3)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
            let name = row.get(1).map(|d| d.to_string()).unwrap_or_default();
            let item = StockDTO {
                supplier: "zefir".to_string(),
                name: name.clone(),
                stock,
                updated: received.into(),
                id: None,
            };
            if tx.send(item).is_err() {
                error!("Ошибка отправки строки в канал...")
            }
        }
    }
}
