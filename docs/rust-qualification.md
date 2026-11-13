# Rust server qualification ledger

**Status: experimental / not approved for production.** Keep PR #85 draft and
unmerged; the production `Dockerfile` and Node server must remain supported
until all review gates in milestone #1 / issue #84 are complete.

The Rust implementation reads and writes the existing Node-compatible on-disk
state. Its storage records and API have explicit interoperability tests; this
ledger separates tested properties from assumptions and remaining gaps.

| Contract surface | Current evidence | Remaining qualification |
|---|---|---|
| Public objects (GET/HEAD, SHA-256 ETag, ranges, cache validators, CORS, mounted URL) | `tests/rust-server-smoke.js`; `tests/rust-protocol-parity.js` with 81 handpicked and 200 seeded protocol requests | Further malformed HTTP, large-file and proxy/nginx integration tests |
| Create/replace/delete and SHA-256 CAS | 300 paired Node/Rust checks; true concurrent `SIGKILL` recovery; mixed-read/write soak | Real power-loss/fsync semantics on deployment filesystems |
| Grants, scopes, operations, quotas, user/issuer tokens, revocation | Rust unit tests, paired HTTP checks, concurrent subtree-revocation `SIGKILL` test | Independent adversarial auth review and larger grant-tree loads |
| Durable journal, atomic replacement and staged files | `tests/rust-crash-recovery.js`, `tests/rust-crash-grants.js`, `tests/rust-disk-failure.js` | More I/O fault schedules and real machine power failures |
| Node on-disk upgrade and rollback | Node→Rust→Node smoke, crash tests reopen Rust data with Node, recover Node-style abandoned atomic JSON files | Production copy rehearsal with full realistic historical data |
| Web Push subscription API, VAPID keys and persistent wake-ups | Rust unit tests, Node-compatible subscription and VAPID state, `SIGKILL`-recovered notifications | Real external Web Push service integration and browser receipt |
| Web Push delivery and SSRF protection | Signed AES128GCM test, local verified HTTPS transport, untrusted cert rejection, 201/410/302 handling, public-address filtering and pinning | Real DNS-rebinding and network fault-injection environment, independent security review |
| Resource limits and shutdown | Upload semaphore tests, idle socket/connection limits, memory soak, non-root container startup and graceful `SIGTERM` | Heavy-write tail latency (>3s p99 in one 32-worker run), sustained load on target hardware and externally constrained memory/cgroups |
| Structural quality and CI | Default rustfmt, strict Clippy, 50-line/10-entry/sibling-import lint, Rust preview workflow, Node CI, Docker preview | Reviewer sign-off and complete acceptance-matrix reconciliation |

## Reproduce

```sh
npm ci
make test
make rust-crash-test
cargo clippy --locked --all-targets -- \
  -D clippy::correctness -D clippy::suspicious \
  -D clippy::perf -D clippy::complexity
cargo build --locked --release -p skrynia-server
SKRYNIA_BENCH_REPETITIONS=2 node benchmarks/compare-rust.js target/release/skrynia-server
make rust-soak
```

`make test` includes the 16 existing Node test files, Rust unit tests, the
Ivanka-compatible structural linter, a persisted-state smoke run, and the
81 + 200 reproducible paired request checks. `make rust-crash-test` deliberately
kills server processes and changes temporary test-data permissions: run under a
non-root user. It is separate from the fast CI/unit target.

`docs/rust-benchmarks.md` records raw observed performance, methodology and
important caveats, including the remaining durable-write latency bottleneck.
Do not use an isolated shared-host benchmark to assert general superiority.

## Release gate

A successful CI run is not production approval. A reviewed evidence trail is
still required for large realistic state migration, power-loss behavior on
real deployment storage, sustained service limits, external Web Push delivery
and security boundaries. Keep the PR draft until those are demonstrated.
