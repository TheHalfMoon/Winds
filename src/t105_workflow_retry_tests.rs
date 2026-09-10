use super::{NewWorkspace, NewWorkstream, Store};
use crate::domain::workflow::{
    RETRY_OUTCOME_AMBIGUOUS_EFFECT, RETRY_OUTCOME_BUDGET_EXHAUSTED, RETRY_OUTCOME_FAILURE_RECORDED,
    RETRY_OUTCOME_NO_PROGRESS, RetryFailureObservation, RetryProgressComparison, SideEffectTruth,
    StageLifecycleState, StageRunIdentity, StageTransitionAuthority, StageTransitionRequest,
    TruthSource, WorkflowRunIdentity, compare_retry_progress,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t105-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn seeded_store(name: &str) -> (PathBuf, Store) {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t105-workspace",
                git_common_dir: "/tmp/t105-git",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T105 workstream",
            },
            2,
        )
        .unwrap();
    let workflow = WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap();
    store.create_workflow_run(&workflow, 3).unwrap();
    let stage = StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap();
    store.create_stage_run(&stage, None, 4).unwrap();
    activate(&store, "stage-1", "activate-1", 5);
    (home, store)
}

fn cleanup(home: PathBuf) {
    if home.exists() {
        fs::remove_dir_all(home).unwrap();
    }
}

fn activate(store: &Store, stage_run_id: &str, operation_id: &str, now_ms: i64) {
    let request = StageTransitionRequest::new(
        operation_id,
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    store
        .transition_stage_run(stage_run_id, &request, now_ms)
        .unwrap();
}

fn observation(
    failure_class: &str,
    checkpoint: Option<&str>,
    basis: &str,
) -> RetryFailureObservation {
    RetryFailureObservation::new(
        failure_class,
        checkpoint,
        basis,
        SideEffectTruth::SafeOrIdempotent,
    )
    .unwrap()
}

fn retry(stage_run_id: &str, ordinal: u32, predecessor: &str) -> StageRunIdentity {
    StageRunIdentity::new(
        stage_run_id,
        "workflow-1",
        "build",
        ordinal,
        Some(predecessor),
    )
    .unwrap()
}

#[test]
fn t105_progress_comparator_is_deterministic_and_candidate_change_alone_is_not_progress() {
    let previous = observation("compile_error", Some("checkpoint-1"), "candidate:a/tree:a");
    let candidate_only = observation("compile_error", Some("checkpoint-1"), "candidate:b/tree:b");
    let checkpoint_progress =
        observation("compile_error", Some("checkpoint-2"), "candidate:b/tree:b");
    let failure_progress = observation("test_failure", Some("checkpoint-1"), "candidate:b/tree:b");

    assert_eq!(
        compare_retry_progress(&previous, &candidate_only),
        RetryProgressComparison::NoProgress
    );
    assert_eq!(
        compare_retry_progress(&previous, &checkpoint_progress),
        RetryProgressComparison::MaterialProgress
    );
    assert_eq!(
        compare_retry_progress(&previous, &failure_progress),
        RetryProgressComparison::MaterialProgress
    );
    assert_eq!(
        compare_retry_progress(&previous, &candidate_only),
        compare_retry_progress(&previous, &candidate_only)
    );
}

#[test]
fn t105_first_and_second_no_progress_retries_are_distinct_and_third_is_rejected() {
    let (home, store) = seeded_store("budget");
    let failure = observation("compile_error", Some("checkpoint-1"), "basis-1");
    let initial = store
        .record_stage_failure("stage-1", "fail-1", &failure, 6)
        .unwrap();
    assert_eq!(initial.outcome_reason, RETRY_OUTCOME_FAILURE_RECORDED);
    assert!(store.load_stage_run("stage-2").is_err());

    let second = retry("stage-2", 2, "stage-1");
    store.create_explicit_retry_stage_run(&second, 7).unwrap();
    activate(&store, "stage-2", "activate-2", 8);
    let first_retry_failure = store
        .record_stage_failure("stage-2", "fail-2", &failure, 9)
        .unwrap();
    assert_eq!(first_retry_failure.consecutive_no_progress_retries, 1);
    assert_eq!(
        first_retry_failure.outcome_reason,
        RETRY_OUTCOME_NO_PROGRESS
    );

    let third = retry("stage-3", 3, "stage-2");
    store.create_explicit_retry_stage_run(&third, 10).unwrap();
    activate(&store, "stage-3", "activate-3", 11);
    let second_retry_failure = store
        .record_stage_failure("stage-3", "fail-3", &failure, 12)
        .unwrap();
    assert_eq!(second_retry_failure.consecutive_no_progress_retries, 2);
    assert_eq!(
        second_retry_failure.outcome_reason,
        RETRY_OUTCOME_BUDGET_EXHAUSTED
    );

    let fourth = retry("stage-4", 4, "stage-3");
    assert!(store.create_explicit_retry_stage_run(&fourth, 13).is_err());
    assert!(store.load_stage_run("stage-4").is_err());
    drop(store);
    cleanup(home);
}

#[test]
fn t105_generic_stage_creation_cannot_bypass_failed_predecessor_retry_policy() {
    let (home, store) = seeded_store("explicit-only");
    let failure = observation("compile_error", None, "basis-1");
    store
        .record_stage_failure("stage-1", "fail-1", &failure, 6)
        .unwrap();
    let second = retry("stage-2", 2, "stage-1");
    assert!(
        store
            .create_stage_run(
                &second,
                Some(crate::domain::workflow::StageAttemptRelation::Retry),
                7,
            )
            .is_err()
    );
    assert!(store.load_stage_run("stage-2").is_err());
    drop(store);
    cleanup(home);
}

#[test]
fn t105_material_progress_resets_consecutive_no_progress_count() {
    let (home, store) = seeded_store("progress-reset");
    let first = observation("compile_error", Some("checkpoint-1"), "basis-a");
    store
        .record_stage_failure("stage-1", "fail-1", &first, 6)
        .unwrap();
    let second = retry("stage-2", 2, "stage-1");
    store.create_explicit_retry_stage_run(&second, 7).unwrap();
    activate(&store, "stage-2", "activate-2", 8);
    store
        .record_stage_failure("stage-2", "fail-2", &first, 9)
        .unwrap();

    let third = retry("stage-3", 3, "stage-2");
    store.create_explicit_retry_stage_run(&third, 10).unwrap();
    activate(&store, "stage-3", "activate-3", 11);
    let progressed = observation("compile_error", Some("checkpoint-2"), "basis-b");
    let result = store
        .record_stage_failure("stage-3", "fail-3", &progressed, 12)
        .unwrap();
    assert_eq!(
        result.comparison,
        Some(RetryProgressComparison::MaterialProgress)
    );
    assert_eq!(result.consecutive_no_progress_retries, 0);
    assert_eq!(result.outcome_reason, RETRY_OUTCOME_FAILURE_RECORDED);

    let fourth = retry("stage-4", 4, "stage-3");
    store.create_explicit_retry_stage_run(&fourth, 13).unwrap();
    drop(store);
    cleanup(home);
}

#[test]
fn t105_ambiguous_non_idempotent_effect_requires_recovery_and_never_retries() {
    let (home, store) = seeded_store("ambiguous-effect");
    let ambiguous = RetryFailureObservation::new(
        "external_write_unknown",
        Some("checkpoint-before-write"),
        "effect:possibly-completed",
        SideEffectTruth::AmbiguousNonIdempotent,
    )
    .unwrap();
    let result = store
        .record_stage_failure("stage-1", "ambiguous-1", &ambiguous, 6)
        .unwrap();
    assert_eq!(result.terminal_state, StageLifecycleState::RecoveryRequired);
    assert_eq!(result.outcome_reason, RETRY_OUTCOME_AMBIGUOUS_EFFECT);
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        StageLifecycleState::RecoveryRequired
    );
    let second = retry("stage-2", 2, "stage-1");
    assert!(store.create_explicit_retry_stage_run(&second, 7).is_err());
    assert!(store.load_stage_run("stage-2").is_err());
    drop(store);
    cleanup(home);
}

#[test]
fn t105_successful_later_retry_preserves_failed_predecessor_history() {
    let (home, store) = seeded_store("history");
    let failure = observation("test_failure", Some("checkpoint-1"), "basis-1");
    store
        .record_stage_failure("stage-1", "fail-1", &failure, 6)
        .unwrap();
    let second = retry("stage-2", 2, "stage-1");
    store.create_explicit_retry_stage_run(&second, 7).unwrap();
    activate(&store, "stage-2", "activate-2", 8);
    store
        .record_stage_failure("stage-2", "fail-2", &failure, 9)
        .unwrap();
    let third = retry("stage-3", 3, "stage-2");
    store.create_explicit_retry_stage_run(&third, 10).unwrap();
    activate(&store, "stage-3", "activate-3", 11);
    let complete = StageTransitionRequest::new(
        "complete-3",
        StageLifecycleState::Active,
        StageLifecycleState::Completed,
        TruthSource::WindsObserved,
        StageTransitionAuthority::WindsPolicy,
    )
    .unwrap();
    store
        .transition_stage_run("stage-3", &complete, 12)
        .unwrap();

    let initial = store.load_stage_run("stage-1").unwrap();
    let first_retry = store.load_stage_run("stage-2").unwrap();
    let success = store.load_stage_run("stage-3").unwrap();
    assert_eq!(initial.lifecycle_state, StageLifecycleState::Failed);
    assert_eq!(
        initial.outcome_reason.as_deref(),
        Some(RETRY_OUTCOME_FAILURE_RECORDED)
    );
    assert_eq!(first_retry.lifecycle_state, StageLifecycleState::Failed);
    assert_eq!(
        first_retry.outcome_reason.as_deref(),
        Some(RETRY_OUTCOME_NO_PROGRESS)
    );
    assert!(initial.failure_observation.is_some());
    assert!(first_retry.failure_observation.is_some());
    assert_eq!(success.lifecycle_state, StageLifecycleState::Completed);
    drop(store);
    cleanup(home);
}

#[test]
fn t105_noncanonical_durable_failure_truth_fails_closed_on_read() {
    let (home, store) = seeded_store("noncanonical-durable");
    let failure = observation("compile_error", Some("checkpoint-1"), "basis-1");
    store
        .record_stage_failure("stage-1", "fail-1", &failure, 6)
        .unwrap();
    store
        .connection
        .execute(
            "UPDATE workflow_stage_runs SET failure_class = 'compile_error' WHERE stage_run_id = 'stage-1'",
            [],
        )
        .unwrap();
    assert!(store.load_stage_run("stage-1").is_err());
    drop(store);

    let store = Store::open(&home).unwrap();
    assert!(store.load_stage_run("stage-1").is_err());
    drop(store);
    cleanup(home);
}
