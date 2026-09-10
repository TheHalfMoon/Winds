use super::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use crate::domain::workflow::{
    CandidateBaselineIdentity, DecisionAppendOutcome, DecisionApplicability,
    DecisionApplicabilityContext, DecisionContentState, RequiredEvidenceCompleteness,
    RetryFailureObservation, SideEffectTruth, StageLifecycleState, StageRunIdentity,
    StageTransitionAuthority, StageTransitionRequest, TruthSource, WorkflowDecisionInput,
    WorkflowDecisionRecord, WorkflowRunIdentity, evaluate_required_decision_evidence_completeness,
};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t108-{name}-{}-{sequence}",
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
                canonical_worktree_root: "/tmp/t108-workspace-1",
                git_common_dir: "/tmp/t108-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-2",
                canonical_worktree_root: "/tmp/t108-workspace-2",
                git_common_dir: "/tmp/t108-git-2",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T108 workstream",
            },
            2,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "T108 session",
            },
            3,
        )
        .unwrap();
    let workflow = WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap();
    store.create_workflow_run(&workflow, 4).unwrap();
    let stage = StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap();
    store.create_stage_run(&stage, None, 5).unwrap();
    (home, store)
}

fn candidate_a() -> CandidateBaselineIdentity {
    CandidateBaselineIdentity::new(&"a".repeat(40), &"b".repeat(40)).unwrap()
}

fn candidate_b() -> CandidateBaselineIdentity {
    CandidateBaselineIdentity::new(&"c".repeat(40), &"d".repeat(40)).unwrap()
}

fn agent_decision(
    id: &str,
    content_state: DecisionContentState,
    safe_rationale: Option<&str>,
    candidate: Option<CandidateBaselineIdentity>,
    evidence_reference: Option<&str>,
    created_unix_ms: i64,
) -> WorkflowDecisionRecord {
    WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: id,
        workflow_run_id: "workflow-1",
        stage_run_id: Some("stage-1"),
        source: TruthSource::AgentReported,
        authority: StageTransitionAuthority::None,
        decision_type: "REVIEW_NOTE",
        decision_result: "RECORDED",
        predecessor_decision_id: None,
        candidate,
        evidence_reference,
        content_state,
        safe_rationale,
        created_unix_ms,
    })
    .unwrap()
}

#[test]
fn t108_schema_integrity_version_context_and_partial_state_fail_closed() {
    let (home, store) = seeded_store("schema");
    assert_eq!(
        store
            .load_workflow_run("workflow-1")
            .unwrap()
            .schema_version,
        1
    );

    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_runs(
                    workflow_run_id, workspace_id, workstream_id, schema_version, created_unix_ms
                 ) VALUES ('workflow-v2', 'workspace-1', 'workstream-1', 2, 6)",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_runs(
                    workflow_run_id, workspace_id, workstream_id, schema_version, created_unix_ms
                 ) VALUES ('workflow-wrong-context', 'workspace-2', 'workstream-1', 1, 6)",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_runs(
                    workflow_run_id, workspace_id, workstream_id, schema_version, created_unix_ms
                 ) VALUES ('', 'workspace-1', 'workstream-1', 1, 6)",
                [],
            )
            .is_err()
    );

    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-2", "workspace-1", "workstream-1").unwrap(),
            6,
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_stage_runs(
                    stage_run_id, workflow_run_id, stage_key, attempt_ordinal,
                    predecessor_stage_run_id, relation_kind, lifecycle_state,
                    created_unix_ms, updated_unix_ms
                 ) VALUES ('stage-cross-workflow', 'workflow-2', 'build', 2,
                           'stage-1', 'RETRY_OF', 'PREPARED', 7, 7)",
                [],
            )
            .is_err()
    );

    store
        .connection
        .execute_batch("DROP INDEX idx_workflow_decisions_stage_time")
        .unwrap();
    let error = store.validate_workflow_schema().unwrap_err().to_string();
    assert!(error.contains("workflow schema object inventory mismatch"));
    drop(store);

    let reopen_error = match Store::open(&home) {
        Ok(_) => panic!("partial workflow schema must not be repaired or treated as clean"),
        Err(error) => error.to_string(),
    };
    assert!(reopen_error.contains("workflow schema object inventory mismatch"));
    cleanup(home);
}

#[test]
fn t108_corrupt_database_open_preserves_exact_historical_bytes() {
    let home = test_home("physical-corruption");
    let database = home.join("winds.db");
    let sentinel = b"Winds T108 corrupt historical database sentinel\0not-sqlite".to_vec();
    fs::write(&database, &sentinel).unwrap();

    let error = match Store::open(&home) {
        Ok(_) => panic!("corrupt historical database must not become a clean Store"),
        Err(error) => error.to_string(),
    };
    assert!(!error.is_empty());
    assert_eq!(fs::read(&database).unwrap(), sentinel);
    cleanup(home);
}

#[test]
fn t108_non_full_decision_markers_require_payload_removal_before_persistence() {
    let (home, store) = seeded_store("redaction");
    let forbidden = "API_KEY=super-secret-t108";
    for state in [
        DecisionContentState::Redacted,
        DecisionContentState::Omitted,
        DecisionContentState::Unavailable,
    ] {
        let error = WorkflowDecisionRecord::new(WorkflowDecisionInput {
            decision_id: "decision-rejected-payload",
            workflow_run_id: "workflow-1",
            stage_run_id: Some("stage-1"),
            source: TruthSource::AgentReported,
            authority: StageTransitionAuthority::None,
            decision_type: "REVIEW_NOTE",
            decision_result: "RECORDED",
            predecessor_decision_id: None,
            candidate: Some(candidate_a()),
            evidence_reference: Some("evidence:stable"),
            content_state: state,
            safe_rationale: Some(forbidden),
            created_unix_ms: 10,
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("non-FULL workflow decision content"));
    }

    let records = [
        agent_decision(
            "decision-full",
            DecisionContentState::Full,
            Some("bounded safe rationale"),
            Some(candidate_a()),
            Some("evidence:stable"),
            10,
        ),
        agent_decision(
            "decision-redacted",
            DecisionContentState::Redacted,
            None,
            Some(candidate_a()),
            Some("evidence:stable"),
            11,
        ),
        agent_decision(
            "decision-omitted",
            DecisionContentState::Omitted,
            None,
            Some(candidate_a()),
            Some("evidence:stable"),
            12,
        ),
        agent_decision(
            "decision-unavailable",
            DecisionContentState::Unavailable,
            None,
            Some(candidate_a()),
            Some("evidence:stable"),
            13,
        ),
    ];
    for record in &records {
        assert_eq!(
            store.append_workflow_decision(record).unwrap(),
            DecisionAppendOutcome::Inserted
        );
    }

    let markers = store
        .connection
        .prepare("SELECT content_state FROM workflow_decisions ORDER BY created_unix_ms")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(markers, vec!["FULL", "REDACTED", "OMITTED", "UNAVAILABLE"]);

    let redacted = store.load_workflow_decision("decision-redacted").unwrap();
    assert_eq!(redacted.content_state, DecisionContentState::Redacted);
    assert_eq!(redacted.safe_rationale, None);
    assert_eq!(redacted.candidate, Some(candidate_a()));
    assert_eq!(
        redacted.evidence_reference.as_deref(),
        Some("evidence:stable")
    );
    assert_eq!(redacted.source, TruthSource::AgentReported);
    assert_eq!(redacted.authority, StageTransitionAuthority::None);

    let leaked: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_decisions
             WHERE COALESCE(safe_rationale, '') LIKE ?1",
            params![format!("%{forbidden}%")],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(leaked, 0);
    store
        .connection
        .execute_batch("PRAGMA wal_checkpoint(FULL)")
        .unwrap();
    let db_bytes = fs::read(home.join("winds.db")).unwrap();
    assert!(
        !db_bytes
            .windows(forbidden.len())
            .any(|window| window == forbidden.as_bytes())
    );
    let wal = home.join("winds.db-wal");
    if wal.exists() {
        let wal_bytes = fs::read(wal).unwrap();
        assert!(
            !wal_bytes
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes())
        );
    }

    drop(store);
    cleanup(home);
}

#[test]
fn t108_required_evidence_completeness_never_promotes_loss_missing_or_stale_truth() {
    let current_candidate = candidate_a();
    let references = vec!["evidence:stable".to_owned()];
    let current =
        DecisionApplicabilityContext::new(Some(current_candidate.clone()), &references).unwrap();
    let complete = agent_decision(
        "decision-complete",
        DecisionContentState::Full,
        None,
        Some(current_candidate.clone()),
        Some("evidence:stable"),
        10,
    );
    assert_eq!(
        evaluate_required_decision_evidence_completeness(&complete, &current),
        RequiredEvidenceCompleteness::Complete
    );

    for state in [
        DecisionContentState::Redacted,
        DecisionContentState::Omitted,
        DecisionContentState::Unavailable,
    ] {
        let incomplete = agent_decision(
            "decision-loss",
            state,
            None,
            Some(current_candidate.clone()),
            Some("evidence:stable"),
            11,
        );
        assert_eq!(
            evaluate_required_decision_evidence_completeness(&incomplete, &current),
            RequiredEvidenceCompleteness::Incomplete
        );
    }

    let missing = agent_decision(
        "decision-missing",
        DecisionContentState::Full,
        None,
        Some(current_candidate.clone()),
        None,
        12,
    );
    assert_eq!(
        evaluate_required_decision_evidence_completeness(&missing, &current),
        RequiredEvidenceCompleteness::Incomplete
    );

    let stale_candidate =
        DecisionApplicabilityContext::new(Some(candidate_b()), &references).unwrap();
    assert_eq!(
        evaluate_required_decision_evidence_completeness(&complete, &stale_candidate),
        RequiredEvidenceCompleteness::Incomplete
    );
    let missing_reference =
        DecisionApplicabilityContext::new(Some(current_candidate), &[]).unwrap();
    assert_eq!(
        evaluate_required_decision_evidence_completeness(&complete, &missing_reference),
        RequiredEvidenceCompleteness::Incomplete
    );
}

#[test]
fn t108_malformed_persisted_decision_candidate_and_content_truth_fail_closed() {
    let (home, store) = seeded_store("malformed-decision");
    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
                decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                decision_type, decision_result, predecessor_decision_id,
                candidate_oid, candidate_tree, evidence_reference, content_state,
                safe_rationale, created_unix_ms
             ) VALUES ('decision-bad-authority', 'workflow-1', 'stage-1',
                       'HUMAN_DECIDED', 'WINDS_POLICY', 'REVIEW_NOTE', 'RECORDED', NULL,
                       NULL, NULL, NULL, 'FULL', NULL, 10)",
            [],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_decision("decision-bad-authority")
            .is_err()
    );

    let bad_oid = "g".repeat(40);
    let good_tree = "b".repeat(40);
    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
                decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                decision_type, decision_result, predecessor_decision_id,
                candidate_oid, candidate_tree, evidence_reference, content_state,
                safe_rationale, created_unix_ms
             ) VALUES ('decision-bad-candidate', 'workflow-1', 'stage-1',
                       'AGENT_REPORTED', 'NONE', 'REVIEW_NOTE', 'RECORDED', NULL,
                       ?1, ?2, 'evidence:bad', 'FULL', NULL, 11)",
            params![bad_oid, good_tree],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_decision("decision-bad-candidate")
            .is_err()
    );

    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_decisions(
                    decision_id, workflow_run_id, source_class, authority_class,
                    decision_type, decision_result, content_state, created_unix_ms
                 ) VALUES ('decision-bad-content-state', 'workflow-1',
                           'AGENT_REPORTED', 'NONE', 'REVIEW_NOTE', 'RECORDED', 'SECRET', 12)",
                [],
            )
            .is_err()
    );
    let retained: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_decisions
             WHERE decision_id IN ('decision-bad-authority', 'decision-bad-candidate')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(retained, 2);

    drop(store);
    cleanup(home);
}

#[test]
fn t108_reconstructed_bindings_require_exact_schema_valid_complete_reports() {
    let (home, store) = seeded_store("reconstruction-corruption");
    for (binding_id, now_ms) in [
        ("binding-missing", 10_i64),
        ("binding-partial", 11_i64),
        ("binding-version", 12_i64),
    ] {
        store
            .connection
            .execute(
                "INSERT INTO workflow_actor_bindings(
                    binding_id, stage_run_id, winds_session_id, runtime_binding_id,
                    continuation_class, bound_unix_ms
                 ) VALUES (?1, 'stage-1', 'session-1', NULL, 'RECONSTRUCTED', ?2)",
                params![binding_id, now_ms],
            )
            .unwrap();
    }
    let missing = store
        .load_workflow_actor_binding("binding-missing")
        .unwrap_err()
        .to_string();
    assert!(missing.contains("missing its required reconstruction report"));

    let partial = r#"{"schema_version":1,"binding_id":"binding-partial","stage_run_id":"stage-1","items":[]}"#;
    store
        .connection
        .execute(
            "INSERT INTO workflow_reconstruction_reports(
                reconstruction_report_id, binding_id, schema_version,
                canonical_report_json, created_unix_ms
             ) VALUES ('report-partial', 'binding-partial', 1, ?1, 13)",
            params![partial],
        )
        .unwrap();
    let partial_error = store
        .load_workflow_actor_binding("binding-partial")
        .unwrap_err()
        .to_string();
    assert!(partial_error.contains("every first-slice category exactly once"));

    let incompatible = r#"{"schema_version":2,"binding_id":"binding-version","stage_run_id":"stage-1","items":[]}"#;
    store
        .connection
        .execute(
            "INSERT INTO workflow_reconstruction_reports(
                reconstruction_report_id, binding_id, schema_version,
                canonical_report_json, created_unix_ms
             ) VALUES ('report-version', 'binding-version', 1, ?1, 14)",
            params![incompatible],
        )
        .unwrap();
    let version_error = store
        .load_workflow_actor_binding("binding-version")
        .unwrap_err()
        .to_string();
    assert!(version_error.contains("unsupported reconstruction report schema version"));

    let retained: (i64, i64) = (
        store
            .connection
            .query_row("SELECT COUNT(*) FROM workflow_actor_bindings", [], |row| {
                row.get(0)
            })
            .unwrap(),
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM workflow_reconstruction_reports",
                [],
                |row| row.get(0),
            )
            .unwrap(),
    );
    assert_eq!(retained, (3, 2));

    drop(store);
    cleanup(home);
}

#[test]
fn t108_restart_preserves_failed_stage_decision_identity_and_stale_history() {
    let (home, store) = seeded_store("restart-history");
    let start = StageTransitionRequest::new(
        "start-stage",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::AgentReported,
        StageTransitionAuthority::None,
    )
    .unwrap();
    store.transition_stage_run("stage-1", &start, 6).unwrap();

    let failure = RetryFailureObservation::new(
        "TEST_FAILURE",
        Some("checkpoint-1"),
        "candidate remained unchanged",
        SideEffectTruth::SafeOrIdempotent,
    )
    .unwrap();
    let resolution = store
        .record_stage_failure("stage-1", "record-failure", &failure, 7)
        .unwrap();
    assert_eq!(resolution.terminal_state, StageLifecycleState::Failed);

    let decision = agent_decision(
        "decision-history",
        DecisionContentState::Full,
        Some("historical failure retained"),
        Some(candidate_a()),
        Some("evidence:failed-attempt"),
        8,
    );
    store.append_workflow_decision(&decision).unwrap();
    drop(store);

    let reopened = Store::open(&home).unwrap();
    reopened.validate_workflow_schema().unwrap();
    let workflow = reopened.load_workflow_run("workflow-1").unwrap();
    assert_eq!(workflow.identity.workflow_run_id, "workflow-1");
    assert_eq!(workflow.identity.workspace_id, "workspace-1");
    assert_eq!(workflow.identity.workstream_id, "workstream-1");
    assert_eq!(workflow.schema_version, 1);

    let stage = reopened.load_stage_run("stage-1").unwrap();
    assert_eq!(stage.lifecycle_state, StageLifecycleState::Failed);
    assert_eq!(stage.failure_observation.as_ref(), Some(&failure));
    assert_eq!(stage.identity.attempt_ordinal, 1);

    let loaded = reopened.load_workflow_decision("decision-history").unwrap();
    assert_eq!(loaded, decision);
    let stale = DecisionApplicabilityContext::new(Some(candidate_b()), &[]).unwrap();
    assert_eq!(
        reopened
            .workflow_decision_applicability("decision-history", &stale)
            .unwrap(),
        DecisionApplicability::Stale
    );
    let decision_count: i64 = reopened
        .connection
        .query_row("SELECT COUNT(*) FROM workflow_decisions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(decision_count, 1);

    drop(reopened);
    cleanup(home);
}
