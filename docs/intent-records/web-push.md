$id-6065450554435260
title: Scoped Web Push
date: 2026/11/10
source: @ottojung
kind: requirement

A user token with subscribe authority creates subscriptions with their own location matcher (a subset of its grant) and mutation event kinds. Changes enqueue retryable wake-up notifications to matching subscriptions. Messages are generic wake-ups. Pending changes coalesce per subscriber, with durable generations preventing concurrent writes from being acknowledged by earlier deliveries. Per-subscription credentials manage updates and deletion.

Push requires HTTPS URLs without embedded credentials. Actual connections pin DNS resolution to public IP destinations. Delivery is timeout-bounded, concurrent within a configured bound, and retains a new notification generation if a mutation occurs during an earlier delivery.
Failures are retried with exponential backoff up to a bounded attempt count. Exhaustion retires the subscription, requiring re-registration. New object changes do not reset failure backoff. Push destinations are denied unless the operator explicitly configures an allowlist. Entries are exact HTTPS hostnames or wildcard subdomain suffixes (such as `*.push.example.org`); an explicit `*` opts into all otherwise valid public endpoints. Registration rejects malformed P-256 browser public keys and authentication secrets.
