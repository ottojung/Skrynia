# Skrynia target qualification plan (K0)

This plan defines what may count as evidence for the owner-approved target at
`docs/intent-records` head `0ccc90f`. It does not assert that the target is
qualified. Old-ABI qualification results, pins and release gates are
historical evidence only; they are not reused as new acceptance.

## Exact-head discipline

Every claim names the repository, branch, commit `HEAD`, tree, parent and the
commands run at that exact checkout. Checks are rerun after a rebase or a new
head. No result is reported from a dirty or unpushed checkout.

## Gates before target release candidate

1. **Static boundary:** `make source-check`, `cargo fmt --all -- --check`,
   source linter (50 code lines/file, 10 entries/dir, sibling-only imports),
   strict Clippy (`correctness`, `suspicious`, `perf`, `complexity`).
   No obsolete imports, no native fuel/epoch/resource-limit machinery, no
   superseded API claims presented as current.
2. **ABI conformance:** fixed 19-function inventory from `target-abi.md`;
   byte-only `read1`/`write1`; signed relative seek and forward input skips;
   shared-RAM `u64` addresses/lengths and dynamic resize; byte atomicity and
   disjoint writes under concurrency; sequentially consistent single-byte CAS
   with release/acquire publication; root `open_directory(".")` plus `sync`;
   generic zero-or-one function `eval` with exact type validation, originating
   context execution, fresh child stores, no implicit imports or mandatory
   memory; arbitrary typed named FFI with discovery and no replay.
3. **Failure and durability:** bounds, EOF, invalid operands, descriptor and
   path errors, trap recovery, crash/restart durability of directory
   mutations, host-safe out-of-resource failures.
4. **Integration:** exact kernel/tools heads, typed discovery consumed by T1
   deployment selection, A1 app compatibility tests on a disposable instance,
   independent review, and the umbrella release gate in issue `ZZYan9Ny`.
   The copy signature remains owner-unresolved and is not a gate artifact.

## Explicit non-goals

No production deployment, DNS/TLS/secret mutation, data migration on real
data, or reuse of old fuel/compile-time quota gates as target acceptance.
No subsystem reimplementation by K0; lanes own their code.
