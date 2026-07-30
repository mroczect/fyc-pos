use criterion::{Criterion, criterion_group, criterion_main};
use fyc_db::connection::create_pool;
use fyc_db::repositories::{RoleRepo, SessionRepo, UserRepo};
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;

fn bench_user_creation(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let user_repo = UserRepo::new(pool);
    let counter = AtomicU64::new(0);

    c.bench_function("create_user", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let username = format!("user{}", id);
            user_repo
                .create_user(&username, "hash", "pubkey", "encpriv")
                .unwrap();
        })
    });
}

fn bench_role_creation(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let role_repo = RoleRepo::new(pool);
    let counter = AtomicU64::new(0);

    c.bench_function("create_role", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let role_name = format!("role{}", id);
            role_repo.create_role(&role_name, "Bench role").unwrap();
        })
    });
}

fn bench_session_create_and_find(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    let user_repo = UserRepo::new(pool.clone());
    let session_repo = SessionRepo::new(pool);
    let user_id = user_repo
        .create_user("benchuser", "hash", "pk", "ek")
        .unwrap();
    let counter = AtomicU64::new(0);

    c.bench_function("session_create", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let hash = format!("hash{}", id);
            session_repo
                .create_session(user_id, &hash, "2099-01-01")
                .unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_user_creation,
    bench_role_creation,
    bench_session_create_and_find
);
criterion_main!(benches);
