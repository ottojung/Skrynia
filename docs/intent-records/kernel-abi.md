$id-1891053258346574
title: Minimal shared-RAM kernel ABI
date: 2026/11/26
source: @ottojung
kind: requirement

Target fixed kernel function inventory (C-like notation; proposed exact
signatures below are the working design, subject to final ABI specification):

    i32 open_ram();
    i32 open_disk(u64 path, u64 len, u32 mode);
    i32 close(i32 fd);
    i32 read1(i32 fd);
    i32 write1(i32 fd, u32 byte);
    i64 seek(i32 fd, i64 shift);
    i32 truncate(i32 fd, u64 size);
    i32 sync(i32 fd);
    i32 mkdir(u64 path, u64 len, u32 mode);
    i32 exists(u64 path, u64 len);
    i32 is_dir(u64 path, u64 len);
    i32 delete(u64 path, u64 len);
    i32 rename(u64 old, u64 old_len, u64 new, u64 new_len);
    i32 open_directory(u64 path, u64 len);
    i32 next_entry(i32 fd, u64 dst, u64 capacity);
    i32 cas1(i32 fd, u64 offset, u32 expected, u32 desired);
    i64 now();
    i32 secure_random_bits(u64 dst, u64 bits);
    i64 eval(u64 code, u64 code_size, i32 function,
             i32 arg0, i32 arg1);

All pointers and buffer lengths are u64 and refer to shared RAM, independent
of WASM linear-memory address width. The fixed inventory contains 19
functions, plus operator-defined named FFI imports in namespace syscall.
read1/write1 are the only fundamental descriptor data transfers.
seek is relative and signed; input supports nonnegative shifts.
CAS compares and conditionally replaces exactly one RAM byte. The proposed
cas1 signature returns the observed byte directly (0..255), or a negative
error; expected and desired must be in 0..255. The exact name and scalar
signature remain working proposals. No CAS buffer or multi-byte CAS.
CAS is sequentially consistent, with acquire/release synchronization of
ordinary RAM accesses. open_directory(".") opens the selected storage
root and allows sync of top-level directory changes.
eval supplies zero or one function, of any integer signature, from the
evaluator. The kernel imposes no import name or application protocol.
No binding array. The proposed function selector is a table index with
a reserved no-function value; exact encoding is not yet finalized.

Remove native cas_ram_u64, read, write, invoke, eval_delegate, eval_buffered,
fs_delegate_scope, try_mutation_lock and release_mutation_lock.
No native registration, program-ID management or application protocols.
Allocation, memory protection and application policy belong to software.

Before implementation finalize error/EOF encodings, relative-seek return
semantics, eval function-selector/entrypoint/result conventions, typed FFI wire encoding and
copy semantics. Remote FFI machine-context access is explicitly deferred
and must not block implementation. This is an intent record, not a claim about the current ABI.
