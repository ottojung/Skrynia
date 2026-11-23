$id-6212596314252106
title: High-performance Rust server
date: 2026/11/12
source: @ottojung
kind: requirement

Skrynia's authoritative HTTP and persistence server shall be implemented exclusively in Rust. Preserve crash recovery, and privilege-boundary semantics. Unsupported persistent state must fail closed, never be ignored or discarded. The server must pass HTTP, persistent-state, crash and concurrency qualification.

$id-0381283944881954
title: Rust source quality and verified throughput
date: 2026/11/12
source: @ottojung
kind: requirement

Adopt the Ivanka structural linter: at most 50 nonblank, noncomment Rust code lines per source file; at most 10 direct entries per directory in Rust source/lint/test trees; sibling-only local imports. Use unmodified default rustfmt and enforce these requirements with CI, tests, and strict Clippy correctness, suspicious, performance, and complexity checks. The server must avoid blocking asynchronous executor workers with durable filesystem work. Report measured throughput and latency using reproducible workloads on the same hardware and filesystem when evaluating optimizations; accept performance changes only when they improve measured results without sacrificing durability or authority.

$id-8052543016377230
title: Filesystem-aware complexity and resource accounting
date: 2026/11/16
source: @ottojung
kind: requirement

Use docs/complexity.md as the normative framework for analyzing Skrynia's complexity. Parametrize filesystem lookup, read, write, namespace mutation, enumeration, and synchronization costs instead of assuming universal O(k) lookup or O(b) disk I/O. Count server CPU, RAM, live/peak persistent space, filesystem-operation traces, concurrency, and recovery separately. Permit disk-backed indexes, temporary files, and durable private metadata, with their costs and consistency obligations explicitly charged. Distinguish current implementation bounds, proved conditional bounds, measurements, and unimplemented optimization goals; do not weaken production durability semantics.
