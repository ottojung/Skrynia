$id-6065450554435260
title: Scoped Web Push
date: 2026/11/10
source: @ottojung
kind: requirement

A user token with subscribe authority creates subscriptions with their own location matcher (a subset of its grant) and mutation event kinds. Changes enqueue at-least-once delivery to matching subscriptions. Messages are generic wake-ups. Pending changes coalesce per subscriber, with durable generations preventing concurrent writes from being acknowledged by earlier deliveries. Per-subscription credentials manage updates and deletion.\n\nPush requires HTTPS URLs without embedded credentials. Actual connections pin DNS resolution to public IP destinations. Delivery is timeout-bounded, concurrent within a configured bound, and retains a new notification generation if a mutation occurs during an earlier delivery.\n