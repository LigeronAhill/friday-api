use std::io::Cursor;

use super::ParsedPriceItem;
use bytes::Bytes;
use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

#[instrument(name = "parsing fancy", skip_all)]
pub async fn parse(cursor: Cursor<Bytes>, tx: UnboundedSender<ParsedPriceItem>) {
    if let Ok(mut workbook) = open_workbook_auto_from_rs(cursor) {
        let sheets = workbook.worksheets();
        if let Some(f) = sheets.first().map(|(_, s)| s) {
            parse_fancy_v2(f, tx.clone());
        }
    }
}
#[instrument(name = "getting storage program range", skip_all)]
fn parse_fancy_v2(sheet: &Range<Data>, tx: UnboundedSender<ParsedPriceItem>) {
    const SUPPLIER: &str = "ФЭНСИ";
    const MANUFACTURERS: [&str; 20] = [
        "ФэнсиБлокс",
        "ФэнсиЛум",
        "Assotiated Weavers*   ",
        "Tasibel    ",
        "Betap     (новинка)",
        "Creatuft",
        "Lano",
        "Osta",
        "Ligne Pure",
        "Haima",
        "Versa",
        "BN International",
        "Dollken",
        "Homakoll",
        "TASIBEL",
        "LANO",
        "CONDOR",
        "BETAP",
        "DOLLKEN",
        "HOMA",
    ];
    const COLLECTION: [&str; 2] = ["Коллекция", "Коллекция - основа"];
    const WIDTH: [&str; 2] = ["Ширина", "Ширина, см"];
    const PILE_COMPOSITION: [&str; 2] = ["Состав", "Состав ворса"];
    const PILE_HEIGHT: &str = "Высота ворса";
    const TOTAL_HEIGHT: &str = "Общая толщина";
    const PILE_WEIGHT: [&str; 4] = [
        "Вес, кг/м2",
        "Вес ворса г/м2",
        "Вес ворса гр/м2",
        "Вес Ворса",
    ];
    const TOTAL_WEIGHT: [&str; 3] = ["Общий вес г/м2", "Вес, кг/м2", "Общий вес, гр/м2"];
    const FIRE: &str = "Сертификат";
    const PURCHASE_ROLL_PRICE: [&str; 3] = ["Дилер рулон ", "Дилер рулон", "Рулон, евро/м2"];
    const PURCHASE_COUPON_PRICE: &str = "Дилер купон";
    const RECOMMENDED_COUPON_PRICE: [&str; 3] = ["Розница", "Розница ", "Розница, евро/м2"];

    let is_headers = |r: &[Data]| {
        let mut count = 0;
        for cell in r.iter() {
            let w = cell.to_string();
            if COLLECTION.contains(&w.as_str())
                || WIDTH.contains(&w.as_str())
                || PILE_COMPOSITION.contains(&w.as_str())
                || PILE_HEIGHT == w
                || TOTAL_HEIGHT == w
                || PILE_WEIGHT.contains(&w.as_str())
                || TOTAL_WEIGHT.contains(&w.as_str())
            {
                count += 1;
            }
            if count > 3 {
                return true;
            }
        }
        return false;
    };
    let mut collection_column = usize::MAX;
    let mut width_column = usize::MAX;
    let mut pile_composition_column = usize::MAX;
    let mut pile_height_column = usize::MAX;
    let mut total_height_column = usize::MAX;
    let mut pile_weight_column = usize::MAX;
    let mut total_weight_column = usize::MAX;
    let mut fire_column = usize::MAX;
    let mut purchase_roll_price_column = usize::MAX;
    let mut purchase_coupon_price_column = usize::MAX;
    let mut recommended_coupon_price_column = usize::MAX;
    let mut manufacturer = String::new();
    for row in sheet.rows() {
        if is_headers(row) {
            collection_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| COLLECTION.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            width_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| WIDTH.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            pile_composition_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| PILE_COMPOSITION.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            pile_height_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| *c == PILE_HEIGHT)
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            total_height_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| *c == TOTAL_HEIGHT)
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            pile_weight_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| PILE_WEIGHT.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            total_weight_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| TOTAL_WEIGHT.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            fire_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| *c == FIRE)
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            purchase_roll_price_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| PURCHASE_ROLL_PRICE.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            purchase_coupon_price_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| *c == PURCHASE_COUPON_PRICE)
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
            recommended_coupon_price_column = row
                .iter()
                .enumerate()
                .find(|(_, c)| RECOMMENDED_COUPON_PRICE.contains(&c.to_string().as_str()))
                .map(|(i, _)| i)
                .unwrap_or(usize::MAX);
        } else if row
            .first()
            .is_some_and(|w| MANUFACTURERS.contains(&w.to_string().as_str()))
        {
            manufacturer = row.first().map(|w| w.to_string()).unwrap_or_default();
        } else if collection_column != usize::MAX {
            if row
                .first()
                .is_some_and(|w| MANUFACTURERS.contains(&w.to_string().as_str()))
            {
                manufacturer = row.first().map(|w| w.to_string()).unwrap_or_default();
            }
            let mut price_item = ParsedPriceItem::builder();
            price_item.supplier(SUPPLIER);
            price_item.manufacturer(&manufacturer);
            let collection = row
                .get(collection_column)
                .map(|w| w.to_string())
                .unwrap_or_default();
            price_item.collection(collection);
            row.get(width_column).map(|d| {
                d.to_string()
                    .replace(['м', 'c', 'm', '0', 'c', 'a', '.', ' '], "")
                    .replace(['&', '+'], " ")
                    .split_whitespace()
                    .for_each(|s| {
                        if let Ok(w) = s.parse::<f64>() {
                            price_item.widths(w);
                        }
                    });
            });
            if let Some(pile_composition) = row.get(pile_composition_column).map(|d| d.to_string())
            {
                price_item.pile_composition(&pile_composition);
            }
            if let Some(pile_height) = row
                .get(pile_height_column)
                .and_then(|d| d.to_string().replace(" мм", "").parse().ok())
            {
                price_item.pile_height(pile_height);
            }
            if let Some(total_height) = row
                .get(total_height_column)
                .and_then(|d| d.to_string().replace(" мм", "").parse().ok())
            {
                price_item.total_height(total_height);
            }
            if let Some(pile_weight) = row
                .get(pile_weight_column)
                .and_then(|d| d.to_string().parse().ok())
            {
                price_item.pile_weight(pile_weight);
            }
            if let Some(total_weight) = row
                .get(total_weight_column)
                .and_then(|d| d.to_string().parse().ok())
            {
                price_item.total_weight(total_weight);
            }
            if let Some(fire) = row.get(fire_column).map(|d| d.to_string()) {
                price_item.fire_certificate(&fire);
            }
            if let Some(purchase_roll_price) = row
                .get(purchase_roll_price_column)
                .and_then(|d| d.to_string().replace(',', ".").trim().parse::<f64>().ok())
            {
                price_item.purchase_roll_price(purchase_roll_price);
            }
            if let Some(purchase_coupon_price) = row
                .get(purchase_coupon_price_column)
                .and_then(|d| d.to_string().replace(',', ".").trim().parse::<f64>().ok())
            {
                price_item.purchase_coupon_price(purchase_coupon_price);
            }
            if let Some(recommended_coupon_price) = row
                .get(recommended_coupon_price_column)
                .and_then(|d| d.to_string().replace(',', ".").trim().parse::<f64>().ok())
            {
                price_item.recommended_coupon_price(recommended_coupon_price);
            }
            if let Ok(price) = price_item.build() {
                if let Err(e) = tx.send(price) {
                    tracing::error!("{e:?}");
                }
            }
        }
    }
}
