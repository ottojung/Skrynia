# F1 intent-record supersession — v3 docs reconciliation

Status: **agent reconciliation under owner authorization; not a user-sourced Intent Record.**
Scope: draft v3 PR #120 development branch at exact base head
`8c0c14655584a7dead3c7df5ce490d23e59f9242` only. `main`, production and other
agents' worktrees are untouched. No Rust/WASM source change, no build, no merge.

Owner authority (Antonina `w4wgIqJnlASKGYrl1UGd8H-realise`): "Kernel knows only
independently operator-supplied root Program0 and generic POST /; Program0 may
only constant-time check private token, strip it, opaque proxy/callback relay.
No named privileged Program1, no kernel v2 HTTP, no root app storage/auth/registry."
F1 source: independent exact-head review by `agent-hdAUxDQ3roEgElWj6SnmPi-factors`,
board comment `2026-11-23T07:34:55.780Z`.

## Provenance rule

- Every retired ID below keeps its verbatim `source: @ottojung` record in this
  branch's parent commit in Git history. Nothing was re-authored, re-signed, or
  attributed to a different source. Deletion is the AGENTS.md-prescribed action
  for an intent that is no longer current.
- Trims delete v2 clauses only. No new claim was added to any user-sourced record;
  the surviving sentences remain verbatim @ottojung text.
- This file and `check_intent_supersession.rs` are agent-authored, sourced to the
  board issue and F1 review — **not** to @ottojung. Do not cite them as user intent.

## Kernel promises vs optional external behavior

- Kernel promises are only `docs/v3-vm-spec.md` and `README.md` on this branch:
  binary `POST /`; independently operator-supplied root WASM via
  `SKRYNIA_ROOT_WASM`; one read-only input FD and one write-only output FD;
  no named P1; no kernel app policy/auth/grants/quota/store; no v2 HTTP facade.
- Optional external behavior is `skrynia-tools` userland WASM (v2 protocol
  compatibility; the dumb Program0 example constant-time checks a private token,
  strips it and relays) and hosting infrastructure. These are **not** kernel
  promises. See README "Specifications and release boundaries" and the retained
  `docs/intent-records/origin-isolation.md` (`$id-6947136701570090`).

## Retired CURRENT records (17 IDs) and their exact v3 supersession

| Retired `$id` | Former file (v2 kernel claim) | Exact v3 supersession |
|---|---|---|
| `$id-6065450554435260` | `web-push.md` — grant-scoped subscriptions, retry/retirement | `docs/v3-vm-spec.md` "Kernel boundary" (no grants/auth/subscriptions/implicit capability distribution); optional v2 push is `skrynia-tools` userland per README "Specifications and release boundaries" |
| `$id-3269280658856461` | `grant-mutation-operations.md` — create/modify/delete/list/delegate/subscribe ops | `docs/v3-vm-spec.md` "Kernel boundary" (no grants/routing/program IDs); "Root imports (Program 0 only)" has no grant op |
| `$id-3378973019068045` | `root-authority.md` — automatic root grant + issuer credential | `docs/v3-vm-spec.md` "Kernel boundary" (Program 0 entirely operator-supplied via `SKRYNIA_ROOT_WASM`; no root grant/issuer credential); README "Running an isolated v3 kernel" |
| `$id-9183047265512048` | `standalone-http-reads.md` — public GET/ETag/HEAD/range/static serving | `docs/v3-vm-spec.md` "HTTP and response contract" (only `POST /`, other paths 404, no body interpretation); README "Running an isolated v3 kernel" (no native health endpoint) |
| `$id-3575263406896832` | `public-reads.md` — public object reads by key | `docs/v3-vm-spec.md` "Kernel boundary" + "HTTP and response contract" (no public object keyspace; kernel does not inspect output meaning) |
| `$id-0950561916992512` | `quotas.md` — grant byte/object budgets | `docs/v3-vm-spec.md` "Kernel boundary" (no quotas/accounting) + "Safety, limits, and determinism boundaries" (physical ceilings only, `ERR_LIMIT`) |
| `$id-2651223578750229` | `scope-delta-index.md` — log-structured quota index over object names | `docs/v3-vm-spec.md` "Kernel boundary" (no lookup/accounting) + "Disk, paths, and descriptors" / "Global RAM semantics" (raw byte primitives; indexes are userland) |
| `$id-4070814447118336` | `atomic-operations.md` — object CRUD + split grant mutation ops | `docs/v3-vm-spec.md` "Global RAM semantics" (linearizable CAS), "Disk, paths, and descriptors" (atomic rename), "Root imports (Program 0 only)"; generic single-writer retained at `docs/intent-records/exclusive-writer.md` |
| `$id-7458341334509459` | `content-sha256-conditional-mutation.md` — PUT/DELETE `/store/{key}`, `X-Skrynia-If-SHA256`, ETags | `docs/v3-vm-spec.md` "HTTP and response contract" (only `POST /`; no route/header interpretation; app errors in output bytes); optional v2 HTTP protocol is userland WASM in `skrynia-tools` |
| `$id-7318492056174302` | `direct-static-serving.md` — nginx serves `GET /store/{key}` | `docs/v3-vm-spec.md` "HTTP and response contract"; retained `docs/intent-records/origin-isolation.md` is external-hosting guidance, not a kernel promise |
| `$id-5943420546985984` | `filesystem-storage.md` — public object files under `STORE_DIR` + private grants | `docs/v3-vm-spec.md` "Kernel boundary" (confined `SKRYNIA_DATA_DIR` subtree) + "Disk, paths, and descriptors"; no public/private object split in the kernel |
| `$id-0203518065789149` | `hierarchical-object-keys.md` — object keys + matchers/quota/listing/GET | `docs/v3-vm-spec.md` "Disk, paths, and descriptors" (confined UTF-8 relative paths, no `..`, <= 4096) + "HTTP and response contract" |
| `$id-5170459438610461` | `large-objects.md` — 10 GB objects, quota, SHA CAS, durable recovery | `docs/v3-vm-spec.md` "HTTP and response contract" (`SKRYNIA_MAX_REQUEST_BYTES` default 16 MiB, `413`, bounded streaming/backpressure); large-object protocols are userland WASM |
| `$id-1313845006123008` | `scoped-listing.md` — grant-scoped key enumeration | `docs/v3-vm-spec.md` "Kernel boundary" (no lookup/routing) + "Root imports (Program 0 only)" (`open_directory`/`next_entry` are raw FS enumeration, not grant-scoped key listing) |
| `$id-2422528008796077` | `skrynia-url-root.md` — `SKRYNIA_URL` public API root | README "Running an isolated v3 kernel" (`SKRYNIA_BIND`) + `docs/v3-vm-spec.md` "HTTP and response contract" (`POST /` only; no `SKRYNIA_URL`) |
| `$id-8284733156847104` | `storage-model.md` — one global key-to-bytes object mapping | `docs/v3-vm-spec.md` "Kernel boundary" + "Global RAM semantics" + "Disk, paths, and descriptors" (generic byte/RAM/file primitives, no global object keyspace) |
| `$id-7317456206882143` | `version-endpoint.md` — public `GET /version` + `/health` | `docs/v3-vm-spec.md` "HTTP and response contract" (only `POST /`, other paths 404) + README "Running an isolated v3 kernel" (TCP liveness; app health is a Program 0 protocol) |

## Retained records with v2 clauses deleted (5 IDs)

| Kept `$id` | Clauses deleted (agent trim, not new user text) | Retained generic requirement / supersession |
|---|---|---|
| `$id-4248198621776748` | "loading grants", "object or grant", "push" | Single-writer lock, fail startup, release on exit; v2 grant/notification state falls under `docs/v3-vm-spec.md` "Kernel boundary" |
| `$id-6214634120670208` | "Skrynia implements generic storage, delegated grants and Web Push, not application-defined code or release mechanisms." | Rust-only/minimal deps retained; v3 generic WASM computer per `docs/v3-vm-spec.md` "Kernel boundary" |
| `$id-6212596314252106` | grant/quota/conditional-byte-hash/public-static/Web Push/notification clauses; "All application meaning stays exclusively in clients." | Rust-only, fail-closed unsupported state, crash/concurrency qualification retained; "Kernel boundary" + "HTTP and response contract" |
| `$id-8052543016377230` | "object, authority, notification, or" qualifier | "do not weaken production durability semantics" retained; resource accounting applies to generic FS/RAM costs per "Safety, limits, and determinism boundaries" |
| `$id-6947136701570090` | "Object prefix grants and " | URL paths do not provide origin separation; hosting infrastructure/client-library responsibility — external behavior, not a kernel promise |

## Non-intent doc relabeled

- `docs/intent-location-authority.md`: `Status: normative.` → v2 compatibility
  target for optional `skrynia-tools` userland WASM, not implemented by the native
  v3 kernel. Content unchanged; current kernel contract stays
  `docs/v3-vm-spec.md` "Kernel boundary".

## Verification

Std-only Rust checker (Rust per AGENTS.md; no Python, no Cargo/Podman build).
From the repository root:

```sh
rustfmt --edition 2024 --check docs/v3/check_intent_supersession.rs
rustc --edition=2024 --test docs/v3/check_intent_supersession.rs -o /tmp/opencode/f1-check
/tmp/opencode/f1-check --nocapture
```

Executed at this exact head: rustfmt check PASS, 1 test passed. The checker uses
only `std` (`fs` and `process::Command` for read-only `git grep` against the
pinned PR120 base SHA `8c0c14655584a7dead3c7df5ce490d23e59f9242`), stays within
AGENTS.md's 50-nonblank-noncomment-line cap, and derives every ID from the
supersession tables and that exact base commit instead of hardcoding counts. It asserts that every
base ID is either retired-and-absent from live records or retained, that trimmed
IDs stay live, that no v2-only token remains in live records, that each mapped
section heading exists in its named file, and that the v3 kernel invariants
(`MUST NOT implement grants`, `POST /` only, `SKRYNIA_ROOT_WASM`, `ERR_PERM`) and
the v2-target relabel are intact. No build, no network, no production access.
Green CI at the base head is *not* a release qualification; PR stays DRAFT,
no main/production merge.
