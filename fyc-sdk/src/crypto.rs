use crate::error::SdkError;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use getrandom::getrandom;
use sha2::{Digest, Sha512};
use zeroize::Zeroizing;

pub fn generate_token() -> Result<String, SdkError> {
    let mut buf = [0u8; 32];
    getrandom(&mut buf).map_err(|e| SdkError::Crypto(format!("Failed to generate token: {e}")))?;
    Ok(hex::encode(buf))
}

pub fn hash_token(token: &str) -> String {
    let digest = Sha512::digest(token.as_bytes());
    hex::encode(digest)
}

pub fn hash_password(password: &str) -> Result<String, SdkError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| SdkError::Crypto(format!("Password hashing failed: {e}")))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, SdkError> {
    let argon2 = Argon2::default();
    let parsed_hash = argon2::password_hash::PasswordHash::new(hash)
        .map_err(|e| SdkError::Crypto(format!("Invalid password hash format: {e}")))?;
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_age_keypair() -> Result<(String, Zeroizing<String>), SdkError> {
    let response = librage::generate_keypair();
    if !response.success {
        let msg = response.error.map(|e| e.message).unwrap_or_default();
        return Err(SdkError::Crypto(format!("Key generation failed: {msg}")));
    }
    let data = response.data.unwrap();
    Ok((data.public_key, data.secret_key))
}

pub fn encrypt_private_key_with_password(
    private_key: &str,
    password: &str,
) -> Result<String, SdkError> {
    let response = librage::encrypt_with_passphrase(private_key.as_bytes(), password);
    if !response.success {
        let msg = response.error.map(|e| e.message).unwrap_or_default();
        return Err(SdkError::Crypto(format!("Encryption failed: {msg}")));
    }
    Ok(response.data.unwrap().as_string())
}

pub fn decrypt_private_key_with_password(
    encrypted_key: &str,
    password: &str,
) -> Result<Zeroizing<String>, SdkError> {
    let cipher_bytes = hex::decode(encrypted_key)
        .map_err(|e| SdkError::Crypto(format!("Invalid hex ciphertext: {e}")))?;
    let response = librage::decrypt_with_passphrase(&cipher_bytes, password);
    if !response.success {
        let msg = response.error.map(|e| e.message).unwrap_or_default();
        return Err(SdkError::Crypto(format!("Decryption failed: {msg}")));
    }
    Ok(Zeroizing::new(response.data.unwrap().as_string()))
}
