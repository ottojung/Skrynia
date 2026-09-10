$id-8114124388761600
title: Shared pinned builder container
date: 2026/09/10
source: @ottojung
kind: requirement

Builds use a shared, explicitly configured/pinned builder container image. Each build gets a disposable checkout/workspace of the exact commit mounted read-write into the container; the builder may modify that disposable checkout. Run make build in the selected subdirectory. The build output directory is build/ under that subdirectory unless configuration very simply specifies otherwise. The builder/container should not gain host privileges.
