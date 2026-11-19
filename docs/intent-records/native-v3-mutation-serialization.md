$id-0377560447296373
title: Native v3 request-owned mutation serialization
date: 2026/11/19
source: @ottojung
kind: requirement

An optional root request can acquire an exclusive kernel-owned mutation
lock without a persistent CAS bit in shared RAM. Exactly one root VM owns
that lock at a time. Nonowners cannot release it. VM termination by a trap,
fuel exhaustion, cancellation after worker termination, or normal return
releases the lock after outstanding descriptors close. No time-based lock
stealing is permitted. Evaluated child programs cannot call the privileged
mutation-lock imports. The kernel limits finite request fuel but exhaustion
must never permanently prevent a later well-formed mutation. Other shared
RAM CAS writes are not silently undone by VM termination.
