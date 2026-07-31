use crate::DbPool;
use crate::error::DbError;
use crate::repositories::traits::AuditRepository;
use rusqlite::{ErrorCode, params};

pub struct AuditRepo {
    pool: DbPool,
}

impl AuditRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn log_with_conn(
        conn: &rusqlite::Connection,
        admin_id: i64,
        action: &str,
        target_user_id: Option<i64>,
        details: Option<&str>,
    ) -> Result<(), DbError> {
        if action.trim().is_empty() {
            return Err(DbError::InvalidInput("Action cannot be empty".into()));
        }
        match conn.execute(
            "INSERT INTO audit_log (admin_id, action, target_user_id, details) VALUES (?1, ?2, ?3, ?4)",
            params![admin_id, action.trim(), target_user_id, details],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _)) if err.code == ErrorCode::ConstraintViolation =>
                Err(DbError::ForeignKeyViolation("Admin user does not exist".into())),
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn log(
        &self,
        admin_id: i64,
        action: &str,
        target_user_id: Option<i64>,
        details: Option<&str>,
    ) -> Result<(), DbError> {
        <Self as AuditRepository>::log(self, admin_id, action, target_user_id, details)
    }
}

impl AuditRepository for AuditRepo {
    fn log(
        &self,
        admin_id: i64,
        action: &str,
        target_user_id: Option<i64>,
        details: Option<&str>,
    ) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::log_with_conn(&conn, admin_id, action, target_user_id, details)
    }
}
