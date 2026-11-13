# Skrynia Cookbook

Practical recipes for building applications on **Skrynia's existing storage
primitives**, without adding application logic to the server.

Each recipe explains which authorities and operations are needed, how clients
coordinate, why the construction works, and where it stops working. These are
**client protocols**, not additional server guarantees. Distinguish what
Skrynia enforces for arbitrary callers from what cooperating clients promise
and from what readers can independently verify.

Skrynia provides public durable objects, atomic create-if-absent, byte-hash
compare-and-swap (CAS), scoped grants, listing, and Web Push wake-ups.
Applications implement their own schemas, indexes, checks, and computation.

## Recipes at a glance

- [Atomic logical transactions](#recipe-atomic-multi-object-logical-transactions):
  publish immutable versions through one CAS commit head.
- [Key discovery](#recipe-scoped-key-discovery-and-contributions):
  independent create-only contributions with scoped listing.
- [Append-only log](#recipe-an-append-only-log-using-only-create-authority):
  ordered publication by cooperating create-only writers, no CAS.
- [Garbage-collected log](#recipe-garbage-collection-using-sealed-epochs):
  revoke old writer authority and collect acknowledged sealed prefixes.
- [CAS logs, queues, and state machines](#recipe-cas-logs-queues-and-state-machines):
  mutable heads for more flexible coordination.
- [Public collaboration](#recipe-public-collaboration-without-an-application-backend):
  posts, community indices, and curation.
- [Delegation and privacy](#recipe-delegation-identity-and-confidentiality):
  signed objects, unlisted keys, encrypted payloads, and quotas.
- [Client-run work](#recipe-client-run-work-and-crash-recovery):
  workers, recovery, and the absence of guaranteed execution.

For a comparative application catalog, see
[application feasibility](application-feasibility.md).

## Ingredients: storage primitives and guarantees

| Primitive | Actual Skrynia interface | What it enables |
| --- | --- | --- |
| Public byte values | `GET /store/{key}`, optionally served by nginx | Shared data, independent readers, static app distribution |
| Create-if-absent | `POST /store/{key}` (201 or 409) | Initialization, unique records, races for one key |
| Atomic single-key replacement | `PUT` of an existing key | Versioned state, publication pointers |
| Compare-and-swap (CAS) | SHA-256 of observed bytes, then `PUT` with `X-Skrynia-If-SHA256` (200 or 412) | Optimistic concurrency, one-key linearization points |
| Conditional deletion | `DELETE` with optional `X-Skrynia-If-SHA256` | Reclamation coordinated with observed object bytes |
| Scoped, delegable grants | Key/prefix matcher, subset of `create` / `modify` / `delete` / `list` / `delegate` / `subscribe`, quotas | Delegated participants, isolation of different storage regions, bounded authority |
| Notifications | Grant-scoped Web Push subscriptions | Wake interested clients without running application code on Skrynia |

Keys match `[A-Za-z0-9_][A-Za-z0-9_.]*`, permitting consecutive dots but
not leading dots. A location matcher is one exact key, a prefix ending in
`*`, all keys (`*`), or the empty set. There is **no**
atomic operation across multiple physical keys, general query evaluator,
application-specific server code, or private read operation.

**Byte-content preconditions.** PUT and DELETE can require the SHA-256
of the current object's bytes using `X-Skrynia-If-SHA256`. The digest is
64 lowercase hexadecimal characters and may be computed from bytes read
through **any** static server. Skrynia's own GET/HEAD ETag is `"<sha256>"`, serving as
an HTTP cache validator; external static servers may use their own ETags.
`POST` rejects existing keys, and `PUT` requires an existing key.

This CAS compares **bytes**, not histories. An A → B → A sequence is
indistinguishable from an uninterrupted A if the bytes return to A.
Higher-level protocols requiring that guarantee must encode a unique
transaction ID or strictly advancing revision in the mutable value.
A returned digest is not a guarantee that a write happened only once.

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

Do not call a client protocol secure merely because a cooperating client performs
validation: untrusted clients can modify or bypass their own checks. Security must
come from **Skrynia-enforced authority boundaries** or **independent
verification of accepted data**.

## Recipe: Atomic multi-object *logical* transactions

**Construction: immutable versions plus one mutable root.**

Suppose a logical database has two accounts, `alice` and `bob`, stored across
multiple immutable objects. A single small object named `db_head` contains
the key of the currently committed manifest:

```json
{"revision":17,"transactionId":"tx_62...","manifest":"manifest_9d..."}
```

The manifest identifies the exact keys of the account records, or the root of
a persistent tree with many records. Each logical revision reuses unchanged
nodes. Every commit **must** assign `revision = observed_revision + 1` and
a fresh, unique `transactionId`, even when the manifest or data is unchanged.
A cooperating client must never publish a `db_head` byte string already
published by this protocol. The commit is the conditional replacement
of `db_head`.

Writer algorithm:

1. Read the exact bytes of `db_head` and compute their SHA-256.
2. Read all required records through that manifest. Evaluate the full
   transaction and its preconditions against this particular snapshot.
3. Create fresh, unique, unreferenced keys for all changed records and
   a new manifest. Prepare a `db_head` value with a strictly advancing
   `revision` and a fresh, unique `transactionId`. Wait for all creations
   to succeed. Publish nothing yet.
4. `PUT db_head` with `X-Skrynia-If-SHA256: <digest-from-step-1>`,
   pointing to the new manifest. **HTTP 200 is the commit point.**
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
- Each committed root has a strictly increasing `revision` and a unique
  `transactionId`, with bytewise distinct head values across all commits.
  Cooperating writers validate this rule when updating the head. A writer
  who publishes arbitrary root contents can violate it, and Skrynia's
  byte-content condition does not itself prevent A → B → A.
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

## Recipe: Scoped key discovery and contributions

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

A `create`-only grant prevents its holder from replacing or deleting
already-created entries. This is a valuable storage-enforced invariant,
not just a client convention. In particular, a grant scoped to one exact
key can allow at most one successful initial publication **at that key**,
provided no broader authority replaces or deletes it.

Sequentially named entries (`log.0`, `log.1`, ...) do **not** by themselves
form a canonical append order: `log.2` may be created before `log.1`, and
a later insertion can change sorted replay order. Requiring a contiguous
prefix avoids observing holes but risks blocking on missing, invalid, or
maliciously occupied slots. Skipping invalid contents deterministically
does not eliminate these ordering and liveness problems. A designated
sequencer can publish numbered slots under restricted authority, with a
single point of trust and availability.

Cooperating writers can also preserve a strictly contiguous sequence by
racing to `POST` the first missing successor, as in the create-only log
recipe below. That claim requires strict writer discipline, no deletions,
and fresh reads; it does not follow merely from sorting arbitrary keys.
When clients require a canonical ordered history without such assumptions,
a CAS-linked log or committed manifest can supply its ordering. A holder
of `modify` authority over the head can still publish arbitrary bytes:
clients must verify acceptable transitions and the application must limit
that authority. `list` alone is key discovery, not a tamper-resistance
guarantee against holders of `modify` or `delete` authority: those can
still replace or remove entries. A maintainer can list a delegated key range
and explicitly delete unwanted contributions with an appropriate write
token. Subspace isolation, cryptographic validation, and client protocols
control what histories participants accept.

## Recipe: An append-only log using only create authority

**Use when:** cooperating writers need one ordered, immutable stream without a
mutable head or a trusted sequencer. This works with **create-only** writer
tokens; readers need only know the keys (and may use a separate `list` token
for discovery). It assumes no deletion of live log entries.

**Layout:** reserve one exclusive prefix, such as `log.e000001.*`, and use
fixed-width decimal slot keys:

```text
log.e000001.000000  # genesis record, created once by the initializer
log.e000001.000001  # first message
log.e000001.000002  # second message
```

Grant cooperating writers `create` **only** for that epoch's keys. Other
writers must not have overlapping `modify` or `delete` authority. The root
operator remains trusted. Numbering is a *client protocol*, not something
Skrynia checks. The genesis object establishes the start of a contiguous
sequence; initialize it before allowing other writers to append.

**Writer:**

1. Starting after genesis, find the *first missing* slot using fresh,
   authoritative point reads (`GET` or `HEAD`). Equivalently, find the
   smallest existing `X0` and then the first missing consecutive successor.
   A scoped listing can be an optimization, but its pages are not an atomic
   snapshot: do not infer missing slots from a partial or stale listing.
2. `POST` a new entry at the missing slot, including a unique application
   operation ID in its bytes. If creation succeeds (201), that slot is the
   append's commit point.
3. On 409 (someone else created the slot first), advance to the next
   successor and retry. Do **not** skip an unverified missing predecessor.
4. If a response is lost, read the attempted key. If it contains this
   operation ID, the append succeeded; if it contains another entry, advance;
   if it remains absent, retry the *same* key. Do not append the same logical
   operation twice merely because the HTTP acknowledgment was lost.

A slower, particularly simple writer needs no listing or read permission:
attempt `POST` at successive slot numbers from genesis, advancing only on
409, until one succeeds. This can be expensive for long logs.

**Reader:** start after genesis and read consecutive slots, stopping at the
first 404. Use authoritative, cache-fresh reads for a guarantee about the
current committed prefix. Every successfully read entry belongs to a stable
prefix; new appends only extend the tail. Readers must not mistake the first
missing key for a permanently sealed end of history.

**Why it works:** by induction, initially only genesis exists. Every
cooperating writer attempts the first absent successor of the existing
contiguous prefix, and `POST` atomically permits only one winner at that
key. Under the no-deletion assumption, a successful append can only extend
the prefix by one. The numeric order of slots therefore agrees with the
linearization order of successful appends; no CAS head is needed.

**Limits:** This is not a Skrynia-enforced *numbering rule*. A malicious
create-only writer can fill future slots, publish invalid payloads, or spend
quota; even if readers deterministically skip invalid contents, they cannot
force progress through a missing or maliciously filled slot. The guarantee
also fails if any authorized actor removes or replaces live entries. Fixed
width must accommodate expected sequence length or trigger a new epoch.
Scanning from genesis costs linear work without a trustworthy acceleration
strategy. Log entries are public, so encrypt confidential payloads in clients.

## Recipe: Garbage collection using sealed epochs

**Use when:** the append-only protocol above needs bounded *retained*
history. Unlike append-only publication, safe reclamation requires trusted
coordination over writer authority and consumer retention.

**Why naive prefix deletion fails:** A writer may observe the current first
entry, then a collector deletes a prefix, and the stale writer successfully
`POST`s at one of the freed names. This reuses a previous position and can
put an apparently new message *before* previously appended messages.
Acknowledgments for the former occupant could cause the new message to be
deleted. Merely deleting contiguous prefixes does not remove this race.

**Layout and participants:**

- A coordinator owns a small versioned `log.control` head (or a CAS head
  pointing to immutable control manifests). It records the active epoch,
  sealed epoch boundaries, the earliest retained position or checkpoint,
  and registered consumers/acknowledgments. Transitions to the head use
  SHA-256 CAS; include advancing revisions to prevent ABA.
- Each epoch has distinct fixed-width keys, such as `log.e000001.000005`
  and `log.e000002.000000`. Its contributors receive a *separate*,
  revocable subtree of `create`-only grants. No other distributed token
  has write authority over an old epoch.
- Readers are **registered consumers** with durable next-unprocessed
  positions. The coordinator serializes consumer enrollment and retained
  floor advances through the same control state. New consumers start at a
  documented checkpoint, not from an already-deleted arbitrary beginning.
  Arbitrary anonymous readers are not promised perpetual replay.

**Epoch transition:**

1. Stop writes to the current epoch by revoking its entire writer-grant
   subtree (or all equivalent write credentials). Wait for revocation to
   complete; a request committed before revocation is included. Ensure
   no overlapping broader grant can continue writing that epoch.
2. Read its final contiguous prefix *after* revocation and publish an
   immutable sealed-epoch descriptor identifying its terminal slot.
   With malicious or faulty writers, do not silently treat a hole as a
   terminal slot; validation/repair policy is application-defined.
3. Publish the next epoch and its ordering relative to the old epoch in
   the CAS-controlled metadata. Only then distribute usable next-epoch
   writer credentials. A crash between revocation and publication may
   temporarily halt appends, but must not allow old-key reuse. Recover
   using persisted control state.
4. Writers append only to the currently published epoch. Readers traverse
   sealed epochs up to their recorded terminal slot, then continue into
   the next epoch. In an *active* epoch, the first missing slot is the
   current end; a missing retained slot in a *sealed* epoch is an error.

**Collecting acknowledged prefixes:**

1. Each registered consumer durably acknowledges a position only after
   processing it and relinquishing any need to reread that prefix. Use
   per-consumer authenticated acknowledgment keys or CAS cursors; account
   for compromised, lost, or falsely advanced consumer credentials.
2. Compute a safe cutoff from the minimum acknowledged position of the
   registered consumers promised replay. A slow/offline consumer blocks
   collection until it catches up or is **explicitly** dropped under the
   application's retention policy.
3. The collector may retire only a prefix belonging to **sealed epochs**
   whose create authority cannot be reissued by untrusted parties. Advance
   the retained floor via CAS against the same state that controls consumer
   enrollment, so a new reader cannot register for history being retired.
4. **After** the retained floor commits, delete retired keys with an
   authorized `delete` grant restricted to sealed epochs. Deletion can
   resume after a crash. Never give an untrusted writer this deletion
   authority; Skrynia has no per-object owner restrictions.
5. Existing consumers resume from their durable position; new consumers
   initialize from the published checkpoint/floor. A reader cannot safely
   infer this position just by asking for the name-smallest *physical*
   object: deletion may be in progress or may have left old bytes behind.

**Safety argument:** no writer can reuse a key after the epoch is sealed,
and the retained floor cannot pass any consumer still entitled to read the
retired prefix. CAS coordinates new registrations with floor advancement.
Consequently, deleting the acknowledged prefix cannot reorder future
appends or deprive a registered, correctly acknowledging reader of data
it has yet to consume.

**Trust and liveness assumptions:** a coordinator/issuer must actually
revoke all write authority (including all relevant descendants), not
reissue it for retired keys, and preserve the epoch and consumer metadata.
Skrynia's root authority can always override subordinate grants. Consumers
must acknowledge honestly; the collector must follow the shared control
protocol. Epoch rotation, callbacks, and collection are run by clients,
not automatically by Skrynia. If no coordinator runs, writers may stall and
garbage collection does not progress. If all past readers must be able to
replay from genesis forever, no finite garbage collection policy can meet
that requirement.

## Recipe: CAS logs, queues, and state machines

**CAS-linked logical log:** Unlike the create-only recipe above, this
uses a mutable head, so the authority and integrity of that head matter.
Represent each entry as an immutable object
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

## Recipe: Public collaboration without an application backend

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

A publicly distributed `modify` or `delete` token for a shared prefix authorizes
its holder to overwrite or delete *any* matching key, including others' posts.
A grant with **only `create`** lets contributors publish new entries without
modifying or deleting any existing entries. Delegate disjoint author prefixes
to bound each participant's authority; trusted curators may maintain a
separate accepted index. Signatures can detect invalid content but do not
prevent deletion by a holder of matching delete authority or guarantee
availability.

More examples follow the same recipe: collaborative documents use a
versioned state or operation log; directories and registries are CAS
indexes; polls or voting systems store independently verifiable signed
ballots and a derived tally; shared games publish ordered valid moves.
For an invited poll, a trusted issuer can give each voter a create-only
grant restricted to one exact ballot key, enforcing at most one creation
for that issued slot. That does not establish voter eligibility, protect
ballot secrecy, prevent Sybil identities, or make the tally trustworthy.
Application-specific validation and presentation remain client-side.

## Recipe: Delegation, identity, and confidentiality

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

**Private payloads over public reads.** Reading an existing object requires
knowing its key but no read token; key enumeration through the Skrynia API
requires `list`. High-entropy unlisted keys can discourage casual discovery,
but are not enforced private reads: keys can leak through URLs, referrers,
logs, or other clients. Applications can encrypt values client-side and
distribute decryption keys independently. This provides payload
confidentiality under appropriate cryptographic assumptions, but Skrynia
still exposes ciphertext, object names, sizes and access patterns to the
storage operator and to anyone who discovers the relevant keys. Exposed
ciphertext is available for offline attacks. Revoking a grant does not
revoke a previously distributed decryption key or erase downloaded copies.

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

## Recipe: Client-run work and crash recovery

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

## Designing a new recipe: when a primitive is actually missing

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
