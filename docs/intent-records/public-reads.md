$id-3575263406896832
title: Public reads
date: 2026/11/10
source: @ottojung
kind: requirement

GET and HEAD require the object's name but no separate read credential;
scoped listing still requires list authority. Cryptographically random,
undisclosed names can act as bearer read capabilities, providing
computational confidentiality when names remain secret and cannot be
enumerated elsewhere. This does not provide per-reader revocation or
protect against name disclosure or inspection by the storage operator.
Applications requiring stronger confidentiality or authenticated contents
implement them in client protocols.
