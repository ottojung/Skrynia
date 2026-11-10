$id-0950561916992512
title: Grant budgets
date: 2026/11/10
source: @ottojung
kind: requirement

The root has unlimited byte/object quotas. Children have positive finite budgets. Resulting location-scoped usage is checked against the acting grant and ancestors, not unrelated overlapping grants. Object size and count, grant metadata, extra user credentials and sponsored push subscriptions consume budget. Clients can guarantee quota headroom for a child without a server-side reservation feature by granting disjoint mutation scopes, bounding each child quota, and maintaining the invariant that the sum of child maxima plus parent-only usage fits each finite ancestor's limits. Unlimited-root direct grants avoid finite ancestor contention. This requires a trustworthy allocator not issuing overlapping or unbudgeted mutation grants; physical disk space is separate.
