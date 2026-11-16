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
sums using two rank/sum queries with O(log_f N) index-page visits,
assuming page nodes include cumulative child summaries. With binary
search among f separator keys, worst-case comparison CPU is
O(k log(f) log_f N) = O(k log N); maintaining the cumulative summaries
adds page-local work on updates.
A disk-backed aggregate trie can follow O(k) logical edges, but its page
visits and seeks depend on layout; O(k) logical steps does **not**
imply O(k) physical time. Both need index maintenance on writes.

## 3A. Optimization without a privileged resource

Filesystem operations are neither free nor infinitely expensive. Treat an
implementation as a **resource vector**, not as one Big-O expression:

~~~text
V = (CPU operations, random read requests, random write requests,
     sequential transfer starts, bytes read, bytes written,
     durability barriers, peak RAM, steady disk, peak disk,
     startup/recovery work)
~~~

Counts refer to a **specified workload**. Include index maintenance on
ordinary writes, not only acceleration of the query that motivated the index.
Report cache hits separately from physical I/O; an assumed always-hot index
must disclose its RAM/page-cache footprint.

### Pareto order and its limits

For the same workload and semantics, design A Pareto-dominates B only when
every component of V(A) is <= the corresponding component of V(B), and
at least one is strictly smaller. This is a strong claim: an auxiliary disk
index **cannot** strictly Pareto-dominate a design with no such index
when persistent disk bytes and index-update writes are counted, unless the
index replaces more costly existing state. Never call an improvement to
one operation or one weighted objective universal dominance.

A design can nevertheless (i) dominate on a **projected subset** of costs,
(ii) dominate for a **specified workload and hardware region** after
evaluating all resource costs, or (iii) improve the Pareto frontier by
making a previously expensive operation asymptotically cheaper. State
which kind of result has actually been proved.

### Parametric hardware scenarios

For a trace, let nr/nw count random read/write transfer starts; ns count
sequential transfer starts; dr/dw count transferred bytes; nf count
durability barriers; and c count application CPU work. Consider the
illustrative, **not calibrated**, family:

~~~text
T(theta) = rho*c + ar*nr + aw*nw + as*ns
           + br*dr + bw*dw + delta*nf
theta = (rho, ar, aw, as, br, bw, delta, B, M)
~~~

All weights are positive and finite. State any ordering assumptions
between weights instead of silently assigning them. This formula is a
decision model, not a filesystem worst-case latency guarantee, and may
overcount when work overlaps. Disk occupancy and peak RAM remain
**constraints or separate objectives**, not omitted because this
particular scalarization does not price them.

Analyze at least three symbolic regimes: (1) expensive random I/O,
(2) inexpensive indexed random I/O with bounded cache, and
(3) limited RAM with cold-cache restarts. Do not invent numeric
hardware coefficients to make a preferred design win.

A workload is a vector of operation **counts and distributions**:
reads, creations, replacements, deletions, new/existing grant scopes,
lists, revocations, subscription changes, push deliveries, and restarts,
together with N/G/S/Q, key lengths, object sizes and grant depth.
An optimization is justified by an explicit break-even inequality or
a dominance proof over a declared workload region. The adversarial
all-writes and all-new-scopes workloads must both be considered.

When no measurements are requested, evaluate symbolically: prove
correctness, derive operation traces and resource bounds, identify
break-even regions, and leave empirical coefficients unknown.
An agent **must not discard an indexed design merely because a disk
cost function has an unbounded worst case**, nor pretend an index wins
if its persistent space and update costs exceed the chosen budgets.
Do not select arbitrary weights, and do not declare a universal winner
from an incomplete cost vector.

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
| POST object | O(k + b + h + s + k log(m)) before compaction and outbox work | Checks, stage b, journal, link/sync, possible run flush/merge |
| PUT object | O(k + b + h + s + k log(m) + b_old if conditional) before compaction/outbox | Checks, stage b, journal, rename/sync, possible index merge; read b_old if conditional |
| DELETE object | O(k + h + s + k log(m) + b_old if conditional) before compaction/outbox | Checks, journal, unlink/sync, possible index merge; read b_old if conditional |
| LIST keys | O(Nk(1 + log(L+1))) current heap selection | Enumerate N names; inspect entry metadata |
| Create grant, existing indexed scope | O(k + h) plus record serialization | Write and sync one grant record |
| Create grant, new exact scope | O(k+h) | Point stat/open and private grant record write |
| Create grant, new prefix scope | O(k+h+k*ell*log(max(2,W))) | Read sorted signed-delta runs with binary seeks; write grant record |
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

**New-scope consistency:** first-prefix queries now run under the writer
lock against a derived signed-delta index: the bounded RAM trie plus
immutable sorted runs. Each run is binary-searched for the prefix range.
The previous optimistic directory-scan retry path has been removed.
Index flushes and compactions are charged on mutations, not grants.

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
| New prefix grant | Run index lookup/seeks and private grant write + durability | RAM for active grant scopes O(Gk) plus bounded delta buffer and run metadata |
| Push outbox rewrite | FSW(p,e) + FSM + FSD | O(p) transient serialization/storage possible; may temporarily hold old and new files |
| Startup | FSE(N,k) + private-state enumeration/read + recovery traces | All live grants, credentials, subscriptions, outbox in memory as implemented |

FSL, FSR, FSW, FSM, FSE and FSD are charged **per invocation**,
not once per HTTP request. This table lists components, not complete
counts of every rename/sync in a particular code revision. A detailed
claim must extract the exact trace, including retries and failed writes.

Current steady-state grant/active-scope RAM is O(Gk), plus a bounded
signed-delta buffer of O(mk) and O(ell) run metadata; m <= 4096
by default. This is **not total server RAM**. Credential state, push state, connection
buffers, upload bodies, listing results, and pending outbox serialization
add their own costs. Current disk occupancy is at least X + D_meta + I_disk;
peak occupancy includes staged data, journal, in-flight outbox replacement,
filesystem metadata, and any indexes. A disk-backed index is allowed,
but its I_disk, RAM cache, and reconstruction costs must be reported.

### Representative parametric bounds

The filesystem terms below refer to the **complete actual trace**, not a
single filesystem call. Let tau_get, tau_list, tau_grant, and tau_mut
denote the corresponding traces. Write chi_cond = 1 only for a conditional
mutation, and chi_push = 1 only when durable push state changes.

~~~text
GET:       O(k + v) + FS(tau_get)       [tau_get includes FSL + FSR(v,e)]
LIST:      O(Nk(1 + log(L+1))) + FS(tau_list)
NEW GRANT: O(k + h + k*ell*log(max(2,W))) + FS(tau_grant) [new prefix]
POST:      O(k + b + h + s + k*log(m) + index_compaction
             + chi_push*(Q log Q + p)) + FS(tau_mut)
PUT:       O(k + b + h + s + k*log(m) + index_compaction + chi_cond*b_old
             + chi_push*(Q log Q + p)) + FS(tau_mut)
DELETE:    O(k + h + s + k*log(m) + index_compaction + chi_cond*b_old
             + chi_push*(Q log Q + p)) + FS(tau_mut)
~~~

For an already indexed scope the additional quota lookup is O(k).
For a new exact-key scope it is one object lookup. For a new prefix
there are ell <= O(log(1+W)) immutable disk runs, each supporting
O(log(max(2,W))) fixed-width binary seeks, plus O(k) RAM-trie lookup.
Here W counts index changes since the most recent rebuild.
The variable `index_compaction` represents actual CPU cost of any
flush/merge triggered by the mutation, which can be linear in the
number of indexed deltas in the worst case; it is only amortized
O(k log(1+W)) across a completed merge schedule. This does not imply
bounded per-mutation latency.
The grant trace includes index page reads and the private record write. For a mutation,
tau_mut includes any old-object hash read, staged payload write, intent
persistence, public namespace change, synchronization, optional
durable outbox rewrite, and derived-run flush/compaction I/O. These are conditional upper-cost expressions
for the present algorithm, not universally valid wall-clock bounds.

Token issuance/revocation additionally serializes the users belonging
to the affected grant: for u_g such users, budget O(u_g + h) CPU plus
the trace of rewriting its O(u_g)-sized private record. Revoking a
subtree currently walks candidates' ancestry (up to O(Gh)),
searches affected subscriptions against the revoked-ID list
(up to O(SC)), and rebuilds live grant counters (up to O(Gk + (G+S)h)),
besides private-record and outbox I/O. These intentionally conservative
bounds are not an API requirement. A faster revocation index must charge
for maintaining that index on other operations.

## 6. Performance goals and permitted design space

The following are **evaluation goals for future designs**, not achieved
guarantees. The complexity model does not mandate location-based quota
indexing, an object-wide index, or a particular filesystem library;
the current API and quota semantics still apply unless separately changed.

| Concern | Evaluation objective |
| --- | --- |
| Direct reads | O(k+v) server CPU and a bounded, explicitly counted filesystem trace; memory independent of v while streaming |
| Object mutation | Work proportional to key, payload, ancestor depth, and affected subscribers, with every durable/index operation charged |
| New-scope grant | Preserve the implemented sublinear indexed-prefix query and account for cold rebuilds |
| Scoped listing | Investigate output-sensitive traversal rather than a full scan; state cursor consistency semantics |
| Notifications | Avoid rewriting all Q pending entries for O(s) changed subscribers |
| Startup/recovery | Account for rebuilding every derived index and replaying all durable intents |
| Memory and disk | Report both independently; no blanket ban on O(N) disk indexes or temporary durable files |

For a **candidate** augmented ordered tree, exact prefix-count and
prefix-byte-sum queries can use O(log_f N) index-page visits and
O(k log N) comparison work when routing within pages is logarithmic
in f; mutations must pay for updating affected aggregate pages and
making those updates recoverable. This recovers the proposed
O(k log N) CPU query bound without claiming that disk access takes
O(k) time.
A fully aggregated key trie can give O(k) logical traversal but not
necessarily O(k) disk page cost. The earlier grant-scope-only design avoided disk indexes but incurred
O(Nk) initialization of a previously unseen prefix. The current
log-structured index retains bounded RAM and enables new prefix queries
through immutable runs. Exact-key scopes use a direct filesystem lookup.

The concrete [disk-backed aggregate-index design](indexed-store-design.md)
applies this model to new-scope grant initialization and scoped listing;
its comparisons are symbolic, with no measurements.
The [sparse-grant workload research](sparse-grant-workload.md) examines
G_new = Theta(sqrt W) = Theta(log R) and an amortized, write-optimized
signed-delta index. It explicitly distinguishes conditional CPU improvement
from universal resource dominance.

## 7. Analytical acceptance checklist

**An unimplemented design can be evaluated without any measurements.**
A complete analytical proposal must provide:

1. Correctness and recovery invariants, including overlapping grants,
   concurrent mutations, interrupted writes and fail-closed recovery.
2. Complexity vectors for **every affected operation**, including
   the operation(s) made slower to accelerate another one.
3. CPU work, peak RAM, steady/peak persistent bytes, random/sequential
   I/O, byte transfers, sync/barrier counts, and rebuild costs.
4. Explicit worst-case, expected or amortized qualifiers and assumptions;
   for an indexed structure, include maintenance and page splits.
5. An exact dominance claim, or a **symbolic break-even region** for
   hardware parameters and workload frequencies. Check all-reads,
   all-writes, rare-grant, grant-heavy, listing-heavy, and restart-heavy
   counterexamples before declaring a design better.
6. A strict separation between implemented behavior, proved
   conditional bounds, and proposed (unimplemented) designs.

**When empirical evaluation is requested**, additionally record the
commit, hardware/filesystem/storage, key and object-size distributions,
N/G/S/Q, caching and concurrency; measure CPU, memory, disk I/O,
fsync/rename counts, p50/p95/p99 latency, throughput and failure
behavior. Measurements are **not a prerequisite for the present
mathematical design comparison**.

See [verification](verification.md) for the current test and performance
evidence. This model does not weaken the production durability, exclusive
writer, object-layout, authority, or recovery requirements.
