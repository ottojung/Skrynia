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

## Filesystem-aware complexity implementation audit

The [filesystem-aware complexity model](complexity.md) charges disk lookup,
reads/writes, directory operations, enumeration, synchronization, and all
auxiliary indexes separately from CPU work and RAM. The following describes
the **current Rust implementation**; no filesystem-independent elapsed-time
bound is claimed. Every row additionally incurs the actual filesystem trace.

| Component | Current CPU work, excluding filesystem | Indexed RAM | Filesystem work |
| --- | --- | --- | --- |
| Direct object GET | `O(k+b)` | `O(1)` streaming buffer | `FSL(N,k) + FSR(b,e)`; reads whole value for SHA-256 |
| New prefix grant | `O(k*ell*log(max(2,W)))` plus ancestry | `O(Gk+mk)` | Binary seeks into ell <= O(log W) immutable delta runs |
| Mutation quota accounting | `O(k+h+k*log(m))` before compaction | `O(Gk+mk)` | In-memory updates; occasional sequential run flush/merge |
| Subscription matching | `O(s_k)` per changed key/event | `O(S)` separately | In-memory exact-key/event index |
| POST/PUT excluding durable staging/journal/outbox | `O(k+b+h+s+k log(m))` plus old bytes and occasional compaction | `O(Gk+mk)` | Staging, journal, public mutation and derived-index flushing/merging each charged |

Here `b` includes old bytes read for SHA-256 conditional PUT.
Authentication uses expected-constant-time hashed token lookup.
Grant lookup assumes expected-constant-time hash table access; the bounds
exclude scheduling waits on the server's shared mutation lock.
The active grant-scope index and capped delta buffer do not scale in
RAM with live object count `N`; the signed-delta runs are private disk files.
Here `m <= 4096` is the current adaptive buffer cap, and `ell` counts runs.
Credential, subscription, and notification state have separate memory
costs; neither `O(Gk)` nor this table means total server RSS is `O(Gk)`.

Mutations incur further real costs: when `s>0`, the current durable
single-file notification outbox sorts `P` pending records and writes its
serialized `p` bytes, adding `O(P log P + p)` CPU and a full outbox
rewrite with filesystem synchronization. Enqueue rollback retains only
the `s` affected prior records. Staging, journaling, and whole-upload
buffers cost persistent space, writes/syncs, and `O(b)` upload RAM.
These are accounted for in the new model, not hidden behind an idealized
constant-time filesystem assumption.

Scoped paginated listing is deliberately excluded from the point-GET
bound: without an all-key index it enumerates `N` directory entries and
retains only `L+1` sorted candidates, costing `O(Nk(1 + log(L+1)))` CPU work and `O(Lk)` result memory,
**plus** full directory-enumeration filesystem cost `FSE(N,k)`.
This is the current implementation's time/space trade-off, not a
required property of the API.

The tests include direct oracle comparisons of the indexed grant totals
and subscription matcher against full scans of unrelated subscribers, plus restart, nested-grant,
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
  enumeration. New prefix grant totals are obtained from the derived
  signed-delta index under the writer lock, not by directory scans.
  Revocation rebuilds active grant counters without rescanning public objects.
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

## Follow-up durability and progress review

- Push acknowledgment recovery is checked at the **runner** as well as in
  the batch writer. A failed atomic outbox write before rename leaves the
  in-memory generation unchanged, keeps the worker running, and retries on
  a later delivery tick. A post-rename/uncertain failure still latches.
- Initializing an unseen prefix scope now sums the bounded in-memory
  delta trie and immutable sorted runs; no mutation-retry scan is needed.
  Object mutations maintain the index under the same writer lock.
  Startup discards all derived runs and rebuilds from authoritative bytes
  after journal recovery. Index read/write corruption requires recovery.
- The removed push-host allowlist is **not** restored. An obsolete
  `SKRYNIA_PUSH_ALLOWED_HOSTS` environment variable fails startup rather
  than silently relaxing the former deployment configuration.
- Current push registrations and key patches require unpadded base64url
  because the `web-push` encryption implementation rejects padding.
  Legacy persisted records satisfying the old bounded-length validation
  do not block startup. Invalid keys are logged, can be repaired using
  a subscription capability, and cannot generate outbound push sends.
- Journal replay and committed-marker failures remain recovery-latching,
  even when the immediate storage error occurred before an atomic rename.
  An object may already be committed: allowing further writes to that
  key could supersede the bytes checked by the initial journal intent,
  making its outstanding notification unrecoverable. Avoiding this latch
  requires a different durable mutation/notification protocol, not simply
  treating all ENOSPC errors as safe to ignore.
- Release panic-abort remains in place. Arithmetic overflow/underflow
  indicates an internal accounting invariant violation, not a benign
  request failure; no reachable user-input panic was established in this
  review. Static review is not a proof of panic freedom.

## Transient push acknowledgement backoff

A pre-rename failure while persisting push acknowledgments must not become
a one-second resend loop. The durable outbox remains unchanged while
a separate **volatile** subscriber/generation cooldown tracks retries.
The worker skips deliveries until exponential deadlines. Delivery failures
also accrue transient retry counts so endpoint retirement remains possible
after persistence recovers; successfully delivered wake-ups are not counted
as failed endpoints. Only affected subscriptions receive cooldowns, and
successful persistence or subscription changes clear stale cooldowns.

Restart discards volatile cooldowns and may resend a wake-up, consistent
with the existing at-least-once semantics. New notification generations
retain transient delivery-failure backoff if their enqueue commits before
the previous acknowledgment. Tests cover disk errors, increasing delays,
lack of immediate reselection, retirement after recovery, and unchanged
durable state on a clean failure.

The periodic journal replay retains its protective latch for mixed sets
of committed and applied-but-uncommitted intents. A narrower optimization
for verified committed-only replay is possible, but blanket retry would
allow a later overwrite to defeat a still-unmarked intent's recovery.
Journal handling is not changed in this fix.
