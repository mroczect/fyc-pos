use crate::app_state::AppState;
use crate::routes::extract_token;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, delete, get, post, put, web};
use fyc_sdk::SdkError;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddProductRequest {
    pub name: String,
    pub price: f64,
    pub category: String,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub name: String,
    pub price: f64,
    pub category: String,
}

#[derive(Deserialize)]
pub struct AddCustomFieldRequest {
    pub name: String,
    pub field_type: String,
}

#[derive(Deserialize)]
pub struct SetCustomValueRequest {
    pub field_name: String,
    pub value: String,
}

#[get("/menu/products")]
async fn list_products(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let menu = state.menu_service();
    match menu.list_products(&token) {
        Ok(products) => Ok(HttpResponse::Ok().json(products)),
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

#[post("/menu/products")]
async fn add_product(
    state: web::Data<AppState>,
    body: web::Json<AddProductRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let menu = state.menu_service();
    match menu.add_product(&token, &body.name, body.price, &body.category) {
        Ok(id) => Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id }))),
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

#[put("/menu/products/{id}")]
async fn update_product(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<UpdateProductRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let id = path.into_inner();
    let menu = state.menu_service();
    match menu.update_product(&token, id, &body.name, body.price, &body.category) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))),
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

#[delete("/menu/products/{id}")]
async fn delete_product(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let id = path.into_inner();
    let menu = state.menu_service();
    match menu.delete_product(&token, id) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))),
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

#[post("/menu/custom-fields")]
async fn add_custom_field(
    state: web::Data<AppState>,
    body: web::Json<AddCustomFieldRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let menu = state.menu_service();
    match menu.add_custom_field(&token, &body.name, &body.field_type) {
        Ok(id) => Ok(HttpResponse::Ok().json(serde_json::json!({ "id": id }))),
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

#[post("/menu/products/{id}/custom-values")]
async fn set_custom_value(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    body: web::Json<SetCustomValueRequest>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let product_id = path.into_inner();
    let menu = state.menu_service();
    match menu.set_custom_value(&token, product_id, &body.field_name, &body.value) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))),
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
    cfg.service(list_products)
        .service(add_product)
        .service(update_product)
        .service(delete_product)
        .service(add_custom_field)
        .service(set_custom_value);
}
