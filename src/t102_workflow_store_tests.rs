use super::{
    NewWorkspace, NewWorkstream, Store, expected_workflow_schema_objects, workflow_schema_objects,
};
use crate::domain::workflow::{
    StageAttemptRelation, StageLifecycleState, StageRunIdentity, StageTransitionAuthority,
    StageTransitionOutcome, StageTransitionRequest, TruthSource, WorkflowRunIdentity,
};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t102-{name}-{}-{sequence}",
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
                canonical_worktree_root: "/tmp/t102-workspace-1",
                git_common_dir: "/tmp/t102-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-2",
                canonical_worktree_root: "/tmp/t102-workspace-2",
                git_common_dir: "/tmp/t102-git-2",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T102 workstream",
            },
            2,
        )
        .unwrap();
    (home, store)
}

fn workflow(id: &str) -> WorkflowRunIdentity {
    WorkflowRunIdentity::new(id, "workspace-1", "workstream-1").unwrap()
}

fn stage(
    id: &str,
    workflow_run_id: &str,
    stage_key: &str,
    ordinal: u32,
    predecessor: Option<&str>,
) -> StageRunIdentity {
    StageRunIdentity::new(id, workflow_run_id, stage_key, ordinal, predecessor).unwrap()
}

#[test]
fn t102_migration_inventory_is_complete_exact_and_idempotent() {
    let home = test_home("inventory");
    let store = Store::open(&home).unwrap();
    let expected = expected_workflow_schema_objects().unwrap();
    let observed = workflow_schema_objects(&store.connection).unwrap();
    assert_eq!(observed, expected);

    let tables = observed
        .iter()
        .filter_map(|(name, (kind, _, _))| (kind == "table").then_some(name.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        tables,
        vec![
            "workflow_actor_bindings",
            "workflow_artifact_baselines",
            "workflow_decisions",
            "workflow_reconstruction_reports",
            "workflow_runs",
            "workflow_stage_runs",
        ]
    );
    assert_eq!(
        observed
            .values()
            .filter(|(kind, _, _)| kind == "index")
            .count(),
        7
    );
    assert_eq!(
        observed
            .values()
            .filter(|(kind, _, _)| kind == "trigger")
            .count(),
        18
    );

    store
        .connection
        .execute_batch(include_str!(
            "../migrations/0010_resumable_workflow_ledger.sql"
        ))
        .unwrap();
    store.validate_workflow_schema().unwrap();
    let foreign_keys: i64 = store
        .connection
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(foreign_keys, 1);
    drop(store);
    cleanup(home);
}

#[test]
fn t102_partial_or_modified_schema_fails_closed_without_silent_repair() {
    let home = test_home("partial-schema");
    let store = Store::open(&home).unwrap();
    store
        .connection
        .execute_batch("DROP INDEX idx_workflow_decisions_stage_time")
        .unwrap();
    let error = store.validate_workflow_schema().unwrap_err().to_string();
    assert!(error.contains("workflow schema object inventory mismatch"));
    drop(store);
    let reopen_error = match Store::open(&home) {
        Ok(_) => panic!("partial workflow schema must not be silently repaired"),
        Err(error) => error.to_string(),
    };
    assert!(reopen_error.contains("workflow schema object inventory mismatch"));
    cleanup(home);

    let home = test_home("modified-schema");
    let store = Store::open(&home).unwrap();
    store
        .connection
        .execute_batch(
            "DROP TRIGGER trg_workflow_runs_no_delete;
             CREATE TRIGGER trg_workflow_runs_no_delete
             BEFORE DELETE ON workflow_runs BEGIN SELECT 1; END;",
        )
        .unwrap();
    let error = store.validate_workflow_schema().unwrap_err().to_string();
    assert!(error.contains("workflow schema object definition mismatch"));
    drop(store);
    cleanup(home);
}

#[test]
fn t102_workflow_hierarchy_and_stage_lineage_fail_closed_at_app_and_database_boundaries() {
    let (home, store) = seeded_store("identity-boundaries");
    store
        .create_workflow_run(&workflow("workflow-1"), 3)
        .unwrap();

    let mismatched =
        WorkflowRunIdentity::new("workflow-bad", "workspace-2", "workstream-1").unwrap();
    assert!(store.create_workflow_run(&mismatched, 3).is_err());
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_runs(
                    workflow_run_id, workspace_id, workstream_id, schema_version, created_unix_ms
                 ) VALUES ('workflow-direct-bad', 'workspace-2', 'workstream-1', 1, 3)",
                [],
            )
            .is_err()
    );

    let first = stage("stage-1", "workflow-1", "build", 1, None);
    store.create_stage_run(&first, None, 4).unwrap();
    let duplicate_ordinal = stage("stage-duplicate", "workflow-1", "build", 1, None);
    assert!(store.create_stage_run(&duplicate_ordinal, None, 4).is_err());

    let wrong_stage = stage("stage-2", "workflow-1", "test", 2, Some("stage-1"));
    assert!(
        store
            .create_stage_run(&wrong_stage, Some(StageAttemptRelation::Retry), 5)
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_stage_runs(
                    stage_run_id, workflow_run_id, stage_key, attempt_ordinal,
                    predecessor_stage_run_id, relation_kind, lifecycle_state,
                    created_unix_ms, updated_unix_ms
                 ) VALUES ('stage-direct-bad', 'workflow-1', 'test', 2,
                           'stage-1', 'RETRY_OF', 'PREPARED', 5, 5)",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_stage_runs SET stage_key = 'rewritten' WHERE stage_run_id = 'stage-1'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_stage_runs SET lifecycle_state = 'ACTIVE' WHERE stage_run_id = 'stage-1'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_stage_runs
                 SET lifecycle_state = 'COMPLETED',
                     last_transition_operation_id = 'forged-agent-complete',
                     last_transition_from_state = 'PREPARED',
                     last_transition_source = 'AGENT_REPORTED',
                     last_transition_authority = 'NONE'
                 WHERE stage_run_id = 'stage-1'",
                [],
            )
            .is_err()
    );

    drop(store);
    cleanup(home);
}

#[test]
fn t102_restart_preserves_identity_lifecycle_and_exact_idempotency_while_stale_cas_loses() {
    let (home, store) = seeded_store("restart-cas");
    store
        .create_workflow_run(&workflow("workflow-1"), 3)
        .unwrap();
    let first = stage("stage-1", "workflow-1", "build", 1, None);
    store.create_stage_run(&first, None, 4).unwrap();

    let activate = StageTransitionRequest::new(
        "operation-activate",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(matches!(
        store.transition_stage_run("stage-1", &activate, 5).unwrap(),
        StageTransitionOutcome::Apply(_)
    ));
    drop(store);

    let store = Store::open(&home).unwrap();
    let loaded_workflow = store.load_workflow_run("workflow-1").unwrap();
    assert_eq!(loaded_workflow.identity, workflow("workflow-1"));
    assert_eq!(loaded_workflow.schema_version, 1);
    assert_eq!(loaded_workflow.terminal_state, None);
    assert_eq!(loaded_workflow.created_unix_ms, 3);

    let loaded_stage = store.load_stage_run("stage-1").unwrap();
    assert_eq!(loaded_stage.identity, first);
    assert_eq!(loaded_stage.relation, None);
    assert_eq!(loaded_stage.lifecycle_state, StageLifecycleState::Active);
    assert_eq!(loaded_stage.created_unix_ms, 4);
    assert_eq!(loaded_stage.updated_unix_ms, 5);
    assert_eq!(
        loaded_stage.last_transition.as_ref().unwrap().operation_id,
        "operation-activate"
    );

    assert_eq!(
        store.transition_stage_run("stage-1", &activate, 6).unwrap(),
        StageTransitionOutcome::IdempotentNoChange
    );
    let stale = StageTransitionRequest::new(
        "operation-stale",
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(store.transition_stage_run("stage-1", &stale, 6).is_err());
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        StageLifecycleState::Active
    );

    let complete = StageTransitionRequest::new(
        "operation-complete",
        StageLifecycleState::Active,
        StageLifecycleState::Completed,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    assert!(matches!(
        store.transition_stage_run("stage-1", &complete, 7).unwrap(),
        StageTransitionOutcome::Apply(_)
    ));
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        StageLifecycleState::Completed
    );

    drop(store);
    cleanup(home);
}

#[test]
fn t102_future_schema_tables_are_inert_but_history_guards_are_enforced() {
    let (home, store) = seeded_store("inert-schema");
    store
        .create_workflow_run(&workflow("workflow-1"), 3)
        .unwrap();
    store
        .create_stage_run(&stage("stage-1", "workflow-1", "build", 1, None), None, 4)
        .unwrap();

    store
        .connection
        .execute(
            "INSERT INTO workflow_artifact_baselines(
                baseline_id, stage_run_id, baseline_kind, stable_reference, created_unix_ms
             ) VALUES ('baseline-1', 'stage-1', 'BOUNDED_BLOB_ARTIFACT', 'sha256:abc', 5)",
            [],
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_artifact_baselines SET stable_reference = 'changed' WHERE baseline_id = 'baseline-1'",
                [],
            )
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, continuation_class, bound_unix_ms
             ) VALUES ('binding-1', 'stage-1', 'UNPROVEN', 5)",
            [],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_reconstruction_reports(
                reconstruction_report_id, binding_id, schema_version, canonical_report_json, created_unix_ms
             ) VALUES ('report-1', 'binding-1', 1, '{}', 6)",
            [],
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_reconstruction_reports(
                    reconstruction_report_id, binding_id, schema_version, canonical_report_json, created_unix_ms
                 ) VALUES ('report-2', 'binding-1', 1, '{}', 6)",
                [],
            )
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_decisions(
                decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                decision_type, decision_result, content_state, created_unix_ms
             ) VALUES ('decision-1', 'workflow-1', 'stage-1', 'AGENT_REPORTED', 'NONE',
                       'NOTE', 'RECORDED', 'FULL', 7)",
            [],
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_decisions SET decision_result = 'REWRITTEN' WHERE decision_id = 'decision-1'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM workflow_decisions WHERE decision_id = 'decision-1'",
                []
            )
            .is_err()
    );

    store
        .create_workflow_run(&workflow("workflow-2"), 8)
        .unwrap();
    store
        .create_stage_run(&stage("stage-2", "workflow-2", "build", 1, None), None, 9)
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_decisions(
                    decision_id, workflow_run_id, stage_run_id, source_class, authority_class,
                    decision_type, decision_result, content_state, created_unix_ms
                 ) VALUES ('decision-cross-workflow', 'workflow-1', 'stage-2',
                           'WINDS_OBSERVED', 'WINDS_POLICY', 'NOTE', 'RECORDED', 'FULL', 10)",
                [],
            )
            .is_err()
    );

    let future_row_count: i64 = store
        .connection
        .query_row(
            "SELECT
                (SELECT COUNT(*) FROM workflow_artifact_baselines) +
                (SELECT COUNT(*) FROM workflow_actor_bindings) +
                (SELECT COUNT(*) FROM workflow_reconstruction_reports) +
                (SELECT COUNT(*) FROM workflow_decisions)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(future_row_count, 4);

    drop(store);
    cleanup(home);
}

#[test]
fn t102_successor_attempt_restart_preserves_exact_lineage() {
    let (home, store) = seeded_store("successor-restart");
    store
        .create_workflow_run(&workflow("workflow-1"), 3)
        .unwrap();
    let first = stage("stage-1", "workflow-1", "build", 1, None);
    store.create_stage_run(&first, None, 4).unwrap();
    let failed = StageTransitionRequest::new(
        "operation-fail",
        StageLifecycleState::Prepared,
        StageLifecycleState::Stale,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    store.transition_stage_run("stage-1", &failed, 5).unwrap();

    let successor = stage("stage-2", "workflow-1", "build", 2, Some("stage-1"));
    store
        .create_stage_run(&successor, Some(StageAttemptRelation::Retry), 6)
        .unwrap();
    drop(store);

    let store = Store::open(&home).unwrap();
    let loaded = store.load_stage_run("stage-2").unwrap();
    assert_eq!(loaded.identity, successor);
    assert_eq!(loaded.relation, Some(StageAttemptRelation::Retry));
    assert_eq!(loaded.lifecycle_state, StageLifecycleState::Prepared);
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        StageLifecycleState::Stale
    );
    let ordinal_count: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_stage_runs
             WHERE workflow_run_id = 'workflow-1' AND stage_key = 'build'",
            params![],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(ordinal_count, 2);

    drop(store);
    cleanup(home);
}
