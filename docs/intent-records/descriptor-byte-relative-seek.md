$id-1891053258346572
title: Byte-only fundamental descriptor I/O and relative seek
date: 2026/11/26
source: @ottojung
kind: requirement

read1 and write1 are the only fundamental descriptor data-transfer
operations. Each transfers exactly one byte on success. Do not retain
buffered read or write as fixed kernel imports. Bulk transfer is a
derived named FFI operation with a reference fallback, not an additional
fundamental capability. The byte operations need no RAM buffer.

seek takes a signed relative displacement from the current position,
not an absolute offset. Zero leaves the position unchanged, +1 advances
one byte, and -1 moves one byte backward where supported.
The input stream descriptor must support nonnegative relative seeking:
a positive shift advances over (discards) input bytes, and zero is a no-op.
Backward seeking on a forward-only input stream is not supported.

Specify overflow, underflow, short skips, end-of-stream and cursor
behavior before implementing the final ABI. The exact scalar return/error
encoding for read1 and write1 and supported output-stream seek behavior
are not established by this record.

Supersedes the earlier bulk-plus-byte I/O intent and absolute-seek ABI.
See kernel-abi.md and ffi-fallbacks.md.
