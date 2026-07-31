use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cryptographic error: {0}")]
    Crypto(#[from] lib_fyc_crypto::CryptoError),

    #[error("Role error: {0}")]
    Role(#[from] lib_fyc_role::RoleError),

    #[error("Token error: {0}")]
    Token(#[from] lib_fyc_token::error::TokenError),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    #[error("Password hash error: {0}")]
    PasswordHash(String),

    #[error("Account already exists")]
    AlreadyExists,
}
