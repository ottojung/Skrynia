.PHONY: test build lint check docker-build source-check benchmark
test: source-check lint
	cargo test --locked --all-targets

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
