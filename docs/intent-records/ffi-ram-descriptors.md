$id-1891053258346573
title: Named FFI imports with machine-context access
date: 2026/11/26
source: @ottojung
kind: requirement

Preserve the named WASM import architecture, with module namespace syscall
and one i32 argument and one i32 result (unsigned u32 bit patterns).
No generic ffi(name, argument) dispatcher. The argument and result are
opaque to the kernel; provider protocols may interpret them as shared RAM
addresses, file descriptor references, scalar values or other handles.

The reference skrynia-server discovers names via GET /syscalls_list and
executes named functions via POST /<name>, retaining the four-byte
little-endian argument and result. libskrynia executes custom Rust
implementations of those names directly, with equivalent semantics.
Do not require a new argument-buffer convention.

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
