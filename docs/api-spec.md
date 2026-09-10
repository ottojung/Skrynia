# Skrynia HTTP API Specification

## Public endpoints

### Health check

```text
GET /_skrynia/health
```

Response: `200 OK`

```json
{ "ok": true }
```

### Client library

```text
GET /_skrynia/client/skrynia.js
```

Serves the browser client library.

### Store operations

Base path: `/_skrynia/store/{namespace}/{key}`

#### Read object

```text
GET /_skrynia/store/{namespace}/{key}
```

Response `200` body is the raw object bytes. Response headers include `Content-Type`, `X-Skrynia-Mode`, and `X-Skrynia-Created`.

#### Create object

```text
POST /_skrynia/store/{namespace}/{key}
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
PUT /_skrynia/store/{namespace}/{key}
Content-Type: {object content type}
X-Skrynia-Capability: {capability}

{body}
```

Response:

```json
{ "ok": true }
```

#### Delete object

```text
DELETE /_skrynia/store/{namespace}/{key}
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

## Management endpoints

Skrynia has no administrative CLI. All lifecycle/namespace administration is exposed over HTTP by the same server process.

Every management request is a `GET` under `/_skrynia/` and must contain a `token` query parameter exactly equal to `SKRYNIA_TOKEN`.

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
GET /_skrynia/deploy?repo={git-url}&commit={sha}&subdir={path}&namespace={ns}&token={token}
```

Optional parameter: `builder={docker-image}`.

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

### Undeploy

```text
GET /_skrynia/undeploy?namespace={ns}&token={token}
```

Deletes the active link, all releases, all stored namespace data, and namespace state.

```json
{ "ok": true, "namespace": "myapp" }
```

### Rollback

```text
GET /_skrynia/rollback?namespace={ns}&token={token}
GET /_skrynia/rollback?namespace={ns}&release={release-id}&token={token}
```

Without `release`, activates the previous release. With `release`, activates that exact retained release.

```json
{ "ok": true, "namespace": "myapp", "release": "20260910120000000-abc123" }
```

### Releases

```text
GET /_skrynia/releases?namespace={ns}&token={token}
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
GET /_skrynia/inspect?namespace={ns}&token={token}
```

Returns the saved deployment metadata for the namespace.

### Namespace create

```text
GET /_skrynia/ns/create?namespace={ns}&token={token}
GET /_skrynia/ns/create?namespace={ns}&quota={bytes}&token={token}
```

Creating an existing namespace preserves its existing quota.

### Namespace remove

```text
GET /_skrynia/ns/remove?namespace={ns}&token={token}
```

Removes namespace storage and state. This operation is distinct from undeploy and does not manage retained release directories.

### Namespace inspect

```text
GET /_skrynia/ns/inspect?namespace={ns}&token={token}
```

Returns namespace quota/usage plus deployment metadata when present.

### Namespace list

```text
GET /_skrynia/ns/list?token={token}
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
