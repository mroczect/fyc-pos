use zeroize::ZeroizeOnDrop;

#[derive(Clone, ZeroizeOnDrop)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub public_key: String,
    pub encrypted_private_key: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("password_hash", &"***")
            .field("public_key", &"***")
            .field("encrypted_private_key", &"***")
            .field("is_active", &self.is_active)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct UserRole {
    pub user_id: i64,
    pub role_id: i64,
}

#[derive(Clone, ZeroizeOnDrop)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: String,
    pub expires_at: String,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("token_hash", &"***")
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub id: i64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RolePermission {
    pub role_id: i64,
    pub permission_id: i64,
}

#[derive(Debug, Clone)]
pub struct AuditLog {
    pub id: i64,
    pub admin_id: i64,
    pub action: String,
    pub target_user_id: Option<i64>,
    pub details: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub category: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ProductCustomField {
    pub id: i64,
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone)]
pub struct ProductCustomValue {
    pub id: i64,
    pub product_id: i64,
    pub field_id: i64,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub status: String,
    pub total: f64,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub unit_price: f64,
}
