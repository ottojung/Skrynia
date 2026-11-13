# Rust server performance comparison (experimental)

The Rust server is not production-qualified. This document records comparable,
reproducible measurements, including unfavorable observations. It is **not** a
performance guarantee, a saturation test, or evidence of full protocol parity.

## Reproduce

On the same Linux machine and filesystem, with Node.js 20+ and Rust installed:

```sh
npm ci
cargo build --locked --release -p skrynia-server
SKRYNIA_BENCH_REPETITIONS=2 node benchmarks/compare-rust.js target/release/skrynia-server
```

The harness starts each server on a separate fresh temporary data directory,
uses the same Node.js HTTP/keep-alive client for both, initializes equivalent
root/user grant credentials, and alternates the run order. Measured operations:

- **Public GET:** 300 GETs of 32 KiB objects, concurrency 8, from 64 initialized objects.
- **Conditional PUT:** 32 independent 8 KiB CAS PUTs, concurrency 4, with the matching prior SHA-256.
- **Grant creation:** 24 independent child grants, concurrency 4, with fixed quotas and scopes.

Creation/setup, warm-up, startup, and shutdown are excluded from timed
operations. Each operation is verified for successful HTTP status and, where
applicable, body bytes or returned credentials. The reported operations/sec
measure total wall time of each workload, not the sum of per-request times.
The percentile figures are individual request round-trip times from the shared
client. No Web Push subscriptions were active. Both servers used durable
filesystem operations and the same ext4 volume.

## Observed results — November 12, 2026

Host: shared `marceline-dev` Linux (x86-64), ext4 workspace, Node.js v24.21.0,
Rust optimized release build. Values are **operations/second**; higher is
better. Two alternating-order trials, not enough to characterize tails.

| Workload | Trial 1 Node | Trial 1 Rust | Trial 2 Node | Trial 2 Rust |
|---|---:|---:|---:|---:|
| Public GET, 32 KiB | 3,184 | 5,513 | 2,235 | 6,511 |
| CAS PUT, 8 KiB | 6.6 | 52.0 | 49.1 | 51.0 |
| Child grant creation | 171.4 | 212.6 | 170.1 | 196.2 |

Public GET p50 latency was 1.06–1.20 ms with Rust versus 1.98–3.45 ms
with Node. Rust GET throughput was approximately 1.7–2.9× Node in these
measurements. The extremely slow first Node CAS trial is a filesystem
latency outlier: **do not infer a Rust write-speedup** from this small sample.
Grant throughput was modestly faster in these trials, but variability is
material. These numbers change with concurrent host load and filesystem
writeback. Rerun under controlled hardware and production-size loads.

### What changed in the read path

The first experimental Rust benchmark achieved only about 190 GET/s. Each
response incurred about 40 ms of TCP delayed acknowledgment. Explicit
`TCP_NODELAY` on accepted Rust connections removed that penalty. Increasing
the bounded streaming read chunk to 64 KiB avoids excessive network writes.
Object reads still hash through a pinned file descriptor, without a digest
cache; nginx continues to serve public object bytes independently.

## Remaining performance acceptance

Measure a wider spectrum of object sizes, long-running read/write mixtures,
connection and memory saturation, notification-heavy writes, cold cache and
large grant trees, with fixed hardware and I/O visibility. Compare failure
modes as well as throughput; do not optimize away fsync durability or grant
checks merely to improve a benchmark.

## Process-crash correctness workload

Run `make rust-crash-test` to exercise real `SIGKILL` events during concurrent
durable object creates and content-hash-conditional replacements. Each test
round restarts Rust on the same directory and checks that all objects are
complete (never torn), all acknowledged writes persisted, and the transaction
journal was drained. It then opens the same directory under the Node reference
server and reads all objects. Responses lost during a crash are conservatively
classified as unknown (a mutation may have committed without its HTTP reply).
The default is six crashes, eight creates and eight replacements per round;
configure `SKRYNIA_CRASH_ROUNDS` / `SKRYNIA_CRASH_WRITES` for larger sweeps.
This is a fault-injection test, **not** a replacement for real power-loss and
filesystem-corruption testing. It runs separately to preserve the fast-suite
latency budget.

Grant-revocation crash testing (`tests/rust-crash-grants.js`) kills the Rust
process after a durable subtree-revocation marker appears. On restart it
verifies full recovery, descendant token invalidation, subscription and
outbox cleanup, object preservation, and the ability of the original Node
server to read the same directory. This test is part of `make rust-crash-test`
and the Rust preview CI job. Configure
`SKRYNIA_REVOKE_CRASH_ROUNDS` / `SKRYNIA_REVOKE_CRASH_WIDTH` for longer runs.

## Sustained mixed-load consistency test

`make rust-soak` builds the optimized Rust binary, then runs a 12-second
`benchmarks/soak-rust.js` probe with 16 persistent HTTP clients against a
private temporary server instance. Ten percent of each worker's requests
are independent content-hash CAS updates; the remainder are public GETs.
GETs verify the full known payload and matching SHA-256 ETag, and the test
checks that the journal is empty after the workload and that Node reads each
final value after Rust exits. Collects p95/p99 read/write request latency,
throughput, and peak resident memory (`/proc/<pid>/status` when available).
Set `SKRYNIA_SOAK_SECONDS` and `SKRYNIA_SOAK_WORKERS` to control duration and
load; set `SKRYNIA_SOAK_SERVER=node` to run the same fixture against the
reference server. This workload is a probe, not a production-size stress test
or statistically controlled latency benchmark.

### Mixed-load sample — November 12, 2026

The following were separate, sequential eight-second runs on the same
`marceline-dev` host, with 16 keep-alive clients per server, 4 KiB objects,
90% GETs and 10% synchronous SHA-256-CAS replacements. Both servers used
throwaway directories on the same ext4 filesystem and were restarted by Node
to verify final object bytes. These are **individual measurements, not
statistical confidence intervals** or a production throughput claim.

| Metric | Optimized Rust | Node v24.21.0 |
|---|---:|---:|
| Operations completed | 4,946 | 1,797 |
| Operations / second | 598.3 | 223.5 |
| Read p95 / p99 (ms) | 0.40 / 0.55 | 294 / 1,029 |
| Write p95 / p99 (ms) | 385 / 492 | 365 / 963 |
| Peak RSS (MiB) | 10.3 | 79.9 |

Both passed full-payload and ETag assertions, journal-drain checks, and
Node restart verification. No notifications were configured. Durable writes
remain relatively slow under contended shared-host storage. Repeat across
controlled production-like workloads and hardware before qualifying release.

### Filesystem error injection

`tests/rust-disk-failure.js` (included in `make rust-crash-test`) uses
throwaway directories and temporary permission changes, so it must be run as
a **non-root** user on a POSIX system. It makes transaction-journal creation
and staging fail independently; both must return 500, fail closed with
`recovery_required`, and preserve the original bytes. It also prevents Push
outbox persistence **after** a durable mutation, verifies the successful HTTP
response carries `notificationPending: true`, restores permissions, restarts
Rust to recover the journal and notification, and confirms the original Node
server can read the state. These are I/O error tests, not a simulated power
loss or full filesystem corruption model.

### Push durability across real process kills

The `rust-crash-recovery.js` SIGKILL harness also registers a matching browser
Push subscription *before* starting concurrent writes. After each restart,
every recovered create must contribute to a durable notification generation
for that subscription. The test conservatively permits extra generations
caused by replaying a committed intent after a crash, but never permits fewer
notifications than the number of applied creates. Notification delivery is
manual here; the test asserts durable enqueueing, not receipt by a real
external push service.

### Heavy-concurrency saturation sample (32 writers/readers)

A separate 30.8-second *optimized Rust* run on the same shared host with
32 keep-alive clients finished **15,312 verified requests** (13,752 reads,
1,560 SHA-256 CAS writes), about **497 operations/second**, and **13.9 MiB**
peak RSS. Read p95/p99 was 0.42/0.58 ms; durable-write p95/p99 was
**1,110/3,203 ms**. Every payload/ETag assertion passed, the journal drained,
and the final Node restart was successful. An earlier run with a benchmark-side
**4-second** HTTP timeout aborted under this load; the complete sample used
`SKRYNIA_SOAK_REQUEST_TIMEOUT_MS=20000` (and the server's 30-second idle
timeout). The benchmark timeout is now explicit and configurable. This is an
important tail-latency bottleneck under filesystem contention, not a data
corruption finding; it remains a production-capacity qualification item.

### Staging-directory sync optimization

The Rust staging path `fsync`s each complete temporary **file** before the
public hard-link/rename and `fsync`s the destination `store/` directory
afterward. It no longer separately syncs `store-tmp/`: a temporary pathname
is never an authority-bearing durable reference, whereas the committed store
pathname is. After a crash, Rust discards abandoned staging files with the
recognizable Node (24-hex suffix) and Rust (32-hex suffix) formats under the
exclusive writer lock; unrelated filenames are preserved. Regression coverage
includes staged-file cleanup, journal replay, real SIGKILL of concurrent
writes, full-object checks, and Node restart. This reduces redundant metadata
syncs without weakening the durable destination-file requirements.

### Journal finalization: avoid an unnecessary success-path rewrite

The journal's initial durable intent plus the durable committed object is
sufficient for recovery **before the mutation lock releases**. An ordinary
successful Push enqueue now removes that initial journal file directly; it
avoids an extra atomic JSON record rewrite and directory sync on every
successful mutation. If durable Push enqueue fails, Rust still persists the
`committed:true` marker *before* returning `notificationPending`, allowing
later writes to the same key without losing that earlier notification on
recovery. Failure to persist the marker itself is fatal and forces recovery;
the server must not continue accepting writes with an unprotected pending
notification. The mutation lock, journal recovery, failure injection and
Node/Rust on-disk interoperability are covered by dedicated tests. The
at-least-once notification contract remains unchanged.

### Journal fast-path measurements (Nov 13 UTC, 2026)

Release builds of exact commits `d53dccb` (durable committed-marker rewrite
on every mutation) and `b8a6f3c` (only persist the marker on enqueue failure)
were verified to have different SHA-256 binary hashes. We ran **four**
sequential, alternating 32-worker trials (baseline, optimized, optimized,
baseline) on `marceline-dev`: 10 seconds per trial, 4 KiB objects, 90% public
GETs and 10% synchronous content-hash CAS writes, 20-second client timeout,
private throwaway data directories. Every trial passed complete-byte and
ETag assertions, drained the transaction journal and reopened the data
successfully under Node.

| Run | Build | Operations/s | Write p95 ms | Write p99 ms | Peak RSS MiB |
|---|---|---:|---:|---:|---:|
| 1 | d53dccb | 535.6 | 876.0 | 1,274.1 | 12.5 |
| 2 | b8a6f3c | 779.5 | 645.8 | 883.4 | 12.5 |
| 3 | b8a6f3c | 754.6 | 609.9 | 807.1 | 12.7 |
| 4 | d53dccb | 526.8 | 872.3 | 1,151.7 | 12.3 |

The improvement is about 44% in measured operations/second across these two
runs per build. p99 writes remain hundreds of milliseconds, and the
experiment cannot determine production tail latency or a confidence interval.
Compare again on target disks, with more trials and realistic notification
subscriptions before authorizing a production migration.

### Pre-staged durable writes outside the global mutation lock

The implementation at `b2ccd43` checks the caller's grant, quota, CAS and
notification capacity before staging an upload. It writes and `fsync`s the
staged object **outside** the global state lock, then reacquires that lock,
rechecks all preconditions and atomically commits the staged inode through the
existing durable journal. An RAII guard removes staged files on failed
revalidation. The public store directory is still `fsync`ed before success.
The pre-commit recheck keeps same-key compare-and-swap and shared-grant quotas
serialized even if several uploads were staged concurrently. Regression
checks include 16-way competing creates/CAS/deletes, 16-way independent
creates against a one-object quota, no leftover staging files, real SIGKILL,
filesystem faults, and cross-runtime Node restart.

A matched, **four-run ABBA** comparison used a release binary from `515982a`
(baseline) and a distinct verified release binary with the pre-staging change
(optimized). Each sequential run on shared `marceline-dev` used 32 persistent
clients, 10-second steady-state workload, 4 KiB objects, 90% GET and 10%
SHA-256 CAS PUT, a 20-second request timeout, and fresh ext4 directories.
Each verified bytes/ETags, a drained journal, and a Node restart:

| Run | Build | Operations/s | Write p95 ms | Write p99 ms | Peak RSS MiB |
|---|---|---:|---:|---:|---:|
| 1 | baseline | 411.3 | 3,285.72 | 4,851.85 | 13.0 |
| 2 | pre-staged | 942.1 | 506.99 | 592.95 | 14.4 |
| 3 | pre-staged | 897.3 | 557.92 | 616.93 | 13.9 |
| 4 | baseline | 460.0 | 2,525.17 | 5,453.15 | 15.2 |

These trials indicate roughly 2.1× throughput and substantially lower write
p99 under this exact workload. They are **not** production performance claims:
only two runs per build were measured on a shared development filesystem, and
the request mix had no Web Push subscriptions. Repeat on deployment hardware
with long runs, simultaneous notification delivery, larger objects, memory
limits and controlled background I/O before release.

### GitHub container memory qualification — November 13, 2026

Rust preview CI run `43384772774` verified the Docker image built at
`67362e9` under a *real enforced* 96 MiB cgroup limit, non-root UID 1000,
and a 24 MiB aggregate upload buffer. A burst of 24 independent 4 MiB
POSTs produced 7 acknowledged successes, 10 explicit `upload_memory_full`
HTTP 503 responses and 7 early transport closures from requests rejected
while transmitting. The probe treats early transport closure as an ambiguous
write outcome; it independently GETs **every key**, requiring either absence
or the exact original bytes and matching SHA-256 ETag. All surviving objects
were intact. No OOM kills occurred. The journal and staging directory were
empty, and the original Node server reopened the persisted state successfully.
This was one burst under cgroup pressure, not a long-running load
benchmark. The local Podman host did not enforce the cgroup limit and its
corresponding run is only a protocol/integrity smoke test.

### Refreshed optimized-build operation benchmark — November 13, 2026

`SKRYNIA_BENCH_REPETITIONS=2 SKRYNIA_BENCH_OBJECTS=32 node
benchmarks/compare-rust.js target/release/skrynia-server` was run on the
same shared `marceline-dev` filesystem. The Rust release binary was verified
to have SHA-256
`5618ac42c9e1e5eec98efbc2dda71794b18227921d42c997d257086c655bc989`,
matching the optimized pre-staged build. Order alternated **Node, Rust,
Rust, Node**, and each invocation used its own private data directory.
The benchmark prepopulates 32 objects, then measures 300 public GETs of
32 KiB values at concurrency 8, 32 content-hash CAS PUTs of 8 KiB at
concurrency 4, and 24 child-grant creations at concurrency 4.

| Trial | Server | GET ops/s | CAS PUT ops/s | Child grants/s |
|---|---|---:|---:|---:|
| 1 | Node | 1,954.8 | 41.8 | 162.1 |
| 1 | Rust | 3,671.3 | 79.4 | 187.1 |
| 2 | Rust | 5,765.5 | 90.6 | 199.6 |
| 2 | Node | 3,717.9 | 53.5 | 184.1 |

The corresponding CAS PUT p95 latencies were **152.54 / 119.31 ms** for
Node and **72.92 / 61.29 ms** for Rust, in trial order. Child-grant p95
was **37.94 / 35.30 ms** for Node and **24.85 / 22.66 ms** for Rust.
These are single-host samples with few durable writes and grant creations,
not confidence intervals or a sustained production workload. They support
continued Rust performance qualification without asserting a universal
speedup or weakening filesystem synchronization.
