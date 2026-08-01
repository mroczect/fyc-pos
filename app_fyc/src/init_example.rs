use lib_fyc_account::{AccountManager, Credentials};
use lib_fyc_db::init_database;
use lib_fyc_role::Role;
use rand::RngCore;
use std::path::Path;
use zeroize::Zeroizing;

fn main() {
    tracing_subscriber::fmt::init();

    let mut secret = Zeroizing::new([0u8; 32]);
    rand::thread_rng().fill_bytes(&mut *secret);
    let secret_bytes = *secret;

    let mut account_manager = AccountManager::new(secret_bytes);

    let dev = account_manager
        .create_account(
            "dev".to_string(),
            Zeroizing::new("devpassword".to_string()),
            Role::Developer,
        )
        .expect("Gagal membuat akun developer");
    println!("✅ Akun developer dibuat: {}", dev.username);

    let creds = Credentials {
        username: "dev".to_string(),
        password: Zeroizing::new("devpassword".to_string()),
    };
    let token = account_manager.login(&creds, 3600).expect("Gagal login");
    println!("✅ Login berhasil, token didapatkan");

    let db_path = Path::new("contoh_coffee_shop.fyc");
    let passphrase = "super_secure_passphrase_123";

    match init_database(db_path, passphrase, &token, &account_manager) {
        Ok(()) => {
            println!("🎉 Database berhasil diinisialisasi!");
            println!("   Path: {}", db_path.display());
            println!("   Branch: main");
            println!("   Commit awal sudah tercatat di journal terenkripsi");

            match lib_fyc_db::storage::file::open_db_file(db_path) {
                Ok(mut file) => {
                    match lib_fyc_db::storage::header::read_and_decrypt_header(
                        &mut file, passphrase,
                    ) {
                        Ok(header) => {
                            println!("✅ Verifikasi header berhasil");
                            println!("   Magic: {:?}", std::str::from_utf8(&header.magic));
                            println!("   Version: {}", header.version);
                            println!("   Branch: {:?}", header.branch_refs);
                        }
                        Err(e) => eprintln!("❌ Gagal verifikasi header: {}", e),
                    }
                }
                Err(e) => eprintln!("❌ Gagal membuka file: {}", e),
            }
        }
        Err(e) => {
            eprintln!("❌ Gagal inisialisasi: {}", e);
            std::process::exit(1);
        }
    }
}
