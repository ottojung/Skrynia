# In-process named FFI with libskrynia

The [typed FFI contract](typed-ffi.md) defines named functions and their
ordered i32/i64 parameter/result types. An optimized kernel executable
can supply equivalent Rust implementations rather than making HTTP calls.

Depend on the workspace package:

```toml
[dependencies]
libskrynia = { git = "https://github.com/ottojung/Skrynia" }
wasmtime = "27"
```

For a provider exposing `double(i32) -> i32`, the matching native example is:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run_optimized(|| {
        Ok(libskrynia::Syscalls::optimized([
            libskrynia::EquivalentCall::new("double", "i", "i", |args| {
                Ok(vec![wasmtime::Val::I32(
                    args[0].unwrap_i32().wrapping_mul(2),
                )])
            })?,
        ])?)
    })
}
```

This is the executable example in `rust-server/examples/custom_kernel.rs`.
Signature strings use `i` for i32 and `l` for i64. Empty strings mean no
parameters or results. The declared signatures and actual result values are
checked before the values reach WASM.

`EquivalentCall::with_context` additionally gives trusted in-process Rust
code a mutable reference to the calling Wasmtime context, including the
machine's shared RAM and descriptor table. A plain numeric address does not
give an HTTP provider access to machine state; that remote access protocol
is explicitly deferred.

The optimized executable supplies the complete set of named provider
functions; it does not merge remote and native name sets or retry a failed
operation using another provider. It is responsible for equivalent results,
effects, concurrency and failure semantics. In-process code is trusted; the
kernel does not certify equivalence or meter userland execution.
