$id-7318492056174302
title: Direct static object serving
date: 2026/11/10
source: @ottojung
kind: requirement

nginx or any other static file server may serve GET /store/{key} directly from STORE_DIR/{key} with suitable MIME types and HTTP cache policy. Each server may derive its own cache validators. Readers can compute SHA-256 from the identical object bytes for conditional mutation through Skrynia. Private state must never be exposed.
