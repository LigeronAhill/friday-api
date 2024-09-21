use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
use chrono::{DateTime, Utc};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

use crate::stock_service::StockItem;

pub async fn parser(files: Vec<Vec<u8>>, received: DateTime<Utc>) -> Vec<StockItem> {
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
                    error!("Ошибка при открытии книги из вложений от 'Фэнси': {e:?}");
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

async fn parse(table: Range<Data>, received: DateTime<Utc>, tx: UnboundedSender<StockItem>) {
    let re = regex::Regex::new(r#"^([A-z]+)\s.+$"#).unwrap();
    let mut name = String::new();
    for row in table.rows() {
        let temp_name = row.first().and_then(|d| d.get_string()).unwrap_or_default();
        if re.is_match(temp_name) {
            let second_name = row.get(4).and_then(|d| d.get_string()).unwrap_or_default();
            name = format!("{temp_name} {second_name}");
        } else if let Some(current) = row
            .get(4)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
            let reserved = row
                .get(6)
                .and_then(|d| d.to_string().trim().parse::<f64>().ok())
                .unwrap_or_default();
            let stock = current - reserved;
            if !name.is_empty() {
                let item = StockItem {
                    name: name.clone(),
                    stock,
                    supplier: "fancy".to_string(),
                    updated: received,
                };
                if tx.send(item).is_err() {
                    error!("Ошибка отправки строки в канал...")
                }
            }
        }
    }
}
