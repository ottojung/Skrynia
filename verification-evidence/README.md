# Independent verification log — issue 8TasxocIctWh2VHtJ7e4EX-temperate

Owner: agent-zMczRnB18uNgQ5FOxRGg89-recruit
Host: marceline-dev (confirmed via `hostname`)
Date: 2026-11-22
Scope: independent reproduction of agent-znh5RvkLAxPWER7dQJAcsq-stay's
2026-11-21T22:06Z terminal-handoff claim (evidence commit 508d180, 31/31 real
HTTP P0->P1->P2 mediated-callback at exact rebuilt kernel 91cd9b8a with
exported-table P1 6b5f793b), with explicit checks of parent ProviderCtx
binding during nested P1->P0 reentry and trap/restore semantics.

## VERDICT
PASS (bounded). The 31/31 claim is reproduced independently: 5 real-HTTP harness
runs at the pinned heads (2 with the exact claimed kernel binary via the
unmodified harness, 3 with my own from-source rebuild via a hash-adjusted copy
that differs by exactly one line) all returned TOTAL=31 PASS=31 FAIL=0,
ALL_OK=True. VM reentrancy tests 2/2 and the trap/restore test 1/1 pass at the
exact head. NOT release-ready — residual blockers listed in section 8.

## Boundaries observed
- Generic parent-mediated callbacks only; no special Program1 and no P0 business logic.
- No merge, no push to main, no prod/DNS/TLS/credentials, no Ivanka/Viche.
- Isolated own checkout + dedicated CARGO_TARGET_DIR (no other agent's target clobbered).
- DRAFT/evidence branch only; no release-ready claim.

## 1. Remote / exact-ref verification
`git ls-remote https://github.com/ottojung/Skrynia.git`:
- 7695fd33856a88ed79f0ffd2b2a5f501d81850c9 -> refs/pull/117/head and
  refs/heads/agent/ChRkty7IwFb2jWFEZmTLjq-pr116-reentrant-fix
- 249ba3421dfe930e4e59445c803ab38808f69d71 -> refs/pull/118/head and
  refs/heads/agent/hQGjVJwW8gkAfcPE3WnB1s/8dWY-buffered-eval
- 508d180fdc719fc903e5185c899b97024d411dfb -> refs/heads/test/p1p2-mediated-http-znh5RvkLAxPWER7dQJAcsq
- main HEAD d9613ba0a14a28b8bec1e1cc87cc05907c8c3668
`git merge-base --is-ancestor 7695fd3 249ba342` => YES.
Evidence files extracted from 508d180 (integration-evidence/):
- run_mediated_http.py sha256 bd96cbb83a81fb844783969022ee6a66d65ab04eeb6a356095c3c8de0eeedea1 (matches claim)
- results.log sha256 0fbf17d3d37cc1f3d646d169b7e7ebae1ed1f0013b1aa7ab1f0639c72214570b (matches claim)
- README.md sha256 c580f16f457579c0cc92031bb0159259eb044fa144c60fe3813613d309373409
The znh5 local harness copy hashes identically to the committed one.

## 2. Artifact SHA256 pins (independently re-hashed on this host, all OK)
- kernel claimed/znh5 rebuild: 91cd9b8a179a1db5dfcc2c2630746a3744cfc8e785d2c476542d0c1703e1e3e4
- kernel kit pinned PR118 (skrynia-server_pr118_249ba342.wasm.bin): 1a2ad3e1528735301b8aa115c7988028dfa90a77751ec219a23c8e80a0cfb552
- kernel MY from-source rebuild: 202c97fe827efa03c096d86198e42cfd4ac49c8663db0d4d6680369e2b0004b2
- P0 (dumb token-gated relay): 6e4f41e280df56ec7feb5a91e618add2b1d24c934561b4c56767de5c40a936f7
- P1 (ordinary cb-guest, exported table): 6b5f793b64a044fb0ca4cb483c0af0cedfa4c3a846ac102a16eb74884a67c9aa
- P2 echo: dc28e490b107084224562cc02a185e99c64727d6dd185eab5645989aa9c17ec5
- malicious P2 (callback_abuse): f9eaa6438c96ef000085e30b61ca49a43a174786a10ac9a19dae128f92a3d52b

Kernel rebuild nondeterminism is expected and pre-existing: the README itself
records kit 1a2ad3e vs znh5 rebuild 91cd9b8a from the SAME commit. My rebuild
202c97fe is a third distinct hash from a different target path on the same
host/toolchain (rustc/cargo 1.93.0); no embedded build paths were found in any
of the binaries (`strings`). Functional equivalence is established below by
running the identical harness against my rebuild (31/31).

Independent raw WASM parse of P1 (dependency-free LEB128 section decoder) confirms:
IMPORTS: skrynia.read, skrynia.write, skrynia.eval_buffered, skrynia.invoke
EXPORTS: memory(kind 2,idx 0), __indirect_function_table(kind 1,idx 0),
skrynia_mediator_cb(func 105), skrynia_main(func 96), __data_end, __heap_base.
So real P1->P2 callback resolution is possible (the 882fe384 no-table blocker is closed).

## 3. Source-level reentrancy fix and trap/restore (read at exact head 249ba342)
- ccd2840 "fix(vm): bind nested invoke closure to immediate provider context":
  exec.rs captures `providers.last()` at child-link time into ProviderCtx;
  link.rs closes over that exact prov in each child skrynia.invoke import;
  inv::call -> call::invoke uses the bound prov (not dynamic stack top).
  Pre-fix call.rs used `c.data().providers.last()` — the original bug.
- call::invoke staging/guard/cur uses prov.inst / prov.guard; saves guest via
  guest.take() and restores cur=prev and guest=saved unconditionally after
  cb_t.call, including the Err/trap path.
- `guest = None` appears only at state/vm.rs:18 (Vm::new init); no runtime
  assignment sets it. No privileged/named Program1 in source.
- buffered/go.rs (004c01b/249ba34) also captures prov at link time and passes it
  to link::add; the buffered child gets eval=false but the bound invoke closure.

## 4. Kernel rebuild from exact source (dedicated target dir)
Commands (own checkout / dedicated target; nothing shared clobbered):
  git clone https://github.com/ottojung/Skrynia.git skrynia
  cd skrynia && git checkout 249ba3421dfe930e4e59445c803ab38808f69d71
  CARGO_TARGET_DIR=../target-kernel CARGO_BUILD_JOBS=2 \
    cargo build --locked --release --package skrynia-server
Result: BUILD_RC=0, "Finished `release` profile [optimized] target(s) in 2m 48s",
one pre-existing `unused_mut` warning in src/boot/root.rs (not denied by clippy).
Rebuilt binary sha256 202c97fe... (see section 2). HEAD confirmed 249ba342.

## 5. VM reentrancy / trap-restore unit tests at exact head
  CARGO_TARGET_DIR=<own>/target-kernel CARGO_BUILD_JOBS=2 \
    cargo test --locked --test vm reentrant::
    => test result: ok. 2 passed; 0 failed; 37 filtered out. RC=0
       - eval::buffered::reentrant::buffered_eval_reentrant_uses_immediate_parent_provider ... ok
       - eval::nested::reentrant::case::reentrant_p1_to_p0_callback_uses_p0_provider_context ... ok
  CARGO_TARGET_DIR=<own>/target-kernel CARGO_BUILD_JOBS=2 \
    cargo test --locked --test vm buffered_eval_reentrant_callback_trap_revive
    => test result: ok. 1 passed; 0 failed; 38 filtered out. RC=0
       - eval::buffered::fault::buffered_eval_reentrant_callback_trap_revive ... ok
The reentrant P1->P0 test uses distinct P0/P1 memories: root P0 cb writes 160;
P1 cb re-invokes its own invoke import to reach P0 and forwards the response;
final output [160] proves the immediate provider (P0) context was bound, not the
dynamic stack top. The trap-revive test has the P0 callback `unreachable` on its
first call then succeed with 77 on the second, proving guest/guard/cur are
restored after a reentrant trap.
(Prior independent verifier agent-XdN2on4 already established these WAT fixtures
are RED at pre-fix ccbbbdf with ERR_FAULT -9 and GREEN at 7695fd3; not re-run here
to keep jobs small.)

## 6. Real HTTP harness runs (fresh disposable dirs, fake 32-byte bearers)
Harness: unmodified `integration-evidence/run_mediated_http.py` (sha256 bd96cbb8)
from commit 508d180; all 5 artifact SHA256 pins verified in-process on every run.
A) Exact claimed kernel 91cd9b8a, UNMODIFIED harness, own fresh MEDIATED_WD:
   - control run 1: TOTAL=31 PASS=31 FAIL=0, ALL_OK=True, RC=0
   - control run 2: TOTAL=31 PASS=31 FAIL=0, ALL_OK=True, RC=0
   - both logs byte-identical to each other AND to the claim's results.log
     (sha256 0fbf17d3d37cc1f3d646d169b7e7ebae1ed1f0013b1aa7ab1f0639c72214570b).
B) My from-source rebuild 202c97fe, hash-adjusted harness copy:
   - `diff -u` vs unmodified shows EXACTLY ONE changed line: the expected kernel
     hash string in the SHA dict (91cd9b8a... -> 202c97fe...). No logic changed.
   - copy sha256 0ccae39e473f20702a63264595c721df166026ad751f0aefa6d7ec3269a8eb7a
   - runs 1/2/3: each TOTAL=31 PASS=31 FAIL=0, ALL_OK=True, RC=0
Total: 5/5 real-HTTP runs pass.

Discriminating reentrant evidence (from attacker report, every run):
  get_own_iid_as_readcap=8  (reentrant P1->P0 storage callback executes; a wrong
                             provider binding would yield ERR_FAULT -9)
  put_own_iid_as_writecap=0 (self write in own namespace succeeds)
  get_sib_read_cap=13, get_sib_iid_as_readcap=13, put_sib_write_cap=13,
  get_zero_cap=13           (cross-app/sibling denial)
  lookup_via_invoke=12, register_via_invoke=12 (non-appdata ops rejected)
  raw_opendisk_via_invoke=-1, garbage_invoke=-1 (private path/garbage rejected)
  read_fd3/4/99=-7, write_fd3/99=-7 (raw FD guesses rejected)
  victim secret TOPSECRET-A unchanged; no `evil` key contamination; victim A still invokable.
The malicious P2 source (kit continuation10 callback_abuse.rs) confirms it calls
skrynia.invoke -> P1 skrynia_mediator_cb -> reentrant P1->P0 storage, so the
negative flow genuinely exercises the path fixed by 7695fd3.

## 7. Artifact / log hashes for this verification
- runner.out / verification-runs.log sha256 fb1ecc987c016f0dbeba0dc1d8b3c74e1ad2b63dc8a8e3f7afaebe1af8505580
- control-run1.log / control-run2.log sha256 0fbf17d3d37cc1f3d646d169b7e7ebae1ed1f0013b1aa7ab1f0639c72214570b
- hash-adjusted harness sha256 0ccae39e473f20702a63264595c721df166026ad751f0aefa6d7ec3269a8eb7a
- own rebuilt kernel sha256 202c97fe827efa03c096d86198e42cfd4ac49c8663db0d4d6680369e2b0004b2

## 8. Residual blockers / not release-ready (owned elsewhere or need human sign-off)
1. Root provider still has guard=None: a dumb P0 that forwards child syscalls
   can reach root paths. Needs kernel intersection-cap fix + security review
   (hAx / kernel #117). Not touched here.
2. Clean fail-closed deny semantics for missing operator files.
3. Full CI qualification (fmt / 7 source lints / strict clippy / make test) on
   every exact head — not re-run here; prior agents did it at 7695fd3.
4. Exhaustive malicious-isolation fuzzing / cancel / timeout / depth / backpressure
   matrix beyond this bounded harness.
5. PR118/PR119/PR122 P0 compatibility split.
No release-ready claim is made. Issue stays open. No merge/deploy/production.
