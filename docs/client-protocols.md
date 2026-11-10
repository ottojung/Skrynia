# Client-side protocols on Skrynia

**Purpose.** This document describes *applications built from Skrynia's existing
primitives*, not additional server features. The relevant distinction is between
what the server enforces, what cooperating clients can coordinate, and what
untrusted clients can verify. All domain-specific computation happens in clients.

Skrynia is an unusually small shared-memory substrate: durable public values,
single-key conditional mutation, and delegable resource-bounded mutation authority.
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
| Scoped, delegable grants | Key/prefix matcher, subset of `create` / `modify` / `delete` / `list` / `delegate` / `subscribe`, quotas | Delegated participants, isolation of different storage regions, bounded authority |
| Notifications | Grant-scoped Web Push subscriptions | Wake interested clients without running application code on Skrynia |

Keys match `[A-Za-z0-9_][A-Za-z0-9_.]*`, permitting consecutive dots but
not leading dots. A location matcher is one exact key, a prefix ending in
`*`, all keys (`*`), or the empty set. There is **no**
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
  by readers. Random key names alone do not enforce immutability: grants
  with `modify` or `delete` can still change them. Delegate `create` only
  when contributions should be immutable to their contributors; use authority
  separation and/or
  digest verification, and plan version retention.
- Use an authoritative, suitably fresh root read if promising linearizability.
  Third-party HTTP caches or replicas may return a stale root. Reading a stale
  but intact immutable manifest still gives a coherent snapshot, not
  necessarily the *latest* one. A useful deployment pattern is to fetch
  the mutable head with fresh validation from the authoritative origin and
  fetch versioned, digest-verified records from ordinary static/CDN servers.
- The root must fit a single object. Larger databases use a small manifest
  pointer into a persistent B-tree, Merkle DAG or similar structure.
- Clients need `create` authority for staging keys and `modify` for the
  root's conditional publication, along with enough
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

## Scoped key discovery and log entries

A holder of `list` can enumerate live keys within any subset of its
grant's `locations` matcher using `GET /store?locations=...`. This
includes objects created by other grants. Applications can use a grant
scoped to `events.*` to discover event entries without maintaining a
second shared index.

Listings return at most 1000 keys per request by default. Follow the
returned `next` cursor until no continuation is present. Multiple pages
do not provide a consistent snapshot while keys are changing.

One event can be one object named `events.<unique-id>`; its bytes hold
the event or a pointer to a payload. Each contributor creates an entry
with `POST` (`create` authority; create-if-absent), so independent contributions need not
rewrite a growing shared file. Clients with `list` discover entries by
range and read their contents publicly. Keys do not inherently define
a chronological order: an application can encode sequence identifiers,
follow predecessor links, or impose an order in a signed manifest.

When clients require a canonical ordered history, a CAS-linked log or
committed manifest supplies its ordering. `list` alone is key discovery,
not a tamper-resistance guarantee against holders of `modify` or `delete`
authority: those can still replace or remove entries. A maintainer can list a delegated key range
and explicitly delete unwanted contributions with an appropriate write
token. Subspace isolation, cryptographic validation, and client protocols
control what histories participants accept.

## 2. Shared logs, queues and state machines

**Append-only logical log:** Represent each entry as an immutable object
`event_<random>` containing its payload and the previous head identifier.
Create the entry first, then CAS a `log_head` pointer to the entry. Two
concurrent appends compete at the head; the loser rereads, creates an entry
with the new predecessor, and retries. Readers walk the committed chain.
This has one ordered commit point independent of key enumeration.

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
transaction. A client-managed index enables arbitrary queries; scoped
`list` covers simpler key-prefix discovery without a SQL query engine.

Theoretical context: CAS is a *universal synchronization primitive* in the
classical shared-memory model (Maurice Herlihy, 1991,
[Wait-Free Synchronization](https://web.cs.wpi.edu/~cs3013/e11/Papers/Herlihy_WaitFreeSync.pdf)).
That result motivates these constructions; it is not a proof that a particular
HTTP implementation is wait-free or withstands arbitrary malicious writers.

## 3. Collaboration without application code on the server

A **public discussion board** is one example, not a built-in data type.

One possible protocol:

1. Give each author a grant for `author_<id>_*` with limited bytes and objects.
   A user's token cannot mutate another author's prefix. An author can
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

A publicly distributed `modify` or `delete` token for a shared prefix authorizes its
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
accepted head, restrict who can modify that head, or define a verifiable
accepted history. A malicious client with head-modification authority can still
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
metadata/subscriptions. Only the acting grant and its ancestors have their
limits checked for a creation or modification. Overlapping grants can therefore
observe usage above their own limits. When participants need independent
capacity accounting, choose disjoint key prefixes, not overlapping ones.
Grant revocation preserves objects and consequently may preserve their
measured quota cost.

### Guaranteeing a participant's usable capacity without a reservation API

There is a straightforward *grant-allocation protocol* for quota headroom.

The simplest case is to give Alice a grant directly beneath the **unlimited
root**, restricted to an exclusive prefix such as `alice.*` and with a
20 MiB `maxBytes` limit. There is no finite ancestor quota that other users
can exhaust, and other clients cannot consume Alice's counted space
provided they have no authority over her prefix. The grant's fixed
metadata and any subscriptions count against its own 20 MiB limit.

For a **finite parent** with several children, allocate *disjoint* key
ranges (for example `users.alice.*`, `users.bob.*`) and choose finite
child limits in advance. For the parent's byte and object limits require:

\[
egin{aligned}
C_{bytes}+\sum_i B_i &\leq B_{parent},\
C_{objects}+\sum_i N_i &\leq N_{parent}.
\end{aligned}
\]

Here `B_i` and `N_i` are child `maxBytes` and `maxObjects`, including
their own descendant usage and metadata. `C` covers *everything charged to
the parent but not charged within one of those child budgets* (for example,
the parent's own grant record, its extra user tokens, directly sponsored
subscriptions, or objects outside child prefixes). Bound `C` as well.

**Why this guarantees headroom:** Other children may consume their
*entire* budgets, but their combined usage cannot exceed the sum of those
limits. Therefore Alice may grow to her budget without any other child's
legitimate operations exhausting the parent's limit first. The same
inequalities can be maintained recursively at any finite ancestor.
No new Skrynia reservation counter or administration operation is needed:
the commitments are expressed using ordinary scoped grants and budget
arithmetic.

**Required discipline:** Don't issue overlapping mutation grants or
unbudgeted new siblings, and don't distribute a parent's broader mutation
credentials to participants who could bypass their child quotas. Include
already-existing objects and retained values from revoked grants in `C`.
The issuer must maintain the budget inequalities whenever it changes the
delegation tree. Such promises are conditional on the allocator's honest
policy; Skrynia currently checks *actual usage*, not the sum of child maxima.

This gives **quota-level guaranteed headroom**, not guaranteed free disk
blocks. Actual physical capacity, server-wide resource costs, root-level
writers and server availability remain separate concerns.

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
