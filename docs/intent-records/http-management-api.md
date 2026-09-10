$id-6903637887272960
title: First-class HTTP management API
date: 2026/09/10
source: @ottojung
kind: requirement

Skrynia lifecycle and namespace administration are HTTP-only. There is no administrative CLI and no admin.js entrypoint. Management endpoints live under /_skrynia/ and are handled by the same server process as storage and health. Every management request supplies token as a query parameter; the configured SKRYNIA_TOKEN must match exactly, and management is disabled when no token is configured. Deploy requires repo, commit, subdir, and namespace query arguments; commit must be a full 40 or 64 hex git object id. Deploy auto-creates the namespace only after successful build validation, preserves existing namespace quotas on redeploy, and returns JSON after the synchronous deployment completes. Rollback, undeploy, releases, inspection, and namespace create/remove/inspect/list are likewise HTTP endpoints returning JSON.
