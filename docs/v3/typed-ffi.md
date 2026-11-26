# Typed named FFI transport

This describes the current generic kernel's named `syscall` imports. It applies
the intent records pinned at main `0ccc90f`; it is not a legacy ABI document.

## Signature metadata

A signature is an ordered list of integer parameter types and an ordered list
of integer result types. Only `i32` and `i64` exist; both lists may be empty
and may mix widths. Internally a signature is written as `i` (i32) and `l`
(i64), for example `ili -> li`.

`GET <provider>/syscalls_list` returns a JSON array of descriptors:

```json
[{"name":"mix","params":["i32","i64","i32"],"results":["i64","i32"]}]
```

Names use the existing syscall-name grammar. Names are unique, duplicates are
rejected, and every descriptor must carry both ordered lists, including empty
ones. The provider's `POST /version`, when present, must advertise the same
signatures; a mismatch is rejected before any module is instantiated.

The kernel's `POST /version` serves build metadata plus the same descriptor
array under `syscalls`.

## Deterministic wire encoding

Arguments and results use one versioned, fixed-width little-endian encoding:

- byte 0: version, currently `1`
- byte 1: number of values (0..=16)
- per value: one type tag (`0x69` = i32, `0x6c` = i64) followed by exactly eight
  bytes of two's-complement little-endian data; an i32 is sign-extended to 64
  bits

The whole body must match the declared signature exactly: wrong version, wrong
count, wrong tag, truncated data, or trailing bytes are rejected. There is no
dispatcher, no argument-buffer or RAM request structure, and no replay or
fallback after a call begins or fails.

## Providers

`Syscalls::http` runs the reference transport: discover typed signatures, then
`POST <provider>/<name>` with the encoded arguments and decode the encoded
results. `Syscalls::optimized` registers transport-free Rust implementations
with the same signatures. `EquivalentCall::with_context` supplies the calling
machine context; HTTP providers never receive it. Machine-context access over
HTTP remains deferred.

Startup matches every root `syscall` import against the advertised exact type
and rejects unresolved or mismatched imports before execution. Results from a
native implementation are checked against the declared result types before
they are returned to Wasm.
