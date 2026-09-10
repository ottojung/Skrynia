# Skrynia Deployment Guide

## Prerequisites

- Node.js 18+ on the server
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
- `/etc/systemd/system/skrynia.service` - Systemd service

## Configuration

Edit `/usr/local/share/skrynia/skrynia.conf`:

```sh
SKRYNIA_PORT=17380
SKRYNIA_DATA_DIR=/var/lib/skrynia
SKRYNIA_BUILDER_IMAGE=ghcr.io/ottojung/skrynia-builder:latest
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

```sh
# Basic deploy
skrynia deploy git@github.com:myorg/myapp.git abc123def . myapp

# Deploy from subdirectory of monorepo
skrynia deploy git@github.com:myorg/monorepo.git def456ghi frontend myapp

# Deploy with custom builder
skrynia deploy git@github.com:myorg/myapp.git abc123def . myapp --builder myregistry/builder:v2
```

The deploy command:
1. Clones the repo to a temporary workspace
2. Checks out the exact commit
3. Runs `make build` in the subdirectory via the builder container
4. Copies `build/` output to an immutable release directory
5. Atomically activates the release

## Rollback

```sh
# Rollback to previous release
skrynia rollback myapp

# Rollback to specific release
skrynia rollback myapp 20260910120000
```

## Undeploy

```sh
# Remove app and all stored data (default)
skrynia undeploy myapp

# Remove app but keep stored data
skrynia undeploy myapp --preserve-data
```

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
