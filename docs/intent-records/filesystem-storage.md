$id-5943420546985984
title: Filesystem backed storage
date: 2026/09/28
source: @ottojung
kind: requirement

Storage is actual inspectable files on disk naturally corresponding to keys. A committed object's public bytes live exactly at `STORE_DIR/{namespace}/{key}`, with no data suffix and no internal sidecars in that public tree. Existence of that file is the sole object visibility/commit marker.

`GET /store/{namespace}/{key}` must be servable directly by nginx as a static file from that public tree, without proxying the read through the Skrynia process. The nginx response must preserve Skrynia's read semantics: raw object bytes as `application/octet-stream` and the native strong `"hex-mtime-hex-size"` ETag used by conditional replacement. Compression or other response transformations that would weaken or change that ETag must be disabled for the store location.

Mutation metadata and temporary files live in separate private directories that must not be web-served. The key policy must prevent traversal and aliasing.
