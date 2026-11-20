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

## Operator-supplied streaming Program 1 (draft)

The optional root-only import run_program1(input_fd, output_fd) transfers two
forward-only descriptor capabilities into a distinct operator-supplied
Program 1 WASM VM. Program 0 may simply call this import and return its
status; the root need not load program code, buffer application bytes,
inspect protocol frames, or implement stateful application semantics.

Operators independently provide SKRYNIA_PROGRAM1_WASM. The kernel validates
and compiles the Program 1 module at startup, before serving requests. If
Program 1 is missing, run_program1 fails closed. Its import allowlist is
restricted to read, write and close; a child attempting seek, truncate,
sync, filesystem paths, global RAM or root eval fails startup validation.
The input FD is read-only, the output FD is write-only. Code runs with
independent memory and finite fuel. Root may not steal back a transferred
descriptor until Program 1 terminates. Large requests and responses
are processed in bounded chunks rather than buffered into Program 0.

This is a transport capability boundary, NOT yet a durable stateful
program API. A nonzero user program needs a separately specified, explicit
least-authority persistent storage/CAS service before handling application
data or immutable code activation; that must not be implemented as
business logic inside the dumb proxy root. Do not deploy this draft
as a production replacement for existing v2 data.

## Explicitly scoped Program 1 persistence (draft)

The operator may *optionally* configure SKRYNIA_PROGRAM1_DATA_DIR as an
existing private directory, separate from SKRYNIA_DATA_DIR and outside any
production v2 data. This configuration grants only the operator-installed
Program 1 WASM a bounded filesystem-root capability. By default it is
absent and Program 1 cannot import ANY filesystem, random, mutation or
nested-evaluation functions, only read/write/close on its two HTTP stream
descriptors. Supplying the directory without Program 1 is a startup error.

When the capability is explicitly configured, the kernel opens that
directory once via O_DIRECTORY | O_NOFOLLOW and uses its independently
owned root FD for **all** Program 1 path operations. The existing
relative-path validator disallows parent traversal, and component-wise
openat operations disallow symlinks. Program 1 may open/read/write/sync
files, create/delete/rename entries, scan directories, acquire/release the
kernel-owned request-scoped mutation lock, draw secure randomness, get the
clock and execute sandboxed nested WASM. It cannot import open_ram,
cas_ram_u64 or run_program1, nor access paths in SKRYNIA_DATA_DIR unless
the operator explicitly places them under the chosen directory.
Nested evaluated programs do not inherit these host capabilities: their
only import remains skrynia.invoke, mediated by Program 1 callbacks.

This is a **generic VM capability** and does not implement application
storage protocols, code release history, authorization or deployment in
the kernel or root Program 0. The separate user Program 1 must implement
all such semantics and must validate untrusted client requests itself.
The operator must choose a dedicated confined directory: this capability
provides full read/write authority WITHIN the directory, not per-key
quotas, revocation or resource isolation from other instances sharing
the same directory. Per-app quotas, data migration, independent security
review and failure-injection acceptance are still required before
production. Root remains only the forwarding program.
