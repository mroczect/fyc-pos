use crate::handler::error::FycDbError;
use tracing::instrument;

#[instrument(skip(plaintext, passphrase))]
pub fn encrypt_with_passphrase(plaintext: &[u8], passphrase: &str) -> Result<Vec<u8>, FycDbError> {
    let response = librage::encrypt_with_passphrase(plaintext, passphrase);
    if response.success {
        Ok(response.data.unwrap().ciphertext.to_vec())
    } else {
        Err(FycDbError::Crypto(response.error.unwrap().message))
    }
}

#[instrument(skip(plaintext))]
pub fn encrypt_with_x25519(plaintext: &[u8], public_key: &str) -> Result<Vec<u8>, FycDbError> {
    let response = librage::encrypt(plaintext, public_key);
    if response.success {
        Ok(response.data.unwrap().ciphertext.to_vec())
    } else {
        Err(FycDbError::Crypto(response.error.unwrap().message))
    }
}
