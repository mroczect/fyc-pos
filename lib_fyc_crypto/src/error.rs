use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

impl From<librage::LibrageError> for CryptoError {
    fn from(e: librage::LibrageError) -> Self {
        CryptoError::EncryptionFailed(e.to_string())
    }
}
