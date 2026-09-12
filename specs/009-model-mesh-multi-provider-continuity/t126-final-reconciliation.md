# T126 Final Reconciliation — Spec 009 Model Mesh & Explicit Multi-Provider Continuity

## Status and closure rule

- Initial T126 candidate base: `d6de382082d79bc939733588749986d6ca8a2fe6` (T125 guarded merge).
- Final T126 qualification base after the required forward-only inherited-fixture repair: `61479d50e72775e6ee663838ded37abfccd907a8`.
- T114..T125 are already `CLOSED_CANONICAL` under the exact ledger below. Spec 009 Tasks Amendment 002 and its bounded inherited T060 fixture repair are also `CLOSED_CANONICAL` before final T126 requalification.
- Relative to the final canonical qualification base, this T126 candidate remains documentation/evidence only: `tasks.md` checked-state reconciliation plus this artifact. It changes no production source, runtime behavior, dependency, lockfile, workflow, migration, schema, provider/runtime execution, credential path, Workbench behavior, or Git authority.
- T126 becomes `CLOSED_CANONICAL` only after this exact documentation candidate lands through guarded normal merge and every actually-triggered post-merge push check succeeds.
- Exact T126 HEAD/TREE, candidate reviews, CI, merge identity, and post-merge checks remain in the PR/merge trail to avoid a self-referential candidate hash.

## 1. Canonical governance chain

| Unit | PR | Canonical merge | Post-merge verification |
| --- | ---: | --- | --- |
| Spec 009 Entry | #166 | `77933ae166b885c09f9d1c1e9971610fe179b230` | `quality` 34640439245 SUCCESS attempt 1 |
| Spec 009 Specification | #167 | `a8525b68d7d5ca83cfd3c475cbae18e131700e07` | `quality` 34641382225 SUCCESS attempt 1 |
| Spec 009 Plan | #168 | `3e1d13730b5ccae41e997515653fb1546ef66f70` | `quality` 34646032061 SUCCESS attempt 1 |
| Spec 009 Tasks | #169 | `6ac57311e17b52c014a6ee14d99ec60968736230` | `quality` 34648490635 SUCCESS attempt 1 |

Every merge in this governance chain has GitHub verification `verified=true`, `reason=valid`.

## 2. T114–T125 canonical implementation ledger

Every canonical task merge below has a valid GitHub signature. Every listed post-merge workflow is an actually-triggered push workflow on that exact canonical merge and completed `SUCCESS` on attempt 1.

| Task | PR | Accepted head | Accepted tree | Canonical merge | Actually-triggered post-merge push verification |
| --- | ---: | --- | --- | --- | --- |
| T114 | #170 | `3d2156100ab969065f8829bee173f881c532b8cb` | `84be93ca41ba526c8cb0f6ce9618991c7d2dee78` | `f2a693394d63909e08ea05ab1e53967d1c1f30f6` | `quality` 34651625062 SUCCESS attempt 1; `windows-terminal` 34651625066 SUCCESS attempt 1 |
| T115 | #171 | `e781b9e9c75bf69060d700686e7a64ca99adf5de` | `20ae97af9bb63fbe354e407013bde82393cd931b` | `97c37c0f6419deae831a1b02abc0291b2216e994` | `quality` 34656078078 SUCCESS attempt 1; `windows-terminal` 34656078112 SUCCESS attempt 1 |
| T116 | #172 | `708eb73645936f20da9d261787214b539605735c` | `c518232daf464e8d7729a7999aea4e7377dd4e24` | `714ad136613d60ae0c151d52e2afad9b3730162e` | `windows-terminal` 34660970801 SUCCESS attempt 1; `quality` 34660970807 SUCCESS attempt 1 |
| T117 | #173 | `a34a942497dd41e34d53a99ff2b30eb2bd8f194e` | `e576707293bdf0d7bb3e77f402ac65e5bc9edda0` | `32e35a66cefca708ef9320a0191cfbbcf9ce3756` | `quality` 34663187004 SUCCESS attempt 1; `windows-terminal` 34663187037 SUCCESS attempt 1 |
| T118 | #174 | `a7a4b5ff8698205f73fdfacbfe61a22d08fc2284` | `4883fa9478bb7075ca8ca30b80051b5ebb174252` | `66ac70669844d5561a5febc6956b90007910db45` | `quality` 34666964878 SUCCESS attempt 1; `windows-terminal` 34666964883 SUCCESS attempt 1 |
| T119 | #175 | `e1db4eb5044bc443ff4c4f454b3251260be85074` | `af6944a456858fd10f2d19b2a79186cd7a5c401a` | `910e0db52627cb44d50e70c786f3c7a1a4483721` | `quality` 34670175347 SUCCESS attempt 1; `windows-terminal` 34670175370 SUCCESS attempt 1 |
| T120 | #176 | `b80c0d892477d563c26fcef59d18b059d2472e51` | `8eb70b3d4818c29b6e484b79436511fce22e0981` | `53006852765083f582adf6754e95b4a843c6847c` | `quality` 34672371045 SUCCESS attempt 1; `windows-terminal` 34672371050 SUCCESS attempt 1 |
| T121 | #177 | `6091d847207cabce3723815e62e3c85ae563f1a8` | `926d2526cccc711febafa5f1ad87e919a55fc4cf` | `9951e1c460815b818cefc6a7ae67fa1e0a7949fe` | `quality` 34702814318 SUCCESS attempt 1; `windows-terminal` 34702814325 SUCCESS attempt 1 |
| T122 | #180 | `225a59ddd8508f4bec9ce2668160e4878e23b3d7` | `02950426ad2bf3f56663a20e208e9bfd9e6a87b0` | `a569534488991b58a77149ceae1833467488175d` | `quality` 34704065926 SUCCESS attempt 1 |
| T123 | #181 | `533f468b9a956acf8e5a73e65229f542a26d610b` | `b05c51dc21973f05ffc40749c9554bc5b5bdd421` | `20d1ea020c79c88ed883fb956c0b4dc401dd8ca6` | `windows-terminal` 34707095529 SUCCESS attempt 1; `quality` 34707095539 SUCCESS attempt 1 |
| T124 | #182 | `865662aaa22fb6f0854351b49119cbdc23d3989a` | `56a317a093f5374ea9c7a0d6a46331fe9ace97f5` | `10d1b9e9cbd4ded56cefecf32f4a04d03d623846` | `quality` 34707928077 SUCCESS attempt 1 |
| T125 | #183 | `0559a6a8636f2165f5de2638deb4a2627093d010` | `a2da524e83ef882955bf46f4e0ec18e8fbe0a594` | `d6de382082d79bc939733588749986d6ca8a2fe6` | `quality` 34708984736 SUCCESS attempt 1 |

## 3. Final implementation-bearing qualification

T123 is the final implementation-bearing Spec 009 task. Its exact accepted candidate is `533f468b9a956acf8e5a73e65229f542a26d610b`, tree `b05c51dc21973f05ffc40749c9554bc5b5bdd421`.

On that exact head, first-attempt pull-request workflows all succeeded:

- `quality` run `34706322484`;
- `release-candidate` run `34706322502`;
- `t097-performance` run `34706322491`;
- `windows-terminal` run `34706322494`.

The fresh independent T123 review found no material finding. Its guarded merge is `20d1ea020c79c88ed883fb956c0b4dc401dd8ca6`, with the exact candidate tree and ordered parents `[a569534488991b58a77149ceae1833467488175d, 533f468b9a956acf8e5a73e65229f542a26d610b]`. Post-merge `quality` 34707095539 and `windows-terminal` 34707095529 succeeded on attempt 1.

T124 and T125 are documentation-only qualification/necessity gates. They do not move implementation state. T124 explicitly distinguishes inherited T123 platform evidence from exact-head T124 repository-quality evidence. T125 closes usage/cost observation as `UNKNOWN` because no production-qualified structured local source exists.

## 4. Historical material evidence is preserved

### Plan review lineage

PR #168 did not treat early review findings as success. Its superseded heads remain material history:

- `3bcd5f42be0c9039b76598a398597d413994d648`: two material findings — continuity identity claims were not role-specific enough to prove source/destination provider/model identity, and persisted authority references were not exact content-bound target authority.
- `cb158d17ec1d8a3453bfd0d7b4e37c1674ba7d2e`: role-specific identity was repaired; one material exact-target authority gap remained.
- `7711c469318a690630a61fbafaf405115d41c131`: versioned target/continuity descriptors and content-bound Model Mesh authority envelopes repaired generic-authority reuse; one exact actor-binding/Winds-session scope gap remained.
- `104e3239830ad9c040a642e21e79251eb1681dd9`: actor-binding/session scope was repaired; one durable `actor_role` reconstructibility gap remained.
- `44ba5d1c1bbd3e3dda5972eb620a9ac74dc0d24f`: immutable target-request `actor_role` closed the remaining gap. Fresh review found no remaining material actionable finding before canonical Plan landing.

Historical local documentation-only Plan full-suite runs that encountered the inherited macOS T090 bounded cleanup class remain failures and were never substituted for exact-head GitHub `quality` success.

### T121 inherited terminal failure and forward repairs

T121 candidate `ac1b8de9aba52431290e27a054edaf0dc20c4a4d` retained exact-head `quality` run `34677828080` attempt 1 as `FAILURE`. The only failed test was `execution::tests::dropping_live_terminal_records_only_proven_cleanup_truth`, which observed truthful `EXITED` state not covered by the inherited fixture. The same head's `release-candidate` 34677828084, `t097-performance` 34677828130, and `windows-terminal` 34677828086 succeeded; those successes never erased the failed quality gate.

Tasks Amendment 001 then canonically authorized one test-only truth reconciliation. The repair preserved production terminal cleanup semantics and accepted all already-canonical truthful cleanup outcomes. T121 was subsequently forward-integrated and fully requalified without force-push/rebase/history rewrite.

Independent T121 review also found and drove forward-only repairs for: read-only inspection creating Store state, incomplete canonical schema validation, cyclic StageRun lineage handling, collection limits applied after materialization, and corrupted non-HUMAN selector projection. The final T121 head `6091d847207cabce3723815e62e3c85ae563f1a8` received fresh review with no new blocking issue before guarded landing.

### T123 and T124 historical evidence

A local T123 full-suite run showed inherited T090 bounded terminal-cleanup failures; an untouched canonical-main control reproduced the same class. Neither run is represented as exact-head success. GitHub exact-head qualification is the successful T123 workflow set recorded above.

T124 initial head `ab904c7b152c178060653f08077aa740e3902733` had a material evidence-integrity wording finding because inherited T123 platform runs were described as if they were exact-head T124 platform evidence. The final forward-only head `865662aaa22fb6f0854351b49119cbdc23d3989a` corrected that distinction and received a fresh no-material-finding review before landing.

### T126 initial quality failure, Amendment 002, and bounded T060 repair

The initial T126 documentation candidate `f3465500dfdb4a44855419843871e6bf2ebbe2fc`, tree `2daf07d5f1989f9c7a9745e49781d261e9a622ef`, preserved exact-head `quality` run `34709700248` attempt 1 as `FAILURE`. The macOS job succeeded. The Ubuntu job ran 624 tests and failed exactly one inherited fixture, `git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session`, with `617 passed; 1 failed; 6 ignored`. Production durably observed `ExecutionStatus::Exited`; the inherited success branch asserted `ExecutionStatus::Interrupted`. The failed run was not rerun, waived, or classified as a flake.

Fresh independent review of that exact T126 head found no material defect in the reconciliation artifact or checked-state reconciliation and explicitly retained the failed `quality` gate as blocking acceptance.

Spec 009 Tasks Amendment 002 then qualified independently on PR #185 from exact head `5fa13768ddd702a30a57faa1ab7a1e737a7e4b7e`, tree `a9a2ed2f7eb9e2cecc4b4e9771c0daec8b3f24e6`. Exact-head `quality` run `34710545884` succeeded on Ubuntu and macOS on attempt 1; independent review found no material finding; the amendment guarded-landed as `2a96d14177e9e5605785021f6918d07ba418a05f` with ordered parents `[d6de382082d79bc939733588749986d6ca8a2fe6, 5fa13768ddd702a30a57faa1ab7a1e737a7e4b7e]`, tree `a9a2ed2f7eb9e2cecc4b4e9771c0daec8b3f24e6`, and GitHub `verified=true` / `reason=valid`. Post-merge `quality` run `34710725816` succeeded on attempt 1.

The amendment authorized only `src/t060_fault_tests.rs` inside `input_and_resize_racing_with_exit_never_reopen_final_session()`. Repair PR #186 exact head `0c55015a459ee25828f668857e20aeb3eb71d850`, tree `20682cf37e68dfb2664eb43e2fcc226892ed3cc7`, changed only that function. It preserved the three canonical bounded-cleanup truths: natural `EXITED/PROCESS_EXITED`, controlled `INTERRUPTED/TERMINATED_BY_WINDS`, and bounded-unproven `OWNERSHIP_LOST/OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`. No production statement, timeout, retry, workflow, dependency, schema, migration, Model Mesh behavior, or authority changed.

Before publication, the repaired focused T060 test passed locally, format and Clippy passed, while one local macOS full-suite attempt retained the known inherited T090 bounded-cleanup class as `598 passed; 7 failed; 4 ignored`. Those seven local T090 failures were not rerun into green and are not represented as acceptance evidence.

On exact repair head `0c55015a459ee25828f668857e20aeb3eb71d850`, first-attempt GitHub workflows all succeeded: `quality` `34711069905`, `release-candidate` `34711069851`, and `windows-terminal` `34711069916`, including Ubuntu/macOS full quality, native Windows terminal, real Windows Server + Ubuntu WSL2, Ubuntu/macOS terminal integration, T063 soaks on Ubuntu/macOS/Windows, T064 regressions, SC-001 soak, and release builds. Independent review found no material finding and zero review threads remained unresolved.

The repair guarded-landed as `61479d50e72775e6ee663838ded37abfccd907a8` with ordered parents `[2a96d14177e9e5605785021f6918d07ba418a05f, 0c55015a459ee25828f668857e20aeb3eb71d850]`, tree `20682cf37e68dfb2664eb43e2fcc226892ed3cc7`, and GitHub `verified=true` / `reason=valid`. Post-merge `quality` `34711467693` and `windows-terminal` `34711467751` both succeeded on attempt 1. Only after those post-merge gates completed was repaired canonical `main` forward-integrated into the existing T126 branch through a normal merge commit; no rebase, force-push, or history rewrite occurred.

## 5. Schema, persistence, dependency, and scope reconciliation

The sole Spec 009 migration is immutable `migrations/0011_model_mesh_continuity.sql`, SHA-256:

```text
9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d
```

Selected migration objects are exactly:

- tables: `model_mesh_target_requests`, `model_mesh_identity_claims`, `model_mesh_continuity_events`, `model_mesh_continuity_identity_claims`;
- indexes: `idx_model_mesh_target_requests_stage_time`, `idx_model_mesh_target_requests_actor_time`, `idx_model_mesh_identity_claims_request_time`, `idx_model_mesh_identity_claims_actor_time`, `idx_model_mesh_continuity_events_request_time`, `idx_model_mesh_continuity_identity_claims_claim`;
- triggers: `trg_model_mesh_target_request_scope_insert`, `trg_model_mesh_target_request_approval_insert`, `trg_model_mesh_target_requests_no_update`, `trg_model_mesh_target_requests_no_delete`, `trg_model_mesh_identity_claim_scope_insert`, `trg_model_mesh_identity_claims_no_update`, `trg_model_mesh_identity_claims_no_delete`, `trg_model_mesh_continuity_event_lineage_insert`, `trg_model_mesh_continuity_event_authority_insert`, `trg_model_mesh_continuity_events_no_update`, `trg_model_mesh_continuity_events_no_delete`, `trg_model_mesh_continuity_identity_claim_role_insert`, `trg_model_mesh_continuity_identity_claims_no_update`, `trg_model_mesh_continuity_identity_claims_no_delete`.

From canonical Tasks merge `6ac57311e17b52c014a6ee14d99ec60968736230` through T125 merge `d6de382082d79bc939733588749986d6ca8a2fe6`, Spec 009 changed one migration and no `Cargo.toml`, `Cargo.lock`, or `.github/workflows` file. Existing `winds.db`/rusqlite remains the selected durable store. No second database, ORM, queue, event-sourcing framework, provider SDK/API, billing/pricing database, telemetry service, provider gateway, policy engine, daemon/server/socket/IPC, remote/browser runtime, MCP/ACP/A2A expansion, semantic/vector/RAG subsystem, learning/training/RL system, or automatic Git action was added.

The Amendment-002-authorized `src/t060_fault_tests.rs` change is test-only and reconciles an inherited terminal fixture; it does not change production terminal behavior.

## 6. Platform, runtime, provider, and live-execution truth boundary

Platform evidence is not transferred between domains. T123 exact-head workflows directly exercised the applicable Ubuntu/Linux, macOS, native Windows, and Windows+Ubuntu WSL2 lanes encoded by the repository workflows. T124 adds no new platform behavior and makes no cross-platform inheritance claim.

None of those platform workflows proves a real Claude planner execution, a real Codex worker execution, provider/model authentication, provider billing, provider-private memory transfer, or physical native resume. Spec 006 live-runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Spec 009 deterministic tests exercise the already admitted `RuntimeKind::Codex` and `RuntimeKind::Claude` contract families without relabelling fixture/persistence evidence as real provider execution.

## 7. FR-001..FR-095 reconciliation

| Requirement | Classification | Canonical evidence / truthful boundary |
| --- | --- | --- |
| FR-001 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-002 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-003 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-004 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-005 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-006 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-007 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-008 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-009 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-010 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-011 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-012 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| FR-013 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-014 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-015 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-016 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-017 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-018 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-019 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-020 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-021 | `PROVEN_DETERMINISTIC` | T115 source-labelled adapters; `src/agentic_runtime.rs`; T117 drift evaluation; T123 forged-identity campaign. |
| FR-022 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-023 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-024 | `EXPLICIT_TRUTHFUL_NONCLAIM` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. First-slice `EXPLICIT_POLICY` is intentionally `POLICY_NOT_AUTHORIZED`; no policy-selected target is claimed. |
| FR-025 | `EXPLICIT_TRUTHFUL_NONCLAIM` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. First-slice `EXPLICIT_POLICY` is intentionally `POLICY_NOT_AUTHORIZED`; no policy-selected target is claimed. |
| FR-026 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-027 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-028 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-029 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-030 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-031 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-032 | `PROVEN_DETERMINISTIC` | T114 exact fail-closed resolver; T115 exact authority revalidation; T123 no-fallback/adversarial coverage. |
| FR-033 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-034 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-035 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-036 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-037 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-038 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-039 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-040 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-041 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-042 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-043 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-044 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-045 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| FR-046 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-047 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-048 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-049 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-050 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-051 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-052 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-053 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-054 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-055 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-056 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-057 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-058 | `PROVEN_DETERMINISTIC` | T117 drift/staleness evaluator; Spec 008 candidate/artifact freshness reuse; T123 drift/history campaign. |
| FR-059 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-060 | `PROVEN_GOVERNANCE_BOUNDARY` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-061 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-062 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-063 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-064 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-065 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-066 | `EXPLICIT_TRUTHFUL_NONCLAIM` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. No credential-reference mechanism was selected; future credential integration remains separately governed. |
| FR-067 | `PROVEN_GOVERNANCE_BOUNDARY` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| FR-068 | `PROVEN_DETERMINISTIC` | T120 read-only target/continuity/reviewer projections; T121 CLI; T123 forged-label campaign. |
| FR-069 | `PROVEN_DETERMINISTIC` | T120 read-only target/continuity/reviewer projections; T121 CLI; T123 forged-label campaign. |
| FR-070 | `PROVEN_DETERMINISTIC` | T120 read-only target/continuity/reviewer projections; T121 CLI; T123 forged-label campaign. |
| FR-071 | `EXPLICIT_TRUTHFUL_NONCLAIM` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. No qualified production usage/cost source exists; retention of observed usage is therefore intentionally not implemented. |
| FR-072 | `PROVEN_DETERMINISTIC` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. |
| FR-073 | `PROVEN_DETERMINISTIC` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. |
| FR-074 | `PROVEN_GOVERNANCE_BOUNDARY` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. |
| FR-075 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-076 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-077 | `PROVEN_GOVERNANCE_BOUNDARY` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-078 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-079 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-080 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-081 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-082 | `PROVEN_GOVERNANCE_BOUNDARY` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| FR-083 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-084 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-085 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-086 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-087 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-088 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-089 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-090 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-091 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-092 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-093 | `PROVEN_PLATFORM_BOUND` | T123 exact implementation-head `quality`, `release-candidate`, `t097-performance`, and `windows-terminal` runs all SUCCESS attempt 1; T124 limits claims to directly exercised domains. |
| FR-094 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| FR-095 | `PROVEN_PLATFORM_BOUND` | T123 exact implementation-head `quality`, `release-candidate`, `t097-performance`, and `windows-terminal` runs all SUCCESS attempt 1; T124 limits claims to directly exercised domains. |

## 8. SC-001..SC-025 reconciliation

| Criterion | Classification | Canonical evidence / truthful boundary |
| --- | --- | --- |
| SC-001 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| SC-002 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| SC-003 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| SC-004 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. |
| SC-005 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-006 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-007 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-008 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-009 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-010 | `PROVEN_DETERMINISTIC` | T118 continuity classifier/context; T119 append-only continuity events; T123 adversarial continuity coverage. |
| SC-011 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| SC-012 | `PROVEN_DETERMINISTIC` | T115/T118 secret-safe boundaries; T123 secret/private-context rejection; no provider credential integration. |
| SC-013 | `PROVEN_DETERMINISTIC` | Frozen migration 0011; T116 store qualification; T121 read-only schema/corruption checks; T123 corrupt-state campaign. |
| SC-014 | `PROVEN_DETERMINISTIC` | T120 read-only target/continuity/reviewer projections; T121 CLI; T123 forged-label campaign. |
| SC-015 | `PROVEN_DETERMINISTIC` | T120 read-only target/continuity/reviewer projections; T121 CLI; T123 forged-label campaign. |
| SC-016 | `EXPLICIT_TRUTHFUL_NONCLAIM` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. The no-qualified-source branch is proven: absent/conflicting/unqualified usage remains `UNKNOWN`; no structured-present production adapter is claimed. |
| SC-017 | `PROVEN_DETERMINISTIC` | T120 projection keeps usage/cost `UNKNOWN`; T123 proves cost prose has no authority; T125 qualifies no production source and closes `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`. |
| SC-018 | `PROVEN_DETERMINISTIC` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. |
| SC-019 | `PROVEN_PLATFORM_BOUND` | T123 exact implementation-head `quality`, `release-candidate`, `t097-performance`, and `windows-terminal` runs all SUCCESS attempt 1; T124 limits claims to directly exercised domains. |
| SC-020 | `T126_FINAL_GATE` | T120 reviewer projection; T123 forged persuasion/outcome campaign; T124 inherited verification/review boundary qualification. T123 implementation-head reviews are canonical; the final T126 exact-head author/Ponytail/independent-review gate remains external to this pre-landing artifact. |
| SC-021 | `PROVEN_GOVERNANCE_BOUNDARY` | Canonical Tasks Global Rules; T122 Workbench necessity = NO; no Cargo/workflow/provider-framework expansion through T125. |
| SC-022 | `PROVEN_PLATFORM_BOUND` | T123 exact implementation-head `quality`, `release-candidate`, `t097-performance`, and `windows-terminal` runs all SUCCESS attempt 1; T124 limits claims to directly exercised domains. |
| SC-023 | `PROVEN_DETERMINISTIC` | T116/T119 append-only persistence; T117 current-vs-history projection; T123 later-success/history attacks. |
| SC-024 | `PROVEN_DETERMINISTIC` | `src/model_mesh.rs`; T114 exact domain/resolver; T115 observation/authority adapters; T116 persistence; T123 adversarial campaign. This is deterministic contract/fixture qualification on the admitted Codex/Claude runtime families, not a real provider execution claim. |
| SC-025 | `EXPLICIT_TRUTHFUL_NONCLAIM` | Spec 006 nonclaims preserved by Tasks/T124/T125: T079/T080/T082 live PASS = NO and real Claude/Codex worker execution = NO. |

## 9. Core invariant reconciliation

- Canonical target descriptors are reconstructible from durable exact work/session joins plus immutable target-request scope, including actor binding, Winds session, actor role, runtime, provider, and model dimensions.
- Exact target/continuity authority is content-bound to versioned canonical descriptors and the exact binding/session/role/work scope; current execution/delegation authority remains independently evaluated and can reduce applicability after approval.
- Generic T076 approval content, generic workflow decisions, agent prose, labels, and same-stage approvals for another target/binding/session/role do not authorize Model Mesh actions.
- Provider/model/runtime/native-session identity remains source-labelled; agent reports never self-promote to Winds-observed truth.
- Resolver behavior is exact and fail-closed. Unknown, unavailable, ambiguous, conflict, stale, authentication-unknown, capability-unavailable, authority-denied, and policy-not-authorized states remain distinct, and there is no silent fallback.
- Native resume, reconstruction, reassignment, handoff, ownership loss, unavailable, and unproven continuity remain distinct. Identifier coincidence does not prove native resume.
- Continuity context preserves exact structured provenance; missing/redacted/material-loss state remains explicit; provider-private state is not invented; reviewer projections exclude source-agent persuasion/winner recommendations by default.
- Model Mesh history remains append-only and replay/collision safe. Later success does not rewrite prior failure, unavailability, stale state, reconstruction loss, or denied authority.
- Secret/authentication boundaries remain separate from identity and availability. No raw credential or provider-private memory path was introduced.
- Read-only CLI inspection validates canonical schema/lineage, refuses active SQLite sidecars, bounds exposed collections before dependent materialization, rejects corrupted non-HUMAN selectors, and does not initialize/mutate the Store.
- Usage/cost remains `UNKNOWN`. No guessed price/token count is represented as observed metadata and usage/cost has no routing/verification/acceptance authority.

## 10. T126 reconciliation checklist and external landing gates

- [x] T114..T125 reconciled to exact accepted heads, trees, canonical merges, valid signatures, and actually-triggered post-merge checks.
- [x] Spec 009 Entry/Specification/Plan/Tasks governance chain reconciled to canonical merges and post-merge verification.
- [x] FR-001..FR-095 individually classified.
- [x] SC-001..SC-025 individually classified; `SC-020` intentionally remains the T126 final exact-head review gate until the PR trail proves it.
- [x] Migration 0011 checksum and selected table/index/trigger inventory reconciled.
- [x] Dependency/workflow/persistence/negative-scope inventory reconciled.
- [x] Exact target authority, source-labelled identity, no-fallback routing, drift/staleness, continuity truth, append-only history, secret/auth posture, reviewer independence, and corruption preservation reconciled.
- [x] T125 usage/cost decision reconciled as `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN`.
- [x] Platform claims are bounded to directly exercised workflow domains; Spec 006 live-runtime nonclaims remain unchanged.
- [x] Historical Plan/T121/T123/T124 failed, superseded, stale, and repaired evidence remains explicitly material and inspectable.
- [x] Initial T126 `quality` failure `34709700248`, canonical Amendment 002, bounded T060 repair, repair exact-head qualification, and repair post-merge verification remain explicitly material and inspectable.

The following gates are intentionally external to this pre-landing artifact and MUST be proven on the exact final T126 commit in the PR/merge trail: repository `quality`; every actually-triggered applicable regression/platform/security workflow; author correctness/safety/governance/evidence-integrity review; Ponytail/YAGNI review; fresh independent substantive review; zero unresolved material findings/threads; exact main/base/head/tree/scope/ruleset/mergeability reconciliation; expected-head guarded normal merge; merge commit/tree/ordered parents/signature verification; and every actually-triggered post-merge push check.

## 11. Closure state machine

Before guarded T126 landing:

```text
T114..T125=CLOSED_CANONICAL
SPEC_009_TASKS_AMENDMENT_002=CLOSED_CANONICAL
T126_T060_FIXTURE_REPAIR=CLOSED_CANONICAL
T126=CANDIDATE_CLOSEOUT
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=NOT_YET_CLOSED
```

Only after this exact candidate lands and post-merge verification succeeds may repository truth state:

```text
T114..T126=CLOSED_CANONICAL
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=CLOSED_CANONICAL
SPEC_009_TASKS=CLOSED_CANONICAL
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

That closure authorizes no later Spec 009 implementation phase and does not authorize a broader provider/runtime fleet, explicit policy routing, provider APIs/SDKs, credential acquisition/brokerage, gateway, daemon/IPC, remote/browser execution, learning/training, semantic memory, new dependencies, automatic winner selection, or automatic Git landing.
