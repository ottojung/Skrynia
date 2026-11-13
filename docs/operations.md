# Operating Skrynia

Skrynia runs one Rust HTTP server over one authoritative filesystem directory.
It never interprets object contents as executable application instructions.

## Configuration

| Environment variable | Default | Purpose |
| --- | --- | --- |
| `SKRYNIA_URL` | required | Public absolute HTTP URL, including optional path prefix |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Persistent object and private authority state |
| `SKRYNIA_HOST` | `127.0.0.1` | Listener interface |
| `SKRYNIA_PORT` | `17380` | Listener port |
| `SKRYNIA_MAX_KEY_LENGTH` | `255` | Maximum flat key length (1–255 bytes; invalid values reject startup) |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Maximum object bytes |
| `SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES` | `41943040` | Aggregate upload-buffer budget |
| `SKRYNIA_MAX_CONNECTIONS` | `512` | Active connection limit |
| `SKRYNIA_SOCKET_IDLE_TIMEOUT_MS` | `30000` | Idle client timeout |
| `SKRYNIA_PUSH_CONCURRENCY` | `8` | Parallel notification deliveries |
| `SKRYNIA_PUSH_SEND_TIMEOUT_MS` | `10000` | Notification request timeout |
| `SKRYNIA_PUSH_MAX_ATTEMPTS` | `12` | Retry attempts before retirement |
| `SKRYNIA_PUSH_ALLOWED_HOSTS` | empty | Optional exact HTTPS endpoint host allowlist |
| `SKRYNIA_PUSH_SUBJECT` | public URL | VAPID contact |

Use the root-credential CLI on the authorized host only:

```sh
SKRYNIA_DATA_DIR=/var/lib/skrynia skrynia-server root-credentials
```

This command only reads the private root grant record. User and issuer
credentials must never be logged or placed in public objects.

## Filesystem and authority

`store/` contains public, byte-preserving regular files. Everything under
`state/` and `store-tmp/` is private. Root and grant records, subscription
state, transaction intents, and notification outboxes must be backed up together.

The server holds an exclusive kernel writer lock in
`state/writer.lock`. Leave the lock inode in place across restarts. A
second server using the same directory must refuse startup. Every required
state directory must be a real directory, never a symlink. Public files are
opened without following symlinks. Nonregular filesystem entries fail closed.

Object creation and replacement are journaled. A process interruption can
leave an incomplete intent, which must be replayed on the next startup.
Never manually discard transaction records.

## Container and reverse proxy

The container runs under UID/GID 1000. Give that UID write permissions to
the persistent mounted data directory. A read-only root filesystem with a
writable data mount is supported.

Terminate HTTPS at a reverse proxy, apply rate and connection limits, and
restrict access to private filesystem paths. The example nginx configuration
serves separate publisher origins with `disable_symlinks on` and dedicated
API routing. Different publishers cannot be isolated merely by key prefixes.
Use independent hostnames to prevent browser-origin privilege sharing.

Only public GET/HEAD responses may be served through an independent static
file server; mutation and grant endpoints require the Rust service.
Static validators may differ from Skrynia's quoted SHA-256 ETag. Conditional
writes therefore use a digest of the observed bytes.

## Notifications

Web Push uses authenticated VAPID and HTTPS. Delivery rejects local, private,
special-use and unresolved destinations. Configure an outbound firewall as
defense in depth. Pending generations are durable and can coalesce; receipt
can be duplicated. A removed or exhausted endpoint is retired.

## Backup and restore

A consistent backup includes `store/`, `store-tmp/`, and all of `state/`.
Stop the writer before taking an offline snapshot, preserve owner/modes,
and verify the copy before replacing a deployment. Restore the complete
directory as a unit and restart only one writer. Do not delete the writer-lock
file or attempt to manually complete an interrupted journal.

## Validation

`make test` runs default Rust formatting verification, structural linting,
strict Clippy, unit tests, and independent HTTP integration tests.
`make docker-build` produces the unprivileged release container.
See [verification](verification.md) for qualification evidence and limits.
