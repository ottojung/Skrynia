# Secret store paths at the deployment boundary

An object key is a bearer credential: anyone who knows `SKRYNIA_URL/store/{namespace}/{key}`
can read that object, and `capability-write` protects mutation, not reading. So the URL
carrying the key is itself the secret, and every component that sees the URL is a place
the secret can be retained.

Skrynia's own half of this is enforced in code and in `make test`. This document covers
the other half, which lives in the reverse proxy and in operator tooling and therefore
**cannot be enforced by this repository**.

## What Skrynia already guarantees

Measured, not asserted:

- **No secret in the process streams.** The store request path never writes the namespace,
  the object key or a capability to stdout or stderr, on any outcome including every
  refusal and an internal filesystem failure. Normal refusals write nothing at all; an
  internal failure logs only a stable error code, never `e.message`, because filesystem
  error messages embed the failing path. Pinned by
  `tests/test.js:test_store_path_neither_logs_nor_reflects_the_secret_url`, which captures
  the process streams across the whole store path and fails on a single byte, and by
  `tests/test.js:test_store_error_logging_is_path_free`, which forces a metadata write
  failure and asserts that neither the namespace nor the key appears, and by
  `tests/test-push.js:test_push_pump_error_logging_is_path_free`, which forces a
  push pump failure and asserts that the namespace does not appear.
- **No echo.** No store response body or response header reflects the namespace, the key
  or the capability. Same test.
- **No referrer.** Every store response, every management response, and every served app
  file carry `Referrer-Policy: no-referrer`, including the plain-text refusals and the
  client library, because the header is set once for every response rather than per route.
  Store and management responses additionally carry `Cache-Control: no-store`, including
  the store and management `405` method refusals (405 is heuristically cacheable). Pinned
  by `tests/test.js:test_secret_url_responses_forbid_referrer_and_retention`.
  This matters for the management API in particular, because `SKRYNIA_TOKEN` travels in
  the query string there.
- **Capabilities never travel in a URL.** Store capabilities (`X-Skrynia-Capability`) and
  push subscription capabilities are header-only. The management token is the one
  deliberate exception and it is why management responses are `no-store`.
- **Private state is `0600` under `0700`.** `store-meta/` (which holds capability hashes)
  and `state/{ns}/push-subs/`, `state/{ns}/push-rules.json`, `state/_push/` (the outbox and
  `vapid.json`) are `0600` files in `0700` directories, so neither the bytes nor the file,
  outbox and subscription names are readable by another local account. Object bytes under `store/` are `0644` by
  design: that tree is deliberately static-file compatible so nginx can serve store GETs
  directly (`docs/intent-records/nginx-store-get-replacement.md`).
- **No third-party sink.** Skrynia's only outbound HTTP is Web Push delivery to the
  endpoint the browser itself registered, and the payload is exactly the channel name,
  never the namespace, key or object bytes. There is no analytics, telemetry or error
  reporter in the server.

## What the operator must do

None of the following is checkable from this repository. Each is a deployment decision.

### Reverse proxy: access logging

The store location must not record the request URI. nginx:

```nginx
location ~ ^/store/ {
    # Never log the URI: the object key is a bearer credential.
    access_log off;
    error_log /var/log/nginx/store-error.log warn;
    ...
}
```

`$uri`, `$request_uri` and `$request` all contain the key, so a log format built from
them is a disclosure channel even when the response body is never logged. `access_log off`
is the only setting that cannot be got wrong by choosing the wrong variable later.

If access logging must stay on for the store location, use a format that emits the
namespace but never the key, for example:

```nginx
log_format store_safe '$remote_addr [$time_local] "$request_method" '
                      'ns=$store_namespace status=$status bytes=$body_bytes_sent';
```

with the namespace extracted by a `map` on `$uri`. Verify it by making one request with a
throwaway key and reading the resulting line.

### Reverse proxy: management access logging

The management API lives at the root, not under `/store/`, so `access_log off` on the
store location does not cover it. Two secrets travel in management query strings:
`SKRYNIA_TOKEN` on every management request, and the exact object key on
`push/rules/set`, `push/rules/get` and `push/rules/remove`. A default root access log
therefore retains both. Turn request logging off for the management endpoints too, or use a
format that emits neither the query string nor the path:

```nginx
location ~ ^/(deploy|undeploy|rollback|releases|inspect|ns/|push/rules/) {
    # SKRYNIA_TOKEN and, for push rules, the object key live in the query string.
    access_log off;
    error_log /var/log/nginx/skrynia-error.log warn;
    ...
}
```

### Reverse proxy: error logging

`error_log` at `warn` or above on the store location does not echo the request URI for
ordinary client errors. At `debug` or `info` it can, because nginx logs the request line
on connection and header events. Keep the store location's error level at `warn` or above,
and do not enable `debug_connection` or `debug_headers` globally.

Skrynia's own error output is separately pinned: see "What Skrynia already guarantees".

### Reverse proxy: HTTPS and HSTS

Terminate TLS in front of Skrynia and redirect port 80 rather than serving it. Send HSTS
on the store location and on the app location, not only on some other vhost. Skrynia
listens on `127.0.0.1` and speaks clear HTTP; TLS is the proxy's job. As a guard against
declaring a cleartext public root, the server refuses to start when `SKRYNIA_URL` uses
`http:` for a non-loopback host, unless `SKRYNIA_ALLOW_INSECURE_HTTP=1` is set. Pinned by
`tests/test.js:test_plain_http_public_root_is_rejected`.

Note the nginx static-file replacement path: when nginx serves store GETs from
`DATA_DIR/store/` directly, nginx's headers, not Skrynia's, are what the client sees. The
`Referrer-Policy` and `Cache-Control` values Skrynia sets must then be set on the nginx
store location too, or the guarantee is lost exactly where the sensitive reads are served.

### Reverse proxy: no upstream CDN or observability layer

Any CDN, WAF, load balancer or log shipper in front of the store location sees the full
path. Confirm none is present, or that it is configured with query/path stripping and no
request-body logging. Skrynia cannot observe this.

### Backup tooling

`DATA_DIR` is the backup unit and it contains `store/{namespace}/{key}` — the secret keys
are the directory tree, not the file contents. Consequently:

- backups of `DATA_DIR` are credential material and must be treated as such: encrypted at
  rest, access-controlled like the live data, and not shipped to a lower-trust store;
- do not back up `store/` to a world-readable location, and do not use a backup tool that
  produces per-file listings retained in logs or in a manifest checked into a repository;
- `store-meta/` holds SHA-256 capability hashes and `state/_push/vapid.json` holds the
  installation's VAPID private key. Both are already `0600`; a backup must preserve that,
  because restoring them `0644` would expose them to every local account;
- when restoring, verify `DATA_DIR/store-meta` and `DATA_DIR/state` permissions before the
  server starts. Skrynia re-applies `0700` to those two roots at startup, but per-namespace
  subdirectories are re-pinned only when they are next written, so inspect the trees
  directly rather than assuming the startup fix reached every subdirectory.

Skrynia cannot enforce any of this: backups are taken by operator tooling outside this
repository.

### Browser-side and app-side

App code that constructs store URLs is the app's own responsibility. Skrynia's client
library (`/client/skrynia.js`) puts capabilities in headers and never in a URL, and sets
no cookies. An app that logs a store URL to `console`, `localStorage`, a service-worker
cache or a third-party analytics endpoint defeats every measure above. Skrynia serves the
app files and therefore cannot see what they do.