use super::{NewWorkspace, NewWorkstream, Store};
use crate::domain::workflow::{
    CandidateBaselineIdentity, DecisionAppendOutcome, DecisionApplicability,
    DecisionApplicabilityContext, DecisionContentState, StageRunIdentity, StageTransitionAuthority,
    TruthSource, WorkflowDecisionInput, WorkflowDecisionRecord, WorkflowRunIdentity,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t106-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: PathBuf) {
    if home.exists() {
        fs::remove_dir_all(home).unwrap();
    }
}
fn seeded_store(name: &str) -> (PathBuf, Store) {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t106-workspace-1",
                git_common_dir: "/tmp/t106-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T106 workstream",
            },
            2,
        )
        .unwrap();
    for workflow_id in ["workflow-1", "workflow-2"] {
        let workflow =
            WorkflowRunIdentity::new(workflow_id, "workspace-1", "workstream-1").unwrap();
        store.create_workflow_run(&workflow, 3).unwrap();
    }
    let stage_1 = StageRunIdentity::new("stage-1", "workflow-1", "review", 1, None).unwrap();
    let stage_2 = StageRunIdentity::new("stage-2", "workflow-2", "review", 1, None).unwrap();
    store.create_stage_run(&stage_1, None, 4).unwrap();
    store.create_stage_run(&stage_2, None, 4).unwrap();
    (home, store)
}

fn candidate(hex: char) -> CandidateBaselineIdentity {
    let oid = hex.to_string().repeat(40);
    let tree_hex = if hex == 'a' { 'b' } else { 'c' };
    CandidateBaselineIdentity::new(&oid, &tree_hex.to_string().repeat(40)).unwrap()
}

fn decision(
    decision_id: &str,
    workflow_run_id: &str,
    decision_result: &str,
    predecessor_decision_id: Option<&str>,
    candidate: Option<CandidateBaselineIdentity>,
    evidence_reference: Option<&str>,
    created_unix_ms: i64,
) -> WorkflowDecisionRecord {
    WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id,
        workflow_run_id,
        stage_run_id: match workflow_run_id {
            "workflow-1" => Some("stage-1"),
            "workflow-2" => Some("stage-2"),
            _ => None,
        },
        source: TruthSource::AgentReported,
        authority: StageTransitionAuthority::None,
        decision_type: "REVIEW_DECISION",
        decision_result,
        predecessor_decision_id,
        candidate,
        evidence_reference,
        content_state: DecisionContentState::Full,
        safe_rationale: Some("bounded rationale"),
        created_unix_ms,
    })
    .unwrap()
}

#[test]
fn t106_append_only_replay_is_idempotent_and_semantic_collision_is_rejected() {
    let (home, store) = seeded_store("replay");
    let first = decision(
        "decision-1",
        "workflow-1",
        "ACCEPTED",
        None,
        Some(candidate('a')),
        Some("evidence:verify-a"),
        10,
    );
    assert_eq!(
        store.append_workflow_decision(&first).unwrap(),
        DecisionAppendOutcome::Inserted
    );
    assert_eq!(
        store.append_workflow_decision(&first).unwrap(),
        DecisionAppendOutcome::IdempotentNoChange
    );
    let collision = decision(
        "decision-1",
        "workflow-1",
        "REJECTED",
        None,
        Some(candidate('a')),
        Some("evidence:verify-a"),
        10,
    );
    assert!(store.append_workflow_decision(&collision).is_err());
    assert_eq!(
        store.list_workflow_decisions("workflow-1").unwrap(),
        vec![first]
    );
    assert!(
        store.connection.execute(
            "UPDATE workflow_decisions SET decision_result = 'REWRITTEN' WHERE decision_id = 'decision-1'",
            [],
        ).is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM workflow_decisions WHERE decision_id = 'decision-1'",
                [],
            )
            .is_err()
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_successor_lineage_preserves_prior_history_and_rejects_impossible_relations() {
    let (home, store) = seeded_store("lineage");
    let first = decision(
        "decision-1",
        "workflow-1",
        "ACCEPTED",
        None,
        Some(candidate('a')),
        None,
        10,
    );
    store.append_workflow_decision(&first).unwrap();
    let reverted = decision(
        "decision-2",
        "workflow-1",
        "REVERTED",
        Some("decision-1"),
        Some(candidate('a')),
        None,
        11,
    );
    store.append_workflow_decision(&reverted).unwrap();
    assert_eq!(
        store.list_workflow_decisions("workflow-1").unwrap(),
        vec![first.clone(), reverted.clone()]
    );

    let wrong_workflow = decision(
        "decision-x",
        "workflow-2",
        "REJECTED",
        Some("decision-2"),
        Some(candidate('a')),
        None,
        12,
    );
    assert!(store.append_workflow_decision(&wrong_workflow).is_err());

    let wrong_type = WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: "decision-3",
        workflow_run_id: "workflow-1",
        stage_run_id: Some("stage-1"),
        source: TruthSource::AgentReported,
        authority: StageTransitionAuthority::None,
        decision_type: "DIFFERENT_DECISION",
        decision_result: "SUPERSEDED",
        predecessor_decision_id: Some("decision-2"),
        candidate: Some(candidate('a')),
        evidence_reference: None,
        content_state: DecisionContentState::Full,
        safe_rationale: Some("different semantic axis"),
        created_unix_ms: 12,
    })
    .unwrap();
    assert!(store.append_workflow_decision(&wrong_type).is_err());
    assert!(
        WorkflowDecisionRecord::new(WorkflowDecisionInput {
            decision_id: "self",
            workflow_run_id: "workflow-1",
            stage_run_id: Some("stage-1"),
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::None,
            decision_type: "REVIEW_DECISION",
            decision_result: "SUPERSEDED",
            predecessor_decision_id: Some("self"),
            candidate: None,
            evidence_reference: None,
            content_state: DecisionContentState::Full,
            safe_rationale: Some("invalid self lineage"),
            created_unix_ms: 13,
        })
        .is_err()
    );
    assert!(
        store.connection.execute(
            "UPDATE workflow_decisions SET predecessor_decision_id = 'decision-2' WHERE decision_id = 'decision-1'",
            [],
        ).is_err()
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_candidate_and_evidence_movement_marks_applicability_stale_without_rewriting_history() {
    let (home, store) = seeded_store("stale");
    let original = decision(
        "decision-1",
        "workflow-1",
        "ACCEPTED",
        None,
        Some(candidate('a')),
        Some("evidence:verify-a"),
        10,
    );
    store.append_workflow_decision(&original).unwrap();

    let applicable =
        DecisionApplicabilityContext::new(Some(candidate('a')), &["evidence:verify-a".to_owned()])
            .unwrap();
    assert_eq!(
        store
            .workflow_decision_applicability("decision-1", &applicable)
            .unwrap(),
        DecisionApplicability::Applicable
    );
    let moved_candidate =
        DecisionApplicabilityContext::new(Some(candidate('d')), &["evidence:verify-a".to_owned()])
            .unwrap();
    assert_eq!(
        store
            .workflow_decision_applicability("decision-1", &moved_candidate)
            .unwrap(),
        DecisionApplicability::Stale
    );
    let moved_evidence =
        DecisionApplicabilityContext::new(Some(candidate('a')), &["evidence:verify-b".to_owned()])
            .unwrap();
    assert_eq!(
        store
            .workflow_decision_applicability("decision-1", &moved_evidence)
            .unwrap(),
        DecisionApplicability::Stale
    );
    assert_eq!(
        store.load_workflow_decision("decision-1").unwrap(),
        original
    );
    assert_eq!(
        store.list_workflow_decisions("workflow-1").unwrap().len(),
        1
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_forged_human_authority_is_rejected_and_source_classes_do_not_self_escalate() {
    let (home, store) = seeded_store("authority");
    assert!(
        WorkflowDecisionRecord::new(WorkflowDecisionInput {
            decision_id: "forged-agent",
            workflow_run_id: "workflow-1",
            stage_run_id: Some("stage-1"),
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::HumanDecision,
            decision_type: "APPROVAL",
            decision_result: "ACCEPTED",
            predecessor_decision_id: None,
            candidate: None,
            evidence_reference: None,
            content_state: DecisionContentState::Full,
            safe_rationale: Some("agent string cannot create human authority"),
            created_unix_ms: 10,
        })
        .is_err()
    );

    let forged_human = WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: "forged-human",
        workflow_run_id: "workflow-1",
        stage_run_id: Some("stage-1"),
        source: TruthSource::HumanDecided,
        authority: StageTransitionAuthority::HumanDecision,
        decision_type: "APPROVAL",
        decision_result: "ACCEPTED",
        predecessor_decision_id: None,
        candidate: Some(candidate('a')),
        evidence_reference: None,
        content_state: DecisionContentState::Full,
        safe_rationale: Some("unbacked human-shaped row"),
        created_unix_ms: 10,
    })
    .unwrap();
    assert!(store.append_workflow_decision(&forged_human).is_err());

    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
            decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
            decision_type, decision_result, candidate_oid, candidate_tree,
            content_state, safe_rationale, created_unix_ms
         ) VALUES (?1, 'workflow-1', 'stage-1', 'HUMAN_DECIDED', 'HUMAN_DECISION',
                   'APPROVAL', 'ACCEPTED', ?2, ?3, 'FULL', 'forged direct row', 10)",
            rusqlite::params!["direct-human", candidate('a').oid, candidate('a').tree],
        )
        .unwrap();
    assert!(store.load_workflow_decision("direct-human").is_err());

    let winds = WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: "winds-observed",
        workflow_run_id: "workflow-1",
        stage_run_id: Some("stage-1"),
        source: TruthSource::WindsObserved,
        authority: StageTransitionAuthority::WindsPolicy,
        decision_type: "VERIFICATION_OBSERVATION",
        decision_result: "RECORDED",
        predecessor_decision_id: None,
        candidate: Some(candidate('a')),
        evidence_reference: Some("evidence:verify-a"),
        content_state: DecisionContentState::Full,
        safe_rationale: Some("Winds observation remains distinct from human acceptance"),
        created_unix_ms: 11,
    })
    .unwrap();
    assert_eq!(
        store.append_workflow_decision(&winds).unwrap(),
        DecisionAppendOutcome::Inserted
    );
    assert_eq!(
        store.load_workflow_decision("winds-observed").unwrap(),
        winds
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_restart_preserves_rejected_reverted_and_superseded_history_in_deterministic_order() {
    let (home, store) = seeded_store("restart-history");
    let rejected = decision("decision-a", "workflow-1", "REJECTED", None, None, None, 10);
    store.append_workflow_decision(&rejected).unwrap();
    let reverted = decision(
        "decision-b",
        "workflow-1",
        "REVERTED",
        Some("decision-a"),
        None,
        None,
        11,
    );
    store.append_workflow_decision(&reverted).unwrap();
    let superseded = decision(
        "decision-c",
        "workflow-1",
        "SUPERSEDED",
        Some("decision-b"),
        None,
        None,
        12,
    );
    store.append_workflow_decision(&superseded).unwrap();
    drop(store);

    let store = Store::open(&home).unwrap();
    assert_eq!(
        store.list_workflow_decisions("workflow-1").unwrap(),
        vec![rejected, reverted, superseded]
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_malformed_or_unbounded_decision_truth_fails_closed_without_mutating_history() {
    let (home, store) = seeded_store("malformed");
    let oversized = "x".repeat(129);
    assert!(
        WorkflowDecisionRecord::new(WorkflowDecisionInput {
            decision_id: "too-large",
            workflow_run_id: "workflow-1",
            stage_run_id: Some("stage-1"),
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::None,
            decision_type: &oversized,
            decision_result: "RECORDED",
            predecessor_decision_id: None,
            candidate: None,
            evidence_reference: None,
            content_state: DecisionContentState::Full,
            safe_rationale: Some("bounded metadata"),
            created_unix_ms: 10,
        })
        .is_err()
    );

    store
        .connection
        .execute_batch("PRAGMA ignore_check_constraints = ON;")
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
                decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                decision_type, decision_result, content_state, created_unix_ms
             ) VALUES ('malformed', 'workflow-1', 'stage-1', 'UNKNOWN', 'NONE',
                       'NOTE', 'RECORDED', 'FULL', 10)",
            [],
        )
        .unwrap();
    assert!(store.load_workflow_decision("malformed").is_err());
    assert!(store.list_workflow_decisions("workflow-1").is_err());
    drop(store);
    cleanup(home);
}

#[test]
fn t106_decision_existence_grants_no_stage_or_workflow_terminal_authority() {
    let (home, store) = seeded_store("no-authority-escalation");
    let observed = WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: "decision-no-authority",
        workflow_run_id: "workflow-1",
        stage_run_id: Some("stage-1"),
        source: TruthSource::WindsObserved,
        authority: StageTransitionAuthority::WindsPolicy,
        decision_type: "CANDIDATE_SELECTION",
        decision_result: "ACCEPTED",
        predecessor_decision_id: None,
        candidate: Some(candidate('a')),
        evidence_reference: Some("evidence:verify-a"),
        content_state: DecisionContentState::Full,
        safe_rationale: Some("decision history is not execution or completion authority"),
        created_unix_ms: 10,
    })
    .unwrap();
    store.append_workflow_decision(&observed).unwrap();
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        crate::domain::workflow::StageLifecycleState::Prepared
    );
    assert_eq!(
        store
            .load_workflow_run("workflow-1")
            .unwrap()
            .terminal_state,
        None
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t106_branched_successor_lineage_is_rejected_at_write_and_read_boundaries() {
    let (home, store) = seeded_store("branched-lineage");
    let first = decision(
        "decision-root",
        "workflow-1",
        "ACCEPTED",
        None,
        Some(candidate('a')),
        None,
        10,
    );
    let successor = decision(
        "decision-successor-a",
        "workflow-1",
        "SUPERSEDED",
        Some("decision-root"),
        Some(candidate('a')),
        None,
        11,
    );
    store.append_workflow_decision(&first).unwrap();
    store.append_workflow_decision(&successor).unwrap();
    let sibling = decision(
        "decision-successor-b",
        "workflow-1",
        "REJECTED",
        Some("decision-root"),
        Some(candidate('a')),
        None,
        12,
    );
    assert!(store.append_workflow_decision(&sibling).is_err());

    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
                decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                decision_type, decision_result, predecessor_decision_id, content_state, created_unix_ms
             ) VALUES ('decision-successor-b', 'workflow-1', 'stage-1', 'AGENT_REPORTED', 'NONE',
                       'REVIEW_DECISION', 'REJECTED', 'decision-root', 'FULL', 12)",
            [],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_decision("decision-successor-a")
            .is_err()
    );
    assert!(
        store
            .load_workflow_decision("decision-successor-b")
            .is_err()
    );
    assert!(store.list_workflow_decisions("workflow-1").is_err());
    drop(store);
    cleanup(home);
}
