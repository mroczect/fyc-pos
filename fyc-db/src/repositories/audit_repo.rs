use crate::error::DbError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct AuditRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl AuditRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn log(
        &self,
        admin_id: i64,
        action: &str,
        target_user_id: Option<i64>,
        details: Option<&str>,
    ) -> Result<(), DbError> {
        if action.trim().is_empty() {
            return Err(DbError::InvalidInput("Action cannot be empty".into()));
        }
        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO audit_log (admin_id, action, target_user_id, details) VALUES (?1, ?2, ?3, ?4)",
            params![admin_id, action.trim(), target_user_id, details],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::InvalidInput("Admin user does not exist".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }
}
