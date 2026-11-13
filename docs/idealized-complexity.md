# Skrynia: idealized execution and complexity model

Status: normative **for asymptotic reasoning and design targets only**.
This document does **not** describe the implementation costs or deployment
assumptions of the production Rust server. The real server still requires
crash recovery, durable private metadata, and its actual filesystem staging.

## Scope

The model preserves Skrynia's current observable operations and semantics:
public object reads, create-if-absent, replacement, byte-hash conditional
mutation, flat key and prefix scopes, grant authority and quotas, listing,
and grant-scoped Web Push wake-ups. It changes the *environment*, not the API.
Only guarantees about events excluded by the environment (notably process
termination) become vacuous.

The goal is to bound ordinary per-object operation costs without an index of
all stored objects and without temporary or auxiliary persistent files.

## Idealized environment

1. **Permanent execution.** Skrynia starts successfully, never terminates
   or restarts, and never loses RAM. There are no process crashes, power
   failures, storage faults, or exogenous interruptions. Memory allocations
   within the specified bounds succeed. Clients and remote push endpoints
   may still behave independently.
2. **Filesystem lookup.** Looking up an object by a key of length at most
   `k` takes `O(k)` time, including existence and byte-length inspection.
   The filesystem provides its own lookup machinery; that machinery is not
   counted as a Skrynia-maintained auxiliary object index.
3. **Byte access.** Reading or writing `b` bytes through a looked-up
   object takes `O(b)` time and supports a constant-size streaming buffer.
4. **Linearizable single-key operations.** The filesystem provides atomic
   create-if-absent, whole-value replace, and delete with respective costs
   `O(k+b)`, `O(k+b)`, and `O(k)`. Concurrent reads never observe
   partial values. These are *assumed primitives*, not claims about how
   ordinary operating systems implement atomic replacement. Waiting for
   unrelated operations is excluded from the work-complexity measure.
5. **Enumeration.** Enumerating `N` stored keys together with their byte
   lengths costs `O(Nk)`, without reading object contents.
6. **Persistent-space restriction.** The only filesystem objects are the
   keys and payloads directly written by clients (plus intrinsic filesystem
   bookkeeping). Skrynia may not write staging objects, journals, sidecar
   digests, indexes, grant files, or notification state. All server-owned
   metadata must be in RAM. Atomicity is provided by assumption 4, not by
   hidden application-level temporary disk usage.
7. **Networking.** Count local CPU work and the number of bytes read or
   written, not transfer delays, remote notification delivery time, or
   contention-induced waiting. Enqueuing a wake-up is part of a mutation;
   actual remote delivery is asynchronous.

This is deliberately stronger than a real filesystem. In particular,
permanent execution makes volatile grant credentials and notification state
sufficient for the modeled lifetime, without changing their API meanings.

## Parameters and accounting

- `N`: number of stored objects; use `max(1,N)` when stating bounds
  that must also cover an empty store.
- `G`: number of active grants, including the root.
- `k`: bound on key and location-matcher lengths (with `k >= 1`).
- `b`: number of payload bytes processed by the operation. For ordinary
  POST and PUT it is the uploaded value length. For conditional PUT it
  includes both the new value and all existing bytes inspected for the
  SHA-256 precondition. For GET it is the complete object's length,
  including bytes examined to compute its SHA-256 ETag even for HEAD,
  a 304 response, or a range request.
- `h`: number of grants on the acting grant's ancestor chain, including
  the acting grant.
- `s`: number of subscriptions matching the mutation and event type.
- `S`, `U`, `Q`: total subscriptions, user credentials, and pending
  notification records, respectively.

Cryptographic digests can be computed incrementally with constant-sized
working memory. Fixed-size identifiers and hash values have constant cost.
The bounds concern algorithmic work; they do not imply constant real-world
latency for synchronous writes.

## Required bounds

| Operation | Required time | Auxiliary RAM budget |
| --- | --- | --- |
| `POST /store/{key}` | `O(k+b+h+s)` | `O(Gk)` for grant/quota/object-index structures |
| `PUT /store/{key}` | `O(k+b+h+s)` | `O(Gk)` for grant/quota/object-index structures |
| `GET /store/{key}` | `O(k+b)` | `O(1)` streaming buffer beyond the key |
| Initialize counters for one new grant scope | `O(Nk)` | `O(Gk)` total after initialization |

These are **requirements of a proposed algorithmic design**, not measured
or currently achieved performance guarantees.

The `O(Gk)` budget is **total resident auxiliary memory** for grant
hierarchy, scope matching, object-count and byte-count aggregates, and
object indexing, not RAM allocated afresh on each POST/PUT. Ordinary
per-request working memory can be `O(k+1)`; a full payload buffer is not
required by the idealized primitives.

The `O(Gk)` budget is **not total process RAM**. Credentials, subscriptions,
pending notifications, network connections, and fixed implementation
overheads require additional memory (for example `O(U+Sk+Q)` when
subscription endpoint and key sizes are bounded). Their storage must be
counted separately rather than silently treated as free.

Grant-scope **counter initialization** has the `O(Nk)` target. A full
grant-creation request additionally performs authorization and ancestry
accounting in `O(k+h)`; the two costs should not be conflated. If a scope
is already indexed, it must not require a fresh object enumeration.

## Achievability sketch

Maintain a RAM-resident trie over the **distinct active exact-key and prefix
grant scopes**, not the complete set of stored object names. Each indexed
scope has an object count and the sum of payload lengths for *all currently
matching objects*, irrespective of the grant that created them.

- There are at most `G` distinct active scopes, using `O(Gk)` RAM.
- A key matches at most `O(k)` distinct prefix/exact scopes. Update their
  counters on every creation, replacement, and deletion; do not scan the
  other stored objects or all grants.
- Independently maintain `O(G)` aggregate grant-descendant metadata
  counters for grant/user/subscription charges. Changes to such metadata
  update the appropriate ancestors, not all descendants.
- To authorize a write, inspect the acting grant and its `h` ancestors,
  using the relevant maintained scope and metadata totals. An overlapping
  but unrelated grant can exceed its quota and does not veto this write;
  its observed usage must nevertheless stay accurate.
- Creating the first grant for a scope initializes that scope's object
  count and payload total by enumerating stored names and lengths:
  `O(Nk)`. Reuse an existing scope's counters for additional grants.
- Index subscriptions by exact/prefix matcher and event type. Finding and
  updating the `s` affected subscriptions takes `O(k+s)`, without
  scanning unrelated registrations or rewriting a global persistent outbox.
  Queue state is in RAM under the permanent-execution assumption.
- POST writes only the new bytes. PUT writes the new bytes; conditional PUT
  may also read the old bytes to check SHA-256, which is included in `b`.
  GET streams the object and computes the exact byte-derived ETag.

The algorithmic bounds above assume suitably indexed credential lookup
(expected constant time for bounded token lengths) and subscription state.
They do **not** depend on a RAM-resident listing of all object keys.
The filesystem serves as the authoritative source for object lookup and
grant-scope initialization.

## Acceptance criteria and exclusions

1. POST and PUT work must be bounded by `O(k+b+h+s)`, with at most
   `O(Gk)` RAM for grant/quota/object-index structures.
2. Initializing one previously unseen grant scope's quota aggregates must
   take `O(Nk)` work using only object names and lengths.
3. Direct object GET must take `O(k+b)` work and use a constant-sized
   streaming buffer beyond its key.
4. A normal per-object request may not scan all stored objects, all grants,
   or all unrelated subscriptions. New-scope initialization and explicitly
   requested enumeration are separate operations.
5. No temporary or auxiliary persistent disk space may be used.
6. Hashing, authentication, quota updates, notification matching and
   enqueueing must be included in the costs rather than hidden.
7. Existing authorization, quota and notification semantics must continue
   to hold during the modeled execution.

No bound is imposed here on scoped listing, grant-subtree revocation,
subscription management, network latency, or the real server's recovery
work. This idealized specification does **not** supersede the production
durability and deployment requirements in the API and operations documents.
