# Skrynia Deployment Guide

## Prerequisites

- Node.js 20+ on the server
- Docker (for builder container)
- git (for cloning app repositories)

## Installation

### Via Config Server

Skrynia is installed as a submodule of Config Server:

```sh
cd /path/to/config-server
make skrynia
```

### Manual installation

```sh
git clone git@github.com:ottojung/Skrynia.git
cd Skrynia
make install
```

This installs:
- `/usr/local/bin/skrynia-server` - Storage server
- `/usr/local/bin/skrynia` - Admin CLI
- `/usr/local/lib/skrynia/` - Server and admin code
- `/usr/local/share/skrynia/` - Configuration and client library
- `/etc/systemd/system/skrynia.service` - Systemd service (runs as `skrynia` user)

## Configuration

Edit `/usr/local/share/skrynia/skrynia.conf`:

```sh
SKRYNIA_PORT=17380
SKRYNIA_DATA_DIR=/var/lib/skrynia
SKRYNIA_BUILDER_IMAGE=ghcr.io/ottojung/skrynia-builder:0.1.0
SKRYNIA_DEFAULT_QUOTA_BYTES=10485760
```

## Nginx integration

Add to your nginx config for `vau.place`:

```nginx
# Skrynia storage API
location /_skrynia/ {
    proxy_pass http://127.0.0.1:17380;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}

# Skrynia app serving
location /a/ {
    proxy_pass http://127.0.0.1:17380;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
}
```

## Deploying an app

All deploy parameters are required keyword flags:

```sh
# Deploy from repo root (repo root is the app)
skrynia deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def \
  --subdir . \
  --namespace myapp

# Deploy from subdirectory of monorepo
skrynia deploy \
  --repo git@github.com:myorg/monorepo.git \
  --commit def456ghi \
  --subdir frontend \
  --namespace myapp

# Deploy with custom builder
skrynia deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def \
  --subdir . \
  --namespace myapp \
  --builder myregistry/builder:v2
```

The deploy command:
1. Validates namespace and subdirectory (no `..`, no absolute paths)
2. Clones the repo to a temporary workspace
3. Checks out the exact commit
4. Validates subdirectory stays inside repo (realpath check)
5. Runs `make build` in the subdirectory via the builder container (whole repo mounted, workdir set to subdir)
6. Validates build output (rejects symlinks and special files)
7. Stages validated output, then atomic rename to release directory
8. Atomically activates the release
9. Updates config with `currentReleaseId`

### Included example

The `examples/hello` directory contains a minimal deployable app with its own Makefile:

```sh
skrynia deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir examples/hello \
  --namespace hello-app
```

The Makefile produces `build/index.html` with a simple HTML page.

## Rollback

```sh
# Rollback to previous release
skrynia rollback myapp

# Rollback to specific release
skrynia rollback myapp 20260910120000-abc123
```

Rollback updates the deployment config metadata with the new `currentReleaseId`.

## Undeploy

```sh
# Remove app, releases, namespace data and state (always destructive)
skrynia undeploy myapp
```

Undeploy always removes releases, stored data, and namespace state. There is no preserve-data option in v1.

## Managing namespaces

```sh
# Create namespace with custom quota
skrynia ns create myns --quota 20971520

# List all namespaces
skrynia ns list

# Inspect namespace usage
skrynia ns inspect myns

# Remove namespace and all data
skrynia ns remove myns
```

## Monitoring

```sh
# Check server health
curl http://127.0.0.1:17380/_skrynia/health

# List releases for an app
skrynia releases myapp

# Inspect deployment config
skrynia inspect myapp
```

## Troubleshooting

### Build fails
- Check builder image exists: `docker pull $SKRYNIA_BUILDER_IMAGE`
- Verify `make build` works in the app subdirectory
- Check container doesn't need host network or privileged mode

### Namespace full
```sh
skrynia ns inspect myns  # Check current usage
# Either increase quota or remove old objects via storage API
```

### Service won't start
```sh
journalctl -u skrynia -n 50
systemctl status skrynia
```

The server runs as the dedicated `skrynia` user with `NoNewPrivileges` and `ProtectSystem=strict`.
