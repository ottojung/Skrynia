$id-3609216151240704
title: Generic browser API
date: 2026/11/10
source: @ottojung
kind: requirement

The minimal browser helper wraps public object reads, bearer-token mutations, scoped key listing, grant delegation and subscriptions. A get computes meta.sha256 from received bytes using crypto.subtle; put and delete support options.sha256 as a content-hash precondition. HTTP failures expose the status and structured server error code and body
to client code. CORS permits requests from external origins without cookie
authentication.
