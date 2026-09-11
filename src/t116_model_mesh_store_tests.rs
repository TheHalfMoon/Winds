use super::{
    ModelMeshClaimSubject, NewModelMeshIdentityClaim, NewModelMeshTargetRequest, NewWindsSession,
    NewWorkspace, NewWorkstream, Store, expected_model_mesh_schema_objects,
    model_mesh_schema_objects,
};
use crate::agentic_authority::{load_model_mesh_approval, record_model_mesh_approval};
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeResumeResolution, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{StageRunIdentity, WorkflowRunIdentity};
use crate::model_mesh::{
    ContextDigest, ContinuityClass, ExactModelId, ExactProviderId, IdentityClaim,
    IdentityDimension, IdentitySourceClass, ModelMeshAuthorityEnvelopeV1,
    ModelMeshContinuityPermissionDescriptorV1, ModelMeshTargetDescriptorV1, TargetDimension,
    TargetRequest, TargetSelector,
};
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t116-{name}-{}-{sequence}",
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

fn runtime_fixture_path() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from(r"C:\winds-t116-codex.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("/tmp/winds-t116-codex")
    }
}

fn runtime_discovery() -> RuntimeDiscovery {
    let executable = runtime_fixture_path();
    RuntimeDiscovery {
        runtime: RuntimeKind::Codex,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 116,
            sha256: "a".repeat(64),
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("t116-runtime-1".into()),
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
                canonical_worktree_root: "/tmp/t116-workspace-1",
                git_common_dir: "/tmp/t116-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T116 workstream",
            },
            2,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "T116 session",
            },
            3,
        )
        .unwrap();
    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap(),
            4,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap(),
            None,
            5,
        )
        .unwrap();
    store
        .create_runtime_session_binding(
            "runtime-binding-1",
            "session-1",
            &runtime_discovery(),
            Some("native-session-1"),
            6,
        )
        .unwrap();
    let runtime = store
        .load_runtime_session_binding("runtime-binding-1")
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-1",
            "stage-1",
            "session-1",
            &RuntimeResumeResolution::Candidate(Box::new(runtime)),
            7,
        )
        .unwrap();
    (home, store)
}

fn seed_second_session_actor(store: &Store) {
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-2",
                workstream_id: "workstream-1",
                display_name: "T116 second session",
            },
            10,
        )
        .unwrap();
    store
        .create_runtime_session_binding(
            "runtime-binding-2",
            "session-2",
            &runtime_discovery(),
            Some("native-session-2"),
            11,
        )
        .unwrap();
    let runtime = store
        .load_runtime_session_binding("runtime-binding-2")
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-session-2",
            "stage-1",
            "session-2",
            &RuntimeResumeResolution::Candidate(Box::new(runtime)),
            12,
        )
        .unwrap();
}

fn target_descriptor(role: &str) -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-binding-1",
        "session-1",
        role,
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-t116").unwrap()),
    )
    .unwrap()
}

fn target_request(role: &str) -> TargetRequest {
    TargetRequest::new(target_descriptor(role), TargetSelector::Human).unwrap()
}

fn approve_target(store: &Store, approval_id: &str, request: &TargetRequest, now_ms: i64) {
    let envelope =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(request.descriptor()).unwrap();
    record_model_mesh_approval(store, approval_id, &envelope, now_ms).unwrap();
    let loaded = load_model_mesh_approval(store, approval_id).unwrap();
    assert_eq!(
        loaded.canonical_content_json,
        envelope.canonical_json().unwrap()
    );
}

fn create_target(store: &mut Store, target_request_id: &str) -> TargetRequest {
    let request = target_request("worker");
    approve_target(store, "selection-approval-1", &request, 8);
    store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id,
            request: &request,
            selection_approval_id: "selection-approval-1",
            created_unix_ms: 9,
        })
        .unwrap();
    request
}

fn sha256_hex(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[test]
fn t116_schema_inventory_is_exact_idempotent_and_partial_state_fails_closed() {
    let home = test_home("schema");
    let store = Store::open(&home).unwrap();
    let expected = expected_model_mesh_schema_objects().unwrap();
    let observed = model_mesh_schema_objects(&store.connection).unwrap();
    assert_eq!(observed, expected);
    assert_eq!(
        observed
            .values()
            .filter(|(kind, _, _)| kind == "table")
            .count(),
        4
    );
    assert_eq!(
        observed
            .values()
            .filter(|(kind, _, _)| kind == "index")
            .count(),
        6
    );
    assert_eq!(
        observed
            .values()
            .filter(|(kind, _, _)| kind == "trigger")
            .count(),
        14
    );

    let continuity_columns = store
        .connection
        .prepare("PRAGMA table_info(model_mesh_continuity_events)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(
        continuity_columns,
        vec![
            "continuity_event_id",
            "target_request_id",
            "source_actor_binding_id",
            "destination_actor_binding_id",
            "continuity_class",
            "context_digest",
            "completeness_state",
            "continuity_permission_digest",
            "authority_approval_id",
            "authority_claim",
            "created_unix_ms",
        ]
    );
    store
        .connection
        .execute_batch(include_str!("../migrations/0011_model_mesh_continuity.sql"))
        .unwrap();
    store.validate_model_mesh_schema().unwrap();

    store
        .connection
        .execute_batch("DROP INDEX idx_model_mesh_identity_claims_actor_time")
        .unwrap();
    let error = store.validate_model_mesh_schema().unwrap_err().to_string();
    assert!(error.contains("Model Mesh schema object inventory mismatch"));
    drop(store);
    let reopen_error = match Store::open(&home) {
        Ok(_) => panic!("partial Model Mesh schema must not be silently repaired"),
        Err(error) => error.to_string(),
    };
    assert!(reopen_error.contains("Model Mesh schema object inventory mismatch"));
    cleanup(home);
}

#[test]
fn t116_target_request_is_exact_approved_idempotent_append_only_and_restart_safe() {
    let (home, mut store) = seeded_store("target");
    let request = target_request("worker");
    approve_target(&store, "selection-approval-1", &request, 8);
    let first = store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id: "target-request-1",
            request: &request,
            selection_approval_id: "selection-approval-1",
            created_unix_ms: 9,
        })
        .unwrap();
    let replay = store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id: "target-request-1",
            request: &request,
            selection_approval_id: "selection-approval-1",
            created_unix_ms: 9,
        })
        .unwrap();
    assert_eq!(first, replay);
    assert_eq!(first.request, request);

    let moved = target_request("planner");
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "target-request-1",
                request: &moved,
                selection_approval_id: "selection-approval-1",
                created_unix_ms: 9,
            })
            .unwrap_err()
            .to_string()
            .contains("idempotency collision")
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE model_mesh_target_requests SET actor_role = 'PLANNER' WHERE target_request_id = 'target-request-1'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM model_mesh_target_requests WHERE target_request_id = 'target-request-1'",
                [],
            )
            .is_err()
    );
    drop(store);

    let reopened = Store::open(&home).unwrap();
    let persisted = reopened
        .load_model_mesh_target_request("target-request-1")
        .unwrap();
    assert_eq!(persisted, first);
    cleanup(home);
}

#[test]
fn t116_target_request_rejects_policy_cross_scope_wrong_and_generic_approval() {
    let (home, mut store) = seeded_store("target-negative");
    let request = target_request("worker");

    let policy =
        TargetRequest::new(request.descriptor().clone(), TargetSelector::ExplicitPolicy).unwrap();
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "policy-request",
                request: &policy,
                selection_approval_id: "missing",
                created_unix_ms: 9,
            })
            .unwrap_err()
            .to_string()
            .contains("not authorized")
    );

    let wrong_target = target_request("planner");
    approve_target(&store, "wrong-approval", &wrong_target, 8);
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "wrong-approved-request",
                request: &request,
                selection_approval_id: "wrong-approval",
                created_unix_ms: 9,
            })
            .is_err()
    );

    let generic_json = r#"{"schema_version":1,"workspace_id":"workspace-1","workstream_id":"workstream-1","session_id":"session-1","target_capability":"run"}"#;
    store
        .connection
        .execute(
            "INSERT INTO agentic_delegation_approvals(
                approval_id, workstream_id, session_id, workspace_id,
                content_digest, canonical_content_json, approved_unix_ms
             ) VALUES (?1, 'workstream-1', 'session-1', 'workspace-1', ?2, ?3, 8)",
            params!["generic-approval", sha256_hex(generic_json), generic_json],
        )
        .unwrap();
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "generic-request",
                request: &request,
                selection_approval_id: "generic-approval",
                created_unix_ms: 9,
            })
            .is_err()
    );

    let cross_stage = ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "different-stage",
        "actor-binding-1",
        "session-1",
        "worker",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-t116").unwrap()),
    )
    .unwrap();
    let cross_stage_request = TargetRequest::new(cross_stage, TargetSelector::Human).unwrap();
    approve_target(&store, "cross-stage-approval", &cross_stage_request, 8);
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "cross-stage-request",
                request: &cross_stage_request,
                selection_approval_id: "cross-stage-approval",
                created_unix_ms: 9,
            })
            .is_err()
    );

    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-2", "workspace-1", "workstream-1").unwrap(),
            10,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("workflow-2-stage", "workflow-2", "build", 1, None).unwrap(),
            None,
            11,
        )
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "workflow-2-actor",
            "workflow-2-stage",
            "session-1",
            &RuntimeResumeResolution::Unavailable,
            12,
        )
        .unwrap();
    let cross_workflow = ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "workflow-2-actor",
        "session-1",
        "worker",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-t116").unwrap()),
    )
    .unwrap();
    let cross_workflow_request = TargetRequest::new(cross_workflow, TargetSelector::Human).unwrap();
    approve_target(
        &store,
        "cross-workflow-approval",
        &cross_workflow_request,
        13,
    );
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "cross-workflow-request",
                request: &cross_workflow_request,
                selection_approval_id: "cross-workflow-approval",
                created_unix_ms: 14,
            })
            .is_err()
    );

    seed_second_session_actor(&store);
    let cross_session = ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-binding-session-2",
        "session-1",
        "worker",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-t116").unwrap()),
    )
    .unwrap();
    let cross_session_request = TargetRequest::new(cross_session, TargetSelector::Human).unwrap();
    approve_target(&store, "cross-session-approval", &cross_session_request, 13);
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "cross-session-request",
                request: &cross_session_request,
                selection_approval_id: "cross-session-approval",
                created_unix_ms: 14,
            })
            .is_err()
    );

    let other_binding_target = ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-binding-session-2",
        "session-2",
        "worker",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-t116").unwrap()),
    )
    .unwrap();
    let other_binding_request =
        TargetRequest::new(other_binding_target, TargetSelector::Human).unwrap();
    approve_target(&store, "other-binding-approval", &other_binding_request, 13);
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "wrong-binding-approval-request",
                request: &request,
                selection_approval_id: "other-binding-approval",
                created_unix_ms: 14,
            })
            .is_err()
    );
    cleanup(home);
}

#[test]
fn t116_identity_claims_are_subject_bound_runtime_bound_idempotent_and_append_only() {
    let (home, mut store) = seeded_store("claims");
    create_target(&mut store, "target-request-1");

    for forbidden_basis in [
        "API_KEY=super-secret",
        "Authorization bearer private-token",
        "provider-private-state:raw-payload",
    ] {
        assert!(
            IdentityClaim::new(
                IdentityDimension::Provider,
                Some("openai"),
                IdentitySourceClass::WindsLocallyObserved,
                Some(forbidden_basis),
            )
            .is_err()
        );
    }

    let requested_provider = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::HumanDecided,
        Some("explicit target request"),
    )
    .unwrap();
    let first = store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-request-provider",
            subject: ModelMeshClaimSubject::RequestTarget("target-request-1".into()),
            claim: &requested_provider,
            runtime_binding_id: None,
            observed_unix_ms: 10,
        })
        .unwrap();
    let replay = store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-request-provider",
            subject: ModelMeshClaimSubject::RequestTarget("target-request-1".into()),
            claim: &requested_provider,
            runtime_binding_id: None,
            observed_unix_ms: 10,
        })
        .unwrap();
    assert_eq!(first, replay);

    let actor_provider = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted structured local observation"),
    )
    .unwrap();
    let actor_stored = store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-actor-provider",
            subject: ModelMeshClaimSubject::Actor("actor-binding-1".into()),
            claim: &actor_provider,
            runtime_binding_id: Some("runtime-binding-1"),
            observed_unix_ms: 10,
        })
        .unwrap();

    assert!(
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: "claim-wrong-runtime",
                subject: ModelMeshClaimSubject::Actor("actor-binding-1".into()),
                claim: &actor_provider,
                runtime_binding_id: Some("missing-runtime-binding"),
                observed_unix_ms: 10,
            })
            .is_err()
    );
    seed_second_session_actor(&store);
    assert!(
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: "claim-valid-but-cross-session-runtime",
                subject: ModelMeshClaimSubject::Actor("actor-binding-1".into()),
                claim: &actor_provider,
                runtime_binding_id: Some("runtime-binding-2"),
                observed_unix_ms: 13,
            })
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_identity_claims(
                    identity_claim_id, target_request_id, claim_subject, dimension,
                    normalized_value, source_class, observation_basis, observed_unix_ms
                 ) VALUES (
                    'claim-secret-direct', 'target-request-1', 'REQUEST_TARGET', 'PROVIDER',
                    'openai', 'HUMAN_DECIDED', 'API_KEY=super-secret', 13
                 )",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_identity_claims(
                    identity_claim_id, target_request_id, claim_subject, dimension,
                    normalized_value, source_class, observation_basis, observed_unix_ms
                 ) VALUES (
                    'claim-private-direct', 'target-request-1', 'REQUEST_TARGET', 'PROVIDER',
                    'openai', 'HUMAN_DECIDED', 'provider-private-state:raw-payload', 13
                 )",
                [],
            )
            .is_err()
    );
    let different = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("different-provider"),
        IdentitySourceClass::HumanDecided,
        Some("explicit target request"),
    )
    .unwrap();
    assert!(
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: "claim-request-provider",
                subject: ModelMeshClaimSubject::RequestTarget("target-request-1".into()),
                claim: &different,
                runtime_binding_id: None,
                observed_unix_ms: 10,
            })
            .unwrap_err()
            .to_string()
            .contains("idempotency collision")
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE model_mesh_identity_claims SET source_class = 'AGENT_REPORTED' WHERE identity_claim_id = 'claim-request-provider'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM model_mesh_identity_claims WHERE identity_claim_id = 'claim-request-provider'",
                [],
            )
            .is_err()
    );
    drop(store);

    let reopened = Store::open(&home).unwrap();
    assert_eq!(
        reopened
            .load_model_mesh_identity_claim("claim-request-provider")
            .unwrap(),
        first
    );
    assert_eq!(
        reopened
            .load_model_mesh_identity_claim("claim-actor-provider")
            .unwrap(),
        actor_stored
    );
    cleanup(home);
}

#[test]
fn t116_continuity_schema_represents_both_authority_classes_and_role_specific_claims() {
    let (home, mut store) = seeded_store("continuity-valid");
    let request = create_target(&mut store, "target-request-1");
    let actor_claim = IdentityClaim::new(
        IdentityDimension::Runtime,
        Some("CODEX"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted runtime binding"),
    )
    .unwrap();
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "actor-runtime-claim",
            subject: ModelMeshClaimSubject::Actor("actor-binding-1".into()),
            claim: &actor_claim,
            runtime_binding_id: Some("runtime-binding-1"),
            observed_unix_ms: 10,
        })
        .unwrap();

    let context_digest = "b".repeat(64);
    let permission = ModelMeshContinuityPermissionDescriptorV1::new(
        "workflow-1",
        "stage-1",
        Some("actor-binding-1"),
        Some("actor-binding-1"),
        ContinuityClass::Handoff,
        request.target_descriptor_digest(),
        ContextDigest::exact(&context_digest).unwrap(),
    )
    .unwrap();
    let permission_digest = permission.digest().unwrap();
    let authority =
        ModelMeshAuthorityEnvelopeV1::for_continuity_permission(request.descriptor(), &permission)
            .unwrap();
    record_model_mesh_approval(&store, "continuity-approval-1", &authority, 11).unwrap();

    store
        .connection
        .execute(
            "INSERT INTO model_mesh_continuity_events(
                continuity_event_id, target_request_id, source_actor_binding_id,
                destination_actor_binding_id, continuity_class, context_digest,
                completeness_state, continuity_permission_digest,
                authority_approval_id, authority_claim, created_unix_ms
             ) VALUES (
                'continuity-required', 'target-request-1', 'actor-binding-1',
                'actor-binding-1', 'HANDOFF', ?1, 'COMPLETE', ?2,
                'continuity-approval-1', 'REQUIRED', 12
             )",
            params![context_digest, permission_digest],
        )
        .unwrap();
    for role in ["SOURCE", "DESTINATION"] {
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_identity_claims(
                    continuity_event_id, actor_role, identity_claim_id
                 ) VALUES ('continuity-required', ?1, 'actor-runtime-claim')",
                params![role],
            )
            .unwrap();
    }
    store
        .connection
        .execute(
            "INSERT INTO model_mesh_continuity_events(
                continuity_event_id, target_request_id, continuity_class,
                completeness_state, authority_claim, created_unix_ms
             ) VALUES (
                'continuity-observation', 'target-request-1', 'UNPROVEN',
                'UNAVAILABLE', 'NO_AUTHORITY_CLAIM', 13
             )",
            [],
        )
        .unwrap();

    assert_eq!(
        store
            .connection
            .query_row(
                "SELECT COUNT(*) FROM model_mesh_continuity_events",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
    assert!(
        store
            .connection
            .execute(
                "UPDATE model_mesh_continuity_events SET completeness_state = 'INCOMPLETE' WHERE continuity_event_id = 'continuity-required'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM model_mesh_continuity_identity_claims WHERE continuity_event_id = 'continuity-required'",
                [],
            )
            .is_err()
    );
    cleanup(home);
}

#[test]
fn t116_continuity_schema_rejects_bad_authority_digest_actor_role_and_lineage() {
    let (home, mut store) = seeded_store("continuity-negative");
    let request = create_target(&mut store, "target-request-1");
    let actor_claim = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted structured local observation"),
    )
    .unwrap();
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "actor-provider-claim",
            subject: ModelMeshClaimSubject::Actor("actor-binding-1".into()),
            claim: &actor_claim,
            runtime_binding_id: Some("runtime-binding-1"),
            observed_unix_ms: 10,
        })
        .unwrap();
    let request_claim = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::HumanDecided,
        Some("requested provider"),
    )
    .unwrap();
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "request-provider-claim",
            subject: ModelMeshClaimSubject::RequestTarget("target-request-1".into()),
            claim: &request_claim,
            runtime_binding_id: None,
            observed_unix_ms: 10,
        })
        .unwrap();

    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, continuity_class,
                    completeness_state, continuity_permission_digest,
                    authority_claim, created_unix_ms
                 ) VALUES ('missing-approval', 'target-request-1', 'HANDOFF',
                           'COMPLETE', ?1, 'REQUIRED', 11)",
                params!["c".repeat(64)],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, continuity_class,
                    completeness_state, continuity_permission_digest,
                    authority_approval_id, authority_claim, created_unix_ms
                 ) VALUES ('orphan-approval', 'target-request-1', 'HANDOFF',
                           'COMPLETE', ?1, 'missing-approval-id', 'REQUIRED', 11)",
                params!["c".repeat(64)],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, continuity_class,
                    completeness_state, continuity_permission_digest,
                    authority_approval_id, authority_claim, created_unix_ms
                 ) VALUES ('malformed-permission-digest', 'target-request-1', 'HANDOFF',
                           'COMPLETE', 'NOT-A-SHA256', 'selection-approval-1', 'REQUIRED', 11)",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, continuity_class,
                    completeness_state, context_digest, authority_claim, created_unix_ms
                 ) VALUES ('bad-digest', 'target-request-1', 'UNPROVEN',
                           'INCOMPLETE', 'NOT-A-DIGEST', 'NO_AUTHORITY_CLAIM', 11)",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, continuity_class,
                    completeness_state, continuity_permission_digest,
                    authority_approval_id, authority_claim, created_unix_ms
                 ) VALUES ('observation-with-authority', 'target-request-1', 'UNPROVEN',
                           'UNAVAILABLE', ?1, 'selection-approval-1',
                           'NO_AUTHORITY_CLAIM', 11)",
                params!["d".repeat(64)],
            )
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO model_mesh_continuity_events(
                continuity_event_id, target_request_id, source_actor_binding_id,
                continuity_class, completeness_state, authority_claim, created_unix_ms
             ) VALUES ('role-event', 'target-request-1', 'actor-binding-1',
                       'UNPROVEN', 'INCOMPLETE', 'NO_AUTHORITY_CLAIM', 11)",
            [],
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_identity_claims(
                    continuity_event_id, actor_role, identity_claim_id
                 ) VALUES ('role-event', 'SOURCE', 'request-provider-claim')",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_identity_claims(
                    continuity_event_id, actor_role, identity_claim_id
                 ) VALUES ('role-event', 'DESTINATION', 'actor-provider-claim')",
                [],
            )
            .is_err()
    );

    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-peer",
            "stage-1",
            "session-1",
            &RuntimeResumeResolution::Unavailable,
            12,
        )
        .unwrap();
    let peer_claim = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted structured local observation"),
    )
    .unwrap();
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "peer-provider-claim",
            subject: ModelMeshClaimSubject::Actor("actor-binding-peer".into()),
            claim: &peer_claim,
            runtime_binding_id: None,
            observed_unix_ms: 13,
        })
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_identity_claims(
                    continuity_event_id, actor_role, identity_claim_id
                 ) VALUES ('role-event', 'SOURCE', 'peer-provider-claim')",
                [],
            )
            .is_err()
    );

    store
        .create_stage_run(
            &StageRunIdentity::new("unrelated-stage", "workflow-1", "other", 1, None).unwrap(),
            None,
            12,
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, continuation_class, bound_unix_ms
             ) VALUES ('unrelated-actor', 'unrelated-stage', 'session-1', 'UNAVAILABLE', 13)",
            [],
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, source_actor_binding_id,
                    continuity_class, completeness_state, authority_claim, created_unix_ms
                 ) VALUES ('bad-lineage', 'target-request-1', 'unrelated-actor',
                           'UNPROVEN', 'INCOMPLETE', 'NO_AUTHORITY_CLAIM', 14)",
                [],
            )
            .is_err()
    );

    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("other-workflow", "workspace-1", "workstream-1").unwrap(),
            15,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("other-workflow-stage", "other-workflow", "build", 1, None)
                .unwrap(),
            None,
            16,
        )
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "other-workflow-actor",
            "other-workflow-stage",
            "session-1",
            &RuntimeResumeResolution::Unavailable,
            17,
        )
        .unwrap();
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO model_mesh_continuity_events(
                    continuity_event_id, target_request_id, source_actor_binding_id,
                    continuity_class, completeness_state, authority_claim, created_unix_ms
                 ) VALUES ('cross-workflow-lineage', 'target-request-1', 'other-workflow-actor',
                           'UNPROVEN', 'INCOMPLETE', 'NO_AUTHORITY_CLAIM', 18)",
                [],
            )
            .is_err()
    );

    assert_eq!(request.descriptor().actor_binding_id(), "actor-binding-1");
    cleanup(home);
}
