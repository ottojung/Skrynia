# Audit: secret store paths as bearer credentials

Date: 2026/10/08
Scope: the complete `GET|POST|PUT|DELETE /store/{namespace}/{key}` request path and
every component that can retain or disclose the key.

An object key is a bearer credential. `capability-write` protects mutation, not reading,
so `SKRYNIA_URL/store/{namespace}/{key}` grants read access to anyone who holds it. The
URL is therefore secret and every layer that sees it is a disclosure surface.

This record states what was measured on the candidate head, the two defects found, and
the guards that now pin them. Operator-owned layers that Skrynia cannot enforce are
covered in `docs/secret-store-path-guard.md`.

## Method

- Read the whole store path: `src/server.js` routing and handlers, `src/shared.js`
  storage, `src/push.js` outbound delivery, `src/client.js` browser library,
  `action/deploy/index.js` CI caller.
- Enumerated every `res.writeHead`/`json` site and every `console.*` site.
- Ran targeted reproductions against a real in-process server (the same harness as
  `tests/test.js`).
- Grepped for any outbound request or telemetry (`https.request`, `fetch`, `XMLHttpRequest`,
  analytics/error-reporting packages) and for request-URL logging.
- Confirmed the automated guards with `make test`.

## Findings

| Area | Verdict | Evidence |
|------|---------|----------|
| Skrynia access logging | Pass after fix | No request-URL logging in `src/`; store refusals write nothing; internal store failures now log a code only |
| Skrynia error logging | **Fixed** | See defects B and D |
| Store response retention | **Fixed** | See defect A |
| Referrer behavior | Pass | `Referrer-Policy: no-referrer` set once in `route()` for every response; pinned by test |
| Browser client | Pass | `src/client.js` puts capabilities in `X-Skrynia-Capability`, never in a URL; sets no cookies; no `console`/`localStorage` writes |
| Third-party requests | Pass | Only outbound HTTP is Web Push to the browser-registered endpoint; payload is exactly the channel name; no analytics/telemetry |
| Reverse proxy | Operator | `docs/secret-store-path-guard.md` |
| Backup tooling | Operator | `docs/secret-store-path-guard.md` |
| HTTPS | Guarded | Server refuses a non-loopback plain-http `SKRYNIA_URL`; TLS remains the proxy's job |

## Defect A: a store `405` was retained

A `PATCH /store/{namespace}/{key}` answers `405 Method not allowed` through a bare
`res.writeHead(405)`. `405` is heuristically cacheable (RFC 9110), so an intermediary
could retain the response keyed by the bearer URL. Every other store response already
carried `Cache-Control: no-store`; this one did not. The management `405` had the same
gap, and its query string carries `SKRYNIA_TOKEN`.

Reproduction before the fix:

```
EVIDENCE-A store PATCH status: 405 cache-control: undefined referrer-policy: "no-referrer"
```

Fix: both `405` sites now send `Cache-Control: no-store`. Pinned by extending
`tests/test.js:test_secret_url_responses_forbid_referrer_and_retention` with a store
method refusal and a management method refusal.

## Defect B: a store failure logged the namespace and key

`handleDelete` caught a metadata cleanup failure and logged `e.message`. Filesystem error
messages embed the failing path, and the metadata path is
`store-meta/{namespace}/{key}.json`, so a single failed cleanup wrote the bearer key to
stderr. The same pattern existed on the push-enqueue failure path.

Reproduction before the fix (metadata rename forced to fail as `ENOSPC` would):

```
EVIDENCE-B captured stderr: "skrynia metadata cleanup error: ENOSPC: no space left on device, rename '/tmp/.../store-meta/secret-ns/secret-key-4d2e9b.json.tmp...' -> '/tmp/.../store-meta/secret-ns/secret-key-4d2e9b.json'\n"
EVIDENCE-B stderr contains namespace: true
EVIDENCE-B stderr contains key: true
```

Fix: store-path failures now go through `logStoreError`, which logs only a stable,
request-independent error code and never `e.message`. After the fix:

```
EVIDENCE-B captured stderr: "skrynia metadata cleanup error: ENOSPC\n"
EVIDENCE-B stderr contains namespace: false
EVIDENCE-B stderr contains key: false
```

Pinned by `tests/test.js:test_store_error_logging_is_path_free`, which forces the failure
and asserts the namespace and key are absent while the failure is still surfaced.

## Defect D: the push pump logged the namespace

The durable push pump (`pumpTick` in `src/push.js`) caught any error and logged
`e.message`. The pump reads `state/{ns}/push-subs/` through `allSubs`, so a
filesystem failure there (EACCES, ENOSPC, EIO) embeds the namespace — half of the
bearer store path — in the message. This is the same pattern as defect B, on the
asynchronous store-mutation path rather than the request path.

Reproduction before the fix (subscription directory forced to fail as EACCES):

```
EVIDENCE-D captured stderr: "skrynia push pump error: EACCES: permission denied, scandir '/tmp/.../state/pump-error-ns/push-subs'\n"
EVIDENCE-D stderr contains namespace: true
```

Fix: the pump error handler now logs only a stable error code, never `e.message`,
matching `logStoreError` in `src/server.js`. After the fix:

```
EVIDENCE-D captured stderr: "skrynia push pump error: EACCES\n"
EVIDENCE-D stderr contains namespace: false
```

Pinned by `tests/test-push.js:test_push_pump_error_logging_is_path_free`, which
forces the subscription-directory failure and asserts the namespace is absent
while the failure is still surfaced.

## Defect C (operator guidance): management URLs carry secrets too

The management API is at the root, not under `/store/`, so the store location's
`access_log off` does not cover it. `SKRYNIA_TOKEN` travels in every management query
string, and `push/rules/{set,get,remove}` also carry the exact object key. The operator
guide now documents turning request logging off for the management endpoints as well.

## Guard summary

- `tests/test.js:test_store_path_neither_logs_nor_reflects_the_secret_url` — process
  streams stay clean and no store response echoes namespace, key or capability.
- `tests/test.js:test_store_error_logging_is_path_free` — internal store failure keeps the
  namespace and key out of stderr.
- `tests/test.js:test_secret_url_responses_forbid_referrer_and_retention` — every store and
  management response forbids the referrer and is `no-store`, including `405`.
- `tests/test.js:test_store_absence_does_not_disclose_namespace_existence` — an absent
  namespace is answered exactly as an absent object.
- `tests/test.js:test_plain_http_public_root_is_rejected` — a non-loopback plain-http
  public root is refused unless explicitly overridden.
- `tests/test-push.js:test_private_files_are_0600` — private files `0600` under `0700`
  directories.
- `tests/test-push.js:test_push_pump_error_logging_is_path_free` — a push pump failure
  keeps the namespace out of stderr.

`make test` passes on the candidate head in under 10 seconds.

## Residual operator responsibilities

Not enforceable from this repository; see `docs/secret-store-path-guard.md`:

- disable request-URI logging on the store **and** management locations;
- keep the store error level at `warn` or above and avoid global debug logging;
- terminate TLS and send HSTS on the store and app locations;
- keep any CDN/WAF/log-shipper from retaining store or management request paths;
- treat `DATA_DIR` backups as credential material, encrypted and access-controlled, and
  re-check `store-meta/` and `state/` permissions after a restore;
- app-side code must not log or cache store URLs.
