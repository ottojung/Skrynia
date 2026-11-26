$id-1891053258346575
title: Sole generic eval primitive and explicit function bindings
date: 2026/11/26
source: @ottojung
kind: requirement

eval is the only kernel execution primitive. Its executable bytes and
binding-array data reside in shared RAM. It instantiates a module with
exactly the functions explicitly supplied by its evaluator, resolving
their identities and validating their types. No implicit imports of
read, write, close, invoke or other functions are injected into children.

A supplied function executes in its originating WASM context, not the
calling child's context. The machine has no knowledge of the userland
invoke protocol. Modules may have no linear memory; no memory export
is required solely to communicate with the machine. Do not copy data
between linear memories implicitly or grow them for callbacks.

Remove eval_delegate, eval_buffered, fs_delegate_scope and kernel-specific
callback protocols. Software implements policy through supplied functions.
Finalize the shared-RAM binding representation, exact types, entrypoint,
result/error encoding and resource limits before implementing the ABI.

Sequential evaluations must not permanently accumulate instances in a
long-lived Wasmtime store. Consider separate child stores borrowing the
suspended parent, with generic typed host trampolines and aggregate fuel
accounting. This is an experimentally validated candidate implementation,
not a required engine architecture. The experiment did not establish
support for cross-store reference-typed values.
