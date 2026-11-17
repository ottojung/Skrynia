$id-6065450554435260
title: Exact-key Web Push
date: 2026/11/17
source: @ottojung
kind: requirement

A user token with subscribe authority creates subscriptions for exactly one concrete object identifier, using normal identifier validation (including slash-containing paths). Prefix, wildcard, and empty location matchers are not accepted, including `*` and names ending in `*`. The token must have subscribe authority over that exact identifier; a grant with prefix scope may authorize subscribing to one key within its scope but does not imply subscribing to the entire prefix. Subscription events select mutation kinds.

Changes to the subscribed exact key enqueue retryable wake-up notifications to matching subscriptions. Changes to unrelated keys never enqueue those notifications. Messages are generic wake-ups. Pending changes coalesce per subscriber, with durable generations preventing concurrent writes from being acknowledged by earlier deliveries. Per-subscription credentials manage updates and deletion.

Mutation matching depends on subscriptions registered for the changed key rather than a scan of prefix matchers or all subscribers. Persistent journal/outbox amplification is assessed separately and is not claimed constant.

Existing prefix, wildcard, or empty registrations are not silently broadened or reinterpreted. Startup validation rejects legacy non-exact subscription locations and requires explicit migration to exact-key registrations.

Push requires HTTPS URLs without embedded credentials. Actual connections pin DNS resolution to public IP destinations. Delivery is timeout-bounded, concurrent within a configured bound, and retains a new notification generation if a mutation occurs during an earlier delivery.
Failures are retried with exponential backoff up to a bounded attempt count.
When persisting a push acknowledgment fails before the atomic rename, its
in-memory backoff prevents repeated sends every second while preserving the
last durable notification; failed deliveries accrue transient retry attempts
without classifying successful deliveries as endpoint failures. The cooldown
is not durable, so a restart may produce duplicate wake-ups. Later mutations
preserve delivery-failure backoff when replacing a pending generation.
Exhaustion retires the subscription, requiring re-registration. New object changes do not reset failure backoff. There is no operator-configured hostname allowlist; any otherwise valid public HTTPS push endpoint is eligible. Registration and key updates require unpadded base64url P-256 browser public keys that lie on the curve and 16-byte authentication secrets. On startup, previously accepted subscription records still pass their historical bounded-length checks even when their keys cannot be used; they remain available for capability-authorized repair or re-registration. Unusable keys cannot trigger outbound delivery and instead follow the bounded retry and retirement policy. A clean pre-rename outbox acknowledgment failure retains the notification and does not poison the server.
