# Optimizing syscall execution with libskrynia

> **Implementation snapshot, not the next ABI.** The existing implementation
> described below predates the approved [target kernel design](../intent-records/kernel-abi.md).
> The intent records govern the migration; older incompatible ABI details
> below are not requirements for the new design.

The [HTTP operational prescription](syscalls.md) defines what syscalls mean.
A custom kernel may eliminate transport entirely if it preserves that
prescription's observable semantics. The library does not define another
category of syscalls, merge name sets, select preferred implementations or
perform fallback calls.

Depend on the workspace package `libskrynia`:

```toml
[dependencies]
libskrynia = { git = "https://github.com/ottojung/Skrynia" }
```

If a reference HTTP server advertises exactly `["double"]` and its POST
endpoint returns twice its four-byte argument modulo 2^32, an equivalent
custom executable is:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run_optimized(|| {
        Ok(libskrynia::Syscalls::optimized([
            libskrynia::EquivalentCall::new("double", |value| {
                Ok(value.wrapping_mul(2))
            })?,
        ])?)
    })
}
```

This example is available in `rust-server/examples/custom_kernel.rs`.
The optimized executable neither reads SKRYNIA_SYSCALL_URL nor performs
HTTP discovery or execution. Its supplied names represent the complete
discovery result of the corresponding reference server. The optimizer is
responsible for matching all names, results, side effects, ordering,
concurrency and failure semantics of that server, not merely successful
return values. An error returned by an equivalent call traps execution.
The optimization cannot be certified automatically for arbitrary endpoints.

The same WASM import `(import "syscall" "double" (func $double (param i32)
(result i32)))` works unchanged in either executable. Imports are resolved
once at startup and the resulting Wasmtime InstancePre is reused for fresh
logical instances. There is no per-invocation name lookup. This is a
transport-elision optimization, not a guarantee of machine-code inlining.
The library does not implicitly pass RAM, guest pointers or new authority
to equivalent calls. Captured state must reproduce the corresponding HTTP
server's state and synchronization. In-process code is trusted and is not
interrupted by WASM fuel; equivalent implementations must enforce the same
bounds as the reference prescription. Provider initialization occurs only
in the server, not in the isolated compiler worker.
