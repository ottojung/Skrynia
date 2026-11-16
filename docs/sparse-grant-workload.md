# Storage-index research for sparse grant creation and abundant reads

Status: **target design for implementation; mathematical analysis, not performance measurements**.
This study refines [the complexity model](complexity.md) and compares
[the augmented B+tree candidate](indexed-store-design.md) with
write-optimized alternatives. It does not change Skrynia's current
location-based quotas, raw public object files, or authorization semantics.

## 1. Workload assumption and its limitations

Let W be the number of object mutations, G_new the cumulative number
of newly allocated grants, and R the number of object reads in the same
horizon. Analyze the regime

~~~text
G_new = Theta(sqrt(W)) = Theta(log R)
~~~

Thus R is exponential in G_new. Let H <= G_new count **distinct,
previously unindexed prefix scopes**. Exact-key scopes and repeated
scopes need not trigger the expensive current scan. Let N_i be the
number of live objects at the i-th first-prefix initialization.
N_i <= N_initial + W, but N_i need not grow, and H new grants may
occur early or in bursts. The scaling relation alone says nothing
about N_i, read sizes, conditional requests, retention, latency or
available RAM; these are independent workload parameters.

Current scans incur O(k * sum_i N_i) CPU plus H enumeration traces.
If H = Theta(sqrt W) and grants are interleaved while
N_i = Theta(W) on average, the total scan work is Theta(k W^(3/2)).
This conclusion **does not hold** if H is small, the store remains
small, or grants are created before the objects.

For the same workload, almost any per-read overhead is amplified by R.
An index that benefits only grants must leave ordinary direct GET
unchanged. Strictly identical application read paths can nevertheless
compete for shared filesystem cache and disk bandwidth; record this
as a possible cross-workload effect, not guaranteed read neutrality.

Do not optimize only total asymptotic runtime: with exponential R,
the unchanged read cost can dominate every design and obscure the
meaningful differences in mutation and grant costs.

## 2. New candidate: log-structured prefix-delta index

Keep the present RAM trie for active grant scopes and the authoritative
public store/{key} object files. Maintain a second **bounded RAM trie**
of pending quota *deltas*, plus a family of immutable, sorted, private
disk runs with prefix-sum summaries. The runs are **derived indexes**;
they are not grant ownership, object contents or an alternate source
of authoritative truth.

For an object change from old optional size x to new optional size y,
write a signed delta:

~~~text
delta_objects = int(new exists) - int(old exists)
delta_bytes   = (new size + 512 if present, else 0)
              - (old size + 512 if present, else 0)
~~~

A same-size replacement has zero delta. Updates use old/new sizes
already obtained by the existing mutation path. A memory prefix trie
maintains signed count/byte aggregates for all pending delta keys in
O(k) logical CPU work per update, with O(mk) RAM for at most m
distinct buffered keys. Canceling deltas may be discarded.

When the buffer fills, freeze it into an immutable **sorted run**.
A run stores per-key net deltas plus a compact page directory with
cumulative signed counts/byte sums. Geometrically sized levels are
merged using a binary-counter schedule; merging sums equal keys,
keeps sorted order, and may remove net-zero records. Pages can be
written mostly sequentially.

For a prefix P:

~~~text
current_objects(P) = mem_objects(P) + sum(run_objects_i(P))
current_bytes(P)   = mem_bytes(P)   + sum(run_bytes_i(P))
~~~

Each run computes an exact half-open prefix range sum by two rank/sum
lookups; the memory trie answers the prefix in O(k).
Signed partial aggregates are permitted but the final sum must equal
the nonnegative authoritative live usage. Use checked wide counters
to prevent overflow and underflow.

**Correctness induction:** initially the runs represent the starting
authoritative store. Every committed create/replace/delete contributes
exactly its old-to-new delta once. Freezing and merging move records
between representations while preserving their sum. Therefore prefix
totals remain identical to a full scan, provided all updates are
visible exactly once at query boundaries.

Grant creation is serialized with mutation publication; a reader
never sees a frozen buffer and its replacement run simultaneously.
The active-grant trie is still updated on every object mutation,
and overlapping grants retain precisely their current semantics.
Ordinary GET/HEAD stays filesystem-direct.

## 3. Symbolic complexity of the candidate

Let m be the RAM buffer's maximum number of distinct keys, B the
index block size, f >= 2 the effective run-directory fanout,
and ell = O(log(1+W/m)) the number of runs/levels.
Use fixed-size signed aggregates and O(k)-byte worst-case entries.
The simplest analysis assumes an initially empty store; for
preexisting objects add a complete initial build from N_initial files.

| Operation | CPU / logical work | Candidate index I/O |
| --- | --- | --- |
| Normal GET / HEAD | Exactly current algorithm | No index read or write on read path |
| Object mutation | O(k) extra RAM-trie update | No immediate index random page rewrite; eventual batch merge |
| New prefix grant | O(k + k*ell*log W) | O(ell * log_f(max(2,W))) random index-page reads |
| Repeat prefix / active scope | Existing scope trie lookup | No disk run query |
| Exact-key new scope | O(k), from direct file stat | One filesystem key lookup, not a run query |
| Flush or compaction | Amortized O(k*log(1+W/m)) extra CPU per mutation | O((k/B)*log(1+W/m)) amortized sequential pages per mutation, plus transfer starts |
| Cold rebuild | Full enumeration, sorting and aggregate construction | Explicit external sorting and durable/index I/O; cannot assume zero cost |

The amortized merge bound comes from each delta record being moved
through O(log(1+W/m)) geometrically increasing levels. It assumes
merges are scheduled as specified and no unbounded per-mutation
compaction overhead is hidden. Full rebuild and exceptionally large
flushes have substantially worse **worst-case latency**; bounded
latency requires incremental or background compaction with a
correctness proof.

Run rank/sum queries can use O(log_f n_i) page visits and
O(k log n_i) comparison work on a run of n_i entries, provided
the immutable runs have page-level cumulative directories.
Across ell runs, use the conservative bounds shown above. Actual
physical costs depend on cache state and file layout. Disk space for
retained historical deltas may be O(Wk), which is potentially much
larger than O(Nk) for repeated mutations on a tiny live store.
Compaction and occasional full rebuilding can reduce that cost but
must be charged.

At a declared horizon W, choose m = Theta(sqrt W). Then
G_new*m = Theta(W): even if every one of the Theta(sqrt W)
new grants scanned an unsorted m-key buffer, the **aggregate** RAM
scan CPU would be only O(Wk). The prefix-aggregate RAM trie is
better: it makes each buffer query O(k). This provides a reason
to allocate Theta(sqrt W) RAM keys to a write buffer.

For an online server without knowledge of final W, choose a
power-of-two epoch schedule with m = Theta(sqrt(W_so_far+1))
and account for resizes/rebuilds. This consumes O(sqrt W*k)
RAM in addition to the existing O(G_active*k) scope trie:
it fits in O(G_active*k) **only if** the number of active grants
is itself Theta(sqrt W). Heavy revocation can break that
relationship; use an explicit RAM cap instead of assuming it.

## 4. Comparing designs in this regime

| Design | Mutations: additional index work | First prefix | Additional persistent space |
| --- | --- | --- | --- |
| Current scope-only trie | O(k) RAM changes, no per-object disk index | O(Nk) scan | No object-wide index |
| Augmented disk B+tree | O(log_f N) random index-page changes, page-local CPU | O(log_f N) page reads | O(Nk) |
| Log-structured prefix deltas | O(k) RAM plus amortized sequential compaction | O(log W * log_f W) page reads | O(Wk) worst-case history |
| Disk aggregate prefix trie | O(k) logical path updates; random I/O depends on packing | O(k) logical traversal | O(Nk) logical trie space |
| Existing trie + exact-key shortcut | No extra mutation work | O(Nk) for new prefix; O(k) for exact key | No new index |

These are conditional bounds, not a strict all-resource ranking.
A disk B+tree can favor latency-sensitive grant creation and listing.
A log-structured delta index can favor frequent updates and rare
prefix queries because it batches index writes. An aggregate disk
trie has simpler prefix lookup but potentially many random node
updates. The current scheme wins on some write-only workloads with
small N or few new prefix scopes.

For the growing-store, interleaved-grant regime with
H = Theta(sqrt W), the baseline's total prefix-scan CPU is
Theta(k W^(3/2)), whereas the log-structured design incurs
O(k W log W + k sqrt W log^2 W) index CPU, plus startup and
other application work. Therefore **its CPU complexity improves
asymptotically** under those assumptions, while its persistent disk
and I/O costs can be greater. This is not universal Pareto dominance.

Compare B+tree and delta-runs by symbolic break-even:

~~~text
W * (cost_Btree_update - cost_delta_update)
  >
H * (cost_delta_prefix - cost_Btree_prefix)
  + extra_restart_cost + extra_build_cost
~~~

Both sides include random/sequential I/O and durability costs under
the same positive finite hardware parameters. Disk/RAM budgets are
independent constraints. No arbitrary coefficients or measurements
are introduced to manufacture a winner.

## 5. Crash safety and operational obligations

Keep the public object files and existing mutation journal
authoritative. A safe first implementation **rebuilds all derived
runs from the public store on every restart after journal replay**,
then resumes accepting grants. No cross-file atomic commit of public
data and private index is assumed, and index files are never trusted
during startup merely because they appear present.

At runtime, a committed object change must update the in-memory
delta before releasing the serialization boundary to grant creation
or indexed listing. If publishing the derived delta fails after the
object commit, stop processing and recover rather than return a stale
quota total. For compaction, retain old visible runs while preparing
new ones, and atomically switch which *complete* set contributes to
queries. Never count a source run and its merged replacement together.
Charged space includes overlapping runs, temporary compaction files,
and recovery scratch.

A faster incremental-replay scheme would need a monotonic committed
mutation sequence, a durable high-water mark, replay without duplicate
deltas, and an ordering proof with Skrynia's notification journal.
That is **future research**, not an assumed property of the design.

This candidate does not automatically optimize LIST. To obtain
output-sensitive listing, a separate live-key ordered view or a
proved bounded-dead-key merge iterator is needed. Stale/deleted
keys distributed across runs can defeat output sensitivity; do not
claim it just because each run is sorted.

## 6. Reads are potentially the real objective

The relation G_new = Theta(log R) means that an extra per-read cost
has enormous aggregate weight. Do not pay for consulting a disk
index on ordinary GET. Skrynia currently computes byte-derived
SHA-256 ETags by reading the object even for HEAD and 304 requests.

A **distinct candidate** is to prepare an inode-bound verified digest
(for example, a filesystem extended attribute) on a staged immutable
object **before** its atomic link/rename into the public store. A
reader opening that same inode could obtain its committed digest
without rehashing the whole value. This might make HEAD, 304 and
range GET much cheaper for large objects while preserving the
same observable SHA-256 ETag *under a strict no-out-of-band-byte-
changes assumption*. Normal full-body GET can retain its current
path whenever fetching extra metadata is not beneficial.

This idea is **not proven safe or implemented**. It requires
invariant-preserving pre-publication metadata writes, reliable
inode-bound lookup across concurrent replacement, xattr support,
correct backup/restore behavior, and explicit treatment of
external corruption or modification. Without inspecting bytes,
one cannot verify arbitrary post-publication bit changes. The
current server's byte-derived ETag contract must not silently
be weakened, and a disk-sidecar lookup keyed only by object name
is not safe across concurrent replacement without synchronization.

Use the number R_head304 of metadata/conditional reads and their
object-size distribution, not R alone. A digest-assisted design
wins only where its saved hashing and data-read costs exceed
the per-write metadata and per-request lookup costs.

## 7. Research decision

1. **First:** exact-key grant creation should stat one file, eliminating
   its O(Nk) scan without any index or additional disk state.
2. **Target indexed design for G_new = Theta(sqrt W):**
   log-structured, signed prefix-delta runs with a bounded RAM
   aggregate buffer; its random-index-update cost is lower in the
   analytical model, at the price of more complex merges and queries.
3. **Keep the augmented B+tree as a competing design** for
   listing-heavy or latency-sensitive grant workloads; it is not
   disproved by the sqrt(W) relation.
4. **Research read digests separately** if HEAD/304/range requests
   constitute a meaningful fraction of the very large R.
5. Before proposing runtime implementation, specify crash/recovery
   proofs, the RAM cap, compaction worst-case latency and disk
   amplification, and testable full-scan oracle invariants.

No benchmarks or timing measurements are required or performed by
this mathematical comparison.

## References

- O'Neil et al. (1996), [The log-structured merge-tree](https://doi.org/10.1007/s002360050048): deferred and batched index maintenance.
- Bender et al. (2007), [Cache-oblivious streaming B-trees](https://dash.harvard.edu/entities/publication/73120378-dc8c-6bd4-e053-0100007fdf3b): streaming/merge indexes with distinct search and insertion I/O costs.
- Brodal, Rysgaard, Svenning (2025), [Buffered partially-persistent external-memory search trees](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2025.82): stronger buffered search-tree trade-offs; their exact bounds do not automatically apply to our signed aggregate runs.
