use crate::config::{CURRENT_VERSION, MAGIC_BYTES};
use crate::handler::error::Result;
use crate::handler::types::Hash;
use lib_fyc_crypto;
use postcard;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub salt: [u8; 32],
    pub master_key: Zeroizing<[u8; 32]>,
    pub branch_refs: Vec<BranchRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRef {
    pub name: String,
    pub commit_hash: Option<Hash>,
}

impl DatabaseHeader {
    pub fn new() -> Self {
        let mut master_key = Zeroizing::new([0u8; 32]);
        rand::thread_rng().fill_bytes(&mut *master_key);
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        DatabaseHeader {
            magic: MAGIC_BYTES,
            version: CURRENT_VERSION,
            salt,
            master_key,
            branch_refs: vec![BranchRef {
                name: crate::config::DEFAULT_BRANCH.to_string(),
                commit_hash: None,
            }],
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(postcard::to_allocvec(self)?)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(postcard::from_bytes(data)?)
    }
}

impl Default for DatabaseHeader {
    fn default() -> Self {
        Self::new()
    }
}

pub fn encrypt_and_store_header(
    file: &mut std::fs::File,
    header: &DatabaseHeader,
    passphrase: &str,
) -> Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    let plaintext = header.to_bytes()?;
    let encrypted = lib_fyc_crypto::encrypt_with_passphrase(&plaintext, passphrase)?;
    let len = encrypted.ciphertext.len() as u64;
    file.seek(SeekFrom::Start(0))?; 
    file.write_all(&len.to_le_bytes())?;
    file.write_all(&encrypted.ciphertext)?;
    file.set_len(8 + len)?; 
    Ok(())
}

pub fn read_and_decrypt_header(
    file: &mut std::fs::File,
    passphrase: &str,
) -> Result<DatabaseHeader> {
    use std::io::Read;
    let mut len_buf = [0u8; 8];
    file.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    let mut encrypted = vec![0u8; len];
    file.read_exact(&mut encrypted)?;
    let decrypted = lib_fyc_crypto::decrypt_with_passphrase(&encrypted, passphrase)?;
    DatabaseHeader::from_bytes(&decrypted.plaintext)
}
