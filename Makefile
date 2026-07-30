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
	snapcat src -f markdown -o dev/src.snapcat.md && snapcat tests -f markdown -o dev/tests.snapcat.md