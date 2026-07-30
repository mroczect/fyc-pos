use crate::auth::AuthService;
use crate::error::SdkError;
use crate::permission::PermissionService;
use fyc_db::{DbError, DbPool};

pub struct ReportService {
    pool: DbPool,
    auth: AuthService,
    permission: PermissionService,
}

impl ReportService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            auth: AuthService::new(pool.clone()),
            permission: PermissionService::new(pool.clone()),
            pool,
        }
    }

    pub fn daily_revenue(&self, token: &str) -> Result<f64, SdkError> {
        let user_id = self.auth.validate_token(token)?;
        if !self.permission.has_permission(user_id, "report:view")? {
            return Err(SdkError::AuthFailed(
                "Missing permission: report:view".into(),
            ));
        }
        let conn = self.pool.get().map_err(DbError::from)?;
        let total: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(total), 0) FROM orders WHERE date(created_at) = date('now') AND status = 'paid'",
                [],
                |row| row.get(0),
            )
            .map_err(DbError::from)?;
        Ok(total)
    }
}
