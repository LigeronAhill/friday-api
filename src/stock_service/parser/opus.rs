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
                    let sheets = wb.worksheets();
                    for (_, table) in sheets {
                        let tx = tx.clone();
                        tokio::spawn(parse(table, received, tx));
                    }
                }
                Err(e) => {
                    error!("Ошибка при открытии книги из вложений от 'Опус-Контракт': {e:?}");
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
    let mut brand = String::new();
    let mut pt = String::new();
    for row in table.rows() {
        if let Some(stock) = row.get(5).and_then(|data| data.get_float()) {
            if let Some(raw_name) = row
                .first()
                .and_then(|data| data.get_string().map(|w| w.to_string()))
            {
                if PRODUCT_TYPES.contains(&raw_name.as_str()) {
                    pt = raw_name;
                    continue;
                } else if BRANDS.contains(&raw_name.as_str()) {
                    brand = raw_name;
                    continue;
                } else if stock > 5.0 {
                    let name = format!("{pt} {brand} {raw_name}");
                    let item = Stock {
                        supplier: "opus".to_string(),
                        name,
                        stock,
                        updated: received.into(),
                        id: uuid::Uuid::new_v4(),
                    };
                    if tx.send(item).is_err() {
                        error!("Ошибка отправки строки в канал...")
                    }
                }
            }
        }
    }
}

const PRODUCT_TYPES: [&str; 16] = [
    "Грязезащита",
    "Интернет-магазин",
    "Искусственная трава",
    "Ковровая плитка",
    "Контрактные обои",
    "Мебель",
    "Осветительное оборудование",
    "Паркет",
    "ПВХ плитка",
    "ПВХ рулонные",
    "Подвесные потолки",
    "Резиновые покрытия",
    "Рулонные ковровые покрытия",
    "Сопутствующие товары",
    "Стеновые панели",
    "Фальшполы",
];

const BRANDS: [&str; 56] = [
    "Betap",
    "Уличные покрытия",
    "Desoma Grass",
    "Betap",
    "Bloq",
    "Innovflor",
    "Interface",
    "IVC (Mohawk)",
    "Tapibel",
    "Виниловые покрытия",
    "Флизелиновые обои под покраску",
    "Ресторация",
    "CSVT",
    "Navigator",
    "ЛЕД-Эффект",
    "РУСВИТАЛЭЛЕКТРО",
    "Barlinek",
    "Coswick",
    "Royal Parket",
    "Карелия Упофлор",
    "паркет VOLVO",
    "Спортивные системы",
    "ADO Floor",
    "Interface",
    "KBS floor",
    "Tarkett",
    "Vertigo",
    "Гомогенный",
    "С защитой от статического электричества / токопроводящий",
    "Спортивный",
    "МЕТАЛЛИЧЕСКИЕ ПОТОЛКИ",
    "МЕТАЛЛИЧЕСКИЕ ПРОСТЫЕ ПОТОЛКИ",
    "МИНЕРАЛЬНЫЕ ПОТОЛКИ",
    "Beka Rubber",
    "Desoma Rubber Fitness Premium",
    "Beaulieu International Group",
    "Betap Tufting B.V.",
    "Condor carpets",
    "Haima",
    "Luxemburg",
    "Синтелон",
    "Материалы для монтажа и ухода",
    "Плинтус",
    "Подложка",
    "Шнур сварочный",
    "FORTIKA CDF",
    "FORTIKA HPL",
    "Swiss KRONO CDF",
    "CBI (Си-Би-Ай)",
    "Fortika",
    "Perfaten, АСП",
    "Конструктор (Аксиома)(Айрон)",
    "Панели других производителей",
    "Сопутствующие товары",
    "Стойки других производителей",
    "Стрингеры",
];
