$id-9183047265512048
title: Standalone HTTP reads
date: 2026/11/10
source: @ottojung
kind: requirement

The Node.js server serves public object GET requests. nginx or another configured static server may serve public object files independently.

Standalone object reads include quoted SHA-256 strong ETags for
If-None-Match revalidation and If-Range, HEAD, and single-byte-range GET
requests. Hashing public files uses asynchronous reads so an anonymous
large GET does not monopolize the HTTP event loop. Each request pins one
file descriptor so concurrent replacement cannot make its ETag describe
different bytes from the response body.
