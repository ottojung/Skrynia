.PHONY: test build lint check docker-build
test: lint
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
