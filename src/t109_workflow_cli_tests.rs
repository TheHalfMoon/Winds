use super::execute;
use crate::domain::workflow::StageLifecycleState;
use crate::store::{NewWorkspace, NewWorkstream, Store};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t109-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn seeded_home(name: &str) -> PathBuf {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t109-workspace-1",
                git_common_dir: "/tmp/t109-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T109 workstream",
            },
            2,
        )
        .unwrap();
    home
}

fn base_flags(home: &Path, action: &str) -> HashMap<String, String> {
    HashMap::from([
        ("action".to_owned(), action.to_owned()),
        ("home".to_owned(), home.to_string_lossy().into_owned()),
        ("workspace-id".to_owned(), "workspace-1".to_owned()),
        ("workstream-id".to_owned(), "workstream-1".to_owned()),
        ("workflow-id".to_owned(), "workflow-1".to_owned()),
    ])
}

fn call(home: &Path, action: &str, extra: &[(&str, &str)]) -> serde_json::Value {
    let mut flags = base_flags(home, action);
    for (key, value) in extra {
        flags.insert((*key).to_owned(), (*value).to_owned());
    }
    execute(flags).unwrap()
}

fn create_workflow(home: &Path) {
    let value = call(home, "create", &[]);
    assert_eq!(value["workflow"]["workflow_run_id"], "workflow-1");
}

fn prepare(home: &Path, stage_id: &str, stage_key: &str) {
    call(
        home,
        "prepare-stage",
        &[("stage-id", stage_id), ("stage-key", stage_key)],
    );
}

fn start(home: &Path, stage_id: &str, operation_id: &str) {
    call(
        home,
        "start-stage",
        &[("stage-id", stage_id), ("operation-id", operation_id)],
    );
}

#[test]
fn t109_command_is_collision_free_and_create_open_require_exact_canonical_identity() {
    let home = seeded_home("create-open");
    assert!(crate::usage().contains("winds workflow --action ACTION"));
    create_workflow(&home);

    let opened = call(&home, "open", &[]);
    assert_eq!(opened["workflow"]["workspace_id"], "workspace-1");
    assert_eq!(opened["workflow"]["workstream_id"], "workstream-1");

    let mut wrong = base_flags(&home, "open");
    wrong.insert("workspace-id".to_owned(), "workspace-forged".to_owned());
    assert!(execute(wrong).is_err());
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t109_stage_prepare_start_and_safe_transition_are_exact_and_terminal_strings_fail_closed() {
    let home = seeded_home("stage");
    create_workflow(&home);
    prepare(&home, "stage-1", "build");
    start(&home, "stage-1", "start-1");

    let waiting = call(
        &home,
        "transition-stage",
        &[
            ("stage-id", "stage-1"),
            ("operation-id", "wait-1"),
            ("to", "WAITING_APPROVAL"),
        ],
    );
    assert_eq!(waiting["stage"]["lifecycle_state"], "WAITING_APPROVAL");
    assert_eq!(waiting["outcome"]["source"], "WINDS_OBSERVED");
    assert_eq!(waiting["outcome"]["authority"], "NONE");

    let mut forged = base_flags(&home, "transition-stage");
    forged.extend([
        ("stage-id".to_owned(), "stage-1".to_owned()),
        ("operation-id".to_owned(), "forged-complete".to_owned()),
        ("to".to_owned(), "COMPLETED".to_owned()),
    ]);
    assert!(execute(forged).is_err());
    let store = Store::open(&home).unwrap();
    assert_eq!(
        store.load_stage_run("stage-1").unwrap().lifecycle_state,
        StageLifecycleState::WaitingApproval
    );
    drop(store);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t109_decision_record_query_preserves_source_and_rejects_human_authority_strings() {
    let home = seeded_home("decision");
    create_workflow(&home);
    prepare(&home, "stage-1", "review");
    let recorded = call(
        &home,
        "decision-record",
        &[
            ("stage-id", "stage-1"),
            ("decision-id", "decision-1"),
            ("decision-type", "REVIEW_RECOMMENDATION"),
            ("decision-result", "ACCEPTED"),
            ("source", "AGENT_REPORTED"),
            ("authority", "NONE"),
            ("content-state", "FULL"),
            ("safe-rationale", "SHIP IT WITH HIGH CONFIDENCE"),
        ],
    );
    assert_eq!(recorded["decision"]["source"], "AGENT_REPORTED");
    assert_eq!(recorded["decision"]["authority"], "NONE");

    let listed = call(&home, "decision-list", &[("stage-id", "stage-1")]);
    assert_eq!(listed["decisions"][0]["decision_result"], "ACCEPTED");
    assert_eq!(listed["decisions"][0]["source"], "AGENT_REPORTED");

    let mut forged = base_flags(&home, "decision-record");
    forged.extend([
        ("stage-id".to_owned(), "stage-1".to_owned()),
        ("decision-id".to_owned(), "decision-human".to_owned()),
        ("decision-type".to_owned(), "APPROVAL".to_owned()),
        ("decision-result".to_owned(), "ACCEPTED".to_owned()),
        ("source".to_owned(), "HUMAN_DECIDED".to_owned()),
        ("authority".to_owned(), "HUMAN_DECISION".to_owned()),
        ("content-state".to_owned(), "FULL".to_owned()),
    ]);
    assert!(execute(forged).is_err());
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t109_inspection_is_non_mutating_and_reviewer_handoff_excludes_builder_persuasion() {
    let home = seeded_home("inspection");
    create_workflow(&home);
    prepare(&home, "stage-1", "review");
    start(&home, "stage-1", "start-review");
    call(
        &home,
        "decision-record",
        &[
            ("stage-id", "stage-1"),
            ("decision-id", "decision-1"),
            ("decision-type", "REVIEW_RECOMMENDATION"),
            ("decision-result", "PASS"),
            ("source", "AGENT_REPORTED"),
            ("authority", "NONE"),
            ("content-state", "FULL"),
            ("safe-rationale", "SHIP IT WITH HIGH CONFIDENCE"),
        ],
    );
    let before_store = Store::open(&home).unwrap();
    let before_stage = before_store.load_stage_run("stage-1").unwrap();
    let before_decisions = before_store.list_workflow_decisions("workflow-1").unwrap();
    drop(before_store);

    let status = call(&home, "status", &[("stage-id", "stage-1")]);
    assert_eq!(status["verification"]["state"], "UNKNOWN");
    assert_eq!(status["human_acceptance"]["state"], "UNKNOWN");
    let resume = call(&home, "resume-preview", &[("stage-id", "stage-1")]);
    assert_eq!(resume["disposition"], "UNAVAILABLE");
    let blocked = call(&home, "why-blocked", &[("stage-id", "stage-1")]);
    assert_eq!(blocked["blocker"], "NONE");
    let handoff = call(&home, "reviewer-handoff", &[("stage-id", "stage-1")]);
    let handoff_text = serde_json::to_string(&handoff).unwrap();
    assert!(!handoff_text.contains("SHIP IT WITH HIGH CONFIDENCE"));
    assert!(!handoff_text.to_ascii_lowercase().contains("confidence"));

    let after_store = Store::open(&home).unwrap();
    assert_eq!(after_store.load_stage_run("stage-1").unwrap(), before_stage);
    assert_eq!(
        after_store.list_workflow_decisions("workflow-1").unwrap(),
        before_decisions
    );
    drop(after_store);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t109_explicit_retry_and_recovery_create_new_attempt_lineage_only_from_qualified_truth() {
    let home = seeded_home("retry-recovery");
    create_workflow(&home);

    prepare(&home, "retry-1", "retry-stage");
    start(&home, "retry-1", "start-retry");
    call(
        &home,
        "record-failure",
        &[
            ("stage-id", "retry-1"),
            ("operation-id", "fail-retry"),
            ("failure-class", "TEST_FAILURE"),
            ("progress-basis", "checkpoint-a"),
            ("side-effect", "SAFE_OR_IDEMPOTENT"),
        ],
    );
    let retry = call(
        &home,
        "retry",
        &[("stage-id", "retry-1"), ("new-stage-id", "retry-2")],
    );
    assert_eq!(retry["relation"], "RETRY_OF");
    assert_eq!(retry["stage"]["attempt_ordinal"], 2);
    assert_eq!(retry["stage"]["predecessor_stage_run_id"], "retry-1");

    prepare(&home, "recovery-1", "recovery-stage");
    start(&home, "recovery-1", "start-recovery");
    let failure = call(
        &home,
        "record-failure",
        &[
            ("stage-id", "recovery-1"),
            ("operation-id", "fail-recovery"),
            ("failure-class", "UNCERTAIN_EFFECT"),
            ("progress-basis", "external-side-effect-unknown"),
            ("side-effect", "AMBIGUOUS_NON_IDEMPOTENT"),
        ],
    );
    assert_eq!(failure["stage"]["lifecycle_state"], "RECOVERY_REQUIRED");
    let recovery = call(
        &home,
        "recover",
        &[("stage-id", "recovery-1"), ("new-stage-id", "recovery-2")],
    );
    assert_eq!(recovery["relation"], "RECOVERY_OF");
    assert_eq!(recovery["stage"]["attempt_ordinal"], 2);

    let mut invalid = base_flags(&home, "recover");
    invalid.extend([
        ("stage-id".to_owned(), "retry-2".to_owned()),
        ("new-stage-id".to_owned(), "forged-recovery".to_owned()),
    ]);
    assert!(execute(invalid).is_err());
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t109_machine_readable_inputs_are_bounded_and_unknown_actions_fail_closed() {
    let home = seeded_home("bounds");
    create_workflow(&home);
    prepare(&home, "stage-1", "inspect");
    let evidence = (0..257)
        .map(|index| format!("evidence-{index}"))
        .collect::<Vec<_>>();
    let mut flags = base_flags(&home, "status");
    flags.insert("stage-id".to_owned(), "stage-1".to_owned());
    flags.insert(
        "evidence-json".to_owned(),
        serde_json::to_string(&evidence).unwrap(),
    );
    assert!(execute(flags).is_err());

    let mut unknown = base_flags(&home, "generic-runtime");
    assert!(execute(std::mem::take(&mut unknown)).is_err());
    fs::remove_dir_all(home).unwrap();
}
