$id-6065450554435260
title: Scoped Web Push
date: 2026/11/10
source: @ottojung
kind: requirement

A user token with subscribe authority creates subscriptions with their own location matcher (a subset of its grant) and mutation event kinds. Changes enqueue retryable wake-up notifications to matching subscriptions. Messages are generic wake-ups. Pending changes coalesce per subscriber, with durable generations preventing concurrent writes from being acknowledged by earlier deliveries. Per-subscription credentials manage updates and deletion.

Push requires HTTPS URLs without embedded credentials. Actual connections pin DNS resolution to public IP destinations. Delivery is timeout-bounded, concurrent within a configured bound, and retains a new notification generation if a mutation occurs during an earlier delivery.
Failures are retried with exponential backoff up to a bounded attempt count.
When persisting a push acknowledgment fails before the atomic rename, its
in-memory backoff prevents repeated sends every second while preserving the
last durable notification; failed deliveries accrue transient retry attempts
without classifying successful deliveries as endpoint failures. The cooldown
is not durable, so a restart may produce duplicate wake-ups. Later mutations
preserve delivery-failure backoff when replacing a pending generation.
Exhaustion retires the subscription, requiring re-registration. New object changes do not reset failure backoff. There is no operator-configured hostname allowlist; any otherwise valid public HTTPS push endpoint is eligible. Registration and key updates require unpadded base64url P-256 browser public keys that lie on the curve and 16-byte authentication secrets. On startup, previously accepted subscription records still pass their historical bounded-length checks even when their keys cannot be used; they remain available for capability-authorized repair or re-registration. Unusable keys cannot trigger outbound delivery and instead follow the bounded retry and retirement policy. A clean pre-rename outbox acknowledgment failure retains the notification and does not poison the server.
