# Node and Rust suites have independent fixtures and may run concurrently.
test:
	$(MAKE) --no-print-directory -j2 test-node rust-test

test-node:
	npm test

rust-test:
	cargo fmt --all -- --check
	cargo test --locked --all-targets
	mkdir -p target
	rustc --test rust-server/source_lints/mod.rs -o target/skrynia-source-lint-tests
	target/skrynia-source-lint-tests
	cargo build --locked --package skrynia-server
	node --test tests/rust-server-smoke.js tests/rust-protocol-parity.js \
	  tests/rust-large-read.js tests/rust-push-parity.js

.PHONY: test test-node rust-test

# Adversarial real-process SIGKILL tests run separately from the fast suite.
rust-crash-test:
	cargo build --locked --package skrynia-server
	node tests/rust-crash-recovery.js
	node tests/rust-crash-grants.js
	node tests/rust-disk-failure.js
	node tests/rust-root-rotation.js
	node tests/rust-metadata-failure.js
	node tests/rust-key-races.js
	node tests/rust-public-symlink.js

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

# Container-level cgroup enforcement and concurrent upload pressure.
# Requires Podman (rootless) or Docker (set SKRYNIA_CONTAINER_ENGINE=docker).
rust-container-memory-test:
	sh scripts/test-rust-container-memory.sh

.PHONY: rust-container-memory-test
