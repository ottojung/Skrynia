# Verification

The Rust test suite checks the actual server and persistent state through
isolated temporary directories. The release gate is `make test`, which
includes a source-language gate, unit tests, HTTP integration tests, default rustfmt, strict Clippy,
and the structural linter (50 code lines per Rust file, 10 direct entries
per source directory, sibling-only local imports).

Integration tests exercise root credential retrieval, public reads,
conditional object mutations, grant attenuation, missing/invalid keys,
private state isolation, and rejection of unsafe notification endpoints.
The crate's unit tests cover journal replay, one canonical durable outbox
write for one or 50 subscribers, one write for 50 acknowledgments,
periodic acknowledgement flushing despite a slow delivery, generation checks,
retry behavior, and fault recovery.
Integration tests verify fan-out generations, attempts, and due times
across a process kill and restart, reject orphaned or corrupted records,
recover interrupted grant revocations without orphaned notifications,
and verify that deleting an idle subscription does not rewrite the outbox.
Additional tests cover Web Push encryption, TLS, DNS filtering, quotas,
and stage-file cleanup.

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
sequential 4 KiB GET, SHA-256-conditioned PUT, and unconditional PUT
operations, in that order. Each PUT replaces the same key with a new
4 KiB payload. The conditional PUT computes the prior-body digest outside
the timed interval. Requests verify successful HTTP responses; the
unconditional PUT workload also reads back the final bytes. The benchmark
server and its files are cleaned up afterward.

Three successive runs on shared `marceline-dev` (2026-11-13, release
binary, 100 operations per workload, local filesystem):

| Run | GET (ops/s) | CAS PUT (ops/s) | Plain PUT (ops/s) |
| --- | ---: | ---: | ---: |
| 1 | 6751.5 | 66.4 | 75.3 |
| 2 | 8161.3 | 20.8 | 70.7 |
| 3 | 6039.7 | 22.2 | 76.2 |

Plain PUT p50 was 12.22–13.13 ms and p99 was 20.55–23.92 ms.
CAS PUT p50 was 13.30–16.44 ms, with unusually large p95 values
(329–358 ms) in two runs. This is **not** a controlled comparison:
the operations execute in a fixed order on a shared host, and resource
contention may explain the tail-latency outliers. Re-run with interleaved
orders, intended deployment storage, sustained concurrency, and notification
load before claiming a CAS overhead or production capacity.
