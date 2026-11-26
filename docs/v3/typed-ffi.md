# Typed named FFI transport

This describes the generic kernel's named `syscall` imports. It follows the
intent records pinned at main `0ccc90f` and the `[K0-CHOICE]` decisions in
`docs/v3/target-abi.md` section 6 at `f538792`. It is not a legacy ABI document.

## Signature metadata

A signature is an ordered list of integer parameter types and an ordered list
of integer result types. Only `i32` and `i64` exist; both lists may be empty
and may mix widths. Internally a signature is written as `i` (i32) and `l`
(i64), for example `ili -> li`.

`GET <provider>/syscalls_list` returns:

```json
{"functions":[{"name":"mix","params":["i32","i64","i32"],"results":["i64","i32"]}]}
```

Names use the existing syscall-name grammar. Names are unique; duplicates and
missing or malformed type lists are rejected. A provider's `POST /version`,
when present, must advertise the same descriptors under `"syscalls"` alongside
`version` and `commit`; a mismatch is rejected before any module is
instantiated. The kernel's `POST /version` serves its descriptors under the
same `"syscalls"` key.

## Deterministic wire encoding

`POST /<name>` and its response use one versioned tagged little-endian layout:

- byte 0: version `0x01`
- byte 1: number of values (0..=16)
- per value: tag `0x32` plus a 4-byte little-endian `i32`, or tag `0x40` plus
  an 8-byte little-endian `i64`; no padding

The body must match the declared signature exactly: wrong version, wrong
count, wrong tag, truncated data or trailing bytes are rejected. For the HTTP
provider these are treated as provider failures before execution; the target
server specification uses `400` for malformed or mismatched input, `404` for
unknown functions and `500` for provider failures. There is no dispatcher, no
argument-buffer or RAM request structure, and no replay or fallback after a
call begins or fails.

## Providers

`Syscalls::http` runs the reference transport: discover typed signatures, then
`POST <provider>/<name>` with encoded arguments and decode encoded results.
`Syscalls::optimized` registers transport-free Rust implementations with the
same signatures. `EquivalentCall::with_context` supplies the calling machine
context; HTTP providers never receive it. Remote machine-context access stays
deferred. HTTP providers may omit functions they cannot implement, including
`copy`; no kernel fallback is required.

Startup matches every root `syscall` import against the advertised exact type
and rejects unresolved or mismatched imports before execution. Native results
are checked against the declared result types before they reach Wasm.

`copy` has exactly four explicit integer arguments; their meanings, order,
widths and result signature remain owner-unresolved and are not encoded here.
