# Skrynia v3 isolated integration (development only)

Status: experimental, **not a release candidate**. The v2 HTTP/grant server still
exists on this branch, and the external compatibility facade has not passed
v2-to-v3 differential parity. Do not deploy over an existing v2 installation.

## Build and test

Skrynia is **only the native VM server**. It must not ship or
compile a privileged Program 0. Any user may supply a conforming
WASM module at `SKRYNIA_ROOT_WASM`.

The separate `skrynia-tools` repository offers one optional Program 0
implementation; it is **not** a dependency of the kernel. Users are free
to provide completely different implementations.

Run `make test` for conformance tests using isolated, self-contained
WebAssembly fixtures. The server repository has **no tests, example
applications, deployment entry points, or build steps that assume the binary
protocol of a particular Program 0**.

The optional Program 0 and its application-specific HTTP integration tests
live entirely in `skrynia-tools`. Cross-repository verification should run
against a disposable v3 server whose root module was separately built by its
owner, never against a deployed v2 data directory.

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
