$id-5841206379054317
title: Object read framing
date: 2026/10/06
source: @ottojung
kind: requirement

A successful object read states the exact byte length of the body in `Content-Length` and does not use chunked transfer encoding, for objects of every size. The declared length always equals the bytes delivered, and the read keeps its `Content-Type`, strong `ETag`, and status semantics unchanged by this. Intermediate proxies must therefore be able to frame the full object deterministically instead of relaying a self-delimited stream.
