use crate::agentic_authority::record_model_mesh_approval;
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeBindingOwnership, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity,
    RuntimeKind, RuntimeResumeResolution, RuntimeSessionBinding, RuntimeVersionEvidence,
    RuntimeVersionState,
};
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineKind, ArtifactBaselineRequirement, BaselineFreshness,
    CandidateBaselineIdentity, StageRunIdentity, WorkflowRunIdentity,
};
use crate::model_mesh::{
    ApprovalApplicability, CurrentAuthorityTruth, DriftApplicability, ExactModelId,
    ExactProviderId, IdentityClaim, IdentityDimension, IdentitySourceClass,
    ModelMeshAuthorityEnvelopeV1, ModelMeshCurrentTargetState, ModelMeshDriftInput,
    ModelMeshEvaluatedTargetRecord, ModelMeshTargetDescriptorV1, SourceRequirements,
    TargetDimension, TargetRequest, TargetSelector, evaluate_identity_claim_drift,
    evaluate_model_mesh_drift, project_current_model_mesh_target,
};
use crate::store::{
    ModelMeshClaimSubject, NewModelMeshIdentityClaim, NewModelMeshTargetRequest, NewWindsSession,
    NewWorkspace, NewWorkstream, Store,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn runtime_discovery(version: &str, sha: char) -> RuntimeDiscovery {
    let executable = if cfg!(windows) {
        PathBuf::from(r"C:\winds-t117-codex.exe")
    } else {
        PathBuf::from("/tmp/winds-t117-codex")
    };
    RuntimeDiscovery {
        runtime: RuntimeKind::Codex,
        state: RuntimeDiscoveryState::Present,
        executable: Some(RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 117,
            sha256: sha.to_string().repeat(64),
        }),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some(version.into()),
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

fn runtime_binding(discovery: &RuntimeDiscovery) -> RuntimeSessionBinding {
    RuntimeSessionBinding {
        binding_id: "runtime-binding-1".into(),
        session_id: "session-1".into(),
        runtime: discovery.runtime,
        executable: discovery.executable.clone().unwrap(),
        version: discovery.version.clone(),
        native_session_id: Some("native-session-1".into()),
        ownership: RuntimeBindingOwnership::Unproven,
        bound_unix_ms: 6,
        ownership_observed_unix_ms: None,
    }
}

fn descriptor(
    stage: &str,
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
        stage,
        actor,
        session,
        role,
        RuntimeKind::Codex,
        provider,
        model,
    )
    .unwrap()
}

fn exact_descriptor() -> ModelMeshTargetDescriptorV1 {
    descriptor(
        "stage-1",
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
        Some("t117 accepted observation"),
    )
    .unwrap()
}

struct DriftFixture {
    request: TargetRequest,
    current_descriptor: ModelMeshTargetDescriptorV1,
    claims: Vec<IdentityClaim>,
    historical_binding: RuntimeSessionBinding,
    current_binding: RuntimeSessionBinding,
    discovery: RuntimeDiscovery,
    baseline_requirements: Vec<ArtifactBaselineRequirement>,
    observed_baselines: Vec<ArtifactBaselineIdentity>,
    approval: ApprovalApplicability,
    authority: CurrentAuthorityTruth,
    require_native_session: bool,
}

impl DriftFixture {
    fn exact() -> Self {
        let discovery = runtime_discovery("t117-runtime-1", 'a');
        let binding = runtime_binding(&discovery);
        let descriptor = exact_descriptor();
        Self {
            request: TargetRequest::new(descriptor.clone(), TargetSelector::Human).unwrap(),
            current_descriptor: descriptor,
            claims: vec![
                winds_claim(IdentityDimension::Provider, "openai"),
                winds_claim(IdentityDimension::Model, "model-a"),
            ],
            historical_binding: binding.clone(),
            current_binding: binding,
            discovery,
            baseline_requirements: Vec::new(),
            observed_baselines: Vec::new(),
            approval: ApprovalApplicability::Exact,
            authority: CurrentAuthorityTruth::Allowed,
            require_native_session: false,
        }
    }

    fn evaluate(&self) -> crate::model_mesh::ModelMeshDriftEvaluation {
        evaluate_model_mesh_drift(&ModelMeshDriftInput {
            request: &self.request,
            current_descriptor: &self.current_descriptor,
            current_claims: &self.claims,
            source_requirements: SourceRequirements::winds_observed(),
            historical_runtime_binding: Some(&self.historical_binding),
            current_runtime_binding: Some(&self.current_binding),
            current_runtime_discovery: Some(&self.discovery),
            require_native_session: self.require_native_session,
            baseline_requirements: &self.baseline_requirements,
            observed_baselines: &self.observed_baselines,
            approval: self.approval,
            current_authority: self.authority,
        })
    }
}

#[test]
fn t117_exact_inputs_are_deterministic_and_applicable() {
    let fixture = DriftFixture::exact();
    let first = fixture.evaluate();
    let second = fixture.evaluate();
    assert_eq!(first, second);
    assert_eq!(first.runtime, DriftApplicability::Applicable);
    assert_eq!(first.provider, DriftApplicability::Applicable);
    assert_eq!(first.model, DriftApplicability::Applicable);
    assert_eq!(first.native_session, DriftApplicability::NotApplicable);
    assert_eq!(
        first.baseline.applicability,
        DriftApplicability::NotApplicable
    );
    assert_eq!(first.overall, DriftApplicability::Applicable);
}

#[test]
fn t117_runtime_executable_or_version_movement_stales_only_runtime_basis() {
    let mut fixture = DriftFixture::exact();
    fixture.discovery = runtime_discovery("t117-runtime-2", 'b');
    let evaluation = fixture.evaluate();
    assert_eq!(evaluation.runtime, DriftApplicability::Stale);
    assert_eq!(evaluation.provider, DriftApplicability::Applicable);
    assert_eq!(evaluation.model, DriftApplicability::Applicable);
    assert_eq!(evaluation.scope, DriftApplicability::Applicable);
    assert_eq!(evaluation.overall, DriftApplicability::Stale);
}

#[test]
fn t117_provider_and_model_movement_conflict_remain_dimension_local() {
    let mut fixture = DriftFixture::exact();
    fixture.claims = vec![
        winds_claim(IdentityDimension::Provider, "openai"),
        winds_claim(IdentityDimension::Model, "model-b"),
    ];
    let moved = fixture.evaluate();
    assert_eq!(moved.provider, DriftApplicability::Applicable);
    assert_eq!(moved.model, DriftApplicability::Stale);

    fixture
        .claims
        .push(winds_claim(IdentityDimension::Model, "model-c"));
    let conflicted = fixture.evaluate();
    assert_eq!(conflicted.provider, DriftApplicability::Applicable);
    assert_eq!(conflicted.model, DriftApplicability::Ambiguous);
    assert_eq!(conflicted.overall, DriftApplicability::Ambiguous);
}

#[test]
fn t117_unspecified_target_dimensions_ignore_unrelated_identity_movement() {
    let discovery = runtime_discovery("t117-runtime-1", 'a');
    let binding = runtime_binding(&discovery);
    let descriptor = descriptor(
        "stage-1",
        "actor-binding-1",
        "session-1",
        "WORKER",
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    );
    let request = TargetRequest::new(descriptor.clone(), TargetSelector::Human).unwrap();
    let claims = vec![
        winds_claim(IdentityDimension::Provider, "provider-a"),
        winds_claim(IdentityDimension::Provider, "provider-b"),
        winds_claim(IdentityDimension::Model, "model-x"),
    ];
    let evaluation = evaluate_model_mesh_drift(&ModelMeshDriftInput {
        request: &request,
        current_descriptor: &descriptor,
        current_claims: &claims,
        source_requirements: SourceRequirements::winds_observed(),
        historical_runtime_binding: Some(&binding),
        current_runtime_binding: Some(&binding),
        current_runtime_discovery: Some(&discovery),
        require_native_session: false,
        baseline_requirements: &[],
        observed_baselines: &[],
        approval: ApprovalApplicability::Exact,
        current_authority: CurrentAuthorityTruth::Allowed,
    });
    assert_eq!(evaluation.provider, DriftApplicability::NotApplicable);
    assert_eq!(evaluation.model, DriftApplicability::NotApplicable);
    assert_eq!(evaluation.overall, DriftApplicability::Applicable);
}

#[test]
fn t117_native_session_identity_never_upgrades_unproven_ownership() {
    let mut fixture = DriftFixture::exact();
    fixture.require_native_session = true;
    let unchanged = fixture.evaluate();
    assert_eq!(unchanged.native_session, DriftApplicability::Unproven);
    assert_eq!(unchanged.overall, DriftApplicability::Unproven);

    fixture.current_binding.ownership = RuntimeBindingOwnership::OwnershipLost;
    fixture.current_binding.ownership_observed_unix_ms = Some(20);
    let lost = fixture.evaluate();
    assert_eq!(lost.native_session, DriftApplicability::Stale);
    assert_eq!(lost.overall, DriftApplicability::Stale);
}

#[test]
fn t117_stage_actor_session_or_role_movement_stales_scope_without_relabelling_identity() {
    for current in [
        descriptor(
            "stage-2",
            "actor-binding-1",
            "session-1",
            "WORKER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
        descriptor(
            "stage-1",
            "actor-binding-2",
            "session-1",
            "WORKER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
        descriptor(
            "stage-1",
            "actor-binding-1",
            "session-2",
            "WORKER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
        descriptor(
            "stage-1",
            "actor-binding-1",
            "session-1",
            "REVIEWER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
    ] {
        let mut fixture = DriftFixture::exact();
        fixture.current_descriptor = current;
        let evaluation = fixture.evaluate();
        assert_eq!(evaluation.scope, DriftApplicability::Stale);
        assert_eq!(evaluation.provider, DriftApplicability::Applicable);
        assert_eq!(evaluation.model, DriftApplicability::Applicable);
        assert_eq!(evaluation.overall, DriftApplicability::Stale);
    }
}

#[test]
fn t117_candidate_artifact_evidence_freshness_reuses_spec008_evaluator() {
    let candidate_a = CandidateBaselineIdentity::new(&"a".repeat(40), &"b".repeat(40)).unwrap();
    let candidate_b = CandidateBaselineIdentity::new(&"c".repeat(40), &"d".repeat(40)).unwrap();
    let requirement = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::WindsVerificationEvidence,
        "verification-run-1",
        Some(candidate_a.clone()),
    )
    .unwrap();
    let baseline = ArtifactBaselineIdentity::new(
        "baseline-1",
        "stage-1",
        ArtifactBaselineKind::WindsVerificationEvidence,
        "verification-run-1",
        Some(candidate_b),
    )
    .unwrap();
    let mut fixture = DriftFixture::exact();
    fixture.baseline_requirements = vec![requirement];
    fixture.observed_baselines = vec![baseline];
    let evaluation = fixture.evaluate();
    assert_eq!(evaluation.baseline.applicability, DriftApplicability::Stale);
    assert_eq!(evaluation.baseline.evaluations.len(), 1);
    assert_eq!(
        evaluation.baseline.evaluations[0].freshness,
        BaselineFreshness::Stale
    );
    assert_eq!(evaluation.overall, DriftApplicability::Stale);
}

#[test]
fn t117_approval_and_current_authority_remain_separate_fail_closed_bases() {
    let mut fixture = DriftFixture::exact();
    fixture.approval = ApprovalApplicability::Mismatch;
    let stale_approval = fixture.evaluate();
    assert_eq!(stale_approval.approval, DriftApplicability::Stale);
    assert_eq!(
        stale_approval.current_authority,
        DriftApplicability::Applicable
    );
    assert_eq!(stale_approval.overall, DriftApplicability::Stale);

    fixture.approval = ApprovalApplicability::Exact;
    fixture.authority = CurrentAuthorityTruth::Denied;
    let denied = fixture.evaluate();
    assert_eq!(denied.approval, DriftApplicability::Applicable);
    assert_eq!(denied.current_authority, DriftApplicability::Denied);
    assert_eq!(denied.overall, DriftApplicability::Denied);
}

#[test]
fn t117_claim_drift_is_source_and_dimension_bound() {
    let historical = winds_claim(IdentityDimension::Provider, "openai");
    let unrelated_model = winds_claim(IdentityDimension::Model, "model-b");
    assert_eq!(
        evaluate_identity_claim_drift(&historical, &[unrelated_model]),
        DriftApplicability::Missing
    );
    let agent_same = IdentityClaim::new(
        IdentityDimension::Provider,
        Some("openai"),
        IdentitySourceClass::AgentReported,
        Some("agent report"),
    )
    .unwrap();
    assert_eq!(
        evaluate_identity_claim_drift(&historical, &[agent_same]),
        DriftApplicability::Stale
    );
}

#[test]
fn t117_current_target_projection_never_uses_latest_wins_and_preserves_history() {
    let fixture = DriftFixture::exact();
    let applicable = fixture.evaluate();
    let mut stale_fixture = DriftFixture::exact();
    stale_fixture.approval = ApprovalApplicability::Stale;
    let stale = stale_fixture.evaluate();
    let records = vec![
        ModelMeshEvaluatedTargetRecord {
            stage_run_id: "stage-1".into(),
            target_request_id: "target-old".into(),
            created_unix_ms: 10,
            drift: stale,
        },
        ModelMeshEvaluatedTargetRecord {
            stage_run_id: "stage-1".into(),
            target_request_id: "target-new".into(),
            created_unix_ms: 20,
            drift: applicable.clone(),
        },
    ];
    let projection = project_current_model_mesh_target(&records).unwrap();
    assert_eq!(
        projection.state,
        ModelMeshCurrentTargetState::Exact("target-new".into())
    );
    assert_eq!(projection.historical.len(), 2);
    assert_eq!(projection.historical[0].target_request_id, "target-old");

    let ambiguous = project_current_model_mesh_target(&[
        records[1].clone(),
        ModelMeshEvaluatedTargetRecord {
            stage_run_id: "stage-1".into(),
            target_request_id: "target-third".into(),
            created_unix_ms: 30,
            drift: applicable,
        },
    ])
    .unwrap();
    assert_eq!(
        ambiguous.state,
        ModelMeshCurrentTargetState::Ambiguous(vec!["target-new".into(), "target-third".into()])
    );

    let mixed_stage = project_current_model_mesh_target(&[
        records[1].clone(),
        ModelMeshEvaluatedTargetRecord {
            stage_run_id: "stage-2".into(),
            target_request_id: "target-other-stage".into(),
            created_unix_ms: 40,
            drift: fixture.evaluate(),
        },
    ]);
    assert!(mixed_stage.is_err());
}

fn test_home() -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t117-history-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

#[test]
fn t117_store_queries_append_only_target_and_claim_history_without_mutating_0011() {
    let home = test_home();
    let mut store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t117-workspace",
                git_common_dir: "/tmp/t117-git",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T117 workstream",
            },
            2,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "T117 session",
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
    let discovery = runtime_discovery("t117-runtime-1", 'a');
    store
        .create_runtime_session_binding(
            "runtime-binding-1",
            "session-1",
            &discovery,
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

    for (id, model, approval_id, approved_at, created_at) in [
        ("target-a", "model-a", "approval-a", 8, 9),
        ("target-b", "model-b", "approval-b", 10, 11),
    ] {
        let target = descriptor(
            "stage-1",
            "actor-binding-1",
            "session-1",
            "WORKER",
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Exact(ExactModelId::new(model).unwrap()),
        );
        let request = TargetRequest::new(target, TargetSelector::Human).unwrap();
        let envelope =
            ModelMeshAuthorityEnvelopeV1::for_target_selection(request.descriptor()).unwrap();
        record_model_mesh_approval(&store, approval_id, &envelope, approved_at).unwrap();
        store
            .create_model_mesh_target_request(NewModelMeshTargetRequest {
                target_request_id: id,
                request: &request,
                selection_approval_id: approval_id,
                created_unix_ms: created_at,
            })
            .unwrap();
    }

    let claim = winds_claim(IdentityDimension::Provider, "openai");
    store
        .create_model_mesh_identity_claim(NewModelMeshIdentityClaim {
            identity_claim_id: "claim-target-a-provider",
            subject: ModelMeshClaimSubject::RequestTarget("target-a".into()),
            claim: &claim,
            runtime_binding_id: Some("runtime-binding-1"),
            observed_unix_ms: 12,
        })
        .unwrap();

    let history = store
        .list_model_mesh_target_requests_for_stage("stage-1")
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].target_request_id, "target-a");
    assert_eq!(history[1].target_request_id, "target-b");
    let claims = store
        .list_model_mesh_identity_claims_for_target_request("target-a")
        .unwrap();
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].identity_claim_id, "claim-target-a-provider");

    drop(store);
    fs::remove_dir_all(home).unwrap();
}
