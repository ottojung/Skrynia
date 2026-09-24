# Skrynia HTTP API Specification

All endpoints in this document are relative to `SKRYNIA_URL`. `SKRYNIA_URL` is the complete externally visible root and may contain any path component; Skrynia adds no reserved prefix.

## Public endpoints

### Health check

```text
GET {SKRYNIA_URL}/health
```

Response: `200 OK`

```json
{ "ok": true }
```

### Running version

```text
GET {SKRYNIA_URL}/version
```

Response: `200 OK`

```json
{
  "version": "1.0.0-122-g9829feb",
  "commit": "9829febf3466d6668f511ca46c867fb03f87a774"
}
```

The values identify the server image currently executing. Published images bake
the existing `git describe` release version and exact source commit into the
image at build time. This endpoint is public and requires no management token.
Source-tree development without baked image metadata reports a distinct
`development` fallback.

Only `GET` is accepted; other methods return `405`.

### Client library

```text
GET {SKRYNIA_URL}/client/skrynia.js
```

Serves the browser client library.

### Store operations

Base path: `{SKRYNIA_URL}/store/{namespace}/{key}`

#### Read object

```text
GET {SKRYNIA_URL}/store/{namespace}/{key}
```

Response `200` body is the raw object bytes. Response headers include `Content-Type`, `X-Skrynia-Mode`, `X-Skrynia-Created`, and `ETag`. The ETag is an opaque quoted tag derived deterministically from the exact object bytes only; rereading unchanged bytes returns the same ETag, and changed bytes return a different ETag. It does not cover object metadata or storage identity.

#### Create object

```text
POST {SKRYNIA_URL}/store/{namespace}/{key}
Content-Type: {object content type}
X-Skrynia-Mode: {immutable|capability-write|public-write}

{body}
```

Response: `201 Created`

```json
{
  "ok": true,
  "mode": "capability-write",
  "capability": "64-char-hex-string"
}
```

The capability is returned only for `capability-write` mode.

#### Replace object

```text
PUT {SKRYNIA_URL}/store/{namespace}/{key}
Content-Type: {object content type}
X-Skrynia-Capability: {capability}
If-Match: {etag} (optional)

{body}
```

Response:

```json
{ "ok": true }
```

`PUT` without `If-Match` is an unconditional replacement. With `If-Match`, the server compares the supplied ETag with the exact current object bytes as an atomic compare-and-replace. A mismatch returns `412` with `{"error":"etag_mismatch"}` and does not change the object. Capability authorization is independently required and is checked before replacement. Competing conditional writers using the same current ETag cannot both succeed.

#### Delete object

```text
DELETE {SKRYNIA_URL}/store/{namespace}/{key}
X-Skrynia-Capability: {capability}
```

Response:

```json
{ "ok": true }
```

Store errors include:

- `400`: invalid namespace, key, percent encoding, or mode
- `403`: immutable object, missing capability, or wrong capability
- `404`: object or namespace does not exist
- `409`: create conflicts with an existing object or namespace has not been created
- `413`: object/request exceeds the configured maximum
- `507`: namespace byte/object quota exceeded

### App serving

```text
GET {base_path}/{namespace}/{path}
```

`base_path` defaults to `/apps` and is configurable via `SKRYNIA_APP_BASE_PATH`. There is no canonical application URL prefix.

Static files are served from the currently active release. Directory requests use `index.html` when present.

### Web Push

Generic durable push driven by store mutations. A management-owned rule maps
an exact namespace + exact key + selected mutation kinds
(`create`, `replace`, `delete`) to one or more named channels. A matching
mutation durably persists one outbox item per channel before the mutation
becomes committed/visible, then the server delivers the channel name
asynchronously. The push payload is exactly the channel name.

#### VAPID public key

```text
GET {SKRYNIA_URL}/push/vapid
```

Response:

```json
{ "publicKey": "base64url-encoded-key" }
```

#### Register a subscription

```text
POST {SKRYNIA_URL}/push/subscriptions?namespace={ns}&channel={channel}
Content-Type: application/json

{ "endpoint": "https://push.example/...", "keys": { "p256dh": "...", "auth": "..." } }
```

Response `201 Created`:

```json
{
  "ok": true,
  "id": "32-char-hex-string",
  "capability": "64-char-hex-string",
  "deduped": false
}
```

The `id` is opaque and the `capability` is bearer authority for update/delete.
Re-registering an endpoint that already exists for the same namespace/channel
is a replacement operation: it must present the current
`X-Skrynia-Capability` header. An authorized re-registration returns the same
id with a fresh capability and `"deduped": true`; the previous capability is
invalidated. There is no public endpoint that enumerates subscriptions. A full
channel returns `507 channel_full`; a namespace that already holds 2000
subscriptions returns `507 namespace_full`.

#### Update a subscription

```text
PUT {SKRYNIA_URL}/push/subscriptions/{id}
X-Skrynia-Capability: {capability}
Content-Type: application/json

{ "endpoint": "https://push.example/...", "keys": { "p256dh": "...", "auth": "..." } }
```

Either or both of `endpoint`/`keys` may be supplied. The capability travels
in the `X-Skrynia-Capability` header (as with store object capabilities),
never in the URL. Response: `{ "ok": true }`.

#### Delete a subscription

```text
DELETE {SKRYNIA_URL}/push/subscriptions/{id}
X-Skrynia-Capability: {capability}
```

Response: `{ "ok": true }`.

Push errors include `400` (invalid namespace, channel, or subscription),
`403` (missing/wrong capability), `404` (unknown id or namespace),
`409 endpoint_in_use` (an update would duplicate another registration in the
same namespace/channel), `413` (request too large), and `507` (channel or
namespace subscription quota exceeded).

When the durable delivery outbox is full, a matching store mutation is not
committed and fails with `507 push_outbox_full` instead of committing
without notification state; retrying the mutation after the outbox drains
succeeds. Unexpected enqueue I/O failures return `500 push_enqueue_failed`.

## Management endpoints

Skrynia has no administrative CLI. All lifecycle/namespace administration is exposed over HTTP by the same server process.

Every management request is a `GET` under `{SKRYNIA_URL}/` and must contain a `token` query parameter exactly equal to `SKRYNIA_TOKEN`.

If `SKRYNIA_TOKEN` is unset, management requests return:

```http
503 Service Unavailable
```

```json
{ "error": "management_api_disabled", "detail": "SKRYNIA_TOKEN is not configured" }
```

Missing or incorrect tokens return `401` with `{"error":"invalid_token"}`.

### Deploy

```text
GET {SKRYNIA_URL}/deploy?repo={ssh-url}&commit={sha}&subdir={path}&namespace={ns}&token={token}
```

Optional parameter: `builder={docker-image}`.

`repo` must be an SSH Git repository URL in canonical scp-like form `user@host:path` (for example `git@github.com:myorg/myapp.git`). Local paths, `file://`, `http://`, `https://`, `ssh://`, and other URL schemes are rejected.

Required constraints:

- `commit` is a full 40- or 64-hex git object id;
- `subdir` is relative and cannot contain `..`;
- namespace matches the normal namespace grammar.

The request is synchronous and returns after clone, build, validation, release activation, and cleanup.

Successful response:

```json
{
  "ok": true,
  "namespace": "myapp",
  "release": "20260910120000000-abc123",
  "path": "/apps/myapp/"
}
```

Errors:

- `400 invalid_repo`: repo is not a valid SSH scp-like URL

### Undeploy

```text
GET {SKRYNIA_URL}/undeploy?namespace={ns}&token={token}
```

Deletes the active link, all releases, all stored namespace data, and namespace state.

```json
{ "ok": true, "namespace": "myapp" }
```

### Rollback

```text
GET {SKRYNIA_URL}/rollback?namespace={ns}&token={token}
GET {SKRYNIA_URL}/rollback?namespace={ns}&release={release-id}&token={token}
```

Without `release`, activates the previous release. With `release`, activates that exact retained release.

```json
{ "ok": true, "namespace": "myapp", "release": "20260910120000000-abc123" }
```

### Releases

```text
GET {SKRYNIA_URL}/releases?namespace={ns}&token={token}
```

```json
{
  "namespace": "myapp",
  "current": "20260910120000000-abc123",
  "releases": ["20260910110000000-def456", "20260910120000000-abc123"]
}
```

### Deployment inspection

```text
GET {SKRYNIA_URL}/inspect?namespace={ns}&token={token}
```

Returns the saved deployment metadata for the namespace.

### Namespace create

```text
GET {SKRYNIA_URL}/ns/create?namespace={ns}&token={token}
GET {SKRYNIA_URL}/ns/create?namespace={ns}&quota={bytes}&token={token}
```

Creating an existing namespace preserves its existing quota.

### Namespace remove

```text
GET {SKRYNIA_URL}/ns/remove?namespace={ns}&token={token}
```

Removes namespace storage and state. This operation is distinct from undeploy and does not manage retained release directories.

### Namespace inspect

```text
GET {SKRYNIA_URL}/ns/inspect?namespace={ns}&token={token}
```

Returns namespace quota/usage plus deployment metadata when present.

### Namespace list

```text
GET {SKRYNIA_URL}/ns/list?token={token}
```
Returns:

```json
{
  "namespaces": [
    {
      "namespace": "myapp",
      "quota": { "bytes": 0, "count": 0, "quotaBytes": 10485760, "maxObjects": 10000 },
      "deployment": null
    }
  ]
}
```

### Push rule set

```text
GET {SKRYNIA_URL}/push/rules/set?namespace={ns}&key={key}&channels={a,b}&on={create,replace,delete}&token={token}
```

Maps one exact namespace + exact key plus the selected mutation kinds to one
or more named channels. `channels` is a comma-separated list of channel names
(`^[a-z0-9][a-z0-9_-]{0,63}$`, at most 8 per rule). `on` is a
comma-separated subset of `create,replace,delete` and defaults to all three.
Setting a rule for an existing key replaces it.

```json
{ "ok": true, "rule": { "key": "orders", "kinds": ["create", "replace"], "channels": ["shop"] } }
```

### Push rule get

```text
GET {SKRYNIA_URL}/push/rules/get?namespace={ns}&key={key}&token={token}
```

Returns `{ "namespace": "myapp", "rule": { ... } }`, or `404 push_rule_not_found`.

### Push rule list

```text
GET {SKRYNIA_URL}/push/rules/list?namespace={ns}&token={token}
```

Returns `{ "namespace": "myapp", "rules": [ ... ] }`.

### Push rule remove

```text
GET {SKRYNIA_URL}/push/rules/remove?namespace={ns}&key={key}&token={token}
```

Returns `{ "ok": true, "namespace": "myapp", "key": "orders" }`.

## Object modes

### immutable

Object cannot be modified or deleted through the store API.

### capability-write (default)

Create returns a 64-character hexadecimal capability. PUT/DELETE require that value in `X-Skrynia-Capability`. Only its SHA-256 verifier is stored server-side.

### public-write

Anyone who knows namespace and key may replace or delete the object.

## Namespace validation

Namespaces match `^[a-z0-9][a-z0-9_-]{0,63}$`.

## Key validation

- non-empty;
- maximum configured length (default 256);
- no NUL, `..`, leading slash, double slash, or slash;
- URL-decoded before validation.

## Namespace quotas

Each namespace records `quotaBytes`, `maxObjects`, current `bytes`, and current `count`. Usage is recalculated from committed object metadata before mutations/inspection.

## Error response format

Errors are JSON:

```json
{ "error": "error_code" }
```

and may include `detail`:

```json
{ "error": "invalid_commit", "detail": "commit must be a full 40 or 64 hex git object id" }
```
