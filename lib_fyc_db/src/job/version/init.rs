use crate::config::DEFAULT_BRANCH;
use crate::handler::error::Result;
use crate::handler::types::Commit;
use crate::journal::chain::JournalChain;
use crate::storage::{file, header};
use lib_fyc_account::AccountManager;
use lib_fyc_role::types::Permission;
use std::path::Path;

pub fn init_database(
    db_path: &Path,
    passphrase: &str,
    token: &str,
    account_manager: &AccountManager,
) -> Result<()> {
    let payload = account_manager.validate_session(token)?;
    if !payload.role.can(Permission::ManageProducts) {
        return Err(crate::handler::error::DbError::Role(
            lib_fyc_role::error::RoleError::PermissionDenied(
                "Only Admin or Developer can initialize database".into(),
            ),
        ));
    }

    let mut file = file::create_db_file(db_path)?;

    let mut header = header::DatabaseHeader::new();
    header::encrypt_and_store_header(&mut file, &header, passphrase)?;

    let commit_template = Commit {
        hash: [0u8; 32],
        parent: None,
        tree: [0u8; 32],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        author: payload.user_id.clone(),
        message: "Initial commit".to_string(),
    };
    let commit_hash: [u8; 32] = blake3::hash(&postcard::to_allocvec(&commit_template)?).into();
    let _commit = Commit {
        hash: commit_hash,
        ..commit_template
    };

    for branch in header.branch_refs.iter_mut() {
        if branch.name == DEFAULT_BRANCH {
            branch.commit_hash = Some(commit_hash);
        }
    }
    header::encrypt_and_store_header(&mut file, &header, passphrase)?;

    let master_key = *header.master_key;
    let mut journal = JournalChain::new(db_path, master_key);
    journal.append("init".to_string(), commit_hash, payload.user_id)?;

    tracing::info!(
        "Database initialized with commit {}",
        hex::encode(commit_hash)
    );
    Ok(())
}
