$id-3609216151240704
title: Client HTTP API and JS helper library
date: 2026/09/10
source: @ottojung
kind: requirement

Client-facing HTTP API and a tiny dependency-free JS helper library should be specified and, to the extent reasonable for the first implementation, implemented. Use a reserved platform URL outside the app base path static trees, e.g. /_skrynia/...; choose a small coherent v1 path scheme. Apps may share an origin; the spec must acknowledge this is not an isolation/security boundary.
