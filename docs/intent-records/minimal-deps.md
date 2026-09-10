$id-6214634120670208
title: Minimal dependencies
date: 2026/09/10
source: @ottojung
kind: constraint

Keep dependencies extremely small. Prefer standard-library facilities and simple filesystem/systemd/nginx integration if that matches existing submodules. Avoid introducing a general web framework/database/container orchestrator unless existing conventions make it clearly simpler.
