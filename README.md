# fyc-pos

Open-source Point of Sale system for 4yours Coffee, built entirely in Rust.

**Status: early development, not yet installable.**

---

## Project Structure

```
fyc-pos/
├── fyc-db/          # database layer (SQLite, migrations, repositories)
├── fyc-sdk/         # business logic & auth service (crypto, sessions, roles)
├── fyc-server/      # (planned) REST API server
├── fyc-pos/         # (planned) main application binary
└── .github/workflows/
```

---

## Components (v0.1.0)

### fyc-db
- SQLite with connection pool (`r2d2_sqlite`)
- Automatic migrations (`roles`, `users`, `user_roles`, `sessions`)
- Foreign key enforcement on every connection
- Parameterized queries (no SQL injection)
- Repository pattern: `UserRepo`, `RoleRepo`, `SessionRepo`
- Full integration test suite (17 tests)
- Benchmarks for core operations

### fyc-sdk
- `AuthService`: `register`, `login`, `logout`, `validate_token`, `user_has_role`
- Password hashing with Argon2id
- Session tokens hashed with SHA-512 (only hash stored)
- Age keypair generation per user (X25519)
- Private key encrypted with passphrase (age passphrase)
- Full integration test suite (13 tests)
- Benchmarks for auth operations

### Security
- Argon2id for password hashing (64 MiB, 3 iterations, 4 parallelism)
- SHA-512 for session token hashing
- Age (X25519) for per-user keypair
- Transactions for atomic registration
- Active-user check during token validation
- Old sessions cleared on new login
- Cargo audit in CI (RUSTSEC-2023-0071 ignored, not applicable)

---

## Quick Start (development only)

```bash
# Build all crates
cargo build

# Run all tests
make test

# Run benchmarks
cargo bench -p fyc-db -- --sample-size 10
cargo bench -p fyc-sdk -- --sample-size 10

# CI pipeline (format, clippy, test)
make ci
```

---

## Recent Changes

- `aec0b38` test(bench): remove sample_size override
- `626a143` ci: update benchmarking configuration
- `29e13f7` fix(ci): restore missing steps and increase benchmark samples
- `faef60e` chore(cargo): add audit ignore and update CI
- `e9c6373` feat: add CI, benchmarks, and SDK auth
- `082fa27` feat(sdk): add authentication and crypto features
- `99ce2a7` feat(db): enhance database with benchmarks, tests, and CI
- `f1a87d5` feat(workspace): add workspace and database crates
- `4831c50` chore(workspace): remove placeholder main functions
- `c82ea3b` feat(workspace): initialize workspace with fyc crates
- `d8bd4e3` Initial commit

---

## License

MIT