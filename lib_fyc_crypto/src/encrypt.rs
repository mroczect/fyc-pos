use crate::error::CryptoError;
use crate::types::EncryptOutput;
use rand::RngCore;
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

pub fn encrypt_symmetric(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    use chacha20poly1305::{
        ChaCha20Poly1305, Nonce,
        aead::{Aead, KeyInit},
    };
    use rand::rngs::OsRng;

    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| CryptoError::Internal("invalid key".into()))?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce =
        Nonce::try_from(&nonce_bytes[..]).map_err(|_| CryptoError::Internal("nonce".into()))?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed("encryption error".into()))?;
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}
