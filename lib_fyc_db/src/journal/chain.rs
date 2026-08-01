use crate::handler::error::Result;
use crate::handler::types::{Hash, JournalEntry};
use std::path::PathBuf;

pub struct JournalChain {
    entries: Vec<JournalEntry>,
    file_path: PathBuf,
    master_key: [u8; 32],
}

impl JournalChain {
    pub fn new(db_path: impl Into<PathBuf>, master_key: [u8; 32]) -> Self {
        let mut path = db_path.into();
        path.set_extension("journal");
        JournalChain {
            entries: Vec::new(),
            file_path: path,
            master_key,
        }
    }

    pub fn append(&mut self, operation: String, data_hash: Hash, user_id: String) -> Result<()> {
        let prev_hash = self.entries.last().map(|e| e.entry_hash);
        let new_entry = JournalEntry::new(prev_hash, operation, data_hash, user_id)?;
        self.entries.push(new_entry);
        self.save_to_file()?;
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let plaintext = postcard::to_allocvec(&self.entries)?;
        let encrypted = lib_fyc_crypto::encrypt_symmetric(&self.master_key, &plaintext)?;
        std::fs::write(&self.file_path, &encrypted)?;
        Ok(())
    }
}
