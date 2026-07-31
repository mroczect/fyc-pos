use criterion::{Criterion, criterion_group, criterion_main};
use fyc_db::connection::create_pool;
use fyc_db::sqlite::RoleRepo;
use fyc_sdk::*;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;

fn seed_and_pool() -> (TempDir, fyc_db::DbPool) {
    let dir = TempDir::new().unwrap();
    let pool = create_pool(dir.path().join("bench.db")).unwrap();
    seed_defaults(&pool).unwrap();
    (dir, pool)
}

fn make_admin(pool: &fyc_db::DbPool) -> (AuthService, String) {
    let auth = AuthService::new(pool.clone());
    let user_id = auth.register("admin", "admin1234").unwrap();
    let role_repo = RoleRepo::new(pool.clone());
    let admin_role = role_repo.get_role_by_name("admin").unwrap().unwrap();
    role_repo
        .assign_role_to_user(user_id, admin_role.id)
        .unwrap();
    let (token, _) = auth.login("admin", "admin1234").unwrap();
    (auth, token)
}

fn bench_register(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let auth = AuthService::new(pool);
    let counter = AtomicU64::new(0);

    c.bench_function("register", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let username = format!("user{}", id);
            auth.register(&username, "password123").unwrap();
        })
    });
}

fn bench_login(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let auth = AuthService::new(pool);
    auth.register("loginuser", "password123").unwrap();

    c.bench_function("login", |b| {
        b.iter(|| {
            auth.login("loginuser", "password123").unwrap();
        })
    });
}

fn bench_validate_token(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let auth = AuthService::new(pool);
    auth.register("tokenuser", "password123").unwrap();
    let (token, _) = auth.login("tokenuser", "password123").unwrap();

    c.bench_function("validate_token", |b| {
        b.iter(|| {
            auth.validate_token(&token).unwrap();
        })
    });
}

fn bench_menu_add_product(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let menu = MenuService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool);
    let counter = AtomicU64::new(0);

    c.bench_function("menu_add_product", |b| {
        b.iter(|| {
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let name = format!("Product{}", id);
            menu.add_product(&admin_token, &name, 15000.0, "Food")
                .unwrap();
        })
    });
}

fn bench_order_create(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let menu = MenuService::new(pool.clone());
    let order_svc = OrderService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool);
    let prod_id = menu
        .add_product(&admin_token, "TestItem", 10000.0, "Cat")
        .unwrap();

    c.bench_function("order_create", |b| {
        b.iter(|| {
            let _ = order_svc
                .create_order(&admin_token, &[(prod_id, 1)])
                .unwrap();
        })
    });
}

fn bench_daily_revenue(c: &mut Criterion) {
    let (_dir, pool) = seed_and_pool();
    let report = ReportService::new(pool.clone());
    let menu = MenuService::new(pool.clone());
    let order_svc = OrderService::new(pool.clone());
    let (_, admin_token) = make_admin(&pool);
    let prod_id = menu
        .add_product(&admin_token, "RevItem", 20000.0, "Cat")
        .unwrap();
    order_svc
        .create_order(&admin_token, &[(prod_id, 1)])
        .unwrap();

    c.bench_function("daily_revenue", |b| {
        b.iter(|| {
            report.daily_revenue(&admin_token).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_register,
    bench_login,
    bench_validate_token,
    bench_menu_add_product,
    bench_order_create,
    bench_daily_revenue,
);
criterion_main!(benches);
