use crate::app_state::AppState;
use crate::routes::extract_token;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, get, post, web};
use fyc_sdk::SdkError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub items: Vec<OrderItemRequest>,
}

#[derive(Deserialize)]
pub struct OrderItemRequest {
    pub product_id: i64,
    pub quantity: i32,
}

#[post("/orders")]
async fn create_order(
    state: web::Data<AppState>,
    body: web::Json<CreateOrderRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let items: Vec<(i64, i32)> = body
        .items
        .iter()
        .map(|i| (i.product_id, i.quantity))
        .collect();
    let order_svc = state.order_service();
    match order_svc.create_order(&token, &items) {
        Ok(order_id) => Ok(HttpResponse::Ok().json(serde_json::json!({ "order_id": order_id }))),
        Err(e) => {
            let status = if matches!(e, SdkError::AuthFailed(_)) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_REQUEST
            };
            Ok(HttpResponse::build(status).json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

#[get("/orders/today")]
async fn get_today_orders(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let order_svc = state.order_service();
    match order_svc.get_today_orders(&token) {
        Ok(orders) => Ok(HttpResponse::Ok().json(orders)),
        Err(e) => {
            let status = if matches!(e, SdkError::AuthFailed(_)) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_REQUEST
            };
            Ok(HttpResponse::build(status).json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

#[get("/orders/{id}")]
async fn get_order_detail(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let order_id = path.into_inner();
    let order_svc = state.order_service();
    match order_svc.get_order_detail(&token, order_id) {
        Ok((order, items)) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "order": order,
            "items": items
        }))),
        Err(e) => {
            let status = if matches!(e, SdkError::AuthFailed(_)) {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::BAD_REQUEST
            };
            Ok(HttpResponse::build(status).json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_order)
        .service(get_today_orders)
        .service(get_order_detail);
}
