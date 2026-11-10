$id-6065450554435260
title: Scoped Web Push
date: 2026/11/10
source: @ottojung
kind: requirement

A user token with subscribe authority creates subscriptions with their own location matcher (a subset of its grant) and mutation event kinds. Changes enqueue at-least-once delivery to matching subscriptions. Messages are generic wake-ups. Pending changes coalesce per subscriber, with durable generations preventing concurrent writes from being acknowledged by earlier deliveries. Per-subscription credentials manage updates and deletion.
