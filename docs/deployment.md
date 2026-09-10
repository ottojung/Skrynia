# Skrynia Deployment Guide

## Prerequisites

- Node.js 20+ on the server
- Docker (for builder container)
- git (for cloning app repositories)

## Configuration

Skrynia is configured via environment variables or options:

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Mutable state: releases, storage, namespace state |
| `SKRYNIA_APP_BASE_PATH` | `/apps` | URL prefix for app serving (set to match your proxy) |
| `SKRYNIA_APP_DIR` | (none) | Filesystem dir for exposed app symlinks (for external web servers) |
| `SKRYNIA_BUILDER_IMAGE` | `skrynia-builder:0.1.0` | Docker image for building apps |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota (10 MiB) |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Default max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length in characters |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max single object size (10 MiB) |

When `SKRYNIA_APP_DIR` is set, the admin CLI creates atomic symlinks
`APP_DIR/{namespace}` pointing to the active release directory. This allows
an external web server (e.g. nginx) to serve app static files directly.

## Staging areas

Skrynia uses two distinct staging locations during deploy:

- **Build workspace** (`DATA_DIR/builds/build-*`): Disposable directory where
  the repo is cloned and `make build` runs. Cleaned up after deploy (success
  or failure). Created automatically on first deploy.

- **Release staging** (`RELEASES_DIR/{ns}/.staging-{pid}`): Validated build
  output is copied here, then atomically renamed into the release directory.
  Lives on the same filesystem as releases for atomic rename.

Releases, storage, and mutable state live under `DATA_DIR`.
The build workspace is also under `DATA_DIR` so that cross-container mounts
(Admin CLI container -> Builder container) see identical paths.

## Running

```sh
# Start the server (standalone mode)
node src/server.js

# Or with custom config
SKRYNIA_PORT=8080 SKRYNIA_DATA_DIR=/data/skrynia node src/server.js
```

## Deploying an app

All deploy parameters are required keyword flags:

```sh
# Deploy a single-repo app
node src/admin.js deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def456...789 \
  --subdir . \
  --namespace myapp

# Deploy from a monorepo subdirectory
node src/admin.js deploy \
  --repo git@github.com:myorg/monorepo.git \
  --commit def456ghi789...012 \
  --subdir frontend \
  --namespace myapp

# Deploy with custom builder
node src/admin.js deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def456...789 \
  --subdir . \
  --namespace myapp \
  --builder myregistry/builder:v2
```

The `--commit` must be a full 40 or 64 hex character git object id.

Deploy process:
1. Validates namespace and subdirectory
2. Clones the repo to a temporary workspace under `DATA_DIR/builds/`
3. Checks out the exact commit and verifies HEAD matches
4. Validates subdirectory stays inside repo (realpath check)
5. Runs `make build` in a disposable container (repo mounted read-write, writable root FS for npm)
6. Validates build output (rejects symlinks and special files)
7. Auto-creates namespace with default quota if absent (preserves existing on redeploy)
8. Copies validated output to staging under `RELEASES_DIR/{ns}/.staging-{pid}`
9. Atomic rename staging to release directory (same filesystem)
10. Atomically activates the release via symlink swap
11. If `SKRYNIA_APP_DIR` is set, atomically exposes `APP_DIR/{ns}` symlink

### Included example

`example/hello` is a minimal deployable app:

```sh
node src/admin.js deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir example/hello \
  --namespace hello-app
```

`example/birthday-list` is a wishlist reservation app demonstrating Skrynia client storage:

```sh
node src/admin.js deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir example/birthday-list \
  --namespace birthday-list
```

## Rollback

```sh
node src/admin.js rollback --namespace myapp
node src/admin.js rollback --namespace myapp --release 20260910120000-abc123
```

## Undeploy

```sh
node src/admin.js undeploy --namespace myapp
```

Removes releases, stored data, namespace state, and APP_DIR symlink.
Always destructive.

## Managing namespaces

```sh
node src/admin.js ns create --namespace myns --quota 20971520
node src/admin.js ns list
node src/admin.js ns inspect --namespace myns
node src/admin.js ns remove --namespace myns
```

## Monitoring

```sh
curl http://127.0.0.1:17380/_skrynia/health
node src/admin.js releases --namespace myapp
node src/admin.js inspect --namespace myapp
```

## Builder

The builder image is a `node:20-alpine` image with `make`, `git`, and `npm`.
The default image is published to GHCR from `builder/Dockerfile`.
Local `make builder` builds the image locally as a developer convenience.
Containers run with `--rm` (disposable), `--user` for output ownership,
and `HOME=/tmp` so npm works under arbitrary numeric UIDs. The root
filesystem is writable so builds (including npm) can produce output. The
app repo is mounted read-write. Builds are disposable, not
security-sandboxed; repositories may access the network during build.

To build the builder image locally:

```sh
make builder
```
