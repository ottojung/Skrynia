# Skrynia v3 isolated integration (development only)

Status: experimental, **not a release candidate**. This branch provides a generic
VM kernel with binary `POST /` only; no v2 HTTP/grant/Web Push compatibility
facade is included. Do not deploy over an existing v2 installation.

## Build and test

Skrynia is **only the native VM server**. It must not ship or
compile a privileged Program 0. Any user may supply a conforming
WASM module at `SKRYNIA_ROOT_WASM`.

The separate `skrynia-tools` repository offers one optional Program 0
implementation; it is **not** a dependency of the kernel. Users are free
to provide completely different implementations.

The root module exports `skrynia_main(input_fd:i32,output_fd:i32)->i32`.
The input is a read-only, forward-only HTTP body descriptor and output is a
write-only, forward-only response descriptor. Both may be closed; neither
supports seeking, truncation or syncing. Successful return is exactly `0`;
a negative return is a VM failure. `write` may be short; the program must
retry for bytes it still needs to send. The kernel uses bounded channels with
backpressure, not a fixed WASM response buffer or global RAM. A failure after
bytes have already been sent terminates the HTTP body with an error rather
than reporting a successful clean EOF. Child programs still expose the
separate invoke-only ABI: no privileged filesystem, RAM or HTTP descriptors
are inherited.

Run `make test` for conformance tests using isolated, self-contained
WebAssembly fixtures. The server repository has **no tests, example
applications, deployment entry points, or build steps that assume the binary
protocol of a particular Program 0**.

The optional Program 0 and its application-specific HTTP integration tests
live entirely in `skrynia-tools`. Cross-repository verification should run
against a disposable v3 server whose root module was separately built by its
owner, never against a deployed v2 data directory.

## Resource-bounded WebAssembly compilation

The kernel compiles root and evaluated guest modules in a **separate, killable
worker process**, not a detached thread. Worker input/output pipes are
handled independently of the watchdog so a hung compiler cannot indefinitely
block the parent while the body is being sent. At most four workers execute
concurrently; additional requests wait for a bounded slot. Compilation is
limited to 8 MiB source, 64 MiB serialized output, 3.5 s wall-clock
execution, 4 s CPU time and 512 MiB virtual address space per worker.
The parent kills and reaps a worker on timeout or wait failure; input/output
thread handles are joined after the worker has exited. A successful
precompile is deserialized under the caller's matching Wasmtime engine.
Input compile errors map to an invalid-module error; timeout/resource
errors map to a limit error. The worker receives an empty environment and cannot inherit application
secrets via environment variables. It runs with no-new-privileges on Linux
and ordinarily uses only compiler memory and pipes; it still has the server
UID and OS filesystem permissions. These per-worker limits are **not**
a full seccomp, filesystem namespace or privilege-separation sandbox.

## Root validation at startup

Every startup compiles the operator-provided root module, then checks its
\`memory\` export, the exact \`skrynia_main(i32,i32)->i32\` signature, and the
privileged host import allowlist and function signatures. Unknown/WASI or
mistyped imports fail startup before the data directory is opened. The kernel
does **not** execute root WASM as part of this static validation. Traps and
resource failures during an actual request still produce VM/transport errors.

## HTTP execution admission

The HTTP adapter admits at most **16 concurrent native VM invocations**;
each admitted request holds its permit until its blocking WASM worker
actually exits, even if HTTP response bytes have already begun streaming.
An excess request fails before starting any VM instance (HTTP 500, the
VM resource-exhaustion response), rather than accumulating new Wasmtime
stores and their 64 MiB per-store memory budgets. Invalid method/path
and declared-overlength request bodies are checked before admission.
A deterministic test holds sixteen requests after early output, rejects
the seventeenth, then proves capacity returns when inputs end.

This limit bounds the count of active HTTP-invoked VM stores, **not**
overall service lifetime or disk I/O latency. A maliciously stalled upload,
a hung filesystem operation, or a disconnected slow client can still hold
a worker. Robust input inactivity/total wall-clock deadlines remain a
separate release security and availability gate.

## Security boundaries and unfinished release gates

Program 0 uses explicit kernel disk and evaluation calls; requests, compiled
modules and all VM linear memories remain separate from global RAM. The kernel
has per-request fuel, stack and StoreLimits plus a bounded, SHA-256-keyed
compiled guest cache shared across requests. Guest imports other than
`skrynia.invoke` are rejected.

These checks do **not** establish full v3 correctness. Remaining requirements
include application authorization/delegation and complete recursive callback
support, cross-request transaction coordination for registry changes, durable
revoke semantics, eliminating v2 business logic from the new native server,
full v2 HTTP/grant/quota/Web Push compatibility in `skrynia-tools`, production
resource limits, comprehensive fault injection, independent security review,
and reproducible migration with rollback. No production cutover is authorized
by these tests.

## Generic parent callbacks and nested descriptor authority

Evaluated WASM guests can import only read/write/close on delegated descriptors
plus the generic invoke callback. While handling invoke, the kernel temporarily
reinstates the **parent's** descriptor authority for the operator-provided
Program 0 callback, then **unconditionally restores the calling guest's**
descriptor restrictions even after a nested evaluation or a callback trap.
Without this context switch, root callbacks cannot read their own open disk
handles (ERR_BADFD); when root callbacks do nested eval, the outer guest
could instead lose its descriptor restrictions and gain access to root disk
handles. Both were reproduced using actual WAT/VM tests before the fix.

These permissions are scoped to generic parent/child evaluation. Neither
Skrynia's HTTP server nor the VM kernel names or configures Program 1,
application versions, v2 routes or any nonzero program. The parent callback
may implement *generic* host-API forwarding, but is not application logic.
Test fixtures under test-fixtures/callback cover parent private disk reads,
nested unprivileged execution, and rejection of attempted descriptor theft.
Parallel HTTP fixture roots use unique directories to prevent test races.
