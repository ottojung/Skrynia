.PHONY: test build lint check docker-build source-check benchmark v3-wasm v3-smoke
test: source-check lint
	cargo test --locked --lib --bins --tests --examples

lint:
	cargo fmt --all -- --check
	mkdir -p target
	rustc --test rust-server/source_lints/mod.rs -o target/skrynia-source-lint-tests
	target/skrynia-source-lint-tests
	cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity

build:
	cargo build --locked --release --package skrynia-server

check: test

docker-build:
	docker build -t skrynia:local .

source-check:
	@if git ls-files | grep -E '\.(js|jsx|mjs|cjs|ts|tsx)$$'; then 	  echo 'Non-Rust implementation source is forbidden'; exit 1; fi

benchmark: build
	cargo run --locked --release --example skrynia_bench

# Development-only, builds Program 0 for a separately sandboxed v3 smoke test.
v3-wasm:
	RUSTC_BOOTSTRAP=1 cargo -Z build-std=std,panic_abort build --target wasm32-unknown-unknown --release -p skrynia-program0

v3-smoke: v3-wasm
	cargo run --locked -p skrynia-server --example v3_smoke -- $(or $(CARGO_TARGET_DIR),target)/wasm32-unknown-unknown/release/skrynia_program0.wasm
