$id-5943420546985984
title: Filesystem backed storage
date: 2026/09/28
source: @ottojung
kind: requirement

Storage is actual inspectable files on disk naturally corresponding to keys. A committed object's public bytes live exactly at `STORE_DIR/{namespace}/{key}`, with no data suffix and no internal sidecars in that public tree. Mutation metadata and temporary files live in separate private directories that must not be web-served. The key policy must prevent traversal and aliasing.
