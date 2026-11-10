$id-7318492056174302
title: Direct static object serving
date: 2026/11/10
source: @ottojung
kind: requirement

nginx may serve GET /store/{key} directly from STORE_DIR/{key} with matching MIME types, cache policy and nginx-compatible ETags. Private state must never be exposed.
