# Real HTTP P0 -> P1 -> P2 mediated-callback acceptance (issue 8TasxocIctWh2VHtJ7e4EX-temperate)

Owner: agent-znh5RvkLAxPWER7dQJAcsq-stay. Host: marceline-dev. Test-only evidence branch; no product source changed, no merge/deploy/production.

## What this proves

A genuine compiled ordinary WASM P1 (native-service cb-guest, `6b5f793b...`, exported
`__indirect_function_table`) executes a registered, stored P2 bytecode over **real HTTP**
through a token-gated, business-logic-free root Program0, with P1's own mediator callback
brokering P2 -> P1 -> P0 storage. A compiled adversarial P2 exercises that reentrant
P1-to-P0 storage callback path and is denied on every cross-app / sibling / private-path /
raw-FD attempt.

## Kernel (reentrancy fix 7695fd3)

- Source remote: https://github.com/ottojung/Skrynia.git
- Built from isolated checkout at `249ba3421dfe930e4e59445c803ab38808f69d71` (PR118 lineage).
  `git merge-base --is-ancestor 7695fd33856a88ed79f0ffd2b2a5f501d81850c9 HEAD` => YES.
  `249ba34` adds the bounded `eval_buffered` bridge P1 needs; the fix itself is commit
  `7695fd3` ("test(vm): cover reentrant nested callback provider identity") atop `ccd2840`
  ("fix(vm): bind nested invoke closure to immediate provider context").
- Rebuilt kernel SHA256: `91cd9b8a179a1db5dfcc2c2630746a3744cfc8e785d2c476542d0c1703e1e3e4`
  (isolated `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS=2`, rustc/cargo 1.93.0).
  The kit's pinned PR118 binary is `1a2ad3e1528735301b8aa115c7988028dfa90a77751ec219a23c8e80a0cfb552`
  (same source commit; rebuild differs by build-environment nondeterminism only).
- VM reentrancy tests at exact head: `cargo test --locked --test vm reentrant::` => 2 passed
  (`reentrant_p1_to_p0_callback_uses_p0_provider_context`,
  `buffered_eval_reentrant_uses_immediate_parent_provider`).

## Pinned artifacts (SHA256 verified by the harness on every run)

| role | SHA256 | source |
|------|--------|--------|
| kernel | 91cd9b8a... (above) | isolated rebuild at 249ba34 |
| P0 (dumb token-gated root relay) | `6e4f41e280df56ec7feb5a91e618add2b1d24c934561b4c56767de5c40a936f7` | kit artifacts |
| P1 (ordinary cb-guest, exported table) | `6b5f793b64a044fb0ca4cb483c0af0cedfa4c3a846ac102a16eb74884a67c9aa` | weighty's pinned rebuild (skrynia-tools aa6333a) |
| P2 echo | `dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5` | kit tests/guests |
| malicious P2 | `f9eaa6438c96ef000085e30b61ca49a43a174786a10ac9a19dae128f92a3d52b` | kit continuation10 callback_abuse guest |

## Commands (from the isolated checkout root)

```
git clone https://github.com/ottojung/Skrynia.git skrynia
cd skrynia && git checkout 249ba3421dfe930e4e59445c803ab38808f69d71
CARGO_TARGET_DIR=../target-kernel CARGO_BUILD_JOBS=2 cargo build --locked --release --package skrynia-server
CARGO_TARGET_DIR=../target-kernel CARGO_BUILD_JOBS=2 cargo test --locked --test vm reentrant::
python3 integration-evidence/run_mediated_http.py     # artifact paths overridable via MEDIATED_* env vars
```

All data dirs are disposable temp dirs with fake 32-byte test bearers; no real credentials,
no production/Ivanka, no DNS/TLS.

## Results (see results.log; 3 consecutive identical runs)

`TOTAL=31 PASS=31 FAIL=0`, `ALL_OK=True`.

Positive real HTTP (token-gated P0 -> P1 -> stored P2): register, lookup, P1-mediated P2
echo invoke (`hello-p1-p2`), appdata put/get, CAS success, wrong-precondition rejection,
and lookup/invoke/appdata durability after server restart — all PASS.

Malicious P2 via P1 mediator callback (the reentrant P1->P0 storage path fixed by 7695fd3):
- reentrant self read returns absent-not-fault (8) and self write in own namespace (0):
  the reentrant storage callback path actually executes and is correctly bound;
- sibling read cap = 13, sibling invoke-id as read cap = 13, sibling write cap = 13,
  zero cap = 13: cross-app/sibling denial;
- non-appdata Lookup/Register via callback = 12; raw private path `security/program0.sha256`
  wire = -1; garbage invoke = -1; raw FD guesses 3/4/99 read and 3/99 write = -7;
- victim A still invokable, victim `secret` unchanged, no `evil` key contamination.

## Release-gate verdict

Bounded end-to-end evidence PASSES with the new exported-table P1 and the 7695fd3 kernel.
NOT release-ready. Out of scope / still open: root-provider `guard=None` intersection-cap
fix and security review (hAx / kernel #117), clean fail-closed deny semantics on missing
operator files, full CI/fmt/clippy/`make test` qualification on every exact head, exhaustive
malicious-isolation/fuzz matrix, and PR118/PR119/PR122 P0 compatibility split. No release
claim is made here.
