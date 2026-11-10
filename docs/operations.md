# Running Skrynia

Skrynia has one HTTP server and one authoritative data directory.
The HTTP API does not contain special administrative endpoints.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| SKRYNIA_URL | required | Canonical absolute HTTP API URL, optionally including a path |
| SKRYNIA_PORT | 17380 | HTTP listen port |
| SKRYNIA_HOST | 127.0.0.1 | Listener bind address; Docker image defaults to 0.0.0.0 for port mapping |
| SKRYNIA_DATA_DIR | /var/lib/skrynia | Persistent object and private authority state |
| SKRYNIA_MAX_KEY_LENGTH | 256 | Max length of each object identifier |
| SKRYNIA_MAX_OBJECT_SIZE | 10485760 | Max bytes per uploaded value |
| SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES | 41943040 | Aggregate buffered upload chunks per process |
| SKRYNIA_MAX_CONNECTIONS | 512 | Maximum accepted HTTP connections |
| SKRYNIA_PUSH_SUBJECT | SKRYNIA_URL | Web Push VAPID contact |
| SKRYNIA_PUSH_SEND_TIMEOUT_MS | 10000 | Delivery timeout in milliseconds |
| SKRYNIA_PUSH_CONCURRENCY | 8 | Maximum concurrent Web Push deliveries |

Start the server after installing Node.js 20+ and dependencies:

```sh
npm ci
SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia" \
SKRYNIA_URL="http://127.0.0.1:17380" \
node src/server.js
```

The initial startup creates a private, durable root grant record.
To retrieve its ID and issuer credential **on the host**, run:

```sh
SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia" node scripts/root-credentials.js
```

This displays a live bearer secret; use only on an authorized terminal.
The record's plaintext issuer secret is stored only on the host in
`state/grants/<root-grant-id>.json` (0600); the containing directory is
0700. Root revocation removes this file. On the next startup, Skrynia
creates a new root over all surviving object data.

The grant operation schema is strict. An existing installation using an
obsolete grant operation set cannot start with incompatible grant records.
For a clean authority reset while retaining object values, back up the data
first, revoke the old root using the old running version, stop that server,
and then start the new version. Root revocation invalidates subordinate
tokens and removes their Web Push subscriptions; the next start creates a
new root. Grant records are not silently converted or migrated.

A single Skrynia server must own the data directory. Do not run multiple
writer processes against the same directory, including across containers
sharing the same volume. Persist the directory across container restarts.
Back up the directory consistently, including `state/` and `store/`.
Restore private credentials and object files together when recovering state.

## Container and network security

The host process defaults to listening only on `127.0.0.1`. The Docker
image sets `SKRYNIA_HOST=0.0.0.0` so ordinary Docker port mappings work:

```sh
docker run --name skrynia -p 127.0.0.1:17380:17380 \
  -v skrynia-data:/var/lib/skrynia \
  -e SKRYNIA_URL=http://127.0.0.1:17380 \
  ghcr.io/ottojung/skrynia:latest
```

Use HTTPS and a reverse proxy in public deployments. Never expose private
grant credentials or `state/`. Authorized API requests use explicit
`Authorization: Bearer` tokens rather than cookies.

Web Push uses HTTPS endpoints with no embedded credentials. At connection
time, Skrynia pins DNS resolution to public routable IP addresses, rejecting
loopback and private services. Enforce an outbound firewall as defense in
depth. `SKRYNIA_PUSH_SEND_TIMEOUT_MS` sets the delivery timeout, and
`SKRYNIA_PUSH_CONCURRENCY` (default 8) bounds simultaneous sends.

## Filesystem and static serving

- `store/{key}`: public raw object files, mode 0644.
- `store-tmp/`: staging files, private.
- `state/grants/`: private grant records and root issuer credential.
- `state/push-subs/`: private Web Push subscription records.
- `state/push/`: VAPID key and persistent delivery queue.
- `state/store-transactions/`: recoverable object mutation intents.
- `state/grant-revocations/`: recoverable subtree revocation intents.
- `state/etag-sequence`: monotonic per-installation object-version sequence.

Only serve `store/` through a public static web server, never `state/`.
File names have no separators and never begin with a dot.
nginx may directly serve `GET /store/{key}` from the public directory,
using standard MIME types and nginx's whole-second mtime-size ETags.
**The ETag scheme must remain nginx-compatible:** Skrynia uses the exact
`"hex-mtime-seconds-hex-size"` format and a monotonic whole-second mtime
per mutation to prevent ABA. Rapid writes can move file mtimes ahead of
wall time. Configure the cache policy consistently, especially for
mutable keys.

The standalone server streams reads from already-open file descriptors,
bounds in-progress upload memory and connections, and defaults to 1000 keys
per listing page. Use `next` / `after` to retrieve further pages.

Skrynia is not tied to nginx; its own HTTP server implements public reads.
Static HTML and JavaScript served to browsers from mutually untrusted
publishers must use separate origins/hostnames. Key scopes do not provide
browser security isolation.

Push traffic and public reads are not quota-metered by grants. Grant quotas
apply to storage objects, grant metadata and push subscriptions. The server
also has bounded request size, number of subscriptions and notification queue
capacity as independent resource safeguards.

## Deployment by clients

An application publisher can upload static objects, arrange a versioned
immutable manifest, and CAS a root pointer to publish a complete release.
Serving HTML with standard MIME types requires only a conventional static
server configuration. This is a client protocol; the storage server is
unaware of application identities, builds, releases or rollbacks.
