# Client-side protocols on Skrynia

**Purpose.** This document describes *applications built from Skrynia's existing
primitives*, not additional server features. The relevant distinction is between
what the server enforces, what cooperating clients can coordinate, and what
untrusted clients can verify. All domain-specific computation happens in clients.

Skrynia is an unusually small shared-memory substrate: durable public values,
single-key conditional mutation, and delegable resource-bounded write authority.
These primitives can support applications conventionally associated with
databases and application servers.

## The substrate and its guarantees

| Primitive | Actual Skrynia interface | What it enables |
| --- | --- | --- |
| Public byte values | `GET /store/{key}`, optionally served by nginx | Shared data, independent readers, static app distribution |
| Create-if-absent | `POST /store/{key}` (201 or 409) | Initialization, unique records, races for one key |
| Atomic single-key replacement | `PUT` of an existing key | Versioned state, publication pointers |
| Compare-and-swap (CAS) | `GET` ETag, then `PUT` with `If-Match` (200 or 412) | Optimistic concurrency, one-key linearization points |
| Conditional deletion | `DELETE` with optional `If-Match` | Reclamation coordinated with a key version |
| Scoped, delegable grants | Key/prefix matcher, subset of `write` / `delegate` / `subscribe`, quotas | Delegated participants, isolation of different storage regions, bounded authority |
| Notifications | Grant-scoped Web Push subscriptions | Wake interested clients without running application code on Skrynia |

The key alphabet is `[A-Za-z0-9_.]+` excluding the exact keys `.` and `..`; a location matcher is one exact key,
a prefix ending in `*`, all keys (`*`), or the empty set. There is **no**
atomic operation across multiple physical keys, general query evaluator,
application-specific server code, or private read operation.

**ETag detail.** Conditional writes compare against the current representation
ETag, not against the current payload bytes. In the current implementation,
successful object mutations use a monotonic global filesystem timestamp
sequence; ETags include that timestamp and object size. Replacing a value with
identical bytes still produces a new ETag. Clients must preserve the exact ETag
returned by `GET`, not calculate one from contents. `POST` rejects existing keys;
`PUT` requires an existing key; `DELETE` supports a version precondition.

### Three different kinds of guarantees

1. **Storage-level:** Skrynia enforces token scope, quotas, and one-key conditional
   mutations, even for an arbitrary caller.
2. **Protocol-level:** cooperating clients can implement stronger *logical*
   abstractions using those primitives. Their assumptions are part of the
   protocol.
3. **Validation-level:** untrusted clients can publish arbitrary bytes where
   authorized; readers can accept only structures whose signatures, hashes,
   provenance and transition rules they verify. Detecting invalid data is not
   the same as preventing its publication, overwriting, deletion or starvation.

Do not call a client protocol secure merely because its JavaScript performs
validation: clients can modify or bypass their own JavaScript. Security must
come from **Skrynia-enforced authority boundaries** or **independent
verification of accepted data**.

## 1. Atomic multi-object *logical* transactions

**Construction: immutable versions plus one mutable root.**

Suppose a logical database has two accounts, `alice` and `bob`, stored across
multiple immutable objects. A single small object named `db_head` contains
the key of the currently committed manifest:

```json
{"version":17,"manifest":"manifest_9d..."}
```

The manifest identifies the exact keys of the account records, or the root of
a persistent tree with many records. Each new version reuses unchanged nodes.
The commit is only the conditional replacement of `db_head`.

Writer algorithm:

1. `GET db_head`, saving both its value and the *exact ETag*.
2. Read all required records through that manifest. Evaluate the full
   transaction and its preconditions against this particular snapshot.
3. Create fresh, unique, unreferenced keys for all changed records and a new
   manifest. Wait for all creations to succeed. Publish nothing yet.
4. `PUT db_head` with `If-Match: <etag-from-step-1>`, pointing to the new
   manifest. **HTTP 200 is the commit point.**
5. If HTTP 412, another client committed first. Fetch the new head and
   **re-evaluate the transaction** (including balance checks, uniqueness,
   predicates and read dependencies); do not merely republish stale results.
6. If the outcome of the request is unknown (network loss), read the head
   and, if necessary, the retained commit history using a unique transaction
   ID. Do not blindly assume commit or abort.

Reader algorithm:

1. Read `db_head` **once** and save the manifest key.
2. Follow only references belonging to that manifest.
3. Never mix references taken from two different heads.

For a transfer `alice -= 2; bob += 2`, the reader sees either both balances
before the transfer or both afterward, even though the two account records
are physically separate files. A second writer might stage a different pair
of balances concurrently; only one CAS against the observed head succeeds.
The loser retries against the winner's state.

**Why this works:** Every committed logical version is constructed before it
is reachable from `db_head`. A successful CAS provides one total order of
commits to that head. Old manifests remain available, so a reader already
using the previous version does not see a partial update. A `POST` failure
for a generated key is a name collision; choose another key, never overwrite
a previously published immutable record.

**Required assumptions:**

- Every logical read/write participating in this transaction is represented
  by the same root's immutable reachable state. Direct writes to account
  files outside this protocol bypass the transaction model.
- Published versioned files **remain unchanged and readable** while referenced
  by readers. Random key names alone do not enforce immutability: the `write`
  operation also permits `PUT` and `DELETE`. Use authority separation and/or
  digest verification, and plan version retention.
- Use an authoritative, suitably fresh root read if promising linearizability.
  Third-party HTTP caches or replicas may return a stale root. Reading a stale
  but intact immutable manifest still gives a coherent snapshot, not
  necessarily the *latest* one. A useful deployment pattern is to fetch
  the mutable head with fresh validation from the authoritative origin and
  fetch versioned, digest-verified records from ordinary static/CDN servers.
- The root must fit a single object. Larger databases use a small manifest
  pointer into a persistent B-tree, Merkle DAG or similar structure.
- Clients need write authority for staging keys and the root, and enough
  quota for temporarily retained versions.

**Exact claim:** This yields serializable transactions and atomic snapshot
publication *for data accessed through this protocol*, given the assumptions.
It does **not** atomically mutate several physical files, provide multi-key
atomic visibility to a client that independently fetches raw keys, or make an
uncooperative writer respect the protocol.

### Failed commits, crashes and reclamation

A client crash before CAS leaves unreachable staging objects. A crash after
CAS leaves a committed version. A lost HTTP acknowledgement may require
checking a transaction ID in committed history. Unreferenced staging records
consume physical quota until explicitly removed.

Deleting an old manifest just because it is no longer the current head is
unsafe: a slow reader may still be using it. Deleting an apparently unreachable
candidate can also race with a writer about to publish it. A correct
reclamation protocol must protect active staging writers and pinned readers,
or use explicitly stated quiescence/retention assumptions. With arbitrary
anonymous public readers, there is no automatic way to prove that *every*
reader has finished. Keeping versions indefinitely is the simplest safe
policy; reclamation trades space for explicit assumptions.

## 2. Shared logs, queues and state machines

**Append-only logical log:** Represent each entry as an immutable object
`event_<random>` containing its payload and the previous head identifier.
Create the entry first, then CAS a `log_head` pointer to the entry. Two
concurrent appends compete at the head; the loser rereads, creates an entry
with the new predecessor, and retries. Readers walk the committed chain.
This has one ordered commit point without an append endpoint.

**Queue / work distribution:** Maintain a queue head or an immutable-tree
queue state under CAS. Clients claim work through conditional state
transitions, and record completion or requeue it. Durable application-level
work continues when another client resumes. Delivery semantics depend on
the protocol: a crashed worker may require retries and operation IDs;
exactly-once external side effects are *not* implied by CAS alone.

**Shared state machines:** Encode a state and a deterministic transition rule
in a versioned root. Clients read, propose `F(state, input)`, and CAS the
result. A rejected CAS triggers reevaluation. This implements counters,
registries, election records, coordination structures and other sequential
objects. It can become a bottleneck under contention; it is not magically
wait-free, fair, or independent of Skrynia's availability.

**Indexes and queries:** Store a client-maintained key directory, inverted
index, or persistent search tree in immutable objects with one published root.
Readers evaluate queries on the selected version in their own process.
Updates to records **and their indexes** can commit together as one *logical*
transaction. No SQL implementation or key-list endpoint is required; a client-managed
index supplies discovery.

Theoretical context: CAS is a *universal synchronization primitive* in the
classical shared-memory model (Maurice Herlihy, 1991,
[Wait-Free Synchronization](https://web.cs.wpi.edu/~cs3013/e11/Papers/Herlihy_WaitFreeSync.pdf)).
That result motivates these constructions; it is not a proof that a particular
HTTP implementation is wait-free or withstands arbitrary malicious writers.

## 3. Collaboration without application code on the server

A **public discussion board** is one example, not a built-in data type.

One possible protocol:

1. Give each author a grant for `author_<id>_*` with limited bytes and objects.
   A user's token cannot write another author's prefix. An author can
   publish, revise, or remove their own contributions according to that
   grant's authority.
2. Authors publish signed posts or a signed index of their own posts. A
   document describes the thread, parent post, author and content. Readers
   verify signatures and application-level rules.
3. A community index maps discussion IDs to posts or author indices, with
   concurrent updates serialized by a CAS root. Alternatively, a curator
   client publishes an accepted index after validating submissions.
4. Browsers read the index and objects publicly and render the discussion.
   Web Push can help active clients notice updates. No dynamic application
   server is needed.
5. Revoking an author's grant disables subsequent writes without deleting
   already published content. Moderation can change the *accepted index*
   without requiring deletion of the underlying contribution.

This supports more than one trust arrangement. The community may use a
moderator-controlled canonical index, several independently signed views,
or agreed client-side rules for selecting valid contributions. It is a
protocol design choice, not a Skrynia feature requirement.

A publicly distributed `write` token for a shared prefix authorizes its
holder to overwrite or delete *any* matching key, including others' posts.
`POST` is create-only as a request **but there is no create-only grant
operation**. For adversarial participants, use separate scopes/credentials
or signed content plus a trusted acceptance boundary, and distinguish
*integrity detection* from guaranteed *availability*.

More examples follow the same recipe: collaborative documents use a
versioned state or operation log; directories and registries are CAS
indexes; polls or voting systems store independently verifiable signed
ballots and a derived tally; shared games publish ordered valid moves.
Application-specific validation and presentation remain client-side.

## 4. Fine-grained trust, identity and privacy

**Capability delegation.** The issuer token mints user tokens and can revoke
the grant; user tokens exercise only the delegated operations. A user with
`delegate` can issue a child grant whose locations and operations are subsets
of its parent. Maximum byte/object budgets constrain writes through the
child and its ancestors. A delegate cannot create an authority it does not
possess. Different participants can hold disjoint or overlapping grants
depending on the desired protocol.

**Signed records.** A signed object can assert authorship. Content hashes can
bind a manifest to exact payload bytes. Clients can verify a transition and
ignore data not justified by their application rules. For a canonical
accepted head, restrict who can write that head, or define a verifiable
accepted history. A malicious client with head-write authority can still
replace/delete it or repeatedly win races, so cryptographic validity and
service availability are distinct claims.

**Private payloads over public reads.** Applications can encrypt values
client-side and distribute decryption keys independently. This provides
payload confidentiality under appropriate cryptographic assumptions, but
Skrynia still exposes ciphertext, object names, sizes and access patterns.
Revoking a grant does not revoke a previously distributed decryption key or
erase copies already downloaded.

**Quotas are scope-based, not reservations.** Usage of a grant counts
matching objects, *including objects another grant created*, plus descendant
metadata/subscriptions. Only the writing grant and its ancestors have their
limits checked for the particular write. Overlapping grants can therefore
observe usage above their own limits. When participants need independent
capacity accounting, choose disjoint key prefixes, not overlapping ones.
Grant revocation preserves objects and consequently may preserve their
measured quota cost.

## 5. Work, liveness and durability

Skrynia stores state but does not execute application-specific computation.
A computation can be performed by *any* client with the necessary data and
authority, including volunteer workers. Work queues, recovery markers,
checkpoints and CAS allow another client to resume interrupted work.
Web Push is a notification channel, not a guarantee that some client will
run. Clients can compute concurrently or independently.

Durability is not the same as progress: if nobody runs an appropriate client,
no application-specific transition takes place. And a single Skrynia service
is a shared point of availability and trust for writes; client replication
does not by itself make the storage service fault-tolerant.

## 6. Designing protocols and identifying genuine gaps

Before proposing a new Skrynia operation:

1. Specify the observable **application invariant** and the adversary model
   (cooperative crash, malicious writer, compromised issuer, unavailable
   storage, stale reader).
2. Identify the minimal **linearization point**: a create-if-absent, a
   conditional replacement, or an accepted signed checkpoint.
3. Put dependent logical state under **one committed root** if a multi-object
   snapshot is needed. Arrange physical objects as immutable candidates.
4. Specify **who can mutate** the root and referenced values, and how clients
   independently authenticate/validate them.
5. Describe **races, failed CAS retries, lost acknowledgements, crashes,
   unreferenced staging objects, and snapshot retention**.
6. Test whether correct clients can complete their work under the stated
   availability and trust assumptions; do not treat missing progress as a
   missing query language.
7. Request a new trusted primitive only if the desired property cannot be
   constructed with acceptable safety, liveness, complexity and cost.

The useful engineering questions are often *how to structure authority,
indices, immutable history, and client participation*, not how to move
application-specific logic into Skrynia.
