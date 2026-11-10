$id-4070814447118336
title: Atomic object CRUD
date: 2026/11/10
source: @ottojung
kind: requirement

GET, create-if-absent, conditional or unconditional replacement, and deletion are atomic per key. Complete bytes become visible at one commit point. Clients can implement multi-object logical atomicity by CAS publishing immutable versions. Append-only grants permit creation and replacement only when new bytes have the old bytes as an exact binary prefix; they never permit deletion.
