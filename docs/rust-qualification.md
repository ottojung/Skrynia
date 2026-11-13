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
| Web Push delivery and SSRF protection | Signed AES128GCM test, independent OpenSSL receiver decryption with tamper and wrong-key checks, local verified HTTPS transport, untrusted cert rejection, 201/410/302 handling, public-address filtering and pinning | Real DNS-rebinding and network fault-injection environment, independent security review |
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

### Fail-closed credential and subscription metadata I/O

`tests/rust-metadata-failure.js` independently denies writes to the private
grant and subscription directories during grant creation, user issuance, user
revocation, subscription creation, capability-protected update, and deletion.
Each failure must return HTTP 500 and force `recovery_required` for subsequent
requests, so a possible post-rename directory-sync error cannot leave live
authority disagreeing with persisted state. Once permissions are restored, a
restart reloads records and restores normal access; checks cover retained
user credentials and subscription records. Runs as non-root under `make
rust-crash-test` and in preview CI.

### Recovery-required state under concurrency

A persistence error freezes the server until restart. HTTP entry checks alone
cannot protect requests which already entered and are waiting for the storage
mutex: every storage, grant, scoped-listing and subscription operation now
rechecks `recovery_required` **after** acquiring the state lock and before
reading private authority or making changes. A threaded unit test explicitly
queues a write behind that lock, marks the state failed, releases the lock,
and requires HTTP 503 with no mutation. The existing filesystem fault-injection
suite still verifies failure recovery across restarts.

### Push completion races after a persistence error

A running Web Push batch may contain multiple already-sent notifications.
The Push completion path now checks the recovery-required flag after taking
the same state lock as HTTP mutations. If an outbox acknowledgement or
retry-state update fails on disk, the completion sets the flag **before**
releasing that lock, preventing the other completions from changing durable
state in the gap before the worker loop observes the failure. A regression
test replaces an outbox JSON path with a directory to produce an actual
filesystem error, confirms the fail-closed transition and verifies that a
second completion is refused. Recovery still replays the unacknowledged
notification at least once after restart.

### Startup-only staging cleanup

The writer lock protects startup removal of recognized abandoned Node/Rust
staging files. Background journal replay no longer repeats that sweep or
syncs an empty journal directory every minute; it only syncs the transaction
directory after removing actual journal records. The startup clean/replay
ordering and process-crash tests remain in place.

### Periodic journal replay failures

The asynchronous Push runner replays pending committed mutations every minute.
That replay now checks health under the storage mutex, sets `recovery_required`
*inside* the mutex on any failure, and stops the runner rather than attempting
another delivery in the same tick. Unexpected Push delivery task failures
also mark the server unhealthy. Unit tests inject a malformed on-disk journal
record, require the recovery call to fail and refuse subsequent mutations.
Startup recovery remains mandatory before serving requests.

### Contended key linearizability

`tests/rust-key-races.js` races 16 independent HTTP clients against the same
key in each of six rounds, first to create (exactly one 201, others 409), then
to replace with the same SHA-256 CAS precondition (exactly one 200, others
412), then to delete with the matching precondition (exactly one 200, others
404). The persisted bytes and SHA-256 ETag must exactly match each winning
mutation. It runs independently under Node and Rust, then restarts the *other*
server on each data directory to check durable compatibility. This is a
regression test for the synchronous CAS/state-lock boundary, not proof of
linearizability for all possible multi-key protocols. Configure
`SKRYNIA_RACE_ROUNDS` and `SKRYNIA_RACE_WIDTH` for longer runs.

### Release identity parity

The Rust preview image accepts the same `SKRYNIA_VERSION` and
`SKRYNIA_COMMIT` Docker build arguments as the Node image (default
`development`/`development`). Rust startup validates either the complete
development pair or a nonempty version and 40–64 character hexadecimal
commit, and `/version` returns the effective values. An incomplete/invalid
pair refuses to start. The Rust protocol smoke test exercises a populated
release pair and an invalid commit. These flags identify a build; they are
not proof of its provenance without trusted CI/image signing.

### Public URL normalization parity

The Rust server now parses `SKRYNIA_URL` with WHATWG-compatible URL semantics,
matching the Node reference on case-insensitive HTTP(S) schemes, dot segments
(including percent-encoded `.` / `..`), empty search/hash delimiters, and
trailing slashes. It rejects relative URLs and nonempty search/hash values.
A live persisted-state smoke test starts Rust with an encoded parent directory
in its configured public path and verifies that requests succeed under the
normalized mount point. Unit tests cover valid and invalid public URLs.

### Public-store symlink and special-file injection

Rust's built-in public GET/HEAD handler and conditional-write SHA-256 reader
open a **pinned regular file descriptor** with `O_NOFOLLOW | O_NONBLOCK`.
This prevents a symbolic link or blocking named pipe introduced after startup
from redirecting an object read into private grant files or hanging an HTTP
worker. The smoke/fault test injects a symlink from `store/` to the private
root-grant JSON *after* Rust has started, verifies GET/HEAD and SHA-256 CAS
fail without exposing credentials, and confirms ordinary reads and writes
still work after the invalid entry is removed. Rust unit tests also exercise
a real FIFO. Startup scanning already rejects nonregular stored entries.
This protects only Rust's built-in file reader: **nginx or another external
static server must independently forbid symlink traversal and untrusted
filesystem writes**. Never give untrusted publishers direct write access to
`SKRYNIA_DATA_DIR/store/`.

### Independently decryptable Web Push receipts

A separate Rust test `signature::tests::receipt` decrypts the encrypted
AES128GCM Push body using OpenSSL's P-256 ECDH, HMAC-SHA256 HKDF and AES-GCM,
not the message builder's encryption/decryption implementation. It verifies
the exact `changed` plaintext, authentic final-record delimiter, and rejection
when either the ciphertext authentication tag or the recipient private key
is wrong. It runs in `cargo test` and Rust preview CI, alongside the local
verified-TLS HTTP sender test. This tests protocol-crypto interoperability
without a live third-party Push service or browser receipt; those remain
separate acceptance gates.

### Parallel staging with commit-time revalidation

The Rust object mutation path preflights authorization, size/quota, CAS and
Push capacity while holding the shared state lock; durable temporary-file
writes and `fsync` now run outside that lock. Immediately before journal
commit, Rust reacquires the lock and **repeats all preconditions**. The final
link/rename and destination-directory sync remain within the serialized
journal transaction. RAII removes unused staged names on losing contenders,
failed quotas, or other errors. The differential key-race test now exercises
two independent dimensions: 16 writers to one CAS key and 16 writers to
different keys sharing an aggregate one-object grant quota. Both Node and
Rust must accept exactly one quota winner and leave no temporary files. The
existing crash/fault suites and four-run performance comparison were also
repeated. See `docs/rust-benchmarks.md` for exact measurements and caveats.
