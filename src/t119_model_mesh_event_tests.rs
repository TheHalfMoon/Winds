use crate::agentic_authority::record_model_mesh_approval;
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeResumeResolution, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{StageRunIdentity, WorkflowRunIdentity};
use crate::model_mesh::{
    ContinuityAuthorityClaim, ContinuityClass, CurrentAuthorityTruth, IdentityClaim,
    IdentityDimension, IdentitySourceClass, ModelMeshAuthorityEnvelopeV1,
    ModelMeshContinuityActorV1, ModelMeshContinuityContextInput, ModelMeshContinuityReferenceV1,
    ModelMeshTargetDescriptorV1, TargetDimension, TargetRequest, TargetSelector, adapt_actor_scope,
    build_model_mesh_continuity_context,
};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshContinuityEvent, NewModelMeshIdentityClaim,
    NewModelMeshTargetRequest, NewWindsSession, NewWorkspace, NewWorkstream, Store,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t119-{name}-{}-{sequence}",
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

fn runtime_discovery(runtime: RuntimeKind, suffix: &str) -> RuntimeDiscovery {
    let executable = if cfg!(windows) {
        PathBuf::from(format!(r"C:\winds-t119-{suffix}.exe"))
    } else {
        PathBuf::from(format!("/tmp/winds-t119-{suffix}"))
    };
    RuntimeDiscovery {
        runtime,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 119,
            sha256: match runtime {
                RuntimeKind::Codex => "a".repeat(64),
                RuntimeKind::Claude => "b".repeat(64),
            },
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some(format!("t119-{suffix}-1")),
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

fn seeded_store(name: &str) -> (PathBuf, Store) {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t119-workspace-1",
                git_common_dir: "/tmp/t119-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T119 workstream",
            },
            2,
        )
        .unwrap();
    for (session_id, now) in [("session-source", 3), ("session-destination", 4)] {
        store
            .create_winds_session(
                NewWindsSession {
                    session_id,
                    workstream_id: "workstream-1",
                    display_name: session_id,
                },
                now,
            )
            .unwrap();
    }
    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap(),
            5,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap(),
            None,
            6,
        )
        .unwrap();
    for (binding_id, session_id, native_id, now) in [
        ("runtime-source", "session-source", "native-source", 7),
        (
            "runtime-destination",
            "session-destination",
            "native-destination",
            8,
        ),
    ] {
        store
            .create_runtime_session_binding(
                binding_id,
                session_id,
                &runtime_discovery(RuntimeKind::Codex, binding_id),
                Some(native_id),
                now,
            )
            .unwrap();
        let runtime = store.load_runtime_session_binding(binding_id).unwrap();
        let actor_id = if binding_id == "runtime-source" {
            "actor-source"
        } else {
            "actor-destination"
        };
        store
            .create_actor_binding_from_runtime_resolution(
                actor_id,
                "stage-1",
                session_id,
                &RuntimeResumeResolution::Candidate(Box::new(runtime)),
                now + 2,
            )
            .unwrap();
    }
    (home, store)
}

fn target_descriptor() -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-destination",
        "session-destination",
        "WORKER",
        RuntimeKind::Codex,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
    .unwrap()
}

fn seed_target(store: &mut Store) -> TargetRequest {
    let request = TargetRequest::new(target_descriptor(), TargetSelector::Human).unwrap();
    let envelope =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(request.descriptor()).unwrap();
    record_model_mesh_approval(store, "selection-approval", &envelope, 11).unwrap();
    store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id: "target-request-1",
            request: &request,
            selection_approval_id: "selection-approval",
            created_unix_ms: 12,
        })
        .unwrap();
    request
}

fn continuity_actor(store: &Store, actor_id: &str) -> ModelMeshContinuityActorV1 {
    let actor = store.load_workflow_actor_binding(actor_id).unwrap();
    let runtime_id = actor.runtime_binding_id.as_deref().unwrap();
    let runtime = store.load_runtime_session_binding(runtime_id).unwrap();
    let stage = store.load_stage_run(&actor.stage_run_id).unwrap();
    let workflow = store
        .load_workflow_run(&stage.identity.workflow_run_id)
        .unwrap();
    let session = store.load_winds_session(&actor.winds_session_id).unwrap();
    let scope = adapt_actor_scope(
        &workflow.identity,
        &stage.identity,
        &session,
        &actor,
        Some(&runtime),
    )
    .unwrap();
    ModelMeshContinuityActorV1::from_actor_scope(&scope, &runtime).unwrap()
}

fn reference(id: &str, identity: &str) -> ModelMeshContinuityReferenceV1 {
    ModelMeshContinuityReferenceV1::new(id, identity).unwrap()
}

fn continuity_context(
    store: &Store,
    target: &ModelMeshTargetDescriptorV1,
    class: ContinuityClass,
    authority: CurrentAuthorityTruth,
    evidence_identity: &str,
) -> crate::model_mesh::ModelMeshContinuityContextV1 {
    let source = continuity_actor(store, "actor-source");
    let destination = continuity_actor(store, "actor-destination");
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(target).unwrap();
    let candidates = vec![reference("candidate.current", "oid:aaaa/tree:bbbb")];
    let artifacts = vec![reference("artifact.binary", "sha256:cccc")];
    let evidence = vec![reference("evidence.quality", evidence_identity)];
    let (source_actor, destination_actor) = match class {
        ContinuityClass::Unproven | ContinuityClass::Unavailable => {
            (Some(&destination), Some(&destination))
        }
        _ => (Some(&source), Some(&destination)),
    };
    build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target,
        continuity_class: class,
        source_actor,
        destination_actor,
        candidate_references: &candidates,
        artifact_references: &artifacts,
        evidence_references: &evidence,
        selection_approval: &approval,
        current_authority: authority,
        reconstruction_report: None,
        markers: &[],
    })
    .unwrap()
}

fn create_actor_claims(
    store: &mut Store,
    actor_id: &str,
    runtime_id: &str,
    prefix: &str,
) -> Vec<String> {
    let claims = [
        (
            "runtime",
            IdentityClaim::new(
                IdentityDimension::Runtime,
                Some("CODEX"),
                IdentitySourceClass::WindsLocallyObserved,
                Some("accepted T119 runtime observation"),
            )
            .unwrap(),
        ),
        (
            "provider",
            IdentityClaim::new(
                IdentityDimension::Provider,
                None,
                IdentitySourceClass::Unavailable,
                Some("provider identity unavailable"),
            )
            .unwrap(),
        ),
        (
            "model",
            IdentityClaim::new(
                IdentityDimension::Model,
                None,
                IdentitySourceClass::Unavailable,
                Some("model identity unavailable"),
            )
            .unwrap(),
        ),
    ];
    let mut ids = Vec::new();
    for (suffix, claim) in claims {
        let id = format!("claim-{prefix}-{suffix}");
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: &id,
                subject: ModelMeshClaimSubject::Actor(actor_id.to_owned()),
                claim: &claim,
                runtime_binding_id: Some(runtime_id),
                observed_unix_ms: 13,
            })
            .unwrap();
        ids.push(id);
    }
    ids
}

fn approve_continuity(
    store: &Store,
    approval_id: &str,
    target: &ModelMeshTargetDescriptorV1,
    context: &crate::model_mesh::ModelMeshContinuityContextV1,
    now_ms: i64,
) {
    let permission = context.permission_descriptor(target).unwrap();
    let envelope =
        ModelMeshAuthorityEnvelopeV1::for_continuity_permission(target, &permission).unwrap();
    record_model_mesh_approval(store, approval_id, &envelope, now_ms).unwrap();
}

#[test]
fn t119_required_event_is_exact_atomic_idempotent_semantic_replay_safe_and_restart_safe() {
    let (home, mut store) = seeded_store("required");
    let request = seed_target(&mut store);
    let source_claims = create_actor_claims(&mut store, "actor-source", "runtime-source", "source");
    let destination_claims = create_actor_claims(
        &mut store,
        "actor-destination",
        "runtime-destination",
        "destination",
    );
    let context = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Allowed,
        "run:119/head:exact",
    );
    approve_continuity(
        &store,
        "continuity-approval",
        request.descriptor(),
        &context,
        14,
    );
    let source_refs = source_claims.iter().map(String::as_str).collect::<Vec<_>>();
    let destination_refs = destination_claims
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let first = store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-required-1",
            target_request_id: "target-request-1",
            context: &context,
            authority_claim: ContinuityAuthorityClaim::Required,
            authority_approval_id: Some("continuity-approval"),
            source_identity_claim_ids: &source_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 15,
        })
        .unwrap();
    assert_eq!(first.continuity_class, ContinuityClass::Reassigned);
    assert_eq!(first.authority_claim, ContinuityAuthorityClaim::Required);
    assert!(first.context_digest.is_some());
    assert!(first.continuity_permission_digest.is_some());
    assert_eq!(
        first.authority_approval_id.as_deref(),
        Some("continuity-approval")
    );

    let replay = store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-required-1",
            target_request_id: "target-request-1",
            context: &context,
            authority_claim: ContinuityAuthorityClaim::Required,
            authority_approval_id: Some("continuity-approval"),
            source_identity_claim_ids: &source_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 15,
        })
        .unwrap();
    assert_eq!(first, replay);

    let semantic_replay = store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-required-replay-new-id",
            target_request_id: "target-request-1",
            context: &context,
            authority_claim: ContinuityAuthorityClaim::Required,
            authority_approval_id: Some("continuity-approval"),
            source_identity_claim_ids: &source_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 16,
        })
        .unwrap();
    assert_eq!(semantic_replay.continuity_event_id, "event-required-1");

    approve_continuity(
        &store,
        "continuity-approval-reissued",
        request.descriptor(),
        &context,
        16,
    );
    let reapproved_replay = store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-required-reapproved",
            target_request_id: "target-request-1",
            context: &context,
            authority_claim: ContinuityAuthorityClaim::Required,
            authority_approval_id: Some("continuity-approval-reissued"),
            source_identity_claim_ids: &source_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 17,
        })
        .unwrap();
    assert_eq!(reapproved_replay.continuity_event_id, "event-required-1");
    assert_eq!(
        store
            .list_model_mesh_continuity_events_for_target_request("target-request-1")
            .unwrap()
            .len(),
        1
    );

    for claim_id in ["claim-source-provider", "claim-source-model"] {
        let claim = store.load_model_mesh_identity_claim(claim_id).unwrap();
        assert_eq!(claim.claim.source, IdentitySourceClass::Unavailable);
        assert_eq!(claim.claim.value, None);
    }
    drop(store);

    let reopened = Store::open(&home).unwrap();
    assert_eq!(
        reopened
            .load_model_mesh_continuity_event("event-required-1")
            .unwrap(),
        first
    );
    drop(reopened);
    cleanup(home);
}

#[test]
fn t119_no_authority_observation_is_bounded_and_never_overrides_current_authority() {
    let (home, mut store) = seeded_store("no-authority");
    let request = seed_target(&mut store);
    let destination_claims = create_actor_claims(
        &mut store,
        "actor-destination",
        "runtime-destination",
        "destination",
    );
    let destination_refs = destination_claims
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let unproven = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Unproven,
        CurrentAuthorityTruth::Denied,
        "run:119/unproven",
    );
    let observed = store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-unproven",
            target_request_id: "target-request-1",
            context: &unproven,
            authority_claim: ContinuityAuthorityClaim::NoAuthorityClaim,
            authority_approval_id: None,
            source_identity_claim_ids: &destination_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 14,
        })
        .unwrap();
    assert_eq!(
        observed.authority_claim,
        ContinuityAuthorityClaim::NoAuthorityClaim
    );
    assert!(observed.continuity_permission_digest.is_none());
    assert!(observed.authority_approval_id.is_none());

    let reassigned = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Allowed,
        "run:119/reassigned",
    );
    assert!(
        store
            .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
                continuity_event_id: "event-reassigned-without-authority",
                target_request_id: "target-request-1",
                context: &reassigned,
                authority_claim: ContinuityAuthorityClaim::NoAuthorityClaim,
                authority_approval_id: None,
                source_identity_claim_ids: &[],
                destination_identity_claim_ids: &destination_refs,
                created_unix_ms: 15,
            })
            .unwrap_err()
            .to_string()
            .contains("only UNAVAILABLE or UNPROVEN")
    );

    let denied_reassigned = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Denied,
        "run:119/denied",
    );
    approve_continuity(
        &store,
        "denied-context-approval",
        request.descriptor(),
        &denied_reassigned,
        15,
    );
    assert!(
        store
            .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
                continuity_event_id: "event-denied-required",
                target_request_id: "target-request-1",
                context: &denied_reassigned,
                authority_claim: ContinuityAuthorityClaim::Required,
                authority_approval_id: Some("denied-context-approval"),
                source_identity_claim_ids: &[],
                destination_identity_claim_ids: &destination_refs,
                created_unix_ms: 16,
            })
            .unwrap_err()
            .to_string()
            .contains("cannot override a denied current authority ceiling")
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t119_wrong_role_claim_and_different_context_permission_fail_closed() {
    let (home, mut store) = seeded_store("negative");
    let request = seed_target(&mut store);
    let source_claims = create_actor_claims(&mut store, "actor-source", "runtime-source", "source");
    let destination_claims = create_actor_claims(
        &mut store,
        "actor-destination",
        "runtime-destination",
        "destination",
    );
    let source_refs = source_claims.iter().map(String::as_str).collect::<Vec<_>>();
    let destination_refs = destination_claims
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let approved_context = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Allowed,
        "run:119/approved-context",
    );
    approve_continuity(
        &store,
        "continuity-approval",
        request.descriptor(),
        &approved_context,
        14,
    );

    assert!(
        store
            .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
                continuity_event_id: "event-wrong-role",
                target_request_id: "target-request-1",
                context: &approved_context,
                authority_claim: ContinuityAuthorityClaim::Required,
                authority_approval_id: Some("continuity-approval"),
                source_identity_claim_ids: &destination_refs,
                destination_identity_claim_ids: &destination_refs,
                created_unix_ms: 15,
            })
            .unwrap_err()
            .to_string()
            .contains("does not belong to the exact source actor")
    );

    let moved_context = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Allowed,
        "run:119/different-context",
    );
    assert_ne!(
        approved_context.digest().unwrap(),
        moved_context.digest().unwrap()
    );
    assert!(
        store
            .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
                continuity_event_id: "event-wrong-context-approval",
                target_request_id: "target-request-1",
                context: &moved_context,
                authority_claim: ContinuityAuthorityClaim::Required,
                authority_approval_id: Some("continuity-approval"),
                source_identity_claim_ids: &source_refs,
                destination_identity_claim_ids: &destination_refs,
                created_unix_ms: 15,
            })
            .unwrap_err()
            .to_string()
            .contains("lacks exact content-bound permission approval")
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t119_unproven_history_remains_after_later_authorized_continuity() {
    let (home, mut store) = seeded_store("history");
    let request = seed_target(&mut store);
    let source_claims = create_actor_claims(&mut store, "actor-source", "runtime-source", "source");
    let destination_claims = create_actor_claims(
        &mut store,
        "actor-destination",
        "runtime-destination",
        "destination",
    );
    let source_refs = source_claims.iter().map(String::as_str).collect::<Vec<_>>();
    let destination_refs = destination_claims
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    let unproven = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Unproven,
        CurrentAuthorityTruth::Denied,
        "run:119/history-unproven",
    );
    store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-history-unproven",
            target_request_id: "target-request-1",
            context: &unproven,
            authority_claim: ContinuityAuthorityClaim::NoAuthorityClaim,
            authority_approval_id: None,
            source_identity_claim_ids: &destination_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 14,
        })
        .unwrap();

    let reassigned = continuity_context(
        &store,
        request.descriptor(),
        ContinuityClass::Reassigned,
        CurrentAuthorityTruth::Allowed,
        "run:119/history-authorized",
    );
    approve_continuity(
        &store,
        "history-continuity-approval",
        request.descriptor(),
        &reassigned,
        15,
    );
    store
        .create_model_mesh_continuity_event(NewModelMeshContinuityEvent {
            continuity_event_id: "event-history-authorized",
            target_request_id: "target-request-1",
            context: &reassigned,
            authority_claim: ContinuityAuthorityClaim::Required,
            authority_approval_id: Some("history-continuity-approval"),
            source_identity_claim_ids: &source_refs,
            destination_identity_claim_ids: &destination_refs,
            created_unix_ms: 16,
        })
        .unwrap();

    let events = store
        .list_model_mesh_continuity_events_for_target_request("target-request-1")
        .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].continuity_class, ContinuityClass::Unproven);
    assert_eq!(events[1].continuity_class, ContinuityClass::Reassigned);
    assert_eq!(
        events[0].authority_claim,
        ContinuityAuthorityClaim::NoAuthorityClaim
    );
    assert_eq!(
        events[1].authority_claim,
        ContinuityAuthorityClaim::Required
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t119_actor_outside_target_stage_lineage_is_rejected_at_frozen_schema_boundary() {
    let (home, mut store) = seeded_store("cross-lineage");
    seed_target(&mut store);

    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-foreign", "workspace-1", "workstream-1").unwrap(),
            20,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-foreign", "workflow-foreign", "build", 1, None).unwrap(),
            None,
            21,
        )
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-foreign",
            "stage-foreign",
            "session-source",
            &RuntimeResumeResolution::Unavailable,
            22,
        )
        .unwrap();

    let error = store
        .connection
        .execute(
            "INSERT INTO model_mesh_continuity_events(
                continuity_event_id, target_request_id, source_actor_binding_id,
                destination_actor_binding_id, continuity_class, context_digest,
                completeness_state, continuity_permission_digest, authority_approval_id,
                authority_claim, created_unix_ms
             ) VALUES (
                'event-cross-lineage', 'target-request-1', 'actor-foreign',
                'actor-destination', 'UNPROVEN', ?1, 'COMPLETE', NULL, NULL,
                'NO_AUTHORITY_CLAIM', 23
             )",
            rusqlite::params!["d".repeat(64)],
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outside canonical target StageRun lineage")
    );
    assert!(
        store
            .list_model_mesh_continuity_events_for_target_request("target-request-1")
            .unwrap()
            .is_empty()
    );
    drop(store);
    cleanup(home);
}
