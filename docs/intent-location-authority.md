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

- Valid grant-user operations: `write`, `delegate`, `subscribe`.
  A grant has an immutable set of these operations.
- A namespace root defaults to all operations.
- A child with no `operations` field inherits its parent's operations. An
  explicit child set must be a subset of its parent's. A user token can create
  subgrants only when its grant has `delegate`.
- `write` authorizes create, replace, and delete of matching object keys,
  and is always checked for all three mutations.
- `subscribe` permits browser PushSubscription creation.
- Object reads are public and have no required operation; nginx may serve
  objects directly. No object-specific write authority exists.
- Grant creation, including issuance of a namespace root, generates an
  issuer token but no user tokens. The issuer token is the only credential
  returned by grant creation.
- An issuer token can generate independent user tokens for exactly its own
  grant through POST /grants/{id}/users. Only issuers may mint these tokens,
  regardless of a grant's delegate operation. A grant may have zero users.
- Grant issuer tokens authorize revocation of their own grant subtree,
  independently of the grant's `operations`. User tokens never have
  issuer authority. `delegate` lets a user create a child grant and
  receive its issuer token.
- The issuer may revoke an individual user token by the opaque ID returned
  when it was issued, without deleting objects. Skrynia stores no labels,
  creation timestamps or user-token index that clients could maintain.
  The management authority may rotate the root issuer token without
  affecting root user tokens, descendants, or objects. The previous
  issuer credential becomes invalid immediately.
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
  No ownership moves during edits. Object sizes and prefix usage may be
  indexed in memory, provided the filesystem can reconstruct the index on
  startup and all successful mutations update the index atomically from the
  API perspective.
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

## Grant revocation

- A grant issuer can revoke its grant and all descendants. Their user
  and issuer credentials become invalid, and subscriptions sponsored by
  the subtree are removed.
- Revocation leaves **all object keys and values unchanged**. Objects have
  no grant ownership; neither the grant hierarchy nor location coverage
  determines their lifetime.
- Revoking the root removes its grant hierarchy and credentials as usual.
  Existing object values remain publicly readable. A new root may be
  issued for the namespace while preserving those values.
- Persistent, fsynced revocation intents make multi-record revocation
  recoverable after interruption. Recovery runs at startup and periodically.
- Values are removed by an authorized object deletion or by namespace
  undeployment. Ordinary grant revocation cannot delete data.

- Committed object mutations and durable push enqueue are linked by a
  recoverable intent record. Notifications are delivered at least once,
  not exactly once. Crashes between enqueue and intent finalization can
  cause duplicate notifications; a failed notification enqueue after a
  committed mutation must never be reported as a rolled-back mutation.
- Management operations require a bearer header; mutations use HTTP
  POST/DELETE with JSON bodies rather than credentials in query strings.
  Administrative listing/inspection of keys, grants and quotas must not
  reveal credential verifiers.
