use actix_web::{get, web, HttpResponse};

use crate::models::AppState;

#[get("/currencies/{char_code}")]
pub async fn currency(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    let char_code = path.into_inner().to_uppercase();
    match state.currency_service.find_currency(&char_code).await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
#[get("/currencies")]
pub async fn currencies(state: web::Data<AppState>) -> HttpResponse {
    match state.currency_service.get_currencies().await {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}
