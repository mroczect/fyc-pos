use lib_fyc_role::types::Role;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub user_id: String,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Zeroizing<String>,
}
