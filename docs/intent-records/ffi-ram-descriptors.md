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
GET /syscalls_list and executes them via POST /<name>. Discovery must
advertise every available function's name and ordered parameter and result
types, including empty lists. POST /version must expose the available
signatures as well as build metadata, for deployment tooling. Generalize the
HTTP argument/result encoding to match each function's WASM signature.
Use a deterministic, typed integer wire representation; finalize discovery
JSON shape, binary encoding and signature-mismatch handling in the ABI.
Reject unresolved or mismatched imports before execution.
libskrynia executes corresponding custom Rust implementations directly
with equivalent semantics. Do not require an argument-buffer convention
or force multiple scalar arguments into a RAM request structure.

Native Rust implementations may use the calling machine context,
including RAM and descriptor operations. Remote HTTP providers receive
the encoded integer arguments; they are not required to have access to
the caller's RAM or descriptors. General remote machine-context access
is deliberately deferred. Do not introduce a private RPC protocol,
shared-memory transport or new kernel endpoints to solve it now.
A descriptor number by itself is insufficient to identify request-local
state. An HTTP provider may omit functions it cannot implement without
machine access, including copy. This does not block the new ABI or
migration to it.

Resolve imports during initialization. See ffi-fallbacks.md for optional
accelerators and fallback behavior. No automatic replay of failed calls.

This record states architectural intent, not existing implementation.
