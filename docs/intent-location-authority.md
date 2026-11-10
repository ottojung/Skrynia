# Intent: location-scoped grant authority and resource accounting

Status: normative. This document defines Skrynia's storage model.

## Names and matchers

- Object keys consist exclusively of one or more ASCII `[a-zA-Z0-9_]` characters.
  A key must be nonempty and no longer than the configured maximum.
- `locations` has grammar
  ```
  locations -> prefix | "*"
  prefix    -> [a-zA-Z0-9_]+ "*" | [a-zA-Z0-9_]* 
  ```
  More precisely: `^[a-zA-Z0-9_]*\\*?$`.
- A pattern ending in `*` matches all valid keys starting with the preceding
  characters. Any pattern without `*` matches its exact key. `""` matches
  no valid keys; `"*"` matches every valid key.
- Matching is bytewise, ASCII, and case-sensitive. No escaping, nesting,
  regular-expression rules, or implicit separators exist.
- Every grant has one immutable `locations` pattern. A namespace root defaults
  to `"*"`. A child with no explicit `locations` inherits its parent's.
- Child locations must denote a subset of the parent's matched valid keys.
  A restrictive child may not widen its parent's authority.

## Operations

- Valid grant-user operations: `write`, `delegate`, `keep`, `release`,
  `subscribe`. A grant has an immutable set of these operations.
- A namespace root defaults to all operations.
- A child with no `operations` field inherits its parent's operations. An
  explicit child set must be a subset of its parent's. A user token can create
  subgrants only when its grant has `delegate`.
- `write` authorizes create, replace, and delete of matching object keys,
  and is always checked for all three mutations.
- `keep` permits `need:true` retention confirmation; `release` permits
  `need:false`. `subscribe` permits browser PushSubscription creation.
- Object reads are public and have no required operation; nginx may serve
  objects directly. No object-specific write authority exists.
- Grant creation, including issuance of a namespace root, generates an
  issuer token but no user tokens. The issuer token is the only credential
  returned by grant creation.
- An issuer token can generate independent user tokens for exactly its own
  grant through POST /grants/{id}/users. Only issuers may mint these tokens,
  regardless of a grant's delegate operation. A grant may have zero users.
- Grant issuer tokens also authorize unconditional recursive revocation
  or cleaning; user tokens never have issuer authority.
- An intentionally public user token with `locations: "some_key"` and
  `operations: ["write"]` provides public write authority only for that key.

## Values, edits, and budgets

- Each object is a value at one key in one namespace. Object metadata does
  not contain any grant identifier or per-object write capability.
- Let `objects(g, S)` be the live objects in state S whose keys match grant
  g's `locations`. Let `descendants(g)` include g itself.
- Charged bytes for a grant in state S:
  ```
  sum(key-value payload bytes + OBJECT_COST for keys matched by locations(g))
  + GRANT_COST for each grant in descendants(g)
  + EXTRA_USER_COST for each user token beyond the first in descendants(g)
  + PUSH_SUB_COST for each subscription sponsored by descendants(g)
  ```
  Charged object count is the number of matching live keys plus the number
  of subscriptions sponsored by descendants(g). Grant metadata consumes bytes
  but not the object-count budget.
- Overlapping location scopes may measure the same physical object under
  multiple independent grants. This is not a second physical copy.
- For every write through grant g, check g's `write` operation and location.
  Check resulting charged bytes and object counts against the maxima of g
  and all transitive ancestors, and **no unrelated grant**.
- Grants impose limits on actions through their own authority, not perpetual
  global invariants on every possible namespace state. Unrelated writes may
  leave a grant's measured usage higher than its maxima.
- Edit is semantically `atomic(delete(key); create(key, newValue))`, with the
  same grant authorizing both steps, and may be implemented as an efficient
  atomic replacement. First conceptually remove the old value, then check
  creation of the entire new value against g's and its ancestors' budgets.
  No exception exists for a size-reducing replacement that still exceeds the
  charged maximum. A failed edit leaves the original value intact.
- Direct deletion by a grant with `write` authority removes the matching
  object regardless of its current resource usage, and doesn't need byte
  headroom. Creating a fresh key charges the full new value and object.
- The writer's identity is not recorded as an object owner or sponsor.
  No ownership moves during edits.
- The overall maximum size of a single HTTP request remains an independent
  server-wide limit. ETag/If-Match conditional replacement remains supported.

## Delegation and subscription resources

- Parent grant capacity limits issuing new grant metadata. The child's
  `maxBytes` and `maxObjects` do not reserve capacity in the parent.
- A newly issued grant need not have currently observed location usage under
  its own cap. Its future writes must satisfy the transition rule above.
- Subscriptions are non-object resources; they are charged to their creating
  grant and its ancestors, and must be created with `subscribe`. They retain
  their own update/removal credential.
- Subscriptions are removed when their sponsoring grant's subtree is revoked.
  Subscription channels are not object keys and are not interpreted through
  `locations`.

## Revocation and retention

- Revoking grant g recursively destroys g and its descendants, invalidates
  all their user and issuer tokens, removes subscriptions sponsored by the
  subtree, and deletes **every live object matching locations(g)**. No
  object-to-grant link is needed.
- Objects authorized by overlapping unrelated grants may also be removed
  when the revoked grant matches their keys. Unrelated grant records and
  credentials survive; their observable usage changes with the object set.
- Descendant location scopes are subsets of g, so removing g's matching
  objects also removes all objects matchable by its descendants.
- Voting-triggered cleanup is semantically the atomic operation
  `revoke(g); issueSame(g)`: the same grant ID, parent, `locations`,
  `operations`, budget limits, user-token verifiers and issuer-token
  verifier are preserved. Descendant grants, their credentials, matching
  objects and sponsored subscriptions are gone. The replacement grant starts
  with a fresh confirmation window.
- Explicit issuer-triggered cleaning has the same semantics as voting-triggered
  cleaning, except that it is immediate and does not depend on votes.
- At each grant's 24-hour checkpoint, cleanup occurs only if the grant
  received at least one release confirmation in the window and neither it
  nor any descendant had a keep confirmation in that window.
  Silence preserves data. Descendant keep protects ancestors, not siblings.
  Issuer revocation/cleaning overrides confirmations.
- The intermediate revoked state is never observable to Skrynia API users.
  Interrupted filesystem deletion is resumed before serving requests.
  Implementations may optimize physical deletion while preserving these
  semantics.

## Out-of-scope assumptions

- There is no historical data reader, compatibility API, alternate write mode,
  per-object write token, or data migration in the server.
- All namespace storage and grant rules are defined by this specification.
