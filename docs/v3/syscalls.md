# Operator-defined syscalls

Program 0 may import functions from the reserved WASM module namespace
`syscall`. Each has exactly one i32 argument and one i32 result, interpreted
as unsigned 32-bit bit patterns. Names are 1–128 ASCII letters, digits,
underscores, dots or hyphens, excluding the path segments "." and "..".
The kernel does not interpret argument or result contents. All other existing
root ABI rules remain unchanged. Evaluated children cannot import this
namespace, including children with delegated evaluation or filesystem access.

## Resolution and execution

The operator registers native functions and may configure one HTTP provider.
The provider's supported names are discovered at startup; native registrations
win when the same name is present in both. An unavailable name, wrong type,
or unknown namespace fails startup. The kernel resolves imports and creates a
Wasmtime InstancePre once at startup; fresh logical instances reuse the
resolved imports. There is no per-call name lookup or HTTP on the native path.
The WASM-to-native boundary is not guaranteed to be machine-code inlined.

## Custom kernel

Depend on the workspace package `libskrynia`, for example:

```toml
[dependencies]
libskrynia = { git = "https://github.com/ottojung/Skrynia" }
```

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut syscalls = libskrynia::Syscalls::new();
    syscalls.register("double", |_, arg| Ok(arg.wrapping_mul(2)))?;
    libskrynia::run(syscalls)
}
```

The repository includes a runnable `custom_kernel` example.
Program 0 can import this function using
`(import "syscall" "double" (func $double (param i32) (result i32)))`.
A native provider receives a reference to process-global RAM, not a guest
linear-memory pointer. It can interpret an argument as a RAM offset, use
the existing synchronized RAM operations, or use it as an ordinary value
or provider-defined handle. It must check transfer lengths/ranges and follow
the same application synchronization protocol as Program 0. The kernel does
not allocate or manage RAM on behalf of a syscall. Provider state can persist
through a captured thread-safe object. Use `libskrynia::run_with` to create
stateful providers inside a factory that is not executed by the isolated
compiler-worker subprocess. Providers must be thread-safe, trusted
and bounded; native code is not sandboxed or interrupted by WASM fuel.

## HTTP provider discovery

Set the single environment variable `SKRYNIA_SYSCALL_URL` to the provider base
URL. There is no `SKRYNIA_SYSCALLS` variable and no operator-supplied name list.
Custom kernels can call `Syscalls::http_provider(base)` instead.

At startup the kernel sends `POST <base>/version` with an empty
`application/octet-stream` body. The provider must answer HTTP 200 with a JSON
object whose `syscalls` key holds an array of strings; extra fields are
ignored. Each name must follow the name rule above and `version` is reserved
because it collides with the discovery route. The kernel rejects non-200
responses, redirects, malformed JSON, wrong types, malformed names, the
reserved name and responses over 8 KiB; the request times out after 30
seconds. Advertised names are validated, sorted, deduplicated and merged with
native registrations (native wins) before imports are resolved and the names
are published through the kernel's own `POST /version`. With no URL set the
kernel is native-only and sends no discovery request.

For an imported name with no native implementation, the kernel issues one
POST to the base URL with the name appended as a path segment. Request body
and successful HTTP 200 response body must each consist of exactly four
little-endian bytes. Requests use application/octet-stream. The HTTP client
reuses connections, has a 30-second timeout, and never follows redirects or
automatically retries. Network failures, non-200 responses, invalid result
lengths and native provider errors trap execution. An operation may have
taken effect despite a failure; the kernel never replays it.

An HTTP provider receives only the numeric argument; it does not acquire
access to the kernel's RAM by receiving an offset. A provider requiring
shared data needs an independently designed data-sharing protocol; the
generic HTTP transport does not implicitly expose RAM or private memory.
Thus scalar/handle-based protocols can be identical across native and HTTP
providers, while RAM-offset-based protocols require an explicit memory bridge
for HTTP interoperability.

## Discovery

`POST /version` returns JSON with the existing `version` and `commit` build
metadata and `syscalls`, a sorted, deduplicated list of registered native and
discovered provider names, whether or not the currently loaded root imports
them. It does not test remote service availability or execute Program 0.
Native-only hosts require no fallback configuration. The fixed `skrynia` ABI
is documented separately and is not included in this list. Other methods on
this endpoint return 404.
