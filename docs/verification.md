# Verification

The Rust test suite checks the actual server and persistent state through
isolated temporary directories. The release gate is `make test`, which
includes unit tests, HTTP integration tests, default rustfmt, strict Clippy,
and the structural linter (50 code lines per Rust file, 10 direct entries
per source directory, sibling-only local imports).

Integration tests exercise root credential retrieval, public reads,
conditional object mutations, grant attenuation, missing/invalid keys,
private state isolation, and rejection of unsafe notification endpoints.
The crate's unit tests cover replay invariants, persisted notifications,
Web Push signing and independent decryption, TLS verification, DNS filtering,
quota accounting, and stage-file cleanup.

Any deployment must use persistent storage, external HTTPS rate limiting,
a tested consistent backup/restore procedure, and a verified isolated
browser-origin configuration. Passing tests alone is not evidence of
end-to-end delivery to an external push provider or resilience to
physical media faults. These require operational checks on the target
environment.

Do not claim a release passed unless the exact pushed commit's checks
are green. Tests and build procedures must not require other runtime languages.
