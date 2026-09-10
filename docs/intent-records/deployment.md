$id-3567678288414720
title: Versioned atomic deployment
date: 2026/09/10
source: @ottojung
kind: requirement

Deployment is versioned and atomic. Finished build output becomes an immutable release directory; activation is an atomic current-pointer/symlink switch. Failed builds leave the previous release untouched. Rollback activates an older retained release. Keep deployment identity/config inspectable: namespace (which is the app name), repo, exact commit, subdirectory, builder image. Do not tie app data lifecycle to releases.
