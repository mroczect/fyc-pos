.PHONY: all build release check test test-verbose bench fmt fmt-check clippy lint clean run install uninstall ci ci-full snap

all: build

build:
	cargo build

release:
	cargo build --release

check:
	cargo check --workspace

test:
	cargo test --workspace

test-verbose:
	cargo test --workspace -- --nocapture

bench:
	cargo bench --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace -- -D warnings

lint: fmt clippy

clean:
	cargo clean

run:
	cargo run -p fyc-pos

install:
	cargo install --path fyc-pos

uninstall:
	cargo uninstall fyc-pos

ci: fmt-check clippy test

ci-full: fmt-check clippy test bench

snap:
	snapcat lib_fyc_account/ -f markdown -o dev/lib_fyc_account.md && snapcat lib_fyc_crypto/ -f markdown -o dev/lib_fyc_crypto.md && snapcat lib_fyc_role/ -f markdown -o dev/lib_fyc_role.md && snapcat lib_fyc_token/ -f markdown -o dev/lib_fyc_token.md && snapcat lib_fyc_db/ -f markdown -o dev/lib_fyc_db.md && snapcat lib_fyc_sdk/ -f markdown -o dev/lib_fyc_sdk.md