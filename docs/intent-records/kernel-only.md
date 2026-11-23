$id-1891053258346566
title: Native v3 kernel must contain no application protocols
date: 2026/11/23
source: @ottojung
kind: requirement

Skrynia's native Rust server is only a generic WebAssembly execution
kernel with a single opaque transport entrypoint, POST /. It receives a
separately operator-selected Program 0 WASM and supplies generic, confined
VM, filesystem, RAM, time, randomness and execution primitives. It must not
implement HTTP object CRUD methods, application routes, grants, quotas,
subscriptions, Web Push, storage schemas, registered applications or
application authorization. The example Program 0 belongs to skrynia-tools,
not this repository or the kernel; it may validate its private token and
opaque-forward requests to separately supplied userland programs.
Do not retain obsolete API specifications, benchmarks or deployment
configurations as live v3 sources. Git history preserves old designs.
