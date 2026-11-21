# Nested eval mediation ABI (draft, kernel 62ea225+)

This document describes the generic parent-mediated nested `eval` capability
added for trusted mediator guests. It is a kernel ABI contract, not an
application policy. The kernel knows only operator-supplied Program 0 and
generic `POST /`; it does not name or special-case Program 1/P2/P3.

## Host imports

Root and mediator guests that have eval capability may import:

- `skrynia.eval(code_ptr: i32, code_len: i32, in_fd: i32, out_fd: i32, cb_idx: i32) -> i32`
  Evaluates `code_ptr..code_ptr+code_len` as a child WASM module. The child
  receives `(in_fd, out_fd)` as `skrynia_main` parameters (stream descriptors
  transferred into the child's isolated capability set). The child may import
  only `skrynia.invoke`, `skrynia.read`, `skrynia.write`, and `skrynia.close`
  with the standard signatures; it MUST NOT import `eval`/`eval_delegate` or
  any privileged host capability. On success returns the child's result code
  (0 on normal completion). Callback provider is the immediate calling instance.

- `skrynia.eval_delegate(code_ptr, code_len, in_fd, out_fd, cb_idx) -> i32`
  Same as `eval`, but the child is explicitly granted eval capability: if the
  child module imports `skrynia.eval` and/or `skrynia.eval_delegate`, the kernel
  links them. The caller must have eval capability (root always does; a guest
  has it only when instantiated via `eval_delegate`). Use this only for trusted
  mediator children; use `eval` for untrusted/leaf children.

`cb_idx` indexes the caller's exported `__indirect_function_table`. The element
must be a function `(req_ptr: i32, req_len: i32, res_ptr: i32, res_cap: i32) -> i32`
if `cb_idx >= 0`. `cb_idx < 0` means no callback; any child `invoke` returns
`ERR_PERM`. Invalid table/signature/index fails with `ERR_INVALID` before
child execution.

## Child callback and isolation

Evaluated children import only `skrynia.invoke(req_ptr, req_len, res_ptr, res_cap) -> i32`
(plus stream fd ops). `invoke` delivers the request to the immediate parent
provider that performed the `eval`/`eval_delegate`:

- Request/response staging buffers are allocated in the **provider instance's**
  linear memory (grown from current memory end as needed), never in root memory
  unless root is the provider.
- The callback executes in the provider instance with the **provider's FD
  capability guard** restored for the duration of the callback. It does not run
  with unrestricted/root FD authority when the provider is a guest mediator.
- The child remains restricted to its own transferred in/out descriptor
  capability set; it cannot access root, parent, or sibling descriptors.
- On callback trap/error, the child's FD guard and execution context are
  restored before the error is reported to the child.

This supports generic multi-level mediation chains (e.g. P0 -> P1 -> P2 -> P3)
where each mediator receives callbacks only for children it directly evaluated.

## Delegation and safety rules

- Rights are passed explicitly: `eval_delegate` grants the child eval capability;
  `eval` does not. A child cannot obtain privileged kernel imports unless
  explicitly linked per this ABI and declared by its module imports.
- Module validation rejects privileged/undeclared imports for the child's
  capability level before instantiation.
- Reentrant callback/eval, stack restore, trap/cancel cleanup, descriptor
  generation isolation, and request-owned mutation lock semantics are preserved
  by the VM.
