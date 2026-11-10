# Skrynia

A small, self-hosted platform for static web applications, public-readable
key/value storage, Web Push and delegable resource grants.

Skrynia does not interpret application data. A namespace has a root grant
issued on its first successful deployment. Grants control which keys may be
changed, which operations users may perform, and storage allowances.

The normative semantics are in
[Location authority and resource accounting](docs/intent-location-authority.md).
See [the HTTP API](docs/api-spec.md) and [deployment guide](docs/deployment.md).

## Keys and location authority

Stored keys are nonempty ASCII strings matching:

~~~regex
^[a-zA-Z0-9_]+$
~~~

A grant's immutable **locations** field uses:
- "*" for all keys;
- "report*" for keys beginning with "report";
- "report_1" for exactly "report_1";
- "" for no valid keys.

Only a trailing "*" is special. Matching is case-sensitive. Children may
only restrict their parent's locations. A child inherits its parent's
locations if omitted.

Operations are: **write**, **delegate**, **keep**, **release**, **subscribe**.
Read permission is always implicit and public. A child may only inherit or
remove parent operations; no subgrant may add privileges.

## Bootstrap an application

~~~sh
make builder
SKRYNIA_URL=http://127.0.0.1:17380/platform \
SKRYNIA_TOKEN=replace-me node src/server.js
~~~

Deploy a verified commit using the
[GitHub deployment action](action/deploy/action.yml) or an authenticated
GET /deploy request. The deployment creates the root grant only once and
returns a one-time user token and issuer token. Subsequent deployments keep
the same root grant, credentials and stored values.

Root grants default to locations "*" and all five operations.
The quota and maxObjects parameters configure their resource budgets.

Keep private root credentials out of static application bundles. For an
intentionally public writer, create a narrowly scoped child grant:

~~~http
POST /grants
Authorization: Bearer <root-user-token>
Content-Type: application/json

{
  "namespace": "example",
  "locations": "comments",
  "operations": ["write"],
  "maxBytes": 65536,
  "maxObjects": 1
}
~~~

The returned **user token** can be published to allow anyone to create,
replace, or delete "comments". It cannot delegate, vote, or subscribe.

## Store values

~~~http
POST /store/example/comments
X-Skrynia-Grant: <grant-user-token>

Hello
~~~

Reads are public:

~~~http
GET /store/example/comments
~~~

To edit the value (an atomic delete + create):

~~~http
PUT /store/example/comments
X-Skrynia-Grant: <grant-user-token>
If-Match: "<current-etag>"

Updated message
~~~

A successful edit uses the new object's full cost, not merely the size
difference. The grant and its ancestors must be able to afford the entire
resulting state. Unrelated grants are not consulted. Deleting a matching key
requires the write operation but no spare capacity.

Values have no owner or grant association. Revocation removes **all** values
at keys matching the revoked grant's locations, including values written
through other grants.

## Cleanup and lifetime

A grant user may affirm a need to keep its space (need:true) when granted
keep, or propose cleanup (need:false) when granted release.

At a 24-hour checkpoint, a release vote triggers cleanup if no keep vote was
recorded for the grant or any descendant in that interval. No votes means
keep. A descendant's keep protects ancestors.

Cleaning is atomic grant revocation followed by reissuance of the same grant,
with the same ID and credentials. Its descendants, matching values and their
subscribed resources are gone; it remains reusable.

The issuer token can clean or revoke immediately. Revocation does not
reissue the grant.

## Resource accounting

Every grant has independent limits on bytes and object count. When a write
uses grant G, check its resulting location-scoped usage and the corresponding
usage of its ancestors only. Overlapping sibling grant limits never constrain
one another's writes.

Object payloads contribute their byte lengths plus a fixed 512-byte
resource overhead per key. Grant metadata, additional user tokens, and Web
Push subscriptions also consume resource allowances.

Storage objects are plain files under store/{namespace}/{key} and may be
served directly by nginx. Strong ETags use an increasing namespace-wide
version sequence; compare-and-replace requests may use If-Match.

## Testing

~~~sh
npm ci
make test
~~~

Use a fresh data directory for a new installation.

Licensed under the GNU Affero General Public License v3.0 or later.
