# Skrynia

Platform for small JavaScript-heavy and static web apps, served canonically at `https://vau.place/a/<app-name>/`.

## Quick start

```sh
# Deploy an app from a git repo
skrynia deploy git@github.com:myorg/myapp.git abc123def . myapp

# Rollback
skrynia rollback myapp

# Undeploy (removes app, releases, namespace data and state)
skrynia undeploy myapp
```

## Architecture

- **Single-server process.** One Skrynia server handles all apps and storage. Node.js event-loop serialization makes concurrent request mutations safe.
- **Namespace = app.** The namespace name is the URL suffix: `/a/<namespace>/`. Namespaces must be created administratively before the public API accepts writes.
- **Versioned deploys.** Each deploy builds into a staging area, validates output, then atomically publishes to an immutable release directory.
- **Shared durable storage.** Namespace + key -> opaque bytes. No list/enumeration API.

## CLI reference

```
skrynia deploy <repo-url> <commit> <subdir> <namespace> [--builder IMAGE]
    Deploy app from git source. Validates subdirectory stays inside
    repository. Builds via container with whole repo mounted. Stages,
    validates (no symlinks/special files), then publishes release.
    Updates namespace config with currentReleaseId.

skrynia undeploy <namespace>
    Remove deployed releases, stored data, and namespace state.
    Always destructive; no preserve-data option in v1.

skrynia rollback <namespace> [release-id]
    Rollback to previous or specified release. Updates config metadata.

skrynia releases <namespace>
    List releases for a namespace.

skrynia inspect <namespace>
    Show deployment config.

skrynia ns create <namespace> [--quota BYTES]
    Create a namespace.

skrynia ns remove <namespace>
    Delete namespace and all its data.

skrynia ns inspect <namespace>
    Show namespace usage and config.

skrynia ns list
    List all namespaces with usage.
```

## Storage API

Base path: `/_skrynia/store/<namespace>/<key>`

| Method | Description |
|--------|-------------|
| `GET` | Read object |
| `POST` | Create object (fails if exists, requires existing namespace) |
| `PUT` | Replace object (atomic, enforces quota after replacement) |
| `DELETE` | Remove object |

### Object modes

- **immutable** - Cannot be modified or deleted via public API.
- **capability-write** (default) - Server returns a one-time write capability on create. Subsequent put/delete require it. Verifier stored in `.cap` file only (single source of truth).
- **public-write** - Anyone who knows namespace/key may modify or delete.

### Headers

- `X-Skrynia-Mode` - Set on create (POST). Values: `immutable`, `capability-write`, `public-write`.
- `X-Skrynia-Capability` - Required for put/delete on capability-write objects.

### Client library

Include `/_skrynia/client/skrynia.js` in your app:

```js
const store = Skrynia.store('my-namespace');
const result = await store.get('key');
if (result) {
  const text = await result.bytes.text();
  const json = await result.bytes.json();
  const raw = await result.bytes.bytes(); // Uint8Array
}
await store.create('key', value, { mode: 'public-write' });
```

Binary-safe: GET returns `RawBytes` with `.text()`, `.json()`, `.bytes()` methods. No automatic response coercion.

## Deployment model

1. Clone repo to disposable temp workspace
2. Validate subdirectory stays inside repository (no escape)
3. Build inside container: whole repo mounted read-only, working directory set to subdirectory (monorepo `../shared` paths available)
4. Validate build output: reject symlinks, block/char devices, FIFOs, sockets
5. Stage validated output, then atomic rename to release directory
6. Atomically switch `current` symlink
7. Update config with `currentReleaseId`
8. Prune old releases (keep last 3)

## Configuration

Environment variables (or `/usr/local/share/skrynia/skrynia.conf`):

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Data directory |
| `SKRYNIA_BUILDER_IMAGE` | `ghcr.io/ottojung/skrynia-builder:0.1.0` | Builder container |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota (10 MiB) |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max object size |

## Runtime invariant

Exactly ONE server process per data directory. The admin CLI and server must not modify the same data directory simultaneously. Atomic filesystem primitives (exclusive create, tmp+rename) provide per-operation safety within a single process.

## Systemd

Runs as dedicated `skrynia` user with `NoNewPrivileges`, `ProtectSystem=strict`, and `ReadWritePaths=/var/lib/skrynia`.
