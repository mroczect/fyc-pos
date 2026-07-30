use zeroize::ZeroizeOnDrop;

#[derive(Debug, Clone, ZeroizeOnDrop)]
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

#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub created_at: String,
    pub expires_at: String,
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
