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
| SKRYNIA_PUSH_MAX_ATTEMPTS | 12 | Failed delivery attempts before retiring a subscription |
| SKRYNIA_PUSH_ALLOWED_HOSTS | empty (unrestricted public HTTPS hosts) | Optional comma-separated exact hostname allowlist for Web Push |
| SKRYNIA_SOCKET_IDLE_TIMEOUT_MS | 30000 | Close HTTP sockets after this many milliseconds of inactivity |

Start the server after installing Node.js 20+, util-linux `flock`, and
the Node dependencies. The server fails closed if `flock` is unavailable:

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

Skrynia's single writer process uses synchronous filesystem fsync
operations to preserve durability. Consequently, high write rates can
delay reads and push deliveries on its event loop. Benchmark on the
target storage medium and apply request rate limits at the reverse proxy
before exposing the service to high-volume anonymous traffic.

Skrynia acquires an exclusive kernel advisory lock on
`state/writer.lock` before accessing persistent grants or objects. Another
instance using the same directory fails startup. The lock releases after
normal shutdown, or automatically on process death; leave the lock file in
place, since removing a locked inode invalidates mutual exclusion. All
instances must share a filesystem with reliable Linux `flock` semantics
(including when mounted into containers). Store data on a persistent volume
and never run competing writers.
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

`SKRYNIA_PUSH_ALLOWED_HOSTS`, when configured, restricts subscription
endpoints to a comma-separated list of **exact** public HTTPS hostnames,
for example `updates.push.services.mozilla.com,fcm.googleapis.com`.
Existing subscriptions outside a newly applied allowlist are retired
without connecting to their endpoints. Other failures back off
exponentially (to at most 30 minutes), and after
`SKRYNIA_PUSH_MAX_ATTEMPTS` failures the subscription is removed.
The client must register a fresh subscription if it wants notifications
again. New writes do not reset failure counts or backoff.

A socket inactivity timeout protects against completely idle readers.
Slow readers that continue making progress and CPU-intensive requests
still require front-end rate limiting and reverse-proxy controls;
connection limits alone are not a full denial-of-service defense.

## Filesystem and static serving

- `store/{key}`: public raw object files, mode 0644.
- `store-tmp/`: staging files, private.
- `state/grants/`: private grant records and root issuer credential.
- `state/push-subs/`: private Web Push subscription records.
- `state/push/`: VAPID key and persistent delivery queue.
- `state/store-transactions/`: recoverable object mutation intents.
- `state/grant-revocations/`: recoverable subtree revocation intents.

Only serve `store/` through a public static web server, never `state/`.
File names have no separators and never begin with a dot.
nginx or any other static server can serve `GET /store/{key}` directly
from the public directory with its own HTTP cache validators, MIME mapping
and cache policy. Skrynia's GET/HEAD sends `ETag: "<sha256>"` computed
from the bytes for caching. A conditional PUT/DELETE uses
`X-Skrynia-If-SHA256` computed from the bytes read through any server.

Object files retain filesystem-provided modification times.
Create/replace journals record SHA-256 of the resulting bytes for recovery.

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
