# Verification

The Rust test suite checks the actual server and persistent state through
isolated temporary directories. The release gate is `make test`, which
includes a source-language gate, unit tests, HTTP integration tests, default rustfmt, strict Clippy,
and the structural linter (50 code lines per Rust file, 10 direct entries
per source directory, sibling-only local imports).

Integration tests exercise root credential retrieval, public reads,
conditional object mutations, grant attenuation, missing/invalid keys,
private state isolation, and rejection of unsafe notification endpoints.
The crate's unit tests cover replay invariants, persisted notifications,
multi-subscriber durable generation batching, acknowledgment and fault replay,
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


## Reproducible HTTP throughput probe

Run `SKRYNIA_BENCH_SAMPLES=100 make benchmark`. The Rust workload driver
builds the release binary, starts it with a fresh private temporary data
directory on a loopback port, issues a root user credential, and measures
sequential 4 KiB GET and SHA-256-conditioned PUT requests. It verifies HTTP
statuses, response bytes and content hashes; every PUT changes the stored
content. All files and the process are cleaned up afterward.

Illustrative single run on shared `marceline-dev` (2026-11-13, release
binary, 100 operations per workload, local filesystem):

| Workload | Operations/s | p50 (ms) | p95 (ms) | p99 (ms) |
| --- | ---: | ---: | ---: | ---: |
| GET 4 KiB | 6146.8 | 0.15 | 0.23 | 0.39 |
| CAS PUT 4 KiB | 67.7 | 12.48 | 21.64 | 47.43 |

This is **not** a controlled measurement or a cross-version comparison.
Synchronous fsync and shared-host contention dominate write latency.
Repeat on the intended deployment filesystem under sustained concurrency
and notification load before asserting performance or resource capacity.
