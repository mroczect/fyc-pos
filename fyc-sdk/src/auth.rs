use crate::crypto;
use crate::error::SdkError;
use fyc_db::DbPool;
use fyc_db::repositories::{RoleRepo, SessionRepo, UserRepo};

pub struct AuthService {
    user_repo: UserRepo,
    role_repo: RoleRepo,
    session_repo: SessionRepo,
}

impl AuthService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            user_repo: UserRepo::new(pool.clone()),
            role_repo: RoleRepo::new(pool.clone()),
            session_repo: SessionRepo::new(pool),
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

        let user_id = self.user_repo.create_user(
            username,
            &password_hash,
            &public_key,
            &encrypted_private_key,
        )?;

        let default_role = "kasir";
        let role = self.role_repo.get_role_by_name(default_role)?;
        let role_id = match role {
            Some(r) => r.id,
            None => self
                .role_repo
                .create_role(default_role, "Default cashier role")?,
        };
        self.role_repo.assign_role_to_user(user_id, role_id)?;

        Ok(user_id)
    }

    pub fn login(&self, username: &str, password: &str) -> Result<(String, i64), SdkError> {
        let user = self
            .user_repo
            .find_by_username(username)?
            .ok_or_else(|| SdkError::AuthFailed("Invalid username or password".into()))?;

        let valid = crypto::verify_password(password, &user.password_hash)?;
        if !valid {
            return Err(SdkError::AuthFailed("Invalid username or password".into()));
        }

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
        let roles = self.role_repo.get_user_roles(user_id)?;
        Ok(roles.iter().any(|r| r.name.eq_ignore_ascii_case(role_name)))
    }
}
