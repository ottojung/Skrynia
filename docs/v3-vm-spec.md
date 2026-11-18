# Skrynia v3 virtual machine — executable contract

Status: **draft, phase-1 contract with prototype conformance checks**.
Milestone: Antonina `dgKBALkpO7M7oKBHrZUzCT-shortly`.
This document defines the v3 replacement kernel contract, not v2.
All ABI types are for WebAssembly 1.0 MVP linear memory on wasm32.
Pointers/lengths/offsets/capacities/counts are `u32` unless stated.

## Kernel boundary

Skrynia is a single-node WebAssembly computer. The Rust kernel provides only
HTTP transport, module validation/instantiation, sandboxed execution,
filesystem operations, global RAM primitives, time, randomness, and physical
safety limits. The kernel MUST NOT implement grants, quotas, accounting,
authentication, registration, lookup, mutable `set`, routing, program IDs,
v2 object-store semantics, or any implicit capability distribution.

Program 0 is configured by `SKRYNIA_ROOT_WASM`, loaded/validated/compiled at
every process initialization and restart. `SKRYNIA_DATA_DIR` is the confined
filesystem subtree for privileged disk access. Initialization fails closed.

## HTTP and response contract

The only endpoint is `POST /` with `Content-Type: application/octet-stream`
and an arbitrary binary body. No selector header, path parameter, query
routing, or kernel interpretation of body structure.

For each request the kernel performs: read request body into kernel memory,
invoke Program 0 once with that body as input, collect Program 0 output from
its private memory, then return that output. Each request is a fresh logical
Program 0 invocation: no shared private WASM globals, stack, or linear memory.
Compiled Program 0 code may be cached; execution state must not be reused.

Successful VM completion returns HTTP `200` with
`Content-Type: application/octet-stream` and the exact output bytes produced
by Program 0. VM completion means the Program 0 entry returned normally and
the kernel copied its declared output without trap or kernel fault.

Transport/VM failures are distinct from application failures:
- malformed request framing or unreadable body: `400`
- method/path not `POST /`: `404`
- configured Program 0 invalid or unavailable: `503`
- request body exceeds `SKRYNIA_MAX_REQUEST_BYTES` (default 16 MiB): `413`
- Program 0 trap, resource-limit exhaustion, invalid ABI use, or kernel
  execution fault: `500`
- Program 0 produced no output descriptor or invalid output range: `500`

Application-level errors are encoded by Program 0 in its output bytes and are
returned as HTTP `200` unless Program 0 fails at the VM level. The kernel does
not inspect output meaning. Legacy v2 HTTP behavior belongs in `skrynia-tools`.

## Memory domains and ownership

1. **Private WASM linear memory**: separate linear memory per module instance.
   Every pointer passed to or from host imports/exports addresses this memory.
   A guest never receives a pointer to another instance's memory or to RAM.
2. **Kernel-owned memory**: request/result staging, frames, descriptors,
   compiled artifacts, and transient copies. Never exposed as guest pointers.
3. **Global RAM**: one contiguous volatile byte-addressable object, shared by
   all invocations that hold/use a RAM descriptor. Empty after process restart.

No runtime component may implicitly read, write, allocate, mirror, or cache
request bodies, private memory, compiled code, descriptors, or kernel metadata
in global RAM. RAM is touched only by explicit `open_ram/read/write/seek/
truncate/cas_ram_u64` operations. Global RAM offsets are not capabilities.

## Root imports (Program 0 only)

Program 0 is instantiated with module name `skrynia` and exactly these imports.
Types: `i32` return values are status/length codes as defined below.

```text
open_disk(path_ptr: u32, path_len: u32, mode: u32) -> i32
truncate(fd: i32, size: u64) -> i32
seek(fd: i32, offset: u64) -> i32
read(fd: i32, dst_ptr: u32, len: u32) -> i32
write(fd: i32, src_ptr: u32, len: u32) -> i32
close(fd: i32) -> i32
mkdir(path_ptr: u32, path_len: u32, mode: u32) -> i32
exists(path_ptr: u32, path_len: u32) -> i32
delete(path_ptr: u32, path_len: u32) -> i32
rename(old_ptr: u32, old_len: u32, new_ptr: u32, new_len: u32) -> i32
open_directory(path_ptr: u32, path_len: u32) -> i32
next_entry(dir: i32, dst_ptr: u32, cap: u32) -> i32
is_dir(path_ptr: u32, path_len: u32) -> i32
open_ram() -> i32
cas_ram_u64(ram_fd: i32, offset: u64, expected: u64, desired: u64) -> i64
sync(fd: i32) -> i32
now() -> i64
secure_random_bits(dst_ptr: u32, bits: u32) -> i32
eval(code_ptr: u32, code_len: u32, input_ptr: u32, input_len: u32,
     cb_table_idx: i32, out_ptr: u32, out_cap: u32) -> i32
eval_output_len() -> i32
```

Status codes: `>= 0` means success/value as specified; negative means error.
`ERR_INVALID = -1`, `ERR_NOT_FOUND = -2`, `ERR_EXISTS = -3`,
`ERR_PERM = -4`, `ERR_IO = -5`, `ERR_NOSPACE = -6`, `ERR_BADFD = -7`,
`ERR_RANGE = -8`, `ERR_FAULT = -9`, `ERR_LIMIT = -10`,
`ERR_OVERFLOW = -11`, `ERR_UNSUPPORTED = -12`.

## `eval` and nested `invoke` ABI

Program 0 and every evaluated child must export `memory` and export
`skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32, out_cap: u32) -> i32`.
Program 0 must export funcref table `__indirect_function_table` when it uses
`eval` callbacks.
`cb_table_idx` indexes that caller-exported table; the element must be a
function `(req_ptr: u32, req_len: u32, res_ptr: u32, res_cap: u32) -> i32`.
Null, wrong-type, wrong-signature, or out-of-range elements fail `eval` with
`ERR_INVALID` before child execution. If `cb_table_idx < 0`, the child is
invoked without a parent callback and any child `invoke` call returns
`ERR_PERM`.

`eval` validates and instantiates `code_ptr..+code_len` as a child module.
The child may import only:
```text
module "skrynia", name "invoke"
  (req_ptr: u32, req_len: u32, res_ptr: u32, res_cap: u32) -> i32
```
Any other import (including memory/table/global/function/WASI) is rejected with
`ERR_INVALID` before child instantiation/side effects.

`eval` copies `input_ptr..+input_len` into child private memory and calls
child export `skrynia_main(in_ptr: u32, in_len: u32, out_ptr: u32,
out_cap: u32) -> i32` in a fresh instance with its own linear memory/globals.
The kernel selects child-private `in_ptr`, `out_ptr`, and `out_cap` regions
within child memory bounds; request bytes are written to `[in_ptr,in_ptr+in_len)`.
On non-negative return `n`, child output is read from `[out_ptr,out_ptr+n)`
in child memory and delivered per `eval` overflow contract. Memories are never aliased.

When the child calls imported `invoke`, the kernel pauses the child, copies
`req_len` bytes from child memory at `req_ptr` into the *parent* callback's
request region (kernel-staged, then exposed at the parent callback's
`req_ptr` in parent private memory for that callback turn), and invokes the
parent callback selected by `cb_table_idx`. The callback writes response bytes
into `res_ptr..+res_cap` in its own private memory and returns produced byte
count or negative error. The kernel copies at most that count into the child's
`res_ptr..+res_cap`. No private memory, table, globals, descriptors, or root
imports are shared between parent and child; only copied byte payloads cross.

Nested `eval` is allowed: a callback may call `eval` itself, creating another
child context. Each active frame has distinct private memory and its own
invoke binding to the direct caller's callback. Maximum host-call nesting
depth is enforced (`SKRYNIA_MAX_EVAL_DEPTH`, default 32); exceeding it yields
`ERR_LIMIT`.

`eval` return and output-buffer contract (no repeated side effects):
- On child success with `n <= out_cap`, `eval` copies exactly `n` bytes from
  child output memory into caller `[out_ptr,out_ptr+n)` and returns `n`.
- If child returns `n > out_cap`, `eval` returns `ERR_OVERFLOW`, copies no
  output bytes, and MUST NOT execute the child again. The kernel records `n`
  in a per-caller-instance slot readable via `eval_output_len() -> i32`
  (last overflow-required length, else `-1`). Committed child side effects remain.
- If child traps or returns negative, `eval` returns a mapped negative error
  (`ERR_FAULT` for trap/unexpected boundary failure unless a specific `ERR_*`
  is propagated) and copies no output bytes.
- Any `invoke` callback returning negative is delivered to the child as that
  negative invoke result; callback traps fail the active `eval` with `ERR_FAULT`.

This one-shot/length-query design prevents hidden re-execution of
effectful children while allowing callers to size buffers deterministically.

## Disk, paths, and descriptors

Descriptors are opaque kernel-issued `i32` handles scoped to one Program 0
request invocation unless explicitly specified otherwise. Closed or stale
handles return `ERR_BADFD`; handle IDs may be recycled only after close and
must not alias a live resource. Each descriptor has an independent seek cursor.

`open_disk` modes: `READ=1`, `WRITE=2`, `CREATE=4`, `EXCL=8`, `TRUNC=16`.
`READ|WRITE` requires existing file unless `CREATE`. `CREATE|EXCL` fails with
`ERR_EXISTS` if present. `TRUNC` requires write capability and truncates to 0
on open. Returned handle supports `read` iff `READ`, `write/truncate/sync`
iff `WRITE`.

Path rules: UTF-8 bytes, no NUL, no absolute path, no empty segment, no `..`
segment after normalization, total path length <= 4096, confined below
`SKRYNIA_DATA_DIR`. Kernel resolves paths using race-resistant confined opens
(e.g. openat-style traversal from the data-dir root, rejecting symlink escape);
lexical checks are necessary but not sufficient.

`mkdir(path,mode)`: `CREATE` creates leaf directory; `EXCL` fails if exists.
Intermediate directories are not implicitly created. `exists` returns 1/0/-err.
`is_dir` returns 1/0/-err. `delete` removes a file or empty directory;
absent target returns success (idempotent absent). Deleting a non-empty
directory returns `ERR_PERM`/`ERR_IO` per filesystem outcome (not success).

`rename(old,new)` atomically replaces the destination path name on the same
filesystem when supported: after success, `new` refers to `old`'s inode/content
and `old` is absent; on failure, state is either old or new state with no
partial name. Destination-existing replacement is allowed for files and for
directories only where platform rename semantics are atomic and supported;
otherwise `ERR_UNSUPPORTED`. Cross-device rename returns `ERR_UNSUPPORTED`.
Rename does not imply `fsync`; durability requires `sync` of affected file and
parent directory descriptors as applicable.

`read(fd,dst,len)` copies up to `len` bytes from cursor into private memory,
advances cursor by bytes read, returns bytes read (`0` at EOF). `write`
copies from private memory, advances cursor, returns bytes written; short
writes are permitted and must be surfaced (`0 < n < len`) rather than hidden.
Neither disk read nor write is atomic with concurrent access or crash.
`truncate(fd,size)` sets logical size; extension bytes read as zero; may fail
with `ERR_NOSPACE`/`ERR_IO`/`ERR_UNSUPPORTED`; not atomic/durable by itself.

`open_directory` returns a directory descriptor. `next_entry` writes one entry
name (not `.`/`..`) into `dst` and returns its byte length. End of iteration
returns `0`. If buffer is too small, returns `ERR_OVERFLOW` and does not
advance the iteration cursor (name can be retried). Concurrent modifications
may be observed or missed; no snapshot is implied. `sync(dir_fd)` requests
directory-entry durability; `sync(file_fd)` requests file content/metadata
durability as provided by the host filesystem. Crash may expose any prefix
consistent with issued sync operations; unsynced operations may be lost or
reordered by storage.

## Global RAM semantics

`open_ram()` returns a descriptor for the single process-global RAM object.
It never allocates a new object. Multiple descriptors and invocations observe
the same bytes. `read/write/seek/truncate/close` on RAM descriptors use the
same ABI as disk where meaningful: transfers copy between private WASM memory
and RAM bytes only. Each RAM descriptor has a private seek cursor; contents
are shared.

`truncate(ram_fd,size)` resizes shared RAM (max `SKRYNIA_MAX_RAM_BYTES`,
default 64 MiB). Growth zero-fills newly addressable bytes. Shrink discards
bytes above `size`. Resize is serialized with respect to other RAM operations
on the resized region boundary: operations fully below new size see old/new
state consistently; operations touching removed bytes fail with `ERR_RANGE`.
Resize itself is not atomic with concurrent readers/writers except through
this range/serialization rule.

`cas_ram_u64(ram_fd, offset, expected, desired) -> i64` requires `offset`
8-byte aligned and `offset+8 <= ram_size`. It atomically loads a little-endian
`u64`, compares to `expected`, stores `desired` on equality, and returns the
observed old value as non-negative `i64` bit pattern reinterpretation is not
used: values are returned in `[0, 2^63-1]` when representable; for the full
`u64` domain the ABI returns low 63 bits in value and encodes bit 63 via
`cas_ram_u64_hi` companion result is avoided by defining return as two-value
struct in future; **phase-1 decision:** return the observed old `u64` as
`i64` via unsigned-to-signed bit cast in wasm (`i64` is a bit container),
and callers interpret bits as unsigned. Misalignment/OOB returns `-ERR_RANGE`
in a distinguished error path (`ERR_RANGE`); on API error no memory changes.
Successful CAS returns old word regardless of match.

CAS operations are linearizable and sequentially consistent with each other.
Ordinary byte `read/write` operations on RAM are data-race-free at host level
(no Rust undefined behavior) but are not atomic with respect to overlapping
CAS or overlapping ordinary accesses except for kernel memory-safety. Programs
must use CAS/explicit protocols for synchronization; overlapping ordinary and
atomic access has program-defined value semantics but must not corrupt kernel
state or other memory domains.

`sync(ram_fd)` returns success and performs no durability action. Process crash/restart discards all RAM contents and all descriptors; RAM is rebuildable cache/state only.

## Safety, limits, and determinism boundaries

Kernel enforces physical ceilings for all instances including Program 0:
fuel/instruction budget, wall-clock deadline, linear memory pages, table
elements, stack/call depth, open descriptors, eval nesting depth, request/body
sizes, RAM size, path lengths, directory entry name lengths, and host I/O
quota. Exceeding a ceiling fails with `ERR_LIMIT` or VM trap mapped to `500`.

`now()` returns Unix epoch milliseconds (`i64`) from wall clock; may jump.
`secure_random_bits(dst,bits)` fills `ceil(bits/8)` bytes in private memory
from OS CSPRNG and zeroes unused high bits in the final byte; fails only on
OS RNG failure or invalid range.

Guest traps, invalid memories, out-of-bounds guest accesses, stack overflow,
and exhausted fuel terminate the active instance/failure path without crashing
the host process or leaking private memory/RAM contents across requests.

## Conformance artifacts (phase 1)

This repository carries executable phase-1 contract artifacts:
- `docs/v3/abi.md` — resolved ABI decisions and rationale.
- `docs/v3/fixtures/*.wat` — minimal canonical modules for import/ABI cases.
- `rust-server/tests/v3_*.rs` and helpers — runnable prototype checks over the
  documented ABI/status/buffer/invoke semantics, executed by `make test`.

These artifacts are contract/prototype evidence, not a claim that the full v3
kernel is implemented in the current v2 server.
