use crate::error::DbError;
use crate::models::User;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct UserRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl UserRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn validate_username(username: &str) -> Result<(), DbError> {
        if username.len() < 3 || username.len() > 30 {
            return Err(DbError::InvalidInput(
                "Username must be 3-30 characters".into(),
            ));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DbError::InvalidInput(
                "Username can only contain alphanumeric and underscore".into(),
            ));
        }
        Ok(())
    }

    pub fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<i64, DbError> {
        Self::validate_username(username)?;
        let conn = self.pool.get()?;
        Self::insert_user(
            &conn,
            username,
            password_hash,
            public_key,
            encrypted_private_key,
        )
    }

    pub fn create_user_with_conn(
        &self,
        conn: &rusqlite::Connection,
        username: &str,
        password_hash: &str,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<i64, DbError> {
        Self::validate_username(username)?;
        Self::insert_user(
            conn,
            username,
            password_hash,
            public_key,
            encrypted_private_key,
        )
    }

    fn insert_user(
        conn: &rusqlite::Connection,
        username: &str,
        password_hash: &str,
        public_key: &str,
        encrypted_private_key: &str,
    ) -> Result<i64, DbError> {
        let result = conn.execute(
            "INSERT INTO users (username, password_hash, public_key, encrypted_private_key) VALUES (?1, ?2, ?3, ?4)",
            params![username, password_hash, public_key, encrypted_private_key],
        );
        match result {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry("Username already exists".into()))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    pub fn find_by_username(&self, username: &str) -> Result<Option<User>, DbError> {
        self.find_user_by_username(username, true)
    }

    pub fn find_by_username_including_inactive(
        &self,
        username: &str,
    ) -> Result<Option<User>, DbError> {
        self.find_user_by_username(username, false)
    }

    pub fn find_by_id(&self, id: i64) -> Result<Option<User>, DbError> {
        self.find_user_by_id(id, true)
    }

    pub fn find_by_id_including_inactive(&self, id: i64) -> Result<Option<User>, DbError> {
        self.find_user_by_id(id, false)
    }

    pub fn deactivate_user(&self, user_id: i64) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE users SET is_active = 0, updated_at = datetime('now') WHERE id = ?1",
            params![user_id],
        )?;
        if affected == 0 {
            return Err(DbError::NotFound("User not found".into()));
        }
        Ok(())
    }

    pub fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        let affected = conn.execute(
            "UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2 AND is_active = 1",
            params![new_password_hash, user_id],
        )?;
        if affected == 0 {
            return Err(DbError::NotFound("Active user not found".into()));
        }
        Ok(())
    }

    fn find_user_by_username(
        &self,
        username: &str,
        active_only: bool,
    ) -> Result<Option<User>, DbError> {
        let conn = self.pool.get()?;
        let query = if active_only {
            "SELECT id, username, password_hash, public_key, encrypted_private_key, is_active, created_at, updated_at FROM users WHERE username = ?1 AND is_active = 1"
        } else {
            "SELECT id, username, password_hash, public_key, encrypted_private_key, is_active, created_at, updated_at FROM users WHERE username = ?1"
        };
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map(params![username], Self::row_to_user)?;
        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    fn find_user_by_id(&self, id: i64, active_only: bool) -> Result<Option<User>, DbError> {
        let conn = self.pool.get()?;
        let query = if active_only {
            "SELECT id, username, password_hash, public_key, encrypted_private_key, is_active, created_at, updated_at FROM users WHERE id = ?1 AND is_active = 1"
        } else {
            "SELECT id, username, password_hash, public_key, encrypted_private_key, is_active, created_at, updated_at FROM users WHERE id = ?1"
        };
        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt.query_map(params![id], Self::row_to_user)?;
        match rows.next() {
            Some(user) => Ok(Some(user?)),
            None => Ok(None),
        }
    }

    fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            public_key: row.get(3)?,
            encrypted_private_key: row.get(4)?,
            is_active: row.get::<_, i32>(5)? == 1,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}
