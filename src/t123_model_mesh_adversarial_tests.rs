use crate::agentic_authority::{
    StoredApproval, record_model_mesh_approval, revalidate_loaded_model_mesh_approval,
};
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeResumeResolution, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{StageRunIdentity, WorkflowRunIdentity};
use crate::model_mesh::{
    ApprovalApplicability, AuthenticationTruth, CapabilityTruth, ContinuityClass,
    ContinuityContextCompleteness, ContinuityMaterialState, CurrentAuthorityTruth,
    DriftApplicability, ExactModelId, ExactProviderId, IdentityClaim, IdentityDimension,
    IdentitySourceClass, ModelMeshAuthorityEnvelopeV1, ModelMeshBaselineDrift,
    ModelMeshContinuityContextInput, ModelMeshContinuityMarkerV1, ModelMeshContinuityReferenceV1,
    ModelMeshDriftEvaluation, ModelMeshEvidenceFreshnessProjection, ModelMeshProjectionTruth,
    ModelMeshTargetDescriptorV1, ModelMeshTargetProjectionInput, SourceRequirements,
    TargetDimension, TargetRequest, TargetResolution, TargetResolverInput, TargetSelector,
    build_model_mesh_continuity_context, project_model_mesh_target, resolve_target,
};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshIdentityClaim, NewModelMeshTargetRequest, NewWindsSession,
    NewWorkspace, NewWorkstream, Store,
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
        "winds-t123-{name}-{}-{sequence}",
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

fn runtime_discovery() -> RuntimeDiscovery {
    let executable = if cfg!(windows) {
        PathBuf::from(r"C:\winds-t123-codex.exe")
    } else {
        PathBuf::from("/tmp/winds-t123-codex")
    };
    RuntimeDiscovery {
        runtime: RuntimeKind::Codex,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 123,
            sha256: "a".repeat(64),
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("t123-runtime-1".into()),
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
                canonical_worktree_root: "/tmp/t123-workspace-1",
                git_common_dir: "/tmp/t123-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T123 workstream",
            },
            2,
        )
        .unwrap();
    seed_session_actor(
        &store,
        "session-1",
        "runtime-binding-1",
        "actor-binding-1",
        3,
    );
    store
        .create_workflow_run(
            &WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap(),
            7,
        )
        .unwrap();
    store
        .create_stage_run(
            &StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap(),
            None,
            8,
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
            9,
        )
        .unwrap();
    (home, store)
}

fn seed_session_actor(
    store: &Store,
    session_id: &str,
    runtime_binding_id: &str,
    _actor_binding_id: &str,
    now_ms: i64,
) {
    store
        .create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id: "workstream-1",
                display_name: session_id,
            },
            now_ms,
        )
        .unwrap();
    store
        .create_runtime_session_binding(
            runtime_binding_id,
            session_id,
            &runtime_discovery(),
            Some(&format!("native-{session_id}")),
            now_ms + 1,
        )
        .unwrap();
}

fn seed_second_actor(store: &Store) {
    seed_session_actor(
        store,
        "session-2",
        "runtime-binding-2",
        "actor-binding-2",
        10,
    );
    let runtime = store
        .load_runtime_session_binding("runtime-binding-2")
        .unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-binding-2",
            "stage-1",
            "session-2",
            &RuntimeResumeResolution::Candidate(Box::new(runtime)),
            12,
        )
        .unwrap();
}

fn target(
    actor: &str,
    session: &str,
    role: &str,
    provider: TargetDimension<ExactProviderId>,
    model: TargetDimension<ExactModelId>,
) -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        actor,
        session,
        role,
        RuntimeKind::Codex,
        provider,
        model,
    )
    .unwrap()
}

fn unspecified_target(actor: &str, session: &str, role: &str) -> ModelMeshTargetDescriptorV1 {
    target(
        actor,
        session,
        role,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
}

fn exact_target() -> ModelMeshTargetDescriptorV1 {
    target(
        "actor-binding-1",
        "session-1",
        "WORKER",
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
    )
}

fn winds_claim(dimension: IdentityDimension, value: &str) -> IdentityClaim {
    IdentityClaim::new(
        dimension,
        Some(value),
        IdentitySourceClass::WindsLocallyObserved,
        Some("T123 accepted local observation"),
    )
    .unwrap()
}

fn resolve(
    request: &TargetRequest,
    claims: &[IdentityClaim],
    authority: CurrentAuthorityTruth,
) -> TargetResolution {
    resolve_target(&TargetResolverInput {
        request,
        claims,
        source_requirements: SourceRequirements::winds_observed(),
        stale: false,
        authentication: AuthenticationTruth::Ready,
        capability: CapabilityTruth::Available,
        approval: ApprovalApplicability::Exact,
        current_authority: authority,
    })
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn persist_target(
    store: &mut Store,
    target_request_id: &str,
    request: &TargetRequest,
    approval_id: &str,
    now_ms: i64,
) {
    let envelope =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(request.descriptor()).unwrap();
    record_model_mesh_approval(store, approval_id, &envelope, now_ms - 1).unwrap();
    store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id,
            request,
            selection_approval_id: approval_id,
            created_unix_ms: now_ms,
        })
        .unwrap();
}

fn exact_drift() -> ModelMeshDriftEvaluation {
    ModelMeshDriftEvaluation {
        runtime: DriftApplicability::Applicable,
        provider: DriftApplicability::Applicable,
        model: DriftApplicability::Applicable,
        native_session: DriftApplicability::Unproven,
        scope: DriftApplicability::Applicable,
        baseline: ModelMeshBaselineDrift {
            applicability: DriftApplicability::NotApplicable,
            evaluations: vec![],
        },
        approval: DriftApplicability::Applicable,
        current_authority: DriftApplicability::Applicable,
        overall: DriftApplicability::Applicable,
    }
}

#[test]
fn t123_forged_identity_unicode_case_whitespace_and_oversize_never_false_match() {
    assert_eq!(ExactProviderId::new(" openai ").unwrap().as_str(), "openai");
    assert_ne!(
        ExactProviderId::new("openai").unwrap(),
        ExactProviderId::new("OpenAI").unwrap()
    );
    assert_ne!(
        ExactProviderId::new("openai").unwrap(),
        ExactProviderId::new("openаi").unwrap()
    );
    assert!(ExactProviderId::new(&"x".repeat(257)).is_err());
    assert!(ExactModelId::new(&"m".repeat(257)).is_err());

    let request = TargetRequest::new(exact_target(), TargetSelector::Human).unwrap();
    let forged = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("openai"),
            IdentitySourceClass::AgentReported,
            Some("model prose says provider identity is proven"),
        )
        .unwrap(),
        IdentityClaim::new(
            IdentityDimension::Model,
            Some("model-a"),
            IdentitySourceClass::AgentReported,
            Some("agent report"),
        )
        .unwrap(),
    ];
    assert_eq!(
        resolve(&request, &forged, CurrentAuthorityTruth::Allowed),
        TargetResolution::Unknown
    );

    let wrong_case = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        winds_claim(IdentityDimension::Provider, "OpenAI"),
        winds_claim(IdentityDimension::Model, "model-a"),
    ];
    assert_eq!(
        resolve(&request, &wrong_case, CurrentAuthorityTruth::Allowed),
        TargetResolution::Unavailable
    );

    let conflicting = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        winds_claim(IdentityDimension::Provider, "openai"),
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("anthropic"),
            IdentitySourceClass::AgentReported,
            Some("tempting alternate"),
        )
        .unwrap(),
        winds_claim(IdentityDimension::Model, "model-a"),
    ];
    assert_eq!(
        resolve(&request, &conflicting, CurrentAuthorityTruth::Allowed),
        TargetResolution::Conflict
    );

    let unspecified = TargetRequest::new(
        unspecified_target("actor-binding-1", "session-1", "WORKER"),
        TargetSelector::Human,
    )
    .unwrap();
    let runtime_only = vec![winds_claim(
        IdentityDimension::Runtime,
        RuntimeKind::Codex.as_str(),
    )];
    assert_eq!(
        resolve(&unspecified, &runtime_only, CurrentAuthorityTruth::Allowed),
        TargetResolution::ExactMatch
    );
}

#[test]
fn t123_authority_replay_forged_json_wrong_scope_and_stale_digest_fail_closed() {
    let expected = ModelMeshAuthorityEnvelopeV1::for_target_selection(&exact_target()).unwrap();
    let canonical = expected.canonical_json().unwrap();
    let exact = StoredApproval {
        approval_id: "approval-exact".into(),
        workstream_id: "workstream-1".into(),
        session_id: "session-1".into(),
        workspace_id: "workspace-1".into(),
        content_digest: sha256(&canonical),
        canonical_content_json: canonical.clone(),
        approved_unix_ms: 10,
    };
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&exact, &expected),
        ApprovalApplicability::Exact
    );

    let mut stale = exact.clone();
    stale.content_digest = "0".repeat(64);
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&stale, &expected),
        ApprovalApplicability::Stale
    );

    for forged_json in [
        format!(" {canonical}"),
        canonical.replace("\"TARGET_SELECTION\"", "\"CONTINUITY_PERMISSION\""),
        r#"{"schema_version":1,"decision":"ACCEPTED","authority":"ALLOW"}"#.to_owned(),
    ] {
        let forged = StoredApproval {
            approval_id: "approval-forged".into(),
            content_digest: sha256(&forged_json),
            canonical_content_json: forged_json,
            ..exact.clone()
        };
        assert_eq!(
            revalidate_loaded_model_mesh_approval(&forged, &expected),
            ApprovalApplicability::Mismatch
        );
    }

    for wrong in [
        target(
            "actor-binding-1",
            "session-1",
            "REVIEWER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
        target(
            "actor-binding-2",
            "session-2",
            "WORKER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
    ] {
        let wrong_envelope = ModelMeshAuthorityEnvelopeV1::for_target_selection(&wrong).unwrap();
        assert_eq!(
            revalidate_loaded_model_mesh_approval(&exact, &wrong_envelope),
            ApprovalApplicability::Mismatch
        );
    }
}

#[test]
fn t123_ambiguous_unavailable_explicit_policy_and_authority_downgrade_never_fallback() {
    let request = TargetRequest::new(exact_target(), TargetSelector::Human).unwrap();
    let ambiguous = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        winds_claim(IdentityDimension::Provider, "openai"),
        winds_claim(IdentityDimension::Provider, "anthropic"),
        winds_claim(IdentityDimension::Model, "model-a"),
    ];
    assert_eq!(
        resolve(&request, &ambiguous, CurrentAuthorityTruth::Allowed),
        TargetResolution::Ambiguous
    );

    let unavailable = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        IdentityClaim::new(
            IdentityDimension::Provider,
            None,
            IdentitySourceClass::Unavailable,
            Some("provider unavailable"),
        )
        .unwrap(),
        winds_claim(IdentityDimension::Model, "model-a"),
    ];
    assert_eq!(
        resolve(&request, &unavailable, CurrentAuthorityTruth::Allowed),
        TargetResolution::Unavailable
    );

    let exact = vec![
        winds_claim(IdentityDimension::Runtime, RuntimeKind::Codex.as_str()),
        winds_claim(IdentityDimension::Provider, "openai"),
        winds_claim(IdentityDimension::Model, "model-a"),
    ];
    assert_eq!(
        resolve(&request, &exact, CurrentAuthorityTruth::Denied),
        TargetResolution::AuthorityDenied
    );

    let policy = TargetRequest::new(exact_target(), TargetSelector::ExplicitPolicy).unwrap();
    assert_eq!(
        resolve(&policy, &exact, CurrentAuthorityTruth::Allowed),
        TargetResolution::PolicyNotAuthorized
    );
}

#[test]
fn t123_secret_private_context_and_forged_native_resume_are_rejected() {
    for secret in [
        "Bearer abcdef",
        "api_key=super-secret",
        "password=hunter2",
        "provider-private-memory: hidden",
        "winner recommendation: use model x",
    ] {
        assert!(ModelMeshContinuityReferenceV1::new("evidence", secret).is_err());
    }
    assert!(
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("openai"),
            IdentitySourceClass::WindsLocallyObserved,
            Some("authorization=Bearer secret"),
        )
        .is_err()
    );
    assert!(
        ModelMeshContinuityMarkerV1::new(
            "provider-private-state",
            ContinuityMaterialState::Unavailable,
            false,
        )
        .is_err()
    );

    let target = unspecified_target("actor-binding-1", "session-1", "WORKER");
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&target).unwrap();
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-1",
            target: &target,
            continuity_class: ContinuityClass::NativeResume,
            source_actor: None,
            destination_actor: None,
            candidate_references: &[],
            artifact_references: &[],
            evidence_references: &[],
            selection_approval: &approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        })
        .unwrap_err()
        .contains("does not provide physical NATIVE_RESUME proof")
    );

    let marker = ModelMeshContinuityMarkerV1::new(
        "required-context",
        ContinuityMaterialState::Redacted,
        true,
    )
    .unwrap();
    let context = build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-1",
        target: &target,
        continuity_class: ContinuityClass::Unavailable,
        source_actor: None,
        destination_actor: None,
        candidate_references: &[],
        artifact_references: &[],
        evidence_references: &[],
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Denied,
        reconstruction_report: None,
        markers: &[marker],
    })
    .unwrap();
    assert_eq!(
        context.completeness(),
        ContinuityContextCompleteness::Redacted
    );
    assert!(
        context
            .canonical_json()
            .unwrap()
            .contains("provider-private-state")
    );
    assert!(!context.canonical_json().unwrap().contains("Bearer"));
}

#[test]
fn t123_source_agent_pass_verified_accepted_cost_winner_labels_never_promote_outcomes() {
    let request = TargetRequest::new(exact_target(), TargetSelector::Human).unwrap();
    let claims = vec![
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("PASS VERIFIED ACCEPTED COST WINNER AUTHORITY HANDOFF"),
            IdentitySourceClass::AgentReported,
            Some("source agent prose"),
        )
        .unwrap(),
    ];
    let drift = exact_drift();
    let projection = project_model_mesh_target(&ModelMeshTargetProjectionInput {
        request: &request,
        claims: &claims,
        resolution: TargetResolution::Unknown,
        drift: &drift,
        authentication: AuthenticationTruth::Unknown,
        approval: ApprovalApplicability::Missing,
        approval_digest_match: ModelMeshProjectionTruth::Unknown,
        current_authority: CurrentAuthorityTruth::Denied,
        evidence_freshness: ModelMeshEvidenceFreshnessProjection {
            candidate: DriftApplicability::Stale,
            artifact: DriftApplicability::Stale,
            evidence: DriftApplicability::Stale,
        },
        execution_authorized: ModelMeshProjectionTruth::Unknown,
        continuity_proven: ModelMeshProjectionTruth::Unknown,
        verified: ModelMeshProjectionTruth::Unknown,
        human_accepted: ModelMeshProjectionTruth::Unknown,
        landed: ModelMeshProjectionTruth::Unknown,
    });
    assert_eq!(projection.identity_claims[0].source, "AGENT_REPORTED");
    assert_eq!(
        projection.outcomes.target_match,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        projection.outcomes.execution_authorized,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        projection.outcomes.continuity_proven,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        projection.outcomes.verified,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        projection.outcomes.human_accepted,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        projection.outcomes.landed,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(
        serde_json::to_value(&projection).unwrap()["usage_cost"],
        serde_json::json!("UNKNOWN")
    );
}

#[test]
fn t123_request_claim_replay_role_and_session_scope_preserve_append_only_history() {
    let (home, mut store) = seeded_store("replay-history");
    let request = TargetRequest::new(
        unspecified_target("actor-binding-1", "session-1", "WORKER"),
        TargetSelector::Human,
    )
    .unwrap();
    persist_target(&mut store, "target-1", &request, "approval-1", 20);
    let replay = store
        .create_model_mesh_target_request(NewModelMeshTargetRequest {
            target_request_id: "target-1",
            request: &request,
            selection_approval_id: "approval-1",
            created_unix_ms: 20,
        })
        .unwrap();
    assert_eq!(replay.target_request_id, "target-1");

    let role_mismatch = TargetRequest::new(
        unspecified_target("actor-binding-1", "session-1", "REVIEWER"),
        TargetSelector::Human,
    )
    .unwrap();
    assert!(
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: "target-1",
                request: &role_mismatch,
                selection_approval_id: "approval-1",
                created_unix_ms: 20,
            })
            .unwrap_err()
            .to_string()
            .contains("idempotency collision")
    );

    let claim = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("PASS VERIFIED ACCEPTED"),
        IdentitySourceClass::AgentReported,
        Some("source agent report"),
    )
    .unwrap();
    let stored_claim = store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-1",
            subject: ModelMeshClaimSubject::RequestTarget("target-1".into()),
            claim: &claim,
            runtime_binding_id: None,
            observed_unix_ms: 21,
        })
        .unwrap();
    assert_eq!(
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: "claim-1",
                subject: ModelMeshClaimSubject::RequestTarget("target-1".into()),
                claim: &claim,
                runtime_binding_id: None,
                observed_unix_ms: 21,
            })
            .unwrap(),
        stored_claim
    );
    let changed_claim = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::AgentReported,
        Some("changed report"),
    )
    .unwrap();
    assert!(
        store
            .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
                identity_claim_id: "claim-1",
                subject: ModelMeshClaimSubject::RequestTarget("target-1".into()),
                claim: &changed_claim,
                runtime_binding_id: None,
                observed_unix_ms: 21,
            })
            .unwrap_err()
            .to_string()
            .contains("idempotency collision")
    );

    seed_second_actor(&store);
    let second = TargetRequest::new(
        unspecified_target("actor-binding-2", "session-2", "WORKER"),
        TargetSelector::Human,
    )
    .unwrap();
    persist_target(&mut store, "target-2", &second, "approval-2", 30);
    persist_target(&mut store, "target-3", &role_mismatch, "approval-3", 40);

    let history = store
        .list_model_mesh_target_requests_for_stage("stage-1")
        .unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].target_request_id, "target-1");
    assert_eq!(history[1].target_request_id, "target-2");
    assert_eq!(history[2].target_request_id, "target-3");
    assert_ne!(
        history[0].request.target_descriptor_digest(),
        history[1].request.target_descriptor_digest()
    );
    assert_ne!(
        history[0].request.target_descriptor_digest(),
        history[2].request.target_descriptor_digest()
    );
    drop(store);
    cleanup(home);
}

#[test]
fn t123_corrupt_selector_fails_closed_and_is_not_destructively_recovered() {
    let (home, mut store) = seeded_store("corrupt-selector");
    let request = TargetRequest::new(
        unspecified_target("actor-binding-1", "session-1", "WORKER"),
        TargetSelector::Human,
    )
    .unwrap();
    persist_target(&mut store, "target-1", &request, "approval-1", 20);

    let trigger_sql: String = store
        .connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='trigger' AND name='trg_model_mesh_target_requests_no_update'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    store
        .connection
        .execute_batch(
            "DROP TRIGGER trg_model_mesh_target_requests_no_update;
             PRAGMA ignore_check_constraints = ON;",
        )
        .unwrap();
    store
        .connection
        .execute(
            "UPDATE model_mesh_target_requests SET selector_class='FORGED' WHERE target_request_id=?1",
            params!["target-1"],
        )
        .unwrap();
    store.connection.execute_batch(&trigger_sql).unwrap();
    store
        .connection
        .execute_batch("PRAGMA ignore_check_constraints = OFF;")
        .unwrap();

    let error = store
        .load_model_mesh_target_request("target-1")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("unknown Model Mesh target selector: FORGED"),
        "{error}"
    );
    let raw: String = store
        .connection
        .query_row(
            "SELECT selector_class FROM model_mesh_target_requests WHERE target_request_id='target-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(raw, "FORGED");
    drop(store);
    cleanup(home);
}
