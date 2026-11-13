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

Adopt the Ivanka structural linter: at most 50 nonblank, noncomment Rust code lines per source file; at most 10 direct entries per directory in Rust source/lint/test trees; sibling-only local imports. Use unmodified default rustfmt and enforce these requirements with CI, tests, and strict Clippy correctness, suspicious, performance, and complexity checks. The server must avoid blocking asynchronous executor workers with durable filesystem work. Report measured throughput and latency using reproducible workloads on the same hardware and filesystem when evaluating optimizations; accept performance changes only when they improve measured results without sacrificing durability or authority.

$id-8052543016377230
title: Idealized execution and asymptotic complexity targets
date: 2026/11/13
source: @ottojung
kind: requirement

Adopt docs/idealized-complexity.md as the explicit hypothetical environment for reasoning about Skrynia's complexity, without weakening actual production durability requirements. Assume permanent execution without exogenous crashes, O(k) filesystem key lookup, O(b) byte reads/writes, atomic one-key mutations as ideal primitives, and no auxiliary or temporary persistent data beyond client-written objects. Require store POST and PUT O(k+b+h+s) time with O(Gk) total RAM for grant/quota/object-index structures, initialization of a previously unseen grant scope's counters in O(Nk), and direct object GET O(k+b). Count SHA-256 precondition reads in b and matching subscription work in s; account separately for credential, subscription, and notification memory.
