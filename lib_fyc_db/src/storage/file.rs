use crate::handler::error::{DbError, Result};
use std::fs::{File, OpenOptions};
use std::path::Path;

pub fn open_db_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .open(path)
        .map_err(|e| DbError::Storage(format!("Cannot open db file: {}", e)))
}

pub fn create_db_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| DbError::Storage(format!("Cannot create db file: {}", e)))
}
