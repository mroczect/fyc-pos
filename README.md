# fyc-pos

**Open-source Point of Sale system for 4yours Coffee, built entirely in Rust.**  
_Status: v0.1.0 – core libraries stable, application interfaces (CLI/GUI/API) under development._

---

## Table of Contents

- [Installation](#installation)
- [Project Structure](#project-structure)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
  - [fyc-sdk](#fyc-sdk)
    - [Initialization](#initialization)
    - [AuthService](#authservice)
    - [MenuService](#menuservice)
    - [OrderService](#orderservice)
    - [ReportService](#reportservice)
    - [PermissionService](#permissionservice)
    - [Seed Defaults](#seed-defaults)
  - [fyc-db](#fyc-db)
    - [Connection Pool](#connection-pool)
    - [Repositories](#repositories)
    - [Models](#models)
    - [Error Types](#error-types)
- [Database Schema](#database-schema)
- [Security](#security)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

The libraries are **not yet published on [crates.io](https://crates.io)**.  
You can use them directly from the Git repository or as local path dependencies.

### Git Repository

Add the following to your `Cargo.toml`:

```toml
[dependencies]
fyc-db = { git = "https://github.com/mroczect/fyc-pos.git", path = "fyc-db" }
fyc-sdk = { git = "https://github.com/mroczect/fyc-pos.git", path = "fyc-sdk" }
```

You can also pin to a specific branch, tag, or commit:

```toml
fyc-db = { git = "https://github.com/mroczect/fyc-pos.git", path = "fyc-db", branch = "master" }
fyc-sdk = { git = "https://github.com/mroczect/fyc-pos.git", path = "fyc-sdk", rev = "a1b2c3d" }
```

### Local Path (when developing inside the workspace)

If you are working within the `fyc-pos` workspace, use path dependencies:

```toml
[dependencies]
fyc-db = { path = "../fyc-db" }
fyc-sdk = { path = "../fyc-sdk" }
```

### Multi‑crate Workspace Setup

If your own project is also a workspace, add the crates as members:

```toml
[workspace]
members = [
    "your-crate",
    "path/to/fyc-db",
    "path/to/fyc-sdk",
]
```

### Future Installation via crates.io

Once stable, the libraries will be published on [crates.io](https://crates.io) and you will be able to install them with:

```bash
cargo add fyc-db
cargo add fyc-sdk
```

Until then, use one of the methods above.
---

## Project Structure

```
fyc-pos/
  fyc-db/          # Database layer: SQLite connection, migrations, repositories.
  fyc-sdk/         # Business logic: authentication, menu, orders, reports, permissions.
  fyc-pos/         # Planned: end-user application (CLI / GUI).
  fyc-server/      # Planned: REST API server for multi-device access.
```

Only `fyc-db` and `fyc-sdk` are production‑ready. `fyc-pos` and `fyc-server` are placeholders for future development.

---

## Architecture

The system follows a strict layered architecture:

- **fyc-db** – pure data access; owns the SQLite schema, connection pool, and parameterised queries. No business logic.
- **fyc-sdk** – consumes `fyc-db` and implements all business rules (authorisation checks, transactional audit logging, cryptographic operations). It is the single entry point for any application.
- **fyc-pos** / **fyc-server** – the presentation layer; they depend exclusively on `fyc-sdk` and are forbidden from accessing `fyc-db` directly.

All operations that modify state are wrapped in database transactions and recorded in an immutable audit log.

---

## Quick Start

This example shows how to initialise the database, create an admin user, and use the core services.

```rust
use fyc_db::connection::create_pool;
use fyc_sdk::*;
use fyc_db::repositories::RoleRepo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create connection pool (SQLite file)
    let pool = create_pool("fyc.db")?;

    // 2. Seed default roles, permissions, and a sample custom field
    seed_defaults(&pool)?;

    // 3. Create the first admin user (registration returns a user with role "kasir")
    let auth = AuthService::new(pool.clone());
    let admin_id = auth.register("admin", "super_secret_123")?;

    // 4. Promote that user to admin
    let role_repo = RoleRepo::new(pool.clone());
    if let Some(admin_role) = role_repo.get_role_by_name("admin")? {
        role_repo.assign_role_to_user(admin_id, admin_role.id)?;
    }

    // 5. Login as admin
    let (token, _) = auth.login("admin", "super_secret_123")?;

    // 6. Use services
    let menu = MenuService::new(pool.clone());
    let espresso_id = menu.add_product(&token, "Espresso", 25000.0, "Coffee")?;

    let order_svc = OrderService::new(pool.clone());
    let order_id = order_svc.create_order(&token, &[(espresso_id, 2)])?;

    let report = ReportService::new(pool.clone());
    let revenue = report.daily_revenue(&token)?;
    println!("Today's revenue: {}", revenue);

    Ok(())
}
```

---

## API Reference

### fyc-sdk

All public types are re‑exported from `fyc_sdk`.

#### Initialization

Every service needs a `DbPool` (alias for `r2d2::Pool<SqliteConnectionManager>`). Obtain one via `fyc_db::connection::create_pool`.

#### AuthService

```rust
pub struct AuthService { /* private */ }

impl AuthService {
    pub fn new(pool: DbPool) -> Self;
    pub fn register(&self, username: &str, password: &str) -> Result<i64, SdkError>;
    pub fn login(&self, username: &str, password: &str) -> Result<(String, i64), SdkError>;
    pub fn logout(&self, token: &str) -> Result<(), SdkError>;
    pub fn validate_token(&self, token: &str) -> Result<i64, SdkError>;
    pub fn user_has_role(&self, user_id: i64, role_name: &str) -> Result<bool, SdkError>;
}
```

- **register** – creates a new user with role `kasir`. Password must be at least 8 characters. Generates an age X25519 keypair, encrypts the private key with the user’s password, and hashes the password with Argon2id. The whole operation is atomic.
- **login** – verifies password, removes all previous sessions for the user, generates a random 32‑byte token, stores its SHA‑512 hash in the `sessions` table, and returns the plain token together with the user ID.
- **logout** – deletes the session identified by the given token.
- **validate_token** – checks that the token hash exists, the session is not expired, and the associated user is still active. Returns the user ID.
- **user_has_role** – checks whether a user possesses a specific role (used for coarse authorisation).

#### MenuService

```rust
pub struct MenuService { /* private */ }

impl MenuService {
    pub fn new(pool: DbPool) -> Self;
    pub fn add_product(&self, token: &str, name: &str, price: f64, category: &str) -> Result<i64, SdkError>;
    pub fn update_product(&self, token: &str, id: i64, name: &str, price: f64, category: &str) -> Result<(), SdkError>;
    pub fn delete_product(&self, token: &str, id: i64) -> Result<(), SdkError>;
    pub fn list_products(&self, token: &str) -> Result<Vec<Product>, SdkError>;
    pub fn add_custom_field(&self, token: &str, name: &str, field_type: &str) -> Result<i64, SdkError>;
    pub fn set_custom_value(&self, token: &str, product_id: i64, field_name: &str, value: &str) -> Result<(), SdkError>;
    pub fn get_product_custom_values(&self, token: &str, product_id: i64) -> Result<Vec<(ProductCustomField, String)>, SdkError>;
}
```

- All mutating methods require the corresponding permission (`product:create`, `product:update`, `product:delete`, `customfield:manage`).
- Operations are wrapped in transactions together with an audit log entry; failure at any step rolls back the entire transaction.
- `delete_product` performs a soft delete (sets `is_active = false`).
- Custom fields can be of type `text`, `number`, or `boolean`; values are stored as text. The underlying table enforces uniqueness per product and field.

#### OrderService

```rust
pub struct OrderService { /* private */ }

impl OrderService {
    pub fn new(pool: DbPool) -> Self;
    pub fn create_order(&self, token: &str, items: &[(i64, i32)]) -> Result<i64, SdkError>;
    pub fn get_today_orders(&self, token: &str) -> Result<Vec<Order>, SdkError>;
    pub fn get_order_detail(&self, token: &str, order_id: i64) -> Result<(Order, Vec<OrderItem>), SdkError>;
}
```

- **create_order** – accepts a list of `(product_id, quantity)` tuples. It fetches the current price of each product (inside the transaction), calculates the total, creates an order with status `paid`, inserts order items, and appends an audit log entry. Permission `order:create` is required.
- **get_today_orders** – returns all orders placed today, newest first. Requires `order:view`.
- **get_order_detail** – returns the order metadata together with its items. Requires `order:view`.

#### ReportService

```rust
pub struct ReportService { /* private */ }

impl ReportService {
    pub fn new(pool: DbPool) -> Self;
    pub fn daily_revenue(&self, token: &str) -> Result<f64, SdkError>;
}
```

- **daily_revenue** – sums the `total` of all paid orders created today. Requires `report:view`.

#### PermissionService

```rust
pub struct PermissionService { /* private */ }

impl PermissionService {
    pub fn new(pool: DbPool) -> Self;
    pub fn has_permission(&self, user_id: i64, permission: &str) -> Result<bool, SdkError>;
}
```

- Checks whether a user has a specific permission by joining `user_roles`, `role_permissions`, and `permissions`.

#### Seed Defaults

```rust
pub fn seed_defaults(pool: &DbPool) -> Result<(), SdkError>;
```

Creates the roles `admin` and `kasir`, defines a set of standard permissions, assigns all permissions to the `admin` role, and inserts a sample custom field `varian`. The function is idempotent; duplicate entries are silently ignored.

---

### fyc-db

The `fyc-db` crate exposes the database layer directly, but typical applications should only use it through `fyc-sdk`.

#### Connection Pool

```rust
pub fn create_pool<P: AsRef<Path>>(db_path: P) -> Result<Pool<SqliteConnectionManager>, DbError>;
```

- Creates an `r2d2` pool with a maximum of 4 connections.
- Every connection automatically executes `PRAGMA foreign_keys = ON;`.
- Runs schema migrations (`CREATE TABLE IF NOT EXISTS`) on the first connection.

#### Repositories

All repositories follow the same pattern: they hold a pool and provide instance methods that obtain a connection from the pool, as well as associated functions (or `_with_conn` methods) that accept a `&rusqlite::Connection` for use within transactions.

- **UserRepo** – `create_user`, `find_by_username`, `find_by_id`, `deactivate_user`, `update_password`, and transactional variants.
- **RoleRepo** – `create_role`, `get_role_by_name`, `assign_role_to_user`, `has_role`, `assign_permission_to_role`, `remove_permission_from_role`, and transactional variants.
- **SessionRepo** – `create_session`, `delete_session_by_token_hash`, `find_valid_session` (joins users to check active status), `cleanup_expired`, `delete_all_for_user`.
- **ProductRepo** – `create`, `find_by_id`, `find_all_active`, `update`, `deactivate`, and transactional variants.
- **ProductCustomRepo** – `create_field`, `get_field_by_name`, `set_value` (upsert via `ON CONFLICT`), `get_values_for_product`.
- **OrderRepo** – `create_order`, `add_order_item`, `get_orders_today`, `get_order_items`, `find_order_by_id`.
- **PermissionRepo** – `create`, `get_by_name`, `get_user_permissions`.
- **AuditRepo** – `log`, `log_with_conn`. Audit entries are insert-only; no update or delete methods exist.

#### Models

Struct definitions (all with `Debug, Clone`):

- `User { id, username, password_hash, public_key, encrypted_private_key, is_active, created_at, updated_at }`
- `Role { id, name, description }`
- `Session { id, user_id, token_hash, created_at, expires_at }`
- `Permission { id, name, description }`
- `AuditLog { id, admin_id, action, target_user_id, details, created_at }`
- `Product { id, name, price, category, is_active, created_at, updated_at }`
- `ProductCustomField { id, name, field_type }`
- `Order { id, user_id, status, total, created_at }`
- `OrderItem { id, order_id, product_id, quantity, unit_price }`

Sensitive fields in `User` and `Session` implement `ZeroizeOnDrop`.

#### Error Types

```rust
pub enum DbError {
    PoolCreation(String),
    MigrationFailed(String),
    QueryError(rusqlite::Error),
    PoolError(r2d2::Error),
    InvalidInput(String),
    NotFound(String),
    DuplicateEntry(String),
    Internal(String),
}
```

`DbError` implements `From<rusqlite::Error>` and `From<r2d2::Error>`. It is re‑exported as `fyc_db::DbError`.

---

## Database Schema

All tables are created automatically via `run_migrations`.

| Table                   | Key Columns                                                                                    | Purpose                                                  |
| ----------------------- | ---------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `roles`                 | `id`, `name` (unique)                                                                          | User roles (admin, cashier, …)                           |
| `users`                 | `id`, `username` (unique), `password_hash`, `public_key`, `encrypted_private_key`, `is_active` | Core user accounts                                       |
| `user_roles`            | `user_id`, `role_id` (composite PK, FK cascade)                                                | Many‑to‑many user‑role mapping                           |
| `sessions`              | `id`, `user_id` (FK cascade), `token_hash` (unique), `expires_at`                              | Active login sessions. Index on `expires_at` for cleanup |
| `permissions`           | `id`, `name` (unique)                                                                          | Granular permissions (e.g. `product:create`)             |
| `role_permissions`      | `role_id`, `permission_id` (composite PK, FK cascade)                                          | Many‑to‑many role‑permission mapping                     |
| `audit_log`             | `id`, `admin_id` (FK), `action`, `target_user_id`, `details`, `created_at`                     | Immutable record of administrative actions               |
| `products`              | `id`, `name` (unique), `price`, `category`, `is_active`                                        | Menu items                                               |
| `product_custom_fields` | `id`, `name`, `field_type` (text/number/boolean)                                               | Runtime‑defined extra fields for products                |
| `product_custom_values` | `product_id`, `field_id`, `value`, unique constraint on `(product_id, field_id)`               | Values for the custom fields of each product             |
| `orders`                | `id`, `user_id` (FK), `status`, `total`, `created_at` (indexed)                                | Sales transactions                                       |
| `order_items`           | `id`, `order_id` (FK cascade), `product_id` (FK), `quantity`, `unit_price`                     | Line items of an order. Index on `order_id`              |

All foreign keys are enforced, and `ON DELETE CASCADE` is applied where appropriate.

---

## Security

- **Password storage** – Argon2id with default parameters (19 MiB memory, 3 iterations, 4 parallelism). Hashed passwords are the only representation stored.
- **Session tokens** – 32 random bytes generated via `getrandom`; only the SHA‑512 hash is persisted. Tokens are validated together with the user’s active status (JOIN on `users`).
- **Private key protection** – each user receives an X25519 keypair generated by the `age` protocol. The private key is encrypted with the user’s password using age’s passphrase mode (scrypt).
- **SQL injection prevention** – all queries use parameterised statements (no string concatenation).
- **Audit trail** – `audit_log` rows can only be inserted; there is no API to modify or delete them. Every mutating business operation writes an entry.
- **Transaction atomicity** – all multi‑step operations (e.g., product creation + audit) run inside a single SQLite transaction. If any step fails, the entire transaction is rolled back.
- **Foreign key enforcement** – enabled on every connection via the pool’s `with_init` hook.
- **Permission checks** – every service method validates the caller’s token and verifies the required permission before executing.

---

## Testing

The project includes a comprehensive integration test suite:

- **fyc-db** – 28 tests covering all repositories, constraint violations, cascade deletes, and custom fields.
- **fyc-sdk** – 23 tests covering authentication flows, authorisation failures, menu management, order creation, and daily revenue reporting.

Run them with:

```bash
cargo test --workspace
```

CI enforces `cargo fmt --check`, `cargo clippy -- -D warnings`, and a security audit (`cargo audit --deny warnings`).

---

## Benchmarks

Micro‑benchmarks are implemented with Criterion. To run them (with a reduced sample size for quick feedback):

```bash
cargo bench -p fyc-db --bench benchmarks -- --sample-size 10
cargo bench -p fyc-sdk --bench benchmarks -- --sample-size 10
```

Typical results (indicative):

| Operation (db)   | Time    |
| ---------------- | ------- |
| `create_user`    | ~157 µs |
| `create_role`    | ~142 µs |
| `session_create` | ~158 µs |
| `create_product` | ~146 µs |
| `create_order`   | ~309 µs |

| Operation (sdk)    | Time                                      |
| ------------------ | ----------------------------------------- |
| `register`         | ~2.1 s (dominated by Argon2 + age keygen) |
| `login`            | ~31 ms                                    |
| `validate_token`   | ~45 µs                                    |
| `menu_add_product` | ~321 µs (including audit)                 |
| `order_create`     | ~422 µs (including audit)                 |
| `daily_revenue`    | ~152 µs                                   |

---

## Contributing

Contributions are welcome. Please open an issue or pull request on the repository. Ensure all tests pass and `make ci` succeeds before submitting.

---

## License

This project is licensed under the [MIT License](LICENSE).
