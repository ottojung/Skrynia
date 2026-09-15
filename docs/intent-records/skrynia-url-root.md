$id-2422528008796077
title: SKRYNIA_URL is the canonical Skrynia HTTP root
date: 2026/09/15
source: @ottojung
kind: requirement

`SKRYNIA_URL` is the canonical externally visible root URL for the complete Skrynia HTTP interface. It may contain any path component or no path component. Skrynia must not assume, append, recognize, or preserve any predefined platform path prefix. Health, management, storage, and the served client library are all addressed relative to `SKRYNIA_URL`. The server derives the accepted public request-path base from the pathname of `SKRYNIA_URL`; its local listen address and port remain independent transport configuration so reverse proxies can map the public URL to the local server.
