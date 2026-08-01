use lib_fyc_db::handler::types::JournalEntry;
use lib_fyc_db::storage::header;
use std::path::PathBuf;

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <database.fyc> [passphrase]", args[0]);
        std::process::exit(1);
    }

    let db_path = PathBuf::from(&args[1]);
    let passphrase = if args.len() >= 3 {
        args[2].clone()
    } else {
        rpassword::prompt_password("Enter passphrase: ").unwrap()
    };

    let mut file = match lib_fyc_db::storage::file::open_db_file(&db_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            std::process::exit(1);
        }
    };

    let header = match header::read_and_decrypt_header(&mut file, &passphrase) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to decrypt header: {}", e);
            std::process::exit(1);
        }
    };

    println!("========== Database Header ==========");
    println!(
        "Magic: {:?}",
        std::str::from_utf8(&header.magic).unwrap_or("invalid")
    );
    println!("Version: {}", header.version);
    println!("Salt: {}", hex::encode(header.salt));
    println!("Master key: {}", hex::encode(&*header.master_key));
    println!("Branches:");
    for branch in &header.branch_refs {
        println!(
            "  - {}: {}",
            branch.name,
            branch
                .commit_hash
                .map(hex::encode)
                .unwrap_or_else(|| "none".to_string())
        );
    }

    let mut journal_path = db_path.clone();
    journal_path.set_extension("journal");
    if !journal_path.exists() {
        println!("\nNo journal file found.");
        return;
    }

    let encrypted_journal = match std::fs::read(&journal_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading journal: {}", e);
            return;
        }
    };

    let master_key = *header.master_key;
    let plaintext = match lib_fyc_crypto::decrypt_symmetric(&master_key, &encrypted_journal) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to decrypt journal: {}", e);
            return;
        }
    };

    let entries: Vec<JournalEntry> = match postcard::from_bytes(&plaintext) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to deserialize journal: {}", e);
            return;
        }
    };

    println!("\n========== Journal ({}) ==========", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        println!("--- Entry #{} ---", i + 1);
        println!("  Hash: {}", hex::encode(entry.entry_hash));
        println!(
            "  Previous: {}",
            entry
                .prev_entry_hash
                .map(hex::encode)
                .unwrap_or_else(|| "none".to_string())
        );
        println!("  Timestamp: {}", entry.timestamp);
        println!("  Operation: {}", entry.operation);
        println!("  Data hash: {}", hex::encode(entry.data_hash));
        println!("  User: {}", entry.user_id);

        if let Some(prev_hash) = entry.prev_entry_hash {
            if i > 0 {
                let expected_prev = entries[i - 1].entry_hash;
                if prev_hash != expected_prev {
                    println!("  ⚠️  RANTAI RUSAK! Prev hash tidak cocok dengan entry sebelumnya.");
                } else {
                    println!("  ✅ Rantai valid");
                }
            }
        }
    }

    let initialized = header.branch_refs.iter().any(|b| b.commit_hash.is_some());
    println!(
        "\nDatabase initialized: {}",
        if initialized { "Yes (commit found)" } else { "No (empty repository)" }
    );
}
