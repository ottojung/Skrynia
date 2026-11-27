# Skrynia v3

Skrynia is a minimal Rust WebAssembly compute and storage kernel. It executes
an independently supplied root program and has no application protocol, grant
system, program registry, subscription service, or built-in root program.
An optional token-checking Program 0 and higher-level programs are maintained
in [skrynia-tools](https://github.com/ottojung/skrynia-tools).

The kernel offers one opaque execution endpoint, `POST /`, and a metadata
endpoint, `POST /version`. Each request gives the root program independent
input and output descriptors. Transfers use buffered `read`/`write` or byte `read1`/`write1`;
the server never puts the HTTP body in shared RAM. The root entrypoint is
`skrynia_main(input_fd: i32, output_fd: i32) -> i32`.

The fixed machine ABI includes confined filesystem operations, dynamically
resizable shared RAM with single-byte CAS, time, randomness, and generic
evaluation of a module with zero or one explicitly supplied integer-signature
function. Fixed ABI 3.1 imports use `skrynia_v3_1`. All buffer addresses and lengths
refer to the caller's exported linear memory and use 64-bit integers.
Shared RAM is accessed explicitly through descriptors; scalar-only programs
need no memory. Allocation, application policy, metering and
execution budgets belong to software running on the kernel.

The root can also import named operator-provided functions in namespace
`syscall`, with ordered arbitrary integer signatures. The reference server
discovers typed function signatures at `GET <provider>/syscalls_list` and
invokes them using a versioned typed binary protocol. Custom executables can
use equivalent Rust implementations in-process through `libskrynia`.

## Development

Use Rust 1.93, OpenSSL development headers and pkg-config:

```sh
make test
make build
```

## Run an isolated kernel

Supply a separately built root WASM and two existing, private, disjoint
filesystem roots.

```sh
export SKRYNIA_ROOT_WASM=/absolute/path/to/root.wasm
export SKRYNIA_OPERATOR_DIR=/absolute/private/operator
export SKRYNIA_USER_DIR=/absolute/private/user
export SKRYNIA_BIND=127.0.0.1:17380
target/release/skrynia-server
```

All three paths are required. Invalid roots or unresolved root imports fail
startup. The kernel never creates a default root program or interprets its
request/response bytes. A container image built with `make docker-build`
contains only the kernel and listens on port 17380. Mount the supplied WASM
read-only and supply the private UID 1000-owned storage directories. Use an
orchestrator TCP liveness check; application health belongs to userland.

## Specification

- [Kernel contract](docs/v3-vm-spec.md): transport, authority, storage and
  execution boundary.
- [Machine ABI](docs/v3/target-abi.md): fixed signatures, scalar conventions
  and explicit unresolved choices.
- [Named typed FFI](docs/v3/typed-ffi.md): discovery and wire format.
- [In-process providers](docs/v3/syscall-optimization.md): native equivalent
  implementations.
- [Qualification](docs/v3/target-qualification.md): exact-head acceptance
  requirements.
- [Intent records](docs/intent-records): directly sourced architectural
  requirements; implementation choices are documented separately.

A passing kernel test does not qualify a complete userland stack. Never
point existing production data at an unqualified userland program. Release
and deployment remain separate operations.

Skrynia is Free Software under AGPL-3.0-or-later.
