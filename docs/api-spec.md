# Skrynia HTTP API

This is the **grant-only** storage model. Older namespace quota files,
unassociated objects and subscriptions are unsupported. There is no migration
path or compatibility API; initialize a new data directory for this release.

All routes below are relative to `SKRYNIA_URL`, which may include a path
component. Grant credentials are bearer secrets; use HTTPS, do not place them
in URLs, and store them securely. No token can be recovered from its verifier.

## Deployment and root grant

`GET /deploy?repo=git@github.com:org/app.git&commit=<full-sha>&subdir=.&namespace=app&token=<management-token>`

A successful first deployment validates and builds the app, then creates a
root grant for its namespace. Optional `quota` and `maxObjects` parameters
set that root's limits; otherwise defaults are taken from
`SKRYNIA_DEFAULT_QUOTA_BYTES` (10 MiB) and
`SKRYNIA_MAX_OBJECT_COUNT` (10,000).

The first deploy response includes:
```json
{
  "ok": true, "namespace": "app", "release": "example", "path": "/apps/app/",
  "rootGrant": {
    "id": "32-hex-id",
    "created": true,
    "issuerToken": "64-hex-secret",
    "userToken": "64-hex-secret",
    "maxBytes": 10485760,
    "maxObjects": 10000,
    "nextCheckAt": 0
  }
}
```

`nextCheckAt` is a Unix timestamp in **milliseconds** (the example above is
a placeholder). Later deployments preserve the root grant and its contents
and return `{"rootGrant":{"created":false,"id":"..."}}`, never its secrets.
The bundled GitHub deploy action masks and exports the credentials as
`root-user-token` and `root-issuer-token` outputs, only on first creation.

No ordinary storage or grant creation works without an existing grant.
`/ns/create` and `/ns/remove` have been removed.

## Grant authority

Every grant belongs to exactly one namespace and has one parent, except the
namespace root. Every grant has an **issuer token** and one or more **user
tokens**. Tokens are separate random bearer secrets.

- An issuer token can **clean the grant's entire space** or **revoke the
  grant entirely**, immediately and unconditionally; it cannot create
  objects, create subgrants, mint user tokens or vote.
- A user token can create objects funded by the grant, create subgrants,
  mint additional user tokens for that grant, and confirm `need` or
  `release`. It cannot unilaterally revoke the grant.
- Existing object write capabilities remain separate: grant user tokens
  authorize *allocation*, not modification of an already-created object.

### Create a subgrant

```http
POST /grants
Authorization: Bearer <parent-user-token>
Content-Type: application/json

{"namespace":"app","maxBytes":5242880,"maxObjects":100}
```

Response (200) contains `id`, `issuerToken`, `userToken`, `maxBytes`,
`maxObjects`, `nextCheckAt`.

The child shares its parent's namespace. A parent does **not** reserve its
whole maximum for each child. Actual consumption of every subtree must
remain under the limits of **every ancestor**. Sibling grants can share
unused headroom and can also compete for it.

### Mint an additional user token

```http
POST /grants/<id>/users?namespace=app
Authorization: Bearer <existing-user-token>
```

Returns `{"grantId":"...","userToken":"..."}`.
This consumes grant metadata allowance. Each token represents an independent
bearer credential, not necessarily a human identity.

### Inspect a grant

```http
GET /grants/<id>?namespace=app
Authorization: Bearer <user-token>
```

Returns public grant fields and current subtree `usage`, excluding token
verifiers. It is usable only by a user token for that exact grant.

## Store

Every newly created object has a funding grant:

```http
POST /store/app/my-key
X-Skrynia-Grant: <grant-user-token>
X-Skrynia-Mode: capability-write

<arbitrary bytes>
```

Returns `201` and a one-time per-object write capability (default mode).
`X-Skrynia-Mode: public-write` is also supported. No anonymous creation,
no immutable mode and no legacy object formats.

- `GET /store/{namespace}/{key}` returns raw bytes with a strong ETag,
  without needing a token.
- `PUT /store/{namespace}/{key}` requires the object's write capability
  when in `capability-write` mode. `If-Match: <etag>` is an optional
  compare-and-replace precondition. Growth is charged to the object's
  original grant and every ancestor, even if a different writer holds the
  object's write capability.
- `DELETE /store/{namespace}/{key}` requires that same object capability
  when applicable and removes the object and its private metadata.

ETag versions use a single monotonic namespace counter, so deletion does not
leave unbounded per-key tombstones, yet delete/recreate cannot reuse an ETag.
Files remain nginx-static-file compatible.

## Resource accounting

Every subtree must obey:

```text
sum(object byte lengths)
+ 512 bytes per stored object (reserved metadata)
+ 2,048 bytes per grant (including the root)
+ 128 bytes per additional grant user token
+ 4,096 bytes per browser PushSubscription
<= maxBytes
```

`maxObjects` counts stored objects and browser push subscriptions in the
subtree. Grants themselves use byte allowance, not object-count allowance.
These charges are conservative upper bounds on private records; no recursive
creation is free. A root grant can have only the resources established at
deployment. There is no separate namespace quota layer.

## Confirmation-based cleanup

Every grant has a 24-hour checkpoint period. Each user token may submit:

```http
POST /grants/<id>/confirmation?namespace=app
Authorization: Bearer <grant-user-token>
Content-Type: application/json

{"need":true}
```

Or `{"need":false}` to propose cleanup. Signals accumulate within the
period. Grant user tokens can be shared and need not represent persons.

At its checkpoint, a grant is **cleaned** exactly if:
1. At least one negative confirmation was recorded for the grant during
   its period; and
2. **No** positive confirmation was recorded during that period for this
   grant **or any descendant**.

Thus one descendant's `need:true` protects all its ancestors. Positive
confirmations do not protect sibling or descendant grants against their own
independently authorized cleanup. Total silence preserves the space.
Checks are performed in the running server and after the next request following
a delay/restart. A server restart does not imply a vote.

A **clean** deletes every object funded by the target or descendants, and
all descendant grants and their tokens. The target grant itself, its user and
issuer tokens, and its limits survive, and can be reused immediately.

### Issuer operations

```http
DELETE /grants/<id>/space?namespace=app
Authorization: Bearer <issuer-token>
```

Unconditionally cleans the space while preserving that grant.

```http
DELETE /grants/<id>?namespace=app
Authorization: Bearer <issuer-token>
```

Unconditionally revokes the grant itself and the entire subtree, including
all objects, descendants, and tokens. It overrides all confirmations.

Both deletion operations persist their complete affected subtree in a private
pending-deletion record first and resume an interrupted deletion on process
restart. Operations are synchronous for the single server process; no
concurrent writes can recreate doomed objects during deletion.

## Push, management and deployment

The PushSubscription creation endpoint
`POST /push/subscriptions?namespace={ns}&channel={channel}` additionally
requires `X-Skrynia-Grant: <user-token>` for new subscriptions. Subscriptions
retain independent update/delete capabilities and are deleted when their
funding grant is cleaned. Other push endpoints and management endpoints
remain documented in the README. Management authentication still uses
`SKRYNIA_TOKEN`.

The node process must remain the single owner of its data directory.
This implementation offers no distributed transaction processing and no
in-place upgrade path for existing storage data.
