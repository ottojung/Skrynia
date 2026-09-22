$id-6065450554435260
title: Durable Web Push on store mutations
date: 2026/09/22
source: @ottojung
kind: requirement

Exact namespace + exact key + selected create/replace/delete kinds map to one
or more named push channels. Rules are managed through the existing management
authority (token-authenticated HTTP endpoints). A store mutation that matches a
rule produces durable at-least-once notification state in the same Skrynia
process and data directory before the mutation is committed/visible, so a
crash after commit never loses a notification; a crash before commit may cause
at most a spurious wake-up. Delivery is asynchronous: provider latency never
holds a mutation request. Push payload is exactly the channel name, never
namespace, key, or object data. One broken subscription never blocks healthy
ones; expired (404/410) subscriptions are removed; transient failures retry
with bounded backoff and bounded storage.

$id-0100292231387731
title: Web Push subscriptions are capability-protected and private
date: 2026/09/22
source: @ottojung
kind: requirement

The installation owns one stable VAPID keypair persisted under private
state; the public key is served publicly and the private key is never exposed
over HTTP or JavaScript. Browsers register a standard PushSubscription for one
namespace/channel and receive an opaque subscription id plus a high-entropy
capability required for update/delete. The same endpoint re-registering for
the same namespace/channel is deduplicated. Private subscription records are
never enumerated or exposed through any public endpoint. Public registration
has abuse limits.

$id-6144677134337762
title: Web Push uses the web-push library
date: 2026/09/22
source: @ottojung
kind: requirement

Web Push encryption and VAPID signing use the established Free Software
`web-push` npm package (pinned in package.json/package-lock.json, installed
as a production dependency in the runtime image), not hand-rolled
cryptography. A fake/injectable push transport exists for deterministic
offline tests.
