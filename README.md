# Skrynia

A minimal platform for deploying and serving small web apps with durable key-value storage.

## Features

- **Deploy from git**: Clone, checkout exact commit, build via container, atomic release activation
- **Storage API**: Namespace-scoped key-value store with immutable, capability-write, and public-write modes
- **App serving**: Static file serving from active releases under a configurable base path (no canonical default; set `SKRYNIA_APP_BASE_PATH` to match your proxy)
- **Admin CLI**: Full lifecycle management with keyword-flag interface
- **Client library**: Tiny browser library for storage access at `/_skrynia/client/skrynia.js`

## Quick start

```sh
# Build the builder image (required for deploys)
make builder

# Start the server
node src/server.js

# Deploy an app
node src/admin.js deploy \
  --repo git@github.com:myorg/myapp.git \
  --commit abc123def456...789 \
  --subdir . \
  --namespace myapp

# Check health
curl http://127.0.0.1:17380/_skrynia/health
```

## Deployment

All deploy parameters are required keyword flags:

```sh
node src/admin.js deploy \
  --repo <git-url> \
  --commit <full-40-or-64-hex-sha> \
  --subdir <path-within-repo> \
  --namespace <app-name> \
  [--builder <docker-image>]
```

The `--commit` must be a full git object id (40 or 64 hex characters). The deploy verifies the checked-out HEAD matches.

Deploy process:
1. Clones repo, checks out exact commit, verifies HEAD
2. Validates subdirectory stays inside repo
3. Runs `make build` in a disposable container (repo mounted read-write, capabilities dropped)
4. Validates build output (no symlinks, no special files)
5. Auto-creates namespace with default quota if absent (preserves existing)
6. Atomic rename staging dir into release dir (same filesystem)
7. Atomically activates release via symlink swap
8. If `SKRYNIA_APP_DIR` is set, atomically exposes `APP_DIR/{namespace}` symlink

### Included example

```sh
# Deploy the hello example from this repo
node src/admin.js deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir example/hello \
  --namespace hello-app
```

`example/hello` is a minimal app with `index.html` and a Makefile that copies it to `build/`.

```sh
# Deploy the birthday-list example (npm-driven build)
node src/admin.js deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir example/birthday-list \
  --namespace birthday-list
```

`example/birthday-list` demonstrates shared wishlist reservations using Skrynia client storage with predetermined item keys and create-if-absent semantics.

## CLI reference

```
node src/admin.js deploy --repo <url> --commit <sha> --subdir <path> --namespace <name> [--builder IMAGE]
node src/admin.js undeploy --namespace <name>
node src/admin.js rollback --namespace <name> [--release <id>]
node src/admin.js releases --namespace <name>
node src/admin.js inspect --namespace <name>

node src/admin.js ns create --namespace <name> [--quota BYTES]
node src/admin.js ns remove --namespace <name>
node src/admin.js ns inspect --namespace <name>
node src/admin.js ns list
```

Commands are positional words; all data arguments are keyword flags.

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Mutable state directory |
| `SKRYNIA_APP_BASE_PATH` | `/apps` | URL prefix for app serving (set to match your proxy) |
| `SKRYNIA_APP_DIR` | (none) | Filesystem dir for exposed app symlinks |
| `SKRYNIA_BUILDER_IMAGE` | `skrynia-builder:0.1.0` | Builder Docker image |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max object size |

## Storage API

- `GET /_skrynia/health` — health check
- `GET /_skrynia/client/skrynia.js` — browser client library
- `GET /_skrynia/store/{ns}/{key}` — read object
- `POST /_skrynia/store/{ns}/{key}` — create object
- `PUT /_skrynia/store/{ns}/{key}` — replace object
- `DELETE /_skrynia/store/{ns}/{key}` — delete object
- `GET {base_path}/{ns}/{path}` — serve app static files (base_path configurable, no canonical default)

See [docs/api-spec.md](docs/api-spec.md) for full details.

## Architecture

- **Server**: Single-process Node.js HTTP server; event-loop serialization for safety
- **Admin CLI**: Keyword-flag interface; all operations use `execFileSync` (no shell injection)
- **Builder**: Published to GHCR from `builder/Dockerfile` (`node:20-alpine` + make + git + npm); local `make builder` is a developer convenience; containers run with `--rm`, `--cap-drop ALL`, `--security-opt no-new-privileges`, `HOME=/tmp`; root filesystem is writable for builds
- **Storage**: Filesystem-based; one `.dat`/`.meta`/`.cap` triplet per object per namespace
- **Releases**: Immutable directories under `RELEASES_DIR/{ns}/`; atomic symlink swap for activation
- **APP_DIR**: Optional external exposure; `APP_DIR/{ns}` symlink to release dir for direct web server access

## Running tests

```sh
make test
```

## License

See repository.
