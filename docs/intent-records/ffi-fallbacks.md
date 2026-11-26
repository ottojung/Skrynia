$id-1891053258346576
title: Named FFI accelerators with guaranteed reference fallbacks
date: 2026/11/26
source: @ottojung
kind: requirement

Distinguish fundamental machine capabilities from derived operations.
Derived operations may be exposed as named syscall WASM imports with
host-provided accelerators and portable reference implementations.
Not all external capabilities have a meaningful fallback.

For each named import, resolve a host-provided implementation if available;
otherwise resolve its reference fallback if defined; otherwise reject
the unsupported import at initialization. Select before execution;
never replay using the fallback after a call has begun or failed.
Preserve the reference operation's observable semantics, including
cursor movement, errors, effects and concurrency guarantees.

Start with copy as the standard derived FFI function:
    (import "syscall" "copy" (func (param i32) (result i32)))
Its opaque argument points to a request in shared RAM identifying source
descriptor, destination descriptor and byte count. Skrynia guarantees
a reference fallback using read1 and write1, implemented natively so
that large copies do not require a WASM instruction loop per byte.
A host may supply a faster implementation through the same named import.
Do not reintroduce fixed bulk read/write kernel imports.

Finalize the request layout, result/error codes, partial progress, EOF,
overlapping ranges, same descriptor behavior and concurrency semantics
before implementing copy. A forward byte loop is not necessarily
memmove-equivalent; reading ahead may change cursor behavior on failure.
Bound and account for host work independently of WASM instruction fuel.
The exact resource-accounting mechanism remains undecided.
