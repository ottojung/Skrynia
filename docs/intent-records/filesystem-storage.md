$id-5943420546985984
title: Filesystem backed storage
date: 2026/09/28
source: @ottojung
kind: requirement

Storage is actual inspectable files on disk naturally corresponding to keys. A committed object's public bytes live exactly at `STORE_DIR/{namespace}/{key}`, with no data suffix and no internal sidecars in that public tree. Existence of that file is the sole object visibility/commit marker.

Skrynia itself must not require nginx or any other external static web server. Its own HTTP server may continue to implement `GET /store/{namespace}/{key}`. However, the storage layout and read semantics must be compatible with replacing all such GET requests at the deployment layer with direct static-file serving from the public store tree, without involving the Skrynia process and without changing observable object-read behavior.

For an nginx deployment, this means the direct static response can preserve Skrynia's read semantics: raw object bytes as `application/octet-stream` and the same strong `"hex-mtime-hex-size"` ETag used by conditional replacement. Compression or other transformations that would weaken or change that ETag must be disabled for that static store location.

Mutation metadata and temporary files live in separate private directories that must not be web-served. The key policy must prevent traversal and aliasing.
