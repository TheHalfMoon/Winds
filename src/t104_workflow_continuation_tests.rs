use super::{NewReconstructedActorBinding, NewWindsSession, NewWorkspace, NewWorkstream, Store};
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeBindingOwnership, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity,
    RuntimeKind, RuntimeResumeResolution, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{
    ReconstructionCategory, ReconstructionContentState, ReconstructionItem,
    ReconstructionSourceClass, ReconstructionTransferState, StageRunIdentity,
    WorkflowContinuationClass, WorkflowRunIdentity, build_reconstruction_preview,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t104-{name}-{}-{sequence}",
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
                canonical_worktree_root: "/tmp/t104-workspace-1",
                git_common_dir: "/tmp/t104-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T104 workstream",
            },
            2,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "T104 session",
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

fn runtime_discovery() -> RuntimeDiscovery {
    RuntimeDiscovery {
        runtime: RuntimeKind::Claude,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: Path::new("/tmp/winds-t104-claude").to_path_buf(),
            canonical_path: Path::new("/tmp/winds-t104-claude").to_path_buf(),
            byte_len: 7,
            sha256: "a".repeat(64),
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("2.1.248-t104-fixture".to_owned()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        capabilities: Vec::new(),
        auth_readiness: AuthReadinessEvidence {
            readiness: AuthReadiness::Unknown,
            source: EvidenceSource::Unavailable,
        },
        agent_execution: AgentExecutionObservation::NotPerformed,
    }
}

fn create_runtime_binding(
    store: &Store,
    binding_id: &str,
    native_session_id: &str,
) -> crate::agentic_runtime::RuntimeSessionBinding {
    store
        .create_runtime_session_binding(
            binding_id,
            "session-1",
            &runtime_discovery(),
            Some(native_session_id),
            6,
        )
        .unwrap();
    store.load_runtime_session_binding(binding_id).unwrap()
}

fn reconstruction_items() -> Vec<ReconstructionItem> {
    vec![
        ReconstructionItem::new(
            ReconstructionCategory::CanonicalWorkContext,
            ReconstructionSourceClass::StoredCanonicalReference,
            "workflow:workflow-1",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::ObjectiveConstraints,
            ReconstructionSourceClass::StoredCanonicalReference,
            "constraints:workflow-1",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::Decisions,
            ReconstructionSourceClass::HumanDecided,
            "decisions:workflow-1",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Redacted,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::CandidateEvidence,
            ReconstructionSourceClass::WindsObserved,
            "candidate-evidence:stage-1",
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Full,
        )
        .unwrap(),
        ReconstructionItem::new(
            ReconstructionCategory::PriorStageOutputs,
            ReconstructionSourceClass::StoredCanonicalReference,
            "prior-stage-output:none",
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
            "provider-private-state:unavailable",
            ReconstructionTransferState::Unavailable,
            ReconstructionContentState::Unavailable,
        )
        .unwrap(),
    ]
}

#[test]
fn t104_reconstruction_preview_is_deterministic_exact_and_side_effect_free() {
    let (home, store) = seeded_store("preview");
    let items = reconstruction_items();
    let mut reversed = items.clone();
    reversed.reverse();

    let before_bindings: i64 = store
        .connection
        .query_row("SELECT COUNT(*) FROM workflow_actor_bindings", [], |row| {
            row.get(0)
        })
        .unwrap();
    let before_reports: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_reconstruction_reports",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let first = store
        .preview_reconstruction("binding-1", "stage-1", "session-1", None, &items)
        .unwrap();
    let second = store
        .preview_reconstruction("binding-1", "stage-1", "session-1", None, &reversed)
        .unwrap();

    assert_eq!(first.canonical_json, second.canonical_json);
    assert_eq!(first.report.items.len(), ReconstructionCategory::ALL.len());
    assert_eq!(
        first
            .report
            .items
            .iter()
            .map(|item| item.category)
            .collect::<Vec<_>>(),
        ReconstructionCategory::ALL.to_vec()
    );
    let after_bindings: i64 = store
        .connection
        .query_row("SELECT COUNT(*) FROM workflow_actor_bindings", [], |row| {
            row.get(0)
        })
        .unwrap();
    let after_reports: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_reconstruction_reports",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        (before_bindings, before_reports),
        (after_bindings, after_reports)
    );

    drop(store);
    cleanup(home);
}

#[test]
fn t104_reconstruction_report_rejects_private_payload_duplicate_and_missing_context() {
    assert!(
        ReconstructionItem::new(
            ReconstructionCategory::ProviderPrivateState,
            ReconstructionSourceClass::Unavailable,
            "secret-provider-payload",
            ReconstructionTransferState::Unavailable,
            ReconstructionContentState::Unavailable,
        )
        .is_err()
    );

    let mut missing = reconstruction_items();
    missing.pop();
    assert!(build_reconstruction_preview("binding-1", "stage-1", &missing).is_err());

    let mut duplicate = reconstruction_items();
    duplicate[6] = duplicate[0].clone();
    assert!(build_reconstruction_preview("binding-1", "stage-1", &duplicate).is_err());
}

#[test]
fn t104_reconstructed_binding_is_atomic_immutable_and_restart_safe() {
    let (home, mut store) = seeded_store("reconstructed");
    let items = reconstruction_items();
    let preview = store
        .preview_reconstruction(
            "binding-reconstructed",
            "stage-1",
            "session-1",
            None,
            &items,
        )
        .unwrap();
    let stored = store
        .create_reconstructed_actor_binding(NewReconstructedActorBinding {
            binding_id: "binding-reconstructed",
            stage_run_id: "stage-1",
            winds_session_id: "session-1",
            runtime_binding_id: None,
            reconstruction_report_id: "report-1",
            items: &items,
            now_ms: 7,
        })
        .unwrap();
    assert_eq!(
        stored.continuation,
        WorkflowContinuationClass::Reconstructed
    );
    let report = stored.reconstruction_report.as_ref().unwrap();
    assert_eq!(report.canonical_json, preview.canonical_json);
    assert_eq!(report.report.binding_id, "binding-reconstructed");
    assert_eq!(report.report.stage_run_id, "stage-1");

    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_actor_bindings SET continuation_class = 'UNAVAILABLE' WHERE binding_id = 'binding-reconstructed'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_reconstruction_reports SET canonical_report_json = '{}' WHERE reconstruction_report_id = 'report-1'",
                [],
            )
            .is_err()
    );

    let second = store.create_reconstructed_actor_binding(NewReconstructedActorBinding {
        binding_id: "binding-rolled-back",
        stage_run_id: "stage-1",
        winds_session_id: "session-1",
        runtime_binding_id: None,
        reconstruction_report_id: "report-1",
        items: &items,
        now_ms: 8,
    });
    assert!(second.is_err());
    let rolled_back: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workflow_actor_bindings WHERE binding_id = 'binding-rolled-back'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(rolled_back, 0);
    drop(store);

    let store = Store::open(&home).unwrap();
    let reopened = store
        .load_workflow_actor_binding("binding-reconstructed")
        .unwrap();
    assert_eq!(
        reopened.continuation,
        WorkflowContinuationClass::Reconstructed
    );
    assert_eq!(
        reopened
            .reconstruction_report
            .as_ref()
            .unwrap()
            .canonical_json,
        preview.canonical_json
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t104_resume_candidate_never_becomes_resumed_and_restart_preserves_nonclaim() {
    let (home, store) = seeded_store("resume-nonclaim");
    let runtime = create_runtime_binding(&store, "runtime-1", "native-session-1");
    let resolution = RuntimeResumeResolution::Candidate(Box::new(runtime.clone()));
    let continuation = store
        .create_actor_binding_from_runtime_resolution(
            "binding-unproven",
            "stage-1",
            "session-1",
            &resolution,
            7,
        )
        .unwrap();
    assert_eq!(continuation, WorkflowContinuationClass::Unproven);
    assert_ne!(continuation, WorkflowContinuationClass::Resumed);
    drop(store);

    let store = Store::open(&home).unwrap();
    let reopened = store
        .load_workflow_actor_binding("binding-unproven")
        .unwrap();
    assert_eq!(reopened.continuation, WorkflowContinuationClass::Unproven);
    assert_eq!(reopened.runtime_binding_id.as_deref(), Some("runtime-1"));
    assert_eq!(
        store
            .load_runtime_session_binding("runtime-1")
            .unwrap()
            .ownership,
        RuntimeBindingOwnership::Unproven
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t104_ownership_loss_stale_ambiguous_and_unavailable_truth_fail_closed() {
    let (home, store) = seeded_store("fail-closed-resolution");
    let runtime = create_runtime_binding(&store, "runtime-1", "native-session-1");
    store
        .mark_runtime_binding_ownership_lost("runtime-1", 8)
        .unwrap();
    let lost = store.load_runtime_session_binding("runtime-1").unwrap();
    assert_eq!(lost.ownership, RuntimeBindingOwnership::OwnershipLost);
    let lost_resolution = RuntimeResumeResolution::Candidate(Box::new(lost));
    assert_eq!(
        store
            .create_actor_binding_from_runtime_resolution(
                "binding-lost",
                "stage-1",
                "session-1",
                &lost_resolution,
                9,
            )
            .unwrap(),
        WorkflowContinuationClass::OwnershipLost
    );

    for (binding_id, resolution) in [
        ("binding-stale", RuntimeResumeResolution::Stale),
        ("binding-unavailable", RuntimeResumeResolution::Unavailable),
        (
            "binding-ambiguous",
            RuntimeResumeResolution::Ambiguous(vec![runtime.clone(), runtime]),
        ),
    ] {
        assert_eq!(
            store
                .create_actor_binding_from_runtime_resolution(
                    binding_id,
                    "stage-1",
                    "session-1",
                    &resolution,
                    10,
                )
                .unwrap(),
            WorkflowContinuationClass::Unavailable
        );
        assert_eq!(
            store
                .load_workflow_actor_binding(binding_id)
                .unwrap()
                .continuation,
            WorkflowContinuationClass::Unavailable
        );
    }

    drop(store);
    cleanup(home);
}

#[test]
fn t104_mismatched_missing_malformed_and_forged_binding_truth_is_rejected() {
    let (home, mut store) = seeded_store("invalid-binding-truth");
    let runtime = create_runtime_binding(&store, "runtime-1", "native-session-1");
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-2",
                workstream_id: "workstream-1",
                display_name: "T104 second session",
            },
            7,
        )
        .unwrap();
    let resolution = RuntimeResumeResolution::Candidate(Box::new(runtime));
    assert!(
        store
            .create_actor_binding_from_runtime_resolution(
                "binding-session-mismatch",
                "stage-1",
                "session-2",
                &resolution,
                8,
            )
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, continuation_class, bound_unix_ms
             ) VALUES ('binding-missing-report', 'stage-1', 'session-1', 'RECONSTRUCTED', 9)",
            [],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_actor_binding("binding-missing-report")
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, continuation_class, bound_unix_ms
             ) VALUES ('binding-forged-resume', 'stage-1', 'session-1', 'RESUMED', 9)",
            [],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_actor_binding("binding-forged-resume")
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, continuation_class, bound_unix_ms
             ) VALUES ('binding-malformed-report', 'stage-1', 'session-1', 'RECONSTRUCTED', 9)",
            [],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_reconstruction_reports(
                reconstruction_report_id, binding_id, schema_version, canonical_report_json, created_unix_ms
             ) VALUES ('report-malformed', 'binding-malformed-report', 1, '{}', 9)",
            [],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_actor_binding("binding-malformed-report")
            .is_err()
    );

    let mismatched_preview =
        build_reconstruction_preview("different-binding", "stage-1", &reconstruction_items())
            .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, continuation_class, bound_unix_ms
             ) VALUES ('binding-mismatched-report', 'stage-1', 'session-1', 'RECONSTRUCTED', 9)",
            [],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_reconstruction_reports(
                reconstruction_report_id, binding_id, schema_version, canonical_report_json, created_unix_ms
             ) VALUES ('report-mismatch', 'binding-mismatched-report', 1, ?1, 9)",
            [&mismatched_preview.canonical_json],
        )
        .unwrap();
    assert!(
        store
            .load_workflow_actor_binding("binding-mismatched-report")
            .is_err()
    );

    let valid_items = reconstruction_items();
    let valid = store
        .create_reconstructed_actor_binding(NewReconstructedActorBinding {
            binding_id: "binding-valid",
            stage_run_id: "stage-1",
            winds_session_id: "session-1",
            runtime_binding_id: None,
            reconstruction_report_id: "report-valid",
            items: &valid_items,
            now_ms: 10,
        })
        .unwrap();
    assert_eq!(valid.continuation, WorkflowContinuationClass::Reconstructed);

    drop(store);
    cleanup(home);
}
