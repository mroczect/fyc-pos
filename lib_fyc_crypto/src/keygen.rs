use crate::error::CryptoError;
use tracing::instrument;
use zeroize::Zeroizing;

#[instrument]
pub fn generate_keypair() -> Result<(Zeroizing<String>, String), CryptoError> {
    let res = librage::generate_keypair();
    if let Some(data) = res.data {
        Ok((data.secret_key, data.public_key))
    } else {
        let msg = res
            .error
            .map(|e| e.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(CryptoError::KeyGenFailed(msg))
    }
}
