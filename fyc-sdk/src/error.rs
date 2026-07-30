use fyc_db::DbError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),
}
