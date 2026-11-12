$id-9183047265512048
title: Standalone HTTP reads
date: 2026/11/10
source: @ottojung
kind: requirement

The Node.js server serves public object GET requests. nginx or another configured static server may serve public object files independently.

Standalone object reads include quoted SHA-256 strong ETags for If-None-Match revalidation and If-Range, HEAD, and single-byte-range GET requests.
