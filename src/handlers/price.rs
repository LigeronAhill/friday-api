use std::io::{Cursor, Read};

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{get, post, web, HttpResponse};
use calamine::{open_workbook_auto_from_rs, DataType, Reader};
use serde::Deserialize;

use crate::models::{AppState, PriceItem};

#[derive(Deserialize)]
struct Query {
    search: String,
}

#[get("/api/prices")]
pub async fn get_price(
    state: web::Data<AppState>,
    query: Option<web::Query<Query>>,
) -> HttpResponse {
    match query {
        Some(q) => match state.storage.find_price_item(q.search.clone()).await {
            Ok(r) => HttpResponse::Ok().json(r),
            Err(e) => HttpResponse::InternalServerError().json(e),
        },
        None => match state.storage.read_all_price_items().await {
            Ok(r) => HttpResponse::Ok().json(r),
            Err(e) => HttpResponse::InternalServerError().json(e),
        },
    }
}

#[derive(Debug, MultipartForm)]
pub struct Upload {
    pub file: TempFile,
}
// #[post("/api/prices")]
pub async fn update_prices(
    state: web::Data<AppState>,
    MultipartForm(form): MultipartForm<Upload>,
) -> HttpResponse {
    match parse_form(form) {
        Ok(items) => match state.storage.update_prices(items).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => HttpResponse::InternalServerError().json(e),
        },
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

#[post("/api/price")]
pub async fn update_price(state: web::Data<AppState>, form: web::Form<PriceItem>) -> HttpResponse {
    match state.storage.update_price(form.0).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

fn parse_form(mut form: Upload) -> crate::Result<Vec<PriceItem>> {
    let mut result = Vec::new();
    let mut data = Vec::new();
    form.file
        .file
        .read_to_end(&mut data)
        .map_err(|e| crate::AppError::Custom(e.to_string()))?;
    let cursor = Cursor::new(data);
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|e| crate::AppError::Custom(e.to_string()))?;
    let sheets = workbook.worksheets();
    for (_, table) in sheets {
        for row in table.rows() {
            let supplier = row
                .first()
                .and_then(|d| d.get_string())
                .map(|w| w.to_uppercase())
                .unwrap_or_default();
            if supplier == "SUPPLIER" {
                continue;
            }
            let product_type = row
                .get(1)
                .and_then(|d| d.get_string())
                .map(|w| w.trim().to_uppercase())
                .unwrap_or_default();
            let brand = row
                .get(2)
                .and_then(|d| d.get_string())
                .map(|w| w.trim().to_uppercase())
                .unwrap_or_default();
            let name = row
                .get(3)
                .and_then(|d| d.get_string())
                .map(|w| w.trim().to_uppercase())
                .unwrap_or_default();
            let purchase_price = row
                .get(4)
                .and_then(|d| {
                    d.to_string()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join("")
                        .replace(',', ".")
                        .parse::<f64>()
                        .ok()
                })
                .unwrap_or_default();
            let purchase_price_currency = row
                .get(5)
                .and_then(|d| d.get_string())
                .map(|w| w.trim().to_uppercase())
                .unwrap_or_default();
            let recommended_price = row.get(6).and_then(|d| {
                d.to_string()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join("")
                    .replace(',', ".")
                    .parse::<f64>()
                    .ok()
            });
            let recommended_price_currency = row
                .get(7)
                .and_then(|d| d.get_string())
                .map(|w| w.trim().to_uppercase());
            let colors = row
                .get(8)
                .and_then(|d| d.get_string())
                .map(|s| {
                    s.split(", ")
                        .map(|c| c.trim().to_uppercase())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let widths = row
                .get(9)
                .and_then(|d| d.get_string())
                .map(|s| {
                    s.split(", ")
                        .flat_map(|w| w.parse::<f64>().ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let item = PriceItem {
                supplier,
                product_type,
                brand,
                name,
                purchase_price,
                purchase_price_currency,
                recommended_price,
                recommended_price_currency,
                colors,
                widths,
            };
            result.push(item)
        }
    }
    Ok(result)
}
