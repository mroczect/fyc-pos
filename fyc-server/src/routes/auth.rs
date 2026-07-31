use crate::app_state::AppState;
use crate::routes::extract_token;
use actix_web::{HttpResponse, get, post, web};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/auth/register")]
async fn register(state: web::Data<AppState>, body: web::Json<RegisterRequest>) -> HttpResponse {
    let auth = state.auth_service();
    match auth.register(&body.username, &body.password) {
        Ok(user_id) => HttpResponse::Ok().json(serde_json::json!({ "user_id": user_id })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[post("/auth/login")]
async fn login(state: web::Data<AppState>, body: web::Json<LoginRequest>) -> HttpResponse {
    let auth = state.auth_service();
    match auth.login(&body.username, &body.password) {
        Ok((token, user_id)) => HttpResponse::Ok().json(serde_json::json!({
            "token": token,
            "user_id": user_id
        })),
        Err(e) => HttpResponse::Unauthorized().json(serde_json::json!({ "error": e.to_string() })),
    }
}

#[post("/auth/logout")]
async fn logout(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let auth = state.auth_service();
    match auth.logout(&token) {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))),
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

#[get("/auth/validate")]
async fn validate(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let auth = state.auth_service();
    match auth.validate_token(&token) {
        Ok(user_id) => Ok(HttpResponse::Ok().json(serde_json::json!({ "user_id": user_id }))),
        Err(e) => {
            Ok(HttpResponse::Unauthorized().json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(logout)
        .service(validate);
}
