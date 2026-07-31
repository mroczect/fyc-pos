use crate::error::CryptoError;
use tracing::instrument;
use zeroize::Zeroizing;

#[instrument]
pub fn generate_keypair() -> Result<(Zeroizing<String>, String), CryptoError> {
    let res = librage::generate_keypair();
    if res.success {
        let data = res.data.unwrap();
        Ok((data.secret_key, data.public_key))
    } else {
        Err(CryptoError::KeyGenFailed(res.error.unwrap().message))
    }
}
