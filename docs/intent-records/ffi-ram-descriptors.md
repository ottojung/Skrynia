$id-1891053258346573
title: FFI providers interpret RAM offsets and descriptor references
date: 2026/11/25
source: @ottojung
kind: requirement

External functions are expected to handle arguments that represent
shared RAM addresses or file descriptor references. A scalar argument
may identify an arbitrarily structured request in RAM or a descriptor
for accessing data; this is not an argument-size limitation requiring
a replacement buffer-based FFI ABI.

The machine does not assign application-specific meanings to foreign
function arguments or results. Providers define their own protocols.
Do not assume that foreign functions require local WASM linear-memory
buffers or mandatory serialization of every argument into a byte stream.

The FFI execution mechanism must make the relevant RAM and descriptor
operations usable by providers. A raw numeric reference alone is not
cross-process access: specify the actual provider access mechanism,
descriptor scope/lifetime, and transport/native equivalence before
implementation. These details are not decided by this record.

This record states architectural intent, not existing implementation.
