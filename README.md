# Skrynia v3

Skrynia is a small Rust WebAssembly kernel, not a built-in database or grant
server. It executes **operator-supplied Program 0 WASM** in response to
opaque binary HTTP requests. All application protocols, storage keys,
authorization, grants, quotas and application semantics belong in Program 0
or the programs it evaluates—not in the native kernel.

The native HTTP API has exactly one endpoint:

- `POST /` with arbitrary `application/octet-stream` bytes.
- Each request runs a fresh logical Program 0 instance, exporting
  `skrynia_main(input_fd: i32, output_fd: i32) -> i32`.
- The input descriptor can only be read and closed; the output descriptor
  can only be written and closed. Neither can seek or truncate. Short
  reads/writes are normal. Responses stream with bounded backpressure.
- A successful Program 0 return means HTTP 200 with uninterpreted response
  bytes. Other methods and paths are not application routing surfaces.

Program 0 has privileged imports for confined disk files/directories,
explicit shared volatile RAM with sequentially consistent 64-bit CAS,
clock, secure randomness, and nested WASM evaluation. Evaluated children
cannot access these privileges directly: their sole import is
`skrynia.invoke`, mediated by the caller. Each guest has private WASM
linear memory, distinct from kernel staging and explicitly opened global RAM.

## Development

Use Rust 1.93, OpenSSL development headers and pkg-config:

```sh
make test
make build
```

The kernel does not build, install, select or bundle a default Program 0.
Operators compile their own conforming WebAssembly module separately.
[skrynia-tools](https://github.com/ottojung/skrynia-tools) contains one
**experimental, optional** userland Program 0 and v2 compatibility work.
Its draft code has not passed the complete v2 release gates.

## Running an isolated v3 kernel

Supply an existing private storage directory and a separately built root WASM:

```sh
export SKRYNIA_ROOT_WASM=/absolute/path/to/root.wasm
export SKRYNIA_DATA_DIR=/absolute/path/to/private/data
export SKRYNIA_BIND=127.0.0.1:17380
target/release/skrynia-server
```

`SKRYNIA_ROOT_WASM` and `SKRYNIA_DATA_DIR` are required. The kernel
compiles the root and validates its imports/exports on **every startup**;
an invalid ABI fails startup rather than the first request. It never
interprets Program 0's request or response body.

A container image can be built with podman or docker (`make docker-build`).
It contains **only the kernel** and listens on port 17380. Mount the root
WASM read-only, set `SKRYNIA_ROOT_WASM` explicitly, and mount a private,
UID 1000-writable `SKRYNIA_DATA_DIR`. No default WASM, application grant,
admin CLI, credential, or native health endpoint is provided. Use an
orchestrator TCP liveness check; application health is a Program 0 protocol.

## Specifications and release boundaries

[The v3 VM specification](docs/v3-vm-spec.md) defines every host import,
memory domain, return code and sandboxing constraint.
[ABI decisions](docs/v3/abi.md) refine ownership and callback behavior.
[Integration notes](docs/v3-integration.md) list unfinished gates.

The older v2 HTTP/grants/Web Push documentation describes the compatibility
**target**, not functionality implemented by the native v3 kernel.
Full delegated grant/quota/revocation/subscription parity, differential
testing, migration, rollback and independent security reviews must pass
before any production replacement. Do not point an existing deployed v2
data directory at an unfinished v3 build.

Skrynia is Free Software under AGPL-3.0-or-later.
