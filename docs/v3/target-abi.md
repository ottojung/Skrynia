# Skrynia target ABI — normative integration specification

> **Status:** target specification for the K0/K1/K2/K3/T1 migration. It is **not**
> a description of the kernel currently built at `main`. The shipped ABI at
> `main 0ccc90f` is an implementation snapshot documented in `v3/abi.md`; where
> that snapshot conflicts with this file or the intent records, this file and
> the intent records govern the target, and the old snapshot is historical only.
>
> **Authority:** `docs/intent-records/*.md` at owner head
> `0ccc90ff7cfa8ee1a62b4a1e90b6b08677976323` (2026-11-26). Nothing here changes
> an owner requirement. Items marked **[K0-CHOICE]** are explicit K0 integration
> decisions needed to make the target executable; they are not owner decisions
> and are open to owner override. Items marked **[UNRESOLVED]** must not be
> guessed or encoded as owner intent.

## 1. Fixed function inventory (19 functions)

All shared-RAM pointers and buffer lengths are `u64`, independent of WASM
linear-memory width. Scalar descriptors, table indices and byte values are
ordinary integer values, never RAM addresses.

    i32 open_ram();
    i32 open_disk(u64 path, u64 len, u32 mode);
    i32 close(i32 fd);
    i32 read1(i32 fd);
    i32 write1(i32 fd, u32 byte);
    i64 seek(i32 fd, i64 shift);
    i32 truncate(i32 fd, u64 size);
    i32 sync(i32 fd);
    i32 mkdir(u64 path, u64 len, u32 mode);
    i32 exists(u64 path, u64 len);
    i32 is_dir(u64 path, u64 len);
    i32 delete(u64 path, u64 len);
    i32 rename(u64 old, u64 old_len, u64 new, u64 new_len);
    i32 open_directory(u64 path, u64 len);
    i32 next_entry(i32 fd, u64 dst, u64 capacity);
    i32 cas1(i32 fd, u64 offset, u32 expected, u32 desired);
    i64 now();
    i32 secure_random_bits(u64 dst, u64 bits);
    i64 eval(u64 code, u64 code_size, i32 function, i32 arg0, i32 arg1);

Operator-defined named FFI imports in namespace `syscall` are additional
functions with provider-defined integer signatures (section 6).

## 2. Descriptor bytes, errors, EOF and relative seek

`read1` and `write1` are the only fundamental descriptor data-transfer
operations; each transfers exactly one byte. There are no fixed kernel bulk
read/write imports and no RAM buffer requirement for byte I/O. Bytes are
`0..255`; a byte operand outside that range is invalid.

**[K0-CHOICE] Scalar encoding.** Negative results are errors, nonnegative
results are values or success status. Codes: `-1` EOF, `-2` EBADF, `-3` EINVAL
(including byte/operand out of range), `-4` ENOTSUP, `-5` EIO, `-6` ERANGE
(overflow/underflow), `-7` bounds violation. Status-only operations return `0`.
`read1` returns the byte `0..255` on success and `-1` at EOF. `write1` returns
`0` on success. Descriptors are linear byte streams with a per-descriptor
cursor; a successful byte transfer advances it by one; an error leaves it
unchanged (except the documented skip clamp below).

**[K0-CHOICE] Relative seek.** `seek(fd, shift)` returns the new absolute
cursor position (`>= 0`) on success or a negative error, with no cursor
movement on error. `shift = 0` is a no-op that still returns the position.
Regular file and RAM descriptors support signed seeking inside `[0, len]`;
`len + shift` outside that interval is `-6`. Forward-only input streams accept
`shift >= 0` only: a positive shift discards that many input bytes, a short
skip clamps the cursor to end-of-stream (subsequent `read1` is `-1`), and a
negative shift is `-4`. Output-only streams accept only `shift = 0`; a nonzero
shift is `-4` (no implicit zero-fill or partial side effects).

## 3. Shared RAM and single-byte CAS

There is exactly one unpartitioned, dynamically resizable shared RAM object.
`truncate` grows with zero-initialized bytes and shrinks by discarding the
tail. Ordinary reads/writes guarantee individual byte atomicity and disjoint
writes do not erase one another; no global RAM mutex is required. Allocation,
protection and synchronization protocols belong to software.

**[K0-CHOICE] CAS.** `cas1(fd, offset, expected, desired)` returns the observed
byte `0..255`, including when the comparison fails; it does not return a
separate status. `expected` and `desired` must be `0..255`, otherwise `-3`.
The operation is sequentially consistent and acts on exactly one byte at any
valid offset without alignment requirements: success is a release operation
that publishes prior ordinary RAM writes to a later acquiring observation;
failed observations also acquire. No CAS buffer and no multi-byte CAS.

`open_directory(".")` opens the authorized storage root itself and `sync` on
that descriptor requests directory durability for top-level creation,
deletion and rename, with no relaxation of path confinement or parent
traversal.

## 4. Generic single-function eval

`eval` is the only kernel execution primitive. Code bytes are read from shared
RAM at `(code, code_size)` and snapshotted before validation and execution.
The module imports zero or one function, selected by the evaluator; the kernel
validates its exact ordered `i32`/`i64` parameter and result types against the
child import. There is no binding array, no implicit import injection, no
non-function import, no prescribed import name, no mandatory linear memory and
no implicit linear-memory copying or growth.

**[K0-CHOICE] Eval conventions.** `eval` returns `0` on success and a negative
code from section 2 on failure (trap, type mismatch, bounds, invalid code).
`function = -1` means no supplied function and requires `arg0 = arg1 = 0`;
`function >= 0` is an index into the evaluator-supplied function table and
binds the child's single function import to that originating-context function
after exact type validation. The child entrypoint is an exported `_start`
with signature `() -> ()`, otherwise the module start section, otherwise
`-3`. The child's call arguments are forwarded to the selected function in its
originating context; `arg0`/`arg1` are opaque `i32` context handles for that
selection. Results reach callers through shared RAM or the supplied function,
not through the `eval` result. Each evaluation uses a fresh child store
borrowing the suspended parent; sequential evaluations do not accumulate
instances in a long-lived store.

**Integration assumption recorded by K0 (from K2, 2026-11-26):** Wasmtime host
functions are store-bound, so the implementation may invoke the supplied
function through a generic typed trampoline in the same store while still
instantiating a fresh child instance for each evaluation and retaining no
instance in a long-lived store. This is an engine implementation concession,
not owner ABI. A separate child store borrowing the suspended parent remains
the experimentally validated candidate and may replace it without an ABI
change. Legacy bulk I/O shims temporarily retained while K1/K2 port tests are
transitional and must be removed before target acceptance.

Wasmtime fuel, epoch deadlines, store resource limits and aggregate fuel
transfer are not part of this ABI. Measurement, metering, budgets and quotas
are implemented by custom userland VMs. Kernel bounds checks, memory safety,
host allocation failures and error handling remain mandatory and are distinct
from application metering.

## 5. Removed inventory

Do not retain `read`, `write`, `invoke`, `cas_ram_u64`, `eval_delegate`,
`eval_buffered`, `fs_delegate_scope`, `try_mutation_lock` or
`release_mutation_lock` as fixed native ABI functions or implicit imports.
Native registration, program-ID management and application protocols do not
belong in the kernel.

## 6. Named typed FFI and deterministic wire encoding

Imports in namespace `syscall` may have any number of `i32`/`i64` parameters,
including zero and mixed widths, and function-specific integer results
including none or several. Integer bit patterns are opaque to the kernel.
Discovery advertises every function name with its ordered parameter and result
types, including empty lists; imports are resolved during initialization and
mismatched or unresolved imports are rejected before execution. No automatic
replay of a failed call.

**[K0-CHOICE] Discovery shape.** `GET /syscalls_list` returns
`{"functions":[{"name":"...","params":["i32","i64"],"results":["i32"]}]}`.
`POST /version` returns build version and commit plus the same descriptors
under `"syscalls"`.

**[K0-CHOICE] Binary wire format.** `POST /<name>` and its response use a
versioned tagged little-endian layout: byte `0 = 0x01`, byte `1 = N` (argument
or result count), followed by `N` records; tag `0x32` is a 4-byte `i32`, tag
`0x40` is an 8-byte `i64`, with no padding. HTTP status `400` is a malformed
body or signature mismatch rejected before execution, `404` an unknown
function and `500` a provider failure. Native Rust implementations receive
the typed integers directly; remote HTTP providers need no caller RAM or
descriptor access. General remote machine-context access is deferred.

## 7. Derived copy — unresolved by owner

`copy` is the standard derived FFI function with exactly four explicit integer
arguments, not a RAM request-structure pointer. Its meanings, order, widths,
return signature, error codes, partial-progress/EOF behavior, overlapping and
same-descriptor behavior, and concurrency semantics are **[UNRESOLVED]** by
the owner. Software providing copy may use a portable `read1`/`write1`
fallback or a native accelerator, selected before execution; an HTTP provider
may omit copy. No kernel fallback is required. K0 will not encode a guessed
copy signature into this ABI; T1 and K3 must coordinate the unresolved parts
without presenting them as owner intent.

## 8. Integration and qualification boundary

Lane integration order is K1 (descriptor/RAM/CAS/fs) -> K2 (eval/engine) ->
K3 (typed FFI/discovery) -> T1 (tools/SDK). A1 prepares app compatibility
tests independently against this specification. After each exact head is
pushed, K0 reviews the diff, runs structural lint, `make test` and CI checks,
and records the exact commit, tree and test evidence before proceeding.
Existing old-ABI qualification artifacts are historical and are **not**
acceptance evidence for this target. No production cutover.
