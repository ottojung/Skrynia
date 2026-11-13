# Skrynia Project Rules

## Simplicity Rule

The implementation must fit in a maintainer's head. Prefer small explicit modules, few dependencies, and no speculative machinery. If a feature would require a large framework, implement the narrowest direct version instead.

## Development Rules

- Keep implementation, administration, tests and benchmarks in Rust. Documentation describes only the current system.
- Every Rust source (including lint/test code) must have at most 50 nonblank, noncomment code lines. Every directory under Rust source/lint/test trees may have at most 10 direct entries, files and directories combined. Local Rust imports must stay within sibling modules. Apply unmodified default `rustfmt`, and enforce the rules mechanically with Ivanka's source linter, `cargo fmt --all -- --check`, and strict Clippy checks.
- Test changes against isolated temporary data. Never modify deployed data as part of development or CI. Use reversible releases and explicit operational review.
- Keep dependencies minimal. Prefer standard-library facilities.
- Before committing, run the test suite and ensure it passes.

## Testing Requirements

- All tests run with a single command: `make test`
- Tests must complete in under 10 seconds wall-clock time.
- Tests assert stable, general product invariants.
- No situational or issue-memorializing tests.
- Aggressively delete obsolete or redundant tests.

## Git Practice

- Commit frequently with small conceptual changes.
- Write helpful commit messages.
- Never squash conceptually unrelated changes.

## Intent Records

Intent Records under `docs/intent-records/*.md` describe the **current desired properties** of Skrynia. They are not a history of superseded requirements; Git history carries that history. When intent changes, update or remove the live record so it states only current intent.

Every independently referenceable current intent has a stable opaque ID of the form `$id-<16 random decimal digits>`. Generate the digits randomly, give them no mnemonic or sequential meaning, and check existing Intent Records for collisions before use.

Keep the same ID while it denotes the same intent. Group related records into scoped files under `docs/intent-records/`.

If current Intent Records conflict, identify the conflicting IDs and surface the conflict instead of silently choosing one.
