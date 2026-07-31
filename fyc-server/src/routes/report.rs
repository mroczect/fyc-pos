use crate::app_state::AppState;
use crate::routes::extract_token;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, get, web};
use fyc_sdk::SdkError;

#[get("/reports/daily-revenue")]
async fn daily_revenue(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let token = extract_token(&req)?;
    let report_svc = state.report_service();
    match report_svc.daily_revenue(&token) {
        Ok(total) => Ok(HttpResponse::Ok().json(serde_json::json!({ "total_revenue": total }))),
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
    cfg.service(daily_revenue);
}
