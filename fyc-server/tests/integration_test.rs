use actix_web::{App, HttpServer, middleware, web};
use fyc_db::connection::create_pool;
use fyc_server::app_state::AppState;
use fyc_server::routes;
use reqwest::StatusCode;
use serde_json::Value;
use std::path::Path;
use tempfile::TempDir;

async fn spawn_server(db_path: &Path) -> (String, fyc_db::DbPool) {
    let pool = create_pool(db_path).unwrap();
    fyc_sdk::seed_defaults(&pool).unwrap();

    let state = web::Data::new(AppState::new(pool.clone()));

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
            .configure(routes::configure)
    })
    .listen(listener)
    .unwrap()
    .run();

    tokio::spawn(server);

    (format!("http://127.0.0.1:{port}"), pool)
}

fn auth_header(token: &str) -> String {
    format!("Bearer {token}")
}

async fn register_user(base_url: &str, username: &str, password: &str) -> (String, i64) {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/v1/auth/register"))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let user_id = body["user_id"].as_i64().unwrap();

    let resp = client
        .post(format!("{base_url}/v1/auth/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();
    (token, user_id)
}

async fn make_admin(
    pool: &fyc_db::DbPool,
    base_url: &str,
    username: &str,
    password: &str,
) -> (String, i64) {
    let (_token, user_id) = register_user(base_url, username, password).await;
    let role_repo = fyc_db::sqlite::RoleRepo::new(pool.clone());
    let admin_role = role_repo.get_role_by_name("admin").unwrap().unwrap();
    role_repo
        .assign_role_to_user(user_id, admin_role.id)
        .unwrap();
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/v1/auth/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    (body["token"].as_str().unwrap().to_string(), user_id)
}

#[tokio::test]
async fn test_auth_flow() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_auth.db");
    let (base_url, _pool) = spawn_server(&db_path).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/v1/auth/register"))
        .json(&serde_json::json!({
            "username": "alice",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body["user_id"].is_number());

    let resp = client
        .post(format!("{base_url}/v1/auth/register"))
        .json(&serde_json::json!({
            "username": "alice",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let resp = client
        .post(format!("{base_url}/v1/auth/login"))
        .json(&serde_json::json!({
            "username": "alice",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();

    let resp = client
        .get(format!("{base_url}/v1/auth/validate"))
        .header("Authorization", auth_header(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .post(format!("{base_url}/v1/auth/logout"))
        .header("Authorization", auth_header(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .get(format!("{base_url}/v1/auth/validate"))
        .header("Authorization", auth_header(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_menu_crud() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_menu.db");
    let (base_url, pool) = spawn_server(&db_path).await;

    let (admin_token, _) = make_admin(&pool, &base_url, "admin", "admin1234").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "name": "Espresso",
            "price": 25000.0,
            "category": "Kopi"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let product_id = body["id"].as_i64().unwrap();

    let resp = client
        .get(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().len() == 1);

    let resp = client
        .put(format!("{base_url}/v1/menu/products/{product_id}"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "name": "Espresso Large",
            "price": 30000.0,
            "category": "Kopi"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .post(format!("{base_url}/v1/menu/custom-fields"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "name": "size",
            "field_type": "text"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .post(format!(
            "{base_url}/v1/menu/products/{product_id}/custom-values"
        ))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "field_name": "size",
            "value": "Large"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .delete(format!("{base_url}/v1/menu/products/{product_id}"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = client
        .get(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_order_flow() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_order.db");
    let (base_url, pool) = spawn_server(&db_path).await;

    let (admin_token, _) = make_admin(&pool, &base_url, "admin", "admin1234").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "name": "Brownie",
            "price": 15000.0,
            "category": "Makanan"
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let product_id = body["id"].as_i64().unwrap();

    let resp = client
        .post(format!("{base_url}/v1/orders"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "items": [
                {"product_id": product_id, "quantity": 2}
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    let order_id = body["order_id"].as_i64().unwrap();

    let resp = client
        .get(format!("{base_url}/v1/orders/today"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().len() >= 1);

    let resp = client
        .get(format!("{base_url}/v1/orders/{order_id}"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["order"]["total"].as_f64().unwrap(), 30000.0);
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_report_daily_revenue() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_report.db");
    let (base_url, pool) = spawn_server(&db_path).await;

    let (admin_token, _) = make_admin(&pool, &base_url, "admin", "admin1234").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "name": "Kopi Susu",
            "price": 15000.0,
            "category": "Kopi"
        }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let product_id = body["id"].as_i64().unwrap();

    client
        .post(format!("{base_url}/v1/orders"))
        .header("Authorization", auth_header(&admin_token))
        .json(&serde_json::json!({
            "items": [
                {"product_id": product_id, "quantity": 2}
            ]
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{base_url}/v1/reports/daily-revenue"))
        .header("Authorization", auth_header(&admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total_revenue"].as_f64().unwrap(), 30000.0);
}

#[tokio::test]
async fn test_permission_denied() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test_perm.db");
    let (base_url, _pool) = spawn_server(&db_path).await;

    let (kasir_token, _) = register_user(&base_url, "kasir", "password123").await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{base_url}/v1/menu/products"))
        .header("Authorization", auth_header(&kasir_token))
        .json(&serde_json::json!({
            "name": "Espresso",
            "price": 25000.0,
            "category": "Kopi"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
