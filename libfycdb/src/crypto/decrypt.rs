use crate::handler::error::FycDbError;
use tracing::instrument;
use zeroize::Zeroizing;

#[instrument(skip(ciphertext, passphrase))]
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &str,
) -> Result<Zeroizing<Vec<u8>>, FycDbError> {
    let response = librage::decrypt_with_passphrase(ciphertext, passphrase);
    if response.success {
        Ok(response.data.unwrap().plaintext)
    } else {
        Err(FycDbError::Crypto(response.error.unwrap().message))
    }
}

#[instrument(skip(ciphertext))]
pub fn decrypt_with_x25519(
    ciphertext: &[u8],
    secret_key: &str,
) -> Result<Zeroizing<Vec<u8>>, FycDbError> {
    let response = librage::decrypt(ciphertext, secret_key);
    if response.success {
        Ok(response.data.unwrap().plaintext)
    } else {
        Err(FycDbError::Crypto(response.error.unwrap().message))
    }
}
