use crate::error::CryptoError;
use crate::types::EncryptOutput;
use tracing::instrument;

#[instrument(skip(plaintext, passphrase))]
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<EncryptOutput, CryptoError> {
    let res = librage::encrypt_with_passphrase(plaintext, passphrase);
    if let Some(data) = res.data {
        Ok(EncryptOutput::new(data.ciphertext.to_vec()))
    } else {
        let msg = res
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(CryptoError::EncryptionFailed(msg))
    }
}

#[instrument(skip(plaintext))]
pub fn encrypt_with_x25519(
    plaintext: &[u8],
    public_key: &str,
) -> Result<EncryptOutput, CryptoError> {
    let res = librage::encrypt(plaintext, public_key);
    if let Some(data) = res.data {
        Ok(EncryptOutput::new(data.ciphertext.to_vec()))
    } else {
        let msg = res
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(CryptoError::EncryptionFailed(msg))
    }
}
