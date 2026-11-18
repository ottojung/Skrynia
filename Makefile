.PHONY: test test-v3-root build lint check docker-build source-check benchmark
test: source-check lint
	cargo test --locked --lib --bins --tests --examples
	cargo check --locked -p skrynia-program0 --features vm-check

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

# Strong acceptance gate: build root WASM and execute it against the Rust kernel.
# Requires the wasm32-unknown-unknown Rust target; no deployed data is used.
test-v3-root:
	cargo build --locked --release -p skrynia-program0 --target wasm32-unknown-unknown
	SKRYNIA_TEST_ROOT_WASM="$(CURDIR)/target/wasm32-unknown-unknown/release/skrynia_program0.wasm" cargo test --locked -p skrynia-program0 --test program0 -- root_live
