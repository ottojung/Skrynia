# Skrynia v3 isolated integration (development only)

Status: experimental, **not a release candidate**. The v2 HTTP/grant server still
exists on this branch, and the external compatibility facade has not passed
v2-to-v3 differential parity. Do not deploy over an existing v2 installation.

## Build and test

Use Rust 1.93 with the `wasm32-unknown-unknown` target and `rust-src` installed.
When only the Rust sources are available, build the target standard library with:

```sh
RUSTC_BOOTSTRAP=1 cargo -Z build-std=std,panic_abort build \
  --target wasm32-unknown-unknown --release -p skrynia-program0
```

The resulting artifact is
`target/wasm32-unknown-unknown/release/skrynia_program0.wasm`
(or below `$CARGO_TARGET_DIR`, when set).

Run `make test` and then the **separate** disposable integration test:

```sh
cargo run --locked -p skrynia-server --example v3_smoke -- \
  target/wasm32-unknown-unknown/release/skrynia_program0.wasm
```

The smoke test uses fresh temporary data only. It executes a real compiled root
and guest WASM, registers a program, invokes it, changes its definition,
invokes the new code, restarts the kernel, recovers the durable version, revokes
it and tests invalid output lengths, memory limits, and directory bounds.

The fixed Program 0 wire framing is currently an application protocol:
version byte 1; operation 1 register, 2 set, 3 invoke, 4 lookup, 5 revoke;
32-byte opaque program identifier, length-prefixed inputs; responses use
version, status byte, 32-bit little-endian body length and body. It is **not**
part of the native kernel or a substitute for the old grant-based protocol.

## Security boundaries and unfinished release gates

Program 0 uses explicit kernel disk and evaluation calls; requests, compiled
modules and all VM linear memories remain separate from global RAM. The kernel
has per-request fuel, stack and StoreLimits plus a bounded, SHA-256-keyed
compiled guest cache shared across requests. Guest imports other than
`skrynia.invoke` are rejected.

These checks do **not** establish full v3 correctness. Remaining requirements
include application authorization/delegation and complete recursive callback
support, cross-request transaction coordination for registry changes, durable
revoke semantics, eliminating v2 business logic from the new native server,
full v2 HTTP/grant/quota/Web Push compatibility in `skrynia-tools`, production
resource limits, comprehensive fault injection, independent security review,
and reproducible migration with rollback. No production cutover is authorized
by these tests.
