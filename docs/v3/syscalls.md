# Syscalls

> **Implementation snapshot, not the next ABI.** The existing implementation
> described below predates the approved [target kernel design](../intent-records/kernel-abi.md).
> The intent records govern the migration; older incompatible ABI details
> below are not requirements for the new design.

Program 0 may import functions from the WASM module namespace `syscall`.
Each has exactly one i32 argument and one i32 result, interpreted as unsigned
32-bit bit patterns. Names are 1–128 ASCII letters, digits, underscores,
dots or hyphens, excluding the path segments "." and "..".
The kernel does not interpret argument or result contents. All other existing
root ABI rules remain unchanged. Evaluated children cannot import this
namespace, including children with delegated evaluation or filesystem access.

## Operational prescription

Set `SKRYNIA_SYSCALL_URL` to the base URL of a separate HTTP server.

At startup Skrynia sends `GET <base>/syscalls_list` without a request body.
The server must answer HTTP 200 with a JSON array of strings, for example
`["publish","unpublish"]`. Each name must follow the name rule above.
Skrynia validates, sorts and deduplicates the names. It rejects non-200
responses, redirects, malformed JSON, wrong types, malformed names and
responses over 8 KiB. The request times out after 30 seconds. If no URL is
configured, the list is empty.

An unavailable name, wrong type or unknown namespace fails startup.
Each invocation of a syscall sends exactly one `POST <base>/<name>`,
with the name appended as a path segment. The request body and successful
HTTP 200 response body must each consist of exactly four little-endian bytes.
Requests use application/octet-stream. Calls have a 30-second timeout.
Skrynia never follows redirects or retries. A network failure, non-200 response
or invalid result length traps execution. An operation may have taken effect
despite a failure; Skrynia never replays it.

The HTTP server receives only the numeric argument; an offset does not grant
access to the kernel's RAM. Any additional data transfer is defined by the
syscall's protocol. No guest memory or global RAM is implicitly exposed.

## Discovery by clients

`POST /version` on Skrynia returns JSON with `version`, `commit` and
`syscalls`, the sorted, deduplicated discovery list, including names not
imported by the current root. It does not invoke the root or rediscover names.
The fixed `skrynia` ABI is documented separately and is not included in this
list. Other methods on this endpoint return 404.

An implementation may optimize the prescription above without changing its
observable behavior. See [Optimizing syscall execution with libskrynia](syscall-optimization.md).
