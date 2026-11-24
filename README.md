# Skrynia v3

Skrynia is a small Rust WebAssembly kernel, not a built-in database or grant
server. It executes **operator-supplied Program 0 WASM** in response to
opaque binary HTTP requests. All application protocols, storage keys,
authentication, grants, quotas, subscriptions and application semantics belong
**only in independently supplied userland WASM**, not in the native kernel.
Program 0 is an operator-supplied opaque token-checking proxy; the example is
maintained in [skrynia-tools](https://github.com/ottojung/skrynia-tools).
The kernel neither builds nor selects a Program 0 and knows nothing about the
proxied application protocol.

The kernel transport has one execution endpoint and one metadata endpoint; HTTP verbs are not application operations:

- `POST /` with arbitrary `application/octet-stream` bytes.
- Each request runs a fresh logical Program 0 instance, exporting
  `skrynia_main(input_fd: i32, output_fd: i32) -> i32`.
- The input descriptor can only be read and closed; the output descriptor
  can only be written and closed. Neither can seek or truncate. Short
  reads/writes are normal. Responses stream with bounded backpressure.
- A successful Program 0 return means HTTP 200 with uninterpreted response
  bytes.
- `POST /version` returns JSON containing `version`, `commit`, and
  `syscalls` (sorted supported custom syscall names), without running WASM.
  Other methods and paths are not application routing surfaces.

Program 0 has privileged imports for confined disk files/directories,
explicit shared volatile RAM with sequentially consistent 64-bit CAS,
clock, secure randomness, and nested WASM evaluation. Evaluated children
cannot access these privileges directly: their sole import is
`skrynia.invoke`, mediated by the caller. Each guest has private WASM
linear memory, distinct from kernel staging and explicitly opened global RAM.

## Custom kernels

The workspace package `libskrynia` provides the same kernel as a Rust library.
Program 0 can import operator-configured functions from the `syscall` WASM
namespace, each with signature `(i32) -> i32`. Native implementations take
precedence over the optional HTTP fallback, with import targets resolved
once during startup. Child programs cannot import these functions directly.
See [custom syscalls](docs/v3/syscalls.md) for the library API, wire format,
configuration and memory contract.

## Development

Use Rust 1.93, OpenSSL development headers and pkg-config:

```sh
make test
make build
```

The kernel does not build, install, select or bundle a default Program 0.
All optional application servers, exact-key subscriptions, admin pages and
compatibility layers are **userland programs** maintained in separate
repositories. No application service or v2 object-store operation is present
in this repository.

## Running an isolated v3 kernel

Supply two existing private, disjoint storage directories and a separately built root WASM:

```sh
export SKRYNIA_ROOT_WASM=/absolute/path/to/root.wasm
export SKRYNIA_OPERATOR_DIR=/absolute/private/operator
export SKRYNIA_USER_DIR=/absolute/private/user
export SKRYNIA_BIND=127.0.0.1:17380
target/release/skrynia-server
```

`SKRYNIA_ROOT_WASM`, `SKRYNIA_OPERATOR_DIR` and `SKRYNIA_USER_DIR` are
required. Both directory roots must exist, be disjoint and private; the
kernel never creates or infers them. The kernel compiles the root and validates its imports/exports on **every startup**;
an invalid ABI fails startup rather than the first request. It never
interprets Program 0's request or response body.

A container image can be built with podman or docker (`make docker-build`).
It contains **only the kernel** and listens on port 17380. Mount the root
WASM read-only and set `SKRYNIA_ROOT_WASM` explicitly. Supply disjoint,
private UID 1000-owned, mode 0700 mounts at `SKRYNIA_OPERATOR_DIR` and
`SKRYNIA_USER_DIR`. No default WASM, application grant,
admin CLI, credential, or native health endpoint is provided. Use an
orchestrator TCP liveness check; application health is a Program 0 protocol.

## Specifications and verification

- [Kernel VM contract](docs/v3-vm-spec.md) defines the host imports,
  memory domains, return codes and sandbox boundary.
- [ABI details](docs/v3/abi.md) specify descriptor ownership, callback
  isolation and other VM boundary decisions.
- `make test` runs v3 kernel tests and structural/format/lint checks.
  CI additionally builds an isolated container and exercises a separately
  supplied disposable root WASM. These tests do not imply production release.

The root program, applications, data migrations, deployment origins, TLS,
application access control and Web Push belong outside this kernel.
Never point existing production data at an unqualified userland program.
The kernel does not ship a release image or deploy automatically; publishing
requires an explicit CI dispatch.

Skrynia is Free Software under AGPL-3.0-or-later.
