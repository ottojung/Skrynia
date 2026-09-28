$id-4521391348921397
title: Version ETags and conditional replacement
date: 2026/09/28
source: @ottojung
kind: requirement

A successful object read returns an opaque strong ETag identifying the current committed object version. Rereading one committed version returns the same ETag, while every successful replacement advances the ETag even if the object bytes are unchanged. Deleting and recreating a key also advances the ETag while the namespace is retained. The storage representation keeps the data-file mtime/version monotonic so the ETag has the same `"hex-mtime-hex-size"` form nginx generates for static files. An object replacement may include If-Match, which is checked against the current committed version as an atomic compare-and-replace after independent mutation authorization. A mismatch fails with 412 and leaves the object unchanged. Omitting If-Match retains unconditional replacement, and competing conditional replacements against one version cannot both succeed.
