use fyc_db::DbPool;
use fyc_db::connection::create_pool;
use fyc_db::repositories::UserRepo;
use fyc_sdk::AuthService;
use std::sync::Arc;
use tempfile::TempDir;

fn setup() -> (Arc<TempDir>, DbPool, AuthService) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let pool = create_pool(db_path).expect("Failed to create pool");
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
    assert!(matches!(err, fyc_sdk::SdkError::Database(_)));
}

#[test]
fn test_register_password_too_short() {
    let (_dir, _pool, auth) = setup();
    let err = auth.register("charlie", "short").unwrap_err();
    assert!(matches!(err, fyc_sdk::SdkError::InvalidInput(_)));
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
    assert!(matches!(err, fyc_sdk::SdkError::AuthFailed(_)));
}

#[test]
fn test_login_invalid_username() {
    let (_dir, _pool, auth) = setup();
    let err = auth.login("ghost", "password123").unwrap_err();
    assert!(matches!(err, fyc_sdk::SdkError::AuthFailed(_)));
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
    assert!(matches!(err, fyc_sdk::SdkError::AuthFailed(_)));
}

#[test]
fn test_validate_token_after_deactivate() {
    let (_dir, pool, auth) = setup();
    let user_id = auth.register("heidi", "password123").unwrap();
    let (token, _) = auth.login("heidi", "password123").unwrap();
    let user_repo = UserRepo::new(pool);
    user_repo.deactivate_user(user_id).unwrap();
    let err = auth.validate_token(&token).unwrap_err();
    assert!(matches!(err, fyc_sdk::SdkError::AuthFailed(_)));
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
