# Verification

The Rust test suite checks the actual server and persistent state through
isolated temporary directories. The release gate is `make test`, which
includes a source-language gate, unit tests, HTTP integration tests, default rustfmt, strict Clippy,
and the structural linter (50 code lines per Rust file, 10 direct entries
per source directory, sibling-only local imports).

Integration tests exercise root credential retrieval, public reads,
conditional object mutations, grant attenuation, missing/invalid keys,
private state isolation, and rejection of unsafe notification endpoints.
The crate's unit tests cover journal replay, one canonical durable outbox
write for one or 50 subscribers, one write for 50 acknowledgments,
periodic acknowledgement flushing despite a slow delivery, generation checks,
retry behavior, and fault recovery.
Integration tests verify fan-out generations, attempts, and due times
across a process kill and restart, reject orphaned or corrupted records,
recover interrupted grant revocations without orphaned notifications,
and verify that deleting an idle subscription does not rewrite the outbox.
Additional tests cover Web Push encryption, TLS, DNS filtering, quotas,
and stage-file cleanup.

Any deployment must use persistent storage, external HTTPS rate limiting,
a tested consistent backup/restore procedure, and a verified isolated
browser-origin configuration. Passing tests alone is not evidence of
end-to-end delivery to an external push provider or resilience to
physical media faults. These require operational checks on the target
environment.

Do not claim a release passed unless the exact pushed commit's checks
are green. Tests and build procedures must not require other runtime languages.


## Reproducible HTTP throughput probe

Run `SKRYNIA_BENCH_SAMPLES=100 make benchmark`. The Rust workload driver
builds the release binary, starts it with a fresh private temporary data
directory on a loopback port, issues a root user credential, and measures
sequential 4 KiB GET, SHA-256-conditioned PUT, and unconditional PUT
operations, in that order. Each PUT replaces the same key with a new
4 KiB payload. The conditional PUT computes the prior-body digest outside
the timed interval. Requests verify successful HTTP responses; the
unconditional PUT workload also reads back the final bytes. The benchmark
server and its files are cleaned up afterward.

Three successive runs on shared `marceline-dev` (2026-11-13, release
binary, 100 operations per workload, local filesystem):

| Run | GET (ops/s) | CAS PUT (ops/s) | Plain PUT (ops/s) |
| --- | ---: | ---: | ---: |
| 1 | 6751.5 | 66.4 | 75.3 |
| 2 | 8161.3 | 20.8 | 70.7 |
| 3 | 6039.7 | 22.2 | 76.2 |

Plain PUT p50 was 12.22–13.13 ms and p99 was 20.55–23.92 ms.
CAS PUT p50 was 13.30–16.44 ms, with unusually large p95 values
(329–358 ms) in two runs. This is **not** a controlled comparison:
the operations execute in a fixed order on a shared host, and resource
contention may explain the tail-latency outliers. Re-run with interleaved
orders, intended deployment storage, sustained concurrency, and notification
load before claiming a CAS overhead or production capacity.

## Idealized complexity implementation audit

The algorithmic target is specified in
[idealized-complexity.md](idealized-complexity.md). That model assumes
permanent execution and stronger single-key atomic filesystem primitives.
The *actual* server retains crash recovery and durable private metadata;
this section distinguishes their costs rather than treating them as free.

| Component | Idealized time | Indexed RAM | Implementation |
| --- | --- | --- | --- |
| Direct object GET | `O(k+b)` | `O(1)` buffer | Filesystem lookup, streamed SHA-256 and response |
| New grant scope initialization | `O(Nk)` | `O(Gk)` | Enumerate existing object names and lengths once |
| Mutation quota accounting | `O(k+h)` | `O(Gk)` | Prefix trie totals, shared per-grant counters and ancestor aggregates |
| Subscription matching | `O(k+s)` | `O(Sk)` separately | Exact/prefix/event trie |
| POST/PUT excluding durability | `O(k+b+h+s)` | `O(Gk)` quota/object index | Filesystem key check, byte processing, scope updates and subscription match |

Here `b` includes old bytes read for SHA-256 conditional PUT.
Authentication uses expected-constant-time hashed token lookup.
Grant lookup assumes expected-constant-time hash table access; the bounds
exclude scheduling waits on the server's shared mutation lock.
The object index no longer scales in RAM with the object count `N`.
Credential, subscription, and notification state have separate memory
costs; neither `O(Gk)` nor this table means total server RSS is `O(Gk)`.

The production server does **not** attain strict end-to-end
`O(k+b+h+s)` for every mutation: when `s>0`, its current durable
single-file notification outbox sorts and writes all `P` pending
records, adding `O(P log P)` CPU and `O(P)` I/O and serialization memory.
Enqueue rollback now retains only the `s` affected prior records. Crash-safe journal/staging files and whole-upload buffers
also require auxiliary persistent space and `O(b)` upload RAM, respectively.
These production costs are **not permitted** by the idealized model.
Removing durability for the sake of an asymptotic claim is not acceptable
for the production server.

Scoped paginated listing is deliberately excluded from the point-GET
bound: without an all-key index it enumerates `N` directory entries and
retains only `L+1` sorted candidates, costing
`O(Nk + N log(L+1))` time and `O(Lk)` result memory. This is the
intended time/space trade-off.

The tests include direct oracle comparisons of the indexed grant totals
and subscription matcher against full scans, plus restart, nested-grant,
revoke, conditional mutation, listing and durability tests.

## Object-count memory probe

One exploratory release-build comparison on shared `marceline-dev`
(2026-11-13 UTC), with an isolated store containing `N` one-byte
objects and only the root grant. Each process was started fresh and
sampled shortly after its health endpoint became ready. The values
are `RssAnon` from Linux `/proc/<pid>/status`, in KiB.

| Stored objects | Before the scope-index refactor | Scope-index refactor |
| ---: | ---: | ---: |
| 0 | 1,024 | 1,280 |
| 5,000 | 1,536 | 1,024 |
| 25,000 | 3,328 | 1,024 |

The original server's resident anonymous memory grows as objects are
added; the new server's does not show that growth. This is consistent
with removing the all-object `BTreeMap`, and does not replace the
asymptotic source-code analysis. These are single samples, not
controlled measurements of a production workload, and do not
include filesystem page-cache memory.

A same-host sequential 100-sample throughput comparison, using the
existing 4 KiB Rust benchmark, produced:

| Build | GET/s | CAS PUT/s | Unconditional PUT/s |
| --- | ---: | ---: | ---: |
| Before scope-index refactor | 6,779.8 | 61.6 | 58.4 |
| Scope-index refactor | 7,378.4 | 77.7 | 58.9 |

The paired samples are too small and the host too variable to
attribute a throughput improvement to the refactor. Memory
independence from `N` is the principal established benefit.

## Robustness review qualification

- Child grant creation rejects missing, wrongly typed and unknown authority
  fields, avoiding accidental inheritance from the parent.
- Atomic JSON writes distinguish clean pre-rename failure from ambiguous
  post-rename failure. Clean failures are retryable without poisoning the
  service; ambiguous failures preserve the recovery latch. `/health` already
  returns 503 when latched; the container healthcheck makes this visible to
  a configured orchestrator. Production Rust uses `panic = "abort"`.
- Scoped listing authorizes under the lock, then releases it before directory
  enumeration. Initial grant scope scans happen outside the lock and retry
  when an intervening object mutation changes the revision. Grant revocation
  rebuilds remaining scope counters in RAM without another full disk scan.
- Subscription public keys are validated as uncompressed P-256 curve points
  with 16-byte authentication secrets. Push destinations have no host allowlist;
  URL validity and public-address resolution remain enforced.
- Rust MSRV, CI verification toolchain and Docker Rust toolchain are 1.93.
- Per-request SHA-256 computation for GET and HTTP cache validators remains
  intentional: a per-object digest cache would consume additional O(N)
  resident memory, conflicting with the established low-memory object
  indexing design. External static servers and reverse-proxy rate limiting
  remain the appropriate defenses against abusive public-read traffic.
- The mandated structural limits necessarily partition Rust files into small
  modules; this review does not silently repeal them or restructure the crate
  facade. Native cryptography for Web Push key operations and rustls for
  outbound TLS are retained rather than replacing verified security code
  merely to reduce the lockfile's dependency count.
