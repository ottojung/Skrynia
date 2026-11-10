# Skrynia HTTP API

This specification is normative together with
[location-authority intent](intent-location-authority.md).

All paths are relative to the canonical SKRYNIA_URL, including its optional
path component. Credentials are unguessable bearer secrets; use HTTPS.

## Namespace bootstrap

A successful first authenticated GET /deploy creates the namespace root grant
after building and validating the static application. Parameters:
- repo: scp-like SSH Git URL
- commit: full verified commit SHA
- subdir: app directory within the repository
- namespace: namespace name
- token: deployment management token
- quota: optional root maxBytes, default 10485760
- maxObjects: optional root maxObjects, default 10000
- locations: optional root location matcher, default "*"
- operations: optional comma-separated root operations, default all operations
- builder: optional builder image override

The first successful deployment returns rootGrant.id, rootGrant.issuerToken,
rootGrant.locations, rootGrant.operations, the budget limits and nextCheckAt.
The issuer token is returned once. No user tokens are created during
deployment. Re-deployment retains the root grant without revealing its
issuer credential again.

The GitHub deployment action masks the issuer token and exports
root-grant-id and root-issuer-token outputs. The issuer may then create
user tokens using POST /grants/{id}/users.

## Keys

A key is a nonempty ASCII string matching:

~~~regex
^[A-Za-z0-9_]+$
~~~

and must not exceed SKRYNIA_MAX_KEY_LENGTH characters (default 256).
Case is significant.

A locations pattern must match:

~~~regex
^[A-Za-z0-9_]*\*?$
~~~

- "*" matches every valid key.
- "abc*" matches keys beginning with "abc", including "abc" itself.
- "abc" matches exactly "abc".
- "" matches no valid keys.

No escaping, separators, alternative patterns, or arbitrary expressions exist.

## Grants

A grant has an immutable namespace, ID, parent ID (null for root),
locations, operations, maxBytes, maxObjects, issuer-token verifier and
zero or more user-token verifiers. A new grant has no users. Grant state
also contains voting timestamps.

Valid operations:

| Operation | Authority |
|---|---|
| write | Create, replace, or delete objects at permitted locations |
| delegate | Create subgrants through this grant's user tokens |
| keep | Submit need:true |
| release | Submit need:false |
| subscribe | Create a browser push subscription |

Reads always require no operation and no token.

A child can only be issued by a user token from a grant containing delegate.
It inherits locations and operations if omitted. Explicit child locations
must denote a subset of the parent's matched keys, and child operations must
be a subset of its parent's. A grant with no operations is valid.

### POST /grants

~~~http
POST /grants
Authorization: Bearer <parent-user-token>
Content-Type: application/json

{
  "namespace": "forum",
  "locations": "reports*",
  "operations": ["write"],
  "maxBytes": 4096,
  "maxObjects": 1
}
~~~

Returns an ID, **issuerToken**, locations, operations, resource maxima
and nextCheckAt. No user token is generated or returned. The issuer token
must be retained securely; it authorizes issuance of user tokens and
unconditional cleaning/revocation.

maxBytes/maxObjects must be positive safe integers. Neither limit reserves
capacity in the parent. Grant metadata itself consumes the parent's budget.

### POST /grants/{id}/users?namespace=forum

~~~http
POST /grants/<grant-id>/users?namespace=forum
Authorization: Bearer <grant-issuer-token>
~~~

Requires the **issuer token of that exact grant**. Returns
`{"grantId":"...","userToken":"..."}`. Every call issues a distinct user
token with the operations specified by the grant, even if the grant lacks
`delegate`. A user token cannot issue other user tokens. The first token
fits within the grant's base metadata charge; subsequent tokens consume
additional metadata allowance.

### GET /grants/{id}?namespace=forum

Requires a user token for exactly that grant. Returns public grant properties,
userCount, and current location-scoped usage. Never returns token verifiers.

## Objects

All values are files, with no creator, owner or grant reference. Read
access is public, including direct nginx static-file serving.

### GET /store/{namespace}/{key}

Returns raw bytes and a strong ETag, or 404.

### POST /store/{namespace}/{key}

~~~http
POST /store/forum/reports1
X-Skrynia-Grant: <user-token>

<raw bytes>
~~~

Requires a grant whose operations include write and whose locations match
the key. Creates only if the key is absent. Returns 201 and {"ok":true}.
Object creation returns no other credential.

### PUT /store/{namespace}/{key}

~~~http
PUT /store/forum/reports1
X-Skrynia-Grant: <user-token>
If-Match: "<etag-if-needed>"

<replacement bytes>
~~~

Requires the write operation and key authorization. Must target an existing
object. If-Match is optional and returns 412 on ETag mismatch.

This operation is semantically atomic delete followed by create. The resulting
full object must fit every limit of the writing grant and its transitive
ancestors, after the old value is logically removed. A failed write preserves
the original object. There is no exception for a shrinking replacement whose
result is still over a grant's budget.

### DELETE /store/{namespace}/{key}

Requires a grant user token with write on the key. If-Match is supported.
Removes the object and releases its charged presence from location usage.
Does not require available quota.

## Resource accounting

Let matched(g) be all live objects at keys matching grant g's locations.

Bytes used by g:

~~~text
sum over matched(g) of (payloadBytes + 512)
+ 2048 per grant in g's inclusive descendant tree
+ 128 per additional user token beyond the first in that tree
+ 4096 per push subscription sponsored by that tree
~~~

Object count = count(matched(g)) + sponsored subscription count in subtree.
Grant metadata is charged in bytes, not in object count.

Only the **grant used for the mutation and its transitive ancestors** are
checked. Overlapping sibling grants are independent constraints on writes
using their respective tokens. A grant may measure usage beyond its own
maximum due to writes from an unrelated grant; it then cannot create an
object or perform an edit that still exceeds its own ceiling, but can delete
objects at its authorized locations.

No value is associated with a grant. Grant sponsorship remains recorded only
for non-object resources such as subscriptions.

## Cleanup and revocation

### POST /grants/{id}/confirmation?namespace=forum

~~~http
Authorization: Bearer <user-token>
Content-Type: application/json

{"need":true}
~~~

true requires keep, false requires release. At every 24-hour checkpoint, any
release signal for that grant triggers cleanup unless there was a keep signal
for that grant or any descendant in the same window. Total silence preserves
the state. A keep signal protects ancestors, not siblings.

### DELETE /grants/{id}/space?namespace=forum

Requires the grant's issuer token. Unconditionally cleans its space, by
revoking the grant and reissuing the exact same grant with identical ID,
parent, locations, operations, resource ceilings and user/issuer credentials,
but a new confirmation window and no descendants.

### DELETE /grants/{id}?namespace=forum

Requires the grant's issuer token. Revokes the grant without reissuance.

Both operations remove the grant's descendants and their credentials, and
push subscriptions sponsored by the revoked subtree. An object is deleted
only if (1) its key matches the target grant's locations and (2) **no
surviving independent grant** covers its key. Independent means neither
ancestor nor descendant of the target grant.

An ancestor does not protect its descendant's keys from that descendant's
cleanup. Descendants and the grant reissued during cleaning do not count as
independent protectors either. A sibling or another independent branch
protects matching keys even if it has no users, no `write` operation,
and no remaining quota. In particular, revoking or cleaning the root
deletes all objects in its scope.

This protection applies to cleanup/revocation, not to ordinary
`DELETE /store/{namespace}/{key}`, which remains authorized through `write`.
A surviving grant can still measure its preserved keys as occupied usage,
so cleanup need not restore an empty budget.

An independent-protection snapshot is persisted before any deletion, so
interrupted cleanup resumes with its original decisions. The deletion is
atomic at the HTTP interface; interrupted deletion is recovered before a
server can accept new requests.

## Browser push subscriptions

POST /push/subscriptions?namespace={ns}&channel={channel} requires a valid
grant user token in X-Skrynia-Grant with the subscribe operation, plus the
subscription request body. Each subscription is charged to its creating
grant and ancestors and removed with that grant's subtree. Subscription
update and removal use that subscription's own credential.

## Deployment and management routes

Authenticated deployment management routes include:
GET /deploy, /undeploy, /rollback, /releases, /inspect, /ns/inspect,
 /ns/list, and /push/rules/{set,get,list,remove}.

A single Skrynia server process owns one data directory.
