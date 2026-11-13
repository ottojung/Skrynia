# Applications and feasibility

Skrynia is public, durable, flat-key object storage with independent scoped
`create`, `modify`, `delete`, `list`, `delegate`, and `subscribe`
authority, per-key atomic mutations, and content-hash compare-and-swap (CAS).
Applications define their own data formats, verification rules, indexes, and
computation in clients. This page gives examples, **not** new server promises.
For constructions and failure conditions, see [client protocols](client-protocols.md).

## Reading the classifications

- **Easy:** A small client protocol for a known set of participants.
- **Moderate:** Requires indexing, credential setup, concurrency control, or
  client-side encryption, but no unusually difficult distributed protocol.
- **Hard:** Significant adversarial, cryptographic, latency, recovery, or
  distributed-protocol engineering.
- **Unavailable:** The defining property cannot be provided by Skrynia alone.

"On Skrynia" means Skrynia plus participating browser, desktop, or mobile
clients; it does **not** silently assume a trusted, always-on application
backend, external payment processor, or server-side application execution.
Where a useful application depends on other infrastructure (WebRTC peers,
STUN/TURN, payment settlement, external workers), that dependency is stated.
Ratings depend on the threat model: restricting writes to known participants
is much easier than accepting arbitrary anonymous participants.

## Examples

| Application | Difficulty | Construction or limiting factor |
| --- | --- | --- |
| Static blog, documentation, personal homepage | Easy | Publish HTML/assets and optionally switch a version manifest via CAS. |
| Personal knowledge base | Easy | Public notes or client-encrypted notes, with client-managed navigation. |
| Public image gallery or file share | Easy (invited) | Publish named byte objects; large files need client-side chunking (default object limit: 10 MiB). |
| Pastebin or guestbook | Easy (invited); Moderate/Hard (open) | One create-only object per contribution; public submissions need admission, abuse, and quota policy. |
| Simple two-person messenger | Easy | Distinct sender prefixes, `list`-based discovery, and optional push wake-ups. |
| Public chat room | Moderate | Key discovery and rendering are simple; open membership, moderation, and prompt delivery are not. |
| Encrypted two-person messenger | Moderate | Clients encrypt payloads and manage authenticated keys; metadata remains observable. |
| Encrypted multi-party messenger | Hard | Group membership, key rotations, compromised devices, and message ordering. |
| Wiki or issue tracker | Moderate | Signed/versioned records, indexes, and concurrent-update policy. |
| Forum or social feed | Moderate (invited); Hard (open) | Author-scoped grants, curated indexes, moderation, identity, and spam resistance. |
| Git-like repository or package registry | Moderate | Immutable versions, signed metadata, and CAS branch/release heads. |
| Searchable document collection | Moderate | Clients maintain and query an inverted index; Skrynia does not execute queries. |
| Shared calendar, todo list, turn-based game | Moderate | Client transitions, an accepted state/log, CAS or restricted create-only slots. |
| Invited-member poll | Moderate | Issuer gives each eligible voter a create-only exact-key credential; tally and verification remain client-defined. |
| Open public election | Hard | Eligibility, Sybil resistance, ballot secrecy, coercion, and tally validity. |
| Collaborative text editor | Hard | CRDT/OT is available, but offline merges, durability, and low-latency propagation remain substantial. |
| Client-side transactional database | Hard | Immutable state/indices and one CAS commit root; retention, contention, and trustworthy writers need design. |
| Trusted-sequencer ledger or contract protocol | Hard | Clients verify deterministic transitions; a chosen sequencer publishes canonical decisions. No Skrynia-side contract execution. |
| Distributed work queue | Moderate | CAS-based claims, leases/retries, and online external worker clients. |
| Volunteer compute coordination | Hard | Untrusted-worker validation, task assignment, and result verification; workers perform all computation. |
| Secure file vault | Moderate/Hard | Client encryption, key management, device recovery, and rollback considerations. |
| Password manager | Hard | Similar storage model to a vault, plus demanding credential, device, and recovery security. |
| Static e-commerce storefront | Easy | Product catalog, static UI, and client-published inventory display. |
| Storefront with actual payment settlement | Requires external service | Skrynia may coordinate an order, but cannot settle payments itself. |
| Peer-to-peer voice/video call | Hard; external network support often needed | Use stored objects for signaling and WebRTC for media; STUN/TURN may be required. |
| Real-time action game | Hard; bounded latency unavailable | Polling/push is not a reliable low-latency gameplay transport; a peer transport or separate server may be needed. |

## Properties Skrynia cannot itself supply

These are **functional boundaries**, not simply the absence of a familiar
server-side API. Client computations can provide a similar user experience
under different trust or liveness assumptions.

| Requirement | Why Skrynia alone cannot guarantee it |
| --- | --- |
| Server-enforced private reads | Anyone knowing an existing key can GET its bytes; `list` requires a token, but reads do not. |
| Revocable confidentiality of previously delivered bytes | Deleting a key or revoking a grant cannot erase copies already fetched or revoke a disclosed decryption key. |
| Secret application actions performed while clients are absent | The server has no application-specific secret-handling or code-execution facility. Its own private issuer credentials are not a generic application secret API. |
| Trusted application-specific timestamps | Filesystem timestamps are not an authenticated time-ordering/attestation protocol. |
| Guaranteed application progress with no active executor | Push wakes clients, but cannot force one to run application logic. |
| Trustless distributed consensus | CAS serializes mutations at one authoritative storage service; it does not make that authority decentralized. |
| Server-side arbitrary compute or smart-contract execution | Clients may compute and verify rules, but the server never executes those rules. |
| Guaranteed low-latency bidirectional media or changes | Web Push is coalesced, retryable notification, not a real-time stream or peer media transport. |

## Security and coordination nuances

### Create-only contributions are especially useful

A `create`-only grant can add objects but **cannot replace or delete
existing objects**. Disjoint author prefixes reduce one author's power over
another's records. An exact-key create-only grant can enforce *at most one
successful create at that key*, provided other actors with broader authority
do not replace/delete it. A trusted voter-credential issuer can use this for
one ballot *slot* per issued voter; Skrynia does not establish real-world
eligibility, identity, secrecy, or one-human-one-vote.

One entry per `log.<number>` key is not automatically a canonical
append-only history. A later `POST log.1` can appear between existing
`log.0` and `log.2`. Sorted listing order is not creation order; listing
also has pagination without a cross-page snapshot. Requiring a contiguous
prefix makes gaps or maliciously filled slots potential availability
blockers. Deterministic replay can ignore invalid entries, but cannot turn
arbitrary arrival order into a stable committed prefix.

For a canonical log, use an explicitly trusted sequencer that publishes
successive slots, or immutable nodes plus a CAS-updated head and a
validation protocol. The CAS design needs `modify` authority over the
head: a malicious holder can replace it with arbitrary bytes, so readers
must validate history and authority must be deliberately constrained.
Create-only immutability applies to the *holder's permissions*, not to
other overlapping grants or the root. No design prevents a powerful
authorized actor from suppressing publication or exhausting shared quota.

### Public reads do not imply public enumeration

A caller with an unguessable key can read the object without a token;
a caller without the key needs `list` authority to enumerate names through
Skrynia's listing API. This enables *unlisted-link obscurity*, not
server-enforced access control. Keys leak through links, referrers,
logs, devices, and other channels; the storage operator can inspect keys,
contents, and sizes. Use client-side encryption for confidential payloads,
and assume ciphertext is available for offline attack if the key leaks.
Even encrypted content exposes some metadata and does not erase copies
already downloaded.

### Public write access and browser origins

A shared write token is a shared capability: anybody holding it can spend
its granted quota, occupy keys, and submit unwanted content. Storage quotas
bound cumulative usage; they do not establish unique human identities,
per-actor request rates, or resistance to Sybil accounts. Open registration
needs an admission/issuer policy and often some continuously available
party to execute it. Admission can also be done by pre-issued invitations
when dynamic signup is unnecessary.

Stored `.html`, JavaScript, and `.svg` can execute as active browser
content when hosted with suitable MIME types. Mutually untrusted publishers
must have separate browser origins, not merely different key prefixes.
See [operations](operations.md) for the hosting requirement.

### Notifications, time, and progress

Skrynia Web Push is a generic `changed` wake-up with coalescing,
retries, duplicates, and possible eventual retirement of subscriptions.
Clients must re-fetch current values and cannot assume every individual
event arrives or arrives within a deadline. Polling can sometimes provide
adequate responsiveness but consumes requests and does not guarantee
real-time latency. A WebRTC application can use Skrynia for signaling, not
as a media relay; Internet connectivity may require external STUN/TURN.

Potential future primitives worth *investigating*, not adopted
requirements: a low-latency change feed, authenticated server sequence/time
metadata, and per-actor rate limiting. Each requires a specified threat
model and an argument that existing client protocols cannot meet the goal
with acceptable cost.
