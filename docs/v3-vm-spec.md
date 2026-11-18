# Skrynia v3 virtual machine — design specification

Status: **draft for implementation and executable conformance testing**.
Milestone: Antonina `dgKBALkpO7M7oKBHrZUzCT-shortly`.
This document describes the *v3 replacement*, not the deployed v2 protocol.
Statements marked **OPEN** must be resolved before declaring the VM stable.

## Purpose and trust boundary

Skrynia is a single-node WebAssembly computer with explicit persistent disk
access and a single shared volatile RAM object. Its Rust kernel implements only
HTTP transport, WASM evaluation and sandboxing, filesystem operations, shared
RAM primitives, time, randomness, and physical execution safety limits.

The kernel MUST NOT implement grants, quota/accounting policies, user
authentication, registration, lookup, mutable `set`, application routing,
program IDs, or the v2 object-store API. Those are application protocols.

Only Program 0 runs with direct privileged imports. All other programs are
evaluated with exactly one imported external function, `skrynia.invoke`,
whose meaning is supplied by the evaluating program. No implicit WASI,
filesystem access, clock, randomness, host memory, or transitive capabilities.

## Initialization and HTTP

- `SKRYNIA_ROOT_WASM` identifies the Program 0 WASM file. It is loaded,
  validated and compiled at *every* process initialization/restart, without
  inferring program identity from any application-maintained registry.
- `SKRYNIA_DATA_DIR` identifies the filesystem subtree made available to
  Program 0. Initialization fails closed on invalid code/configuration.
- The ONLY application endpoint is `POST /`, taking arbitrary binary bytes
  with `Content-Type: application/octet-stream`. No program selector header,
  path parameter, URI routing, or kernel-side body interpretation.
- The request body is passed to a fresh logical invocation of Program 0.
  Output bytes are passed back as the response. On successful VM completion,
  HTTP status is 200 and the content type is application/octet-stream.
  VM/transport failures are distinct from application-level failures.
- Any legacy HTTP endpoints, Web Push dispatch, static GET reads, legacy
  response statuses and authorization headers must be supplied by a separate
  `skrynia-tools` compatibility facade, not by this kernel.
- The kernel may cache compiled Program 0 code across requests, but no request
  shares implicit private WASM globals, stack, or linear memory with another.
- **OPEN:** precise HTTP transport error statuses and request/body size caps.

## Storage and memory domains

Three distinct memory domains MUST remain separate:

1. **Private WASM linear memory**, one isolated execution context per invocation.
   All caller-visible pointer/length pairs in the privileged kernel ABI refer
   to this memory. A child has its *own*, separate memory.
2. **Kernel-owned memory**, used for HTTP bodies, results, compilation/cache,
   call frames, descriptor metadata, and transient transport copies.
3. **Global RAM**, exactly one contiguous volatile byte-addressable storage
   object. It persists across invocations but is empty after a process restart.

**Absolute invariant:** No runtime component may implicitly read, write,
allocate, reserve, or store any kernel metadata, request/response body, WASM
linear memory, compiled code, or eval arguments in global RAM. Global RAM is
accessed ONLY as an explicit target of `open_ram()`, `read`, `write`,
`truncate`, `seek`, or `cas_ram_u64` requested by authorized code.

No private WASM memory is aliased to global RAM. An integer RAM offset is not a
capability; possession of the privileged storage interface is the authority.

## Root privileged operations

The following signatures are schematic. The ABI must define integer sizes,
value encoding, error/status, and ownership without implicit dynamic arrays.

```text
disk = open_disk(path_ptr, path_len, mode)
ok = truncate(fd, new_size)
ok = seek(fd, absolute_offset)
n = read(fd, dst_private_ptr, len)
n = write(fd, src_private_ptr, len)
close(fd)

ok = mkdir(path_ptr, path_len, mode)
present = exists(path_ptr, path_len)
ok = delete(path_ptr, path_len)                 # absent is success
ok = rename(old_ptr, old_len, new_ptr, new_len) # atomic namespace rename
dir = open_directory(path_ptr, path_len)
entry_len = next_entry(dir, dst_ptr, dst_capacity)
directory = is_dir(path_ptr, path_len)

ram = open_ram()                                # same underlying object always
old = cas_ram_u64(ram_offset, expected, desired)
sync(fd)                                       # files and directories
now() -> unix_millis
secure_random_bits(dst_private_ptr, bit_count)
eval(wasm_private_ptr, wasm_len, input_private_ptr, input_len,
     invoke_handler, output_private_ptr, output_capacity)
```

- `open_disk` and `mkdir` modes control create-if-absent,
  open-if-present, and fail-if-condition-not-met behavior. Whether a newly
  opened disk descriptor permits read, write, or both is part of the mode.
- `read` returns number of bytes actually transferred; zero at EOF.
  `write` returns number of bytes actually transferred; short writes must
  be handled by application code. Errors are not disguised as EOF.
- Neither disk `read`, `write` nor `truncate` promises atomicity,
  especially with concurrent access or process/storage failures.
- `truncate` may fail when allocation or resource limits cannot be met.
  **OPEN:** sparse files vs physical preallocation contract.
- `rename` is a single atomic namespace replacement where supported by the
  confined filesystem. No implication of multi-file transactions or automatic
  crash durability. **OPEN:** destination-exists behavior, directory renames,
  and filesystem portability constraints.
- `sync(file)` commits file contents/metadata as provided by underlying
  storage; `sync(directory)` is available to durably publish directory-entry
  operations. **OPEN:** precise crash guarantees by supported filesystem.
- `open_directory` iterates entries; modifications during iteration need
  not yield a snapshot. End-of-iteration differs from failure or undersized
  output buffer. `close` applies to directory descriptors too.
- Paths are confined under `SKRYNIA_DATA_DIR`: reject absolute paths, `..`,
  null bytes, and escapes via symlinks or race-prone path resolution.
  Do not rely on lexical validation alone for filesystem confinement.
- Descriptors are opaque process-local handles, not OS numeric FDs. Handle
  recycling must not let a stale token refer to a different resource.
  **OPEN:** descriptor persistence between invocations vs invocation-scoped
  handles; cursors must not race unintentionally across different callers.

## Global RAM and synchronization

- `open_ram()` always refers to the same shared storage, not an allocation
  or a new RAM object. Its descriptor may have a per-invocation seek cursor,
  while its contents are globally visible.
- Explicit reads and writes transfer bytes between caller-private memory and
  the global RAM object. No pointer passed to `read` or `write` ever denotes
  global RAM directly.
- `truncate(ram_fd,size)` grows/shrinks the shared address space subject to
  resource limits; newly exposed bytes read as zero. Global offsets are stable.
- `cas_ram_u64(offset,expected,desired)` atomically compares and conditionally
  replaces a little-endian 64-bit word, returning the observed old word.
  Require 8-byte alignment and in-bounds offset. CAS is linearizable, with
  sequentially consistent ordering relative to other atomic operations.
- Ordinary RAM access overlapping CAS must be data-race-safe at kernel level;
  do not expose unsynchronized native memory that could cause Rust UB.
  Programs remain responsible for higher-level synchronization protocols.
- `sync(ram_fd)` is a successful no-op: RAM is volatile and coherently shared,
  not durable. It is not a memory lock or conditional transaction.
- Crash/restart clears all global RAM. Programs must rebuild caches/registries
  from durable disk state; RAM alone is never authoritative for persistence.
- **OPEN:** resource limit, growth granularity, concurrency of resize with I/O.

## Evaluated WebAssembly and callbacks

- The kernel validates the module before execution. Child modules importing
  anything except the exactly defined `skrynia.invoke` signature are rejected,
  including imported memory/tables, functions other than `invoke`, and WASI.
- The caller defines the invoked function's behavior. The child cannot obtain
  its parent's private memory, root imports, registry secrets or kernel API by
  simply being invoked. Returning secret IDs is an explicit application choice.
- Each `eval` uses a distinct guest memory/instance context. Nested `eval`
  and callbacks may recurse subject to explicit physical depth limits.
- The kernel may cache *compiled code* by code hash and runtime version, and
  reuse initialized templates/pools with isolation. It MUST NOT couple caching
  to a mutable, application-defined program ID or global RAM.
- Child module code and input supplied to `eval` must be snapshotted or
  protected from mutation during compilation and execution; a concurrent
  writer must not change which bytes were validated vs executed.
- Host work (file I/O, compilation, allocation, host callbacks) is separately
  bounded; raw guest instruction counts alone cannot protect these surfaces.
- **OPEN:** exact binary `eval`/`invoke` ABI, parent-export callback
  registration, result continuation/short-output semantics, guest allocator,
  trap propagation and reentrancy. In particular, output-buffer-too-small
  MUST NOT silently re-run a call with side effects.

## Safety and accounting

- The kernel enforces emergency fuel/time, memory, stack/call-depth,
  compilation and host-I/O ceilings for all native WASM, including Program 0.
- Application-defined gas budgets (with refill over time or no refill) are
  policy code, not a kernel grant system. Untrusted metered execution can use
  a WASM interpreter inside WASM, or a demonstrably equivalent instrumenter.
- `now` returns Unix wall time in milliseconds; callers must tolerate clock
  changes. Cryptographic random bits come from OS-secure randomness.
- The kernel must never disclose authorization-bearing IDs, root code or
  private paths in error messages or access logs.

## Minimum conformance suite

1. POST arbitrary bytes to Program 0; get exactly its output, no VM access
   to global RAM unless Program 0 explicitly calls RAM operations.
2. Restart loads the exact configured Program 0 code again; errors fail closed.
3. Guest with only `skrynia.invoke` succeeds; forbidden import/WASI fails.
4. Nested guest callback cannot get parent/root memory or capabilities.
5. File I/O with private pointers, short reads/writes, errors and confinement.
6. Directory iteration, atomic same-filesystem rename, file+directory sync.
7. Global RAM is shared across invocations and cleared after restart, with
   no implicit HTTP, compilation or kernel bookkeeping allocations.
8. Aligned CAS has one winner under contention, correct old-value semantics;
   unaligned/OOB access fails without memory corruption.
9. Repeated application-level `set` changes behavior without creating a new
   native ID; this must be tested in the root-program project, not Rust kernel.
10. Guest traps, infinite loops, resource limits and malformed modules do not
    crash the host or leak cross-invocation memory.
11. Warm-call/no-op/write latency and throughput measured separately from
    disk durability and network RTT.
12. v2 HTTP parity test suite must pass through skrynia-tools compatibility
    facade before deployment can replace existing v2 installations.
