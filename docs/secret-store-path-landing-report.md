# Landing report: secret store path audit (Kawun issue #152)

Date: 2026/10/08
Branch: fix/152-secret-path-guards
Release branch: release/2026-09-30

## Verdict

The secret-URL audit is complete and landed. All repo-side guards are in place,
pinned by non-vacuous tests, and the full suite passes on the exact release head.

## What was assessed

- Local `fix/152-secret-path-guards` had diverged from origin (6 local vs 5 remote
  commits) but trees were identical — a rebase with no content change. Local reset
  to origin tip `e758537`.
- `fix/152-namespace-oracle` (`88961a4`) is pushed; local matches origin exactly.
- `release/2026-09-30` is the active release branch (not yet promoted to `main`).
- The fix branch was merged into `release/2026-09-30` and pushed (`885b4f8`).
- A remaining repo-side gap was found and landed: the push pump error handler
  logged `e.message`, which embeds the namespace on filesystem failures. Fixed to
  log a stable error code only, pinned by
  `tests/test-push.js:test_push_pump_error_logging_is_path_free`.

## Test evidence

`make test` on the merged release head:

- Core: 47 passed, 0 failed
- Regression: 16 passed, 0 failed
- Store: 15 passed, 0 failed
- Push: 22 passed, 0 failed

Total: 100 passed, 0 failed.

## Guards landed

- `tests/test.js:test_store_path_neither_logs_nor_reflects_the_secret_url`
- `tests/test.js:test_store_error_logging_is_path_free`
- `tests/test.js:test_secret_url_responses_forbid_referrer_and_retention`
- `tests/test.js:test_store_absence_does_not_disclose_namespace_existence`
- `tests/test.js:test_plain_http_public_root_is_rejected`
- `tests/test-push.js:test_private_files_are_0600`
- `tests/test-push.js:test_push_pump_error_logging_is_path_free`

## Remaining blockers (human / externally owned)

These are not enforceable from this repository. Each is a deployment or operator
decision.

1. **Port-80 redirect** — nginx must redirect port 80 to TLS; Skrynia listens on
   `127.0.0.1` clear HTTP only.
2. **HSTS** — send `Strict-Transport-Security` on the store and app locations.
3. **Referrer-Policy on nginx store location** — when nginx serves store GETs
   directly from `DATA_DIR/store/`, nginx's headers replace Skrynia's. The
   `Referrer-Policy: no-referrer` must be set on the nginx store location too.
4. **Cache-Control on nginx store location** — same nginx static-file path;
   `Cache-Control: no-store` must be set on the nginx store location.
5. **Key-free access log** — `access_log off` on the store location and on the
   management location (which carries `SKRYNIA_TOKEN` and object keys in query
   strings).
6. **CDN retention** — no CDN, WAF, or log shipper may sit in front of the store
   location with request-path retention.
7. **Deploying the Skrynia nginx fix** — the nginx config changes above must be
   deployed to the live reverse proxy.
8. **Backup handling** — `DATA_DIR` backups are credential material; encrypt,
   access-control, and verify `store-meta/` and `state/` permissions after restore.

## Board comment

```
Secret-URL audit for #152 is landed on release/2026-09-30 (885b4f8).

Verdict: complete. 100/100 tests pass on the exact release head. All repo-side
guards are in place and pinned by non-vacuous tests.

Remaining blockers are operator-owned and recorded in
docs/secret-store-path-landing-report.md:
- port-80 redirect, HSTS, Referrer-Policy and Cache-Control on the nginx store
  location, key-free access log, CDN retention, deploying the nginx fix, and
  backup handling.

gh is absent so no PR was opened; the merge was done locally and pushed.
```
