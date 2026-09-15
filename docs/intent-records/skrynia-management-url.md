$id-3827109451238841
title: GitHub deploy action uses Skrynia management URL
date: 2026/09/15
source: @ottojung
kind: requirement

The reusable GitHub deployment action treats `skrynia-url` as the Skrynia management API root itself, for example `https://vau.place/_skrynia`, and resolves the deployment endpoint relative to that URL as `deploy`. Callers must not pass only the public site origin when the management API is mounted under a path.
