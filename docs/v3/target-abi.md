# Skrynia target ABI — normative integration specification

> **Status:** normative ABI for the target kernel implementation.
> Whole-stack acceptance is governed separately by the qualification plan.
>
> **Authority:** current `docs/intent-records/*.md`, revised by owner approval
> 2026-11-27 for private linear-memory buffers and bulk I/O. Items marked
> **[K0-CHOICE]** are explicit integration
> decisions needed to make the target executable; they are not owner decisions
> and are open to owner override. Items marked **[UNRESOLVED]** must not be
> guessed or encoded as owner intent.

## 1. Fixed function inventory (21 functions; ABI 3.1)

All fixed ABI buffer pointers and lengths are `u64`, independent of WASM
linear-memory width. A pointer addresses the calling instance's ordinary
(non-shared) exported linear memory named `memory`. A caller that uses only
scalar functions need not export or define memory. Scalar descriptors,
table indices and byte values are ordinary integers. Descriptor offsets
(including cas1) address the descriptor, not linear memory.

**[K0-CHOICE] ABI versioning.** Fixed imports use namespace `skrynia_v3_1`.
The old namespace is not linked; a module importing old fixed functions
must be rebuilt. POST /version includes `"abi":"skrynia_v3_1"`. The root
entrypoint, named syscall namespace, and child single-function import
conventions are unchanged. There is no old-pointer compatibility mode.

Buffer-based operations validate the complete requested range before any
descriptor/file/RNG effect: missing or nonordinary exported memory is -3;
u64 addition overflow is -6; a range outside the current memory is -7.
A zero-length buffer must still refer to a valid memory position
(including one-past-end). Buffer lengths must fit a nonnegative i64 count.
No borrowed view survives a host call or reentry into WASM. No implicit
allocation, copying between instances, or memory growth is performed.

    i32 open_ram();
    i32 open_disk(u64 path, u64 len, u32 mode);
    i32 close(i32 fd);
    i64 read(i32 fd, u64 ptr, u64 size);
    i64 write(i32 fd, u64 ptr, u64 size);
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

`read1` and `write1` transfer exactly one byte and need no linear memory.
Bytes are `0..255`; a byte operand outside that range is invalid.

`read` and `write` transfer between the descriptor and private linear
memory. A successful call returns the actual number of bytes transferred
(0..size); a partial transfer is valid, and the caller may continue at the
updated cursor. A positive-length read returns 0 at EOF, not -1. An error
before progress returns a negative code; if some progress was made, return
its count. The cursor advances by the returned count, and a read only
changes the corresponding destination prefix. A zero-length call validates
the descriptor's access mode and the buffer and makes no transfer.
Bulk RAM access is only byte-atomic; it is not a transaction or snapshot.
Writes to RAM do not implicitly resize it. At the end of the allocated RAM
a nonempty write with no progress reports a bounds error; a short write
that reaches the end returns its count. A read at or beyond RAM EOF (e.g.
after a concurrent shrink) returns 0. Transport-level errors may have
irreversibly consumed input outside the accepted stream limit; they are
not successful reads. Ordinary disk write durability still requires sync.

**[K0-CHOICE] Scalar encoding.** Negative results are errors, nonnegative
results are values or success status. Codes: `-1` EOF, `-2` EBADF, `-3` EINVAL
(including byte/operand out of range), `-4` ENOTSUP, `-5` EIO, `-6` ERANGE
(overflow/underflow), `-7` bounds violation, `-8` ENOENT (genuine absence of a
qualified path only; distinct from `-2` EBADF and `-5` EIO). Status-only operations return `0`.
`read1` returns the byte `0..255` on success and `-1` at EOF. `write1` returns
`0` on success. Descriptors are linear byte streams with a per-descriptor
cursor; a successful byte transfer advances it by one; an error leaves it
unchanged (except the documented skip clamp below).

**[K0-CHOICE] Relative seek.** `seek(fd, shift)` returns the new absolute
cursor position (`>= 0`) on success or a negative error. A failed seek on
forward-only input may already have consumed bytes; those cannot be rewound.
Other failed seeks do not move the cursor. `shift = 0` is a no-op that
returns the current position, including after a concurrent shrink.
Nonzero signed seeks on regular file and RAM descriptors stay inside `[0, len]`;
`position + shift` outside that interval is `-6`. Forward-only input streams accept
`shift >= 0` only: a positive shift discards that many input bytes, a short
skip clamps the cursor to end-of-stream (subsequent `read1` is `-1`), and a
negative shift is `-4`. Output-only streams accept only `shift = 0`; a nonzero
shift is `-4` (no implicit zero-fill or partial side effects).

## 3. Shared RAM and single-byte CAS

There is exactly one unpartitioned, dynamically resizable shared RAM object.
`truncate` grows with zero-initialized bytes and shrinks by discarding the
tail. The sparse reference implementation does not impose the former 64 MiB
logical limit: the largest supported size is i64::MAX bytes because seek
returns a nonnegative i64 position. Physical page allocation can fail.
Host-side path staging remains bounded independently of RAM size.
These are implementation choices, not application resource metering.
Ordinary reads/writes guarantee individual byte atomicity and disjoint
writes do not erase one another; no global RAM mutex is required. Allocation,
protection and synchronization protocols belong to software.

**[K0-CHOICE] CAS.** `cas1(fd, offset, expected, desired)` returns the observed
byte `0..255`, including when the comparison fails; it does not return a
separate status. `expected` and `desired` must be `0..255`, otherwise `-3`.
The operation is sequentially consistent and acts on exactly one byte at any
valid offset without alignment requirements: success is a release operation
that publishes prior ordinary RAM writes to a later acquiring observation;
failed observations also acquire. No CAS buffer and no multi-byte CAS.

**[K10-IMPLEMENTATION-CHOICE] Explicit confined roots.** Every path passed to
`open_disk`, `mkdir`, `exists`, `is_dir`, `delete`, `rename` and
`open_directory` must begin with exactly one namespace token, `user/` or
`operator/`. The token alone selects that root; the kernel strips it and
applies the unchanged relative-path grammar (no absolute paths, `..`, `//`,
trailing slash, NUL or relative suffixes over 4096 bytes). Unqualified paths,
token escapes and cross-token renames are rejected with `-3`; there is no
implicit default, overlay or fallback between the two physically distinct roots.
A native filesystem caller may supply a matching explicit root constraint;
it cannot redirect the path's namespace token. Never
treat `-8` (the only genuine ENOENT code), `-5` EIO or `-2` EBADF as absence of
a path. `open_directory("user/.")` or `open_directory("operator/.")` opens that
namespace root, and `sync` on that descriptor requests directory durability
for top-level creation, deletion and rename, with no relaxation of path
confinement or parent traversal. This is a recorded implementation choice,
not a separate owner approval.

## 4. Generic single-function eval

`eval` is the only kernel execution primitive. Code bytes are read from the evaluator's
exported linear memory at `(code, code_size)` and snapshotted before validation
and execution. No borrowed memory view survives instantiation or callback.
The module imports zero or one function, selected by the evaluator; the kernel
validates its exact ordered `i32`/`i64` parameter and result types against the
child import. There is no binding array, no implicit import injection, no
non-function import, no prescribed import name, no mandatory child linear memory and
no implicit linear-memory copying or growth.

**[K0-CHOICE] Eval conventions.** `eval` returns `0` on success and a negative
code from section 2 on failure (trap, type mismatch, bounds, invalid code).
`function = -1` means no supplied function and requires `arg0 = arg1 = 0`;
`function >= 0` is an index into the evaluator-supplied function table and
binds the child's single function import to that originating-context function
after exact type validation. The entrypoint is either an exported
`_start () -> ()` or the module start section, not both. Validate entrypoint
and binding before instantiation, so invalid entrypoints cannot execute
a start section. Trap/instantiation failure returns -5; invalid selectors,
import types and entrypoints return -3. `arg0`/`arg1` are reserved when a
function is supplied; they are not injected into its arguments. No owner
decision assigns them further semantics. Results reach callers through shared
RAM or the supplied function, not through the eval result.
Each child uses a fresh store borrowing the suspended parent via a typed
scalar trampoline. The originating function executes in its own context,
and sequential evaluations do not accumulate instances.

Wasmtime fuel, epoch deadlines, store resource limits and aggregate fuel
transfer are not part of this ABI. Measurement, metering, budgets and quotas
are implemented by custom userland VMs. Kernel bounds checks, memory safety,
host allocation failures and error handling remain mandatory and are distinct
from application metering.

## 5. Removed inventory

Do not retain `invoke`, `cas_ram_u64`, `eval_delegate`,
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
`POST /version` returns build version, commit, ABI identifier and the same descriptors
under `"syscalls"`.

**[K0-CHOICE] Binary wire format.** See [typed FFI](typed-ffi.md) for
the complete versioned tagged little-endian layout, including the extended
count field for 255 or more values. Native Rust implementations receive
typed integers directly; general remote machine-context access is deferred.

## 7. Derived copy — unresolved by owner

`copy` is the standard derived FFI function with exactly four explicit integer
arguments, not a RAM request-structure pointer. Its meanings, order, widths,
return signature, error codes, partial-progress/EOF behavior, overlapping and
same-descriptor behavior, and concurrency semantics are **[UNRESOLVED]** by
the owner. Copy may optimize transfers between RAM ranges or disk and RAM.
Software providing copy may use a portable buffered read/write fallback
(or byte I/O) or a native accelerator, selected before execution; an HTTP provider
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
