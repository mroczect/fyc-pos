use crate::error::CryptoError;
use crate::types::DecryptOutput;
use tracing::instrument;
use zeroize::Zeroizing;

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

pub fn decrypt_symmetric(
    key: &[u8; 32],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    use chacha20poly1305::{
        ChaCha20Poly1305, Nonce,
        aead::{Aead, KeyInit},
    };
    if ciphertext.len() < 12 + 16 {
        return Err(CryptoError::DecryptionFailed("ciphertext too short".into()));
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes).map_err(|_| CryptoError::Internal("nonce".into()))?;
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| CryptoError::Internal("invalid key".into()))?;
    let plaintext = cipher
        .decrypt(&nonce, encrypted)
        .map_err(|_| CryptoError::DecryptionFailed("decryption error".into()))?;
    Ok(Zeroizing::new(plaintext))
}
