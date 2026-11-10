$id-0950561916992512
title: Grant budgets
date: 2026/11/10
source: @ottojung
kind: requirement

The root has unlimited byte/object quotas. Children have positive finite budgets. Resulting location-scoped usage is checked against the acting grant and ancestors, not unrelated overlapping grants. Object size and count, grant metadata, extra user credentials and sponsored push subscriptions consume budget.
