use fyc_db::DbPool;
use fyc_db::connection::create_pool;
use fyc_db::sqlite::{RoleRepo, UserRepo};
use fyc_sdk::*;
use std::sync::Arc;
use tempfile::TempDir;

fn setup() -> (Arc<TempDir>, DbPool, AuthService) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let pool = create_pool(db_path).expect("Failed to create pool");
    let auth = AuthService::new(pool.clone());
    (Arc::new(dir), pool, auth)
}

fn setup_with_seed() -> (Arc<TempDir>, DbPool, AuthService) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let pool = create_pool(db_path).expect("Failed to create pool");
    seed_defaults(&pool).expect("Failed to seed defaults");
    let auth = AuthService::new(pool.clone());
    (Arc::new(dir), pool, auth)
}

#[test]
fn test_register_success() {
    let (_dir, _pool, auth) = setup();
    let user_id = auth
        .register("alice", "password123")
        .expect("Failed to register");
    assert!(user_id > 0);
    assert!(auth.user_has_role(user_id, "kasir").unwrap());
}

#[test]
fn test_register_duplicate_user() {
    let (_dir, _pool, auth) = setup();
    auth.register("bob", "password123").unwrap();
    let err = auth.register("bob", "password123").unwrap_err();
    assert!(matches!(err, SdkError::Database(_)));
}

#[test]
fn test_register_password_too_short() {
    let (_dir, _pool, auth) = setup();
    let err = auth.register("charlie", "short").unwrap_err();
    assert!(matches!(err, SdkError::InvalidInput(_)));
}

#[test]
fn test_login_success() {
    let (_dir, _pool, auth) = setup();
    auth.register("dave", "password123").unwrap();
    let (token, user_id) = auth.login("dave", "password123").unwrap();
    assert!(!token.is_empty());
    assert!(user_id > 0);
}

#[test]
fn test_login_wrong_password() {
    let (_dir, _pool, auth) = setup();
    auth.register("eve", "password123").unwrap();
    let err = auth.login("eve", "wrongpassword").unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_login_invalid_username() {
    let (_dir, _pool, auth) = setup();
    let err = auth.login("ghost", "password123").unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_logout() {
    let (_dir, _pool, auth) = setup();
    auth.register("frank", "password123").unwrap();
    let (token, _) = auth.login("frank", "password123").unwrap();
    auth.logout(&token).unwrap();
    assert!(auth.validate_token(&token).is_err());
}

#[test]
fn test_validate_token_success() {
    let (_dir, _pool, auth) = setup();
    auth.register("grace", "password123").unwrap();
    let (token, user_id) = auth.login("grace", "password123").unwrap();
    let validated = auth.validate_token(&token).unwrap();
    assert_eq!(validated, user_id);
}

#[test]
fn test_validate_token_invalid() {
    let (_dir, _pool, auth) = setup();
    let err = auth.validate_token("nonexistent").unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_validate_token_after_deactivate() {
    let (_dir, pool, auth) = setup();
    let user_id = auth.register("heidi", "password123").unwrap();
    let (token, _) = auth.login("heidi", "password123").unwrap();
    UserRepo::new(pool).deactivate_user(user_id).unwrap();
    let err = auth.validate_token(&token).unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_user_has_role_true() {
    let (_dir, _pool, auth) = setup();
    let user_id = auth.register("ivan", "password123").unwrap();
    assert!(auth.user_has_role(user_id, "kasir").unwrap());
}

#[test]
fn test_user_has_role_false() {
    let (_dir, _pool, auth) = setup();
    let user_id = auth.register("judy", "password123").unwrap();
    assert!(!auth.user_has_role(user_id, "admin").unwrap());
}

#[test]
fn test_login_clears_old_sessions() {
    let (_dir, _pool, auth) = setup();
    auth.register("karl", "password123").unwrap();
    let (token1, _) = auth.login("karl", "password123").unwrap();
    let (token2, _) = auth.login("karl", "password123").unwrap();
    assert!(auth.validate_token(&token1).is_err());
    assert!(auth.validate_token(&token2).is_ok());
}

fn make_admin(pool: &DbPool, username: &str, password: &str) -> (AuthService, String) {
    let auth = AuthService::new(pool.clone());
    let user_id = auth.register(username, password).unwrap();
    let role_repo = RoleRepo::new(pool.clone());
    let admin_role = role_repo.get_role_by_name("admin").unwrap().unwrap();
    role_repo
        .assign_role_to_user(user_id, admin_role.id)
        .unwrap();
    let (token, _) = auth.login(username, password).unwrap();
    (auth, token)
}

#[test]
fn test_menu_add_product_requires_permission() {
    let (_dir, pool, _auth) = setup_with_seed();
    let menu = MenuService::new(pool.clone());
    let (_, kasir_token) = {
        let auth = AuthService::new(pool.clone());
        auth.register("kasir", "password123").unwrap();
        let (t, _) = auth.login("kasir", "password123").unwrap();
        (auth, t)
    };
    let err = menu
        .add_product(&kasir_token, "Espresso", 25000.0, "Kopi")
        .unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_menu_add_product_success() {
    let (_dir, pool, _auth) = setup_with_seed();
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "adminuser", "admin1234");
    let id = menu
        .add_product(&admin_token, "Espresso", 25000.0, "Kopi")
        .unwrap();
    assert!(id > 0);
}

#[test]
fn test_menu_list_products() {
    let (_dir, pool, _auth) = setup_with_seed();
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin2", "admin1234");
    menu.add_product(&admin_token, "Latte", 20000.0, "Kopi")
        .unwrap();
    let products = menu.list_products(&admin_token).unwrap();
    assert_eq!(products.len(), 1);
}

#[test]
fn test_menu_custom_field() {
    let (_dir, pool, _auth) = setup_with_seed();
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin3", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Cappuccino", 22000.0, "Kopi")
        .unwrap();
    let _field_id = menu.add_custom_field(&admin_token, "size", "text").unwrap();
    menu.set_custom_value(&admin_token, prod_id, "size", "Large")
        .unwrap();
    let vals = menu
        .get_product_custom_values(&admin_token, prod_id)
        .unwrap();
    assert_eq!(vals.len(), 1);
    assert_eq!(vals[0].0.name, "size");
    assert_eq!(vals[0].1, "Large");
}

#[test]
fn test_create_order_requires_permission() {
    let (_dir, pool, _auth) = setup_with_seed();
    let order_svc = OrderService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin4", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Brownie", 15000.0, "Makanan")
        .unwrap();
    let (_, kasir_token) = {
        let auth = AuthService::new(pool.clone());
        auth.register("kasir2", "password123").unwrap();
        let (t, _) = auth.login("kasir2", "password123").unwrap();
        (auth, t)
    };
    let err = order_svc
        .create_order(&kasir_token, &[(prod_id, 1)])
        .unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}

#[test]
fn test_create_order_success() {
    let (_dir, pool, _auth) = setup_with_seed();
    let order_svc = OrderService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin5", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Croissant", 18000.0, "Makanan")
        .unwrap();
    let order_id = order_svc
        .create_order(&admin_token, &[(prod_id, 2)])
        .unwrap();
    assert!(order_id > 0);
}

#[test]
fn test_get_today_orders() {
    let (_dir, pool, _auth) = setup_with_seed();
    let order_svc = OrderService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin6", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Teh", 5000.0, "Minuman")
        .unwrap();
    order_svc
        .create_order(&admin_token, &[(prod_id, 1)])
        .unwrap();
    let orders = order_svc.get_today_orders(&admin_token).unwrap();
    assert!(!orders.is_empty());
}

#[test]
fn test_get_order_detail() {
    let (_dir, pool, _auth) = setup_with_seed();
    let order_svc = OrderService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin7", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Espresso", 25000.0, "Kopi")
        .unwrap();
    let order_id = order_svc
        .create_order(&admin_token, &[(prod_id, 1)])
        .unwrap();
    let (order, items) = order_svc.get_order_detail(&admin_token, order_id).unwrap();
    assert_eq!(order.total, 25000.0);
    assert_eq!(items.len(), 1);
}

#[test]
fn test_daily_revenue() {
    let (_dir, pool, _auth) = setup_with_seed();
    let report_svc = ReportService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let order_svc = OrderService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool, "admin8", "admin1234");
    let prod_id = menu
        .add_product(&admin_token, "Kopi Susu", 15000.0, "Kopi")
        .unwrap();
    order_svc
        .create_order(&admin_token, &[(prod_id, 2)])
        .unwrap();
    let revenue = report_svc.daily_revenue(&admin_token).unwrap();
    assert_eq!(revenue, 30000.0);
}

#[test]
fn test_report_requires_permission() {
    let (_dir, pool, _auth) = setup_with_seed();
    let report_svc = ReportService::new(pool.clone());
    let (_, kasir_token) = {
        let auth = AuthService::new(pool.clone());
        auth.register("kasir3", "password123").unwrap();
        let (t, _) = auth.login("kasir3", "password123").unwrap();
        (auth, t)
    };
    let err = report_svc.daily_revenue(&kasir_token).unwrap_err();
    assert!(matches!(err, SdkError::AuthFailed(_)));
}
