$id-1891053258346571
title: All machine ABI buffers address shared RAM
date: 2026/11/25
source: @ottojung
kind: requirement

All memory pointers and buffer ranges in the Skrynia machine ABI refer to
the single shared RAM object, never to a caller's WASM linear memory.
This applies to descriptor transfers, paths, directory entries, random
output, CAS observation buffers (if used), and eval code and binding arrays.
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

read on a RAM descriptor copies from its cursor into a shared RAM range;
write copies from a shared RAM range into the descriptor's cursor.
The same ABI applies to all other supported descriptor types.
Exact overlapping-copy semantics remain to be specified.

eval is the sole evaluation operation. Its code and binding-array pointers
now address shared RAM, superseding the previous local-linear-memory design.
It binds only explicitly supplied functions; no implicit invoke or other
application-specific imports are supplied by the machine. The binding
representation and complete signature remain subject to ABI specification.

This record states architectural intent, not existing implementation.
