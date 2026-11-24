$id-6212596314252106
title: High-performance Rust server
date: 2026/11/12
source: @ottojung
kind: requirement

Skrynia's generic HTTP transport and WASM execution kernel shall be implemented exclusively in Rust. Preserve filesystem durability and privilege-boundary semantics. An invalid root WASM, unsafe path or host I/O failure must fail closed. Application-level persistent schemas, journals and recovery belong to userland programs, not the native kernel. The kernel must pass transport, filesystem, sandbox, crash and concurrency qualification.

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

Analyze the actual generic WASM kernel using a filesystem-aware resource
model. Count CPU, peak RAM, live/peak disk, file operations, kernel VM work,
concurrency and crash recovery separately. State the assumptions under which
complexity bounds hold and distinguish measured results from targets. Avoid
silently equating filesystem operations with constant-time access. Do not add
application object, grant, notification, or quota semantics to the native
kernel in pursuit of an optimization. Preserve durability and sandbox safety.

$id-4160689473985217
title: Operator-extensible kernel syscalls
date: 2026/11/23
source: @ottojung
kind: requirement

The main repository shall provide the libskrynia Rust library for custom
kernel builds. Program 0 may import arbitrary operator-configured names
from the syscall namespace with the signature i32 -> i32. Native
implementations are registered at startup. An optional HTTP provider is
configured only by SKRYNIA_SYSCALL_URL and advertises its names at startup
through POST <base>/version; the kernel validates, sorts, deduplicates and
merges those names with native registrations, with native names winning.
Resolve imports once at startup without runtime name dispatch; there is no
operator-supplied syscall name list. The kernel must not understand
capability-specific argument or result formats. Evaluated children must not
receive direct custom imports. POST /version shall expose build version,
commit and the supported custom syscall names.
