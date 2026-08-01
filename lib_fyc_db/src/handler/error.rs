use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cryptographic error: {0}")]
    Crypto(#[from] lib_fyc_crypto::CryptoError),

    #[error("Account error: {0}")]
    Account(#[from] lib_fyc_account::AccountError),

    #[error("Token error: {0}")]
    Token(#[from] lib_fyc_token::error::TokenError),

    #[error("Role error: {0}")]
    Role(#[from] lib_fyc_role::RoleError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Invalid database file: {0}")]
    InvalidFile(String),

    #[error("Database already initialized")]
    AlreadyInitialized,

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Version control error: {0}")]
    Version(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

pub type Result<T> = std::result::Result<T, DbError>;
