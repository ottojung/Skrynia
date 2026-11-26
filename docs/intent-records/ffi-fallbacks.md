$id-1891053258346576
title: Optional named FFI accelerators and software fallbacks
date: 2026/11/26
source: @ottojung
kind: requirement

Distinguish fundamental machine capabilities from derived operations.
Derived operations may be exposed as named syscall WASM imports with
host-provided accelerators and portable software implementations.
Not all external capabilities have a meaningful fallback.

Software providing a derived operation may use an available named FFI
implementation or its own reference fallback. The fallback belongs to
the software providing that operation (Program 0, Program 1 or another
program/library), not necessarily to the native kernel. The kernel must
not guarantee that every derived FFI name exists. Ordinary unresolved
WASM imports still fail linking. Select the implementation before
execution; never replay using a fallback after a call has begun or
failed. The availability detection and software linking mechanism
remain to be specified. Preserve reference observable semantics,
including cursor movement, errors, effects and concurrency guarantees.

Start with copy as the standard derived FFI function. It has exactly
four explicit integer arguments, not a single pointer to a RAM request
structure. The meanings, order and widths of the four arguments and
the return signature remain to be finalized; do not invent them.
A software provider can implement copy using read1 and write1 when
an accelerated FFI implementation is unavailable. A custom libskrynia
implementation may accelerate it natively. The HTTP syscall provider
may omit copy entirely; it need not access the machine's RAM or
descriptors. Do not require a native kernel fallback or make remote
machine-context access a prerequisite for this design.
Do not reintroduce fixed bulk read/write kernel imports.

Finalize the signature, result/error codes, partial progress, EOF,
overlapping ranges, same descriptor behavior and concurrency semantics
before implementing copy. A forward byte loop is not necessarily
memmove-equivalent; reading ahead may change cursor behavior on failure.
Bound and account for host work independently of WASM instruction fuel.
The exact resource-accounting mechanism remains undecided.
