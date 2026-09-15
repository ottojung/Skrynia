$id-3827109451238841
title: GitHub deploy action uses Skrynia management URL
date: 2026/09/15
source: @ottojung
kind: requirement

The reusable GitHub deployment action treats `skrynia-url` as the canonical Skrynia HTTP root itself and resolves the deployment endpoint relative to that URL as `deploy`. Callers must pass the complete externally visible Skrynia URL, including any configured path component. The action must not add, remove, or recognize any predefined path prefix.
