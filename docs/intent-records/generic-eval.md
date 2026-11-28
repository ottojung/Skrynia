$id-1891053258346575
title: Sole generic eval primitive with zero or one supplied function
date: 2026/11/27
source: @ottojung
kind: requirement

eval is the only kernel execution primitive. Its executable bytes reside
in the evaluator's exported linear memory, addressed by a u64 offset and
length. Snapshot the bytes before execution; no shared-RAM staging required. It instantiates a
module importing zero or one function, selected explicitly by the
evaluator. There is no binding array. No implicit imports of read, write,
close, invoke or other functions are injected into children.

The supplied function may have any integer signature: any number of i32
and i64 arguments and integer results, including zero and mixed widths.
The evaluator selects the function; the kernel validates the exact type
against the child's import. The kernel does not prescribe an import
module/name or application protocol. Non-function imports are not supplied.
skrynia-tools may supply its userland invoke convention as this function.

The function executes in its originating WASM context, not the child's
context. Modules not using fixed buffer-based calls may have no linear memory;
evaluating a child requires the evaluator's exported memory. Do not copy data between linear
memories implicitly or grow them for callbacks.

Remove eval_delegate, eval_buffered, fs_delegate_scope and kernel-specific
callback protocols. Finalize function-selector encoding, entrypoint and
result/error conventions before implementing the ABI. A caller table index
and reserved no-function value are the working proposal, not a fixed ABI.
Resource metering and execution budgets belong to custom userland VMs.

Sequential evaluations must not permanently accumulate instances in a
long-lived store. Separate child stores borrowing the suspended parent
with generic typed host trampolines are an experimentally validated
candidate, not a required engine architecture.

This supersedes the prior multiple-function binding-array design.
