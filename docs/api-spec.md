# Skrynia Storage API Specification

## Endpoints

### Health check

```
GET /_skrynia/health
```

Response: `200 OK`
```json
{ "ok": true }
```

### Client library

```
GET /_skrynia/client/skrynia.js
```

Serves the installed browser client library. Returns `404` if not installed.

### Store operations

Base path: `/_skrynia/store/{namespace}/{key}`

#### GET - Read object

```
GET /_skrynia/store/{namespace}/{key}
```

Response: `200 OK`
- Body: raw object bytes
- `Content-Type`: from object metadata
- `X-Skrynia-Mode`: object mode
- `X-Skrynia-Created`: creation timestamp

Errors:
- `400 Bad Request`: invalid namespace, key, or percent-encoding
- `404 Not Found`: key does not exist

#### POST - Create object

```
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

The `capability` field is only present for `capability-write` mode.

Errors:
- `400 Bad Request`: invalid mode
- `409 Conflict`: key already exists
- `413 Payload Too Large`: object exceeds max size
- `507 Insufficient Storage`: namespace quota exceeded (detail: `quota_bytes` or `max_objects`)

#### PUT - Replace object

```
PUT /_skrynia/store/{namespace}/{key}
Content-Type: {object content type}
X-Skrynia-Capability: {capability}

{body}
```

Response: `200 OK`
```json
{ "ok": true }
```

Errors:
- `403 Forbidden`: immutable object, missing capability, or wrong capability
- `404 Not Found`: key does not exist
- `413 Payload Too Large`: replacement exceeds max size
- `507 Insufficient Storage`: replacement would exceed namespace quota (detail: `quota_bytes`)

#### DELETE - Remove object

```
DELETE /_skrynia/store/{namespace}/{key}
X-Skrynia-Capability: {capability}
```

Response: `200 OK`
```json
{ "ok": true }
```

Errors:
- `403 Forbidden`: immutable object, missing capability, or wrong capability
- `404 Not Found`: key does not exist

### App serving

```
GET /a/{namespace}/{path}
```

Serves static files from the currently active release for the namespace.
Falls back to `index.html` for directory requests.

Response: `200 OK` with static file content, or:
- `400 Bad Request`: invalid namespace or percent-encoding
- `403 Forbidden`: path traversal attempt
- `404 Not Found`: file not found
- `503 Service Unavailable`: no active release

## Object modes

### immutable
Object cannot be modified or deleted via public API. Use for static assets, versioned data.

### capability-write (default)
On create, server returns a 64-character hex capability string. Subsequent put/delete operations must include `X-Skrynia-Capability` header with this value. The capability is stored as a SHA-256 hash server-side; the plaintext capability is never stored.

### public-write
Anyone who knows the namespace and key may modify or delete the object. No capability required.

## Namespace validation

Namespaces must match `^[a-z0-9][a-z0-9_-]{0,63}$`:
- Lowercase alphanumeric, hyphens, underscores only
- Must start with a letter or digit
- 1-64 characters

## Key validation

- Keys must be non-empty strings
- Maximum length: 256 characters (configurable)
- No null bytes, no `..`, no leading `/`, no double slashes, no forward slashes
- Keys are URL-decoded before validation

## Namespace quotas

Each namespace has:
- `quotaBytes`: maximum total size of stored objects (default 10 MiB)
- `maxObjects`: maximum number of objects (default 10,000)
- `bytes`: current total size (recalculated on each mutation)
- `count`: current object count (recalculated on each mutation)

Quota enforcement happens atomically during create and put operations. PUT enforces the new total quota *before* writing, returning `507` if the replacement would exceed the limit.

## Error response format

All error responses use JSON:
```json
{ "error": "error_code" }
```

Some errors include additional detail:
```json
{ "error": "namespace_full", "detail": "quota_bytes" }
```
