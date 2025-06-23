use std::io::Cursor;

use bytes::Bytes;
use calamine::{open_workbook_auto_from_rs, DataType, Range, Reader};
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use super::ParsedPriceItem;

#[instrument(name = "parsing fox", skip_all)]
pub async fn parse(cursor: Cursor<Bytes>, tx: UnboundedSender<ParsedPriceItem>) {
    if let Ok(mut workbook) = open_workbook_auto_from_rs(cursor) {
        let sheets = workbook.worksheets();
        for (sheet_name, sheet) in sheets {
            tracing::info!("SHEET: {sheet_name}");
            parse_table(&sheet, tx.clone());
        }
    }
}
fn parse_table(table: &Range<calamine::Data>, tx: UnboundedSender<ParsedPriceItem>) {
    match Cols::try_from(table) {
        Ok(cols) => {
            for row in table.rows() {
                let Some(pcp) = row.get(cols.purchase_coupon_price).and_then(price) else {
                    continue;
                };
                let Some(collection) = row.get(cols.collection).map(|d| d.to_string()) else {
                    continue;
                };
                let mut item = ParsedPriceItem::builder();
                item.supplier("БРАТЕЦ ЛИС");
                item.purchase_coupon_price(pcp);
                item.collection(collection);
                item.manufacturer = row.get(cols.manufacturer).map(|d| d.to_string());
                item.widths = row.get(cols.widths).map(widths);
                item.recommended_coupon_price =
                    row.get(cols.recommended_coupon_price).and_then(price);
                item.pile_composition = row.get(cols.pile_composition).map(|d| d.to_string());
                item.pile_height = row.get(cols.pile_height).and_then(price);
                item.total_height = row.get(cols.total_height).and_then(price);
                item.pile_weight = row.get(cols.pile_weight).and_then(weight);
                item.total_weight = row.get(cols.total_weight).and_then(weight);
                item.durability_class = row.get(cols.durability_class).and_then(class);
                item.fire_certificate = row.get(cols.fire_certificate).map(|d| d.to_string());
                match item.build() {
                    Ok(p) => {
                        if let Err(e) = tx.send(p) {
                            tracing::error!("{e:?}");
                        }
                    }
                    Err(e) => tracing::error!("{e:?}"),
                }
            }
        }
        Err(e) => tracing::error!("{e:?}"),
    }
}
fn price(data: &calamine::Data) -> Option<f64> {
    data.to_string()
        .trim()
        .replace([' ', '*', '€'], "")
        .replace(',', ".")
        .parse::<f64>()
        .ok()
}
fn widths(data: &calamine::Data) -> Vec<f64> {
    data.to_string()
        .split_whitespace()
        .flat_map(|w| {
            w.replace("м,", "")
                .replace('м', "")
                .replace(',', ".")
                .parse::<f64>()
                .ok()
        })
        .collect::<Vec<_>>()
}
fn weight(data: &calamine::Data) -> Option<i32> {
    data.to_string()
        .trim()
        .replace(' ', "")
        .replace(',', ".")
        .parse::<i32>()
        .ok()
}
fn class(data: &calamine::Data) -> Option<i32> {
    data.to_string()
        .split('/')
        .last()
        .and_then(|w| w.parse::<i32>().ok())
}
struct Cols {
    manufacturer: usize,
    collection: usize,
    widths: usize,
    pile_composition: usize,
    pile_height: usize,
    total_height: usize,
    pile_weight: usize,
    total_weight: usize,
    durability_class: usize,
    fire_certificate: usize,
    purchase_roll_price: usize,
    purchase_coupon_price: usize,
    recommended_coupon_price: usize,
}
impl TryFrom<&Range<calamine::Data>> for Cols {
    type Error = anyhow::Error;

    fn try_from(value: &Range<calamine::Data>) -> anyhow::Result<Self> {
        let re = regex::Regex::new(r"^[А-Я]+[а-я\s\.,/\d]+$")?;
        let is_headers = |row: &[calamine::Data]| {
            let mut count = 0;
            for cell in row.iter().map(|d| d.to_string()) {
                if re.is_match(&cell) {
                    count += 1;
                }
            }
            return count > 7;
        };
        let mut result = Self {
            manufacturer: usize::MAX,
            collection: usize::MAX,
            widths: usize::MAX,
            pile_composition: usize::MAX,
            pile_height: usize::MAX,
            total_height: usize::MAX,
            pile_weight: usize::MAX,
            total_weight: usize::MAX,
            durability_class: usize::MAX,
            fire_certificate: usize::MAX,
            purchase_roll_price: usize::MAX,
            purchase_coupon_price: usize::MAX,
            recommended_coupon_price: usize::MAX,
        };
        let is_manufacturer = |data: &str| data.to_lowercase().contains("бренд");
        let is_collection = |data: &str| data.to_lowercase().contains("коллекция");
        let is_widths = |data: &str| {
            data.to_lowercase().contains("ширина") || data.to_lowercase().contains("размер")
        };
        let is_pile_composition = |data: &str| data.to_lowercase().contains("нить");
        let is_pile_height = |data: &str| data.to_lowercase().contains("высота ворса");
        let is_total_height = |data: &str| data.to_lowercase().contains("общая высота");
        let is_pile_weight = |data: &str| data.to_lowercase().contains("вес ворса");
        let is_total_weight = |data: &str| data.to_lowercase().contains("общий вес");
        let is_durability = |data: &str| data.to_lowercase().contains("класс");
        let is_fire = |data: &str| data.to_lowercase().contains("сертификат");
        let is_purchase_roll_price = |data: &str| data.to_lowercase().contains("цена рулон");
        let is_purchase_coupon_price = |data: &str| {
            data.to_lowercase().contains("дилер базовая, руб./м2")
                || data.to_lowercase().contains("цена нарезка")
        };
        let is_recommended_coupon_price = |data: &str| {
            data.to_lowercase().contains("ррц") || data.to_lowercase().contains("розница, руб./м2")
        };
        'outer: for row in value.rows() {
            if is_headers(row) {
                for (i, data) in row.iter().map(|d| d.to_string()).enumerate() {
                    if is_manufacturer(&data) {
                        result.manufacturer = i;
                    } else if is_collection(&data) {
                        result.collection = i;
                    } else if is_widths(&data) {
                        result.widths = i;
                    } else if is_pile_composition(&data) {
                        result.pile_composition = i;
                    } else if is_pile_height(&data) {
                        result.pile_height = i;
                    } else if is_total_height(&data) {
                        result.total_height = i;
                    } else if is_pile_weight(&data) {
                        result.pile_weight = i;
                    } else if is_total_weight(&data) {
                        result.total_weight = i;
                    } else if is_durability(&data) {
                        result.durability_class = i;
                    } else if is_fire(&data) {
                        result.fire_certificate = i;
                    } else if is_purchase_roll_price(&data) {
                        result.purchase_roll_price = i;
                    } else if is_purchase_coupon_price(&data) {
                        result.purchase_coupon_price = i;
                    } else if is_recommended_coupon_price(&data) {
                        result.recommended_coupon_price = i;
                    }
                }
                break 'outer;
            }
        }
        if result.purchase_roll_price == usize::MAX
            && result.purchase_coupon_price == usize::MAX
            && result.recommended_coupon_price == usize::MAX
        {
            Err(anyhow::anyhow!("Wrong headers"))
        } else {
            Ok(result)
        }
    }
}

#[instrument(name = "parsing carpet tile", skip_all)]
fn parse_carpet_tile(
    table: calamine::Range<calamine::Data>,
    sender: UnboundedSender<ParsedPriceItem>,
) {
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
