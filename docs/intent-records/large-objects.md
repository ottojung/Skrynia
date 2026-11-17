$id-5170459438610461
title: Large streamed objects
date: 2026/11/17
source: @ottojung
kind: requirement

Skrynia supports objects up to 10,000,000,000 bytes by default.
An incoming large HTTP body must stream into a private staging file
without requiring memory proportional to the object size. Retain
single-key atomic commits, content SHA-256 CAS, quota enforcement,
durable recovery, and cleanup of abandoned staging files. Reverse
proxies must permit the same object size and stream request bodies.
