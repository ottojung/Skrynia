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
