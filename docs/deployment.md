# Skrynia Deployment Guide

## Prerequisites

- Node.js 20+ on the server
- Docker (for builder container)
- git (for cloning app repositories)

## Installation

```sh
git clone git@github.com:ottojung/Skrynia.git
cd Skrynia
make install
```

This builds the local builder Docker image and installs:
- `/usr/local/bin/skrynia-server` — Storage server
- `/usr/local/bin/skrynia` — Admin CLI
- `/usr/local/lib/skrynia/` — Server and admin code
- `/usr/local/share/skrynia/` — Configuration and client library
- `/etc/systemd/system/skrynia.service` — Systemd service (runs as `skrynia` user)

## Deploying an app

All deploy parameters are required keyword flags:

```sh
# Deploy a single-repo app
skrynia deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def456...789 \
  --subdir . \
  --namespace myapp

# Deploy from a monorepo subdirectory
skrynia deploy \
  --repo git@github.com:myorg/monorepo.git \
  --commit def456ghi789...012 \
  --subdir frontend \
  --namespace myapp

# Deploy with custom builder
skrynia deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def456...789 \
  --subdir . \
  --namespace myapp \
  --builder myregistry/builder:v2
```

The `--commit` must be a full 40 or 64 hex character git object id.

Deploy process:
1. Validates namespace and subdirectory
2. Clones the repo to a temporary workspace
3. Checks out the exact commit and verifies HEAD matches
4. Validates subdirectory stays inside repo (realpath check)
5. Runs `make build` in a read-only container (repo mounted read-write, capabilities dropped)
6. Validates build output (rejects symlinks and special files)
7. Auto-creates namespace with default quota if absent (preserves existing on redeploy)
8. Stages validated output under `RELEASES_DIR/{ns}/.staging-{pid}`
9. Atomic rename to release directory (same filesystem)
10. Atomically activates the release via symlink swap

### Included example

`examples/hello` is a minimal deployable app:

```sh
skrynia deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir examples/hello \
  --namespace hello-app
```

## Rollback

```sh
skrynia rollback --namespace myapp
skrynia rollback --namespace myapp --release 20260910120000-abc123
```

## Undeploy

```sh
skrynia undeploy --namespace myapp
```

Removes releases, stored data, and namespace state. Always destructive.

## Managing namespaces

```sh
skrynia ns create --namespace myns --quota 20971520
skrynia ns list
skrynia ns inspect --namespace myns
skrynia ns remove --namespace myns
```

## Monitoring

```sh
curl http://127.0.0.1:17380/_skrynia/health
skrynia releases --namespace myapp
skrynia inspect --namespace myapp
```

## Builder

The builder image is built locally during `make install`. It is a `node:20-alpine` image with `make` and `git`. Containers run with `--read-only` root filesystem, `--cap-drop ALL`, and `--no-new-privileges`. The app repo is mounted read-write so `make build` can write `build/`.

To rebuild the builder image manually:

```sh
docker build -t skrynia-builder:0.1.0 builder/
```
