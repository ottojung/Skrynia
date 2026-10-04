$id-5798228980615130
title: Secret store paths are not retained or disclosed
date: 2026/09/30
source: @ottojung
kind: requirement

A store object key is a bearer credential: anyone who knows `SKRYNIA_URL/store/{namespace}/{key}` can read the object, because capability-write protects mutation, not reading. The URL carrying the key is therefore itself secret, and every component that sees that URL is a place the secret can be retained or handed to an unintended party.

In Skrynia's own code this means:

- the store request path writes nothing to stdout or stderr on any outcome, including every refusal;
- no store, management, or app-serving response reflects the namespace, the object key, or a capability back to the caller in a body or a header;
- every response, including refusals and plain-text answers, carries `Referrer-Policy: no-referrer`, so no page served by Skrynia can send a store path as a `Referer` to a third party;
- store and management responses carry `Cache-Control: no-store`, so no intermediary retains a response served for a bearer URL — the management API's `SKRYNIA_TOKEN` travels in the query string;
- capabilities travel only in request headers, never in a URL;
- private state — `store-meta/`, `state/{ns}/push-subs/`, `state/{ns}/push-rules.json`, `state/_push/` including the VAPID private key — is `0600` files under `0700` directories, so neither the bytes nor the file and subscription names are readable by another local account;
- Skrynia's only outbound third-party request is Web Push delivery to an endpoint the browser itself registered, with a payload of exactly the channel name.

Object bytes under `store/` stay `0644` and `store/` stays `0755`, because that tree is deliberately static-file compatible; the key names in it are the disclosure surface, not the bytes.

The remaining half of this property lives outside Skrynia and cannot be enforced by Skrynia: reverse-proxy access and error logging, TLS and HSTS, CDN or observability layers, and backup handling of `DATA_DIR`. See `docs/secret-store-path-guard.md`.