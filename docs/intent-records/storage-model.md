$id-8284733156847104
title: Shared durable storage not a database
date: 2026/09/10
source: @ottojung
kind: requirement

The platform provides shared durable storage. It is intentionally NOT a conventional database. Model: namespace + key -> opaque bytes/file. Binary values are first-class; JSON is only a client-library convenience. Namespaces are created by deploy or admin CLI, not by the public API.
