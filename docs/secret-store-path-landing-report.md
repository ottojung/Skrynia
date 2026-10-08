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
- A reconciliation pass over every remaining stderr site then found and landed
  four more defects of the same family, documented as defects E-H in
  `docs/secret-store-path-audit.md`: the startup legacy migration threw errors
  embedding `ns/key`; an uncaught store read failure crashed the process with
  the full object path in stderr; the management catch-all logged `e.stack`,
  whose first line can embed the namespace; and the push subscription handlers
  rethrew filesystem errors, crashing with the record path in stderr.

## Test evidence

`make test` on the merged release head:

- Core: 52 passed, 0 failed
- Regression: 16 passed, 0 failed
- Store: 15 passed, 0 failed
- Push: 22 passed, 0 failed

Total: 105 passed, 0 failed.

## Guards landed

- `tests/test.js:test_store_path_neither_logs_nor_reflects_the_secret_url`
- `tests/test.js:test_store_error_logging_is_path_free`
- `tests/test.js:test_store_read_failure_is_path_free_and_answered`
- `tests/test.js:test_legacy_migration_failure_is_path_free`
- `tests/test.js:test_legacy_migration_filesystem_failure_is_path_free`
- `tests/test.js:test_management_filesystem_failure_is_path_free`
- `tests/test.js:test_push_subscription_failure_is_path_free`
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

## Human blocker (issue #152)

No repo-side work remains. The secret-URL audit is complete on
release/2026-09-30 at 1fe263e (origin tip). All repo-side guards are in place,
pinned by non-vacuous tests, and the full suite passes (105/105).

The following items are human-only edge items that cannot be enforced from
this repository:

1. **HSTS** — send `Strict-Transport-Security` on the store and app locations.
2. **Referrer-Policy on nginx store location** — when nginx serves store GETs
   directly from `DATA_DIR/store/`, nginx's headers replace Skrynia's. The
   `Referrer-Policy: no-referrer` must be set on the nginx store location too.
3. **Cache-Control on nginx store location** — same nginx static-file path;
   `Cache-Control: no-store` must be set on the nginx store location.
4. **Port-80 redirect** — nginx must redirect port 80 to TLS.
5. **Key-free access log** — `access_log off` on the store and management
   locations.
6. **CDN retention** — no CDN, WAF, or log shipper may retain request paths.
7. **Deploying the nginx fix** — the nginx config changes must be deployed.
8. **Backup handling** — `DATA_DIR` backups are credential material.

These are operator-owned and recorded in docs/secret-store-path-guard.md.

## Board comment

```
Secret-URL audit for #152 is landed on release/2026-09-30 (1fe263e).

Verdict: complete. 105/105 tests pass on the exact release head. All repo-side
guards are in place and pinned by non-vacuous tests. No repo-side work remains.

Remaining blockers are operator-owned and recorded in
docs/secret-store-path-landing-report.md:
- HSTS, Referrer-Policy and Cache-Control on the nginx store location,
  port-80 redirect, key-free access log, CDN retention, deploying the nginx
  fix, and backup handling.

gh is absent so no PR was opened; the merge was done locally and pushed.
```
