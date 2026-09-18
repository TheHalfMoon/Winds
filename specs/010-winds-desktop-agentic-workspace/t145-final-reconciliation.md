# T145 Final Reconciliation — Spec 010 Winds Desktop Agentic Workspace

## Status and closure rule

- Canonical implementation and T144-accepted visual candidate: `ff30e62f129525954acf27468d553018adbc05c6`, tree `9e6720933d0e1ce8bc27ecfb93d3774d16ad51d3`.
- T127..T143 are already `CLOSED_CANONICAL` under the merge/evidence ledger below.
- T144 is `CLOSED_CANONICAL` by the independently authored human Founder PASS at PR #222 comment `5727446349`, reconciled without a polish successor and without merging the evidence-carrier branch.
- This T145 candidate is documentation/evidence only: checked-state reconciliation in `tasks.md` plus this artifact. It changes no production source, dependency, lockfile, workflow, migration, schema, runtime/provider authority, security boundary, desktop behavior, or Git authority.
- T145 becomes `CLOSED_CANONICAL` only after this exact documentation candidate passes final gates, lands by guarded normal merge with an expected-head guard, and every actually-triggered post-merge push check succeeds.
- Exact T145 HEAD/TREE, final author/Ponytail/independent reviews, CI, merge identity, and post-merge checks remain in the PR/merge trail to avoid a self-referential candidate hash.

## 1. Canonical governance chain

| Unit | PR | Canonical merge | Post-merge verification |
| --- | ---: | --- | --- |
| Spec 010 Entry | #187 | `b653318236e885fcfd8de684afe5d733862fbc96` | `quality` 34713792141 SUCCESS attempt 1 |
| Spec 010 Specification | #188 | `9190afdc9b1db93d69909c5d3acbce48690522f7` | `quality` 34714992834 SUCCESS attempt 1 |
| Spec 010 Plan | #189 | `facbefc3ce3308b826c6faef4080a6c085ad2041` | `quality` 34716142293 SUCCESS attempt 1 |
| Spec 010 Tasks | #190 | `b9bea493ab0fe7e261f128b3ec4c7e03e6495bc2` | `quality` 34717148653 SUCCESS attempt 1 |

All four governance merges are GitHub-verified with `verified=true`, `reason=valid`. Canonical Tasks acceptance authorized T127 only; each later task remained dependency-blocked until its predecessor closed.

## 2. T127–T143 canonical implementation ledger

Every row below is a canonical normal merge on the first-parent history of `main`. Listed run ids are actually-triggered post-merge push verification on that exact merge. A retained failure is called out explicitly rather than being relabelled.

| Unit | PR | Canonical merge | Post-merge verification |
| --- | ---: | --- | --- |
| T127 | #191 | `09d50fe185c061f23d21d4865f82b07a55736765` | quality 34718924772; windows-terminal 34718924805 — SUCCESS |
| Amendment 001 | #192 | `37129870df6257b0a50fdeac5a578aa1e4ca86cb` | quality 34720010739 — SUCCESS |
| T128 | #193 | `7bef628ee97e7667ee1fdc1663349d28d957ec5c` | quality 34723328354; desktop-quality 34723328435 — SUCCESS |
| T129 | #194 | `2192d93b4d08a3a23920c657db384ed6ca0ed495` | quality 34730712575; desktop-quality 34730712573 — SUCCESS |
| T130 | #195 | `1bd47465eaf1053ef19b921f7eef26a124029211` | quality 34740184552; windows-terminal 34740184548 — SUCCESS |
| T131 | #196 | `83dba640891313e5e36b38fc00d9d7f96129be4a` | quality 34768646015; windows-terminal 34768646020 — SUCCESS |
| T132 | #197 | `4f50f96335881c502006079d8d349752c8711e7f` | quality 34827696346; windows-terminal 34827696336; desktop-quality 34827696363 — SUCCESS |
| T133 | #198 | `574dc135d6c9ccf34a3cc824251fe51083ec18f9` | quality 34881035325; desktop-quality 34881035277 — SUCCESS |
| T134 | #199 | `1ed0fdb6a9c7a8c80a8c633ac59f219e92de2707` | quality 34890375364; desktop-quality 34890375378; windows-terminal 34890375398 — SUCCESS |
| T135 | #200 | `a6cb7b38aa47076b6f6b13bac07f713e59d9f085` | quality 34908127735; desktop-quality 34908127649; windows-terminal 34908127432; t135 34908127505 — SUCCESS |
| T135 repair | #201 | `ace55306e1e68f43c56a5e7acf49258b7565e7a3` | quality 34913795707; desktop-quality 34913795651; t135 34913795767 — SUCCESS |
| T136 | #202 | `94190a34379ae7f467828fdfc1a519449f9a6e9d` | quality 34921815246; desktop-quality 34921815048; windows-terminal 34921815134; t135 34921815216; t136 34921815060 — SUCCESS |
| T137 | #203 | `61ef22bcf620d69ec351ff896c829a006c95ca44` | quality 34932582008; desktop-quality 34932581895; windows-terminal 34932581919; t135 34932581915; t136 34932582004; t137 34932581935 — SUCCESS |
| T138 | #204 | `d057db587f8c638b2be48d80dcdceae843f4cbe4` | quality 34940973717; desktop-quality 34940973746; windows-terminal 34940973684; t135 34940973778; t136 34940973748; t137 34940973740; t138 34940973729 — SUCCESS |
| T139 | #205 | `26f557bf07deaf667730e711ba988b8438179567` | quality 34945696405; desktop-quality 34945696318; t135–t139 all SUCCESS |
| T140 | #206 | `4d95f04e4c8216218db737b55ba8f50649b37c43` | quality 34952824830; desktop-quality 34952824860; t135–t140 all SUCCESS |
| T141 | #207 | `f3bee1a51143aecfdf7102bdcc34d3f11c6cafd8` | quality 35002614860; desktop-quality 35002614769; t135–t141 all SUCCESS |
| T142 | #208 | `f89dd4f65173b298f36b5da85e2520d6df1dddda` | quality 35034668482; desktop-quality 35034668519; windows-terminal 35034668469; t135–t142 all SUCCESS |
| Amendment 002 | #209 | `54880f256551659be5fb4b4456754c7c9f69d9d9` | quality 35036384884 — SUCCESS |
| T142A | #210 | `8d75cab455f11afddeaa6fbe6244dbb1021d1467` | quality 35040530363; desktop-quality 35040530400; t135–t142a all SUCCESS |
| Amendment 003 | #213 | `55c29ebb5833a1f2856e6f3a820cc5484940b705` | quality 35093607276 — SUCCESS |
| Amendment 004 | #214 | `4afa6dca5682a01cc56efd01167f14164d90cd8a` | quality 35113347555 — SUCCESS |
| Amendment 005 | #216 | `74f8d9aa39ecd2fb0a49d99418e9264d85d3a657` | quality 35116477397 — SUCCESS |
| T143 T090 repair | #215 | `50c0f0aea5519eaa14e64fe062bef4a2b6ac6cd0` | quality 35165443611; windows-terminal 35165443610; t141 35165443655; t142 35165443644 — SUCCESS |
| Amendment 006 | #218 | `c6f80ec91120d1a3f906fd6e01e159a47dd7ad0b` | quality 35172009236 — SUCCESS |
| T143 selection repair | #219 | `3dbd5828120095eff3b9db4c358357918fd701b7` | desktop/T135–T142A push workflows SUCCESS; quality 35173120818 FAILED on macOS SQLite I/O and remains historical |
| Amendment 007 | #220 | `5b89a4a5a8b42d0c9bac1fb3b2da3e27f274266f` | quality 35174273459 — SUCCESS |
| T143 macOS quality repair | #221 | `bac506c4c879f30f6291e9d265a716bb2dc67364` | quality 35223114429; windows-terminal 35223114485 — SUCCESS |
| T143 final | #211 | `ff30e62f129525954acf27468d553018adbc05c6` | quality 35310901117; desktop-quality 35310901072; t135 35310901089; t136 35310901037; t137 35310901038; t138 35310901017; t139 35310901131; t140 35310901095; t141 35310901080; t142 35310901107; t142a 35310901109; t143 35310901082 — SUCCESS |

The failed post-merge quality run on `3dbd5828...` is not relabelled. Amendment 007 and PR #221 were the forward-only governance/qualification repair path. No failed candidate or failed run is reused as passing evidence.

## 3. Final implementation-bearing qualification

The final implementation-bearing T143 head is `6bc67242eb150cee17a4111587baa133d6e4f09c`, tree `9e6720933d0e1ce8bc27ecfb93d3774d16ad51d3`. It received a 24/24-file, 100%-coverage delegated exact-head review with zero material findings and PASS disposition. Exact-head T143 run `35309093080` succeeded and retained artifact `10533402246` with digest `sha256:6ec49318c50b9744e4f1fa45768a05793e5e83f221bc701cfebc33684a65855e`.

Guarded landing produced canonical merge `ff30e62f129525954acf27468d553018adbc05c6`, ordered parents `bac506c4c879f30f6291e9d265a716bb2dc67364` then `6bc67242eb150cee17a4111587baa133d6e4f09c`, and the same accepted tree `9e6720933d0e1ce8bc27ecfb93d3774d16ad51d3`.

Post-merge T143 run `35310901082` succeeded with artifact `10533950621`, digest `sha256:bf51b9f56a702c832dc28b2ce03871a6478f54323da3b9a49af2b066e96168f2`, exact canonical commit/tree binding, and `all_checks_pass=true`. Canonical measurements: large-search 23 ms p95, selection 28 ms p95, single/dual 25 ms p95, composer 2 ms p95, cached right-dock 15 ms p95, large-scroll 17 ms p95, large-focus 8 ms p95; 101 Projects / 1003 Sessions; cold launch 1110.398 ms p95 across 20 samples; idle CPU 0.5645%; renderer+host RSS 316.273 MiB under the Amendment-003 320 MiB ceiling; inherited huge-output/resize and exact-candidate assembly PASS.

Every workflow actually triggered by `ff30e62...` completed SUCCESS. No pre-final T143 head is qualification authority for the accepted release candidate.

## 4. T144 human visual acceptance

T144 created no implementation successor because the first exact candidate received a human PASS with no material visual defect.

- release candidate: `ff30e62f129525954acf27468d553018adbc05c6`;
- tree: `9e6720933d0e1ce8bc27ecfb93d3774d16ad51d3`;
- desktop build run: `35310901107`; macOS artifact id: `10533139511`;
- evidence-carrier PR #222, branch head `a5562313b1bd36d2b606c76813d4992129fbb88b`;
- reviewed capture set: `docs/evidence/010-t144-founder-visual-review/manifest.json` on the evidence carrier, 10 captures;
- independently authored human record: https://github.com/TheHalfMoon/Winds/pull/222#issuecomment-5727446349;
- reviewer/account `@TheHalfMoon`; GitHub timestamp `2026-09-18T08:36:59Z`;
- decision `FOUNDER_T144_VISUAL_DECISION=PASS`; no material visual defects.

Automation did not author or synthesize the Founder decision. It only reconciled the independently authored record. PR #222 was closed unmerged because it was an evidence review carrier, not a release-candidate successor; canonical `main` therefore remained the exact human-reviewed candidate.

## 5. Historical material evidence remains historical

`docs/provenance/010-t143-performance.md` is the retained T143 failure/diagnostic ledger. It preserves the stale T142A gate failure, repeated RSS failures, rejected allocator/JSC experiments, transport/readiness defects, selection/focus races, the macOS SQLite quality failure, large-fixture timeout, large-search/focus scalability failures, and the measured path that justified bounded paged windowing.

- No failed T143 candidate was rerun-to-green and represented as first-attempt authority.
- Post-merge quality run `35173120818` on merge `3dbd5828...` remains a real failure.
- Amendment 007 and PR #221 are forward-only repairs, not a waiver.
- Successful sub-results from failed exact heads are diagnostic only and never replace final `all_checks_pass=true`.
- The human T144 PASS is bound only to `ff30e62...` / `9e67209...` and would have become stale if a polish successor had moved the candidate.

## 6. Schema, persistence, dependency, and scope reconciliation

Spec 009 migration 0011 is byte-identical to canonical Spec 009 closeout `10b7c7be292549e3a8f5315e7d513020be85b4a3`: blob `2ba3e3eb1e21e49780966829dc433d7b94577284`, SHA-256 `9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d`.

Spec 010 adds exactly one presentation migration: `migrations/0012_desktop_presentation.sql`, blob `c4b38a229273f1617e1a2cff8f96d1b899e39faa`, SHA-256 `b1972c19531fdc3d358abdaf1d948037cd7067e9eb49717ff3b5211ddc971f41`. It stores removable presentation metadata/layout and does not replace canonical identity or prove restored child ownership.

Final direct desktop dependencies: `@tauri-apps/api=2.11.1`, `@xterm/addon-fit=0.11.0`, `@xterm/xterm=6.0.0`, `react=19.3.0`, `react-dom=19.3.0`. Direct dev dependencies: `@tauri-apps/cli=2.11.4`, `@types/node=22.20.2`, `@types/react=19.3.0`, `@types/react-dom=19.3.0`, `@vitejs/plugin-react=6.1.1`, `typescript=7.0.2`, `vite=8.3.0`. Package/toolchain contract is npm 10.9.8 / Node 22.22.3. Host uses `tauri=2.11.5`, `tauri-build=2.6.3`, and local `winds-control`.

`docs/provenance/010-t128-desktop-dependencies.md` remains the dependency qualification basis. `lucide-react` is not in the final manifest and is not claimed as installed. T145 adds no dependency, lockfile, migration, workflow, source, runtime, or product behavior change.

## 7. Runtime, renderer, child-process, and platform truth boundary

`RUST_WINDS_AUTHORITY != RENDERER_PRESENTATION`; `RUNTIME != PROVIDER != MODEL`; `CANONICAL_WINDS_SESSION != NATIVE_PROVIDER_SESSION`; `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`; `PRESENTATION_RESTORE != LIVE_CHILD_OWNERSHIP`; `DIFF_RENDERED != VERIFIED`; `DONE != VERIFIED`.

Rust retains privileged Store/Git/PTY/ConPTY and canonical truth seams. Renderer content is untrusted presentation and has no generic shell/filesystem/Git/runtime dispatcher. Child terminal/agent text cannot manufacture runtime identity, evidence, human decisions, attention authority, or host actions.

T139 closes direct agent launch truthfully: `DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED`, `DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED`, `REAL_CLAUDE_EXECUTION=NO`, `REAL_CODEX_WORKER_EXECUTION=NO`. Inherited Spec 006 nonclaims remain `T079_LIVE_PASS=NO`, `T080_LIVE_PASS=NO`, `T082_WORKER_LIVE_PASS=NO`, `REAL_CLAUDE_EXECUTION=NO`, `REAL_CODEX_WORKER_EXECUTION=NO`. Codex/Claude session fixtures prove presentation/provenance/targeting only, not real provider execution.

T142 directly qualifies macOS 15, Ubuntu 24.04 Linux, native Windows 2025, and Ubuntu WSL2 as a separate Windows-hosted execution/path domain. It explicitly retains `OS_LEVEL_TAURI_WINDOW_FOCUS_AUTOMATION=NOT_CLAIMED`, `OS_LEVEL_IME_INJECTION_AUTOMATION=NOT_CLAIMED`, and `LIVE_NATIVE_SYSTEM_THEME_TRANSITION_AUTOMATION=NOT_CLAIMED`. T143 performance is Ubuntu/WebKitGTK-reference-bound, not cross-platform equivalence.

## 8. Evidence key

| Key | Canonical evidence |
| --- | --- |
| E128 | T128 inert shell/dependency qualification; `docs/provenance/010-t128-desktop-dependencies.md`; desktop dependency/security tests. |
| E129 | T129 Quiet Current visual system; `desktop/tests/visual-system.test.mjs`. |
| E130 | migration 0012; `src/t130_desktop_presentation_tests.rs`; removable presentation persistence. |
| E131 | `src/t131_desktop_projection_tests.rs`; Rust-owned Project/Session/runtime/attention projections. |
| E132 | `src/t132_desktop_bridge_tests.rs`; `desktop/tests/left-dock-model.test.mjs`. |
| E133 | `desktop/tests/session-surface-model.test.mjs`; typed work events, trust source, fixture-only composer. |
| E134 | `src/t134_dual_session_tests.rs`; `desktop/tests/dual-session-model.test.mjs`; exact-target/no-broadcast races. |
| E135 | `src/t135_desktop_terminal_tests.rs`; terminal model tests; `t135-terminal-renderer`. |
| E136 | `src/t136_desktop_files_tests.rs`; right-dock model tests; `t136-files-changes`. |
| E137 | `src/t137_desktop_inspection_tests.rs`; exact-source Evidence/Context/Artifacts; `t137-evidence-context-artifacts`. |
| E138 | `src/t138_desktop_attention_tests.rs`; command-palette/right-dock tests; `t138-human-attention`. |
| E139 | `docs/provenance/010-t139-desktop-runtime-authority.md`; runtime-authority tests/workflow. |
| E140 | `desktop/tests/t140-accessibility.test.mjs`; `t140-accessibility`. |
| E141 | `desktop/tests/t141-adversarial-security.test.mjs`; retained Rust adversarial suites; `t141-desktop-security`. |
| E142 | `docs/provenance/010-t142-native-platform.md`; native-platform tests/workflows. |
| E143 | T143 provenance + closure comment `5725974384`; post-merge run `35310901082`, artifact `10533950621`. |
| E144 | Human Founder PASS comment `5727446349`, exact candidate/tree/build/capture binding. |

## 9. FR-001..FR-133 reconciliation

| Requirement | Classification | Canonical evidence / truthful boundary |
| --- | --- | --- |
| FR-001 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-002 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-003 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-004 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-005 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-006 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-007 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-008 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-009 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-010 | `PROVEN_DETERMINISTIC` | E130 + E131. |
| FR-011 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-012 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-013 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-014 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-015 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-016 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-017 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-018 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-019 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-020 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140. |
| FR-021 | `PROVEN_PERFORMANCE_BOUND` | E132 + E143; >=100 Project / >=1000 Session scale and stable search/selection/focus. |
| FR-022 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140; dock collapse retains command-surface access. |
| FR-023 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-024 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-025 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-026 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-027 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-028 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-029 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-030 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-031 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-032 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-033 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-034 | `PROVEN_DETERMINISTIC` | E134 + E130; two-slot identity, zero broadcast, peer preservation, shared-worktree truth, presentation-only restore. |
| FR-035 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-036 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-037 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-038 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-039 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-040 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-041 | `PROVEN_DETERMINISTIC` | E131 + E132 + E139; requested/observed/unknown/mismatch identity stays source-labelled. |
| FR-042 | `PROVEN_GOVERNANCE_BOUNDARY` | E129 + E132; Winds-owned treatment without copied proprietary logo assets. |
| FR-043 | `PROVEN_DETERMINISTIC` | E131 + E139; runtime/provider/model remain distinct. |
| FR-044 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-045 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-046 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-047 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-048 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-049 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-050 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-051 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-052 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-053 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-054 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-055 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E138 + E141; immutable right-dock binding, stale rejection, read-only truth/attention. |
| FR-056 | `PROVEN_DETERMINISTIC` | E133 + E134; exact Session Slot target and consequential-target clarity. |
| FR-057 | `PROVEN_DETERMINISTIC` | E133 + E134; exact Session Slot target and consequential-target clarity. |
| FR-058 | `PROVEN_PLATFORM_BOUND` | E135 + E142; terminal renderer presents existing Rust PTY/ConPTY ownership. |
| FR-059 | `PROVEN_SECURITY_BOUND` | E133 + E135 + E141; terminal/agent output remains untrusted. |
| FR-060 | `PROVEN_DETERMINISTIC` | E133 + E137; agent activity cannot self-promote to verification/human decision. |
| FR-061 | `PROVEN_DETERMINISTIC` | E132 + E134; exact-slot selection and keyboard navigation. |
| FR-062 | `PROVEN_DETERMINISTIC` | E134 + E135; visual close is separate from explicit process termination. |
| FR-063 | `PROVEN_DETERMINISTIC` | E133 + E134 + E139; historical/non-live/unavailable state explicit. |
| FR-064 | `PROVEN_GOVERNANCE_BOUNDARY` | E131 + E139; runtime continuity distinctions preserved; direct Codex/Claude launch remains unauthorized. |
| FR-065 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-066 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-067 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-068 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-069 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-070 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-071 | `PROVEN_DETERMINISTIC` | E138 + E132 + E140; keyboard-reachable navigation/search, ambiguity, pointer parity, no authority by ranking. |
| FR-072 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-073 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-074 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-075 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-076 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-077 | `PROVEN_DETERMINISTIC` | E138 + E132; deterministic accepted-fact attention and completion != verification. |
| FR-078 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; Winds-owned visual system and dark/light accepted. |
| FR-079 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; Winds-owned visual system and dark/light accepted. |
| FR-080 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; Winds-owned visual system and dark/light accepted. |
| FR-081 | `PROVEN_DETERMINISTIC` | E129 + E140; brand accent separate from semantic state encoding. |
| FR-082 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; bounded purposeful motion. |
| FR-083 | `PROVEN_DETERMINISTIC` | E129 + E140 + E144; reduced motion preserves state. |
| FR-084 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; material treatment/density/distinct Winds identity accepted. |
| FR-085 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; material treatment/density/distinct Winds identity accepted. |
| FR-086 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; material treatment/density/distinct Winds identity accepted. |
| FR-087 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-088 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-089 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-090 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-091 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-092 | `PROVEN_DETERMINISTIC` | E140 + E132 + E134 + E138; accessible names/roles/states, non-color semantics, focus, 200% scale, high contrast. |
| FR-093 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-094 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-095 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-096 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-097 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-098 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-099 | `PROVEN_PERFORMANCE_BOUND` | E143; exact-candidate launch/interaction/idle/scale and provenance-bound raw evidence. |
| FR-100 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-101 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-102 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-103 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-104 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-105 | `PROVEN_SECURITY_BOUND` | E128 + E135 + E136 + E137 + E141; untrusted renderer/content and fixed typed bridge. |
| FR-106 | `PROVEN_GOVERNANCE_BOUNDARY` | E128 + E141; local operation has no hidden cloud/control-plane requirement. |
| FR-107 | `PROVEN_SECURITY_BOUND` | E128 + E141; secret-shaped material campaign and presentation metadata boundary. |
| FR-108 | `PROVEN_GOVERNANCE_BOUNDARY` | E141 + canonical Tasks; no browser/remote/public IPC/daemon/generic plugin/automatic Git landing. |
| FR-109 | `PROVEN_PLATFORM_BOUND` | E142; claims limited to directly exercised macOS/Linux/native-Windows/WSL domains. |
| FR-110 | `PROVEN_PLATFORM_BOUND` | E142; claims limited to directly exercised macOS/Linux/native-Windows/WSL domains. |
| FR-111 | `PROVEN_PLATFORM_BOUND` | E142; claims limited to directly exercised macOS/Linux/native-Windows/WSL domains. |
| FR-112 | `PROVEN_PLATFORM_BOUND` | E142; claims limited to directly exercised macOS/Linux/native-Windows/WSL domains. |
| FR-113 | `PROVEN_DETERMINISTIC + TRUTHFUL_RUNTIME_NONCLAIM` | E133 + E139; agent surface proven as presentation; real direct Codex/Claude execution remains unauthorized. |
| FR-114 | `PROVEN_DETERMINISTIC + TRUTHFUL_RUNTIME_NONCLAIM` | E133 + E134 + E139; exact IME-aware composer is fixture-only and cannot imply live dispatch. |
| FR-115 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; typed work events/provenance retain agent-reported vs Winds-observed vs human-decided. |
| FR-116 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; typed work events/provenance retain agent-reported vs Winds-observed vs human-decided. |
| FR-117 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; typed work events/provenance retain agent-reported vs Winds-observed vs human-decided. |
| FR-118 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; typed work events/provenance retain agent-reported vs Winds-observed vs human-decided. |
| FR-119 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; typed work events/provenance retain agent-reported vs Winds-observed vs human-decided. |
| FR-120 | `PROVEN_DETERMINISTIC` | E134; adversarial target race proves exact one-slot targeting and zero broadcast. |
| FR-121 | `PROVEN_GOVERNANCE_BOUNDARY` | E133 + E134 + E135 + E139; consequential actions preserve underlying authority; unauthorized agent dispatch stays unavailable. |
| FR-122 | `PROVEN_DETERMINISTIC` | E132 + E138; truthful background lifecycle/attention rollups. |
| FR-123 | `PROVEN_GOVERNANCE_BOUNDARY` | E133 + E141; no hidden chain-of-thought access or claim. |
| FR-124 | `PROVEN_SECURITY_BOUND` | E133 + E141; content selection/copy cannot inject trusted controls/host action. |
| FR-125 | `PROVEN_DETERMINISTIC` | E130 + E133 + E139; history/presentation truth without fabricated live completeness. |
| FR-126 | `PROVEN_HUMAN_VISUAL` | E129 + E133 + E144; progressive inline interaction accepted. |
| FR-127 | `PROVEN_HUMAN_VISUAL` | E129 + E133 + E144; dense typed-event scanability accepted. |
| FR-128 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; complete component states accepted. |
| FR-129 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; short purposeful transitions and reduced-motion behavior accepted. |
| FR-130 | `PROVEN_DETERMINISTIC` | E132 + E143; Project-scoped Session organization scales to 1003 Sessions. |
| FR-131 | `PROVEN_HUMAN_VISUAL` | E129 + E144; familiar interaction grammar accepted. |
| FR-132 | `PROVEN_HUMAN_VISUAL + GOVERNANCE_BOUNDARY` | E129 + E144; accepted Winds identity is distinct; no competitor trade-dress copy claimed. |
| FR-133 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E141; immutable binding and delayed-response race rejection prevent cross-session truth attribution. |

## 10. SC-001..SC-031 reconciliation

| Criterion | Classification | Canonical evidence / truthful boundary |
| --- | --- | --- |
| SC-001 | `PROVEN_DETERMINISTIC` | E130 + E131 + E132; multi-Project/multi-Session organization preserves canonical identity. |
| SC-002 | `PROVEN_DETERMINISTIC + PLATFORM_BOUND` | E134 + E135; exactly two independent visible targets, exact input ownership, zero implicit broadcast. |
| SC-003 | `PROVEN_DETERMINISTIC` | E134; resize/swap/maximize/restore/replace/close preserve the untouched peer. |
| SC-004 | `PROVEN_DETERMINISTIC` | E134; shared-worktree condition is explicit. |
| SC-005 | `PROVEN_DETERMINISTIC` | E132 + E139; runtime identity states preserve source truth. |
| SC-006 | `PROVEN_DETERMINISTIC` | E132 + E140; identity retains text/glyph semantics without color dependence. |
| SC-007 | `PROVEN_DETERMINISTIC` | E132 + E138; collapsed Project exposes material attention. |
| SC-008 | `PROVEN_SECURITY_BOUND` | E132 + E141; duplicate aliases remain ambiguous. |
| SC-009 | `PROVEN_SECURITY_BOUND` | E136; Files/Changes follow exact current worktree binding. |
| SC-010 | `PROVEN_SECURITY_BOUND` | E136 + E137; candidate/tree movement invalidates trusted dock treatment. |
| SC-011 | `PROVEN_SECURITY_BOUND` | E133 + E141; forged trusted-state text cannot elevate authority. |
| SC-012 | `PROVEN_DETERMINISTIC` | E132 + E138 + E140; primary flows are keyboard-reachable with visible focus. |
| SC-013 | `PROVEN_DETERMINISTIC` | E132 + E134 + E138 + E140; pointer parity retained. |
| SC-014 | `PROVEN_HUMAN_VISUAL` | E129 + E140 + E144; reduced-motion/high-contrast/dark/light/compact/200% fixtures preserve distinctions. |
| SC-015 | `PROVEN_HUMAN_VISUAL` | E144; exact-candidate independent Founder aesthetic PASS. |
| SC-016 | `PROVEN_PERFORMANCE_BOUND` | E143; cold launch and interaction budgets pass. |
| SC-017 | `PROVEN_PERFORMANCE_BOUND` | E143 + E134; dual/output/resize/focus stress passes without ownership contamination. |
| SC-018 | `PROVEN_PERFORMANCE_BOUND` | E143; idle CPU/RSS and large Project/Session campaigns pass. |
| SC-019 | `PROVEN_SECURITY_BOUND` | E141; hostile clipboard/link/trusted-badge/bridge fixtures fail closed. |
| SC-020 | `PROVEN_PLATFORM_BOUND` | E142; every released platform claim directly exercised; no domain substitution. |
| SC-021 | `PROVEN_GOVERNANCE_BOUNDARY` | E141 + scope inventory; no daemon/public IPC/remote/browser/generic-plugin/automatic-routing/automatic-landing expansion. |
| SC-022 | `PROVEN_DETERMINISTIC` | E130 + E141; removable presentation metadata does not destroy canonical history. |
| SC-023 | `PROVEN_DETERMINISTIC` | E130 + E139; restart never proves restored live-child ownership. |
| SC-024 | `T145_FINAL_GATE` | Implementation candidate ff30e62 has full canonical implementation evidence; fresh exact T145 reviews/CI/guarded landing/post-merge remain required. |
| SC-025 | `T145_FINAL_GATE` | This artifact reconciles all FR/SC and inherited truth boundaries; closes only after exact T145 landing/post-merge proof. |
| SC-026 | `PROVEN_DETERMINISTIC + TRUTHFUL_RUNTIME_NONCLAIM` | E133 + E134; two Codex/Claude-style fixture Sessions use fixture-only prompts/typed events with zero broadcast; not real provider execution. |
| SC-027 | `PROVEN_DETERMINISTIC` | E133 + E136 + E137; representative work stream covers command/file/diff/test/approval/error/completion with trust source and dock navigation. |
| SC-028 | `PROVEN_PERFORMANCE_BOUND` | E132 + E143; 101 Projects / 1003 Sessions keep deterministic search/selection. |
| SC-029 | `PROVEN_GOVERNANCE_BOUNDARY` | E133 + E141; no hidden model chain-of-thought access/claim. |
| SC-030 | `PROVEN_HUMAN_VISUAL` | E144; Founder PASS covers the complete Winds craft rubric. |
| SC-031 | `PROVEN_SECURITY_BOUND` | E136 + E137 + E141; delayed A-result after switching to B cannot render as B truth. |

## 11. Core invariant and negative-scope reconciliation

- Project/Session aliases, pin/order/collapse/archive state, layout state, and theme/density preferences remain presentation metadata and never replace canonical identity or authority.
- Dual-session dispatch is exact-target only. Shared Project/worktree context or visual proximity never creates broadcast authority.
- Right-dock requests/results retain immutable Project/Session/workspace/worktree/candidate/tree/source binding; late mismatches are discarded/staled, never rebound to the focused Session.
- Rust remains the privileged authority plane; terminal lifecycle stays Rust-owned; renderer and child output remain untrusted data.
- Evidence/Context/Artifacts/Needs You are projections over accepted facts; rendering never grants verification, acceptance, approval, or landing authority.
- Human Founder visual acceptance remains independently authored and exact-candidate bound.
- Migration 0011 remains unchanged from Spec 009; migration 0012 is presentation persistence only.
- Spec 006 live-runtime nonclaims remain intact; Spec 010 does not claim real direct Codex/Claude execution.
- T142 platform claims are domain-specific and T143 performance claims are Ubuntu/WebKitGTK-reference-bound.
- Historical failed/rejected/superseded evidence remains inspectable and is not rewritten by later success.
- No persistent daemon, public IPC/control server, browser runtime, remote control plane, generic plugin execution, provider fallback router, hidden cloud requirement, learning subsystem, second database, or automatic Git landing is introduced.
- T145 changes no product behavior and authorizes no successor implementation task.

## 12. T145 exact-candidate checklist and external landing gates

- Changed scope stays limited to `tasks.md` checked-state reconciliation and this T145 artifact unless a separately justified focused evidence correction is required.
- `git diff --check` and repository `quality` must pass on the exact final T145 head.
- Canonical frontend/desktop/platform/security/performance implementation evidence remains bound to accepted implementation tree `9e6720933d0e1ce8bc27ecfb93d3774d16ad51d3`; the docs-only T145 candidate must not masquerade as a new implementation build.
- Author correctness/safety/governance/evidence-integrity review and Ponytail/YAGNI review must pass.
- Fresh independent substantive review must cover the exact final T145 head with zero material findings and zero unresolved material threads.
- Immediately before landing, reconcile current `main`, PR base/head/tree, changed scope, ruleset, mergeability, and behind-by state.
- If `main` moves, forward-integrate it by normal merge and restart exact-head candidate-bound qualification/review; never rebase or force-push.
- Guarded normal merge must use the exact expected T145 head.
- Verify merge tree, ordered parents, GitHub signature, and every actually-triggered post-merge push workflow before declaring closure.

## 13. Closure state machine

Current pre-landing truth:

```text
T127..T144=CLOSED_CANONICAL
T145=IN_QUALIFICATION
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS=CLOSED_CANONICAL
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=NOT_YET_CLOSED
SUCCESSOR_IMPLEMENTATION_AUTHORIZED=NO
```

Only after exact T145 guarded landing and successful post-merge verification may repository truth state:

```text
T127..T145=CLOSED_CANONICAL
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS=CLOSED_CANONICAL
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SUCCESSOR_IMPLEMENTATION_AUTHORIZED=NO
```

Closing T145 authorizes no later Spec 010 implementation phase, no direct Codex/Claude live launch, no daemon/IPC/browser/remote/plugin expansion, and no automatic landing. Any later product program requires separate canonical authority.
