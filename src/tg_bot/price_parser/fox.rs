use std::io::Cursor;

use bytes::Bytes;
use calamine::{open_workbook_auto_from_rs, DataType, Reader};
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use super::ParsedPriceItem;

#[instrument(name = "parsing fox", skip_all)]
pub async fn parse(cursor: Cursor<Bytes>, tx: UnboundedSender<ParsedPriceItem>) {
    if let Ok(mut workbook) = open_workbook_auto_from_rs(cursor) {
        let sheets = workbook.worksheets();
        for (sheet_name, sheet) in sheets {
            match sheet_name.as_str() {
                "Коммерческий линолеум" => {
                    parse_commercial_linoleum(sheet, tx.clone());
                }
                "Ковровая плитка" => {
                    parse_carpet_tile(sheet, tx.clone());
                }
                _ => continue,
            }
        }
    }
}

#[instrument(name = "parsing carpet tile", skip_all)]
fn parse_carpet_tile(table: calamine::Range<calamine::Data>, sender: UnboundedSender<ParsedPriceItem>) {
    for row in table.rows() {
        if let Some(pcp) = row.get(13).and_then(|d| {
            d.to_string()
                .replace(['*', '€', ' '], "")
                .parse::<f64>()
                .ok()
        }) {
            let Some(brand) = row.first().map(|d| d.to_string().trim().to_uppercase()) else {
                continue;
            };
            let Some(collection) = row.get(1).map(|d| d.to_string().trim().to_uppercase()) else {
                continue;
            };
            let Some(durability_class) = row.get(2).and_then(|d| d.get_int().map(|i| i as i32))
            else {
                continue;
            };
            let Some(fire_certificate) = row.get(3).map(|d| d.to_string().trim().to_uppercase())
            else {
                continue;
            };
            let Some(pile_composition) = row.get(4).map(|d| d.to_string().trim().to_uppercase())
            else {
                continue;
            };
            let Some(total_height) = row.get(8).and_then(|d| {
                d.to_string()
                    .replace(',', ".")
                    .replace(' ', "")
                    .parse::<f64>()
                    .ok()
            }) else {
                continue;
            };
            let Some(pile_height) = row.get(9).and_then(|d| {
                d.to_string()
                    .replace(',', ".")
                    .replace(' ', "")
                    .parse::<f64>()
                    .ok()
            }) else {
                continue;
            };
            let Some(total_weight) = row
                .get(10)
                .and_then(|d| d.to_string().replace(' ', "").parse::<i32>().ok())
            else {
                continue;
            };
            let Some(pile_weight) = row
                .get(11)
                .and_then(|d| d.to_string().replace(' ', "").parse::<i32>().ok())
            else {
                continue;
            };
            let mut price_item = ParsedPriceItem::builder();
            price_item
                .manufacturer(brand)
                .collection(collection)
                .pile_composition(pile_composition)
                .supplier("БРАТЕЦ ЛИС")
                .pile_height(pile_height)
                .pile_weight(pile_weight)
                .fire_certificate(fire_certificate)
                .total_height(total_height)
                .durability_class(durability_class)
                .total_weight(total_weight)
                .purchase_coupon_price(pcp);
            if let Some(rcp) = row.get(14).and_then(|d| {
                d.to_string()
                    .replace(['*', '€', ' '], "")
                    .parse::<f64>()
                    .ok()
            }) {
                price_item.recommended_coupon_price(rcp);
            }
            match price_item.build() {
                Ok(p) => {
                    if let Err(e) = sender.send(p) {
                        tracing::error!("{e:?}");
                    }
                }
                Err(e) => tracing::error!("{e:?}"),
            }
        }
    }
}

#[instrument(name = "parsing commercial linoleum", skip_all)]
fn parse_commercial_linoleum(
    table: calamine::Range<calamine::Data>,
    sender: UnboundedSender<ParsedPriceItem>,
) {
    for row in table.rows() {
        let Some(prp) = row.get(11).and_then(|d| d.to_string().parse::<f64>().ok()) else {
            continue;
        };
        let Some(brand) = row.first().map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let Some(collection) = row.get(1).map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let Some(linoleum_type) = row.get(2).map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let mut price_item = ParsedPriceItem::builder();
        price_item
            .purchase_roll_price(prp)
            .supplier("БРАТЕЦ ЛИС")
            .manufacturer(brand)
            .pile_composition(linoleum_type)
            .collection(format!("{collection} РУЛОНЫ"));
        if let Some(widths) = row.get(3).map(|d| {
            d.to_string()
                .split_whitespace()
                .flat_map(|w| {
                    w.replace("м,", "")
                        .replace('м', "")
                        .replace(',', ".")
                        .parse::<f64>()
                        .ok()
                })
                .collect::<Vec<_>>()
        }) {
            for w in widths {
                price_item.widths(w);
            }
        }
        if let Some(class) = row.get(4).and_then(|d| {
            d.to_string()
                .trim()
                .split("/")
                .last()
                .and_then(|f| f.parse::<i32>().ok())
        }) {
            price_item.durability_class(class);
        }
        if let Some(fire) = row.get(8).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.fire_certificate(fire);
        }
        match price_item.build() {
            Ok(p) => {
                if let Err(e) = sender.send(p) {
                    tracing::error!("{e:?}");
                }
            }
            Err(e) => tracing::error!("{e:?}"),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokio::sync::mpsc::unbounded_channel;
    #[tokio::test]
    async fn test_fox_parse() -> Result<()> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
        let file = std::fs::read("input_for_tests/fox.xlsx")?;
        let b = Bytes::copy_from_slice(&file);
        let cursor = Cursor::new(b);
        let (tx, mut rx) = unbounded_channel();
        tokio::spawn(parse(cursor, tx));
        let mut prices = Vec::new();
        while let Some(p) = rx.recv().await {
            prices.push(p);
        }
        let body = serde_json::to_string_pretty(&prices)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs();
        let file_name = format!("out_of_tests/fox/{now}.json");
        std::fs::write(file_name, body)?;
        Ok(())
    }
}
