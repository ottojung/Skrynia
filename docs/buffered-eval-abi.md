# Buffered eval FD bridge ABI (draft, stacked on PR117 head 7695fd3)

Generic bounded mediator bridge for synchronous `P1 -> P2` evaluation when the
mediator guest cannot manufacture separate independently buffered stream FDs
for the child request/response (it only owns its initial input/output FDs).

No application/business policy in kernel or Program0, no specially named
Program1. Kernel remains generic `POST /` + operator-supplied Program0.

## Host import (eval-capable contexts only)

- `skrynia.eval_buffered(code_ptr: i32, code_len: i32, req_ptr: i32, req_len: i32, res_ptr: i32, res_cap: i32, cb_idx: i32) -> i32`

Available to root and to **eval-delegated trusted mediator guests** (same
capability contexts that receive `eval`/`eval_delegate`; linked only when eval
capability is present). Untrusted leaf guests evaluated without eval delegation
MUST NOT import it (module validation rejects it as undeclared/forbidden).

## Semantics

1. Caller (mediator P1) provides child module bytes at `code_ptr..+code_len`
   and child request bytes at `req_ptr..+req_len` in caller linear memory.
2. Kernel creates a **read-only input FD** backed by a bounded in-memory copy of
   the request (not a blocking pipe) and a **write-only child output FD** backed
   by a bounded in-memory spool with capacity `res_cap` (not a blocking channel).
   P2 receives exactly these two FDs as `skrynia_main(in_fd, out_fd)` — same
   streaming FD contract as `eval`/`eval_delegate`.
3. Child executes synchronously. If `cb_idx >= 0`, callback delivery uses the
   **immediate-parent provider context** (`ProviderCtx`) of the calling mediator
   instance (same as nested eval callback binding), preserving P0->P1->P2
   reentrant child callbacks and denying root/sibling/provider confusion.
4. On child success (status 0), kernel copies buffered child output
   (`0..written <= res_cap`) to `res_ptr` in caller memory and returns the byte
   count. If produced output exceeds the mediator-declared `res_cap`, the child
   write fails with `ERR_LIMIT` and evaluation fails accordingly (no partial
   leaked FDs). If a successful child produced more bytes than fit the copy
   contract, `ERR_OVERFLOW` is returned.
5. On child trap/fuel/error or callback fault, the error/trap status is returned
   to the caller; provider/guard/cur context is restored and guest FDs/spool are
   cleaned up (no partial leaked FDs). Mediator may retry/revive with subsequent
   calls (subject to VM/resource state).

## Bounds and errors

- `code_len` bounded by `MAX_MODULE_BYTES` (`ERR_LIMIT`).
- `req_len` and `res_cap` bounded by `MAX_IO` (`ERR_LIMIT`); request bytes are
  copied from caller memory before child run.
- Module validation: child treated as non-eval leaf (`eval`/`eval_delegate`/
  `eval_buffered` imports forbidden for child module unless separately delegated
  via a different eval path) — untrusted leaves denied.
- Normal memory/range/badfd/invalid/limit/overflow/fault/trap codes apply as per
  existing VM conventions.

## Isolation

- Child accesses only its transferred input/output FDs and callback `invoke`
  (when provided) through immediate-parent mediation; no other FD, root,
  sibling, or privileged host access. Callback executes with provider FD guard
  and provider instance context, restored on return/trap.
- Original streaming `eval`/`eval_delegate` remain unchanged for large/full-duplex
  FD-transfer cases.

## Non-goals / notes

- Not a full-duplex pipe pair; no deadlock-prone finite synchronous parent+child
  pipe. Buffering is bounded and kernel-owned for this bridge.
- HTTP request cancellation follows existing VM/request termination cleanup; VM
  tests cover trap/callback-fault/revive and bounded overflow/limits. Further
  composition/cancel-under-load qualification required on canonical kit.
