$id-1313845006123008
title: Grant-scoped key enumeration
date: 2026/11/10
source: @ottojung
kind: requirement

The user-token operation `list` returns all live keys matching a requested exact, prefix, wildcard or empty location matcher. It requires the requested matcher to be a non-strict subset of the grant's locations. Grant users without `list` and anonymous readers cannot enumerate keys. Keys are listed independently of which grant created them. Applications may construct further indexes and atomic logical transactions using objects and CAS.
