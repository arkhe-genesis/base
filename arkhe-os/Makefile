.PHONY: all build test check clippy fmt clean

all: build test

build:
	cargo build --workspace --release

test:
	cargo test --workspace -- --nocapture

check:
	cargo check --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

clean:
	cargo clean