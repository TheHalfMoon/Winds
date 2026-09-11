# T113 Final Reconciliation - Spec 008 Resumable Workflow & Decision Ledger

## Status and closure rule

- Canonical T113 base: `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` (T112 guarded merge).
- This artifact is documentation/evidence only. It changes no source, migration, workflow, dependency, runtime, provider/browser, daemon/IPC, remote, learning, plugin, or landing behavior.
- T101..T112 are already `CLOSED_CANONICAL`. T113 becomes `CLOSED_CANONICAL` only after this exact documentation candidate lands by guarded normal merge and all actually-triggered post-merge push checks succeed.
- Exact T113 HEAD/TREE, review, CI, merge identity, and post-merge workflow evidence live in the PR/merge trail; this document does not fabricate a self-referential pre-commit hash.

## Canonical governance chain

- Spec 008 specification canonical merge: `5f25341c2f317f528e542d4d90a112c784ab24dc`.
- Spec 008 Plan canonical merge: `7993b496393beac282cf2ca611a2021e268ef2f6`.
- Spec 008 Tasks canonical merge: `fe79cc238cefa79f39544ce46c3bc7fc6e296ba5`.
- The Tasks file records `SPEC_008_ENTRY=CLOSED_CANONICAL`, `SPEC_008_SPEC=CLOSED_CANONICAL`, and `SPEC_008_PLAN=CLOSED_CANONICAL` as inherited canonical inputs.

## T101-T112 canonical implementation ledger

Every merge below was re-read from canonical first-parent history. GitHub verification for each listed merge is `verified=true`, `reason=valid`. Every listed push workflow completed `success` on attempt 1.

| Task | Canonical merge | Tree | Actually-triggered post-merge push checks |
| --- | --- | --- | --- |
| T101 | `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` | `209c093627c8d2a0892e01cb4250b0bdfde56406` | quality #1155; windows-terminal #768 |
| T102 | `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13` | `9e251ef9657176e8d52538b66f0ad633f7adc4f6` | quality #1157; windows-terminal #770 |
| T103 | `7490aa2eba9472c767c77de6799d7be0c7bc672f` | `115567e6a03495a4309465da24b7b725bf5bcac7` | quality #1159; windows-terminal #772 |
| T104 | `fdb222050678ccaf443838ae236dacf7294ac075` | `45ea9aa56bfd1bd1a43322e997a9d96687851c88` | quality #1162; windows-terminal #775 |
| T105 | `4544bf8c47f13bb15d586bf3f6be91dc881b6333` | `203a5dfd3a0c72401c7c0e56a96078ab6788e320` | quality #1164; windows-terminal #777 |
| T106 | `1828a9d8c48144a1ed9cd519cbd9527b981f1457` | `27e5cb40c137c5e5fea1bd4fa02730dc293f0e26` | quality #1171; windows-terminal #782 |
| T107 | `987cbed5f01f8890ece61cbe39c45e487ade9a66` | `b1dbf025ae6dd217f19fa66b05df4c20f47872b3` | quality #1175; windows-terminal #786 |
| T108 | `5c8f5259c53aedb3f12fc30f237887949740147a` | `5f447cfb8d8de6c4685808949b9de95318fac391` | quality #1177; windows-terminal #788 |
| T109 | `d13e2931cfb39cbc0f48b8602904873c47cd350a` | `51766359c9493fba1ff0f9786f340720ebc2d103` | quality #1186; windows-terminal #793 |
| T110 | `7f17473a3d24c902677932c0f770831178b44f08` | `44c265d91fb7c9b0e205d5a6812d99deaa794de2` | quality #1188 |
| T111 | `0a247394fa4e060cd7fbef31af2e2b530b1bbf41` | `8b3a7e7ff1b146e2358759cedfb2806758ac7d7f` | quality #1190; windows-terminal #795 |
| T112 | `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` | `4aac451c4e36e04341223bdf5d2bbf40bca506ef` | quality #1195; windows-terminal #800 |

T106 has an unrelated canonical first-parent cleanup commit `5d861eabde7e7a97cc1e57aaf0c6acf900d25483` immediately before its guarded merge; T106 closure identity remains the merge above. Interleaved Spec 007 T097 governance/CI commits before T109 are not relabelled as Spec 008 evidence.

## T112 final adversarial closure evidence

- Final T112 candidate: `305925064e3bb34d1af29b5f14fdd05a0f43fca2`, tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`.
- First-attempt exact-head candidate workflows `quality #1194`, `windows-terminal #799`, `t097-performance #48`, and `release-candidate #818` all succeeded.
- Fresh exact-head independent review reported no material actionable findings remaining after the final append-only `REJECTED -> REVERTED -> ACCEPTED` lineage repair.
- Guarded merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` has the candidate tree, ordered parents prior canonical main then exact candidate, and a valid GitHub signature.
- Post-merge push checks `quality #1195` and `windows-terminal #800` succeeded on attempt 1.

## Historical material evidence - preserved, not rewritten

- Plan run `34395694562`, macOS job `102614732764`, remains the first material T057 bounded-cleanup failure. It is not classified as a flake and was not rerun into acceptance.
- T112 first focused development attempt remains `9 passed / 1 failed`; the durable trigger rejected malformed forged stage state earlier than the fixture expected.
- Prior-head local parallel full-unit attempt remains `463 passed / 1 failed / 5 ignored` at `git::process_scope::tests::short_owned_process_quiesces`; an untouched canonical-main control reproduced the inherited local parallel-execution failure (`451 passed / 1 failed / 5 ignored`). Neither result is used as exact-head qualification.
- The stale-satisfied-gate adversarial reproducer remains `11 passed / 1 failed` before the forward-only projection repair.
- The final reverted-history fixture preserves a development compile failure `E0382` before a fixture-only `clone()` repair.
- Rejected, reverted, stale, superseded, corrupt, and failed history remains material after later success.

## Schema, persistence, and dependency reconciliation

- Sole Spec 008 migration: immutable canonical `migrations/0010_resumable_workflow_ledger.sql`.
- Tables: `workflow_runs`, `workflow_stage_runs`, `workflow_artifact_baselines`, `workflow_actor_bindings`, `workflow_reconstruction_reports`, `workflow_decisions`.
- Indexes: `idx_workflow_runs_workstream_created`, `idx_workflow_stage_runs_workflow_stage`, `idx_workflow_stage_runs_workflow_state`, `idx_workflow_artifact_baselines_stage`, `idx_workflow_actor_bindings_stage`, `idx_workflow_decisions_workflow_time`, `idx_workflow_decisions_stage_time`.
- Triggers: `trg_workflow_runs_hierarchy_insert`, `trg_workflow_runs_identity_update`, `trg_workflow_runs_no_delete`, `trg_workflow_stage_runs_lineage_insert`, `trg_workflow_stage_runs_initial_state_insert`, `trg_workflow_stage_runs_transition_update`, `trg_workflow_stage_runs_identity_update`, `trg_workflow_stage_runs_no_delete`, `trg_workflow_artifact_baselines_no_update`, `trg_workflow_artifact_baselines_no_delete`, `trg_workflow_actor_bindings_identity_insert`, `trg_workflow_actor_bindings_no_update`, `trg_workflow_actor_bindings_no_delete`, `trg_workflow_reconstruction_reports_no_update`, `trg_workflow_reconstruction_reports_no_delete`, `trg_workflow_decisions_identity_insert`, `trg_workflow_decisions_no_update`, `trg_workflow_decisions_no_delete`.
- Spec 008 introduced no `Cargo.toml` or `Cargo.lock` diff and no new direct dependency.
- Existing `rusqlite`, `serde`, `serde_json`, `sha2`, and accepted Git/process/runtime/terminal modules were reused only in accepted roles.
- `winds.db` remains the only durable store. No second database, ORM, event-sourcing framework, queue, distributed lease/lock service, or generic DAG scheduler was added.

## Platform and live-runtime truth boundary

- Platform proof is not transferred between domains. Native Windows, real WSL2, Linux PTY, and macOS PTY claims remain limited to directly exercised lanes.
- Spec 006 live-runtime nonclaims remain unchanged: `T079_LIVE_PASS=NO`, `T080_LIVE_PASS=NO`, `T082_WORKER_LIVE_PASS=NO`, `REAL_CLAUDE_EXECUTION=NO`, `REAL_CODEX_WORKER_EXECUTION=NO`.
- FR-020/FR-026 are satisfied by deterministic fail-closed continuation semantics and reuse of accepted Spec 006 truth; no unproven real live-runtime resume success is claimed.
- T110 canonical decision remains `TUI_NOT_REQUIRED_FIRST_SLICE`; no TUI production implementation was introduced.

## FR-001..FR-078 reconciliation

| Requirement | Classification | Canonical evidence path / truthful boundary |
| --- | --- | --- |
| FR-001 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-002 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-003 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-004 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-005 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-006 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-007 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-008 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-009 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; merges `ec841b99b0c6adc881e28e307fc4cf32ba3a91ea` and `2b9e98a1f4678acd0e78e77875ff43f6b07b6f13`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-010 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-011 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-012 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-013 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-014 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-015 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-016 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-017 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t103_workflow_baseline_tests.rs`; canonical T103 merge `7490aa2eba9472c767c77de6799d7be0c7bc672f`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-018 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-019 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-020 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-021 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-022 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-023 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-024 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-025 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-026 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/agentic_runtime.rs`; `src/t104_workflow_continuation_tests.rs`; T104 merge `fdb222050678ccaf443838ae236dacf7294ac075`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 live-runtime nonclaims remain in `specs/008-resumable-workflow-decision-ledger/tasks.md` lines 34-44 |
| FR-027 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-028 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-029 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-030 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-031 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-032 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-033 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-034 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t105_workflow_retry_tests.rs`; T105 merge `4544bf8c47f13bb15d586bf3f6be91dc881b6333`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-035 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-036 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-037 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-038 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-039 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-040 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-041 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-042 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-043 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/store.rs`; `src/t106_workflow_decision_tests.rs`; T106 merge `1828a9d8c48144a1ed9cd519cbd9527b981f1457`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-044 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-045 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-046 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-047 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-048 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-049 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-050 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/workflow_projection.rs`; `src/t107_workflow_projection_tests.rs`; T107 merge `987cbed5f01f8890ece61cbe39c45e487ade9a66`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-051 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-052 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-053 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-054 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-055 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-056 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-057 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-058 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-059 | `PROVEN_DETERMINISTIC` | `migrations/0010_resumable_workflow_ledger.sql`; `src/store.rs`; `src/t102_workflow_store_tests.rs`; `src/t108_workflow_recovery_tests.rs`; T108 merge `5c8f5259c53aedb3f12fc30f237887949740147a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-060 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8; `Cargo.toml`; `Cargo.lock`; `migrations/0010_resumable_workflow_ledger.sql`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef` contains no second durable store or newly authorized subsystem |
| FR-061 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-062 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-063 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-064 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-065 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-066 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-067 | `PROVEN_DETERMINISTIC` | `src/workflow_projection.rs`; `src/workflow_cli.rs`; `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; T109 merge `d13e2931cfb39cbc0f48b8602904873c47cd350a`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| FR-068 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-069 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-070 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-071 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-072 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-073 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-074 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-075 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-076 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8, 19-21; `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef`; no forbidden expansion is present in the landed Spec 008 source inventory |
| FR-077 | `PROVEN_PLATFORM_BOUND` | `src/t111_workflow_regression_tests.rs`; `.github/workflows/quality.yml`; `.github/workflows/windows-terminal.yml`; T111 merge `0a247394fa4e060cd7fbef31af2e2b530b1bbf41` with post-merge quality #1190 and windows-terminal #795; T112 post-merge quality #1195 and windows-terminal #800 |
| FR-078 | `PROVEN_GOVERNANCE_BOUNDARY` | `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; `specs/008-resumable-workflow-decision-ledger/tasks.md` Standard Acceptance Gate and Global Rules; canonical T111/T112 merges `0a247394fa4e060cd7fbef31af2e2b530b1bbf41` / `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; final T113 scope remains documentation-only |

## SC-001..SC-020 reconciliation

| Criterion | Classification | Canonical evidence path |
| --- | --- | --- |
| SC-001 | `PROVEN_DETERMINISTIC` | `src/workflow.rs`; `src/t101_workflow_domain_tests.rs`; `src/t102_workflow_store_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-002 | `PROVEN_DETERMINISTIC` | `src/t101_workflow_domain_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-003 | `PROVEN_DETERMINISTIC` | `src/t103_workflow_baseline_tests.rs`; `src/store.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-004 | `PROVEN_DETERMINISTIC` | `src/t104_workflow_continuation_tests.rs`; `src/t111_workflow_regression_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1`; Spec 006 nonclaims in `specs/008-resumable-workflow-decision-ledger/tasks.md` |
| SC-005 | `PROVEN_DETERMINISTIC` | `src/t105_workflow_retry_tests.rs`; `src/workflow.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-006 | `PROVEN_DETERMINISTIC` | `src/t105_workflow_retry_tests.rs`; `src/store.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-007 | `PROVEN_DETERMINISTIC` | `src/t101_workflow_domain_tests.rs`; `src/t105_workflow_retry_tests.rs`; `src/t106_workflow_decision_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-008 | `PROVEN_DETERMINISTIC` | `src/t106_workflow_decision_tests.rs`; `src/store.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-009 | `PROVEN_DETERMINISTIC` | `src/t107_workflow_projection_tests.rs`; `src/workflow_projection.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-010 | `PROVEN_DETERMINISTIC` | `src/t104_workflow_continuation_tests.rs`; `src/t108_workflow_recovery_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-011 | `PROVEN_DETERMINISTIC` | `src/t108_workflow_recovery_tests.rs`; `src/store.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-012 | `PROVEN_DETERMINISTIC` | `src/t108_workflow_recovery_tests.rs`; `src/workflow.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-013 | `PROVEN_DETERMINISTIC` | `src/t107_workflow_projection_tests.rs`; `src/t109_workflow_cli_tests.rs`; `src/workflow_cli.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-014 | `PROVEN_DETERMINISTIC` | `src/t101_workflow_domain_tests.rs`; `src/t106_workflow_decision_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-015 | `PROVEN_DETERMINISTIC` | `src/t101_workflow_domain_tests.rs`; `src/t107_workflow_projection_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; canonical T112 merge `6e8ca96070ee2b56c231f6b4c1f101d5fa589ab1` |
| SC-016 | `PROVEN_PLATFORM_BOUND` | `src/t111_workflow_regression_tests.rs`; `.github/workflows/quality.yml`; `.github/workflows/windows-terminal.yml`; T111 post-merge quality #1190/windows-terminal #795 and T112 post-merge quality #1195/windows-terminal #800 |
| SC-017 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/tasks.md` Standard Acceptance Gate; PR #165 exact-head Author/Ponytail/independent-review trail; exact-head `quality` run is candidate-bound and must be requalified after any head movement |
| SC-018 | `PROVEN_GOVERNANCE_BOUNDARY` | `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`; `specs/008-resumable-workflow-decision-ledger/tasks.md` Global Rules 6-8; `Cargo.toml`; `Cargo.lock`; canonical T112 tree `4aac451c4e36e04341223bdf5d2bbf40bca506ef` |
| SC-019 | `PROVEN_PLATFORM_BOUND` | `src/t111_workflow_regression_tests.rs`; `.github/workflows/windows-terminal.yml`; T111 windows-terminal #795 and T112 windows-terminal #800; no cross-platform substitution claimed |
| SC-020 | `PROVEN_DETERMINISTIC` | `src/t105_workflow_retry_tests.rs`; `src/t106_workflow_decision_tests.rs`; `src/t108_workflow_recovery_tests.rs`; `src/t112_workflow_adversarial_tests.rs`; `specs/008-resumable-workflow-decision-ledger/plan-amendment-001-t057-fail-closed-cli-fixture.md` preserves Plan failure run `34395694562` |

## Final negative-scope reconciliation

- No daemon/persistent owner/server/socket/HTTP/SSE/WebSocket or new IPC/RPC/control protocol.
- No remote execution/mobile/team continuation or service control plane.
- No provider/model API, Model Mesh, automatic provider routing, credential brokerage, or new provider execution authority.
- No browser automation/profile/CDP runtime, Browser Twin, or screenshot verification authority.
- No MCP runtime, ACP dependency expansion, A2A, generic plugin/workflow framework, integration SDK, or marketplace architecture.
- No learning/training/RL, skill promotion, protected holdout execution, semantic/vector/RAG memory, or automatic policy mutation.
- No new direct dependency and no second durable store.
- No automatic candidate winner, merge, rebase, cherry-pick, push, PR creation, or landing behavior was introduced by Spec 008.
- Research sources remain design references only; no donor runtime/dependency admission is claimed.

## T113 reconciliation checklist and external landing gates

- [x] T101..T112 reconciled to exact canonical merge/tree/post-merge evidence.
- [x] FR-001..FR-078 individually classified.
- [x] SC-001..SC-020 individually classified.
- [x] Schema/migration/dependency/durable-store inventory reconciled.
- [x] Freshness, append-only lineage, finite retry, reconstruction loss, redaction completeness, corrupt-state preservation, and historical evidence semantics reconciled.
- [x] Platform claims limited to directly exercised domains; Spec 006 live-runtime nonclaims preserved.
- [x] Historical material failures remain inspectable, including Plan run `34395694562` and T112 development failures.
- [x] Negative scope proves no forbidden expansion or automatic landing path.
The following are external candidate/landing gates and intentionally are not self-certified inside this pre-landing artifact. They must be proven on the exact final T113 commit in the PR/merge trail: repository `quality`; Author correctness/safety/governance/evidence-integrity review; Ponytail/YAGNI review; fresh independent substantive review; zero unresolved material findings/threads; final main/base/head/tree/scope/ruleset/mergeability race reconciliation; expected-head guarded normal merge; merge identity/tree/ordered parents/signature verification; and every actually-triggered post-merge push check.

## Closure state machine

Before guarded T113 landing:

```text
T101..T112=CLOSED_CANONICAL
T113=CANDIDATE_CLOSEOUT
SPEC_008_FIRST_IMPLEMENTATION_PROGRAM=NOT_YET_CLOSED
```

Only after this exact candidate lands and post-merge verification succeeds:

```text
T101..T113=CLOSED_CANONICAL
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=CLOSED_CANONICAL
SPEC_008_TASKS=CLOSED_CANONICAL
SPEC_008_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

That closure authorizes no later Spec 008 phase and no daemon/IPC, remote/provider/browser orchestration, learning, generic plugin/workflow engine, new dependency, automatic winner, or automatic landing behavior.
