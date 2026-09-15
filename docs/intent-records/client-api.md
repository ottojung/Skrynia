$id-3609216151240704
title: Client HTTP API and JS helper library
date: 2026/09/10
source: @ottojung
kind: requirement

Client-facing HTTP API and a tiny dependency-free JS helper library should be specified and, to the extent reasonable for the first implementation, implemented. Use the canonical `SKRYNIA_URL` root outside the app base path static trees; choose a small coherent v1 path scheme relative to that root and do not assume a predefined platform prefix. Apps may share an origin; the spec must acknowledge this is not an isolation/security boundary.
