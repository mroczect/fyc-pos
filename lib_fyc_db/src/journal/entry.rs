use crate::handler::error::Result;
use crate::handler::types::{Hash, JournalEntry};
use blake3::Hasher;

impl JournalEntry {
    pub fn new(
        prev_entry_hash: Option<Hash>,
        operation: String,
        data_hash: Hash,
        user_id: String,
    ) -> Result<Self> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::handler::error::DbError::Journal(format!("clock error: {}", e)))?
            .as_secs();
        let mut entry = JournalEntry {
            entry_hash: [0u8; 32],
            prev_entry_hash,
            timestamp: now,
            operation,
            data_hash,
            user_id,
        };
        entry.entry_hash = entry.compute_hash()?;
        Ok(entry)
    }

    fn compute_hash(&self) -> Result<Hash> {
        let mut hasher = Hasher::new();
        let copy = JournalEntry {
            entry_hash: [0u8; 32],
            prev_entry_hash: self.prev_entry_hash,
            timestamp: self.timestamp,
            operation: self.operation.clone(),
            data_hash: self.data_hash,
            user_id: self.user_id.clone(),
        };
        let data = postcard::to_allocvec(&copy)?;
        hasher.update(&data);
        Ok(hasher.finalize().into())
    }
}
