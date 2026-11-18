.PHONY: test build lint check docker-build source-check benchmark v3-smoke
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

# Skrynia is only a kernel; operators provide their own Program 0 WASM.
v3-smoke:
	@test -n "$(PROGRAM0_WASM)" || (echo "Set PROGRAM0_WASM to user-supplied root WASM" >&2; exit 1)
	cargo run --locked -p skrynia-server --example v3_smoke -- "$(PROGRAM0_WASM)"
