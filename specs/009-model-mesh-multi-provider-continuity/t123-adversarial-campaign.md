# T123 Adversarial Model Mesh Truth Campaign

Status: `CANDIDATE_EVIDENCE`

## Exact predecessor basis

```text
T122_MERGE=a569534488991b58a77149ceae1833467488175d
T122_TREE=02950426ad2bf3f56663a20e208e9bfd9e6a87b0
T122_DECISION=WORKBENCH_MODEL_MESH_NECESSARY_NO
T122_POST_MERGE_QUALITY=SUCCESS_ATTEMPT_1
MIGRATION_0011=IMMUTABLE
```

T123 attacks the deterministic Model Mesh truth model. It creates no provider execution, live-runtime proof, credential operation, routing authority, Workbench path, dependency, migration, or Git mutation behavior.

## Campaign coverage

| Attack | Exact deterministic evidence | Result |
| --- | --- | --- |
| Runtime/model prose forges provider/model identity | `t114_agent_report_cannot_satisfy_winds_observed_identity_requirement`; `t123_forged_identity_unicode_case_whitespace_and_oversize_never_false_match` | FAIL_CLOSED |
| Unicode/case/whitespace/oversize identity collision | `t114_inputs_are_bounded_and_reject_control_or_malformed_digest_content`; `t123_forged_identity_unicode_case_whitespace_and_oversize_never_false_match` | FAIL_CLOSED |
| Unspecified provider/model silently filled | `t114_unspecified_provider_and_model_stay_explicit_and_are_not_inferred_from_runtime`; `t115_runtime_adapter_never_infers_provider_or_model_from_codex_or_claude`; T123 unspecified-dimension assertion | ZERO_INFERENCE |
| Ambiguous/unavailable target with tempting alternate | `t114_unavailable_ambiguous_and_conflicting_identity_never_selects_a_winner`; `t123_ambiguous_unavailable_explicit_policy_and_authority_downgrade_never_fallback` | ZERO_FALLBACK |
| Generic approval, forged approval JSON, stale digest, wrong purpose/target/role/binding/session/stage/work scope | `t115_stale_digest_malformed_generic_and_wrong_schema_approvals_fail_closed`; `t116_target_request_rejects_policy_cross_scope_wrong_and_generic_approval`; `t123_authority_replay_forged_json_wrong_scope_and_stale_digest_fail_closed` | FAIL_CLOSED |
| Two bindings in same stage/role but different sessions | `t116_target_request_is_exact_approved_idempotent_append_only_and_restart_safe`; `t123_request_claim_replay_role_and_session_scope_preserve_append_only_history` | DISTINCT_DIGESTS |
| Same binding/session/target with role-only mismatch | `t114_every_material_target_scope_dimension_moves_the_digest`; `t123_request_claim_replay_role_and_session_scope_preserve_append_only_history` | DISTINCT_DIGESTS |
| Current authority ceiling reduced after approval | `t117_approval_and_current_authority_remain_separate_fail_closed_bases`; `t119_no_authority_observation_is_bounded_and_never_overrides_current_authority`; `t123_ambiguous_unavailable_explicit_policy_and_authority_downgrade_never_fallback` | AUTHORITY_DENIED |
| Replay/duplicate target requests and identity claims | `t116_target_request_is_exact_approved_idempotent_append_only_and_restart_safe`; `t116_identity_claims_are_subject_bound_runtime_bound_idempotent_and_append_only`; T123 replay test | IDEMPOTENT_OR_COLLISION |
| Replay/duplicate continuity events and role associations | `t119_required_event_is_exact_atomic_idempotent_semantic_replay_safe_and_restart_safe`; `t119_wrong_role_claim_and_different_context_permission_fail_closed` | IDEMPOTENT_OR_REJECTED |
| Provider/model/runtime/native-session/stage/candidate/artifact/authority drift | T117 complete drift suite | STALE_CONFLICT_DENIED_AS_APPLICABLE |
| Forged native resume after restart/ownership loss | `t117_native_session_identity_never_upgrades_unproven_ownership`; `t118_same_runtime_native_id_coincidence_and_resumed_label_remain_unproven`; `t118_context_rejects_false_native_resume_wrong_direction_and_wrong_approval`; T123 native-resume assertion | ZERO_FALSE_NATIVE_RESUME |
| Missing/redacted/incomplete context and provider-private memory | `t118_missing_redacted_and_material_loss_context_are_explicit_not_filled_from_prose`; `t118_provider_private_state_is_explicitly_unavailable_without_lowering_complete_context`; T123 context test | EXPLICIT_LOSS |
| Secret-like durable payload attempts | `t118_reference_input_rejects_secret_private_and_persuasive_prose`; T123 secret/reference/observation-basis assertions | REJECTED |
| Malformed/unknown enum/schema/partial/orphan/corrupt state | `t116_schema_inventory_is_exact_idempotent_and_partial_state_fails_closed`; T121 corruption/lineage/sidecar tests; `t123_corrupt_selector_fails_closed_and_is_not_destructively_recovered` | PRESERVED_FAIL_CLOSED |
| Source-agent PASS/VERIFIED/ACCEPTED/cost/winner/authority/continuity labels | `t120_target_projection_preserves_scope_sources_and_separate_outcomes`; `t120_reviewer_projection_is_freshness_sensitive_and_contains_no_winner_or_persuasion_surface`; `t123_source_agent_pass_verified_accepted_cost_winner_labels_never_promote_outcomes` | ZERO_PROMOTION |
| Later target/continuity success erases earlier failure/unavailable history | `t117_current_target_projection_never_uses_latest_wins_and_preserves_history`; `t119_unproven_history_remains_after_later_authorized_continuity`; T123 append-only request history | ZERO_REWRITE |

## New T123 focused tests

```text
t123_forged_identity_unicode_case_whitespace_and_oversize_never_false_match
t123_authority_replay_forged_json_wrong_scope_and_stale_digest_fail_closed
t123_ambiguous_unavailable_explicit_policy_and_authority_downgrade_never_fallback
t123_secret_private_context_and_forged_native_resume_are_rejected
t123_source_agent_pass_verified_accepted_cost_winner_labels_never_promote_outcomes
t123_request_claim_replay_role_and_session_scope_preserve_append_only_history
t123_corrupt_selector_fails_closed_and_is_not_destructively_recovered
```

The focused campaign passed locally before candidate publication. Full exact-candidate repository qualification and independent review remain required before this document may become canonical closure evidence.

## Local inherited terminal-regression control

The first local all-target regression attempt on the T123 working candidate did **not** pass and remains material evidence. It reported six Spec 007 T090 terminal lifecycle failures, all because the owned child exit could not be proven inside the bounded cleanup window. No T123/Model Mesh test failed.

A separate detached control worktree at the unchanged canonical T122 merge `a569534488991b58a77149ceae1833467488175d` was then exercised with the same `cargo test --locked --all-targets --all-features` command. The control failed seven T090 tests with the same bounded-cleanup-window failure class. This proves the observed local failure class is inherited/environmental rather than introduced by the T123 diff; it does **not** turn either failed run into success.

```text
T123_LOCAL_FULL_REGRESSION_ATTEMPT_1=FAILURE_6_T090_BOUNDED_CLEANUP
T122_CANONICAL_CONTROL_LOCAL_REGRESSION=FAILURE_7_T090_BOUNDED_CLEANUP
T123_FOCUSED_TESTS=7_PASS
LOCAL_FAILURE_RELABELED_GREEN=NO
FULL_REGRESSION_ACCEPTANCE=PENDING_EXACT_CANDIDATE_CI
```

The repository's canonical predecessor evidence remains the successful exact-head and post-merge GitHub quality runs already accepted for T122. T123 closure still requires fresh GitHub `quality` success on the exact T123 candidate; this local control comparison is diagnostic evidence only.

## Nonclaims and boundaries

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
T123_PROVIDER_EXECUTION=NOT_PERFORMED
T123_CREDENTIAL_OPERATION=NOT_PERFORMED
T123_NEW_DEPENDENCY=NO
T123_MIGRATION_CHANGE=NO
T123_RUNTIME_CHANGE=NO
T123_WORKBENCH_CHANGE=NO
T123_GIT_MUTATION_AUTHORITY=NO
```

No test fixture success is real provider/model execution, authentication readiness, physical native resume, verification, human acceptance, landing, or broader authority.

## Candidate closure rule

T123 may close only after the exact final candidate passes repository quality, full regression, correctness/safety/governance/evidence-integrity review, Ponytail/YAGNI review, fresh independent substantive review, zero unresolved material findings, guarded normal merge, and post-merge verification.

Until that landing completes:

```text
T123=CANDIDATE_ONLY
T124=BLOCKED_BY_DEPENDENCY
```
