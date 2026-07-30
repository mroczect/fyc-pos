use crate::error::DbError;
use crate::models::Permission;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct PermissionRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl PermissionRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create(&self, name: &str, description: &str) -> Result<i64, DbError> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput(
                "Permission name cannot be empty".into(),
            ));
        }
        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO permissions (name, description) VALUES (?1, ?2)",
            params![name.trim(), description],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry("Permission already exists".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<Option<Permission>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT id, name, description FROM permissions WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(Permission {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        match rows.next() {
            Some(p) => Ok(Some(p?)),
            None => Ok(None),
        }
    }

    pub fn get_user_permissions(&self, user_id: i64) -> Result<Vec<Permission>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT p.id, p.name, p.description FROM permissions p
             JOIN role_permissions rp ON p.id = rp.permission_id
             JOIN user_roles ur ON rp.role_id = ur.role_id
             WHERE ur.user_id = ?1",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(Permission {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        let mut perms = Vec::new();
        for p in rows {
            perms.push(p?);
        }
        Ok(perms)
    }
}
