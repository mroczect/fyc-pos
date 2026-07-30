use crate::error::SdkError;
use fyc_db::DbPool;
use fyc_db::repositories::PermissionRepo;

pub struct PermissionService {
    repo: PermissionRepo,
}

impl PermissionService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            repo: PermissionRepo::new(pool),
        }
    }

    pub fn has_permission(&self, user_id: i64, permission: &str) -> Result<bool, SdkError> {
        let perms = self.repo.get_user_permissions(user_id)?;
        Ok(perms.iter().any(|p| p.name == permission))
    }
}
