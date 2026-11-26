$id-1891053258346571
title: All machine ABI buffers address shared RAM
date: 2026/11/26
source: @ottojung
kind: requirement

All memory pointers and buffer ranges in the Skrynia machine ABI refer to
the single shared RAM object, never to a caller's WASM linear memory.
This applies to paths, directory entries, random output, CAS observation
buffers (if used), eval code and binding arrays, and derived FFI buffers.
Scalar values, descriptor identifiers and function-table indices remain
values of their respective types, not RAM addresses.

RAM is unpartitioned by the machine; allocation and protection policies
belong to software. RAM remains dynamically resizable via truncate:
growth initializes newly exposed bytes to zero, shrink discards the tail.
Ordinary RAM reads and writes guarantee individual byte atomicity, not
atomic multi-byte transfers or snapshots. Disjoint writes must not erase
one another. CAS remains the special atomic synchronization operation.
Conflicting accesses to regions removed or recreated by concurrent
truncation need not have meaningful data semantics; host safety is mandatory.
Ordinary RAM data operations must not require a global RAM mutex.

RAM is accessible through descriptors like other byte-addressable resources.
The only fundamental descriptor data-transfer operations are read1 and
write1. Bulk RAM transfers are copies between descriptors, supplied as
derived FFI rather than native read/write imports. Exact overlapping-copy
semantics remain to be specified.

eval code and binding-array pointers address shared RAM. A WASM module
need not define or export linear memory. See generic-eval.md.

This record states architectural intent, not existing implementation.
