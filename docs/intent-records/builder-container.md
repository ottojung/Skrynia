$id-8114124388761600
title: Shared pinned builder container
date: 2026/09/10
source: @ottojung
kind: requirement

Builds use a shared, explicitly configured/pinned builder container image. The default image is built locally from builder/Dockerfile during make install. Each build gets a disposable checkout/workspace of the exact commit mounted read-write into the container; the builder root filesystem is writable for builds. Run make build in the selected subdirectory. The build output directory is build/ under that subdirectory unless configuration very simply specifies otherwise. The container runs with --rm (disposable), --cap-drop ALL, --no-new-privileges, --network none. Custom --builder override is supported. The builder image includes npm for npm-driven builds.
