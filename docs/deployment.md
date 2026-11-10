# Deploying Skrynia

Skrynia serves static application releases and a grant-controlled key/value
store. A successful first deployment creates its namespace root grant.

## Environment

| Variable | Default | Purpose |
|---|---|---|
| SKRYNIA_PORT | 17380 | HTTP listener |
| SKRYNIA_URL | required | Canonical HTTP base URL, possibly including a path |
| SKRYNIA_TOKEN | none | Management API token |
| SKRYNIA_DATA_DIR | /var/lib/skrynia | State, object values and release files |
| SKRYNIA_APP_DIR | unset | External app symlink directory |
| SKRYNIA_APP_BASE_PATH | /apps | Public app route |
| SKRYNIA_BUILDER_IMAGE | skrynia-builder:0.1.0 | Builder image |
| SKRYNIA_DEFAULT_QUOTA_BYTES | 10485760 | Root grant maxBytes |
| SKRYNIA_MAX_OBJECT_COUNT | 10000 | Root grant maxObjects |
| SKRYNIA_MAX_OBJECT_SIZE | 10485760 | Object payload maximum |
| SKRYNIA_MAX_KEY_LENGTH | 256 | Maximum key length |

The Node.js server is the only owner of its data directory. A private
reverse proxy may forward API calls; nginx may serve GET object files from
store/{namespace}/{key} directly. Use HTTPS for any request with credentials.

### Local setup

~~~sh
npm ci
make builder
SKRYNIA_URL=http://127.0.0.1:17380/platform \
SKRYNIA_TOKEN=your-management-secret \
node src/server.js
~~~

## First deployment

~~~sh
curl -sS -X POST 'http://127.0.0.1:17380/platform/deploy' \
  -H 'Authorization: Bearer your-management-secret' \
  -H 'Content-Type: application/json' \
  --data '{"repo":"git@github.com:org/app.git",
           "commit":"0123456789012345678901234567890123456789",
           "subdir":".","namespace":"myapp"}'
~~~

An authenticated deploy verifies the exact commit, runs the build in a
disposable Docker container, validates output, activates an immutable
release using an atomic symlink switch, and retains up to three releases.

It creates a root grant for the namespace if necessary, with:
- locations = "*" (or the optional locations deployment parameter)
- operations = ["write","delegate","keep","release","subscribe"]
  (or comma-separated optional operations parameter)
- maxBytes = quota parameter (or configured default)
- maxObjects = maxObjects parameter (or configured default)

Its one-time rootGrant response contains the root grant ID and its
**issuer token**, but no user token. Preserve the issuer credential in a
secrets store. Subsequent deployments return only the ID.

To obtain a root user token, call POST /grants/{id}/users?namespace=myapp
with \`Authorization: Bearer <root-issuer-token>\`. You can issue as many
distinct user tokens as the grant's resource allowance permits. Only the
issuer can do this; the \`delegate\` operation controls creation of child
grants, not user-token issuance.

The GitHub deploy action masks and exports root-grant-id and
root-issuer-token outputs. Use the issuer token to mint user credentials
separately.

## Issuing authority to the application

Root grants can create narrower children. For example, share a grant whose
locations is "comments", operations is ["write"], maxBytes is 65536 and
maxObjects is 1. It allows object writes to exactly that location without
grant delegation, votes, or subscriptions.

Do not embed an unrestricted root user token in a publicly served static app.

An application creates/edits/deletes objects using X-Skrynia-Grant with
a grant user token. Public reads and nginx static GET serving need no token.

## Release and namespace operations

Management requests use `Authorization: Bearer $SKRYNIA_TOKEN`.
`GET` routes are read-only; `POST` and `DELETE` require JSON bodies.

- GET /releases?namespace=myapp
- POST /rollback with `{"namespace":"myapp","release":"ID"}`
- GET /inspect?namespace=myapp
- GET /ns/inspect?namespace=myapp
- GET /ns/list
- GET /ns/grants?namespace=myapp
- GET /ns/keys?namespace=myapp&prefix=notes&limit=100
- POST /ns/issuer/rotate with `{"namespace":"myapp"}`
- DELETE /undeploy with `{"namespace":"myapp"}`

`scripts/cli.py` implements these operations. For example:

~~~sh
python3 scripts/cli.py ns grants --namespace myapp
python3 scripts/cli.py ns keys --namespace myapp --prefix notes
python3 scripts/cli.py ns rotate-issuer --namespace myapp
~~~

If the root issuer token was lost during an interrupted deployment, use
`ns rotate-issuer` to issue a new root issuer token. This invalidates the
old issuer credential, without affecting user tokens, grants or objects.

To list, issue or revoke individual user tokens, set
`SKRYNIA_ISSUER_TOKEN` and use the CLI `grant list-users`,
`grant issue-user` or `grant revoke-user` commands.

Undeploy removes the app release and its namespace resources.

## State and push

Data directory layout:

~~~text
releases/{namespace}/
store/{namespace}/{key}
store-tmp/
state/{namespace}/config.json
state/{namespace}/etag-sequence
state/{namespace}/grants/{id}.json
state/{namespace}/push-subs/
state/{namespace}/push-rules.json
state/_push/vapid.json
state/_push/outbox/
state/_grant-deletions/
~~~

Object keys consist solely of ASCII letters, digits, and underscore.
No per-object credentials or owner metadata are stored. Grant records are
private and must not be served as static files.

Push subscription creation requires a grant with subscribe authority and
X-Skrynia-Grant. Its independent subscription credential is used for later
subscription updates/removal. Preserve the private VAPID key across process
restarts.

## Running tests

~~~sh
npm ci
make test
~~~

See [the HTTP API](api-spec.md) and
[location-authority intent](intent-location-authority.md) for full rules.
