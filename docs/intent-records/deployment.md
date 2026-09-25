$id-3567678288414720
title: Versioned atomic deployment
date: 2026/09/10
source: @ottojung
kind: requirement

Deployment is versioned and atomic. Finished build output becomes an immutable release directory; activation is an atomic current-pointer/symlink switch. Failed builds leave the previous release untouched. Rollback activates an older retained release. Keep deployment identity/config inspectable: namespace (which is the app name), repo, exact commit, subdirectory, builder image. Do not tie app data lifecycle to releases.

Before activation, the completed staged tree is recursively canonicalized to directories 0755 and regular files 0644. Symlinks and special files are rejected earlier during build output validation. An independent permission validation pass confirms canonical modes before the atomic rename; failure aborts deployment and leaves the previous active release unchanged.

Every Git operation and the builder have finite server-side deadlines. The builder has a deterministic container name and CID file; timeout force-removes that container before releasing the deployment lock and cleaning the attempt workspace. Timeouts retain bounded, scrubbed subprocess diagnostics, and health and storage traffic remain responsive while cleanup runs.
