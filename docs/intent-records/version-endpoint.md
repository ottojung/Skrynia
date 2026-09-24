$id-7317456206882143
title: Running Skrynia version endpoint
date: 2026/09/24
source: @ottojung
kind: requirement

Skrynia exposes a public version endpoint relative to SKRYNIA_URL so operators can directly determine the exact running Skrynia build instead of inferring it from deployment configuration. The production response identifies both the release version and exact source commit baked into the running server image.
