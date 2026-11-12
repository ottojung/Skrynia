$id-7458341334509459
title: SHA-256 byte-content conditional mutation
date: 2026/11/11
source: @ottojung
kind: requirement

PUT and DELETE on /store/{key} accept an optional X-Skrynia-If-SHA256
header containing exactly 64 lowercase hexadecimal characters. They compare
the SHA-256 digest of the current object's bytes synchronously with the
mutation. A mismatch returns HTTP 412 content_mismatch; a malformed digest
returns HTTP 400 invalid_precondition. Mutations reject other HTTP
precondition headers with HTTP 400. POST creates only an absent key.

Skrynia GET and HEAD produce a strong ETag containing the quoted lowercase
SHA-256 of the object bytes, and honor If-None-Match and If-Range as HTTP
cache validators. Objects are directly servable as public files through
arbitrary static servers whose cache validators may differ. A browser client
computes meta.sha256 from the received bytes using WebCrypto and supplies
options.sha256 for conditional PUT and DELETE.

Object files use filesystem-provided timestamps. No persistent object
sequence or per-object digest state exists. Create/replace journal intents
contain the resulting content digest and recovery verifies it against
filesystem bytes.

The predicate compares bytes only: A → B → A is indistinguishable from
unchanged A. Applications needing to detect this must encode a strictly
advancing revision or unique transaction ID within mutable object bytes.
