use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use chrono::Days;
use rust_moysklad as ms;
use rust_woocommerce as woo;
use serde::Serialize;

use crate::models::Stock;

pub async fn pause(hours: u64) {
    let secs = 60 * 60 * hours;
    tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
}
pub fn convert_to_create(
    ms_product: &ms::Product,
    ms_data: &MsData,
    woo_data: &WooData,
    stock: &[Stock],
) -> Option<impl Serialize + Clone + Send + Sync + 'static> {
    let sku = ms_product.article.as_ref()?.to_uppercase();
    let quantity = get_quantity(&sku, stock) as i32;
    let country = ms_data
        .countries
        .iter()
        .find(|c| ms_product.country.clone().is_some_and(|m| c.meta == m.meta))
        .map(|f| f.name.clone())
        .unwrap_or(String::from("Россия"));
    let mut uom = ms_data
        .uoms
        .iter()
        .find(|u| ms_product.uom.clone().is_some_and(|m| u.meta == m.meta))
        .map(|f| f.name.clone())
        .unwrap_or(String::from("м2"));
    let mut desc = ms_product.description.clone().unwrap_or_default();
    let ms_product_type = ProductType::from(ms_product);
    if ms_product_type == ProductType::Other {
        return None;
    } else if ms_product_type == ProductType::Carpet || ms_product_type == ProductType::Mat {
        if !desc
            .to_lowercase()
            .contains("цена указана за один квадратный метр")
        {
            desc = format!("Цена указана за один квадратный метр. {desc}")
        }
    } else if ms_product_type == ProductType::CarpetTile {
        desc = desc.replace("Цена указана за один квадратный метр. ", "");
        if !desc
            .to_lowercase()
            .contains("цена указана за одну упаковку")
        {
            desc = format!("Цена указана за одну упаковку. {desc}")
        }
    }
    let category_id = get_category_id(ms_product_type.clone(), woo_data)?;
    let regular_price = get_ms_product_price(ms_product, PriceTag::Regular, ms_data)?;
    let regular_price = format!("{regular_price:.2}");
    let sale_price = get_ms_product_price(ms_product, PriceTag::Sale, ms_data);
    let sale_price = match sale_price {
        Some(s) => {
            if s > 2.0 {
                format!("{s:.2}")
            } else {
                String::new()
            }
        }
        None => String::new(),
    };
    let name = ms_product.name.clone()?;
    let weight = ms_product.weight.unwrap_or_default();
    let ms_attributes = ms_product.attributes.clone()?;
    let mut result = woo::Product::builder();
    result
        .name(name)
        .sku(&sku)
        .categories(category_id)
        .product_type(woo::ProductType::Simple)
        .status(woo::ProductStatus::Publish)
        .catalog_visibility(woo::CatalogVisibility::Visible)
        .short_description(desc)
        .regular_price(regular_price)
        .sale_price(sale_price)
        .backorders(woo::BackordersStatus::Yes)
        .manage_stock()
        .weight(format!("{weight:.2}"))
        .stock_status(woo::StockStatus::Onbackorder)
        .stock_quantity(quantity);
    if let Some(woo_attribute) = woo_data.attributes.get("Страна") {
        let woo_att = woo::Attribute::builder()
            .id(woo_attribute.id)
            .name("Страна")
            .visible()
            .option(country)
            .build();
        result.attribute(woo_att);
    }
    let mut length = 1.0;
    let mut width = 1.0;
    let mut height = 1.0;
    let mut min_quantity = 1.0;
    let mut quantity_step = 1.0;
    for ms_att in ms_attributes {
        let name = ms_att.name.clone();
        let option = match ms_att.value {
            rust_moysklad::AttributeValue::Custom(c) => c.name,
            rust_moysklad::AttributeValue::String(s) => s,
            _ => String::new(),
        };
        if option.is_empty() {
            continue;
        }
        if name == "Ширина рулона, м" {
            let w = option
                .clone()
                .parse::<f64>()
                .map(|d| d * 100.0)
                .unwrap_or(1.0);
            if w != 1.0 {
                min_quantity = w / 100.0 * 2.0;
                quantity_step = 0.1;
                let l = 10000.0 / w;
                width = w;
                length = l;
            }
        } else if name == "Общая толщина, мм" {
            let h = option
                .clone()
                .parse::<f64>()
                .map(|d| d / 10.0)
                .unwrap_or(1.0);

            if ms_product_type.clone() == ProductType::CarpetTile {
                height = 18.0;
                uom = String::from("уп")
            } else {
                height = h;
            }
        } else if name == "Размер плитки, см" {
            let sizes = option
                .clone()
                .split('x')
                .map(String::from)
                .collect::<Vec<_>>();
            if sizes.len() == 2 {
                if let Ok(w) = sizes[0].clone().parse() {
                    width = w;
                }
                if let Ok(l) = sizes[1].clone().parse() {
                    length = l;
                }
            }
        }
        if let Some(woo_attribute) = woo_data.attributes.get(&name) {
            let woo_att = woo::Attribute::builder()
                .id(woo_attribute.id)
                .name(name)
                .visible()
                .option(option)
                .build();
            result.attribute(woo_att);
        }
    }
    let length = format!("{length:.2}");
    let width = format!("{width:.2}");
    let height = format!("{height:.2}");
    result
        .dimensions(width, length, height)
        .meta_data("_woo_uom_input", format!("/{uom}"))
        .meta_data("_alg_wc_pq_min", format!("{min_quantity:.2}"))
        .meta_data("_alg_wc_pq_step", format!("{quantity_step:.2}"));
    let result = result.build();
    Some(result)
}

pub fn convert_to_update(
    ms_product: &ms::Product,
    woo_product: &woo::Product,
    ms_data: &MsData,
    woo_data: &WooData,
    stock: &[Stock],
) -> Option<impl Serialize + Clone + Send + Sync + 'static> {
    if let Some(last_upd) = ms_product.updated {
        let now = chrono::Local::now().naive_local();
        if let Some(yesterday) = now.checked_sub_days(Days::new(1)) {
            if last_upd.le(&yesterday) {
                return None;
            }
        }
        let last_woo_upd = woo_product.date_modified;
        if last_upd.lt(&last_woo_upd) {
            return None;
        }
    }

    let ms_regular_price = get_ms_product_price(ms_product, PriceTag::Regular, ms_data)?;
    let ms_sale_price = get_ms_product_price(ms_product, PriceTag::Sale, ms_data)?;
    let ms_product_type = ProductType::from(ms_product);
    if ms_product_type == ProductType::Other {
        return None;
    }
    let mut uom = ms_data
        .uoms
        .iter()
        .find(|u| ms_product.uom.clone().is_some_and(|m| u.meta == m.meta))
        .map(|f| f.name.clone())
        .unwrap_or(String::from("м2"));
    let ms_attributes = ms_product.attributes.clone()?;
    let name = ms_product.name.as_ref().unwrap_or(&woo_product.name);
    let mut desc = ms_product
        .description
        .clone()
        .unwrap_or(woo_product.short_description.clone());
    let ms_product_type = ProductType::from(ms_product);
    if ms_product_type == ProductType::Other {
        return None;
    } else if ms_product_type == ProductType::Carpet || ms_product_type == ProductType::Mat {
        if !desc
            .to_lowercase()
            .contains("цена указана за один квадратный метр")
        {
            desc = format!("Цена указана за один квадратный метр. {desc}")
        }
    } else if ms_product_type == ProductType::CarpetTile {
        desc = desc.replace("Цена указана за один квадратный метр. ", "");
        if !desc
            .to_lowercase()
            .contains("цена указана за одну упаковку")
        {
            desc = format!("Цена указана за одну упаковку. {desc}")
        }
    }
    let weight = match ms_product.weight {
        Some(w) => format!("{w:.2}"),
        None => woo_product.weight.clone(),
    };

    let country = ms_data
        .countries
        .iter()
        .find(|c| ms_product.country.clone().is_some_and(|m| c.meta == m.meta))
        .map(|f| f.name.clone())
        .unwrap_or(String::from("Россия"));

    let mut result = woo::Product::builder();
    let (status, catalog_visibility) =
        if ms_product.archived.is_some_and(|a| a) || ms_product.archived.is_none() {
            (woo::ProductStatus::Draft, woo::CatalogVisibility::Hidden)
        } else {
            (woo::ProductStatus::Publish, woo::CatalogVisibility::Visible)
        };
    let sku = woo_product.sku.clone();
    let quantity = get_quantity(&sku, stock) as i32;
    result
        .id(woo_product.id)
        .sku(&woo_product.sku)
        .name(name)
        .product_type(woo::ProductType::Simple)
        .status(status)
        .catalog_visibility(catalog_visibility)
        .short_description(desc)
        .regular_price(format!("{ms_regular_price:.2}"))
        .backorders(woo::BackordersStatus::Yes)
        .manage_stock()
        .stock_quantity(quantity)
        .stock_status(woo::StockStatus::Onbackorder)
        .weight(weight);
    for cat in woo_product.categories.iter() {
        result.categories(cat.id);
    }
    if let Some(woo_attribute) = woo_data.attributes.get("Страна") {
        let woo_att = woo::Attribute::builder()
            .id(woo_attribute.id)
            .name("Страна")
            .visible()
            .option(country)
            .build();
        result.attribute(woo_att);
    }
    let mut length = 1.0;
    let mut width = 1.0;
    let mut height = 1.0;
    let mut min_quantity = 1.0;
    let mut quantity_step = 1.0;
    if ms_sale_price > 1.0 {
        result.sale_price(format!("{ms_sale_price:.2}"));
    } else {
        result.sale_price(String::new());
    }
    for ms_att in ms_attributes.clone() {
        let name = ms_att.name.clone();
        let option = match ms_att.value {
            rust_moysklad::AttributeValue::Custom(c) => c.name,
            rust_moysklad::AttributeValue::String(s) => s,
            _ => String::new(),
        };
        if option.is_empty() {
            continue;
        }
        if name == "Ширина рулона, м" {
            let w = option
                .clone()
                .parse::<f64>()
                .map(|d| d * 100.0)
                .unwrap_or(1.0);
            if w != 1.0 {
                min_quantity = w / 100.0 * 2.0;
                quantity_step = 0.1;
                let l = 10000.0 / w;
                width = w;
                length = l;
            }
        } else if name == "Общая толщина, мм" {
            let h = option
                .clone()
                .parse::<f64>()
                .map(|d| d / 10.0)
                .unwrap_or(1.0);

            if ms_product_type.clone() == ProductType::CarpetTile {
                height = 18.0;
                uom = String::from("уп")
            } else {
                height = h;
            }
        } else if name == "Размер плитки, см" {
            let sizes = option
                .clone()
                .replace('х', "x")
                .split('x')
                .map(String::from)
                .collect::<Vec<_>>();
            if sizes.len() == 2 {
                if let Ok(w) = sizes[0].clone().parse() {
                    width = w;
                } else {
                    width = 50.0;
                }
                if let Ok(l) = sizes[1].clone().parse() {
                    length = l;
                } else {
                    length = 50.0;
                }
            }
        }
        if let Some(woo_attribute) = woo_data.attributes.get(&name) {
            let woo_att = woo::Attribute::builder()
                .id(woo_attribute.id)
                .name(name)
                .visible()
                .option(option)
                .build();
            result.attribute(woo_att);
        }
    }
    let length = format!("{length:.2}");
    let width = format!("{width:.2}");
    let height = format!("{height:.2}");
    result
        .dimensions(width, length, height)
        .meta_data("_woo_uom_input", format!("/{uom}"))
        .meta_data("_alg_wc_pq_min", format!("{min_quantity:.2}"))
        .meta_data("_alg_wc_pq_step", format!("{quantity_step:.2}"));
    let result = result.build();
    Some(result)
}
#[derive(Clone)]
pub struct MsData {
    pub currencies: Vec<ms::Currency>,
    pub countries: Vec<ms::Country>,
    pub uoms: Vec<ms::Uom>,
    pub products: HashMap<String, ms::Product>,
}

#[derive(Clone)]
pub struct WooData {
    pub products: HashMap<String, woo::Product>,
    pub attributes: HashMap<String, woo::Attribute>,
    pub categories: HashMap<String, woo::Category>,
}

#[derive(PartialEq, Clone)]
enum ProductType {
    Carpet,
    CarpetTile,
    Rug,
    Mat,
    Other,
}
impl From<&ms::Product> for ProductType {
    fn from(value: &ms::Product) -> Self {
        let ms_path = value
            .path_name
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_default();
        let path = ms_path
            .split('/')
            .collect::<Vec<_>>()
            .first()
            .map(|w| w.to_string())
            .unwrap_or_default();
        match path.as_str() {
            "Ковролин" => Self::Carpet,
            "Ковровая плитка" => Self::CarpetTile,
            "Ковры" => Self::Rug,
            "Циновки" => Self::Mat,
            _ => Self::Other,
        }
    }
}
impl Display for ProductType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ProductType::Carpet => "Ковролин",
            ProductType::CarpetTile => "Ковровая плитка",
            ProductType::Rug => "Ковры",
            ProductType::Mat => "Циновки",
            ProductType::Other => "Invalid",
        };
        write!(f, "{s}")
    }
}
fn get_category_id(product_type: ProductType, woo_data: &WooData) -> Option<i32> {
    woo_data
        .categories
        .get(&product_type.to_string())
        .map(|c| c.id)
}
fn get_ms_product_price(
    ms_product: &ms::Product,
    price_tag: PriceTag,
    ms_data: &MsData,
) -> Option<f64> {
    ms_product
        .sale_prices
        .iter()
        .find(|price| price.price_type.name == price_tag.to_string())
        .map(|price| {
            let currency_id = price.currency.meta.href.clone();
            let currency_id = currency_id
                .split('/')
                .collect::<Vec<_>>()
                .pop()
                .unwrap_or_default();
            let rate = ms_data
                .currencies
                .iter()
                .find(|c| c.id.to_string() == currency_id)
                .map(|c| c.rate)
                .unwrap_or_default();
            price.value * rate / 100.0
        })
}
const REGULAR_PRICE: &str = "Цена продажи";
const SALE_PRICE: &str = "Акция";
enum PriceTag {
    Regular,
    Sale,
}
impl Display for PriceTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regular => write!(f, "{REGULAR_PRICE}"),
            Self::Sale => write!(f, "{SALE_PRICE}"),
        }
    }
}
pub fn get_quantity(sku: &str, stock: &[Stock]) -> f64 {
    let mut temp = stock.to_vec();
    for word in sku.split_whitespace() {
        temp = temp
            .into_iter()
            .filter(|s| {
                s.name
                    .replace(',', ".")
                    .to_uppercase()
                    .contains(&word.replace(',', ".").to_uppercase())
            })
            .collect::<Vec<_>>();
    }
    temp.iter().map(|s| s.stock).sum::<f64>()
}
