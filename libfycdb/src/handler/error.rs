use librage::LibrageError as RageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FycDbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Journal integrity violation: {0}")]
    Journal(String),

    #[error("Version control error: {0}")]
    Version(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("librage error: {0}")]
    Librage(#[from] RageError),
}

pub type Result<T> = std::result::Result<T, FycDbError>;
