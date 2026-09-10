# Skrynia

A minimal platform for deploying and serving small web apps with durable key-value storage.

## Features

- **Deploy from git**: Clone, checkout exact commit, build via container, atomic release activation
- **Storage API**: Namespace-scoped key-value store with immutable, capability-write, and public-write modes
- **App serving**: Static file serving from active releases at `/a/{namespace}/`
- **Admin CLI**: Full lifecycle management with keyword-flag interface
- **Client library**: Tiny browser library for storage access at `/_skrynia/client/skrynia.js`

## Quick start

```sh
# Install (builds local builder image, installs to /usr/local)
make install

# Deploy an app
skrynia deploy \
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
skrynia deploy \
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
3. Runs `make build` in a read-only container (repo mounted read-write)
4. Validates build output (no symlinks, no special files)
5. Auto-creates namespace with default quota if absent (preserves existing)
6. Atomic rename staging dir into release dir (same filesystem)
7. Atomically activates release via symlink swap

### Included example

```sh
# Deploy the hello example from this repo
skrynia deploy \
  --repo git@github.com:ottojung/Skrynia.git \
  --commit $(git rev-parse HEAD) \
  --subdir examples/hello \
  --namespace hello-app
```

`examples/hello` is a minimal app with `index.html` and a Makefile that copies it to `build/`.

## CLI reference

```
skrynia deploy --repo <url> --commit <sha> --subdir <path> --namespace <name> [--builder IMAGE]
skrynia undeploy --namespace <name>
skrynia rollback --namespace <name> [--release <id>]
skrynia releases --namespace <name>
skrynia inspect --namespace <name>

skrynia ns create --namespace <name> [--quota BYTES]
skrynia ns remove --namespace <name>
skrynia ns inspect --namespace <name>
skrynia ns list
```

Commands are positional words; all data arguments are keyword flags.

## Configuration

`/usr/local/share/skrynia/skrynia.conf`:

```sh
SKRYNIA_PORT=17380
SKRYNIA_DATA_DIR=/var/lib/skrynia
SKRYNIA_BUILDER_IMAGE=skrynia-builder:0.1.0
SKRYNIA_DEFAULT_QUOTA_BYTES=10485760
```

## Storage API

- `GET /_skrynia/health` — health check
- `GET /_skrynia/client/skrynia.js` — browser client library
- `GET /_skrynia/store/{ns}/{key}` — read object
- `POST /_skrynia/store/{ns}/{key}` — create object
- `PUT /_skrynia/store/{ns}/{key}` — replace object
- `DELETE /_skrynia/store/{ns}/{key}` — delete object
- `GET /a/{ns}/{path}` — serve app static files

See [docs/api-spec.md](docs/api-spec.md) for full details.

## Architecture

- **Server**: Single-process Node.js HTTP server; event-loop serialization for safety
- **Admin CLI**: Keyword-flag interface; all operations use `execFileSync` (no shell injection)
- **Builder**: Local Docker image (`node:20-alpine` + make + git); runs with `--read-only`, `--cap-drop ALL`, `--no-new-privileges`
- **Storage**: Filesystem-based; one `.dat`/`.meta`/`.cap` triplet per object per namespace
- **Releases**: Immutable directories under `RELEASES_DIR/{ns}/`; atomic symlink swap for activation

## Running tests

```sh
make test
```

## License

See repository.
