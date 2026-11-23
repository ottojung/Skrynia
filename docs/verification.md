# Verification — current Skrynia v3 kernel

The native kernel exposes exactly one endpoint, `POST /`, accepts only
opaque binary request bodies and executes a separately operator-supplied root
WASM. It does not provide a native health, grant, store, version or Web Push
API. The example in `skrynia-tools` is an optional dumb token-gated proxy;
business logic lives in ordinary nonzero userland programs.

## Source, VM and isolation gates

Use Rust 1.93 (OpenSSL headers and pkg-config installed):

```sh
make test
```

This checks default rustfmt, Clippy, 50-code-line/10-entry structural limits
and regression suites for the generic VM ABI, input/output directional file
descriptors, isolated operator/user disk roots, WASM module validation,
resource limits, nested eval/callback capabilities, HTTP failure handling,
bounded input backpressure and cancellation, RAM CAS, and restart behavior.
The kernel stores no app-specific grants, quota ledger or subscriptions.

A green test suite proves source-level invariants in a disposable environment,
not real production persistence, external push delivery, backup correctness
or a safe deployment. Real app-specific end-to-end acceptance belongs to the
userland service repositories and must use compiled, pinned WASM artifacts.

## Reproducible current-kernel HTTP benchmark

```sh
SKRYNIA_BENCH_SAMPLES=100 make benchmark
```

The Rust example builds and launches the current v3 kernel on an ephemeral
loopback port with a disposable, separately compiled WAT-derived Program 0
root. It configures distinct temporary `SKRYNIA_OPERATOR_DIR` and
`SKRYNIA_USER_DIR`; no live data, API key or production service is used.

The measured sequential workloads are:
- `VM POST / no-op`: one-byte request read and one-byte output write.
- `VM POST / disk write+file sync 4 KiB`: stream and write 4096 bytes into
  an isolated user-root file, sync it and acknowledge. The benchmark
  independently verifies exact file contents and absence of an operator-root
  copy before cleaning its disposable data and terminating the child.

The example's own Rust test (`cargo test --example skrynia_bench`) exercises
the real `boot::Kernel` against the identical WASM fixture and checks both
no-op and synced write paths. The benchmark does **not** measure the obsolete
v2 GET/conditional PUT protocol. Timing on a shared host is diagnostic only:
repeat under production-equivalent hardware, multiple concurrency levels
and realistic workloads before citing latency, capacity or a speedup.

## Release and operation boundaries

Before production: verify exact release commit+artifact digests; independently
review the separate kernel, operator's Program 0 and ordinary WASM P1/P2;
validate deploy/rollback, verified backup and restore, confidential
operator/user roots, HTTPS/DNS/rate limits, customer capability provisioning,
real browser/provider Web Push and app migrations using production-like data.

For previous v2-only grant/HTTP/storage timing, see the separately labeled
[historical v2 verification](verification-v2-history.md). Do not use the v2
endpoints or initialization instructions with the v3 kernel.
