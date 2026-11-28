$id-1891053258346571
title: Explicit shared RAM and private ABI buffers
date: 2026/11/27
source: @ottojung
kind: requirement

All fixed ABI buffer pointers and lengths are u64 and address the calling
WASM instance's exported linear memory, including paths, directory entries,
random output, bulk I/O buffers and eval code. Use an exported memory named
memory; scalar-only callers need not have memory. Never retain a borrowed
linear-memory view across a host call or reentrant WASM execution. Do not
implicitly share or copy linear memory between instances. Descriptor offsets
(such as cas1) are not linear-memory pointers. FFI signatures and address
interpretations remain provider-defined.

RAM is unpartitioned by the machine; allocation and protection policies
belong to software. RAM remains dynamically resizable via truncate:
growth initializes newly exposed bytes to zero, shrink discards the tail.
Ordinary RAM reads and writes guarantee individual byte atomicity, not
atomic multi-byte transfers or snapshots. Disjoint writes must not erase
one another. CAS is the special atomic synchronization operation and
acts on exactly one byte, at any valid RAM byte offset, without alignment
requirements. It atomically compares that byte with an expected byte and
replaces it with a desired byte on equality. It must be linearizable with
other CAS and ordinary accesses to that byte. CAS operations must be
sequentially consistent, with acquire/release synchronization of ordinary
byte accesses elsewhere in RAM: a successful release CAS publishes prior
writes to an acquiring CAS that observes that publication. Failed CAS
observations also have acquire semantics. Return the observed byte
as a scalar, not through a RAM buffer. Multi-byte CAS is not a fundamental
machine operation. This supersedes the former 64-bit CAS requirement.
Conflicting accesses to regions removed or recreated by concurrent
truncation need not have meaningful data semantics; host safety is mandatory.
Ordinary RAM data operations must not require a global RAM mutex.

RAM is accessible explicitly through descriptors, like other byte-addressable
resources, with read1/write1 or read/write. Copy between RAM locations, or
between files and RAM, can be provided as an optional optimized named FFI;
it is not a fundamental kernel transfer requirement. See ffi-fallbacks.md.

No ABI pointer implicitly addresses shared RAM. Its allocation, protection
and synchronization protocols remain userland responsibilities.
