pub mod error;
pub mod types;

use error::TokenError;
use std::collections::HashSet;
use tracing::instrument;
use types::TokenPayload;
use zeroize::Zeroizing;

pub struct TokenManager {
    secret: Zeroizing<[u8; 32]>,
    revoked: HashSet<[u8; 32]>,
}

impl TokenManager {
    pub fn new(secret: [u8; 32]) -> Self {
        Self {
            secret: Zeroizing::new(secret),
            revoked: HashSet::new(),
        }
    }

    #[instrument(skip(self))]
    pub fn generate_token(&self, payload: &TokenPayload) -> Result<String, TokenError> {
        let serialized = postcard::to_allocvec(payload)?;
        let hash = blake3::keyed_hash(&self.secret, &serialized);
        let token = format!(
            "{}.{}",
            base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                &serialized
            ),
            base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                hash.as_bytes()
            )
        );
        tracing::debug!("Token generated for user {}", payload.user_id);
        Ok(token)
    }

    #[instrument(skip(self))]
    pub fn validate_token(&self, token: &str) -> Result<TokenPayload, TokenError> {
        let token_hash = blake3::hash(token.as_bytes());
        if self.revoked.contains(token_hash.as_bytes()) {
            return Err(TokenError::InvalidSignature);
        }

        let parts: Vec<&str> = token.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(TokenError::InvalidSignature);
        }
        let encoded_payload = parts[0];
        let encoded_hash = parts[1];

        let serialized = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            encoded_payload,
        )?;
        let expected_hash = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            encoded_hash,
        )?;
        if expected_hash.len() != 32 {
            return Err(TokenError::InvalidSignature);
        }

        let computed_hash = blake3::keyed_hash(&self.secret, &serialized);
        if constant_time_eq::constant_time_eq(computed_hash.as_bytes(), &expected_hash) {
            let payload: TokenPayload =
                postcard::from_bytes(&serialized).map_err(|_| TokenError::InvalidSignature)?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| TokenError::Internal(format!("clock error: {}", e)))?
                .as_secs();
            if payload.exp < now {
                tracing::warn!("Token expired for user {}", payload.user_id);
                return Err(TokenError::Expired);
            }
            tracing::debug!("Token validated for user {}", payload.user_id);
            Ok(payload)
        } else {
            tracing::warn!("Invalid token signature");
            Err(TokenError::InvalidSignature)
        }
    }

    #[instrument(skip(self))]
    pub fn revoke_token(&mut self, token: &str) {
        let token_hash = blake3::hash(token.as_bytes());
        self.revoked.insert(*token_hash.as_bytes());
        tracing::debug!("Token revoked");
    }
}
