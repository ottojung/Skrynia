# Skrynia

A minimal platform for deploying and serving small web apps with durable key-value storage.

## Features

- **Deploy from git over SSH**: Clone from an SSH scp-like repo URL, checkout exact commit, build via container, atomic release activation
- **Storage API**: Namespace-scoped key-value store with immutable, capability-write, and public-write modes
- **Web Push**: Durable mutation-driven push; exact namespace/key/kinds map to named channels, payload is exactly the channel name
- **App serving**: Static file serving from active releases under a configurable base path
- **HTTP management API**: Deployment, rollback, undeploy, release inspection, and namespace management relative to `SKRYNIA_URL`
- **Client library**: Tiny browser library served relative to `SKRYNIA_URL` at `/client/skrynia.js`

Skrynia has no administrative CLI. Management is performed through the HTTP server.

## Quick start

Skrynia has no reserved HTTP path prefix. `SKRYNIA_URL` is the complete externally visible root; all health, management, storage, and client-library endpoints are resolved relative to it. Examples in this README that use `/platform` assume `SKRYNIA_URL=http://127.0.0.1:17380/platform`; that path is illustrative only.

```sh
# Build the builder image (required for deploys)
make builder

# Configure the management token and start the server
SKRYNIA_URL=http://127.0.0.1:17380/platform SKRYNIA_TOKEN=replace-me node src/server.js

# Check health
curl http://127.0.0.1:17380/platform/health

# Deploy an app. Quote URLs containing '&'.
curl 'http://127.0.0.1:17380/platform/deploy?repo=git@github.com:myorg/myapp.git&commit=0123456789012345678901234567890123456789&subdir=.&namespace=myapp&token=replace-me'
```

## Management API

All management endpoints are `GET` requests relative to `SKRYNIA_URL` and require a `token` query parameter equal to `SKRYNIA_TOKEN`.

```text
GET /platform/deploy?repo=...&commit=...&subdir=...&namespace=...&token=...
GET /platform/undeploy?namespace=...&token=...
GET /platform/rollback?namespace=...&release=...&token=...
GET /platform/releases?namespace=...&token=...
GET /platform/inspect?namespace=...&token=...
GET /platform/ns/create?namespace=...&quota=...&token=...
GET /platform/ns/remove?namespace=...&token=...
GET /platform/ns/inspect?namespace=...&token=...
GET /platform/ns/list?token=...
GET /platform/push/rules/set?namespace=...&key=...&channels=...&on=...&token=...
GET /platform/push/rules/get?namespace=...&key=...&token=...
GET /platform/push/rules/list?namespace=...&token=...
GET /platform/push/rules/remove?namespace=...&key=...&token=...
```

`release`, `builder`, and `quota` are optional where shown. Deploy requires `repo`, `commit`, `subdir`, and `namespace`. `repo` must be an SSH Git URL in scp-like form `user@host:path` (e.g. `git@github.com:myorg/myapp.git`). Local paths, `file://`, `http://`, `https://`, `ssh://`, and other URL schemes are rejected. `commit` must be a full 40- or 64-character hexadecimal git object id, and Skrynia verifies that the checked-out HEAD exactly matches it.

If `SKRYNIA_TOKEN` is unset, management endpoints return `503 management_api_disabled`. Missing or incorrect tokens return `401 invalid_token`.

Management calls return JSON. A deployment request stays open while clone, build, validation, activation, and cleanup run.

## Deployment

Deploy process:

1. Validate SSH scp-like repo URL, namespace, exact commit id, and repository subdirectory.
2. Clone the repository into a disposable workspace under `DATA_DIR/builds/`.
3. Check out the exact commit and verify HEAD.
4. Run `make build` in the configured builder container.
5. Validate `build/` output and reject symlinks/special files.
6. Auto-create the namespace only after a valid build.
7. Atomically install the immutable release directory.
8. Atomically activate the release through one symlink.
9. Keep the three newest releases.

The builder container is writable and disposable (`--rm`). It may access the network. The repository is mounted read-write and npm is present in the default builder image.

### Included examples

```sh
curl 'http://127.0.0.1:17380/platform/deploy?repo=git@github.com:ottojung/Skrynia.git&commit=0123456789012345678901234567890123456789&subdir=example/hello&namespace=hello-app&token=replace-me'

curl 'http://127.0.0.1:17380/platform/deploy?repo=git@github.com:ottojung/Skrynia.git&commit=0123456789012345678901234567890123456789&subdir=example/birthday-list&namespace=birthday-list&token=replace-me'
```

`example/hello` is a minimal Makefile-driven app. `example/birthday-list` is an npm-built wishlist reservation app demonstrating Skrynia shared storage and create-if-absent reservation semantics.

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_URL` | (required) | Canonical externally visible root URL for all Skrynia HTTP endpoints; may contain any path component |
| `SKRYNIA_TOKEN` | (none) | Required token for all management HTTP endpoints |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Mutable state directory |
| `SKRYNIA_APP_BASE_PATH` | `/apps` | URL prefix for app serving (set to match your proxy) |
| `SKRYNIA_APP_DIR` | (none) | Filesystem dir for exposed app symlinks |
| `SKRYNIA_BUILDER_IMAGE` | `skrynia-builder:0.1.0` | Builder Docker image |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max object size |
| `SKRYNIA_PUSH_SEND_TIMEOUT_MS` | `10000` | Maximum time for one Web Push delivery attempt before retrying it as transient |

## Storage API

- `GET /platform/health` — health check
- `GET /platform/client/skrynia.js` — browser client library
- `GET /platform/store/{ns}/{key}` — read object
- `POST /platform/store/{ns}/{key}` — create object
- `PUT /platform/store/{ns}/{key}` — replace object
- `DELETE /platform/store/{ns}/{key}` — delete object
- `GET /platform/push/vapid` — VAPID public key
- `POST /platform/push/subscriptions?namespace={ns}&channel={ch}` — register a browser PushSubscription; replacing an existing endpoint requires its `X-Skrynia-Capability`
- `PUT /platform/push/subscriptions/{id}` — update a subscription with `X-Skrynia-Capability`
- `DELETE /platform/push/subscriptions/{id}` — delete a subscription with `X-Skrynia-Capability`
- `GET {base_path}/{ns}/{path}` — serve app static files

See [docs/api-spec.md](docs/api-spec.md) for the complete HTTP API.

## Architecture

- **Server**: Single-process Node.js HTTP server; storage and management operations share one process
- **Management**: Token-authenticated HTTP endpoints; there is no admin CLI or `admin.js`
- **Builder**: Published to GHCR from `builder/Dockerfile` (`node:20-alpine` + make + git + npm); containers are writable and disposable with `--rm`
- **Storage**: Filesystem-based; one `.dat`/`.meta`/`.cap` triplet per object per namespace
- **Releases**: Immutable directories under `RELEASES_DIR/{ns}/`; atomic symlink swap for activation
- **APP_DIR**: Optional external exposure; `APP_DIR/{ns}` symlink points directly to the active release

## Running tests

```sh
make test
```

## License

See repository.
