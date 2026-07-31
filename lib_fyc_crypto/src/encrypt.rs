use crate::error::CryptoError;
use crate::types::EncryptOutput;
use tracing::instrument;

#[instrument(skip(plaintext, passphrase))]
pub fn encrypt_with_passphrase(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<EncryptOutput, CryptoError> {
    let res = librage::encrypt_with_passphrase(plaintext, passphrase);
    if res.success {
        Ok(EncryptOutput::new(res.data.unwrap().ciphertext.to_vec()))
    } else {
        Err(CryptoError::EncryptionFailed(res.error.unwrap().message))
    }
}

#[instrument(skip(plaintext))]
pub fn encrypt_with_x25519(
    plaintext: &[u8],
    public_key: &str,
) -> Result<EncryptOutput, CryptoError> {
    let res = librage::encrypt(plaintext, public_key);
    if res.success {
        Ok(EncryptOutput::new(res.data.unwrap().ciphertext.to_vec()))
    } else {
        Err(CryptoError::EncryptionFailed(res.error.unwrap().message))
    }
}
