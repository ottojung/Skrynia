$id-4521391348921397
title: Byte-version ETags and conditional replacement
date: 2026/09/24
source: @ottojung
kind: requirement

A successful object read returns an opaque ETag derived deterministically from the exact stored object bytes; it does not cover metadata or storage identity. An object replacement may include If-Match, which is checked against that exact current byte version as an atomic compare-and-replace after independent mutation authorization. A mismatch fails with 412 and leaves the object unchanged. Omitting If-Match retains unconditional replacement, and competing conditional replacements against one version cannot both succeed.
