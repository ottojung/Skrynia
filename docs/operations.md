# Running Skrynia

Skrynia has one HTTP server and one authoritative data directory.
The HTTP API does not contain special administrative endpoints.

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| SKRYNIA_URL | required | Canonical absolute HTTP API URL, optionally including a path |
| SKRYNIA_PORT | 17380 | Local listener on 127.0.0.1 |
| SKRYNIA_DATA_DIR | /var/lib/skrynia | Persistent object and private authority state |
| SKRYNIA_MAX_KEY_LENGTH | 256 | Max length of each object identifier |
| SKRYNIA_MAX_OBJECT_SIZE | 10485760 | Max bytes per uploaded value |
| SKRYNIA_PUSH_SUBJECT | SKRYNIA_URL | Web Push VAPID contact |
| SKRYNIA_PUSH_SEND_TIMEOUT_MS | 10000 | Delivery timeout in milliseconds |

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

A single Skrynia server must own the data directory. Do not run multiple
writer processes against the same directory, including across containers
sharing the same volume. Persist the directory across container restarts.
Back up the directory consistently, including `state/` and `store/`.
Restore private credentials and object files together when recovering state.

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
File names have no separators, and `.` / `..` are invalid identifiers.
nginx may directly serve `GET /store/{key}` from the public directory,
using standard MIME types and nginx's normal mtime-size ETags. Configure
the cache policy consistently with the Skrynia HTTP implementation,
particularly for mutable keys.

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
