$id-0950561916992512
title: Namespace quotas and bounds
date: 2026/09/10
source: @ottojung
kind: requirement

Initial namespace quota is 10 MiB stored object bytes by default. Also impose simple defensive bounds so zero-byte-object abuse cannot bypass it: max object count, max key length, max object/request size. Choose conservative simple defaults and document/configure them. Quota enforcement and object creation/replacement must remain race-safe. A simple per-namespace lock is acceptable/preferred.
