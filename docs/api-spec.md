# Skrynia HTTP API

The paths below are relative to `SKRYNIA_URL`, including its optional URL
path prefix. All mutation credentials are bearer secrets in request headers.
Use HTTPS outside local development. No API request relies on cookies.

## Object names and authority scopes

An object key must be a nonempty ASCII string, at most
`SKRYNIA_MAX_KEY_LENGTH` characters (default 256), matching
`[A-Za-z0-9_][A-Za-z0-9_.]*`. Keys cannot begin with a dot;
consecutive dots are unrestricted after the first character.
Case matters; `/`, spaces and `*` are not valid object-key characters.

A `locations` matcher is an exact key (`alice.index.html`),
a prefix followed by `*` (`alice.*`), `*` for all keys, or
`""` for no keys. Matchers use the same non-dot initial character as keys, with an optional
trailing `*`; `*` and `""` are also valid matchers.
A child grant's locations must select a subset of its parent's matched keys.

## Object endpoints

| HTTP method | Path | Semantics |
| --- | --- | --- |
| GET / HEAD | /store/{key} | Public raw bytes or metadata, strong ETag, conditional / range reads; 404 if absent |
| GET | /store?locations={matcher} | List matching keys, requires a user token with `list` |
| POST | /store/{key} | Create if absent; requires `create` and authorized user token; 201 or 409 |
| PUT | /store/{key} | Replace if present; requires `modify` and authorized user token; 200 or 404 |
| DELETE | /store/{key} | Delete if present; requires `delete` and authorized user token; 200 or 404 |

Mutations and listings use `Authorization: Bearer <user-token>`. Both PUT and DELETE accept an
optional `If-Match: "<etag>"` and return HTTP 412 if the current ETag
does not match. The strong ETag changes on every successful object mutation,
including an identical-byte PUT and deletion followed by recreation.
Objects are files under `SKRYNIA_DATA_DIR/store/{key}`; the server's ETag
is in nginx's `"hex-mtime-seconds-hex-size"` format. Storage journals and
temporary files live outside the public store.

GET supports `If-None-Match` (304 without body), and `HEAD` returns
metadata without the object body. Single-range GET requests (`Range:
bytes=start-end`), including suffix ranges, return 206 with `Content-Range`,
or 416 for unsatisfiable ranges. A matching strong `If-Range` ETag authorizes
a partial response; a stale validator returns the full current object.

The GET response uses the filename extension to select common browser MIME
types (including `.html`, `.js`, `.mjs`, `.css`, `.json`, `.svg`,
`.png`, and `.wasm`). Unknown extensions produce
`application/octet-stream`. Static servers serving the same directory must
configure MIME types consistently. A GET uses `Cache-Control: no-cache`
and `X-Content-Type-Options: nosniff`.

Object writes commit atomically per key. A failed conditional mutation
preserves the existing object. The request/body maximum
(`SKRYNIA_MAX_OBJECT_SIZE`, default 10 MiB) is independent of grant quota.
The server also bounds aggregate in-progress buffered uploads
(`SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES`, default 40 MiB) and HTTP connections
(`SKRYNIA_MAX_CONNECTIONS`, default 512). Uploads exceeding the process
buffer limit receive HTTP 503. Public reads stream from a pinned file
descriptor rather than copying the entire object into memory.

**nginx ETag compatibility is mandatory:** the original whole-second mtime
sequence and the standard nginx `"hex-mtime-seconds-hex-size"` ETag format
are unchanged. High write rates can move stored mtimes ahead of wall time;
clients must treat ETags as opaque validators.

### Scoped key listing

`GET /store?locations=alice.*` returns JSON like
`{"keys":["alice.app.js","alice.index.html"]}`. The requester supplies
its user token in `Authorization: Bearer`; the server checks the `list` operation
and requires `locations` to be a non-strict subset of the token's grant
scope. Exact matches, prefix matchers, `*` (all), and the empty matcher
(no keys) follow the same grammar as grant locations.

Keys are sorted, and every currently stored object matching the supplied
matcher is returned, **regardless of which grant created it**. Optional
`limit` (integer, 1–1000) and `after` (exclusive key cursor) parameters
allow bounded listing. A non-final page includes `next` holding the last
returned key; request the next page with `after=<next>`. Without a `limit`,
the server returns at most 1000 keys per page; clients must follow `next`
to enumerate larger ranges. Pagination does not
create a snapshot across requests; concurrent insertions before the cursor
may be missed. Use manifests for consistent histories. Replacement
and deletion immediately affect the result. A missing, duplicated, or
invalid matcher yields HTTP 400; a matcher outside the caller's authority
yields HTTP 403. Listing is not available to anonymous callers. There is
no separate privileged enumeration API or creator-tracking metadata.

A listing is a snapshot of the keys at request time, not a lock on a range;
clients should account for concurrent creations and deletions. To clean up a
subgrant's storage, use exclusive prefixes; a shared prefix may contain keys
written by other grants. Listing confers no deletion permission. Large ranges
return large responses, so callers should narrow their matcher as needed.

## Grants and credentials

All grants form one rooted tree. Each grant has:

- `id`, `parent` (null for root), `locations`, `operations`
- `maxBytes` and `maxObjects` (positive safe integers for child grants)
- an issuer credential and zero or more independently revocable user credentials

Operations: `create`, `modify`, `delete`, `list`, `delegate`, and `subscribe`.

- `create` authorizes `POST` to a key that does not exist, including its
  initial binary payload. It does not authorize `PUT` or `DELETE`.
- `modify` authorizes `PUT` to an existing key, including changing its
  bytes arbitrarily (subject to quota and optional `If-Match`). It does
  not authorize creation or deletion.
- `delete` authorizes `DELETE` of an existing key; deletion does not
  require `create` or `modify`.
- `list` permits enumerating existing keys in a requested location range
  that is a non-strict subset of the calling grant's `locations`.
- `delegate` permits child grants; `subscribe` permits Web Push subscriptions.
- Reading a known object's bytes remains public and unauthenticated.

Listing is authorized by a **grant user token**, not an issuer token.
A grant with `list` but without mutation operations can discover keys
without changing data. All six operations can be delegated independently.

On each server start, after completing recovery, if the tree has no root,
Skrynia creates one with `locations:"*"`, all operations and
`maxBytes:null,maxObjects:null`. Null means unlimited **only for the root**.
The root issuer token is stored in its private 0600 grant record, accessible
using a local operator command; it is never returned by HTTP.

| HTTP method | Path | Auth | Meaning |
| --- | --- | --- | --- |
| POST | /grants | Parent user token in Authorization | Create child grant |
| GET | /grants/{id} | Exact grant user token in Authorization | Inspect grant and current usage |
| POST | /grants/{id}/users | Exact issuer token in Authorization | Issue a user token |
| DELETE | /grants/{id}/users/{userId} | Exact issuer token in Authorization | Revoke one user token |
| DELETE | /grants/{id} | Exact issuer token in Authorization | Revoke grant subtree |

Example:

```http
POST /grants
Authorization: Bearer <parent-user-token>
Content-Type: application/json

{"locations":"alice.*","operations":["create","modify","delete","delegate"],
 "maxBytes":65536,"maxObjects":10}
```

The response contains the new grant's ID and one-time issuer token; it
does **not** generate a user token. `delegate` must be present in the
parent's operations. A child may narrow locations and operations, never widen
them. Child limits do not reserve capacity in ancestors.

Issuing a user token returns `grantId`, `userId`, and `userToken`.
Credentials are independent: user tokens cannot issue more user tokens,
while issuer tokens cannot write objects. The issuer token can revoke its
own grant, including the root. Revocation invalidates all subordinate tokens
and removes sponsored push subscriptions, **but never deletes object values**.
A missing root is recreated on a subsequent server start, with fresh tokens.

### Resource accounting

For grant g:

```text
bytesUsed(g) =
  sum over currently matching objects of (payloadBytes + 512)
  + 2048 per grant in the inclusive descendant tree
  + 128 per extra user token beyond the first in that tree
  + 4096 per sponsored push subscription in that tree

objectsUsed(g) =
  count of matching stored objects
  + count of sponsored push subscriptions in its descendant tree
```

Only the grant through which a creation or modification occurs and all its
ancestors have their resulting limits checked. Unrelated grants cannot veto the action.
An overlapping grant can measure usage beyond its limit because writes
through independent grants are not constrained by its budget. Objects
have no writer, grant owner, or expiration.

A PUT is charged against the full resulting object value, with the old
value logically removed first. DELETE needs no quota headroom.
Creation of new grants and user credentials consumes ancestor metadata budget.

## Grant-scoped Web Push

`GET /push/vapid` returns the server's public VAPID key.

`POST /push/subscriptions`, with `Authorization: Bearer <user-token>`:

```json
{
  "locations": "alice.*",
  "events": ["create", "replace", "delete"],
  "endpoint": "https://push-service.example/subscription",
  "keys": {"p256dh":"...","auth":"..."}
}
```

The `subscribe` operation is required and the subscription location
matcher must be a subset of the grant's locations. `events` may be omitted,
defaulting to all three mutation kinds. The server returns `id`,
`capability`, and `deduped`. The same endpoint can have independent
registrations for different scopes or events. A registration for the same
endpoint, scope, and events may be refreshed with its own capability.

`PUT /push/subscriptions/{id}` changes an endpoint or cryptographic keys;
`DELETE /push/subscriptions/{id}` removes it. These two routes require
`X-Skrynia-Capability`. A subscription's locations/events are immutable.
A grant revocation removes subscriptions funded by its subtree.

Subscriptions are private server records. Matching object mutations enqueue
durable at-least-once wake-ups, **coalescing multiple pending changes into
one notification per subscription**. If an update occurs during delivery,
its newer notification remains pending. Push messages carry the constant
`changed` and no object data. Delivery is retryable and duplicate wake-ups
are possible. A full notification outbox rejects a mutation needing new
entries before committing it, with HTTP 507. Deliveries have configurable
timeouts and bounded concurrency (default eight simultaneous sends).
Endpoints must use HTTPS; outbound DNS answers are validated and pinned
to publicly routable addresses.

## Other endpoints

`GET /health`, `GET /version`, and `GET /client/skrynia.js` are public.

For browser applications, the API provides
`Access-Control-Allow-Origin: *`, CORS preflight, and exposure of the ETag
header. Mutations require explicit bearer credentials, not browser cookies.

The filesystem is authoritative; object-size indexes are rebuilt on startup.
Skrynia requires a **single writer process per data directory**, and
does not offer physical cross-key ACID transactions. See
[client-side protocols](client-protocols.md) for how to build atomic logical
multi-object transactions using immutable files and one CAS commit point.
