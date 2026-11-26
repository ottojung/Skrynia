$id-1891053258346572
title: Single-byte descriptor I/O and relative seek
date: 2026/11/25
source: @ottojung
kind: requirement

Add read1 and write1 as descriptor operations for reading and writing
exactly one byte. They complement bulk read/write, which transfer bytes
between descriptors and shared RAM. Single-byte I/O is intended to avoid
requiring an intermediate RAM buffer for ordinary scalar byte access.

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

This record supersedes incompatible absolute-seek and non-seekable-input
intent. It states architectural intent, not existing implementation.
