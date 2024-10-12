use std::io::Cursor;

use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
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
                    if let Some(Ok(table)) = wb.worksheet_range_at(0) {
                        let tx = tx.clone();
                        tokio::spawn(parse(table, received, tx));
                    }
                }
                Err(e) => {
                    error!("Ошибка при открытии книги из вложений от 'Братец Лис': {e:?}");
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
    let mut name = String::new();
    let re = regex::Regex::new(r#"^[А-я]+\s.+$"#).unwrap();
    for row in table.rows() {
        let temp_name = row.get(2).and_then(|d| d.get_string()).unwrap_or_default();
        if re.is_match(temp_name) {
            name = temp_name.to_string();
        } else if let Some(stock) = row
            .get(6)
            .and_then(|d| d.to_string().trim().parse::<f64>().ok())
        {
            let item = Stock {
                supplier: "fox".to_string(),
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
