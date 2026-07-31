use crate::error::CryptoError;
use crate::types::DecryptOutput;
use tracing::instrument;

#[instrument(skip(ciphertext, passphrase))]
pub fn decrypt_with_passphrase(
    ciphertext: &[u8],
    passphrase: &str,
) -> Result<DecryptOutput, CryptoError> {
    let res = librage::decrypt_with_passphrase(ciphertext, passphrase);
    if let Some(data) = res.data {
        Ok(DecryptOutput::new(data.plaintext.to_vec()))
    } else {
        let msg = res
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(CryptoError::DecryptionFailed(msg))
    }
}

#[instrument(skip(ciphertext))]
pub fn decrypt_with_x25519(
    ciphertext: &[u8],
    secret_key: &str,
) -> Result<DecryptOutput, CryptoError> {
    let res = librage::decrypt(ciphertext, secret_key);
    if let Some(data) = res.data {
        Ok(DecryptOutput::new(data.plaintext.to_vec()))
    } else {
        let msg = res
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(CryptoError::DecryptionFailed(msg))
    }
}
