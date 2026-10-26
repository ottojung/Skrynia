# Skrynia

A minimal, self-hosted platform for deploying static web apps with
grant-accounted persistent key-value storage and Web Push.

**This release is intentionally incompatible with earlier persisted storage.
There are no migrations. Deploy it against a fresh data directory.**

## Concept

Skrynia does not execute application-specific server code. Browser apps store
arbitrary bytes and independently interpret/validate them. Skrynia enforces
only generic permissions, atomicity, retention, and resource limits.

All persistent app-created storage must be funded by a **grant**:

- The first successful deployment of an app creates a **root grant**.
- Each grant has one **issuer token** (unconditional cleanup or revocation)
  and one or more **user tokens** (object creation, subgrants, retention
  confirmation and issuing additional user tokens).
- Creating a subgrant charges its metadata against its parent's allowance.
  Every subtree's actual consumption is bounded by its grant and all ancestors.
- Independent write capabilities still protect the contents of individual
  stored objects; user tokens authorize *new allocation*.
- Cleanup removes all stored objects and descendant grants but **preserves the
  cleaned grant and its tokens**. Full revocation removes even that grant.

### 24-hour retention decisions

Grant users may signal `need:true` (keep) or `need:false` (propose cleanup).
At the grant's next 24-hour checkpoint it is cleaned if at least one release
was proposed and **nobody in its entire subtree** affirmed a need to keep it.

A descendant's affirmative confirmation protects every ancestor. Total
silence preserves everything. Issuer actions override confirmations.

This protocol does not assume token holders represent individual people;
they may represent apps, scripts, custodians or other resource users.

## Deploy and bootstrap a root grant

```sh
make builder
SKRYNIA_URL=http://127.0.0.1:17380/platform \
SKRYNIA_TOKEN=replace-me node src/server.js
```

`SKRYNIA_URL` is the complete canonical HTTP root. Any path component is
supported; all paths below are relative to it.

```sh
curl 'http://127.0.0.1:17380/platform/deploy?repo=git@github.com:myorg/myapp.git&commit=0123456789012345678901234567890123456789&subdir=.&namespace=myapp&token=replace-me'
```

The deploy response includes a `rootGrant` containing `id`,
`issuerToken`, and `userToken` **on first successful deployment only**.
Store the secrets immediately. Deployments of the same namespace retain its
existing root grant and do **not** return the tokens again.

Deployment accepts optional `quota` and `maxObjects` for the initial root.
Defaults are `SKRYNIA_DEFAULT_QUOTA_BYTES=10485760` and
`SKRYNIA_MAX_OBJECT_COUNT=10000`.

The [bundled GitHub deploy action](action/deploy/action.yml) exposes
`root-grant-id`, `root-user-token`, and `root-issuer-token` as masked
one-time action outputs.

## Create storage with a grant

```http
POST /store/myapp/hello
X-Skrynia-Grant: <root-user-token>
X-Skrynia-Mode: capability-write

Hello world
```

The response contains a write capability for `PUT`/`DELETE` on that object.
`GET` remains publicly readable. `If-Match` supports atomic
compare-and-replace using the returned ETag. `public-write` mode permits
any caller to update/delete that particular object.

To create a subgrant:

```http
POST /grants
Authorization: Bearer <parent-user-token>
Content-Type: application/json

{"namespace":"myapp","maxBytes":5242880,"maxObjects":100}
```

Returns an independent child `id`, `issuerToken`, and `userToken`.

To send a retention confirmation:

```http
POST /grants/<id>/confirmation?namespace=myapp
Authorization: Bearer <user-token>
Content-Type: application/json

{"need":true}
```

To clean a grant (including all descendants), while retaining its authority:

```http
DELETE /grants/<id>/space?namespace=myapp
Authorization: Bearer <issuer-token>
```

To revoke the grant and all its descendants, including all tokens:

```http
DELETE /grants/<id>?namespace=myapp
Authorization: Bearer <issuer-token>
```

See [the complete HTTP API](docs/api-spec.md) for grant-user issuance,
inspection, resource accounting, voting intervals, and error semantics.

## Deployment architecture

Deploys clone exact SSH Git commits, check out and verify the full hash, run
`make build` inside a disposable Docker builder, validate the static output,
and atomically activate a new release via symlink swap. Up to three releases
are retained for rollback. A build failure leaves the active app unchanged.

- Node.js single-process server, one owner per data directory
- Raw storage files under `store/{namespace}/{key}` (nginx may serve GETs)
- Private per-object capabilities and grant records under separate state dirs
- Strong static-file-compatible ETags with a namespace-wide monotonic version
  counter: deleting an object removes its metadata, without leaving tombstones
- Recursive cleanup is restart-recoverable using persistent deletion records
- Durable Web Push notifications for store mutations, with subscription
  creation charged to a grant

Management routes (token-authenticated `GET`) are `/deploy`, `/undeploy`,
`/rollback`, `/releases`, `/inspect`, `/ns/inspect`, `/ns/list`,
and `/push/rules/{set,get,list,remove}`.

Storage operations: `GET`, `POST`, `PUT`, `DELETE` on
`/store/{namespace}/{key}`. Web Push registration requires a grant user
token on first registration. The small browser library is served at
`/client/skrynia.js`.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `SKRYNIA_PORT` | `17380` | HTTP listener |
| `SKRYNIA_URL` | required | Canonical HTTP root, including path |
| `SKRYNIA_TOKEN` | none | Management token |
| `SKRYNIA_DATA_DIR` | `/var/lib/skrynia` | Persistent files |
| `SKRYNIA_APP_BASE_PATH` | `/apps` | App serving base |
| `SKRYNIA_APP_DIR` | none | External app symlinks |
| `SKRYNIA_BUILDER_IMAGE` | `skrynia-builder:0.1.0` | Build container |
| `SKRYNIA_DEFAULT_QUOTA_BYTES` | `10485760` | Root grant byte limit |
| `SKRYNIA_MAX_OBJECT_COUNT` | `10000` | Root object-count limit |
| `SKRYNIA_MAX_OBJECT_SIZE` | `10485760` | Per-object payload limit |

## Testing

```sh
npm ci
make test
```

Licensed under the GNU Affero General Public License v3.0 or later.
