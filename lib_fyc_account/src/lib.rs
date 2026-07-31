pub mod error;
pub mod types;

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use error::AccountError;
use lib_fyc_role::types::Role;
use lib_fyc_token::{TokenManager, types::TokenPayload};
use once_cell::sync::Lazy;
use tracing::instrument;
use types::{Account, Credentials};
use uuid::Uuid;
use zeroize::Zeroizing;

static ARGON2: Lazy<Argon2> = Lazy::new(|| {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19_456, 2, 1, Some(32)).expect("valid argon2 params"),
    )
});

const MAX_TOKEN_EXPIRY_SECS: u64 = 3600;

pub struct AccountManager {
    token_manager: TokenManager,
    accounts: Vec<Account>,
}

impl AccountManager {
    pub fn new(token_secret: [u8; 32]) -> Self {
        Self {
            token_manager: TokenManager::new(token_secret),
            accounts: Vec::new(),
        }
    }

    #[instrument(skip(self, password))]
    pub fn create_account(
        &mut self,
        username: String,
        password: Zeroizing<String>,
        role: Role,
    ) -> Result<Account, AccountError> {
        if self.accounts.iter().any(|a| a.username == username) {
            return Err(AccountError::AlreadyExists);
        }
        let hash = hash_password(&password)?;
        let account = Account {
            user_id: Uuid::new_v4().to_string(),
            username,
            password_hash: hash,
            role,
        };
        self.accounts.push(account.clone());
        tracing::info!("Account created for {}", account.username);
        Ok(account)
    }

    #[instrument(skip(self, credentials))]
    pub fn login(
        &self,
        credentials: &Credentials,
        requested_expiry: u64,
    ) -> Result<String, AccountError> {
        let account = self
            .accounts
            .iter()
            .find(|a| a.username == credentials.username)
            .ok_or(AccountError::UserNotFound)?;

        let valid = verify_password(&credentials.password, &account.password_hash).unwrap_or(false);
        if !valid {
            return Err(AccountError::InvalidCredentials);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| AccountError::Internal(format!("clock error: {}", e)))?
            .as_secs();

        let token_expiry = std::cmp::min(requested_expiry, MAX_TOKEN_EXPIRY_SECS);
        let payload = TokenPayload {
            user_id: account.user_id.clone(),
            role: account.role,
            exp: now + token_expiry,
        };

        let token = self.token_manager.generate_token(&payload)?;
        tracing::info!("User {} logged in", account.username);
        Ok(token)
    }

    #[instrument(skip(self))]
    pub fn validate_session(&self, token: &str) -> Result<TokenPayload, AccountError> {
        Ok(self.token_manager.validate_token(token)?)
    }

    #[instrument(skip(self, new_password, token))]
    pub fn change_password(
        &mut self,
        user_id: &str,
        new_password: Zeroizing<String>,
        token: &str,
    ) -> Result<(), AccountError> {
        let payload = self.token_manager.validate_token(token)?;
        if payload.user_id != user_id && payload.role != Role::Admin {
            return Err(AccountError::PermissionDenied);
        }
        let account = self
            .accounts
            .iter_mut()
            .find(|a| a.user_id == user_id)
            .ok_or(AccountError::UserNotFound)?;
        let new_hash = hash_password(&new_password)?;
        account.password_hash = new_hash;
        tracing::info!("Password changed for user {}", account.username);
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn revoke_token(&mut self, token: &str) -> Result<(), AccountError> {
        self.token_manager.revoke_token(token);
        Ok(())
    }
}

fn hash_password(password: &str) -> Result<String, AccountError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = ARGON2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AccountError::PasswordHash(e.to_string()))?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AccountError> {
    let parsed = PasswordHash::new(hash).map_err(|e| AccountError::PasswordHash(e.to_string()))?;
    Ok(ARGON2.verify_password(password.as_bytes(), &parsed).is_ok())
}
