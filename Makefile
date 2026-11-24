.PHONY: test build lint check docker-build source-check
test: source-check lint
	cargo test --locked --lib --bins --tests --examples

lint:
	cargo fmt --all -- --check
	mkdir -p target
	rustc --test rust-server/source_lints/mod.rs -o target/skrynia-source-lint-tests
	target/skrynia-source-lint-tests
	cargo clippy --locked --all-targets -- -D clippy::correctness -D clippy::suspicious -D clippy::perf -D clippy::complexity

build:
	cargo build --locked --release --package libskrynia

check: test

CONTAINER_ENGINE ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null)
docker-build:
	$(CONTAINER_ENGINE) build -t skrynia-v3:local .

source-check:
	@if git ls-files | grep -E '\.(js|jsx|mjs|cjs|ts|tsx)$$'; then 	  echo 'Non-Rust implementation source is forbidden'; exit 1; fi
	@if git ls-files | grep -E '^(evidence/|examples/nginx-origins\.conf|rust-server/examples/bench/|docs/(api-spec|operations|cookbook|application-feasibility|complexity|indexed-store-design|intent-location-authority|sparse-grant-workload|verification)\.md|docs/v3/intent-supersession\.md)'; then \
	  echo 'Obsolete native object-store/API sources are forbidden'; exit 1; fi
	@if git grep -n -E '(/grants/|/store/|rootIssuerToken|X-Skrynia-If-SHA256|SKRYNIA_URL)' -- README.md docs rust-server/src rust-server/tests 2>/dev/null; then \
	  echo 'Obsolete native object-store protocol references are forbidden'; exit 1; fi
