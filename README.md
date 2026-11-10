# Skrynia

**Static web hosting with persistent storage and delegable permissions.**

Skrynia deploys web apps from Git and gives them a small, durable key–value store. Apps can read and write arbitrary bytes over HTTP without running their own backend. Skrynia doesn't interpret their data or require accounts: **grants** control where clients can write, which operations they can perform, and how much storage they can use.

- **Deploy from Git.** Build a specific commit with `make build` in Docker, publish it atomically, and roll back to an earlier release.
- **Store ordinary files.** Binary-safe values live at named keys. Reads are public (and can be served directly by nginx); conditional writes use ETags.
- **Delegate authority.** Issue restricted grants for individual keys or prefixes, with independent operations and resource budgets.
- **Manage lifetimes.** Grants can be cleaned or revoked recursively, including through periodic retention confirmations.
- **Notify clients.** Web Push subscriptions and rules connect storage events to browser notifications.

Skrynia runs as a single Node.js service. Application code remains static HTML, JavaScript, and CSS.

## Quick start

You need **Node.js 20+**, **Docker**, and **Git**. To deploy an app, the Skrynia host also needs SSH access to its Git repository.

```sh
git clone https://github.com/ottojung/Skrynia.git
cd Skrynia
npm ci
make builder

export SKRYNIA_URL=http://127.0.0.1:17380/platform
export SKRYNIA_TOKEN=replace-with-a-secret
export SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia"
node src/server.js
```

Skrynia listens on `127.0.0.1:17380` by default. Put it behind a reverse proxy when exposing it publicly, and use HTTPS for requests carrying tokens. `SKRYNIA_URL` is the API's canonical external URL; it can include a path such as `/platform`.

### Deploy an app

An app repository needs a `Makefile` whose `build` target writes its static output to `build/`. From another shell with the same `SKRYNIA_URL` and `SKRYNIA_TOKEN`:

```sh
python3 scripts/cli.py deploy \
  --namespace hello \
  --repo git@github.com:your-org/hello.git \
  --commit "$APP_COMMIT" \
  --subdir .
```

Set `APP_COMMIT` to the full commit SHA from the app repository (for example, `git rev-parse HEAD` there). The app is published at `/apps/hello/` by default. Later deployments replace the active release without discarding its storage; earlier releases remain available for rollback.

**On the first deployment**, Skrynia also issues the namespace's **root grant**, returning its ID and two secrets:

| Credential | What it allows |
| --- | --- |
| **User token** | Use the grant's permitted operations, including writing and delegating |
| **Issuer token** | Clean or revoke the grant and its descendants |

Save these secrets securely when they first appear; they are not returned on subsequent deployments. The [GitHub deployment action](action/deploy/action.yml) can deploy a commit and exposes the one-time root credentials as masked outputs.

## Give an app permission to write

Instead of handing a browser the root token, create a narrowly scoped grant. For example, authorize writes only to keys beginning with `notes`:

```sh
curl -sS -X POST "$SKRYNIA_URL/grants" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN" \
  -H "Content-Type: application/json" \
  --data '{
    "namespace": "hello",
    "locations": "notes*",
    "operations": ["write"],
    "maxBytes": 65536,
    "maxObjects": 10
  }'
```

The response contains the new grant's `userToken` and `issuerToken`. Using that **user token**:

```sh
# Create a value.
curl -sS -X POST "$SKRYNIA_URL/store/hello/notes1" \
  -H "X-Skrynia-Grant: $NOTES_USER_TOKEN" \
  --data-binary 'Hello'

# Anyone can read it; the response includes an ETag.
curl -i "$SKRYNIA_URL/store/hello/notes1"

# Replace it, optionally using the ETag to prevent lost updates.
curl -sS -X PUT "$SKRYNIA_URL/store/hello/notes1" \
  -H "X-Skrynia-Grant: $NOTES_USER_TOKEN" \
  -H 'If-Match: "<etag-from-GET>"' \
  --data-binary 'Updated'
```

The same token can delete `notes1`, but it cannot write `settings`, create subgrants, vote on retention, or register push subscriptions. A token with only `write` at one exact key is also a way to offer public writing at that key. **Anyone with that token can replace or delete its contents**, so share it deliberately.

### How grants work

Every grant combines:

| Field | Meaning |
| --- | --- |
| `locations` | `"*"` (all keys), `"notes*"` (prefix), `"notes1"` (exact), or `""` (none) |
| `operations` | Any subset of `write`, `delegate`, `keep`, `release`, `subscribe` |
| `maxBytes`, `maxObjects` | Limits on mutations performed through the grant and its ancestors |

Keys use only ASCII letters, digits, and underscores (`[A-Za-z0-9_]+`). A child grant can narrow its parent's locations and operations but can never expand them. Reading needs no grant and is always public—do not store private data expecting access control on `GET`.

Grants authorize **actions**, not ownership of values. A write must fit the resulting usage of the writing grant and its ancestors; unrelated grants never veto it. A replacement is logically an atomic delete followed by a creation, so the *entire replacement value* must fit. Objects don't carry creator or grant identities.

### Retention and revocation

Every 24 hours, a grant checks its retention confirmations. A `release` confirmation proposes cleaning its space; a `keep` confirmation from that grant **or any descendant** prevents it. Silence leaves everything intact.

- **Clean:** Delete all keys matching the grant's locations and all descendant grants, then reissue the same grant with unchanged credentials and fresh voting state.
- **Revoke:** Do the same without reissuing the grant; its credentials cease to work.

An issuer can perform either action immediately. Because cleanup follows **locations**, it can remove matching values written through an unrelated grant with overlapping scope. This is intentional.

## Integration and operations

The browser helper is served at `/client/skrynia.js`. For example, after loading it:

```js
(async () => {
  const store = Skrynia.store('hello', notesUserToken);
  await store.create('notes2', 'A note');
  const result = await store.get('notes2');
  await store.put('notes2', 'Revised', { etag: result.meta.etag });
})();
```

Web Push subscriptions can be registered with a grant that has `subscribe`; configurable rules route object changes to notification channels. Skrynia keeps subscription state and delivery work durable.

Persistent state lives in `SKRYNIA_DATA_DIR` (default `/var/lib/skrynia`). Run **one Skrynia server per data directory**. Object files under `store/{namespace}/{key}` can be served directly by nginx; keep grant records and other private state inaccessible.

For details, see the [HTTP API](docs/api-spec.md), [deployment guide](docs/deployment.md), and [normative grant semantics](docs/intent-location-authority.md).

## Development

```sh
npm ci
make test
```

Skrynia is Free Software, licensed under **AGPL-3.0-or-later**.
