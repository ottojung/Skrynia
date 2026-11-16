# Security model: read capabilities and application rules

This document explains confidentiality and application invariants using Skrynia's
**existing** interface. It adds no server promises. See the
[HTTP API](api-spec.md), [grant semantics](intent-location-authority.md),
and [Skrynia Cookbook](cookbook.md).

## Public GET, secret names, and confidentiality

"Public reads" means that a known-key GET has no independent read-token
authorization check. It does **not** mean that every object is discoverable.
Skrynia's listing operation requires a scoped user token. An undisclosed,
cryptographically random object name can therefore serve as a **bearer read
capability**: possession of the name authorizes retrieval.

For example, a uniformly random 128-bit name, encoded as 32 hexadecimal
characters, has 2^128 possibilities. If an attacker learns nothing about
the name, makes q distinct guesses, and m independent undisclosed names
occupy that search space, the probability of finding *any* one is bounded
by min(1, q*m/2^128); for one target, by q/2^128. This is a computational
confidentiality argument, not a guarantee that a particular application
keeps its names secret.

The argument assumes all of the following:

- Names are generated with a cryptographically secure random generator,
  with sufficient entropy, rather than from identities, timestamps,
  counters, or public content hashes.
- Unauthorized parties cannot enumerate them through a listing grant,
  reverse proxy, exposed directory, publicly readable index, predictable
  neighboring name, or separate application endpoint.
- Capabilities are shared only with intended readers. Browser referrers,
  access logs, analytics, links, caches, third-party scripts, backups,
  and client devices must not disclose them. HTTPS protects transport,
  not every endpoint or log.
- The storage operator and every server serving the object are trusted
  with its plaintext and name. Otherwise the application encrypts it
  before storage.

Under these assumptions, a secret name provides strong read confidentiality
comparable to a bearer credential in an authorization header **for that
attacker model**. URL paths are often more widely exposed than protected
headers, so security in deployment is not automatically equivalent.

A name is a *shared* bearer capability, not a reader identity. Two readers
using the same name cannot be distinguished by Skrynia's GET interface.
A disclosed name grants access to whoever learns it; holders of matching
listing authority can discover it too. Selective revocation of one recipient
while retaining another requires new names and/or encryption keys,
republication where needed, and handling cached copies. Neither deleting
the object nor revoking write grants erases bytes already downloaded.
A fresh name does not make an old disclosed copy confidential.

Encryption is complementary protection when the operator is untrusted,
names may be revealed, or content must be protected independently of
location secrecy. It is **not** a prerequisite for confidentiality in
every secret-name application.

Skrynia does not itself generate random object names, manage the distribution
of read capabilities, or guarantee their secrecy.

## Three different enforcement properties

**1. Server-enforced authority and mutations.** Skrynia enforces grant
operations and scopes, quotas, create-if-absent, atomic single-key updates
and deletions, and optional old-byte SHA-256 preconditions even if a client
is malicious. Some application invariants can be expressed directly by
key layout and grants. For example, an exact-key create-only grant permits
at most one successful creation while no other authority deletes or
replaces that key. Higher-authority holders, including root, remain trusted.

**2. Client-verified acceptance.** An application may define which stored
records or state transitions count as *accepted*, using signatures,
hashes, histories, proofs, or deterministic predicates. For any computable
predicate with the needed evidence available, a correctly implemented
verifier can reject invalid data even from malicious publishers. This
provides safety of **accepted** state under the protocol's assumptions.
It does not itself guarantee availability, progress, freshness, protection
against history deletion, or a unique canonical history.

**3. Pre-storage application-specific validation.** The Skrynia server
does not evaluate arbitrary predicates over *proposed* application bytes.
To see the limit, fix a permitted PUT, a key and its existing bytes,
one grant, quota headroom, and an optional matching old-byte precondition.
Consider equal-length proposed values 0x00 and 0x01 under the rule
"the first byte must be zero." Both meet the same operation, scope,
size, and old-value-hash checks, but only one meets that content rule.
The server cannot reject just the second *on the basis of that rule*.
This is a limit of **direct content-predicate enforcement**, not a
limit on client verification or on encoding some rules in authority.

A trusted validator holding the only publication capability can check
arbitrary computable proposals before accepting them into canonical
storage, but is an additional application executor, not Skrynia server
logic. Independent cryptographic verification can instead let readers
reject invalid submissions without trusting such a validator. Neither
approach guarantees progress when the necessary participants disappear.

## Questions for each application protocol

Who may publish bytes? Who defines accepted state, and how do readers
verify it? Who may overwrite or delete evidence? How are stale or
contradictory histories detected? Must progress continue while clients
are offline? How are read capabilities generated, shared and rotated?

The [Cookbook](cookbook.md) gives concrete constructions and their
failure conditions. A safety argument must not silently substitute
"well-behaved writers" for enforcement against arbitrary authorized
writers.
