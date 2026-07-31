use crate::error::DbError;
use crate::models::*;

pub trait UserRepository {
    fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<i64, DbError>;
    fn find_by_username(&self, username: &str) -> Result<Option<User>, DbError>;
    fn find_by_username_including_inactive(&self, username: &str) -> Result<Option<User>, DbError>;
    fn find_by_id(&self, id: i64) -> Result<Option<User>, DbError>;
    fn find_by_id_including_inactive(&self, id: i64) -> Result<Option<User>, DbError>;
    fn deactivate_user(&self, user_id: i64) -> Result<(), DbError>;
    fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), DbError>;
}

pub trait RoleRepository {
    fn create_role(&self, name: &str, description: &str) -> Result<i64, DbError>;
    fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, DbError>;
    fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> Result<(), DbError>;
    fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, DbError>;
    fn assign_permission_to_role(&self, role_id: i64, permission_id: i64) -> Result<(), DbError>;
    fn remove_permission_from_role(&self, role_id: i64, permission_id: i64) -> Result<(), DbError>;
    fn has_role(&self, user_id: i64, role_name: &str) -> Result<bool, DbError>;
}

pub trait SessionRepository {
    fn create_session(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: &str,
    ) -> Result<i64, DbError>;
    fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), DbError>;
    fn find_valid_session(&self, token_hash: &str) -> Result<Option<Session>, DbError>;
    fn cleanup_expired(&self) -> Result<usize, DbError>;
    fn delete_all_for_user(&self, user_id: i64) -> Result<usize, DbError>;
}

pub trait ProductRepository {
    fn create(&self, name: &str, price: f64, category: &str) -> Result<i64, DbError>;
    fn find_by_id(&self, id: i64) -> Result<Option<Product>, DbError>;
    fn find_all_active(&self) -> Result<Vec<Product>, DbError>;
    fn update(&self, id: i64, name: &str, price: f64, category: &str) -> Result<(), DbError>;
    fn deactivate(&self, id: i64) -> Result<(), DbError>;
}

pub trait ProductCustomRepository {
    fn create_field(&self, name: &str, field_type: &str) -> Result<i64, DbError>;
    fn get_field_by_name(&self, name: &str) -> Result<Option<ProductCustomField>, DbError>;
    fn set_value(&self, product_id: i64, field_id: i64, value: &str) -> Result<(), DbError>;
    fn get_values_for_product(
        &self,
        product_id: i64,
    ) -> Result<Vec<(ProductCustomField, String)>, DbError>;
}

pub trait OrderRepository {
    fn create_order(&self, user_id: i64, status: &str, total: f64) -> Result<i64, DbError>;
    fn add_order_item(
        &self,
        order_id: i64,
        product_id: i64,
        quantity: i32,
        unit_price: f64,
    ) -> Result<i64, DbError>;
    fn get_orders_today(&self) -> Result<Vec<Order>, DbError>;
    fn get_order_items(&self, order_id: i64) -> Result<Vec<OrderItem>, DbError>;
    fn find_order_by_id(&self, order_id: i64) -> Result<Option<Order>, DbError>;
}

pub trait PermissionRepository {
    fn create(&self, name: &str, description: &str) -> Result<i64, DbError>;
    fn get_by_name(&self, name: &str) -> Result<Option<Permission>, DbError>;
    fn get_user_permissions(&self, user_id: i64) -> Result<Vec<Permission>, DbError>;
}

pub trait AuditRepository {
    fn log(
        &self,
        admin_id: i64,
        action: &str,
        target_user_id: Option<i64>,
        details: Option<&str>,
    ) -> Result<(), DbError>;
}
