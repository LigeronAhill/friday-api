use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::models::AppState;

#[derive(Deserialize)]
struct Query {
    limit: Option<String>,
    offset: Option<String>,
    search: Option<String>,
}

#[get("/stock")]
pub async fn stock(state: web::Data<AppState>, query: Option<web::Query<Query>>) -> HttpResponse {
    match query {
        Some(q) => {
            if let Some(search) = q.search.to_owned() {
                match state.storage.find_stock(search).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            } else if let Some(limit) = q.limit.to_owned().and_then(|l| l.parse().ok()) {
                let offset = q
                    .offset
                    .to_owned()
                    .and_then(|o| o.parse().ok())
                    .unwrap_or_default();
                match state.storage.get_stock(limit, offset).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            } else {
                match state.storage.get_stock(100, 0).await {
                    Ok(r) => HttpResponse::Ok().json(r),
                    Err(e) => HttpResponse::InternalServerError().json(e),
                }
            }
        }
        None => match state.storage.get_stock(100, 0).await {
            Ok(r) => HttpResponse::Ok().json(r),
            Err(e) => HttpResponse::InternalServerError().json(e),
        },
    }
}
