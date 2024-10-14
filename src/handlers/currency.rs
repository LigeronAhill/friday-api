use actix_web::{get, web, HttpResponse};

use crate::models::AppState;

#[get("/currencies")]
pub async fn currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.get_latest_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
#[get("/currencies/month")]
pub async fn monthly_currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.storage.get_monthly_currency_rates().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
