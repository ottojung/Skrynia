# Skrynia v3 ABI decisions

These ABI decisions accompany the native v3 kernel VM contract. The kernel
has no knowledge of the application protocol spoken by an operator-supplied
root program.

## Nested eval callbacks

**Decision:** a child module imports exactly one host function,
`(import "skrynia" "invoke" (func (param i32 i32 i32 i32) (result i32)))`
with parameters `(req_ptr, req_len, res_ptr, res_cap)` in the *child*'s
private linear memory. The parent selects a callback per `eval` by passing
`cb_table_idx`, an index into the parent's exported funcref table
`__indirect_function_table`. The callback element must have the same
signature and executes in the *parent* instance context.

**Rationale:** `invoke` is the only capability surface exposed to untrusted
children. Binding per `eval` avoids ambient globals and avoids exposing parent
function pointers/memory. The table form is standard and host-validatable.
`cb_table_idx < 0` means no callback; any child `invoke` returns `ERR_PERM`.

**Memory/ownership:** kernel copies request bytes child→kernel→parent and
response bytes parent→kernel→child. No linear memory, tables, globals, stack,
or descriptors are shared. Pointers are always interpreted in the accessed
instance's private memory.

## Private pointer ownership and lifetime

**Decision:** every pointer argument is owned by the addressed instance's
current private linear memory for the duration of the synchronous host
operation and bounds-checked against that memory. `eval` snapshots child code
and input bytes before validation/execution so caller-memory mutation cannot
change validated/executed bytes. Output is written only to explicitly provided
or kernel-selected I/O buffers as defined below.

## Safe transient I/O buffer allocation

**Decision:** kernel-selected transient buffers (child `skrynia_main` input and
output window; parent callback request and response regions for an `invoke`
turn) are allocated by the kernel in the target instance's linear memory at
addresses `>= base_size`, where `base_size` is that memory's byte size before
buffer preparation for the turn. The kernel may grow linear memory by zeroed
pages to make space, subject to limits; it never writes kernel-selected buffer
bytes below `base_size`. Therefore kernel buffer preparation cannot overwrite
instantiated data segments, active stack contents, or active heap contents that
existed before the turn. Buffers are turn-scoped and may be reused after the
turn. Guest-supplied buffers (caller-chosen eval I/O, `invoke` req/res ranges,
disk/RAM/random/entry buffers) are bounds-checked and accessed only within the
supplied range; guests must choose them so writes do not corrupt required
state. This contract is valid for modules with non-zero data segments.

## Output-buffer overflow without repeated side effects

**Decision:** `eval` and `next_entry` are single-shot. On output overflow the
host performs no second execution, copies no partial output, does not advance
iteration cursor, returns `ERR_OVERFLOW`, and stores required length in a
per-caller-instance slot readable via `eval_output_len() -> i32` (last
overflow-required length, else `-1`). Callers may allocate and intentionally
re-run/retry only when they choose a fresh operation; the failed call itself
is never implicitly replayed.

## Response/status semantics

**Decision (authoritative):** Program 0 exports
`skrynia_main(input_fd: i32, output_fd: i32) -> i32`. Both handles are
request-local, opaque descriptors. The input accepts only `read` and `close`;
the output accepts only `write` and `close`. Both are forward-only.
The kernel never copies a whole response from guest memory or implicitly
uses global RAM. Program 0 returns `0` on success, a negative VM-visible
error on failure, and no byte-count result. `write` may be partial.

For `POST /`, kernel connects the body to input and the output descriptor to
an incrementally consumed HTTP response using bounded staging/backpressure.
An error before output is emitted can become HTTP 500; an error after output
is emitted closes the HTTP stream with an error (never clean EOF or automatic
re-execution). Child modules still use a distinct, invoke-only interface:
`skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32, out_cap: u32) -> i32`
because they cannot import privileged `read`/`write`; their caller may
provide mediated stream behavior through `invoke`.

## RAM CAS ABI and atomicity

**Decision:** `cas_ram_u64(ram_fd, offset, expected, desired, observed_ptr) -> i32`
returns `0` on a completed CAS operation and writes the observed old `u64`
(little-endian, full 64-bit domain including high-bit/all-ones patterns) to
`observed_ptr` in caller private memory. Negative returns are `ERR_*` status
codes and perform no RAM mutation and no observed-value write, so API errors
cannot be confused with observed bit patterns that match negative `i32` codes.
`cas_ram_u64` operations are linearizable and sequentially consistent with all
`cas_ram_u64` operations. Ordinary RAM byte `read/write` are host-safe (no Rust
UB/kernel corruption) but not atomic with overlapping ordinary or CAS accesses;
overlapping observations are program-level unspecified except for safety.
Correct synchronization requires CAS/alignment/non-overlap protocols.

## Descriptors, paths, and crash details

**Decision:** descriptors are per-request opaque `i32` handles, recycled only
after close within that request scope. Paths confined by race-resistant
traversal from data root: reject NUL, absolute, empty/`.`/`..` segments,
symlink escape, and >4096 bytes. `rename` is atomic same-filesystem
replacement where supported; cross-device returns `ERR_UNSUPPORTED`; durability
requires explicit `sync` of file and parent directory. Un-synced disk state
may be lost/reordered on crash consistent with filesystem guarantees; all RAM
and descriptors are lost on process restart.

## Coordination notes

- **Kernel (runtime implementation):** must implement the unambiguous
  `cas_ram_u64(..., observed_ptr) -> i32` status+out-parameter ABI, transient
  buffer allocation via pre-turn memory growth (never clobbering existing data
  segments/stack/heap), strict bounds/import validation, and CAS linearizability
  without implicit RAM access.
- **Program 0:** must treat `cas_ram_u64` status separately from observed
  values, provide valid observed/result buffers, and not assume kernel I/O
  buffers persist across turns. Application protocol behavior is out of scope
  for this v3 contract except compatibility boundaries stated in the spec.

## Qualification boundary

These VM behavior rules are implemented in the Rust v3 kernel. Test fixtures
exercise ABI validation, callback isolation, memory limits, physical resource
limits, filesystem confinement and streaming. Compliance of an independently
supplied root WASM, its child programs and their application-level protocols
is a separate userland qualification; it is not implied by kernel tests.

## Root request/response descriptors

The native kernel accepts only user-supplied Program 0 modules; the optional
`skrynia-tools` implementation is just one such module.

The root entrypoint is `skrynia_main(input_fd:i32, output_fd:i32)->i32`.
The first descriptor reads request bytes into WASM-private buffers on demand;
the second writes bytes from WASM-private buffers to the HTTP response on
demand. `close` works on either; all other operations besides `read` on
input and `write` on output return `ERR_PERM`. Neither has a seek cursor.
Invalid memory ranges never consume or emit bytes. Errors do not masquerade
as clean EOF. A short write is permitted; callers can write until done.
The kernel must not stage entire request or response bodies in global RAM or
private WASM memory; use bounded, backpressured transient kernel buffers.
