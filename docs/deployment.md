# Skrynia Deployment Guide

## Prerequisites

- Node.js 20+ when running Skrynia directly
- Docker (the server launches builder containers during deploy)
- git and an SSH client for cloning app repositories

The published Skrynia runtime image already includes git, OpenSSH client, and the Docker CLI.

## Configuration

Skrynia is configured via environment variables or `createServer()` options:

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_TOKEN` | (none) | Management API token; management is disabled when unset |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Mutable state: releases, storage, namespace state, build workspaces |
| `SKRYNIA_APP_BASE_PATH` | `/apps` | URL prefix for app serving (set to match your proxy) |
| `SKRYNIA_APP_DIR` | (none) | Filesystem dir for exposed app symlinks |
| `SKRYNIA_BUILDER_IMAGE` | `skrynia-builder:0.1.0` | Docker image for building apps |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota (10 MiB) |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Default max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max single object size (10 MiB) |

When `SKRYNIA_APP_DIR` is set, deployment atomically creates or replaces `APP_DIR/{namespace}` as the one active-release symlink. An external web server such as nginx can serve that directory directly.

## Runtime container mounts

When the server runs in Docker, deployment happens inside the long-lived Skrynia server process. The runtime therefore needs explicit access to:

- `SKRYNIA_DATA_DIR` read-write;
- `SKRYNIA_APP_DIR` read-write when configured, because activation replaces symlinks there;
- `/var/run/docker.sock` so the server can launch the builder container;
- repository credentials such as `/root/.ssh` read-only when private SSH repositories are used.

The Skrynia runtime root filesystem can remain `--read-only`; give it a writable `/tmp` tmpfs. The separate builder container is intentionally writable and disposable.

## Staging areas

Skrynia uses two staging locations during deploy:

- **Build workspace** (`DATA_DIR/builds/build-*`): disposable clone/workspace. Removed after every deployment attempt.
- **Release staging** (`RELEASES_DIR/{ns}/.staging-{pid}`): validated build output before atomic rename into the immutable release directory.

Both are under `DATA_DIR` so the path is visible to the server and to builder containers through the Docker host bind mount.

## Running

```sh
SKRYNIA_TOKEN=replace-me node src/server.js
```

Health remains public:

```sh
curl http://127.0.0.1:17380/_skrynia/health
```

## Deploying an app

Deployment is an authenticated HTTP request. The URL must be quoted in a shell because it contains `&` characters.

```sh
curl 'http://127.0.0.1:17380/_skrynia/deploy?repo=git@github.com:myorg/myapp.git&commit=0123456789012345678901234567890123456789&subdir=.&namespace=myapp&token=replace-me'
```

A custom builder can be supplied with `builder=...`.

The request is synchronous: curl returns after clone, build, validation, activation, and cleanup complete.

Deploy process:

1. Validate namespace, subdirectory, and full 40- or 64-hex commit id.
2. Clone to `DATA_DIR/builds/build-*`.
3. Check out the requested commit and verify exact HEAD equality.
4. Resolve the requested subdirectory and reject escapes outside the clone.
5. Run `make build` in the builder container.
6. Require `build/` and reject symlinks/special files in output.
7. Auto-create the namespace only after a valid build.
8. Copy output to release staging and atomically rename it to the release directory.
9. Atomically replace the one active-release symlink.
10. Save deployment metadata and retain the newest three releases.

### Included examples

```sh
curl 'http://127.0.0.1:17380/_skrynia/deploy?repo=git@github.com:ottojung/Skrynia.git&commit=0123456789012345678901234567890123456789&subdir=example/hello&namespace=hello-app&token=replace-me'

curl 'http://127.0.0.1:17380/_skrynia/deploy?repo=git@github.com:ottojung/Skrynia.git&commit=0123456789012345678901234567890123456789&subdir=example/birthday-list&namespace=birthday-list&token=replace-me'
```

## Rollback and releases

```sh
# Roll back to the previous release
curl 'http://127.0.0.1:17380/_skrynia/rollback?namespace=myapp&token=replace-me'

# Roll back to a named release
curl 'http://127.0.0.1:17380/_skrynia/rollback?namespace=myapp&release=20260910120000000-abc123&token=replace-me'

# List releases
curl 'http://127.0.0.1:17380/_skrynia/releases?namespace=myapp&token=replace-me'

# Inspect deployment metadata
curl 'http://127.0.0.1:17380/_skrynia/inspect?namespace=myapp&token=replace-me'
```

## Undeploy

```sh
curl 'http://127.0.0.1:17380/_skrynia/undeploy?namespace=myapp&token=replace-me'
```

Undeploy is destructive: it removes the active link, releases, stored data, and namespace state.

## Managing namespaces

```sh
curl 'http://127.0.0.1:17380/_skrynia/ns/create?namespace=myns&quota=20971520&token=replace-me'
curl 'http://127.0.0.1:17380/_skrynia/ns/list?token=replace-me'
curl 'http://127.0.0.1:17380/_skrynia/ns/inspect?namespace=myns&token=replace-me'
curl 'http://127.0.0.1:17380/_skrynia/ns/remove?namespace=myns&token=replace-me'
```

## Builder

The default builder is a `node:20-alpine` image containing `make`, `git`, and `npm`, published to GHCR from `builder/Dockerfile`.

Builder containers are deliberately simple: writable root filesystem, normal network access, repository mounted read-write, `HOME=/tmp`, and `--rm` so each build container is thrown away. Skrynia does not present the builder as a security sandbox.

To build the builder image locally:

```sh
make builder
```
