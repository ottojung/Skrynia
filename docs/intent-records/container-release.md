$id-6521611260366427
title: Unprivileged container and validated image publication
date: 2026/11/12
source: @ottojung
kind: requirement

The container runs the Skrynia process without root privileges, with a
writable persistent data directory and support for a read-only root
filesystem. The server acquires the required kernel lock directly. Registry publication
runs only after the Rust test, structural lint, security checks and the
built image's container smoke test succeed.
Published images include a source-commit-specific tag.
