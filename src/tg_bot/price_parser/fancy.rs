use std::io::Cursor;

use bytes::Bytes;
use calamine::{open_workbook_auto_from_rs, Data, DataType, Range, Reader};
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use super::PriceItem;

#[instrument(name = "parsing fancy", skip_all)]
pub async fn parse(cursor: Cursor<Bytes>, tx: UnboundedSender<PriceItem>) {
    if let Ok(mut workbook) = open_workbook_auto_from_rs(cursor) {
        let sheets = workbook.worksheets();
        for (_, sheet) in sheets {
            parse_storage_program(storage_program(sheet.clone()), tx.clone());
            parse_creatuft(creatuft(sheet.clone()), tx.clone());
            parse_tasibel(tasibel(sheet.clone()), tx.clone());
            parse_lano(lano(sheet.clone()), tx.clone());
            parse_condor_betap(condor_betap(sheet), tx.clone());
        }
    }
}
#[instrument(name = "getting storage program range", skip_all)]
fn storage_program(sheet: Range<Data>) -> Range<Data> {
    let (start, mut end) = (0, 0);
    for (i, row) in sheet.rows().enumerate() {
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "CREATUFT")
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            end = i;
            break;
        }
    }
    sheet.range((start as u32, 0), (end as u32, 12))
}
#[instrument(name = "getting creatuft range", skip_all)]
fn creatuft(sheet: Range<Data>) -> Range<Data> {
    let (mut start, mut end) = (0, 0);
    let mut start_founded = false;
    for (i, row) in sheet.rows().enumerate() {
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "CREATUFT")
            && !start_founded
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            start = i;
            start_founded = true
        }
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "TASIBEL")
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            end = i;
            break;
        }
    }
    sheet.range((start as u32, 0), (end as u32, 10))
}
#[instrument(name = "getting tasibel range", skip_all)]
fn tasibel(sheet: Range<Data>) -> Range<Data> {
    let (mut start, mut end) = (0, 0);
    let mut start_founded = false;
    for (i, row) in sheet.rows().enumerate() {
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "TASIBEL")
            && !start_founded
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            start = i;
            start_founded = true
        }
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "LANO")
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            end = i;
            break;
        }
    }
    sheet.range((start as u32, 0), (end as u32, 10))
}
#[instrument(name = "getting lano range", skip_all)]
fn lano(sheet: Range<Data>) -> Range<Data> {
    let (mut start, mut end) = (0, 0);
    let mut start_founded = false;
    for (i, row) in sheet.rows().enumerate() {
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "LANO")
            && !start_founded
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            start = i;
            start_founded = true
        }
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "CONDOR")
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            end = i;
            break;
        }
    }
    sheet.range((start as u32, 0), (end as u32, 10))
}
#[instrument(name = "getting condor_betap range", skip_all)]
fn condor_betap(sheet: Range<Data>) -> Range<Data> {
    let end = sheet.end().unwrap_or_default();
    let mut start = 0;
    for (i, row) in sheet.rows().enumerate() {
        if row
            .first()
            .is_some_and(|d| d.to_string().to_uppercase() == "CONDOR")
            && row.get(1).is_some_and(|d| d.to_string().is_empty())
        {
            start = i;
            break;
        }
    }
    sheet.range((start as u32, 0), end)
}
#[instrument(name = "parsing storage program", skip_all)]
fn parse_storage_program(sheet: Range<Data>, tx: UnboundedSender<PriceItem>) {
    let mut brand = String::new();
    for row in sheet.rows() {
        let Some(rec_price_value) = row.get(12) else {
            continue;
        };
        let Ok(recommended_price) = rec_price_value
            .to_string()
            .replace(',', ".")
            .trim()
            .parse::<f64>()
        else {
            continue;
        };
        if let Some(manufacturer) = row.first().map(|d| d.to_string().trim().to_uppercase()) {
            if !manufacturer.is_empty() {
                brand = manufacturer;
            }
        }
        let Some(collection) = row.get(1).map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let mut price_item = PriceItem::builder();
        price_item
            .supplier("ФЭНСИ ФЛОР")
            .manufacturer(&brand)
            .collection(collection)
            .recommended_coupon_price(recommended_price);
        if let Some(pile_composition) = row.get(3).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.pile_composition(pile_composition);
        };
        if let Some(width) = row.get(2).and_then(|d| {
            d.to_string()
                .trim()
                .replace("м", "")
                .replace(',', ".")
                .parse::<f64>()
                .ok()
        }) {
            price_item.widths(width);
        }
        if let Some(pile_height) = row.get(4).and_then(|d| {
            d.to_string()
                .trim()
                .replace(" мм", "")
                .replace(',', ".")
                .parse::<f64>()
                .ok()
        }) {
            price_item.pile_height(pile_height);
        }
        if let Some(total_height) = row.get(5).and_then(|d| {
            d.to_string()
                .trim()
                .replace(" мм", "")
                .replace(',', ".")
                .parse::<f64>()
                .ok()
        }) {
            price_item.total_height(total_height);
        }
        if let Some(pile_weight) = row
            .get(6)
            .and_then(|d| d.to_string().trim().parse::<i32>().ok())
        {
            price_item.pile_weight(pile_weight);
        }
        if let Some(total_weight) = row
            .get(7)
            .and_then(|d| d.to_string().trim().parse::<i32>().ok())
        {
            price_item.total_weight(total_weight);
        }
        if let Some(fire) = row.get(9).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.fire_certificate(fire);
        }
        if let Some(prp) = row
            .get(10)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_roll_price(prp);
        }
        if let Some(pcp) = row
            .get(11)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_coupon_price(pcp);
        }
        if let Ok(price) = price_item.build() {
            if let Err(e) = tx.send(price) {
                tracing::error!("{e:?}");
            }
        }
    }
}
#[instrument(name = "parsing creatuft", skip_all)]
fn parse_creatuft(sheet: Range<Data>, tx: UnboundedSender<PriceItem>) {
    for row in sheet.rows() {
        let Some(rcp) = row
            .get(9)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let Some(collection) = row.first().map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let mut price_item = PriceItem::builder();
        price_item
            .supplier("ФЭНСИ ФЛОР")
            .manufacturer("CREATUFT")
            .collection(collection)
            .recommended_coupon_price(rcp);
        if let Some(pile_composition) = row.get(5).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.pile_composition(pile_composition);
        };
        if let Some(widths) = row.get(3).map(|d| {
            d.to_string()
                .trim()
                .replace("cm", "")
                .replace("00", "")
                .split('+')
                .flat_map(|w| w.parse::<f64>().ok())
                .collect::<Vec<_>>()
        }) {
            for w in widths {
                price_item.widths(w);
            }
        }
        if let Some(total_weight) = row.get(6).and_then(|d| d.get_float()) {
            price_item.total_weight((total_weight * 1000.0) as i32);
        }
        if let Some(prp) = row
            .get(7)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_roll_price(prp);
        }
        if let Some(pcp) = row
            .get(8)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_coupon_price(pcp);
        }
        if let Ok(price) = price_item.build() {
            if let Err(e) = tx.send(price) {
                tracing::error!("{e:?}");
            }
        }
    }
}
#[instrument(name = "parsing tasibel", skip_all)]
fn parse_tasibel(sheet: Range<Data>, tx: UnboundedSender<PriceItem>) {
    for row in sheet.rows() {
        let Some(rcp) = row
            .get(8)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let Some(collection) = row.first().map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let mut price_item = PriceItem::builder();
        price_item
            .supplier("ФЭНСИ ФЛОР")
            .manufacturer("TASIBEL")
            .collection(collection)
            .recommended_coupon_price(rcp);
        if let Some(pile_composition) = row.get(5).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.pile_composition(pile_composition);
        };
        if let Some(widths) = row.get(1).map(|d| {
            d.to_string()
                .trim()
                .replace("ca.", "")
                .replace("cm", "")
                .replace(' ', "")
                .replace('-', "&")
                .split("&")
                .flat_map(|w| w.parse::<i32>().map(|d| d as f64 / 100.0).ok())
                .collect::<Vec<_>>()
        }) {
            for w in widths {
                price_item.widths(w);
            }
        }
        if let Some(total_weight) = row.get(3).and_then(|d| d.get_float()) {
            price_item.total_weight((total_weight * 1000.0) as i32);
        }
        if let Some(pile_weight) = row
            .get(4)
            .and_then(|d| d.to_string().trim().parse::<i32>().ok())
        {
            price_item.pile_weight(pile_weight);
        }
        if let Some(prp) = row
            .get(6)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_roll_price(prp);
        }
        if let Some(pcp) = row
            .get(7)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_coupon_price(pcp);
        }
        if let Ok(price) = price_item.build() {
            if let Err(e) = tx.send(price) {
                tracing::error!("{e:?}");
            }
        }
    }
}
fn parse_lano(sheet: Range<Data>, tx: UnboundedSender<PriceItem>) {
    for row in sheet.rows() {
        let Some(rrp) = row
            .get(6)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let Some(collection) = row.first().map(|d| d.to_string().trim().to_uppercase()) else {
            continue;
        };
        let mut price_item = PriceItem::builder();
        price_item
            .supplier("ФЭНСИ ФЛОР")
            .manufacturer("LANO")
            .collection(collection)
            .recommended_roll_price(rrp);
        if let Some(pile_composition) = row.get(4).map(|d| d.to_string().trim().to_uppercase()) {
            price_item.pile_composition(pile_composition);
        };
        if let Some(w) = row.get(3).and_then(|d| {
            d.to_string()
                .trim()
                .parse::<i32>()
                .map(|d| d as f64 / 100.0)
                .ok()
        }) {
            price_item.widths(w);
        }
        if let Some(total_weight) = row.get(2).and_then(|d| {
            d.to_string()
                .split_whitespace()
                .map(|w| w.trim())
                .collect::<Vec<_>>()
                .join("")
                .parse::<i32>()
                .ok()
        }) {
            price_item.total_weight(total_weight);
        }
        if let Some(pile_weight) = row.get(1).and_then(|d| {
            d.to_string()
                .split_whitespace()
                .map(|w| w.trim())
                .collect::<Vec<_>>()
                .join("")
                .parse::<i32>()
                .ok()
        }) {
            price_item.pile_weight(pile_weight);
        }
        if let Some(prp) = row
            .get(5)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        {
            price_item.purchase_roll_price(prp);
        }
        if let Ok(price) = price_item.build() {
            if let Err(e) = tx.send(price) {
                tracing::error!("{e:?}");
            }
        }
    }
}
fn parse_condor_betap(sheet: Range<Data>, tx: UnboundedSender<PriceItem>) {
    let mut brand = "CONDOR";
    for row in sheet.rows() {
        for cell in row {
            if cell.to_string().contains("BETAP") {
                brand = "BETAP";
            }
        }
        let Some(rcp) = row
            .get(7)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let Some(collection) = row.first().map(|w| w.to_string().trim().to_uppercase()) else {
            continue;
        };
        let Some(pile_composition) = row.get(1).map(|w| w.to_string().trim().to_uppercase()) else {
            continue;
        };
        let Some(pile_weight) = row
            .get(2)
            .and_then(|d| d.to_string().trim().parse::<i32>().ok())
        else {
            continue;
        };
        let Some(widths) = row.get(3).map(|d| {
            d.to_string()
                .trim()
                .split(",")
                .flat_map(|w| w.trim().parse::<f64>().ok())
                .collect::<Vec<_>>()
        }) else {
            continue;
        };
        let Some(prp) = row
            .get(5)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let Some(pcp) = row
            .get(6)
            .and_then(|d| d.to_string().trim().replace(',', ".").parse::<f64>().ok())
        else {
            continue;
        };
        let mut price_item = PriceItem::builder();
        price_item
            .manufacturer(brand)
            .supplier("ФЭНСИ ФЛОР")
            .collection(collection)
            .pile_composition(pile_composition)
            .pile_weight(pile_weight)
            .purchase_roll_price(prp)
            .purchase_coupon_price(pcp)
            .recommended_coupon_price(rcp);
        for w in widths {
            price_item.widths(w);
        }
        if let Ok(price) = price_item.build() {
            if let Err(e) = tx.send(price) {
                tracing::error!("{e:?}");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokio::sync::mpsc::unbounded_channel;
    #[tokio::test]
    async fn test_fancy_parse() -> Result<()> {
        let file = std::fs::read("input_for_tests/fancy.xlsx")?;
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
        let file_name = format!("out_of_tests/fancy/{now}.json");
        std::fs::write(file_name, body)?;
        Ok(())
    }
}
