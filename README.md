# Skrynia

**A durable public object store with delegable, resource-bounded authority.**

Skrynia stores opaque bytes at named keys. Every object is publicly readable.
Bearer-token grants separately authorize object creation, modification, and
deletion, as well as key listing, delegation and subscriptions. A single global root grant
provides all authority. Applications, websites, indexes, logs, and versioned
releases are **client-side protocols** built on ordinary stored objects.

Skrynia itself runs one Node.js process. It never executes application code.
It has no application registry or privileged HTTP administration API.

## Start

Requires Node.js 20+ and util-linux `flock` on the server host.

```sh
npm ci
export SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia"
export SKRYNIA_URL="http://127.0.0.1:17380"
node src/server.js
```

On startup Skrynia ensures a root grant exists, creating one if absent.
The root has `locations: "*"`, all operations, and **unlimited** `maxBytes`
and `maxObjects` (represented by JSON `null`). It is a normal,
revocable grant. Revocation preserves object values; the next server start
issues a fresh root and fresh issuer credential.

The root's issuer token is stored **only in its private, mode-0600 grant file**
alongside the root record. Retrieve the root ID and issuer token on the host:

```sh
node scripts/root-credentials.js
```

The command reads `SKRYNIA_DATA_DIR`; treat its output as a secret.
The container image runs as non-root UID 1000; mounted data directories
must be writable by that user.
There is no endpoint that reveals or rotates the root issuer token.
Do not place this credential in browser JavaScript.

Issue a root user token with the issuer credential:

```sh
curl -fsS -X POST "$SKRYNIA_URL/grants/$ROOT_GRANT_ID/users" \
  -H "Authorization: Bearer $ROOT_ISSUER_TOKEN"
```

## Use the store

Store keys begin with an ASCII letter, digit, or underscore. Subsequent
characters may also be dots, with any number of consecutive dots allowed.
Slashes are not permitted.
Examples include `index.html`, `alice.app.js`, `alice.settings.json`.

```sh
curl -fsS -X POST "$SKRYNIA_URL/store/alice.index.html" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN" \
  --data-binary '<h1>Hello</h1>'

# Anybody can read it. The response includes an ETag.
curl -i "$SKRYNIA_URL/store/alice.index.html"
```

`POST` creates only if absent and requires `create`; `PUT` replaces an
existing object and requires `modify`; `DELETE` requires `delete`.
`PUT` and `DELETE` accept `X-Skrynia-If-SHA256` for byte-content
compare-and-swap. Pass the 64-character lowercase SHA-256 of the current
object bytes; a mismatch returns 412 and a malformed value returns 400.
Any static file server can provide the bytes used to compute this digest,
regardless of its own HTTP cache validators.

Skrynia's GET/HEAD sends a quoted SHA-256 ETag for cache validation and
supports `If-None-Match`, `If-Range`, HEAD and byte ranges. Listings
support `limit` / `after` pagination.
Grants can delegate these permissions independently—for example, `create`
without `modify` or `delete` lets a contributor add new keys but not
rewrite or remove existing entries.

A grant with `list` can enumerate all objects in any matching subrange:

```sh
curl -fsS "$SKRYNIA_URL/store?locations=alice.*" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN"
```

Results are sorted keys, regardless of who created them. A listing needs
a user token with `list` and its matcher must be a subset of that grant's
`locations`. It can be delegated without granting mutation access. For
participant cleanup, give each participant a separate prefix: listing
by prefix does not establish who created a key.

Skrynia serves standard extensions with their normal HTTP MIME types,
including HTML, JavaScript, CSS, and JSON. nginx or another configured static
server may serve the same object files directly from
`SKRYNIA_DATA_DIR/store/`. Keep everything else in the data directory private.
If hosting **mutually untrusted HTML applications**, isolate their browser
origins using separate hostnames; key prefixes aren't browser sandboxes.

### Delegate a subspace

```sh
curl -fsS -X POST "$SKRYNIA_URL/grants" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN" \
  -H 'Content-Type: application/json' \
  --data '{"locations":"alice.*","operations":["create","modify","delete","list","delegate","subscribe"],
           "maxBytes":1048576,"maxObjects":100}'
```

The response returns a new grant ID and **issuer token**. The issuer uses
`POST /grants/{id}/users` to issue independent user tokens. User tokens
perform the granted operations; issuer tokens issue/revoke credentials and
revoke grant subtrees. Each child may narrow its parent's key scope and
operations. Its quota is evaluated on writes through its grant or descendants.
No object belongs to a grant, and revoking a grant never deletes objects.

A client can publish a website, index, or immutable manifest within its scope.
An external deployment tool can implement atomic publication by CAS on a
single small root object, without server-side deployment logic.

### Web Push

A user token with `subscribe` authority can register interest in any
location scope contained in its own grant:

```json
{
  "locations": "alice.*",
  "events": ["create", "replace", "delete"],
  "endpoint": "https://push-service.example/subscription",
  "keys": {"p256dh": "browser-p256dh", "auth": "browser-auth"}
}
```

POST this to `/push/subscriptions` with `Authorization: Bearer`. The response
contains a subscription ID and a separate capability for updating/removing
that subscription. Skrynia sends a generic `changed` wake-up; clients fetch
current values. Delivery attempts are durable and retryable, so duplicate
wake-ups are possible; pending changes coalesce per subscription. Permanent
failures and exhaustion of the retry budget retire the subscription.
Each subscription consumes the sponsoring grant's quotas.

## Architectural properties

- **Shared memory:** Public objects are independently retrievable without
  running application-specific backend code.
- **CAS:** Byte-hash conditional mutation can serialize client-authored
  logical transactions over immutable objects and a uniquely identified
  mutable commit head.
- **Delegation:** Grants constrain operations, location prefixes, and budgets.
  Scoped `list` enables key discovery and delegated cleanup without giving
  broader enumeration authority.
- **Client protocols:** Applications define schemas, indexing, publication,
  cryptographic validation, and computation outside Skrynia.
- **One writer process:** An exclusive kernel file lock prevents multiple
  Skrynia instances from owning a data directory. HTTP API mutations
  are serialized; static readers can be served separately.

Skrynia supports CORS for bearer-authenticated browser requests from other
origins. It does not use cookie authentication.

See the [HTTP API](docs/api-spec.md),
[grant semantics](docs/intent-location-authority.md),
[client-constructed protocols](docs/client-protocols.md), and
[operations](docs/operations.md).

## Development

```sh
make test
```

Skrynia is Free Software under **AGPL-3.0-or-later**.
