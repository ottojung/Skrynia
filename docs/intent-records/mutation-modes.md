$id-0562329623214080
title: Object mutation modes
date: 2026/09/10
source: @ottojung
kind: requirement

Object mutation mode at creation: immutable, capability-write, or public-write. Default capability-write. For capability-write creation, the server returns a fresh high-entropy write capability once; subsequent put/delete require it. The capability is bearer authority, not user identity. It must not be obtainable through get and should be stored server-side only as a safe verifier/hash, not plaintext if simple to do. Immutable cannot be changed/deleted through public API. Public-write may be modified/deleted by anyone who knows namespace/key. Do not build roles/accounts/auth around this. If a simple mode-change API makes implementation significantly larger, omit it from v1 and specify mode as fixed for object lifetime.
