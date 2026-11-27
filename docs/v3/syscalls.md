# Named syscall imports

The current [typed FFI contract](typed-ffi.md) defines named integer
signatures, discovery, version metadata, wire encoding, validation and failure
semantics. The root imports functions from WASM namespace `syscall`. Evaluated
children receive only the explicitly supplied function; they do not obtain
implicit provider imports.

Set `SKRYNIA_SYSCALL_URL` to the reference HTTP provider's base URL. Discovery
and calls use a 30-second timeout, no redirects and no retries. A non-success
response, malformed result or transport failure traps the call. A failed call
may already have had an effect; do not replay it.

The [in-process equivalent provider](syscall-optimization.md) eliminates HTTP
while preserving the declared function signatures and observable semantics.
