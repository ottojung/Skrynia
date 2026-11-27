$id-1891053258346572
title: Fundamental descriptor I/O and relative seek
date: 2026/11/27
source: @ottojung
kind: requirement

Retain read1 and write1, each transferring exactly one byte on success.
Add buffered read and write as fixed kernel imports, transferring between a
descriptor and the caller's private WASM linear memory. Buffer pointers
and lengths are u64. Return the actual byte count or a negative error;
read returns zero at EOF. Partial transfers are allowed, the full buffer
range is validated before I/O, and the cursor moves by actual progress.
Byte operations require no linear memory. Bulk RAM operations guarantee
individual byte atomicity, not multi-byte transactions.

seek takes a signed relative displacement from the current position,
not an absolute offset. Zero leaves the position unchanged, +1 advances
one byte, and -1 moves one byte backward where supported.
The input stream descriptor must support nonnegative relative seeking:
a positive shift advances over (discards) input bytes, and zero is a no-op.
Backward seeking on a forward-only input stream is not supported.

See kernel-abi.md, shared-ram-abi.md and ffi-fallbacks.md.
