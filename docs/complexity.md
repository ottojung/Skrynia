# Skrynia complexity model

Status: normative **as a method of analyzing complexity**, not as a claim that
the current server meets an unimplemented performance target.

This document separates logical work, filesystem work, volatile memory,
persistent space, durability, and concurrency. Its goal is to let us compare
implementations without assuming an unrealistically fast filesystem or
forbidding useful disk-backed data structures. Observable semantics are those
in [the HTTP API](api-spec.md) and [grant specification](intent-location-authority.md).

## 1. Rules of analysis

1. Describe the **actual algorithm** and its filesystem-operation trace.
   Charge for indexes, metadata, staging, journals, and notifications too.
2. State **CPU work**, **peak application RAM**, **live and peak disk space**,
   **filesystem I/O**, and **waiting/network costs** separately. An asymptotic
   CPU bound is not a latency guarantee.
3. Distinguish a **worst-case bound**, an **expected bound** (with stated
   distribution/hash assumptions), an **amortized bound** (with a stated
   sequence), and an **observed measurement**. None substitutes for another.
4. State what the filesystem guarantees. Without a specified filesystem
   implementation there is **no universal useful upper bound** for disk latency.
5. Allow auxiliary **RAM and disk indexes**. Include their construction,
   updates, restart/recovery, persistent size, and consistency obligations.
6. A failed operation and an interrupted operation also consume resources.
   Retried work, contention, lock waiting, and crash recovery must not be
   silently assigned zero cost.
7. Normal application semantics remain unchanged. In particular, object
   quota accounting is by **matching grant location**, not by object owner.

## 2. Parameters and units

| Symbol | Meaning |
| --- | --- |
| N | Number of live stored objects |
| G | Number of active grants |
| U | Number of user credentials |
| S | Number of active push subscriptions |
| Q | Number of pending notification records |
| k | Maximum key/scope length in bytes; actual key length may be smaller |
| h | Acting grant ancestry depth |
| b | New request payload bytes; b_old = old object length when inspected |
| v | Full stored object length read or hashed |
| r | Response body bytes (possibly r < v) |
| s | Number of matching push subscriptions for a mutation |
| L | Listing response limit (1-1000); c = number of candidate keys inspected |
| C | Number of grants revoked in a subtree |
| j | Durable journal/intent bytes processed by an operation |
| p | Serialized byte size of the pending outbox, not just Q |
| B | Filesystem/page transfer unit for an optional external-memory model |
| M | Available RAM for an optional external-memory model |

Lengths and counts are nonnegative; use max(1,N) inside logarithms.
The symbol b **does not** hide reading the previous object: conditional
mutation has an additional b_old cost. A range GET can return r bytes while
reading and hashing all v bytes. Report actual bytes, not only page counts.

## 3. Filesystem-cost interface

Define cost functions for the filesystem under a declared platform,
cache state, directory layout, and concurrency regime:

| Function | Operation being charged |
| --- | --- |
| FSL(N,k) | Lookup/stat a named entry of length at most k |
| FSR(b,e) | Read b bytes spread across e extents/requests |
| FSW(b,e) | Write b bytes spread across e extents/requests |
| FSM(N,k) | Create, link, rename, or unlink a directory entry |
| FSE(N,k) | Enumerate N entries and their name/length metadata |
| FSD(f,d) | Make f files and d directories durable (including synchronization barriers) |

These are **declared cost envelopes** for each operation, not claims that
actual filesystems share the same running time. The functions may depend on
hardware, caching, path topology, filesystem size, fragmentation, and load.
For a particular workload, distinguish cold-cache and warm-cache envelopes.
FSL and FSM may also depend on directory sizes; N is a conservative scale
parameter, not proof of logarithmic or constant-time directory lookup.

Reading or writing b bytes through the API entails Omega(b) bytes of
logical data movement, and examining arbitrary k-byte keys entails Omega(k)
work. **Lower bounds alone do not yield upper bounds.** In particular,
FSL = Omega(k) says nothing about whether lookup is fast.

Let tau be the sequence of actual filesystem operations, including
index and durability operations. Define FS(tau) as the sum of their
individual charged costs. For a sequential execution, an honest upper-cost
expression has the form

```text
T <= O(server CPU work) + FS(tau) + network + scheduling/lock waiting
```

Use a separate bound for peak RAM. When I/O and computation overlap,
the sum remains a conservative accounting expression rather than an exact
elapsed-time prediction. A filesystem call may itself perform CPU work;
do not add that internal CPU a second time.

**No hidden free operations:** even a page-cache hit, write buffering,
directory rename, fsync, or rebuilding an index must appear in tau.
Batching may change tau and reduce its total cost.

### Optional block-I/O specialization

For algorithm comparisons, a page/block contains B bytes, and the index
may retain at most M bytes of working state in RAM. Report:

- Random and sequential page reads/writes separately.
- CPU comparisons, hashing, and serialization independently.
- Resident memory and disk pages, including temporary copy-on-write pages.
- Synchronization/barrier count and write amplification.

Under explicit balanced-tree assumptions, a lookup can cost
O(log_f N) page visits, where f is the actual node fanout, and
O(k log_f N) character work for full-length key comparisons.
Only if node layout supports fanout Theta(B/k) (and k fits the page)
may one replace f with Theta(B/k). Writing O(log_f N) index
nodes is not equivalent to O(1) durable writes.

An augmented ordered B-tree can compute prefix object counts and byte
sums using two rank/sum queries with O(log_f N) index-page visits.
A disk-backed aggregate trie can follow O(k) logical edges, but its page
visits and seeks depend on layout; O(k) logical steps does **not**
imply O(k) physical time. Both need index maintenance on writes.

## 4. Logical algorithmic costs in the current Rust server

The following are code-path **work estimates**, not measured latencies.
All lines also incur FS(tau) for their actual operation traces. Shared
authentication is expected O(1) hashed-token lookup after processing the
token; grant ancestry checks add O(h). Network transfer, scheduling, and
contention are excluded from the CPU column.

| Operation | CPU / application work | Additional filesystem trace |
| --- | --- | --- |
| GET / HEAD object | O(k + v) for validation, streaming SHA-256 / ETag | Lookup and read v bytes; response transfers r bytes |
| Absent-object GET | O(k) | Lookup |
| POST object | O(k + b + h + s) plus outbox work below | Two existence/metadata checks, stage b, journal, create/link, sync |
| PUT object | O(k + b + h + s + b_old if conditional) plus outbox work | Checks, stage b, journal, atomic rename, sync; read b_old if conditional |
| DELETE object | O(k + h + s + b_old if conditional) plus outbox work | Checks, journal, unlink, sync; read b_old if conditional |
| LIST keys | O(Nk + N log(L+1)) current heap selection | Enumerate N names; inspect entry metadata |
| Create grant, existing indexed scope | O(k + h) plus record serialization | Write and sync one grant record |
| Create grant, new scope | O(Nk + k + h) per scan attempt | Enumerate N names and sizes, plus grant record |
| Inspect grant / issue or revoke user token | Hash/lookup plus per-record serialization; token mutation can depend on users in that grant | Read/write private grant record as required |
| Revoke subtree | Depends on C, G, S and affected outbox records; cannot claim O(1) | Revocation intent, remove grant/subscription records, rebuild indexes |
| Register/update/delete push subscription | Scope/auth work plus affected record/outbox work | Subscription record mutation; possible durable outbox rewrite |
| Notification delivery / acknowledgment | Selection plus affected Q-record processing | Persist pending/retry/ack state; external HTTPS deliveries |
| Startup / recovery | Grant/subscription loading, quota index rebuild, intent replay | Enumerate objects and private records; replay pending journals |
| Health / version | O(1) local state | No object-directory scan |

For LIST, c may be smaller than N only with a different indexing
strategy; the current directory scan visits all N. The result is
not a snapshot under concurrent modifications.

**Current outbox amplification:** when a mutation produces at least
one pending notification, the single durable outbox is sorted and
rewritten. For Q pending entries and p serialized bytes, include
O(Q log Q + p) CPU, O(p) buffering/serialization space where needed,
FSW(p,e), metadata replacement, and durability costs. A delivery
batch acknowledgment can likewise rewrite the outbox. The total is
not generally O(k+b+h+s). This is an implementation cost, not a
required property of Skrynia's API.

**New-scope concurrency:** initialization attempts at most two
lock-free full scans; when revisions change it falls back to a scan
under the writer lock. Worst-case total scan work is O(Nk) per
attempt, with a constant number of attempts, plus lock waiting.
Grant creation does not assume a preexisting per-object index.

**Revocation and recovery:** the current server preserves object
contents on grant revocation and rebuilds grant-scope counters from
its retained scope information. Actual cleanup scales with affected
private records; there is no basis for advertising constant-time
revocation or restart. Recovery work scales with the outstanding
journal/outbox state, which must be named in any performance claim.

## 5. Explicit filesystem traces and resource bounds

Let X be total bytes in live client objects and D_meta be durable private
metadata bytes. Let I_disk be bytes of any optional persistent index.

| Path | Necessary trace components | Peak application RAM / disk observations |
| --- | --- | --- |
| GET / HEAD | FSL(N,k) + FSR(v,e) | Streaming buffer O(1) in object size, excluding network/framework buffers |
| Object POST | FSL + FSW(b,e) + FSM + FSW(j,e) + FSD, plus notification/index costs | Current whole-body upload buffer O(b); stage/journal require extra disk |
| Object PUT | POST-style writes + conditional FSR(b_old,e), if requested | Old and staged new versions may coexist before rename |
| Object DELETE | FSL + FSM + journal/durability; optional FSR(b_old,e) | No replacement payload, but private journal still consumes space |
| LIST | FSE(N,k), plus metadata reads as needed | O(Lk) candidate memory |
| New grant scope | FSE(N,k), then private record write and durability | Existing grant-scope trie RAM O(Gk); enumeration not a full object index |
| Push outbox rewrite | FSW(p,e) + FSM + FSD | O(p) transient serialization/storage possible; may temporarily hold old and new files |
| Startup | FSE(N,k) + private-state enumeration/read + recovery traces | All live grants, credentials, subscriptions, outbox in memory as implemented |

FSL, FSR, FSW, FSM, FSE and FSD are charged **per invocation**,
not once per HTTP request. This table lists components, not complete
counts of every rename/sync in a particular code revision. A detailed
claim must extract the exact trace, including retries and failed writes.

Current steady-state grant/quota object-index RAM is O(Gk), independently
of N, **not total server RAM**. Credential state, push state, connection
buffers, upload bodies, listing results, and pending outbox serialization
add their own costs. Current disk occupancy is at least X + D_meta;
peak occupancy includes staged data, journal, in-flight outbox replacement,
filesystem metadata, and any indexes. A disk-backed index is allowed,
but its I_disk, RAM cache, and reconstruction costs must be reported.

## 6. Performance goals and permitted design space

The following are **evaluation goals for future designs**, not achieved
guarantees. The complexity model does not mandate location-based quota
indexing, an object-wide index, or a particular filesystem library;
the current API and quota semantics still apply unless separately changed.

| Concern | Evaluation objective |
| --- | --- |
| Direct reads | O(k+v) server CPU and a bounded, explicitly counted filesystem trace; memory independent of v while streaming |
| Object mutation | Work proportional to key, payload, ancestor depth, and affected subscribers, with every durable/index operation charged |
| New-scope grant | Investigate sublinear-in-N lookup using disk indexes while preserving exact counts and sums |
| Scoped listing | Investigate output-sensitive traversal rather than a full scan; state cursor consistency semantics |
| Notifications | Avoid rewriting all Q pending entries for O(s) changed subscribers |
| Startup/recovery | Account for rebuilding every derived index and replaying all durable intents |
| Memory and disk | Report both independently; no blanket ban on O(N) disk indexes or temporary durable files |

For a **candidate** augmented ordered tree, exact prefix-count and
prefix-byte-sum queries can use O(log_f N) index-page visits and
O(k log_f N) key-comparison work; mutations must pay for updating
affected aggregate pages and making those updates recoverable. This
recovers the proposed O(k log N) CPU bound (when f is bounded below by
a constant) without claiming that disk access takes O(k) time.
A fully aggregated key trie can give O(k) logical traversal but not
necessarily O(k) disk page cost. The existing grant-scope-only trie
avoids object-proportional RAM/index storage at the price of
O(Nk) initialization of a previously unindexed prefix.

Exact-key new scopes are a special case: they need only one keyed
object lookup/stat, not a full prefix scan. This is an optimization
opportunity, **not a claim that the current scan implementation does it**.

## 7. Evidence and acceptance checklist

Every proposed optimization or benchmark must report:

1. Exact commit, hardware, filesystem, mount/storage type, data size,
   object-size and key distributions, grant depth, N/G/S/Q, cache
   conditions, and concurrency level.
2. Correctness of location-scoped quota totals under overlapping grants,
   concurrent mutation, recovery, and crash/failure injection.
3. CPU work, peak RAM, steady/peak disk bytes, logical I/O, physical
   block I/O when measured, fsync/rename counts, and write amplification.
4. Wall time and throughput distributions (not only averages), including
   p50/p95/p99, contention, slow or failed storage, and outbox pressure.
5. Index creation, update, deletion, and restart/rebuild cost. Any
   persistent index must be either durably consistent with objects or
   safely reconstructible from authoritative state.
6. A clear separation between **measured implementation behavior**,
   **proved asymptotic bounds under assumptions**, and **unimplemented
   candidate designs**.

See [verification](verification.md) for the current test and performance
evidence. This model does not weaken the production durability, exclusive
writer, object-layout, authority, or recovery requirements.
