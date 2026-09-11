$id-5714237952735232
title: Namespace is app identity
date: 2026/09/10
source: @ottojung
kind: requirement

The namespace name IS the deployed app name and URL suffix. Canonical app URL is {base_path}/{namespace}/ where base_path is configurable via SKRYNIA_APP_BASE_PATH (no canonical default). Do not maintain a separate app-name vs namespace-name concept in v1 unless absolutely necessary internally; the namespace is the app identity. Namespaces are administrative quota/accounting units, not security principals. Any browser/app/client that knows a namespace and key may read it, subject only to per-object mutation capability semantics. Namespace creation, deletion, and configuration are performed through the token-authenticated HTTP management API.
