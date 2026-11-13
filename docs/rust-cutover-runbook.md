# Rust migration rehearsal and rollback checklist

**The Rust rewrite is experimental. PR #85 must remain unmerged and the
production Node server stays authoritative until the release gates are signed
off.** None of the commands below deploys Rust or edits a live data directory.

## Prepare an offline snapshot

1. Identify the actual production data volume and its filesystem, mount options,
   reverse proxy, public origins, grant configuration, Push endpoints and
   resource limits. Record the running Node image's full commit digest.
2. Make a **consistent, private offline backup**, by stopping the Node writer
   or taking a storage snapshot with coherent filesystem semantics. Do not copy
   a directory while its writer is mutating it. Preserve ownership, modes,
   symlinks, directory structure, and all of `store/`, `store-tmp/` and
   `state/`. Never delete `state/writer.lock`; its inode is the coordination
   point for exclusive writers. Never put the backup under a publicly served
   directory.
3. Confirm the source contains no pending transaction or grant-revocation
   journals, and no abandoned temporary upload files. If there are pending
   records, do **not** manually delete them: recover with the supported Node
   server first and then take a fresh clean offline snapshot.

For example, after obtaining a *quiescent* snapshot in a private location:

```sh
npm ci
cargo build --locked --package skrynia-server
make rust-snapshot-test
node tests/rust-offline-snapshot.js /path/to/offline/skrynia-snapshot
```

The checker requires readable directories and makes another private temporary
copy using Node's filesystem API. Only the temporary copy is ever opened by
Rust or Node. Both processes bind to local loopback and run with manual Push
delivery; neither attempts external Push requests. The checker hashes every
public object and all private grant, subscription, VAPID and outbox records,
requires clean staging and journal directories, reads every public object
through both HTTP implementations, verifies matching SHA-256 ETags, and
requires identical durable state after each restart. It prints only counts,
never bearer tokens or object contents. The supplied snapshot is read-only
from the checker's perspective; it is not mutated.

This is deliberately a **read-only interoperability test**. Separate existing
synthetic tests exercise actual Rust mutations and Node rollback, including
160 objects, delegated grants, CAS and pending Push notifications. Both types
of evidence are required; neither substitutes for the other.

## Release decision

Do not authorize production cutover until all of the following have documented
results on the intended deployment hardware:

- The offline-snapshot check above passes on a *representative complete copy*
  of actual historical data, including private subscriptions and grants.
- Rust and Node agree on the complete protocol test corpus, all crash/fault
  tests, concurrent CAS and quota rules, and the independent VAPID crypto test.
- A browser receives a real end-to-end Web Push notification from an external
  provider, including retry, expiration and revoked-subscription behavior.
- Sustained mixed workloads meet chosen latency and resource budgets on the
  target disks and cgroup configuration; backup and restore are rehearsed.
- The reverse proxy isolates untrusted publisher origins, prevents static
  symlink traversal, rate-limits anonymous reads, and never exposes `state/`.
- An independent review explicitly approves production migration and rollback.

A production switch, if authorized later, requires a fresh restorable offline
backup, stopping the old Node writer before acquiring the same data directory
with Rust, checking health and private authority without printing secrets,
and retaining the exact old Node image and rollback procedure. A rollback must
first stop Rust, then start the old Node image on a compatible, consistent
state snapshot. Never run both writers concurrently. If a startup or state
comparison fails, do not discard the backup or attempt improvised repairs.

See [qualification ledger](rust-qualification.md) and
[operational rules](operations.md). This document is a review procedure,
**not** authorization to change production.
