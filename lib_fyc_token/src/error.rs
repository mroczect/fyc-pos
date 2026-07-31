use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Token expired")]
    Expired,
    #[error("Invalid token signature")]
    InvalidSignature,
    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Internal error: {0}")]
    Internal(String),
}
