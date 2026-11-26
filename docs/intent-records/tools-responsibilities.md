$id-1891053258346577
title: skrynia-tools supplies userland and optional host extensions
date: 2026/11/26
source: @ottojung
kind: requirement

The Skrynia machine contains no embedded programs, application protocols
or application policy. The example Program 0 belongs to skrynia-tools
and is supplied separately to the kernel by the operator. It is a dumb
proxy checking its private authorization token and providing exactly
one function, invoke, to the evaluated application. The invoke protocol
belongs entirely to userland, not the kernel.

skrynia-tools supplies:
- Program 0 and the userland invoke convention.
- WASM ABI bindings, serialization and convenient wrappers.
- HTTP syscall provider implementations, initially publish and unpublish,
  and optional derived-operation accelerators.
- Corresponding custom Rust implementations for libskrynia where provided.
- Deployment and self-hosted administration UI, including version listing,
  atomic rollback and grant creation.

Optional Program 1 may implement higher-level application services,
including a v2-style API if wanted, outside the kernel. Neither Program 0
nor Program 1 is embedded in or special-cased by the server beyond the
generic machine entrypoint for the externally supplied root.

Publication policy and nginx integration belong to the tools/provider,
not the kernel. Migrate tools and applications to the new ABI after its
remaining protocol details are finalized.
