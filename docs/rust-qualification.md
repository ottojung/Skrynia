# Rust server qualification ledger

**Status: experimental / not approved for production.** Keep PR #85 draft and
unmerged; the production `Dockerfile` and Node server must remain supported
until all review gates in milestone #1 / issue #84 are complete.

The Rust implementation reads and writes the existing Node-compatible on-disk
state. Its storage records and API have explicit interoperability tests; this
ledger separates tested properties from assumptions and remaining gaps.

| Contract surface | Current evidence | Remaining qualification |
|---|---|---|
| Public objects (GET/HEAD, SHA-256 ETag, ranges, cache validators, CORS, mounted URL) | `tests/rust-server-smoke.js`; `tests/rust-protocol-parity.js` with 81 handpicked and 200 seeded protocol requests | More malformed HTTP and external proxy/nginx integration; 4MiB pinned read/overlapping CAS tested |
| Create/replace/delete and SHA-256 CAS | 300 paired Node/Rust checks; true concurrent `SIGKILL` recovery; mixed-read/write soak | Real power-loss/fsync semantics on deployment filesystems |
| Grants, scopes, operations, quotas, user/issuer tokens, revocation | Rust unit tests, paired HTTP checks, concurrent subtree-revocation `SIGKILL` test | Independent adversarial auth review and larger grant-tree loads |
| Durable journal, atomic replacement and staged files | `tests/rust-crash-recovery.js`, `tests/rust-crash-grants.js`, `tests/rust-disk-failure.js` | More I/O fault schedules and real machine power failures |
| Node on-disk upgrade and rollback | Node→Rust→Node smoke, crash tests reopen Rust data with Node, recover Node-style abandoned atomic JSON files | Production copy rehearsal with full realistic historical data |
| Web Push subscription API, VAPID keys and persistent wake-ups | Rust unit tests, paired Push lifecycle/capability/error checks, Node-compatible subscription and VAPID state, `SIGKILL`-recovered notifications | Real external Web Push service integration and browser receipt |
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

## Pinned read invariant

`tests/rust-large-read.js` streams a 4 MiB public object over GET while a
SHA-256-CAS PUT replaces that key. The GET must return the original complete
bytes and ETag from its pinned descriptor; a subsequent GET must return the
replacement bytes/ETag. A ranged read verifies offsets, and Node must read
the replacement from the Rust directory after restart. It runs under `make
test` and in Rust preview CI.

## Web Push HTTP parity

`tests/rust-push-parity.js` compares the live Node and Rust APIs on independent
throwaway directories. It checks VAPID key publication, subscription creation,
malformed inputs, missing/wrong capabilities, deduplication and rotation,
patch validation precedence, endpoint conflicts, and deletion. Opaque random
IDs/capabilities are schema-validated but not compared for equality. This is
not a substitute for external Web Push delivery receipts.

## Offline upgrade-and-rollback rehearsal

`make rust-migration-test` starts the Node reference server on a freshly
created private directory, populates 100 checked objects through HTTP across
four delegated team grants, creates four Web Push subscriptions and pending
notifications, and captures the original VAPID keys and user credentials. It
shuts down Node, starts Rust on exactly the same directory, verifies the
objects and scoped listing, and performs conditional modifications, deletions
and new creations. Finally, it reopens the data with Node and checks the
resulting objects, subscriptions, outbox, and journal. The test uses the
literal same files, not a simulated format conversion or API mock.

Run `SKRYNIA_MIGRATION_OBJECTS_PER_LEAF=40 make rust-migration-test` to expand
the corpus to 160 objects. This demonstrates the cross-runtime file format
and credential boundary in a synthetic deployment-like dataset. It is **not**
proof that arbitrary historical production data or external integrations
will migrate safely. Before release, rehearse with a full *copy* of real data,
with a documented backup and rollback plan.

### Root authority rotation

`tests/rust-root-rotation.js` proves that deleting the root grant (a permitted
action) invalidates old bearer/issuer tokens without deleting public object
data, and that the next Rust startup creates a fresh root with different
credentials. It also validates that new authority can create new objects and
that the original Node server can read all retained values while old user
credentials remain revoked. Runs in `make rust-crash-test` and Rust CI.
