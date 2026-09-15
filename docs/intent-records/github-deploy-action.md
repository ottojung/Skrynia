$id-1104093926425326
title: Reusable GitHub Action for deployment
date: 2026/09/15
source: @ottojung
kind: requirement

Skrynia publishes a reusable GitHub Action under action/deploy/ that app repositories can reference as `uses: ottojung/Skrynia/action/deploy@<ref>`. The action wraps Skrynia's existing authenticated HTTP deploy endpoint (GET /_skrynia/deploy) rather than duplicating server deployment logic. It accepts a Skrynia management token via a secret input, derives sensible defaults from GitHub context (repository, commit SHA), and supports optional subdir, builder, and explicit repo/commit overrides. The action is a dependency-free Node 20 JavaScript action using only Node built-ins (global fetch, URL, fs, process); it requires no npm packages and no shell helpers.
