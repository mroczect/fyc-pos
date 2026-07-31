use crate::DbPool;
use crate::error::DbError;
use crate::models::Role;
use crate::repositories::traits::RoleRepository;
use crate::validation;
use rusqlite::{ErrorCode, params};

pub struct RoleRepo {
    pool: DbPool,
}

impl RoleRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_role_with_conn(
        conn: &rusqlite::Connection,
        name: &str,
        description: &str,
    ) -> Result<i64, DbError> {
        validation::validate_role_name(name)?;
        Self::insert_role(conn, name, description)
    }

    pub fn get_role_by_name_with_conn(
        conn: &rusqlite::Connection,
        name: &str,
    ) -> Result<Option<Role>, DbError> {
        Self::query_role_by_name(conn, name)
    }

    pub fn assign_role_to_user_with_conn(
        conn: &rusqlite::Connection,
        user_id: i64,
        role_id: i64,
    ) -> Result<(), DbError> {
        Self::insert_user_role(conn, user_id, role_id)
    }

    pub fn insert_role(
        conn: &rusqlite::Connection,
        name: &str,
        description: &str,
    ) -> Result<i64, DbError> {
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

    pub fn insert_user_role(
        conn: &rusqlite::Connection,
        user_id: i64,
        role_id: i64,
    ) -> Result<(), DbError> {
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

    pub fn create_role(&self, name: &str, description: &str) -> Result<i64, DbError> {
        <Self as RoleRepository>::create_role(self, name, description)
    }

    pub fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        <Self as RoleRepository>::get_role_by_name(self, name)
    }

    pub fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> Result<(), DbError> {
        <Self as RoleRepository>::assign_role_to_user(self, user_id, role_id)
    }

    pub fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, DbError> {
        <Self as RoleRepository>::get_user_roles(self, user_id)
    }

    pub fn assign_permission_to_role(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> Result<(), DbError> {
        <Self as RoleRepository>::assign_permission_to_role(self, role_id, permission_id)
    }

    pub fn remove_permission_from_role(
        &self,
        role_id: i64,
        permission_id: i64,
    ) -> Result<(), DbError> {
        <Self as RoleRepository>::remove_permission_from_role(self, role_id, permission_id)
    }

    pub fn has_role(&self, user_id: i64, role_name: &str) -> Result<bool, DbError> {
        <Self as RoleRepository>::has_role(self, user_id, role_name)
    }

    pub fn query_role_by_name(
        conn: &rusqlite::Connection,
        name: &str,
    ) -> Result<Option<Role>, DbError> {
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

    pub fn query_user_roles(
        conn: &rusqlite::Connection,
        user_id: i64,
    ) -> Result<Vec<Role>, DbError> {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.name, r.description FROM roles r JOIN user_roles ur ON r.id = ur.role_id WHERE ur.user_id = ?1"
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

    pub fn insert_role_permission(
        conn: &rusqlite::Connection,
        role_id: i64,
        permission_id: i64,
    ) -> Result<(), DbError> {
        match conn.execute(
            "INSERT INTO role_permissions (role_id, permission_id) VALUES (?1, ?2)",
            params![role_id, permission_id],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry(
                    "Permission already assigned or role/permission does not exist".into(),
                ))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }
}

impl RoleRepository for RoleRepo {
    fn create_role(&self, name: &str, description: &str) -> Result<i64, DbError> {
        validation::validate_role_name(name)?;
        let conn = self.pool.get()?;
        Self::insert_role(&conn, name, description)
    }

    fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, DbError> {
        let conn = self.pool.get()?;
        Self::query_role_by_name(&conn, name)
    }

    fn assign_role_to_user(&self, user_id: i64, role_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::insert_user_role(&conn, user_id, role_id)
    }

    fn get_user_roles(&self, user_id: i64) -> Result<Vec<Role>, DbError> {
        let conn = self.pool.get()?;
        Self::query_user_roles(&conn, user_id)
    }

    fn assign_permission_to_role(&self, role_id: i64, permission_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        Self::insert_role_permission(&conn, role_id, permission_id)
    }

    fn remove_permission_from_role(&self, role_id: i64, permission_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "DELETE FROM role_permissions WHERE role_id = ?1 AND permission_id = ?2",
            params![role_id, permission_id],
        )?;
        if affected == 0 {
            Err(DbError::NotFound("Role or permission not found".into()))
        } else {
            Ok(())
        }
    }

    fn has_role(&self, user_id: i64, role_name: &str) -> Result<bool, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = ?1 AND r.name = ?2")?;
        Ok(stmt.exists(params![user_id, role_name])?)
    }
}
