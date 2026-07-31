use fyc_db::DbError;
use fyc_db::connection::create_pool;
use fyc_db::sqlite::{
    AuditRepo, OrderRepo, PermissionRepo, ProductCustomRepo, ProductRepo, RoleRepo, SessionRepo,
    UserRepo,
};
use std::sync::Arc;
use tempfile::TempDir;

fn setup() -> (Arc<TempDir>, fyc_db::DbPool) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let pool = create_pool(db_path).expect("Failed to create pool");
    (Arc::new(dir), pool)
}

#[test]
fn test_create_user_success() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    let id = user_repo
        .create_user("john_doe", "hash123", "pubkey", "enc_privkey")
        .expect("Failed to create user");
    assert!(id > 0);
    let user = user_repo.find_by_username("john_doe").unwrap().unwrap();
    assert!(user.is_active);
}

#[test]
fn test_create_user_duplicate() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    user_repo.create_user("alice", "hash", "pk", "ek").unwrap();
    let err = user_repo
        .create_user("alice", "hash", "pk", "ek")
        .unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_create_user_invalid_username() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    let err = user_repo.create_user("ab", "hash", "pk", "ek").unwrap_err();
    assert!(matches!(err, DbError::InvalidInput(_)));
}

#[test]
fn test_find_by_username_not_found() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    assert!(user_repo.find_by_username("nobody").unwrap().is_none());
}

#[test]
fn test_deactivate_user() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    let id = user_repo.create_user("bob", "hash", "pk", "ek").unwrap();
    user_repo.deactivate_user(id).unwrap();
    assert!(user_repo.find_by_username("bob").unwrap().is_none());
    let user = user_repo
        .find_by_username_including_inactive("bob")
        .unwrap()
        .unwrap();
    assert!(!user.is_active);
}

#[test]
fn test_update_password_active_user() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    let id = user_repo
        .create_user("charlie", "oldhash", "pk", "ek")
        .unwrap();
    user_repo.update_password(id, "newhash").unwrap();
    assert_eq!(
        user_repo
            .find_by_username("charlie")
            .unwrap()
            .unwrap()
            .password_hash,
        "newhash"
    );
}

#[test]
fn test_update_password_inactive_user() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool);
    let id = user_repo.create_user("dave", "hash", "pk", "ek").unwrap();
    user_repo.deactivate_user(id).unwrap();
    let err = user_repo.update_password(id, "newhash").unwrap_err();
    assert!(matches!(err, DbError::NotFound(_)));
}

#[test]
fn test_create_role_success() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool);
    let id = role_repo.create_role("admin", "Administrator").unwrap();
    assert!(id > 0);
    assert_eq!(
        role_repo.get_role_by_name("admin").unwrap().unwrap().name,
        "admin"
    );
}

#[test]
fn test_create_role_duplicate() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool);
    role_repo.create_role("kasir", "Cashier").unwrap();
    let err = role_repo.create_role("kasir", "Cashier").unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_create_role_invalid_name() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool);
    let err = role_repo.create_role("", "desc").unwrap_err();
    assert!(matches!(err, DbError::InvalidInput(_)));
}

#[test]
fn test_assign_role_to_user_success() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let role_repo = RoleRepo::new(pool.clone());
    let user_id = user_repo.create_user("eve", "hash", "pk", "ek").unwrap();
    let role_id = role_repo.create_role("manager", "Manager").unwrap();
    role_repo.assign_role_to_user(user_id, role_id).unwrap();
    let roles = role_repo.get_user_roles(user_id).unwrap();
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].name, "manager");
}

#[test]
fn test_assign_role_duplicate() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let role_repo = RoleRepo::new(pool.clone());
    let user_id = user_repo.create_user("frank", "hash", "pk", "ek").unwrap();
    let role_id = role_repo.create_role("staff", "Staff").unwrap();
    role_repo.assign_role_to_user(user_id, role_id).unwrap();
    let err = role_repo.assign_role_to_user(user_id, role_id).unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_assign_role_nonexistent_user() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool.clone());
    let role_id = role_repo.create_role("temp", "Temp").unwrap();
    let err = role_repo.assign_role_to_user(9999, role_id).unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_session_lifecycle() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let session_repo = SessionRepo::new(pool);
    let user_id = user_repo
        .create_user("sessionuser", "hash", "pk", "ek")
        .unwrap();
    let token_hash = "abcdef123456";
    let expires = "2099-12-31 23:59:59";
    let session_id = session_repo
        .create_session(user_id, token_hash, expires)
        .unwrap();
    assert!(session_id > 0);
    let session = session_repo
        .find_valid_session(token_hash)
        .unwrap()
        .unwrap();
    assert_eq!(session.user_id, user_id);
    session_repo
        .delete_session_by_token_hash(token_hash)
        .unwrap();
    assert!(
        session_repo
            .find_valid_session(token_hash)
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_session_cleanup_expired() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let session_repo = SessionRepo::new(pool);
    let user_id = user_repo
        .create_user("cleanupuser", "hash", "pk", "ek")
        .unwrap();
    session_repo
        .create_session(user_id, "expiredhash", "2000-01-01 00:00:00")
        .unwrap();
    let deleted = session_repo.cleanup_expired().unwrap();
    assert_eq!(deleted, 1);
    assert!(
        session_repo
            .find_valid_session("expiredhash")
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_session_duplicate_token_hash() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let session_repo = SessionRepo::new(pool);
    let user_id = user_repo
        .create_user("dupsession", "hash", "pk", "ek")
        .unwrap();
    session_repo
        .create_session(user_id, "samehash", "2099-01-01 00:00:00")
        .unwrap();
    let err = session_repo
        .create_session(user_id, "samehash", "2099-01-01 00:00:00")
        .unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_create_permission() {
    let (_dir, pool) = setup();
    let perm_repo = PermissionRepo::new(pool);
    let id = perm_repo.create("user:create", "Create user").unwrap();
    assert!(id > 0);
    assert_eq!(
        perm_repo.get_by_name("user:create").unwrap().unwrap().name,
        "user:create"
    );
}

#[test]
fn test_assign_permission_to_role_and_check_user() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let role_repo = RoleRepo::new(pool.clone());
    let perm_repo = PermissionRepo::new(pool.clone());
    let user_id = user_repo
        .create_user("permuser", "hash", "pk", "ek")
        .unwrap();
    let role_id = role_repo.create_role("tester", "Tester role").unwrap();
    let perm_id = perm_repo.create("test:perm", "Test permission").unwrap();
    role_repo.assign_role_to_user(user_id, role_id).unwrap();
    role_repo
        .assign_permission_to_role(role_id, perm_id)
        .unwrap();
    let perms = perm_repo.get_user_permissions(user_id).unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms[0].name, "test:perm");
}

#[test]
fn test_remove_permission_from_role() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool.clone());
    let perm_repo = PermissionRepo::new(pool.clone());
    let role_id = role_repo.create_role("remover", "Remover role").unwrap();
    let perm_id = perm_repo.create("temp:perm", "Temp").unwrap();
    role_repo
        .assign_permission_to_role(role_id, perm_id)
        .unwrap();
    role_repo
        .remove_permission_from_role(role_id, perm_id)
        .unwrap();
    let user_repo = UserRepo::new(pool.clone());
    let user_id = user_repo
        .create_user("removeuser", "hash", "pk", "ek")
        .unwrap();
    role_repo.assign_role_to_user(user_id, role_id).unwrap();
    let perms = perm_repo.get_user_permissions(user_id).unwrap();
    assert!(perms.is_empty());
}

#[test]
fn test_audit_log() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let audit_repo = AuditRepo::new(pool.clone());
    let admin_id = user_repo.create_user("admin", "hash", "pk", "ek").unwrap();
    audit_repo
        .log(admin_id, "user:create", Some(admin_id), Some("test user"))
        .unwrap();
    let conn = pool.get().unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM audit_log WHERE admin_id = ?1",
            rusqlite::params![admin_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_foreign_key_constraint_on_assign() {
    let (_dir, pool) = setup();
    let role_repo = RoleRepo::new(pool.clone());
    let role_id = role_repo.create_role("testrole", "Test").unwrap();
    let err = role_repo.assign_role_to_user(99999, role_id).unwrap_err();
    assert!(matches!(err, DbError::DuplicateEntry(_)));
}

#[test]
fn test_cascade_delete_user() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let role_repo = RoleRepo::new(pool.clone());
    let session_repo = SessionRepo::new(pool.clone());
    let user_id = user_repo
        .create_user("cascadeuser", "hash", "pk", "ek")
        .unwrap();
    let role_id = role_repo.create_role("cascade_role", "Test").unwrap();
    role_repo.assign_role_to_user(user_id, role_id).unwrap();
    session_repo
        .create_session(user_id, "cascade_token", "2099-01-01 00:00:00")
        .unwrap();
    let conn = pool.get().unwrap();
    conn.execute(
        "DELETE FROM users WHERE id = ?1",
        rusqlite::params![user_id],
    )
    .unwrap();
    let count_roles: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM user_roles WHERE user_id = ?1",
            rusqlite::params![user_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count_roles, 0);
    let count_sessions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE user_id = ?1",
            rusqlite::params![user_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count_sessions, 0);
}

#[test]
fn test_create_product_success() {
    let (_dir, pool) = setup();
    let product_repo = ProductRepo::new(pool);
    let id = product_repo.create("Espresso", 25000.0, "Kopi").unwrap();
    assert!(id > 0);
    let product = product_repo.find_by_id(id).unwrap().unwrap();
    assert_eq!(product.name, "Espresso");
    assert_eq!(product.price, 25000.0);
}

#[test]
fn test_create_product_invalid_price() {
    let (_dir, pool) = setup();
    let product_repo = ProductRepo::new(pool);
    let err = product_repo.create("Test", -5.0, "").unwrap_err();
    assert!(matches!(err, DbError::InvalidInput(_)));
}

#[test]
fn test_update_product() {
    let (_dir, pool) = setup();
    let product_repo = ProductRepo::new(pool);
    let id = product_repo.create("Latte", 20000.0, "Kopi").unwrap();
    product_repo
        .update(id, "Latte Large", 25000.0, "Kopi")
        .unwrap();
    let updated = product_repo.find_by_id(id).unwrap().unwrap();
    assert_eq!(updated.name, "Latte Large");
    assert_eq!(updated.price, 25000.0);
}

#[test]
fn test_deactivate_product() {
    let (_dir, pool) = setup();
    let product_repo = ProductRepo::new(pool);
    let id = product_repo.create("Mocha", 30000.0, "Kopi").unwrap();
    product_repo.deactivate(id).unwrap();
    assert!(product_repo.find_by_id(id).unwrap().is_none());
    let all = product_repo.find_all_active().unwrap();
    assert!(!all.iter().any(|p| p.id == id));
}

#[test]
fn test_custom_field() {
    let (_dir, pool) = setup();
    let custom_repo = ProductCustomRepo::new(pool.clone());
    let product_repo = ProductRepo::new(pool);
    let field_id = custom_repo.create_field("size", "text").unwrap();
    let product_id = product_repo.create("Cappuccino", 22000.0, "Kopi").unwrap();
    custom_repo
        .set_value(product_id, field_id, "Large")
        .unwrap();
    let values = custom_repo.get_values_for_product(product_id).unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].0.name, "size");
    assert_eq!(values[0].1, "Large");
}

#[test]
fn test_create_order() {
    let (_dir, pool) = setup();
    let user_repo = UserRepo::new(pool.clone());
    let product_repo = ProductRepo::new(pool.clone());
    let order_repo = OrderRepo::new(pool);
    let user_id = user_repo
        .create_user("orderuser", "hash", "pk", "ek")
        .unwrap();
    let prod_id = product_repo.create("Brownie", 15000.0, "Makanan").unwrap();
    let order_id = order_repo.create_order(user_id, "paid", 15000.0).unwrap();
    order_repo
        .add_order_item(order_id, prod_id, 2, 15000.0)
        .unwrap();
    let items = order_repo.get_order_items(order_id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].quantity, 2);
    let orders_today = order_repo.get_orders_today().unwrap();
    assert!(orders_today.iter().any(|o| o.id == order_id));
}
