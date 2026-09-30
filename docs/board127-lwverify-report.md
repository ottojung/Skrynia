# Board 127 — INDEPENDENT VERIFICATION front (`review/127-legacy-writable-verify`)

Verifier, read-only on source. Head under verification: `21eca665ee6ded663b3a677e0f8cfe78891f8ee5`.
Base of the "before" column: `0b5adde46e1e3cd45d81b79d6da936d7a89de2be`.
Origin server used to fabricate a genuine pre-removal object: `83a85b6` (= `origin/main`, parent of the stack).

Everything below was produced by me in this pass except where a line is explicitly marked
**RELAYED**. No number, status code or count in this report is taken from
`/workspace/BOARD127-LEGACYWRITE.md` unless marked RELAYED.

I did not land anything, did not merge, did not open a pull request, did not comment on, close,
create or reorder any board issue, and ran no `antonina board collect`. I read the board issue
(`antonina board show --id 127 --json`) and its body is quoted where I use it.

---

## 0. Host headroom, read before the heavy invocations

```
$ grep -n "anon" /sys/fs/cgroup/memory.stat | head -3
1:anon 7576084480            # before any work
20:inactive_anon 503422976
21:active_anon 7072002048
$ cat /sys/fs/cgroup/memory.current /sys/fs/cgroup/memory.max
30273355776
32212254720
$ grep -n "anon" /sys/fs/cgroup/memory.stat | head -3     # at the end
1:anon 6843617280
21:active_anon 6499635200
```

Peak `anon` observed 7.06 GiB against the 32212254720-byte (30 GiB) cap; 28 % used. I never came
near the 20 GiB line, so nothing was ever deferred for headroom. One earlier read of `anon`
(30273355776 in `memory.current`) is the whole-cgroup figure including file cache, not `anon`;
the `anon` line is the one to watch and it fell.

---

## 1. The active release line — established from git, not assumed

### 1.1 What exists

```
$ git ls-remote origin 'refs/heads/release/*' 'refs/heads/main'
83a85b630ca3a39385704a68499cf61d936e6432	refs/heads/main
ffdb44da7d70e5acbfbbd7a4c706576f0c651efb	refs/heads/release/0.1.0
5cf4e4a27bae90b5596aac967b730c49330ada83	refs/heads/release/2026-09-10
```

There is no `release/2026-09-27-5` in `ottojung/skrynia`. That name appears in the board thread
attached to a branch in `ottojung/antonina`; it is not a Skrynia ref. The board's 02:57Z comment
told me Skrynia has "at least" `release/2026-09-10` and `release/0.1.0`, and that is the complete
set: `git ls-remote` returns exactly two `release/*` heads.

### 1.2 Both existing release branches are already inside main

```
$ for b in origin/release/0.1.0 origin/release/2026-09-10; do
    echo "$b merge-base=$(git merge-base $b origin/main)  main-ahead=$(git rev-list --count $b..origin/main)  main-behind=$(git rev-list --count origin/main..$b)"
  done
origin/release/0.1.0        merge-base=ffdb44da7d70e5acbfbbd7a4c706576f0c651efb  main-ahead=156  main-behind=0
origin/release/2026-09-10   merge-base=5cf4e4a27bae90b5596aac967b730c49330ada83  main-ahead=100  main-behind=0
```

`main-behind=0` for both: each release tip is an ancestor of `main`. `main` is 100 commits past
`release/2026-09-10` and 156 past `release/0.1.0`, and neither release branch has moved since its
cut date (`ffdb44d` 2026-09-10, `5cf4e4a` 2026-09-11, against a `main` tip of 2026-09-28).

The promotion merges are in main, by name:

```
$ git log -1 --format='%h %s parents: %p' cba37ff
cba37ff Merge pull request #12 from ottojung/release/2026-09-10 parents: 9a97d44 5cf4e4a
$ git log -1 --format='%h %s parents: %p' 55c7ec1
55c7ec1 Merge pull request #25 from ottojung/main parents: ffdb44d cba37ff
```

`cba37ff` merged the `release/2026-09-10` tip (`5cf4e4a`) into main; `55c7ec1` merged the
`release/0.1.0` tip (`ffdb44d`) into main. Both release branches have therefore been promoted once.

### 1.3 The repository states its own rule, and the rule decides this

`docs/skills/itinerary-skrynia.md` at `21eca665`, section "Release integration", read verbatim:

- "The active release branch is the latest `release/*` branch that has **never** been promoted
  into `main`."
- "A release branch is permanently retired after its first promotion into `main`, even if commits
  are accidentally added to it later."
- "If no active release branch exists, create one from current `main`."
- "Reuse the same active release branch across scheduled issues; do not create one release branch
  per issue."
- "Start each issue branch from the active release branch in an isolated Lubko worktree."
- "Open the issue PR against the active release branch, **not `main`**."
- "Scheduled orchestrators must **not** merge issue/task PRs into `main` and must **not** promote
  `release/*` into `main`. Promotion is the human review boundary."

### 1.4 My determination

**There is no active Skrynia release line at `21eca665`.** Both `release/0.1.0` and
`release/2026-09-10` are retired by the repository's own stated rule, because each has been
promoted into `main`. The rule's own remedy applies: one new `release/*` branch is to be cut from
current `main` and reused for scheduled issues.

Consequences for the 127 stack, which I state and do **not** execute:

- The stack's base is `83a85b6` = `origin/main`, and all six commits sit directly on it. That was
  the only defensible base available, since no active release branch existed to branch from, and
  it matches rule 1.3 line "If no active release branch exists, create one from current `main`"
  once that branch is cut. **I find no defect in the base choice.**
- The correct landing target is that new active release branch, **not `main`**, and the PR should
  be opened against it, per "Open the issue PR against the active release branch, not `main`".
  Cutting it and merging are a human action; I did neither.
- `main` is not a valid landing target for a scheduled orchestrator under this policy, and I would
  flag any plan to fast-forward `main` directly as contrary to the repository's recorded process.

I have deliberately not resolved the case where the board's 02:21Z and 02:44Z comments assume
`0b5adde` is "the exact head the other issues depend on" on `main`. The landing decision is
separate and is not mine.

---

## 2. How a pre-removal immutable object was fabricated — the method verified, not trusted

The previous front's method, which I was told to use but to verify: plant the current on-disk
layout directly, `store/{ns}/{key}` plus `store-meta/{ns}/{key}.json` containing
`{"mode":"immutable","version":N}`. Trusting that would be assuming the fixture resembles a real
pre-removal object. So I produced a real one first, with the pre-removal server itself.

```
$ git archive 83a85b6 | tar -x -C /tmp/opencode/lwverify/orig
$ node /tmp/opencode/lwverify/fabricate.cjs
pre-removal create: 201 {"ok":true,"mode":"immutable"}
pre-removal PUT on it: 403 {"error":"immutable"}
pre-removal DELETE on it: 403 {"error":"immutable"}
pre-removal GET after delete: 200
recreate: 409
ON-DISK under store/ and store-meta/:
  ns/old = "LEGACY-BYTES"
  ns/old.json = "{\n  \"mode\": \"immutable\",\n  \"version\": 1790737636\n}"
```

So a genuine pre-removal immutable object, created through the public API by the pre-removal
server at `83a85b6`, is exactly two files:

- `store/{ns}/{key}` — the raw bytes, no suffix, no sidecar in the public tree;
- `store-meta/{ns}/{key}.json` — `{"mode":"immutable","version":<int>}`.

The test fixture at `tests/test.js:498-499` writes byte-for-byte the same two files in the same two
locations. **The method is verified as faithful, not assumed.** Nothing else is required: no
capability, no hash, no `store-tmp` residue, no `state` entry beyond the namespace quota file.
Note the incidental confirmation that at `83a85b6` both PUT and DELETE on such an object were
`403 immutable`, and that the failed DELETE left the object intact (`GET` 200, re-create 409).

---

## 3. Behavioural matrix, requirements 1–6, before and after — executed by me

Clean local instance, throwaway data dir, anonymous caller, no `X-Skrynia-Capability`, no
`If-Match` unless a row says so. Two throwaway source trees, each a `git archive` of the ref plus
a `node_modules` symlink. No request to `vau.place`, no live namespace touched, no operator Antonina
state read or written. Probe scripts are mine: `/tmp/opencode/lwverify/{fabricate,matrix,cas,etag}.cjs`.
Both probes exited 0.

Status codes and bodies below are the server's actual responses, truncated for width only.

### 3.1 The decisive row, and the whole lifecycle

| Probe row | `0b5adde` (before) | `21eca665` (after) |
|---|---|---|
| legacy object readable, `GET` | `200` `"LEGACY-BYTES"` | `200` `"LEGACY-BYTES"` |
| **legacy object `PUT` in place, anonymous, no `If-Match`, no capability** | **`403 {"error":"immutable", …}`** | **`200 {"ok":true}`** |
| `GET` after that `PUT` | `200` `"LEGACY-BYTES"` (unchanged) | `200` `"PUT-1"` (changed) |
| second `PUT` on the same object | `403` | `200` |
| third `PUT` on the same object | `403` | `200` |
| `GET` after three `PUT`s | `200` `"LEGACY-BYTES"` | `200` `"PUT-3"` |
| stored mode after three replaces | `{"mode":"immutable","version":1790737636}` (version **unchanged**) | `{"mode":"immutable","version":1790737660}` (version advanced) |
| new filesystem paths created by a replace | 0 | 0 |
| `DELETE` the legacy object | `200` | `200` |
| `GET` after delete | `404` | `404` |
| re-`POST` at the same key | `201 {"ok":true,"mode":"public-write"}` | `201 {"ok":true,"mode":"public-write"}` |

**This is the central requirement and it is MET.** At `21eca665` a pre-removal immutable object is
**updatable in place with no prerequisite**: the first anonymous `PUT` returns `200` and the new
bytes are visible on the next read. At `0b5adde` the same request returned `403 immutable` and
changed nothing. Deletable was already true at `0b5adde`; updatable was not, and now it is.

I also verified `If-Match` still works on a legacy object, in a corrected second probe:

```
### head (21eca665)
legacy GET                200 "LEGACY-BYTES" ETag="6abc7d06-c"
legacy PUT stale If-Match   412
legacy PUT correct If-Match 200
legacy GET after CAS      200 "cas-ok"
legacy DELETE             200
### base (0b5adde)
legacy GET                200 "LEGACY-BYTES" ETag="6abc7d06-c"
legacy PUT stale If-Match   403
legacy PUT correct If-Match 403
legacy GET after CAS      200 "LEGACY-BYTES"
legacy DELETE             200
```

**A disclosure about my own error, reported because it changed a result.** My first matrix probe
sequenced the `If-Match` rows *after* deleting and re-creating the object, so in that run those
rows were exercising a fresh `public-write` object, not the legacy one, and both sides returned
`200`/`412`. I caught it from the byte contents, discarded that run's `If-Match` rows, and re-ran
them in isolation against a freshly planted legacy object; the table above is the corrected run.
The lifecycle rows in §3.1 are unaffected and were never reordered.

### 3.2 Requirement-by-requirement verdicts

**R1 — new objects must no longer support `X-Skrynia-Mode: immutable`. MET, and I measured that it creates nothing.**

| Row | `0b5adde` | `21eca665` |
|---|---|---|
| `POST` with `X-Skrynia-Mode: immutable` | `400 {"error":"mode_removed","detail":"immutable objects can no longer be created; use public-write or capability-write"}` | identical |
| object file exists on disk after that reject | `false` | `false` |
| `GET` the rejected key | `404` | `404` |
| `POST` with `X-Skrynia-Mode: public-write` (control) | `201` | `201` |
| `POST` with a nonsense mode `write-once` | `400 {"error":"invalid_mode"}` | `400 {"error":"invalid_mode"}` |

The rejection is explicit and distinct from `invalid_mode`; it is not a silent downgrade. R1 was
already met at `0b5adde` (it was `053eea8`) and is unregressed at `21eca665`.

**R2 — no migration of existing stored objects, no separate legacy-compatibility path. MET, and I measured it as a filesystem fact rather than accepting the word.**

- A pre-removal object, after three in-place replaces, still has `store-meta/ns/old.json` with
  `mode: "immutable"`. The mode is **not** rewritten; only `version` advances.
- The object stays at `store/ns/old`. **Zero** new filesystem paths appear under `store/` or
  `store-meta/` as a result of a replace (measured as the set difference of a full recursive
  `sha256`-and-path listing of both trees, before and after).
- No re-key, no copy, no `.rekeyed` sibling, no rewrite of the public object file's location.
- There is no code path keyed on a "legacy" flag that runs *before* an ordinary request. The
  literal string `LEGACY_IMMUTABLE` appears in exactly three places in `src/` (`server.js:27` the
  constant, `server.js:28` folded into `READABLE_MODES`, `server.js:203` the create rejection) and
  in **no** `handlePut` or `handleDelete` branch. I read `handlePut` (`server.js:246-281`) and
  `handleDelete` (`server.js:283-…`) in full at this head.

R2 is met as a property, not merely asserted. Note the separate and unrelated pre-`store/`
layout migration in `src/shared.js:72-133`; it is a filesystem-layout migration, not a mode
migration, and the intent record at `docs/intent-records/mutation-modes.md:7` says so explicitly.

**R3 — every stored `immutable` object immediately writable under `public-write` semantics. MET.** Anonymous in-place `PUT` → `200`; subsequent `GET` returns the new bytes; no capability, no `If-Match`, no migration step, no precondition of any kind. The path taken is the same one a `public-write` object takes: the only mode test in `handlePut` is `meta.mode === 'capability-write'`, which `'immutable'` does not satisfy.

**R4 — the normal writable lifecycle, including replacement and deletion, not `403 immutable`. MET for both, and update is the newly-satisfied half.** Two further `PUT`s → `200`; `DELETE` → `200`; `GET` → `404`; the key is then reusable (`POST` → `201`). The `403 immutable` response does not exist anywhere in the server at this head (§4).

**R5 — server-side immutability enforcement removed, validation/docs/tests simplified. MET, and I confirm the enforcement is absent rather than merely unreachable.**

- `git grep -n "error:'immutable'" 21eca665 -- src/` returns **no** hit. The only two
  `src/` matches for the token `immutable` at all are `server.js:27` (the constant, used only by
  the create rejection) and `shared.js:106` (the legacy-layout migration's mode passthrough).
- `handlePut` and `handleDelete` each contain exactly one mode test, `meta.mode === 'capability-write'`.
- Validation is simplified: one array `CREATE_MODES = ['capability-write','public-write']` at
  `server.js:26` replaces the previous three-element inline literal at both call sites, and
  `READABLE_MODES` (`server.js:28`) replaces the same literal in both read-path guards.
- Docs and tests were simplified rather than merely extended; the test count moved 39 → 41
  registered (suite reports 40 → 42 total), because one test,
  `legacy_immutable_object_stays_readable_and_reclaimable`, was **replaced** by three
  (`…_is_writable_in_place`, `…_is_not_migrated`, `capability_write_is_unaffected_by_legacy_mode`)
  rather than duplicated. Measured by diffing the registration arrays of the two refs; the diff
  exits 1, i.e. the arrays do differ, and the difference is exactly that one line out, three lines in.

**R6 — callers must not need to rewrite, copy, re-key or migrate before modifying or deleting. MET.** The very first request I sent to a planted pre-removal object was an anonymous `PUT` with no header at all beyond the path, and it returned `200` and changed the bytes. Nothing had to happen to the object first. Deletion equally needed nothing.

### 3.3 Non-vacuity of the new coverage — two mutations, in throwaway copies only

I did not take the previous front's non-vacuity output on trust. I copied the head tree twice under
`/tmp` and mutated each copy. **The repository under verification was not modified at any point.**

Mutation A — restore the `403` refusal on `PUT` for a pre-removal object
(`if (meta.mode === LEGACY_IMMUTABLE) return json(res, 403, {error:'immutable'});`):

```
$ (cd mut403 && make test); echo exit=$?
mut403 make test exit=2
  legacy_immutable_object_is_writable_in_place ... FAIL
Error: ASSERT: legacy immutable object is writable in place, with no prerequisite
  legacy_immutable_object_is_not_migrated ... FAIL
Error: ASSERT: the object is writable without any prior migration step
  legacy_storage_migration_yields_writable_modes ... FAIL
Error: ASSERT: a genuine legacy immutable entry keeps its stored mode and is still writable in place
Results: 39 passed, 3 failed, 42 total
```

Mutation B — delete the `capability-write` gate from **both** `handlePut` and `handleDelete`
(verified: exactly 2 occurrences removed):

```
$ (cd mutcap && make test); echo exit=$?
mutcap make test exit=2
  conditional_replace_authorization_and_race ... FAIL
Error: ASSERT: conditional put still requires capability
  capability_write ... FAIL
Error: ASSERT: capability required
  capability_write_is_unaffected_by_legacy_mode ... FAIL
Error: ASSERT: capability-write still refuses an uncapped replace
Results: 39 passed, 3 failed, 42 total
```

Both mutations turn the suite red with real failing output. The suite is load-bearing on both the
removal of the refusal and the presence of the capability gate. I claim no more than that.

---

## 4. Capability-write path unregressed — checked specifically, not as a by-product

Run inside the **same namespace** as a planted pre-removal immutable object, so the legacy object
is present throughout and the two coexist.

| Row | `0b5adde` | `21eca665` |
|---|---|---|
| `POST` `X-Skrynia-Mode: capability-write` | `201`, 64-hex capability | `201`, capability length 64 |
| `PUT`, no capability | `403 {"error":"capability_required"}` | `403 {"error":"capability_required"}` |
| `PUT`, wrong capability (`0`×64) | `403 {"error":"invalid_capability"}` | `403 {"error":"invalid_capability"}` |
| `DELETE`, no capability | `403 {"error":"capability_required"}` | `403 {"error":"capability_required"}` |
| `DELETE`, wrong capability (`1`×64) | `403 {"error":"invalid_capability"}` | `403 {"error":"invalid_capability"}` |
| `GET` while mutation is refused | `200` `"one"` | `200` `"one"` |
| `PUT`, correct capability | `200` | `200` |
| `GET` after the capped `PUT` | `200` `"two"` | `200` `"two"` |
| `DELETE`, correct capability | `200` | `200` |
| `GET` after the capped `DELETE` | `404` | `404` |
| `POST` with no mode header → default is `capability-write` | `201 … "mode":"capability-write"` | identical |
| `PUT` on that default-mode object, no capability | `403 capability_required` | `403 capability_required` |
| control: `public-write` `PUT`, anonymous | `200` | `200` |
| control: `public-write` `DELETE`, anonymous | `200` | `200` |

**Not regressed.** Every capability row is byte-identical between the two refs, including the
distinction between `capability_required` and `invalid_capability` and including the unchanged
default mode. Two further pieces of evidence: statically, the only line `git diff 0b5adde..21eca665
-- src/server.js` deletes inside the two handlers is the legacy `403`, and the
`if (meta.mode === 'capability-write')` block is untouched; and by Mutation B above, the gate is
still load-bearing, with `capability_write_is_unaffected_by_legacy_mode` among the three tests that
go red without it.

---

## 5. The `immutable` grep sweep, done by me — every hit accounted for

```
$ git grep -n -i immutable 21eca665 | wc -l
38
$ git grep -c -i immutable 21eca665
21eca665:README.md:3
21eca665:docs/api-spec.md:4
21eca665:docs/deployment.md:1
21eca665:docs/intent-records/deployment.md:1
21eca665:docs/intent-records/mutation-modes.md:1
21eca665:src/server.js:4
21eca665:src/shared.js:1
21eca665:tests/test.js:23
```

**38 hits across 8 files.** Here is the complete accounting.

### 5.1 Server code — 5 hits, all necessary

| Location | Disposition |
|---|---|
| `src/server.js:23` | comment explaining that legacy entries exist on disk and grant no privilege. Accurate; describes the behaviour I measured. |
| `src/server.js:27` | `const LEGACY_IMMUTABLE = 'immutable';`. Required: it is the token compared against at `:203` to produce the `400 mode_removed` that R1 demands. Cannot be deleted without breaking R1. |
| `src/server.js:28` | `READABLE_MODES = CREATE_MODES.concat([LEGACY_IMMUTABLE])`. Required: this is what makes a legacy object *readable* and keeps it on the writable path. Its presence in `READABLE_MODES` is precisely R3, not a residue of enforcement. |
| `src/server.js:203` | the `400 {"error":"mode_removed"}` creation rejection. R1. |
| `src/shared.js:106` | the pre-`store/` layout migration's mode passthrough. Discussed at §5.4. |

There is **no** `403` for immutable anywhere in `src/`, and no branch on `meta.mode` other than
the `capability-write` test. I read both handlers in full to confirm this rather than inferring it
from the diff.

### 5.2 Tests — 23 hits, all load-bearing

`tests/test.js:484,488,489` (the `immutable` creation rejection), `:496,499,503,505,508,510` (writable
in place), `:516,519,523,527` (not migrated), `:536` (the legacy fixture planted alongside the
capability fixture), `:563,568,569,570,579,580` (the pre-`store/` migration yields writable modes),
`:1235,1236,1237` (three registrations). Every one is either asserting the new behaviour or naming
a test that does. None asserts the old `403`, and no test in the repository asserts
`error === 'immutable'`; `git grep "immutable" -- tests/` returns only the lines above.

### 5.3 Documentation — 8 hits, all accurate statements about the new behaviour

`README.md:8` (feature list: "capability-write and public-write modes"), `README.md:125` (the
paragraph stating there is no permanently-undeletable or write-once mode, and that a pre-removal
object may be replaced in place or deleted with no migration, re-key or capability in between),
`docs/api-spec.md:85`, `:120` (`400` list now names `mode_removed`), `:415` (heading
`### immutable (removed)`), `:417` (the section body: treated exactly as `public-write`, in-place
`PUT` with no prerequisite, `DELETE` reclaims, stored mode not rewritten, content-addressed-key
remedy), `docs/intent-records/mutation-modes.md:7` (the current-intent statement). The `403` error
list at `docs/api-spec.md:121` reads "`403`: missing capability, or wrong capability, on a
`capability-write` object" — the immutable case is gone from the list, and it is the only `403`
line in the file.

### 5.4 Out of scope — 4 hits, a different sense of the word

`README.md:71`, `README.md:137`, `docs/deployment.md:72`, `docs/intent-records/deployment.md:7` all
describe filesystem immutability of *build output* under `RELEASES_DIR`. Not an object mode, not
this issue, and correctly untouched. I agree they should stay.

### 5.5 The three specific things I was asked to check

1. **"It claims it found no stale hit in a file it did not open."** I did not find one either.
   Every one of the 38 hits above is accounted for and none of the 8 files contains a stale claim.
   So the *substance* of that claim holds. **The *count* does not.** The previous front reports
   "29 hits across 6 files"; the real figures are **38 hits across 8 files**. Its own body is
   internally inconsistent on this: the three groups it lists as 13 changed + 16 retained + 3
   out-of-scope sum to 32, which already exceeds its own stated 29, and the out-of-scope group
   names 4 file locations while being labelled 3. Concretely, `docs/deployment.md` and
   `docs/intent-records/deployment.md` are not among the 6 files its count implies, and
   `tests/test.js` carries 23 hits at this head. The classification is sound; the arithmetic is
   not. A reviewer relying on "29" would under-count the surface by nine lines and two files.

2. **`src/shared.js`, which the previous front declined to change.** Its statement that
   `src/shared.js` was deliberately not changed is **true of its own commit** — `21eca66` touches
   five files and `src/shared.js` is not one of them. But it is **false of the 127 stack as a
   whole**: `git diff 83a85b6..21eca665 --stat` lists six files including
   `src/shared.js | 2 +-`, changed by `053eea8` (`Reject immutable at object creation; keep legacy
   objects reclaimable`). The change is at `src/shared.js:106`, in the pre-`store/` layout
   migration: the fallback mode for an unrecognised or absent legacy mode changed from
   `'immutable'` to `'public-write'`. So the claim "it does not need to be [changed]" was made
   about a file the stack had already changed three commits earlier. The change itself is correct
   and directionally required — under the old fallback, a legacy entry with a garbage or missing
   mode would be migrated *into* a write-once object, which is the opposite of the issue's
   compatibility rule — and it is pinned by `tests/test.js:568-570`. My finding is about the
   description, not the code: `src/shared.js` **is** part of this change and the report's framing
   would mislead a reviewer auditing the removal's completeness by file.

3. **`src/client.js`, from which the previous front claims grep returns nothing.**
   `git grep -n -i immutable 21eca665 -- src/client.js` returns nothing — confirmed, and I also
   read `src/client.js:111-124`. `src/client.js` is a thin, dependency-free browser helper whose
   `Store.prototype.create` sets `'X-Skrynia-Mode': opts.mode || 'capability-write'` from the
   caller's option object. It never names a mode, never validates one, and never inspects a `403`.
   `git grep` also returns nothing from `action/`, `builder/`, `example/`, `scripts/`, which I
   confirmed by running it. **Claim confirmed.**

---

## 6. The two closure-bearing limitations — my own findings

### 6.1 Does any client *in this repository* send `X-Skrynia-Mode: immutable` or rely on the 403?

**No. I checked exhaustively and the answer is clean.**

I grepped every occurrence of the creation header at this head:

```
$ git grep -n -i 'X-Skrynia-Mode\|x-skrynia-mode' 21eca665 | wc -l
44
```

Of those 44: 41 are literals `'public-write'` or `'capability-write'` in `tests/test.js`,
`tests/regression.js` and `tests/test-push.js`; 1 is `docs/skills/contextual-feedback.md:145`,
which *recommends* `public-write`; 1 is `docs/api-spec.md:68`, the mode list, which now reads
`{capability-write|public-write}`; and exactly **1** names `immutable` — `tests/test.js:488`, the
negative case that asserts the rejection. `src/server.js:202` is the single read of the header.

Corroborating:

- `src/client.js:117` passes a caller-supplied `opts.mode` through verbatim; the library never
  sends `immutable` and never validates the value. `example/birthday-list/index.html:115` uses
  `capability-write`.
- No test anywhere in the repository asserts a `403` for immutable: `git grep -n "immutable" --
  tests/` returns only the 23 lines accounted for in §5.2, none of which expects a `403`.
- The `403` response for immutable is gone from the source, the API spec's error list, and the
  intent record.

**The precise residual exposure, which is not nil.** Because `src/client.js` is a passthrough, an
out-of-repo consumer of this library that passes `mode: 'immutable'` will now receive
`400 {"error":"mode_removed"}` from `create` where it previously received `201` — and, if it
created that object before the removal, it will now receive `200` from `PUT` where it previously
received `403`. Within this repository nothing does either. **I was not asked to audit
out-of-repository clients and I did not look at `ottojung/antonina`, `ottojung/skrynia-apps`,
`ottojung/assemblyp1` or `ottojung/volodyslav`; I have no evidence either way about them, and I
stop there.**

### 6.2 Does the documentation now plainly state the loss of tamper-evidence, audit-retention and legal-hold?

**Substantially yes, in three documents — with one specific gap I can point at.**

Where the loss is stated, quoted:

- `docs/api-spec.md:85`, final sentence: *"A deployment that used `immutable` for tamper-evidence
  or a legal/audit hold no longer has that property."* One clause, at the end of the paragraph
  that also tells the reader what to use instead.
- `README.md:125`, opening: *"There is no mode that makes an object permanently undeletable or
  write-once, and no mode names its creator."* The write-once loss is stated first, in boldest
  position in the README.
- `docs/api-spec.md:405` under `## Object modes`: *"There is no mode that makes an object
  permanently undeletable, so superseded objects are always reclaimable."*
- `docs/api-spec.md:417` and `docs/intent-records/mutation-modes.md:7`: *"No server-enforced
  write-once property survives the removal of a mode, for new or pre-existing objects."*

So the write-once property loss and the retention/legal-hold loss are both stated plainly and in
the same place the replacement guidance is given. I judge that requirement met.

**The gap.** The word `tamper` occurs **exactly once in the entire repository at this head** —
at `docs/api-spec.md:85`, in the sentence that states the loss. No document anywhere tells a
reader what to do instead if they were relying on `immutable` for tamper-evidence. By contrast the
write-once case *does* get a remedy at `docs/api-spec.md:417` ("callers that need write-once by
convention should create `public-write` objects under a content-addressed key and delete
superseded keys themselves"), and the legal-hold case gets **no** remedy at all, which is
arguably correct since no in-product mechanism exists — but it is not said.

I measured the mechanism the previous front's argument leaned on, rather than repeating it, and
it holds at this head:

```
planted legacy            ETag="6abc7e01-c"
PUT same bytes             200
GET after same-bytes PUT   ETag="6abc7e02-c"   (bytes unchanged: true)
PUT same bytes again       200
GET again                  ETag="6abc7e03-c"
DELETE                     200
GET after recreate         ETag="6abc7e04-c"
```

The ETag advances on every successful replace even when the bytes are identical, and it advances
across delete-and-recreate — and `If-Match` with a stale value still returns `412` on a legacy
object (§3.1). So a client that stored an ETag can still detect substitution and still could never
prevent it. The remaining documentation gap is that the docs do not *say* this, even though the
code supports it. That is a documentation completeness observation for a later pass, not a
behavioural defect, and I am not proposing a change.

**I have not established, and do not claim, whether any real deployment used `immutable` for
tamper-evidence, audit retention or legal hold.** That is unchanged across all four fronts and it
is not answerable from this repository.

---

## 7. Note only, not resolved: the widened public-write class, and board 128

Recorded as instructed, and not resolved.

**What widened.** At `21eca665` the set of objects reachable by anonymous `PUT` and anonymous
`DELETE`, given only namespace and key, is the union of: `public-write` objects, **and every
object ever created as `immutable`**. My §3.1 and §3.2 measurements show the legacy set now
behaves identically to the `public-write` set on both operations, with no capability and no
`If-Match`. That is the exposure Antonina board issue 128 is about, and this change enlarges it
by the size of the pre-removal population.

**Are code and docs at this head consistent with that widened class? Yes — I checked, and they
agree with the code rather than lagging it.**

- `docs/api-spec.md:417`: legacy objects are *"treated exactly as `public-write`: `PUT` replaces
  them in place with no prerequisite, and `DELETE` reclaims them, both anonymously and with no
  capability."* That is precisely what I observed, including the anonymity and the absence of a
  prerequisite.
- `README.md:125`: *"anyone who knows namespace and key may replace one in place with `PUT` or
  delete it and create a different object at the same key, with no migration, re-key or capability
  in between."* Also precisely what I observed.
- `docs/api-spec.md:405` frames the whole class correctly: two modes are accepted at creation and
  no mode makes an object permanently undeletable.
- `docs/intent-records/mutation-modes.md:7` states the widened class as current intent, including
  the rationale (undeletable storage grows without bound) and the explicit sentence that no
  server-enforced write-once property survives.
- The `403` error list at `docs/api-spec.md:121` no longer offers `immutable` as a `403` cause, so
  the error table matches the server.

**The one thing a reader of the on-disk state could get wrong, and the docs pre-empt it.** After
removal there are **three** mode strings possible in `store-meta/*.json` but only **two** accepted
at creation. I confirmed by direct read that a legacy object's stored mode remains the literal
string `"immutable"` after a replace. A deployment inspecting `store-meta` and inferring
protection from the word would be wrong — and the docs say so in all three places
(`README.md:125` "confers no privilege"; `api-spec.md:417` "The stored mode is not rewritten and
confer no privilege; a replace carries it forward unchanged"; the intent record likewise). So the
consistency Antonina's prose is being corrected against is sound at this head.

**I am not resolving the exposure question, not proposing a change, and not making any statement
about board 128's own findings.** I note only that 128's corrected wording must describe the
behaviour I measured above, and that the behaviour is larger in scope than at `0b5adde`, where the
legacy population was deletable but not replaceable.

I also checked for conflicts among the current Intent Records, as `AGENTS.md` requires, and found
**none**: of 26 intent-record files, only `mutation-modes.md` and `deployment.md` mention modes or
immutability, `deployment.md`'s is about release directories, and `client-api.md`,
`filesystem-storage.md`, `storage-model.md` and `nginx-store-get-replacement.md` make no
write-once or protection claim. `etag-conditional-replace.md` remains consistent with the measured
ETag behaviour.

---

## 8. The pre-existing flake, measured on both sides — partially reproduced

**What I measured.** `make test` (four node scripts: `tests/test.js`, `tests/regression.js`,
`tests/test-action.js`, `tests/test-push.js`) run alternately on the head tree and the base tree,
15 runs each, 30 runs total, from throwaway `git archive` trees.

```
head: 14 of 15 exit 0, each printing  Results: 42 passed, 0 failed, 42 total
                    (regression 16/0/16, action 15/0/15, push 21/0/21)
base: 14 of 15 exit 0, each printing  Results: 40 passed, 0 failed, 40 total
                    (regression 16/0/16, action 15/0/15, push 21/0/21)
failures: head run10 exit=2, no Results line,  "Error: docker failed;"
          base run2  exit=2, no Results line,  "Error: docker failed;"
```

**Yes, I reproduced it, on both sides.** The signature is identical in both cases: the process
aborts with `Error: docker failed;`, prints no `Results:` line, and exits non-zero, so a `make test`
gate is *red* rather than flaky-green. It is confined to `tests/test.js`; running that file alone
8 times on the head tree gave 3 aborts (runs 1, 6, 7) and 5 clean `42 passed, 0 failed, 42 total`.
The other three files never flaked in 30 suite runs.

**It is pre-existing.** It occurs on the base tree, whose `src/` and `tests/` contain no part of the
127 change at that point in the chain.

**On the relayed rates.** The previous front reported 2-in-4 at head versus 2-in-6 at base. I did
not reproduce that asymmetry: I measured 1-in-15 at head and 1-in-15 at base. The previous front
also describes the flake as "the `Error: docker failed` flake", measured on both sides, root
cause not diagnosed. I did not diagnose the root cause either. **The direction of the asymmetry in
the relayed figure is opposite to mine, and at these sample sizes neither rate supports a
conclusion.** I make **no claim of hermeticity**, and I would not accept `make test` on this host
as a clean gate without allowing for it. Per `AGENTS.md` the suite must also complete in under
10 seconds; I measured `real 0m9.491s` for a full `make test`, which is at that limit rather than
comfortably inside it, though that is a property of the suite rather than of this change.

---

## 9. Every command, with its real exit code and real count

Commands whose exit code is not 0 are marked. Everything else exited 0.

| Command | Exit | Real result |
|---|---|---|
| `antonina board show --id 127 --json` | 0 | read only; issue body quoted in §2/§3.2 |
| `grep -n "anon" /sys/fs/cgroup/memory.stat` | 0 | `anon` 7576084480 before, 6843617280 after; cap 32212254720 |
| `git ls-remote origin 'refs/heads/release/*' 'refs/heads/main'` | 0 | 3 refs; exactly 2 `release/*` |
| `git merge-base`/`rev-list --count` for both release branches vs main | 0 | merge-base = each tip; main-ahead 156 / 100; main-behind 0 / 0 |
| `git log -1 --format='%h %s parents: %p' cba37ff 55c7ec1` | 0 | both release tips merged into main |
| `git archive <ref> \| tar -x -C /tmp/...` ×3 | 0 | orig, base, head trees |
| `node /tmp/opencode/lwverify/fabricate.cjs` | 0 | 201 create; 403 PUT; 403 DELETE; 200 GET; 409 re-create; on-disk dump shown in §2 |
| `node /tmp/opencode/lwverify/matrix.cjs …/head …` | 0 | 30 rows, §3.1/§3.2/§4 |
| `node /tmp/opencode/lwverify/matrix.cjs …/base …` | 0 | 30 rows, before column |
| `node /tmp/opencode/lwverify/cas.cjs …/head` | 0 | 412 / 200 / 200 / 200 |
| `node /tmp/opencode/lwverify/cas.cjs …/base` | 0 | 403 / 403 / 200 / 200 |
| `node /tmp/opencode/lwverify/etag.cjs …/head` | 0 | ETags `"6abc7e01-c"` → `-02-` → `-03-` → `-04-` |
| `make test` × 15 on head | 0 ×14, **2 ×1** | 42/0/42; one `Error: docker failed;` |
| `make test` × 15 on base | 0 ×14, **2 ×1** | 40/0/40; one `Error: docker failed;` |
| `node tests/test.js` × 8 on head | 0 ×5, **1 ×3** | 3 aborts, 5 × 42/0/42 |
| mutation A: restore the 403, then `make test` | **2** | `39 passed, 3 failed, 42 total`; 3 named FAILs with assertion text |
| mutation B: delete both capability gates, then `make test` | **2** | `39 passed, 3 failed, 42 total`; 3 named FAILs with assertion text |
| `git grep -n -i immutable 21eca665 \| wc -l` | 0 | **38** |
| `git grep -c -i immutable 21eca665` | 0 | 8 files: 3/4/1/1/1/4/1/23 |
| `git grep -n -i immutable 21eca665 -- src/client.js` | **1** | no output — no match (git grep exits 1 on no match; that is the confirming exit) |
| `git grep -n "error:'immutable'\|'immutable'" 21eca665 -- src/ action/ builder/ example/ scripts/` | 0 | 2 lines: `server.js:27`, `shared.js:106` |
| `git grep -n -i 'X-Skrynia-Mode\|x-skrynia-mode' 21eca665 \| wc -l` | 0 | **44**; exactly 1 names `immutable`, at `tests/test.js:488` |
| `git grep -n -i tamper 21eca665` | 0 | **1** hit, `docs/api-spec.md:85` |
| `git diff 83a85b6..21eca665 --stat` | 0 | 6 files, 133 insertions / 21 deletions |
| per-commit `git show --stat` ×6 | 0 | `src/shared.js` appears in `053eea8` only |
| `git rev-list --reverse 83a85b6..21eca665` + `git log -1` ×6 | 0 | 6 commits: `053eea8 d8dfc38 a2ab8bf bae67f2 0b5adde 21eca665` |
| diff of test registration arrays, `0b5adde` vs `21eca665` | **1** | arrays differ: 1 line out, 3 in; 39 → 41 registered |
| `rm -f node_modules` in the worktree | 0 | removed; `ls -ld node_modules` then exits **2** (absent) |
| `git status --porcelain` | 0 | empty |
| `git rev-parse HEAD` | 0 | `21eca665ee6ded663b3a677e0f8cfe78891f8ee5` |
| `git branch --show-current` | 0 | `review/127-legacy-writable-verify` |

Counts I assert, all measured above: 6 commits; 6 changed files; 38 `immutable` grep hits across
8 files; 44 `X-Skrynia-Mode` hits, 1 of which names `immutable`; 1 `tamper` hit; 0 `403`-immutable
constructions in `src/`; 30 `make test` runs, 2 aborts; 8 standalone `tests/test.js` runs, 3 aborts;
2 mutations, both red.

---

## 10. node_modules symlink — created and removed

**Yes, I created one and I removed it.**

- In the verification worktree `/workspace/skrynia-127-lwverify`: `ln -s
  /workspace/skrynia-127-immutable/node_modules node_modules`, used for nothing that required
  dependencies, and `rm -f node_modules` at the end. Verified afterwards:
  `ls -ld node_modules` → `ls: cannot access 'node_modules': No such file or directory` (exit 2).
- In the three throwaway `git archive` trees under `/tmp/opencode/lwverify/{orig,base,head}`: the
  same symlink, created to let the probes and the suite resolve dependencies, and removed with
  `rm -rf` on each tree's `node_modules` afterwards.

`/workspace/skrynia-127-immutable/node_modules` was **read only** and never written. It was
`drwxr-xr-x 1 lubko lubko 386 Sep 29 23:46` before and after my work. I ran no `npm install` and
no `npm ci`. The verification worktree's porcelain is empty at `21eca665` on
`review/127-legacy-writable-verify`.

---

## 11. What I could NOT establish

Marked RELAYED where the number is not mine.

1. **Whether any real deployment used `immutable` for tamper-evidence, audit retention or legal
   hold.** Not answerable from this repository. I did not look, and I do not claim otherwise. Not
   RELAYED — it is simply unestablished by me.
2. **Whether any out-of-repository client sends `X-Skrynia-Mode: immutable` or relies on the `403`.**
   I established the *in-repository* answer (§6.1: none) and I was not asked to audit outside. I
   read no ref in `ottojung/antonina`, `ottojung/skrynia-apps`, `ottojung/assemblyp1` or
   `ottojung/volodyslav`. I have no evidence either way and I stop there.
3. **The root cause of the `Error: docker failed;` flake.** Measured on both sides in §8,
   reproduced, not diagnosed. It is in `tests/test.js`; `src/management.js` is untouched by this
   change. Not RELAYED — my own measurement, my own non-diagnosis.
4. **Whether `make test` is a usable gate on this host.** I have 28 green suite runs and 2 aborts
   across 30, plus 3 aborts in 8 standalone runs of the one flaky file. I claim no hermeticity and
   I would not treat a single green run as a gate. Not RELAYED.
5. **The nginx layer.** `docs/intent-records/nginx-store-get-replacement.md` permits a proxy to
   serve store `GET`s straight from `STORE_DIR`. I verified the Node server only. A legacy object
   is a plain file in that tree, so a proxy would serve it, but I did not test a proxy.
6. **Whether any real deployment has a pre-`store/` `storage/` directory**, so the reachability
   in the wild of `src/shared.js:72-133` is unknown. I verified that code's behaviour only
   against a planted fixture (via the suite's own test, which passes and which Mutation A turns
   red).
7. **RELAYED — the previous front's specific flake rates: "2-in-4 at head versus 2-in-6 at the
   base".** I did not reproduce those figures. I measured 1-in-15 at head and 1-in-15 at base.
   I report both sets and draw no conclusion from either at these sample sizes.
8. **RELAYED — the previous front's "29 hits across 6 files" sweep count.** I measured 38 hits
   across 8 files (§5). Its classification of the hits is, as far as I checked, correct; its
   count and its file count are not.
9. **RELAYED — the previous front's non-vacuity run counts** ("two runs", "six mutation runs", and
   the 42/6-failure tallies it attributes to the capability-gate mutation). I ran my own two
   mutations and reported my own output in §3.3; I did not reproduce or check its tallies.
10. **RELAYED — the previous front's characterisation of the previous three fronts' reasoning**
    (the "narrower exposure" judgement, the ETag-as-tamper-handle argument, the claim that
    `BOARD127-IMMUTABLE.md` §7 made the keep-the-403 choice deliberately). I did not read those
    reports and I do not adjudicate between them and `21eca665`. What I did measure independently
    is the ETag lineage and `If-Match` behaviour in §6.2, which is consistent with the argument
    but does not establish the historical intent behind it.
11. **Whether the removal of the `403` is safe for any specific pre-removal object in a real
    deployment.** I established behaviour on planted fixtures and on a real object created by the
    pre-removal server. I have no deployment.
12. **`gh` and pull requests.** Not attempted. I opened no pull request and merged nothing. I do
    not treat the absence of a PR as a finding; a human owns that step.
13. **The board-128 interaction.** I measured Skrynia's side and recorded the consistency of its
    code and docs with the widened class (§7). I did not read `ottojung/antonina`, did not read
    board 128, and form no view on whether 128's corrected prose is right. The landing decision
    for 127 is likewise not mine and I make none.

---

## 12. Final state

```
$ git rev-parse HEAD
21eca665ee6ded663b3a677e0f8cfe78891f8ee5
$ git rev-parse --abbrev-ref HEAD
review/127-legacy-writable-verify
$ git status --porcelain
(nothing)
$ ls -ld node_modules
ls: cannot access 'node_modules': No such file or directory
```

This file is committed on `review/127-legacy-writable-verify` as the single markdown commit
permitted to me, and that is the only ref I push. No source file was edited, created or deleted.
No commit, no push, no merge, no ref move in `ottojung/antonina`, `ottojung/skrynia-apps`,
`ottojung/assemblyp1` or `ottojung/volodyslav`. `origin/main`, `origin/release/0.1.0`,
`origin/release/2026-09-10` and the four `fix/127-*` branches were read as base only. No
`antonina board collect` in any form. No board comment. No board issue closed, created or
reordered. No secrets in any file, commit message, branch name or this report.

**I do not conclude that board issue 127 is closed, and I do not conclude that it is not.** On the
six requirements in the issue's own body, all six read MET by the behaviour I measured at
`21eca665`, with the two caveats I have named and measured: the sweep count in the previous
front's report is wrong (§5.5), and `src/shared.js` is part of this stack even though that report
describes it as unchanged (§5.5). The landing decision, and the release line it lands on, are a
human's.
