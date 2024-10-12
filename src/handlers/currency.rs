use actix_web::{get, web, HttpResponse};

use crate::models::AppState;

#[get("/api/currencies")]
pub async fn currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.get_latest_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
#[get("/api/currencies/month")]
pub async fn monthly_currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.get_monthly_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
