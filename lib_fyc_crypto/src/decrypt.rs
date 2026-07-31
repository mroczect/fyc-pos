use crate::error::CryptoError;
use crate::types::DecryptOutput;
use tracing::instrument;

#[instrument(skip(ciphertext, passphrase))]
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &str,
) -> Result<DecryptOutput, CryptoError> {
    let res = librage::decrypt_with_passphrase(ciphertext, passphrase);
    if res.success {
        Ok(DecryptOutput::new(res.data.unwrap().plaintext.to_vec()))
    } else {
        Err(CryptoError::DecryptionFailed(res.error.unwrap().message))
    }
}

#[instrument(skip(ciphertext))]
pub fn decrypt_with_x25519(
    ciphertext: &[u8],
    secret_key: &str,
) -> Result<DecryptOutput, CryptoError> {
    let res = librage::decrypt(ciphertext, secret_key);
    if res.success {
        Ok(DecryptOutput::new(res.data.unwrap().plaintext.to_vec()))
    } else {
        Err(CryptoError::DecryptionFailed(res.error.unwrap().message))
    }
}
