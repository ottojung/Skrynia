$id-2651223578750229
title: Derived log-structured quota index
date: 2026/11/16
source: @ottojung
kind: requirement

Maintain exact location-based grant quota accounting while making previously unseen prefix grants efficient through a derived, log-structured, signed-delta index over object names and byte lengths. Retain direct-file public reads, authorization, and quota behavior. Buffer bounded-memory deltas, compact sorted immutable runs, and answer prefix counts and sums without scanning all public objects during normal operation. All indexes are reconstructible from authoritative public object files after journal recovery; corrupted or incomplete runtime index updates must fail closed, never return incorrect quotas. Account explicitly for compaction, extra disk storage, concurrency, and restart rebuilds.
