use criterion::{Criterion, criterion_group, criterion_main};
use fyc_db::connection::create_pool;
use fyc_sdk::AuthService;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;

fn bench_register(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let auth = AuthService::new(pool);
    let counter = AtomicU64::new(0);

    c.bench_function("register", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let username = format!("user{}", id);
            auth.register(&username, "password123").unwrap();
        })
    })
    .sample_size(10);
}

fn bench_login(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let auth = AuthService::new(pool);
    auth.register("loginuser", "password123").unwrap();

    c.bench_function("login", |b| {
        b.iter(|| {
            auth.login("loginuser", "password123").unwrap();
        })
    });
}

fn bench_validate_token(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let auth = AuthService::new(pool);
    auth.register("tokenuser", "password123").unwrap();
    let (token, _) = auth.login("tokenuser", "password123").unwrap();

    c.bench_function("validate_token", |b| {
        b.iter(|| {
            auth.validate_token(&token).unwrap();
        })
    });
}

criterion_group!(benches, bench_register, bench_login, bench_validate_token);
criterion_main!(benches);
