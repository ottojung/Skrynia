test: test-node rust-test

test-node:
	npm test

rust-test:
	cargo fmt --all -- --check
	cargo test --locked --all-targets
	mkdir -p target
	rustc --test rust-server/source_lints/mod.rs -o target/skrynia-source-lint-tests
	target/skrynia-source-lint-tests
	cargo build --locked --package skrynia-server
	node tests/rust-server-smoke.js

.PHONY: test test-node rust-test
