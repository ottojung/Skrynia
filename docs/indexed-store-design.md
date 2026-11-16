# Indexed object statistics: design study

Status: **proposed design, not implemented**. The current server still uses
a scope-only in-memory quota trie and scans objects for new prefix scopes
and scoped listing. This study preserves [current API](api-spec.md),
[location-based grant quotas](intent-location-authority.md), flat public
object files, and the single authoritative writer. It does **not** adopt
object ownership or change client-visible listing/cursor semantics.

For comparing architectures use [the filesystem-aware complexity
model](complexity.md). No benchmarks, timings, or numerical hardware-cost
assumptions are used in this design study.

## 1. Goal and limits of domination

The existing implementation excels at low-RAM reads/writes, but initializing
a previously unseen prefix scope scans N object entries. Listing also
scans N entries even for a small result.

Our candidate should:

- Replace O(Nk) *CPU* new-prefix initialization with O(k log N)
  key-comparison work and O(log_f N) index-page visits.
- Replace O(Nk(1+log(L+1))) CPU per listing page with an ordered seek
  plus work for returned keys.
- Preserve O(Gk) quota-scope RAM plus a **bounded** index cache.
- Preserve direct file GET/HEAD, content-hash CAS, grants and recovery.
- Charge all disk-page updates, cold rebuilds, and index disk space.

**An impossibility for strict unconditional Pareto dominance:** maintaining
an extra exact index that records information not kept by the current
system consumes additional persistent bytes and mutation effort on at
least some workloads. An all-write workload with no new scopes or listing
benefit is a counterexample. Thus this proposal aims to improve the
**Pareto frontier** and dominate for explicit workload/hardware regions;
it must not claim all-resource dominance for all workloads.

## 2. Concrete architecture

Keep the in-memory trie of **active grant scopes**. Add a derived,
bounded-cache, **disk-backed augmented B+tree** over *every live object
key*, with each leaf storing (key, payload byte length). Internal edges
store a subtree object count and a subtree payload-byte sum, together
with per-node cumulative summaries to support rank/sum queries without
linearly scanning all child edges. This shifts some work to index updates.

~~~text
               root: (count, sum of lengths)
                 /                    \
       internal range A          internal range B
        /         \                /         \
    ordered leaf  ordered leaf  ordered leaf  ordered leaf
    (key,length)  (key,length)  (key,length)  (key,length)

Public store/{key} remains the authoritative opaque byte file.
Private derived index is outside store/ and never publicly served.
~~~

The index does **not** store object payloads, grant ownership, or
application data. The root grant already indexes the `*` matcher, so
the important new-scope problem concerns previously unseen prefixes. Use a configurable memory-bounded page cache; do not
silently pin the entire tree in RAM. It must work with a cold cache.

Operations:

1. **Exact-key grant scope:** inspect the object by its exact path (or
   perform a point lookup in the index). The empty scope has zero
   objects. No full directory scan is needed, even without an index.
2. **Prefix scope:** turn the prefix into a lexicographic half-open
   interval [prefix, successor(prefix)), using an unbounded upper
   endpoint when no successor exists. Two cumulative rank/sum
   searches yield exact object count and byte sum. The root aggregate
   answers the wildcard scope directly. Add 512 bytes per object,
   exactly as the existing grant accounting requires.
3. **Scoped LIST:** seek the first matching key after the optional
   exclusive cursor, then iterate ordered leaf records until the
   limit or prefix boundary. Maintain current weakly consistent
   pagination semantics: no invented cross-page snapshot guarantee.
4. **POST/PUT/DELETE:** update the leaf and aggregate ancestors
   using old/new lengths, alongside the existing in-memory active
   scope counters. Quota approval still checks only the acting grant
   and its ancestors. Unrelated overlapping grants still do not veto
   writes. Reads remain served from public files, not this index.
5. **Revocation:** do not transfer object ownership (there is none).
   The existing retained-scope count approach remains applicable.

A B+tree is preferred over a naive disk trie because bounded-height
page-level traversals are easier to cost; a compressed/page-packed
aggregate trie is an alternative subject to the same I/O analysis.

## 3. Consistency and failure model

The index is **derived state**, not an additional source of truth.
Public object files and the existing mutation journal remain
authoritative. A minimum-correctness version deliberately rebuilds the
index on every startup **after journal recovery**, using actual stored
object names and lengths. Old derived index pages are never trusted at
startup. This costs a full enumeration and index construction but
avoids requiring an atomic cross-file commit of data and index metadata.

Inside a running writer:

- Hold the existing serialization boundary when checking quotas,
  committing an object, and making the new index state visible.
- Update the index only after the authoritative mutation reaches the
  existing required commit point; make it visible before releasing the
  boundary to grant creation or LIST. If the public change succeeds
  but index update fails, latch recovery-required rather than serve
  incorrect quota or listing results.
- A crash before or during the derived update is safe only because
  startup replays the mutation journal and **reconstructs** the index.
- Never delete or alter journal intent handling simply because an
  index can be rebuilt. If outbox persistence fails ambiguously,
  retain the current protective recovery latch.
- The index lives under a private data path; its pages may be buffered,
  but their write traffic, transient buffers, and storage still count.
  A restart must discard untrusted/stale index state rather than
  deciding it is correct from timestamps or a missing dirty flag.
- Static GET/HEAD continue reading the same public files directly.

A faster persistent checkpoint and incremental journal replay may be
considered separately, but then index consistency, crash ordering,
recovery and checkpoint costs need their own proof.

## 4. Conditional complexity bounds

Let f>=2 be effective B+tree page fanout; q = ceil(log_f(max(2,N))).
Let t=O(L) be listed keys, and let K_out be the bytes in their names.
Assume bounded page size, valid balanced-tree invariants, and no
asymptotically unbounded compaction on an ordinary mutation.

| Operation | Current CPU (ignoring FS) | Indexed candidate CPU (ignoring FS) | Candidate extra disk I/O |
| --- | --- | --- | --- |
| Exact-key new scope | O(Nk) current scan | O(k), using filesystem stat | One point lookup instead of N-entry scan |
| New prefix scope | O(Nk) per scan | O(kq log f) | O(q) random page reads |
| Wildcard `*` scope | Already indexed via root grant | Already indexed via root grant | No new query needed |
| LIST page | O(Nk(1+log(L+1))) | O(kq log f+K_out+L) | Seek O(q) pages and sequential leaf pages |
| GET/HEAD | O(k+v) | Same | None; remains filesystem-direct |
| POST/PUT/DELETE | O(k+b+h+s), excluding conditional hash/outbox | O(k+b+h+s+(k log f+f)q), same exclusions | O(q) index-page reads/writes, plus any split/merge |
| Existing grant scope | O(k+h) | Same | None needed for quota total |
| Restart/recovery | O(Nk) object scan plus private replay | Object scan plus index rebuild | Additional O(Nk) derived-index bytes written |
| Subtree revocation | Cost depends on grants/subscriptions | Same baseline algorithm | No index-object transfer |

For B+tree insertion/deletion, routing at a node takes O(k log f)
worst-case CPU with full-key comparisons. Maintaining cumulative
summaries and shifting entries takes O(f) arithmetic operations per
modified node, with additional O(kf) CPU if uncompressed full-length
keys are moved. Consequently, the table's update estimate assumes
fixed-size node metadata and page-local key movement costs are charged
separately; O((k log f+fk)q) is a conservative alternative. The page
cache still must account for all touched or dirtied pages.
The I/O trace must count dirty-page flushes, internal splits,
write amplification, and page-cache misses. The candidate improves
**new-prefix initialization** unambiguously in algorithmic search
work, not necessarily in wall time for every device/workload.

The steady private index uses O(Nk) persistent bytes (up to
page-layout overhead). Peak build/rebuild space may be higher.
Index-cache memory is bounded by a declared C_cache; aggregate server
RAM includes O(Gk + C_cache + Lk + b) plus credentials, push state,
network buffers and other request-specific memory. An incremental
bulk builder may use external-memory sorting; its read/write CPU,
I/O, scratch space and restart cost must be included, not assumed free.

## 5. Symbolic break-even analysis

For a workload window, let:

- H be newly encountered **prefix** scopes (not scope reuses);
- J be listing pages served;
- W be object mutations;
- R be index rebuilds/restarts.

Under a fixed hardware-cost vector theta, write:
- S = existing per-prefix scan cost;
- Qp = candidate two-rank-query cost;
- E = existing per-list-page scan cost;
- Li = candidate seek-and-iterate cost;
- U = *incremental* index maintenance per mutation;
- D = index rebuild cost per restart;
- A = initial index construction cost.

Common baseline object/journal/outbox costs cancel when unchanged.
The candidate improves the aggregated elapsed-cost model exactly in the
parameter region where

~~~text
H*(S-Qp) + J*(E-Li) > W*U + R*D + A
~~~

with RAM and disk-space budgets separately satisfied. No numerical
values are assumed. If H=J=0 and W>0, this fails: the index cannot
be a universal winner. If N grows, H or J is non-negligible, and
index operations remain sublinear, the index can win for a broad
region of positive finite hardware costs. State the region rather
than choosing convenient weights.

This inequality also applies to a workload-guided **optional index**:
start with the existing algorithm, count full scans and their actual
costs, and enable derived indexing only when a declared workload
policy justifies the rebuild and ongoing maintenance. The transition
itself costs A and needs a consistency barrier. An adaptive policy
does not establish universal Pareto dominance, either.

## 6. Recommended development order (without measurements)

1. Implement **exact-key scope initialization by a single stat**.
   This is an independent improvement requiring no object-wide index,
   no additional persistent state and no changed grant semantics.
2. Specify and prove the on-disk index invariants: one record per live
   key, correct length, correct subtree count/byte sums, strict sorted
   order, and atomic visibility relative to authorized mutations.
3. Implement a derived-index **rebuild-from-store** and failure latch
   before using indexed quota calculations. Test restart, interrupted
   PUT, missing/truncated index and overlap cases using a full-scan
   reference oracle; do not rely on timing assertions.
4. Add prefix aggregate reads and ordered listing. Preserve scoped
   listing permissions and the current exclusive-cursor behavior.
5. Account symbolically for every filesystem operation, cache page,
   split, copy, scratch byte, and synchronization, including push
   outbox amplification that this candidate does not solve.
6. Consider replacing the current single-file push outbox with an
   incremental durable structure **as a separate proposal**. It can
   improve Q-dependent rewrites, but it carries a distinct recovery
   proof and must not be bundled into an unreviewable storage rewrite.

### Design verdict

This index is a **candidate frontier improvement**, not an unconditional
resource-Pareto dominator. It preserves Skrynia's simplicity at the
public protocol and authority layers, offers logarithmic prefix
accounting and output-sensitive listing, and does not require
ownership-based quotas. It pays for those gains in persistent space,
write amplification and cold-start rebuilds. The symbolic inequality
makes the trade-off reviewable without measurements.
