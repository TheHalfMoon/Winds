use crate::domain::workflow::projection::{
    AuthorityCeilingTruth, ExternalGateState, ExternalGateTruth, ResumeDisposition,
    WorkflowProjectionInput, project_resume_preview, project_workflow_status,
};
use crate::domain::workflow::{
    LatestStageTruth, StageLifecycleState, StageRunIdentity, StageTransitionAuthority,
    StageTransitionRequest, TruthSource, WorkflowRunIdentity, evaluate_stage_transition,
    workflow_completion_eligible,
};

fn projection_input(
    lifecycle_state: StageLifecycleState,
    lifecycle_source: TruthSource,
    lifecycle_authority: StageTransitionAuthority,
) -> WorkflowProjectionInput {
    WorkflowProjectionInput {
        expected_workspace_id: "workspace-1".to_owned(),
        expected_workstream_id: "workstream-1".to_owned(),
        workflow: WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap(),
        stage: StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap(),
        lifecycle_state,
        lifecycle_source,
        lifecycle_authority,
        actor: None,
        current_candidate: None,
        baselines: Vec::new(),
        decisions: Vec::new(),
        retry_failure: None,
        retry_outcome_reason: None,
        verification: ExternalGateTruth::unknown(),
        human_acceptance: ExternalGateTruth::unknown(),
        authority_ceiling: AuthorityCeilingTruth::Unknown,
        prospective_reconstruction: None,
        reassignment_proven: false,
    }
}

#[test]
fn t111_agent_reported_terminal_authority_remains_rejected() {
    for requested in [
        StageLifecycleState::Completed,
        StageLifecycleState::Cancelled,
    ] {
        let request = StageTransitionRequest::new(
            "forged-terminal",
            StageLifecycleState::Active,
            requested,
            TruthSource::AgentReported,
            StageTransitionAuthority::None,
        )
        .unwrap();
        assert!(evaluate_stage_transition(StageLifecycleState::Active, None, &request).is_err());
    }
}

#[test]
fn t111_external_gates_keep_verification_and_human_acceptance_sources_distinct() {
    let mut input = projection_input(
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::None,
    );
    input.verification =
        ExternalGateTruth::satisfied(TruthSource::AgentReported, "forged-verification").unwrap();
    assert!(project_workflow_status(&input).is_err());

    input.verification = ExternalGateTruth::unknown();
    input.human_acceptance =
        ExternalGateTruth::satisfied(TruthSource::WindsObserved, "forged-acceptance").unwrap();
    assert!(project_workflow_status(&input).is_err());
}

#[test]
fn t111_missing_actor_binding_never_becomes_resumed() {
    let input = projection_input(
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::None,
    );
    let preview = project_resume_preview(&input).unwrap();
    assert_eq!(preview.disposition, ResumeDisposition::Unavailable);
    assert_eq!(preview.continuation, None);
}

#[test]
fn t111_procedural_completion_does_not_imply_external_gate_success() {
    let latest = [LatestStageTruth {
        state: StageLifecycleState::Completed,
        source: TruthSource::WindsObserved,
        authority: StageTransitionAuthority::WindsPolicy,
    }];
    assert!(workflow_completion_eligible(&latest));

    let input = projection_input(
        StageLifecycleState::Completed,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    );
    let status = project_workflow_status(&input).unwrap();
    assert_eq!(status.verification.state, ExternalGateState::Unknown);
    assert_eq!(status.human_acceptance.state, ExternalGateState::Unknown);
}
