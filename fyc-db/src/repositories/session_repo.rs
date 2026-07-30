use crate::error::DbError;
use crate::models::Session;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ErrorCode, params};

pub struct SessionRepo {
    pool: Pool<SqliteConnectionManager>,
}

impl SessionRepo {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    pub fn create_session(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: &str,
    ) -> Result<i64, DbError> {
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

    pub fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), DbError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM sessions WHERE token_hash = ?1",
            params![token_hash],
        )?;
        Ok(())
    }

    pub fn find_valid_session(&self, token_hash: &str) -> Result<Option<Session>, DbError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT s.id, s.user_id, s.token_hash, s.created_at, s.expires_at
             FROM sessions s
             JOIN users u ON s.user_id = u.id
             WHERE s.token_hash = ?1 AND s.expires_at > datetime('now') AND u.is_active = 1",
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

    pub fn cleanup_expired(&self) -> Result<usize, DbError> {
        let conn = self.pool.get()?;
        let deleted = conn.execute(
            "DELETE FROM sessions WHERE expires_at <= datetime('now')",
            [],
        )?;
        Ok(deleted)
    }

    pub fn delete_all_for_user(&self, user_id: i64) -> Result<usize, DbError> {
        let conn = self.pool.get()?;
        let deleted = conn.execute("DELETE FROM sessions WHERE user_id = ?1", params![user_id])?;
        Ok(deleted)
    }
}
