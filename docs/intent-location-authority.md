# Grant authority and resource accounting

Status: normative.

## One root and one keyspace

Every installation has one global object keyspace. Objects are named
by one ASCII key matching `[A-Za-z0-9_][A-Za-z0-9_.]*`. Keys cannot
begin with a dot but may contain any number of subsequent dots. They are
bounded by a configurable maximum length,
and are not hierarchical filesystem paths.

A grant's location matcher is an exact key, a prefix ending in `*`,
all keys (`*`) or no keys (empty string). Every grant holds an immutable
location matcher and an immutable subset of operations.

On startup, after recovering pending transactions and grant revocations,
the server creates an unrestricted root grant if none exists. Its
`locations` is `*`, its operations are all operations, and its byte
and object limits are unbounded (represented by JSON null). Its issuer
credential is stored in its private, atomic root grant record for operator
retrieval. Root revocation is valid and deletes the root record and
all descendant grant records, but not the values. No root is created
until the next startup.

## Authority

Valid operations are `create`, `modify`, `delete`, `list`, `delegate`,
and `subscribe`. Reading a known object's bytes requires no grant.
`create` permits creating a nonexistent key and supplying its initial bytes.
`modify` permits replacing an existing key's bytes. `delete` permits
deletion of an existing key. No mutation permission implies another:
`create` alone does not authorize replacement or removal. `list`
enables enumeration of
all live keys matching a requested location matcher that is a non-strict
subset of the calling grant's locations. Listings are independent of
which grant created each object. `delegate` enables issuing a child
grant under a parent user token. A child cannot enlarge the parent's
locations or operation set. It may have fewer or no operations.

An issuer token is independent from user tokens. Each new grant returns
an issuer token; no user token is issued automatically. The issuer may
mint separate user tokens and revoke them independently. A user token
cannot issue more user tokens. An issuer token cannot write objects.
The issuer can revoke the whole grant subtree, including the root.
Grant revocation removes descendant subscription records but leaves all
object bytes unchanged.

## Budgets

Child grants have positive finite integer `maxBytes` and `maxObjects`
limits; the root is unbounded. Grant limits constrain transitions through
the grant and its ancestors, not every independent mutation of shared
object state. No grant owns an object's lifetime.

For a grant g, bytes used are the sum of payload sizes plus 512 bytes per
matching object, 2048 bytes per grant in its inclusive descendant tree,
128 bytes per additional user credential beyond the first in that tree,
and 4096 bytes per sponsored push subscription. Object count is the count
of matching objects plus sponsored descendant subscriptions.

Before creation or modification, Skrynia checks the resulting usage against
the acting grant and all its ancestors. The previous object is logically removed
before a replacement is checked. An unrelated grant with overlapping
locations is never consulted. A grant can therefore observe usage
higher than its own maximum after an unrelated grant's mutation.
A delete is permitted even if usage exceeds the limit.

Creating grants and user credentials consumes metadata quota in ancestors.
Subscription creation checks the `subscribe` operation and scope subset,
then charges sponsored subscriptions to the creating grant and ancestors.
Subscriptions are removed when the sponsoring grant is revoked.

## Consistency and trust

Individual object creations, replacements and deletions have one atomic
filesystem commit point. ETags are opaque, monotonic representation
versions, compatible with nginx mtime-size ETags. If-Match implements
single-object conditional mutation. A sequence of immutable version
objects can be published atomically as one *logical* transaction by
CAS on a single root object. Clients must keep referenced versions intact
and reevaluate failed CAS operations; see
[client-side protocols](client-protocols.md).

Object reads are public. Applications wishing to authenticate contents,
enforce higher-level acceptance policies or encrypt data implement these
properties in client protocols and authority distribution. Authentication
of data is distinct from guaranteed storage availability.

Skrynia sends grant-authorized, location-scoped Web Push wake-ups with
at-least-once delivery; notification payloads reveal no keys or bytes.
Storage mutation intents and subtree revocation intents are durable and
recovered on process startup. One process owns a data directory.
