$id-6521611260366427
title: Unprivileged container and validated image publication
date: 2026/11/12
source: @ottojung
kind: requirement

The container runs the Skrynia process without root privileges, with a
writable persistent data directory and support for a read-only root
filesystem. The image includes the kernel-locking utility required by
the storage service. Registry publication runs only after all supported
Node test jobs and the built image's container smoke test succeed.
Published images include a source-commit-specific tag.
