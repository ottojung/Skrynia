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
	node tests/rust-protocol-parity.js
	node tests/rust-large-read.js
	node tests/rust-push-parity.js

.PHONY: test test-node rust-test

# Adversarial real-process SIGKILL tests run separately from the fast suite.
rust-crash-test:
	cargo build --locked --package skrynia-server
	node tests/rust-crash-recovery.js
	node tests/rust-crash-grants.js
	node tests/rust-disk-failure.js

.PHONY: rust-crash-test

# Mixed-workload safety/latency/RSS probe; defaults to a 12-second release run.
rust-soak:
	cargo build --locked --release --package skrynia-server
	node benchmarks/soak-rust.js target/release/skrynia-server

.PHONY: rust-soak

# Offline Node -> Rust -> Node rehearsal on delegated state and pending Push data.
rust-migration-test:
	cargo build --locked --package skrynia-server
	node tests/rust-migration-rehearsal.js

.PHONY: rust-migration-test
