use lib_fyc_role::types::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenPayload {
    pub user_id: String,
    pub role: Role,
    pub exp: u64,
}
