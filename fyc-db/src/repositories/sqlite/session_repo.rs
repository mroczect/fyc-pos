use crate::DbPool;
use crate::error::DbError;
use crate::models::Session;
use crate::repositories::traits::SessionRepository;
use regex::Regex;
use rusqlite::{ErrorCode, params};

pub struct SessionRepo {
    pool: DbPool,
}

impl SessionRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    fn validate_expires_format(expires: &str) -> Result<(), DbError> {
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        if !re.is_match(expires) {
            return Err(DbError::InvalidInput(
                "expires_at must be YYYY-MM-DD HH:MM:SS".into(),
            ));
        }
        Ok(())
    }

    pub fn create_session(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: &str,
    ) -> Result<i64, DbError> {
        Self::validate_expires_format(expires_at)?;
        <Self as SessionRepository>::create_session(self, user_id, token_hash, expires_at)
    }

    pub fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), DbError> {
        <Self as SessionRepository>::delete_session_by_token_hash(self, token_hash)
    }

    pub fn find_valid_session(&self, token_hash: &str) -> Result<Option<Session>, DbError> {
        <Self as SessionRepository>::find_valid_session(self, token_hash)
    }

    pub fn cleanup_expired(&self) -> Result<usize, DbError> {
        <Self as SessionRepository>::cleanup_expired(self)
    }

    pub fn delete_all_for_user(&self, user_id: i64) -> Result<usize, DbError> {
        <Self as SessionRepository>::delete_all_for_user(self, user_id)
    }
}

impl SessionRepository for SessionRepo {
    fn create_session(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: &str,
    ) -> Result<i64, DbError> {
        Self::validate_expires_format(expires_at)?;
        let conn = self.pool.get()?;
        match conn.execute(
            "INSERT INTO sessions (user_id, token_hash, expires_at) VALUES (?1, ?2, ?3)",
            params![user_id, token_hash, expires_at],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == ErrorCode::ConstraintViolation =>
            {
                Err(DbError::DuplicateEntry(
                    "Session token hash already exists".into(),
                ))
            }
            Err(e) => Err(DbError::QueryError(e)),
        }
    }

    fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM sessions WHERE token_hash = ?1",
            params![token_hash],
        )?;
        Ok(())
    }

    fn find_valid_session(&self, token_hash: &str) -> Result<Option<Session>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT s.id, s.user_id, s.token_hash, s.created_at, s.expires_at FROM sessions s JOIN users u ON s.user_id = u.id WHERE s.token_hash = ?1 AND s.expires_at > datetime('now') AND u.is_active = 1",
        )?;
        let mut rows = stmt.query_map(params![token_hash], |row| {
            Ok(Session {
                id: row.get(0)?,
                user_id: row.get(1)?,
                token_hash: row.get(2)?,
                created_at: row.get(3)?,
                expires_at: row.get(4)?,
            })
        })?;
        match rows.next() {
            Some(session) => Ok(Some(session?)),
            None => Ok(None),
        }
    }

    fn cleanup_expired(&self) -> Result<usize, DbError> {
        let conn = self.pool.get()?;
        let deleted = conn.execute(
            "DELETE FROM sessions WHERE expires_at <= datetime('now')",
            [],
        )?;
        Ok(deleted)
    }

    fn delete_all_for_user(&self, user_id: i64) -> Result<usize, DbError> {
        let conn = self.pool.get()?;
        let deleted = conn.execute("DELETE FROM sessions WHERE user_id = ?1", params![user_id])?;
        Ok(deleted)
    }
}
