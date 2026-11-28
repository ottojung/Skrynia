# Skrynia v3 kernel contract

The kernel executes independently supplied WASM on a single node and provides
generic storage and machine operations. The [machine ABI](v3/target-abi.md)
defines the fixed signatures and scalar conventions; [typed FFI](v3/typed-ffi.md)
defines optional named imports. The [intent records](intent-records) are the
source of architectural requirements. Derivative implementation choices are
not additional owner requirements.

## Root and transport

At startup the kernel loads, validates and compiles the operator-supplied
`SKRYNIA_ROOT_WASM`. No root program is built in. The required entrypoint is
`skrynia_main(input_fd: i32, output_fd: i32) -> i32`; there is no mandatory
linear memory export. Only exact fixed kernel imports in namespace `skrynia_v3_1`
and configured typed named functions in namespace `syscall` are resolved.
Unknown namespaces, unresolved imports and wrong types reject startup.

The only execution endpoint is `POST /` with an opaque binary body.
`POST /version` returns build version, commit, ABI identifier and discovered typed syscall
metadata without invoking the root. Other methods/paths do not dispatch
application operations.

Each request creates a fresh root instance and an independent descriptor
table. The kernel supplies a read-only input stream and write-only output
stream. Input is pulled incrementally through bounded transport buffers; it
is not copied into global RAM or WASM memory. For `Transfer-Encoding: chunked`
requests the transport completes a bounded preflight of the entire request
body (size, per-chunk inactivity and total deadline) before starting the root,
so an oversized or broken chunked stream cannot be masked by early root output;
this is the documented exception to incremental input delivery. Known-length
bodies remain duplex, with output available before request EOF. Output bytes are streamed
incrementally, with bounded backpressure. The root may explicitly close
either descriptor; all request-local descriptors are released when the
invocation finishes.

A root return of zero means success; negative values are errors and a positive
value is invalid. Successful completion returns HTTP 200 with the exact
uninterpreted output bytes. A VM failure before response commitment produces
an error status; a failure after commitment terminates the body stream with
an error, not fabricated EOF. Neither an uncertain effect nor partial output
is retried by the kernel. Application-level failures are userland response
bytes, not kernel routing decisions.

The reference transport admits 16 concurrent execution requests and rejects
new requests with 503 while no permit is available. It bounds request bytes
(`SKRYNIA_MAX_REQUEST_BYTES`, default 16 MiB) and transport inactivity/total
times (`SKRYNIA_INPUT_INACTIVITY_SECS`, `SKRYNIA_REQUEST_TOTAL_SECS`,
`SKRYNIA_RESPONSE_TOTAL_SECS`). `SKRYNIA_REQUEST_TOTAL_SECS` is the absolute
deadline for request-body intake, including the chunked preflight;
`SKRYNIA_RESPONSE_TOTAL_SECS` is the absolute deadline for output, measured
from request arrival. These are transport protections, not
instruction metering or a guarantee that unrestricted native WASM terminates.
An oversized request is rejected with 413; a transport error is not EOF.

## Memory and execution boundary

There are three distinct memory domains:

- Private WASM linear memory, if a module has any, is for that module's
  computations and explicit fixed-ABI buffers. Calls taking a pointer require
  an ordinary exported memory named `memory`.
- Kernel-owned memory holds implementation metadata, compiled modules,
  descriptor tables and transient I/O buffers. Its pointers are not exposed.
- Shared RAM is one process-global volatile byte-addressable object, initially
  empty. Access is explicit via descriptors, including bulk and byte I/O.
  Descriptor offsets remain 64-bit. RAM is not implicitly populated with HTTP bodies
  or module private memory. Its allocation and protection are userland
  protocols, not per-program kernel partitions.

An evaluated module receives at most one explicitly supplied function; it
has no implicit descriptors, kernel functions, filesystem scope, memories
or tables. The function executes in its originating context. A child instance
is destroyed after evaluation. No arbitrary userland VM is guaranteed to
terminate or consume a bounded amount of resources: metering and policy
belong in separately supplied userland VMs.

## Filesystem and durability

The reference implementation confines filesystem operations to existing
disjoint private roots configured by the operator. Filesystem paths are
qualified UTF-8 paths of the form `user/...` or `operator/...`, with no NUL,
absolute path, parent traversal, empty segments or trailing slash, and at most
4096 bytes. `user/.` and `operator/.` name the two root directories. Confinement
uses descriptor-relative traversal and rejects symlink traversal; lexical
validation alone is insufficient.

Descriptors are opaque nonnegative i32 handles; stale or closed handles
must not alias a live resource. Each descriptor has its own cursor. Bulk read/write use private linear buffers;
read1/write1 require no buffer. The machine ABI defines relative
seek, supported operations, EOF, shared-RAM resize and CAS. File/directory
mutation and ordinary disk writes are not automatically durable. Rename
uses the host's atomic same-filesystem operation where supported. For
durability, sync the affected file and parent directories, including the
storage root for top-level mutations. Directory iteration does not promise
a snapshot under concurrent mutations.

Root selection must be explicit and consistent across reads, writes,
existence checks, directory iteration, rename and sync. Qualification must
include actual persisted userland writes followed by reads and restart,
not only host-preseeded read fixtures. Selection uses the explicit namespace
for every path operation; read/write mode does not select a different root.
Rename must remain within one root. Genuine absence is distinct from a bad
descriptor or an I/O failure (see the machine ABI).

The kernel does not implement application authentication, grants, storage
schemas, quotas, accounting, program IDs, subscriptions or Web Push. Such
policies and any HTTP application method handling belong to userland.
