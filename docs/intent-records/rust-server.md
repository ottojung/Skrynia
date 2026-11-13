$id-6212596314252106
title: High-performance Rust server
date: 2026/11/12
source: @ottojung
kind: requirement

Skrynia's authoritative HTTP and persistence server shall be implemented exclusively in Rust. Preserve storage, grant, quota, conditional byte-hash mutation, crash recovery, public static serving, Web Push, and privilege-boundary semantics. Unsupported persistent state must fail closed, never be ignored or discarded. The server must pass HTTP, persistent-state, notification, crash and concurrency qualification. All application meaning stays exclusively in clients.

$id-0381283944881954
title: Rust source quality and verified throughput
date: 2026/11/12
source: @ottojung
kind: requirement

Adopt the Ivanka structural linter: at most 50 nonblank, noncomment Rust code lines per source file; at most 10 direct entries per directory in Rust source/lint/test trees; sibling-only local imports. Use unmodified default rustfmt and enforce these requirements with CI, tests, and strict Clippy correctness, suspicious, performance, and complexity checks. The server should avoid blocking asynchronous executor workers with durable filesystem work and demonstrate meaningful throughput/latency improvements using reproducible same-hardware, same-filesystem workloads without sacrificing durability or authority.
