use super::{
    ActorBindingProjectionInput, ActorRoleTruth, AuthorityCeilingTruth, BlockerTruth,
    DecisionProjectionInput, ExternalGateState, ExternalGateTruth, RequiredAction,
    ResumeDisposition, WorkflowProjectionInput, project_resume_preview, project_reviewer_handoff,
    project_why_blocked, project_workflow_status,
};
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineKind, ArtifactBaselineRequirement,
    CandidateBaselineIdentity, DecisionApplicability, DecisionContentState,
    RETRY_OUTCOME_BUDGET_EXHAUSTED, RETRY_OUTCOME_FAILURE_RECORDED, ReconstructionCategory,
    ReconstructionContentState, ReconstructionItem, ReconstructionSourceClass,
    ReconstructionTransferState, RetryFailureObservation, SideEffectTruth, StageLifecycleState,
    StageRunIdentity, StageTransitionAuthority, TruthSource, WorkflowContinuationClass,
    WorkflowDecisionInput, WorkflowDecisionRecord, WorkflowRunIdentity,
    build_reconstruction_preview, evaluate_artifact_baseline_requirement,
};

const OID_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE_A: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OID_B: &str = "cccccccccccccccccccccccccccccccccccccccc";
const TREE_B: &str = "dddddddddddddddddddddddddddddddddddddddd";

fn candidate(oid: &str, tree: &str) -> CandidateBaselineIdentity {
    CandidateBaselineIdentity::new(oid, tree).unwrap()
}

fn reconstruction_report(
    binding_id: &str,
    stage_run_id: &str,
) -> crate::domain::workflow::ReconstructionReport {
    let items = vec![
        ReconstructionItem::new(
            ReconstructionCategory::CanonicalWorkContext,
            ReconstructionSourceClass::StoredCanonicalReference,
            "work-context:workspace-a/workstream-a",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::ObjectiveConstraints,
            ReconstructionSourceClass::StoredCanonicalReference,
            "objective-constraints:stored",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::Decisions,
            ReconstructionSourceClass::StoredCanonicalReference,
            "decisions:ledger",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::CandidateEvidence,
            ReconstructionSourceClass::StoredCanonicalReference,
            "candidate-evidence:partial",
            ReconstructionTransferState::Reconstructed,
            ReconstructionContentState::Redacted,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::PriorStageOutputs,
            ReconstructionSourceClass::DerivedReconstruction,
            "prior-stage-output:omitted",
            ReconstructionTransferState::Omitted,
            ReconstructionContentState::Omitted,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::RuntimeNativeContext,
            ReconstructionSourceClass::Unavailable,
            "runtime-native-context:unavailable",
            ReconstructionTransferState::Unavailable,
            ReconstructionContentState::Unavailable,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::ProviderPrivateState,
            ReconstructionSourceClass::Unavailable,
            "provider-private-state:no-longer-transferable",
            ReconstructionTransferState::NoLongerTransferable,
            ReconstructionContentState::Unavailable,
        )
        .unwrap(),
    ];
    build_reconstruction_preview(binding_id, stage_run_id, &items)
        .unwrap()
        .report
}

fn decision(
    decision_id: &str,
    created_unix_ms: i64,
    current: bool,
    rationale: Option<&str>,
) -> DecisionProjectionInput {
    DecisionProjectionInput {
        record: WorkflowDecisionRecord::new(WorkflowDecisionInput {
            decision_id,
            workflow_run_id: "workflow-a",
            stage_run_id: Some("stage-a"),
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::None,
            decision_type: "REVIEW_RECOMMENDATION",
            decision_result: "RECORDED",
            predecessor_decision_id: None,
            candidate: Some(if current {
                candidate(OID_A, TREE_A)
            } else {
                candidate(OID_B, TREE_B)
            }),
            evidence_reference: Some(if current {
                "evidence-current"
            } else {
                "evidence-stale"
            }),
            content_state: DecisionContentState::Full,
            safe_rationale: rationale,
            created_unix_ms,
        })
        .unwrap(),
        applicability: if current {
            DecisionApplicability::Applicable
        } else {
            DecisionApplicability::Stale
        },
    }
}

fn baseline(current: bool) -> crate::domain::workflow::BaselineEvaluation {
    let expected = candidate(OID_A, TREE_A);
    let observed_candidate = if current {
        expected.clone()
    } else {
        candidate(OID_B, TREE_B)
    };
    let requirement = ArtifactBaselineRequirement::new(
        "stage-a",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:current",
        Some(expected),
    )
    .unwrap();
    let observed = ArtifactBaselineIdentity::new(
        if current {
            "baseline-current"
        } else {
            "baseline-stale"
        },
        "stage-a",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:current",
        Some(observed_candidate),
    )
    .unwrap();
    evaluate_artifact_baseline_requirement(&requirement, &[observed])
}

fn base_input() -> WorkflowProjectionInput {
    WorkflowProjectionInput {
        expected_workspace_id: "workspace-a".to_owned(),
        expected_workstream_id: "workstream-a".to_owned(),
        workflow: WorkflowRunIdentity::new("workflow-a", "workspace-a", "workstream-a").unwrap(),
        stage: StageRunIdentity::new("stage-a", "workflow-a", "implement", 1, None).unwrap(),
        lifecycle_state: StageLifecycleState::Active,
        lifecycle_source: TruthSource::WindsObserved,
        lifecycle_authority: StageTransitionAuthority::None,
        actor: Some(ActorBindingProjectionInput {
            binding_id: "binding-a".to_owned(),
            stage_run_id: "stage-a".to_owned(),
            winds_session_id: "session-a".to_owned(),
            runtime_binding_id: Some("runtime-a".to_owned()),
            continuation: WorkflowContinuationClass::Unproven,
            role: ActorRoleTruth::Known {
                role: "Worker".to_owned(),
                source: TruthSource::WindsObserved,
            },
            reconstruction_report: None,
        }),
        current_candidate: Some(candidate(OID_A, TREE_A)),
        baselines: vec![baseline(true), baseline(false)],
        decisions: vec![
            decision(
                "decision-z",
                12,
                false,
                Some("SHIP IT WITH HIGH CONFIDENCE"),
            ),
            decision("decision-a", 11, true, None),
        ],
        retry_failure: None,
        retry_outcome_reason: None,
        verification: ExternalGateTruth::pending(Some(TruthSource::WindsObserved)),
        human_acceptance: ExternalGateTruth::unknown(),
        authority_ceiling: AuthorityCeilingTruth::Known {
            source: TruthSource::WindsObserved,
            authority: StageTransitionAuthority::WindsPolicy,
            reference: Some("policy:stage-transition".to_owned()),
        },
        prospective_reconstruction: None,
        reassignment_proven: false,
    }
}

#[test]
fn t107_status_projection_is_deterministic_exact_source_labelled_and_freshness_aware() {
    let input = base_input();
    let mut reordered = input.clone();
    reordered.baselines.reverse();
    reordered.decisions.reverse();

    let first = project_workflow_status(&input).unwrap();
    let second = project_workflow_status(&reordered).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.workflow.workflow_run_id, "workflow-a");
    assert_eq!(first.workflow.workspace_id, "workspace-a");
    assert_eq!(first.workflow.workstream_id, "workstream-a");
    assert_eq!(first.stage.stage_run_id, "stage-a");
    assert_eq!(first.stage.attempt_ordinal, 1);
    assert_eq!(first.lifecycle_state, StageLifecycleState::Active);
    assert_eq!(first.lifecycle_source, TruthSource::WindsObserved);
    assert_eq!(first.lifecycle_authority, StageTransitionAuthority::None);
    assert_eq!(first.decisions[0].decision_id, "decision-a");
    assert_eq!(
        first.decisions[0].applicability,
        DecisionApplicability::Applicable
    );
    assert_eq!(
        first.decisions[1].applicability,
        DecisionApplicability::Stale
    );
    assert!(
        first
            .baselines
            .iter()
            .any(|item| { item.freshness == crate::domain::workflow::BaselineFreshness::Stale })
    );
    assert_eq!(first.verification.state, ExternalGateState::Pending);
    assert_eq!(first.human_acceptance.state, ExternalGateState::Unknown);
}

#[test]
fn t107_every_lifecycle_state_projects_without_aliasing_identity() {
    for lifecycle_state in StageLifecycleState::ALL {
        let mut input = base_input();
        input.lifecycle_state = lifecycle_state;
        if lifecycle_state == StageLifecycleState::Failed {
            input.retry_failure = Some(
                RetryFailureObservation::new(
                    "CHECK_FAILED",
                    Some("checkpoint-1"),
                    "candidate-and-checkpoint-unchanged",
                    SideEffectTruth::SafeOrIdempotent,
                )
                .unwrap(),
            );
            input.retry_outcome_reason = Some(RETRY_OUTCOME_FAILURE_RECORDED.to_owned());
            input.lifecycle_authority = StageTransitionAuthority::WindsPolicy;
        }
        if lifecycle_state == StageLifecycleState::RecoveryRequired {
            input.retry_failure = Some(
                RetryFailureObservation::new(
                    "AMBIGUOUS_EFFECT",
                    None,
                    "effect-completion-unknown",
                    SideEffectTruth::AmbiguousNonIdempotent,
                )
                .unwrap(),
            );
            input.retry_outcome_reason =
                Some(crate::domain::workflow::RETRY_OUTCOME_AMBIGUOUS_EFFECT.to_owned());
            input.lifecycle_authority = StageTransitionAuthority::WindsPolicy;
        }
        let projected = project_workflow_status(&input).unwrap();
        assert_eq!(projected.lifecycle_state, lifecycle_state);
        assert_eq!(projected.workflow.workflow_run_id, "workflow-a");
        assert_eq!(projected.stage.stage_run_id, "stage-a");
    }
}

#[test]
fn t107_canonical_work_context_and_cross_stage_material_fail_closed() {
    let mut mismatched_workspace = base_input();
    mismatched_workspace.expected_workspace_id = "workspace-other".to_owned();
    assert!(project_workflow_status(&mismatched_workspace).is_err());

    let mut wrong_stage_parent = base_input();
    wrong_stage_parent.stage =
        StageRunIdentity::new("stage-a", "workflow-other", "implement", 1, None).unwrap();
    assert!(project_workflow_status(&wrong_stage_parent).is_err());

    let mut cross_stage_decision = base_input();
    cross_stage_decision.decisions[0].record.stage_run_id = Some("stage-other".to_owned());
    assert!(project_reviewer_handoff(&cross_stage_decision).is_err());

    let mut cross_stage_baseline = base_input();
    cross_stage_baseline.baselines[0].requirement.stage_run_id = "stage-other".to_owned();
    assert!(project_workflow_status(&cross_stage_baseline).is_err());
}

#[test]
fn t107_resume_preview_distinguishes_proven_reconstruction_reassignment_unavailable_and_loss() {
    let mut exact = base_input();
    exact.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::Resumed;
    assert_eq!(
        project_resume_preview(&exact).unwrap().disposition,
        ResumeDisposition::ExactNativeResume
    );

    let mut unproven = base_input();
    unproven.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::Unproven;
    assert_eq!(
        project_resume_preview(&unproven).unwrap().disposition,
        ResumeDisposition::Unavailable
    );

    let mut reconstruct = base_input();
    reconstruct.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::Unavailable;
    reconstruct.prospective_reconstruction =
        Some(reconstruction_report("prospective-binding", "stage-a"));
    let preview = project_resume_preview(&reconstruct).unwrap();
    assert_eq!(preview.disposition, ResumeDisposition::WindsReconstruction);
    assert!(preview.reconstruction.is_some());

    let mut reassignment = base_input();
    reassignment.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::Unavailable;
    reassignment.reassignment_proven = true;
    assert_eq!(
        project_resume_preview(&reassignment).unwrap().disposition,
        ResumeDisposition::Reassignment
    );

    let mut lost = base_input();
    lost.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::OwnershipLost;
    assert_eq!(
        project_resume_preview(&lost).unwrap().disposition,
        ResumeDisposition::OwnershipLostHandling
    );

    let mut blocked = base_input();
    blocked.lifecycle_state = StageLifecycleState::WaitingApproval;
    assert_eq!(
        project_resume_preview(&blocked).unwrap().disposition,
        ResumeDisposition::Blocked
    );
}

#[test]
fn t107_why_blocked_uses_proven_state_and_never_invents_generic_blocker() {
    let mut approval = base_input();
    approval.lifecycle_state = StageLifecycleState::WaitingApproval;
    let projected = project_why_blocked(&approval).unwrap();
    assert_eq!(projected.blocker, BlockerTruth::WaitingApproval);
    assert_eq!(projected.required_action, RequiredAction::ExplicitApproval);

    let mut external = base_input();
    external.lifecycle_state = StageLifecycleState::WaitingExternal;
    let projected = project_why_blocked(&external).unwrap();
    assert_eq!(projected.blocker, BlockerTruth::WaitingExternal);
    assert_eq!(projected.required_action, RequiredAction::ExternalCondition);

    let mut failed = base_input();
    failed.lifecycle_state = StageLifecycleState::Failed;
    failed.lifecycle_authority = StageTransitionAuthority::WindsPolicy;
    failed.retry_failure = Some(
        RetryFailureObservation::new(
            "CHECK_FAILED",
            None,
            "same-failure-basis",
            SideEffectTruth::SafeOrIdempotent,
        )
        .unwrap(),
    );
    failed.retry_outcome_reason = Some(RETRY_OUTCOME_FAILURE_RECORDED.to_owned());
    let projected = project_why_blocked(&failed).unwrap();
    assert_eq!(projected.blocker, BlockerTruth::RetryRequired);
    assert_eq!(projected.required_action, RequiredAction::ExplicitRetry);

    failed.retry_outcome_reason = Some(RETRY_OUTCOME_BUDGET_EXHAUSTED.to_owned());
    let projected = project_why_blocked(&failed).unwrap();
    assert_eq!(projected.blocker, BlockerTruth::RecoveryRequired);
    assert_eq!(projected.required_action, RequiredAction::Recovery);

    let mut generic = base_input();
    generic.lifecycle_state = StageLifecycleState::Blocked;
    let projected = project_why_blocked(&generic).unwrap();
    assert_eq!(projected.blocker, BlockerTruth::Unknown);
    assert_eq!(projected.required_action, RequiredAction::Unknown);
    assert_eq!(projected.source, None);
    assert_eq!(projected.detail_reference, None);
}

#[test]
fn t107_reviewer_handoff_excludes_persuasion_and_keeps_exact_authority_context() {
    let input = base_input();
    let handoff = project_reviewer_handoff(&input).unwrap();
    assert_eq!(handoff.workflow.workflow_run_id, "workflow-a");
    assert_eq!(handoff.stage.stage_run_id, "stage-a");
    assert_eq!(handoff.current_candidate, Some(candidate(OID_A, TREE_A)));
    assert_eq!(handoff.authority_ceiling, input.authority_ceiling);
    assert_eq!(handoff.decisions.len(), 2);
    let rendered = format!("{handoff:?}");
    assert!(!rendered.contains("SHIP IT WITH HIGH CONFIDENCE"));
    assert!(!rendered.to_ascii_lowercase().contains("confidence"));
    assert!(!rendered.to_ascii_lowercase().contains("proposed verdict"));
}

#[test]
fn t107_reconstructed_actor_surfaces_omitted_unavailable_and_redacted_context() {
    let mut input = base_input();
    let report = reconstruction_report("binding-a", "stage-a");
    let actor = input.actor.as_mut().unwrap();
    actor.continuation = WorkflowContinuationClass::Reconstructed;
    actor.runtime_binding_id = None;
    actor.reconstruction_report = Some(report);
    let status = project_workflow_status(&input).unwrap();
    let reconstruction = status.actor.unwrap().reconstruction.unwrap();
    assert_eq!(
        reconstruction.items.len(),
        ReconstructionCategory::ALL.len()
    );
    assert!(reconstruction.items.iter().any(|item| {
        item.content_state == ReconstructionContentState::Omitted
            && item.transfer_state == ReconstructionTransferState::Omitted
    }));
    assert!(reconstruction.items.iter().any(|item| {
        item.content_state == ReconstructionContentState::Unavailable
            && matches!(
                item.transfer_state,
                ReconstructionTransferState::Unavailable
                    | ReconstructionTransferState::NoLongerTransferable
            )
    }));
    assert!(
        reconstruction
            .items
            .iter()
            .any(|item| { item.content_state == ReconstructionContentState::Redacted })
    );
}

#[test]
fn t107_agent_material_cannot_be_projected_as_verified_or_human_accepted() {
    let mut agent_verified = base_input();
    agent_verified.verification =
        ExternalGateTruth::satisfied(TruthSource::AgentReported, "agent-pass").unwrap();
    assert!(project_workflow_status(&agent_verified).is_err());

    let mut winds_human = base_input();
    winds_human.human_acceptance =
        ExternalGateTruth::satisfied(TruthSource::WindsObserved, "winds-accepted").unwrap();
    assert!(project_reviewer_handoff(&winds_human).is_err());

    let mut valid = base_input();
    valid.verification =
        ExternalGateTruth::satisfied(TruthSource::WindsObserved, "verify-run-1").unwrap();
    valid.human_acceptance =
        ExternalGateTruth::satisfied(TruthSource::HumanDecided, "human-decision-1").unwrap();
    let status = project_workflow_status(&valid).unwrap();
    assert_eq!(status.verification.state, ExternalGateState::Satisfied);
    assert_eq!(status.human_acceptance.state, ExternalGateState::Satisfied);
}

#[test]
fn t107_terminal_stage_resume_is_unavailable_without_inventing_a_blocker() {
    for lifecycle_state in [
        StageLifecycleState::Cancelled,
        StageLifecycleState::Completed,
    ] {
        let mut input = base_input();
        input.lifecycle_state = lifecycle_state;
        input.lifecycle_source = TruthSource::WindsObserved;
        input.lifecycle_authority = StageTransitionAuthority::WindsPolicy;
        let preview = project_resume_preview(&input).unwrap();
        assert_eq!(preview.disposition, ResumeDisposition::Unavailable);
        let blocked = project_why_blocked(&input).unwrap();
        assert_eq!(blocked.blocker, BlockerTruth::None);
        assert_eq!(blocked.required_action, RequiredAction::None);
    }
}

#[test]
fn t107_reconstruction_identity_mismatch_fails_closed() {
    let mut input = base_input();
    let actor = input.actor.as_mut().unwrap();
    actor.continuation = WorkflowContinuationClass::Reconstructed;
    actor.runtime_binding_id = None;
    actor.reconstruction_report = Some(reconstruction_report("binding-other", "stage-a"));
    assert!(project_workflow_status(&input).is_err());

    let mut prospective = base_input();
    prospective.actor.as_mut().unwrap().continuation = WorkflowContinuationClass::Unavailable;
    prospective.prospective_reconstruction =
        Some(reconstruction_report("future-binding", "stage-other"));
    assert!(project_resume_preview(&prospective).is_err());
}

#[test]
fn t107_lifecycle_and_authority_ceiling_share_canonical_source_authority_rules() {
    let mut invalid_lifecycle = base_input();
    invalid_lifecycle.lifecycle_source = TruthSource::HumanDecided;
    invalid_lifecycle.lifecycle_authority = StageTransitionAuthority::WindsPolicy;
    assert!(project_workflow_status(&invalid_lifecycle).is_err());
    assert!(project_reviewer_handoff(&invalid_lifecycle).is_err());

    let mut invalid_ceiling = base_input();
    invalid_ceiling.authority_ceiling = AuthorityCeilingTruth::Known {
        source: TruthSource::HumanDecided,
        authority: StageTransitionAuthority::WindsPolicy,
        reference: Some("policy:invalid-human-pair".to_owned()),
    };
    assert!(project_workflow_status(&invalid_ceiling).is_err());
    assert!(project_reviewer_handoff(&invalid_ceiling).is_err());

    let mut valid_human = base_input();
    valid_human.lifecycle_source = TruthSource::HumanDecided;
    valid_human.lifecycle_authority = StageTransitionAuthority::HumanDecision;
    valid_human.authority_ceiling = AuthorityCeilingTruth::Known {
        source: TruthSource::HumanDecided,
        authority: StageTransitionAuthority::HumanDecision,
        reference: Some("human-decision:stage-authority".to_owned()),
    };
    assert!(project_workflow_status(&valid_human).is_ok());
    assert!(project_reviewer_handoff(&valid_human).is_ok());
}

#[test]
fn t107_projection_calls_are_side_effect_free_over_their_inputs() {
    let input = base_input();
    let before = input.clone();
    let _ = project_workflow_status(&input).unwrap();
    let _ = project_resume_preview(&input).unwrap();
    let _ = project_why_blocked(&input).unwrap();
    let _ = project_reviewer_handoff(&input).unwrap();
    assert_eq!(input, before);
}
