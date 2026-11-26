$id-1891053258346574
title: Minimal shared-RAM kernel ABI
date: 2026/11/26
source: @ottojung
kind: requirement

Target fixed kernel function inventory (C-like notation; proposed exact
signatures below are the working design, subject to final ABI specification):

    i32 open_ram();
    i32 open_disk(u32 path, u32 len, u32 mode);
    i32 close(i32 fd);
    i32 read1(i32 fd);
    i32 write1(i32 fd, u32 byte);
    i64 seek(i32 fd, i64 shift);
    i32 truncate(i32 fd, u64 size);
    i32 sync(i32 fd);
    i32 mkdir(u32 path, u32 len, u32 mode);
    i32 exists(u32 path, u32 len);
    i32 is_dir(u32 path, u32 len);
    i32 delete(u32 path, u32 len);
    i32 rename(u32 old, u32 old_len, u32 new, u32 new_len);
    i32 open_directory(u32 path, u32 len);
    i32 next_entry(i32 fd, u32 dst, u32 capacity);
    i32 cas_ram_u64(i32 fd, u64 offset, u64 expected,
                    u64 desired, u32 observed);
    i64 now();
    i32 secure_random_bits(u32 dst, u32 bits);
    i64 eval(u32 code, u32 code_size, u32 bindings,
             u32 bindings_count, i32 arg0, i32 arg1);

All pointers refer to shared RAM. The fixed inventory contains 19
functions, plus operator-defined named FFI imports in namespace syscall.
read1/write1 are the only fundamental descriptor data transfers.
seek is relative and signed; input supports nonnegative shifts.

Remove native read, write, invoke, eval_delegate, eval_buffered,
fs_delegate_scope, try_mutation_lock and release_mutation_lock.
No native registration, program-ID management or application protocols.
Allocation, memory protection and application policy belong to software.

Before implementation finalize error/EOF encodings, relative-seek return
semantics, eval binding/result conventions, FFI context access and copy
semantics. This is an intent record, not a claim about the current ABI.
