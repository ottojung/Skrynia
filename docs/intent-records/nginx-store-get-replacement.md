$id-7318492056174302
title: Nginx store GET replacement
date: 2026/09/28
source: @ottojung
kind: requirement

A deployment must be able to replace all `GET /store/{namespace}/{key}` requests with nginx serving `STORE_DIR/{namespace}/{key}` directly, without proxying those reads through the Skrynia process and without changing the observable object-read semantics. This compatibility requirement is specifically for nginx; Skrynia does not promise that arbitrary static-file servers can replace store GETs.
