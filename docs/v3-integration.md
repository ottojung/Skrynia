# Skrynia v3 isolated integration (development only)

Status: experimental, **not a release candidate**. Skrynia v3 exposes
binary POST only; the withdrawn v2 HTTP/grant/Web Push compatibility goals
are not a release gate. Apps migrate to native Program 0 protocols and
separately hosted static HTTPS origins. Do not replace an existing v2
installation until independent migration and rollback are verified.

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

## Root validation at startup

Every startup compiles the operator-provided root module, then checks its
\`memory\` export, the exact \`skrynia_main(i32,i32)->i32\` signature, and the
privileged host import allowlist and function signatures. Unknown/WASI or
mistyped imports fail startup before the data directory is opened. The kernel
does **not** execute root WASM as part of this static validation. Traps and
resource failures during an actual request still produce VM/transport errors.

## Security boundaries and unfinished release gates

Program 0 uses explicit kernel disk and evaluation calls; requests, compiled
modules and all VM linear memories remain separate from global RAM. The kernel
has per-request fuel, stack and StoreLimits plus a bounded, SHA-256-keyed
compiled guest cache shared across requests. Guest imports other than
`skrynia.invoke` are rejected.

These checks do **not** establish full v3 correctness. Remaining work
includes operator capability issuance and authorization, native versioned
code activation and rollback, stress and failure injection, independent
security review, and reproducible app-data migration. No legacy HTTP
GET/grants/Web Push facades are required. No production cutover is
authorized by these tests.

## Request-scoped mutation serialization

An optional root may call skrynia.try_mutation_lock() -> i32: 1 acquires
a kernel-owned global mutation lock, 0 means another request holds it.
Only its owner can call skrynia.release_mutation_lock() -> i32 and get 0;
nonowners get -4. The lock is not recursive. These imports are root-only
and are not inherited by evaluated child programs.

The kernel tracks ownership on the request VM and closes its descriptors
before releasing the lock when that VM terminates, including due to a
WASM trap, fuel exhaustion, or failed output transport. It never steals a
live request's lock using timestamps. An HTTP disconnect alone does not
terminate a still-running worker and thus cannot release its lock early.
The open_ram and cas_ram_u64 imports remain general shared memory
operations and are not automatically rolled back on VM termination.

The current VM fuel limit is 128 million units per root request.
A fuel-exhausted root may fail, but it must not disable the next mutation.
This lock serializes Program 0's mutation calls but does not turn a set
of filesystem syscalls into a durable transaction; the operator-supplied
Program 0 remains responsible for atomic publication and crash recovery.
