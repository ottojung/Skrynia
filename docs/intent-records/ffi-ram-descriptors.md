$id-1891053258346573
title: Named FFI imports with machine-context access
date: 2026/11/26
source: @ottojung
kind: requirement

Preserve named WASM function imports in module namespace syscall.
Generalize FFI to arbitrary integer-argument signatures: any number of
i32 and i64 parameters, including zero and mixed widths. Do not impose
a universal (i32) -> i32 signature or a fixed argument count. Support
function-specific integer results, including no result and multiple
integer results; exact result-type policy is to be finalized in the ABI.
No generic ffi(name, argument) dispatcher. Integer bit patterns are opaque
to the kernel; providers may interpret them as shared RAM addresses, file
descriptor references, scalar values or other handles.

The reference skrynia-server discovers available named functions via
GET /syscalls_list and executes them via POST /<name>. Generalize the
HTTP argument/result encoding to match each function's WASM signature.
Use a deterministic, typed integer wire representation; finalize discovery
metadata, encoding and signature-mismatch handling in the ABI.
libskrynia executes corresponding custom Rust implementations directly
with equivalent semantics. Do not require an argument-buffer convention
or force multiple scalar arguments into a RAM request structure.

Native providers must receive access to the calling machine context,
including its RAM and descriptor operations. HTTP providers must obtain
equivalent access through a private call-scoped interface. The concrete
transport, authentication, descriptor scope/lifetime, cancellation,
concurrency and revocation protocol remains to be specified. A descriptor
number by itself is insufficient to identify request-local state.
Do not expose new public application routes in the kernel for FFI.

Resolve imports during initialization. See ffi-fallbacks.md for optional
accelerators and fallback behavior. No automatic replay of failed calls.

This record states architectural intent, not existing implementation.
