use crate::crypto;
use crate::error::SdkError;
use fyc_db::sqlite::{AuditRepo, RoleRepo, SessionRepo, UserRepo};
use fyc_db::{DbError, DbPool};

pub struct AuthService {
    pool: DbPool,
    user_repo: UserRepo,
    role_repo: RoleRepo,
    session_repo: SessionRepo,
}

impl AuthService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            user_repo: UserRepo::new(pool.clone()),
            role_repo: RoleRepo::new(pool.clone()),
            session_repo: SessionRepo::new(pool.clone()),
            pool,
        }
    }

    pub fn register(&self, username: &str, password: &str) -> Result<i64, SdkError> {
        if password.len() < 8 {
            return Err(SdkError::InvalidInput(
                "Password must be at least 8 characters".into(),
            ));
        }

        let (public_key, private_key) = crypto::generate_age_keypair()?;
        let encrypted_private_key =
            crypto::encrypt_private_key_with_password(&private_key, password)?;
        let password_hash = crypto::hash_password(password)?;

        let conn = self.pool.get().map_err(DbError::from)?;
        conn.execute("BEGIN", []).map_err(DbError::from)?;

        let user_id = match UserRepo::create_user_with_conn(
            &conn,
            username,
            &password_hash,
            &public_key,
            &encrypted_private_key,
        ) {
            Ok(id) => id,
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e.into());
            }
        };

        if let Err(e) = AuditRepo::log_with_conn(
            &conn,
            user_id,
            "user:register",
            Some(user_id),
            Some(username),
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        let default_role = "kasir";
        let role_id = match RoleRepo::get_role_by_name_with_conn(&conn, default_role) {
            Ok(Some(r)) => r.id,
            Ok(None) => {
                match RoleRepo::create_role_with_conn(&conn, default_role, "Default cashier role") {
                    Ok(id) => id,
                    Err(DbError::DuplicateEntry(_)) => {
                        RoleRepo::get_role_by_name_with_conn(&conn, default_role)?
                            .ok_or_else(|| SdkError::Internal("Role creation conflict".into()))?
                            .id
                    }
                    Err(e) => {
                        let _ = conn.execute("ROLLBACK", []);
                        return Err(e.into());
                    }
                }
            }
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(e.into());
            }
        };

        if let Err(e) = RoleRepo::assign_role_to_user_with_conn(&conn, user_id, role_id) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.into());
        }

        conn.execute("COMMIT", []).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            DbError::from(e)
        })?;
        Ok(user_id)
    }

    pub fn login(&self, username: &str, password: &str) -> Result<(String, i64), SdkError> {
        let user = self
            .user_repo
            .find_by_username(username)?
            .ok_or_else(|| SdkError::AuthFailed("Invalid username or password".into()))?;

        if !crypto::verify_password(password, &user.password_hash)? {
            return Err(SdkError::AuthFailed("Invalid username or password".into()));
        }

        self.session_repo.delete_all_for_user(user.id)?;

        let token = crypto::generate_token()?;
        let token_hash = crypto::hash_token(&token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        let expires_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();

        self.session_repo
            .create_session(user.id, &token_hash, &expires_str)?;
        Ok((token, user.id))
    }

    pub fn logout(&self, token: &str) -> Result<(), SdkError> {
        let token_hash = crypto::hash_token(token);
        self.session_repo
            .delete_session_by_token_hash(&token_hash)?;
        Ok(())
    }

    pub fn validate_token(&self, token: &str) -> Result<i64, SdkError> {
        let token_hash = crypto::hash_token(token);
        let session = self
            .session_repo
            .find_valid_session(&token_hash)?
            .ok_or_else(|| SdkError::AuthFailed("Invalid or expired session".into()))?;
        Ok(session.user_id)
    }

    pub fn user_has_role(&self, user_id: i64, role_name: &str) -> Result<bool, SdkError> {
        if role_name.trim().is_empty() {
            return Ok(false);
        }
        Ok(self.role_repo.has_role(user_id, role_name)?)
    }
}
