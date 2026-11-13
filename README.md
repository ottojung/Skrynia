# Skrynia

Skrynia is a durable public object store with delegable, resource-bounded
authority, implemented entirely in Rust.

Objects are opaque byte strings addressed by flat keys. Reads are public;
authenticated, scoped grants independently authorize creation, modification,
deletion, listing, delegation, and Web Push subscriptions. Skrynia implements no
application-specific schemas, business logic, or privileged deployment API.

## Build and run

Install Rust 1.93 or newer, an OpenSSL development package, and pkg-config.

```sh
make test
cargo build --locked --release --package skrynia-server
export SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia"
export SKRYNIA_URL="http://127.0.0.1:17380"
target/release/skrynia-server
```

The server creates an unrestricted root grant if none exists. To inspect
the root ID and issuer credential **on the authorized host**, run:

```sh
SKRYNIA_DATA_DIR="$HOME/.local/share/skrynia" \
    target/release/skrynia-server root-credentials
```

The output is a secret JSON object with `id` and `issuerToken`.
Never publish it or embed it in untrusted content. Root credentials remain
in the private data directory, not in an HTTP endpoint.

## Basic HTTP operations

Use the root issuer credential to obtain a separate user token:

```sh
curl -fsS -X POST "$SKRYNIA_URL/grants/$ROOT_GRANT_ID/users" \
  -H "Authorization: Bearer $ROOT_ISSUER_TOKEN"
```

With the resulting user token:

```sh
curl -fsS -X POST "$SKRYNIA_URL/store/alice.index.html" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN" \
  --data-binary '<h1>Hello from Skrynia</h1>'

curl -i "$SKRYNIA_URL/store/alice.index.html"
```

`POST` creates only if absent; `PUT` updates existing keys; `DELETE`
removes them. `PUT` and `DELETE` optionally accept
`X-Skrynia-If-SHA256` with the current lowercase SHA-256 of the bytes.
A mismatch returns HTTP 412. The HTTP reader provides a quoted SHA-256
ETag and supports conditional requests, HEAD, and single byte ranges.

To delegate a limited key prefix:

```sh
curl -fsS -X POST "$SKRYNIA_URL/grants" \
  -H "Authorization: Bearer $ROOT_USER_TOKEN" \
  -H 'Content-Type: application/json' \
  --data '{"locations":"alice.*","operations":["create","modify","delete","list"],"maxBytes":1048576,"maxObjects":100}'
```

Child grants can narrow scope and operations but never extend them. Creating
a child returns its issuer credential, which separately issues user tokens.
Revoking a grant invalidates descendants and subscriptions, but preserves
public object bytes. Each grant's quotas include its scoped storage and
delegated metadata.

## Hosting and notifications

Objects are ordinary files under `SKRYNIA_DATA_DIR/store/`. A static
server can serve them directly; never publish `state/`. Independent,
mutually untrusted publishers require **separate HTTPS browser origins**.
See [the nginx example](examples/nginx-origins.conf).

Web Push subscriptions are grant-scoped and use a separate update capability.
Notifications carry a generic `changed` wake-up; recipients fetch current
values. Delivery state persists across restarts and duplicate wake-ups are
possible. The server validates public HTTPS endpoints and pins DNS resolution.

The container image runs as UID 1000. Build and check it with:

```sh
make docker-build
```

## Documentation

The [HTTP API](docs/api-spec.md), [grant semantics](docs/intent-location-authority.md),
[client protocols](docs/client-protocols.md), [operations](docs/operations.md),
and [verification](docs/verification.md) specify the current system.
Intent Records in `docs/intent-records/` capture the live requirements.

Skrynia is Free Software under **AGPL-3.0-or-later**.
