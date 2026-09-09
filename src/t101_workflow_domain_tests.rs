use crate::workflow::{
    AppliedStageTransition, LatestStageTruth, StageAttemptRelation, StageLifecycleState,
    StageRunIdentity, StageTransitionAuthority, StageTransitionOutcome, StageTransitionRequest,
    TruthSource, WorkflowRunIdentity, evaluate_stage_transition, is_legal_stage_transition,
    validate_successor_attempt, workflow_completion_eligible,
};

fn stage(id: &str, ordinal: u32, predecessor: Option<&str>) -> StageRunIdentity {
    StageRunIdentity::new(id, "workflow-1", "build", ordinal, predecessor)
        .expect("valid stage identity")
}

#[test]
fn t101_workflow_and_stage_identity_are_canonical_not_display_or_runtime_identity() {
    let workflow = WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1")
        .expect("valid workflow identity");
    assert_eq!(workflow.workflow_run_id, "workflow-1");
    assert_eq!(workflow.workspace_id, "workspace-1");
    assert_eq!(workflow.workstream_id, "workstream-1");

    for invalid in ["", " ", "\t\n"] {
        assert!(WorkflowRunIdentity::new(invalid, "workspace-1", "workstream-1").is_err());
        assert!(WorkflowRunIdentity::new("workflow-1", invalid, "workstream-1").is_err());
        assert!(WorkflowRunIdentity::new("workflow-1", "workspace-1", invalid).is_err());
        assert!(StageRunIdentity::new(invalid, "workflow-1", "build", 1, None).is_err());
        assert!(StageRunIdentity::new("stage-1", invalid, "build", 1, None).is_err());
        assert!(StageRunIdentity::new("stage-1", "workflow-1", invalid, 1, None).is_err());
    }
    assert!(StageRunIdentity::new("stage-1", "workflow-1", "build", 0, None).is_err());

    let same_label_different_identity = [
        WorkflowRunIdentity::new("workflow-a", "workspace-a", "workstream-a").unwrap(),
        WorkflowRunIdentity::new("workflow-b", "workspace-b", "workstream-b").unwrap(),
    ];
    assert_ne!(
        same_label_different_identity[0],
        same_label_different_identity[1]
    );
}

#[test]
fn t101_material_new_work_requires_exact_distinct_successor_attempt_identity() {
    let predecessor = stage("stage-1", 1, None);
    let successor = stage("stage-2", 2, Some("stage-1"));
    for relation in [
        StageAttemptRelation::Retry,
        StageAttemptRelation::Reconstruction,
        StageAttemptRelation::Reassignment,
        StageAttemptRelation::Recovery,
    ] {
        validate_successor_attempt(&predecessor, &successor, relation)
            .expect("valid successor attempt");
    }

    let reused_id = stage("stage-1", 2, Some("stage-1"));
    assert!(
        validate_successor_attempt(&predecessor, &reused_id, StageAttemptRelation::Retry).is_err()
    );
    let skipped_ordinal = stage("stage-3", 3, Some("stage-1"));
    assert!(
        validate_successor_attempt(&predecessor, &skipped_ordinal, StageAttemptRelation::Retry)
            .is_err()
    );
    let wrong_predecessor = stage("stage-2", 2, Some("stage-other"));
    assert!(
        validate_successor_attempt(
            &predecessor,
            &wrong_predecessor,
            StageAttemptRelation::Retry
        )
        .is_err()
    );

    let wrong_workflow =
        StageRunIdentity::new("stage-2", "workflow-2", "build", 2, Some("stage-1")).unwrap();
    assert!(
        validate_successor_attempt(&predecessor, &wrong_workflow, StageAttemptRelation::Retry)
            .is_err()
    );
    let wrong_stage =
        StageRunIdentity::new("stage-2", "workflow-1", "test", 2, Some("stage-1")).unwrap();
    assert!(
        validate_successor_attempt(&predecessor, &wrong_stage, StageAttemptRelation::Retry)
            .is_err()
    );
}

fn expected_legal(from: StageLifecycleState, to: StageLifecycleState) -> bool {
    use StageLifecycleState::{
        Active, Blocked, Cancelled, Completed, Failed, Prepared, RecoveryRequired, Stale,
        WaitingApproval, WaitingExternal,
    };
    matches!(
        (from, to),
        (Prepared, Active | Stale | Cancelled | RecoveryRequired)
            | (
                Active,
                WaitingApproval
                    | WaitingExternal
                    | Blocked
                    | Failed
                    | Stale
                    | Cancelled
                    | Completed
                    | RecoveryRequired
            )
            | (
                WaitingApproval,
                Active | Blocked | Failed | Stale | Cancelled | RecoveryRequired
            )
            | (
                WaitingExternal,
                Active | Blocked | Failed | Stale | Cancelled | RecoveryRequired
            )
            | (
                Blocked,
                Active | Failed | Stale | Cancelled | RecoveryRequired
            )
    )
}

#[test]
fn t101_transition_matrix_accepts_every_legal_pair_and_rejects_every_illegal_pair() {
    for from in StageLifecycleState::ALL {
        for to in StageLifecycleState::ALL {
            assert_eq!(
                is_legal_stage_transition(from, to),
                expected_legal(from, to),
                "transition mismatch for {from:?} -> {to:?}"
            );
        }
        assert_eq!(
            from.is_terminal(),
            matches!(
                from,
                StageLifecycleState::Failed
                    | StageLifecycleState::Stale
                    | StageLifecycleState::Cancelled
                    | StageLifecycleState::Completed
                    | StageLifecycleState::RecoveryRequired
            )
        );
    }
}

#[test]
fn t101_transition_evaluation_is_fail_closed_and_exactly_idempotent() {
    let request = StageTransitionRequest::new(
        "operation-1",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    let applied = match evaluate_stage_transition(StageLifecycleState::Prepared, None, &request)
        .expect("legal transition")
    {
        StageTransitionOutcome::Apply(applied) => applied,
        StageTransitionOutcome::IdempotentNoChange => {
            panic!("first application cannot be no-change")
        }
    };
    assert_eq!(applied.from, StageLifecycleState::Prepared);
    assert_eq!(applied.to, StageLifecycleState::Active);

    assert_eq!(
        evaluate_stage_transition(StageLifecycleState::Active, Some(&applied), &request).unwrap(),
        StageTransitionOutcome::IdempotentNoChange
    );

    let different_operation = StageTransitionRequest::new(
        "operation-2",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(
        evaluate_stage_transition(
            StageLifecycleState::Active,
            Some(&applied),
            &different_operation
        )
        .is_err()
    );

    let same_state = StageTransitionRequest::new(
        "operation-3",
        StageLifecycleState::Active,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(evaluate_stage_transition(StageLifecycleState::Active, None, &same_state).is_err());

    let terminal_resurrection = StageTransitionRequest::new(
        "operation-4",
        StageLifecycleState::Completed,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(
        evaluate_stage_transition(StageLifecycleState::Completed, None, &terminal_resurrection)
            .is_err()
    );
}

#[test]
fn t101_agent_reported_text_cannot_cancel_complete_or_satisfy_workflow_completion() {
    for requested in [
        StageLifecycleState::Cancelled,
        StageLifecycleState::Completed,
    ] {
        let request = StageTransitionRequest::new(
            "forged-agent-operation",
            StageLifecycleState::Active,
            requested,
            TruthSource::AgentReported,
            StageTransitionAuthority::None,
        )
        .unwrap();
        assert!(evaluate_stage_transition(StageLifecycleState::Active, None, &request).is_err());
    }

    for source in [TruthSource::WindsObserved, TruthSource::HumanDecided] {
        let request = StageTransitionRequest::new(
            "canonical-operation",
            StageLifecycleState::Active,
            StageLifecycleState::Completed,
            source,
            match source {
                TruthSource::WindsObserved => StageTransitionAuthority::WindsPolicy,
                TruthSource::HumanDecided => StageTransitionAuthority::HumanDecision,
                TruthSource::AgentReported => StageTransitionAuthority::None,
            },
        )
        .unwrap();
        assert!(matches!(
            evaluate_stage_transition(StageLifecycleState::Active, None, &request).unwrap(),
            StageTransitionOutcome::Apply(_)
        ));
    }

    assert!(!workflow_completion_eligible(&[LatestStageTruth {
        state: StageLifecycleState::Completed,
        source: TruthSource::AgentReported,
        authority: StageTransitionAuthority::None,
    }]));
}

#[test]
fn t101_workflow_completion_is_procedural_and_requires_every_latest_stage_resolved() {
    assert!(!workflow_completion_eligible(&[]));

    let canonical_complete = LatestStageTruth {
        state: StageLifecycleState::Completed,
        source: TruthSource::WindsObserved,
        authority: StageTransitionAuthority::WindsPolicy,
    };
    let authorized_cancel = LatestStageTruth {
        state: StageLifecycleState::Cancelled,
        source: TruthSource::HumanDecided,
        authority: StageTransitionAuthority::HumanDecision,
    };
    assert!(workflow_completion_eligible(&[
        canonical_complete,
        authorized_cancel
    ]));

    for unresolved in [
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        StageLifecycleState::WaitingApproval,
        StageLifecycleState::WaitingExternal,
        StageLifecycleState::Blocked,
        StageLifecycleState::Failed,
        StageLifecycleState::Stale,
        StageLifecycleState::RecoveryRequired,
    ] {
        assert!(!workflow_completion_eligible(&[
            canonical_complete,
            LatestStageTruth {
                state: unresolved,
                source: TruthSource::WindsObserved,
                authority: StageTransitionAuthority::WindsPolicy,
            },
        ]));
    }

    assert!(!workflow_completion_eligible(&[
        canonical_complete,
        LatestStageTruth {
            state: StageLifecycleState::Cancelled,
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::None,
        },
    ]));
}

#[test]
fn t101_operation_identity_must_be_non_empty() {
    for invalid in ["", " ", "\t"] {
        assert!(
            StageTransitionRequest::new(
                invalid,
                StageLifecycleState::Prepared,
                StageLifecycleState::Active,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .is_err()
        );
    }
}

#[test]
fn t101_exact_prior_operation_must_match_source_and_transition_shape() {
    let prior = AppliedStageTransition {
        operation_id: "operation-1".to_owned(),
        from: StageLifecycleState::Prepared,
        to: StageLifecycleState::Active,
        source: TruthSource::WindsObserved,
        authority: StageTransitionAuthority::WindsPolicy,
    };
    let mismatched_source = StageTransitionRequest::new(
        "operation-1",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::HumanDecided,
        StageTransitionAuthority::HumanDecision,
    )
    .unwrap();
    assert!(
        evaluate_stage_transition(
            StageLifecycleState::Active,
            Some(&prior),
            &mismatched_source
        )
        .is_err()
    );

    let mismatched_target = StageTransitionRequest::new(
        "operation-1",
        StageLifecycleState::Prepared,
        StageLifecycleState::Blocked,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(
        evaluate_stage_transition(
            StageLifecycleState::Active,
            Some(&prior),
            &mismatched_target
        )
        .is_err()
    );
}
