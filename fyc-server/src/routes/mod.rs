pub mod auth;
pub mod menu;
pub mod order;
pub mod report;

use actix_web::{HttpRequest, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            .configure(auth::configure)
            .configure(menu::configure)
            .configure(order::configure)
            .configure(report::configure),
    );
}

pub fn extract_token(req: &HttpRequest) -> Result<String, actix_web::Error> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or_else(|| {
            actix_web::error::ErrorBadRequest(
                "Missing or malformed Authorization header. Expected: Bearer <token>",
            )
        })
}
