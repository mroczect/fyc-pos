use crate::error::DbError;
use crate::models::Role;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct RoleRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl RoleRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create_role(&self, name: &str, description: &str) -> Result<i64, DbError> {
        if name.trim().is_empty() {
            return Err(DbError::InvalidInput("Role name cannot be empty".into()));
        }
        if name.len() < 2 || name.len() > 50 {
            return Err(DbError::InvalidInput(
                "Role name must be 2-50 characters".into(),
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == ' ' || c == '-')
        {
            return Err(DbError::InvalidInput(
                "Role name can only contain alphanumeric, underscore, space, or hyphen".into(),
            ));
        }

        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO roles (name, description) VALUES (?1, ?2)",
            params![name, description],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry("Role already exists".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, name, description FROM roles WHERE name = ?1")?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(Role {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        match rows.next() {
            Some(role) => Ok(Some(role?)),
            None => Ok(None),
        }
    }

    pub fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
            params![user_id, role_id],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry(
                    "User already has this role or user/role does not exist".into(),
                ))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT r.id, r.name, r.description FROM roles r
             JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = ?1",
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok(Role {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
            })
        })?;
        let mut roles = Vec::new();
        for role in rows {
            roles.push(role?);
        }
        Ok(roles)
    }

    pub fn has_role(&self, user_id: i64, role_name: &str) -> Result<bool, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = ?1 AND r.name = ?2"
        )?;
        Ok(stmt.exists(params![user_id, role_name])?)
    }
    pub fn assign_permission_to_role(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT OR IGNORE INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)",
            params![role_id, permission_id],
        )?;
        Ok(())
    }

    pub fn remove_permission_from_role(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM role_permissions WHERE role_id = ?1 AND permission_id = ?2",
            params![role_id, permission_id],
        )?;
        Ok(())
    }
}
