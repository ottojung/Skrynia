$id-4070814447118336
title: Atomic CRUD operations
date: 2026/09/28
source: @ottojung
kind: requirement

Public store operations are get, create, put, delete. Each operation on one object is atomic and the public object file itself is the visibility/commit marker. Create prepares complete bytes and private mutation state before atomically publishing a new file without replacing an existing key. Put prepares complete replacement bytes and atomically renames them over the current public file, including atomic compare-and-replace when If-Match is supplied. Delete commits by unlinking the public file. Direct static readers may observe the complete old version or complete new version, never partial contents or an uncommitted create.
