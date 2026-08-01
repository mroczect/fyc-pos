use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];
pub type PageNumber = u64;
pub type Timestamp = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: Hash,
    pub parent: Option<Hash>,
    pub tree: Hash,
    pub timestamp: Timestamp,
    pub author: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub entry_hash: Hash,
    pub prev_entry_hash: Option<Hash>,
    pub timestamp: Timestamp,
    pub operation: String,
    pub data_hash: Hash,
    pub user_id: String,
}
