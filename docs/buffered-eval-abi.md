# Buffered eval FD bridge ABI (stacked on PR117 head `7695fd3`)

Generic bounded mediator bridge for synchronous `P1 -> P2` evaluation when a
mediator guest cannot manufacture separate independently buffered stream FDs
for child request/response (it only owns its initial input/output FDs).

No application/business policy in kernel or Program0, no specially named
Program1. Kernel remains generic `POST /` + operator-supplied Program0.

## Host import (eval-capable contexts only)

```wat
(import "skrynia" "eval_buffered"
  (func $eval_buffered
    (param $code_ptr i32) (param $code_len i32)
    (param $req_ptr i32)  (param $req_len i32)
    (param $res_ptr i32)  (param $res_cap i32)
    (param $cb_idx i32)
    (result i32)))
```

- Available to **root** and to **eval-delegated trusted mediator guests** (the
  same capability contexts that receive `eval`/`eval_delegate`; linked only when
  eval capability is present). Untrusted leaf guests evaluated without eval
  delegation MUST NOT import it (module validation rejects forbidden imports).
- `*_ptr` values are byte offsets into caller linear memory; `*_len`/`res_cap`
  are byte counts/capacities.
- `cb_idx < 0`: no callback. `cb_idx >= 0`: index into the caller's exported
  `__indirect_function_table`; entry must have signature
  `(param i32 i32 i32 i32) (result i32)` `(req_ptr, req_len, res_ptr, res_cap) -> n|err`
  and is invoked with the **immediate-parent provider context** (`ProviderCtx`)
  of the calling mediator instance.

## Return value

- `>= 0`: child succeeded and `n` bytes of buffered child output were copied to
  `res_ptr` in caller memory (`n <= res_cap`).
- `< 0`: error/trap status (existing VM/kernel error codes), e.g. `ERR_LIMIT`
  (`-10`), `ERR_OVERFLOW` (`-11`), `ERR_INVALID` (`-1`), `ERR_FAULT` (`-9`),
  `ERR_RANGE` (`-8`), `ERR_BADFD` (`-7`), etc., per existing conventions.

## Semantics

1. Caller provides child module bytes at `[code_ptr, code_ptr+code_len)` and
   request bytes at `[req_ptr, req_ptr+req_len)` in caller linear memory.
2. Kernel copies request bytes into a **bounded read-only input FD** and creates
   a **bounded write-only output spool FD** with capacity `res_cap` (in-memory,
   not blocking pipes/channels). Child receives exactly these two FDs as
   `skrynia_main(in_fd, out_fd)` with the same streaming FD contract as
   `eval`/`eval_delegate`.
3. Child executes synchronously. Callback delivery (if any) is bound to the
   immediate-parent provider context, preserving `P0->P1->P2` reentrant child
   callbacks and denying root/sibling/provider confusion. Child module is
   validated as a **non-eval leaf** for this path (`eval`/`eval_delegate`/
   `eval_buffered` imports forbidden).
4. Child may write and/or explicitly close its output FD; buffered bytes written
   before close/error are retained and captured for the caller on child success.
   Writes beyond `res_cap` fail with `ERR_LIMIT` for the child write.
5. On child success (status 0): kernel returns copied byte count. On child
   trap/fuel/error or callback fault: error/trap status is returned to caller;
   provider/guard/current-instance context is restored and child FDs/spool are
   cleaned up (no partial leaked FDs). Subsequent calls may retry/revive subject
   to VM/resource state (e.g. store fuel is not refueled within a request).

## Bounds and errors

- `code_len <= MAX_MODULE_BYTES` (module cache/compile bound) else `ERR_LIMIT`.
- `req_len <= MAX_IO` and `res_cap <= MAX_IO` else `ERR_LIMIT`.
- `MAX_IO = 64 * 1024 * 1024` bytes (`rust-server/src/boot/limits.rs`).
- Standard memory/range/permission/fault/overflow/limit codes apply.

## Isolation

- Child accesses only its transferred input/output FDs and `invoke` callback
  (when provided) through immediate-parent mediation; no other FD, root,
  sibling, or privileged host access. Callback executes with provider FD guard
  and provider instance context, restored on return/trap.
- Original streaming `eval`/`eval_delegate` remain unchanged for large/full-duplex
  FD-transfer cases.

## Non-goals / notes

- Not a full-duplex pipe pair; no deadlock-prone finite synchronous parent+child
  pipe.
- HTTP request cancellation follows existing VM/request termination cleanup.
  VM/WAT coverage includes access/limits/overflow/isolation/repeat, child trap
  and callback fault/revive (including reentrant path), fuel-exhaustion clean
  failure, and guest-close-of-output capture. Further real compiled
  mediator/native-service and composition/cancel-under-load qualification is
  required on canonical kit before claiming guest `P1->P2` works.
