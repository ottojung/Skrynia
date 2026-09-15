$id-1104093926425326
title: Reusable GitHub Action for deployment
date: 2026/09/15
source: @ottojung
kind: requirement

Skrynia publishes a reusable GitHub Action under action/deploy/ that app repositories can reference as `uses: ottojung/Skrynia/action/deploy@<ref>`. The action wraps Skrynia's existing authenticated HTTP deploy endpoint (GET /_skrynia/deploy) rather than duplicating server deployment logic. It accepts a Skrynia management token via a secret input, derives sensible defaults from GitHub context (repository, commit SHA), and supports optional subdir, builder, and explicit repo/commit overrides. The action is implemented as a composite action with a POSIX shell script, keeping it dependency-light and compatible with standard GitHub Actions runners.
