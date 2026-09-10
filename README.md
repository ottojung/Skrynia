# Skrynia

Platform for small JavaScript-heavy and static web apps, served canonically at `https://vau.place/a/<app-name>/`.

## Quick start

```sh
# Deploy an app from a git repo
skrynia deploy git@github.com:myorg/myapp.git abc123def . myapp

# Rollback
skrynia rollback myapp

# Undeploy (removes app and all stored data by default)
skrynia undeploy myapp
```

## Architecture

- **Single-server process.** One Skrynia server handles all apps and storage. Node.js event-loop serialization makes concurrent request mutations safe.
- **Namespace = app.** The namespace name is the URL suffix: `/a/<namespace>/`.
- **Versioned deploys.** Each deploy creates an immutable release directory. Activation is an atomic symlink switch.
- **Shared durable storage.** Namespace + key -> opaque bytes. No list/enumeration API.

## CLI reference

```
skrynia deploy <repo-url> <commit> <subdir> <namespace> [--builder IMAGE]
    Deploy app from git source. Creates/ensures namespace, builds via
    container, creates release, atomically activates.

skrynia undeploy <namespace> [--preserve-data]
    Remove deployed release state. By default also deletes namespace
    and all stored data. --preserve-data keeps stored data.

skrynia rollback <namespace> [release-id]
    Rollback to previous or specified release.

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
| `POST` | Create object (fails if exists) |
| `PUT` | Replace object (atomic) |
| `DELETE` | Remove object |

### Object modes

- **immutable** - Cannot be modified or deleted via public API.
- **capability-write** (default) - Server returns a one-time write capability on create. Subsequent put/delete require it.
- **public-write** - Anyone who knows namespace/key may modify or delete.

### Headers

- `X-Skrynia-Mode` - Set on create (POST). Values: `immutable`, `capability-write`, `public-write`.
- `X-Skrynia-Capability` - Required for put/delete on capability-write objects.

### Client library

Include `/_skrynia/client/skrynia.js` in your app:

```js
const store = Skrynia.store('my-namespace');
await store.create('key', 'value', { mode: 'public-write' });
const { data, meta } = await store.get('key');
```

## Deployment model

1. `skrynia deploy` clones the repo to a disposable workspace
2. Runs `make build` inside a container with the workspace mounted
3. Copies `build/` output to an immutable release directory
4. Atomically switches the `current` symlink
5. Failed builds leave previous release untouched

## Configuration

Environment variables (or `/usr/local/share/skrynia/skrynia.conf`):

| Variable | Default | Description |
|----------|---------|-------------|
| `SKRYNIA_PORT` | `17380` | Server listen port |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Data directory |
| `SKRYNIA_BUILDER_IMAGE` | `ghcr.io/ottojung/skrynia-builder:latest` | Builder container |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Default namespace quota (10 MiB) |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Max objects per namespace |
| `SKRYNIA_MAX_KEY_LENGTH` | `256` | Max key length |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Max object size |

## Runtime invariant

Exactly ONE server process per data directory. The admin CLI and server must not modify the same data directory simultaneously. Atomic filesystem primitives (exclusive create, tmp+rename) provide per-operation safety within a single process.
