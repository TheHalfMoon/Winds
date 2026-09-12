use crate::agentic_runtime::{
    EvidenceSource, RuntimeBindingOwnership, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeSessionBinding, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::WindsSessionRecord;
use crate::domain::workflow::{StageRunIdentity, WorkflowContinuationClass, WorkflowRunIdentity};
use crate::model_mesh::{
    ApprovalApplicability, AuthenticationTruth, ContinuityClass, CurrentAuthorityTruth,
    DriftApplicability, ExactModelId, ExactProviderId, IdentityClaim, IdentityDimension,
    IdentitySourceClass, ModelMeshAuthorityEnvelopeV1, ModelMeshBaselineDrift,
    ModelMeshBlockerCategory, ModelMeshContinuityActorV1, ModelMeshContinuityContextInput,
    ModelMeshContinuityProjectionInput, ModelMeshDriftEvaluation,
    ModelMeshEvidenceFreshnessProjection, ModelMeshProjectionTruth, ModelMeshTargetDescriptorV1,
    ModelMeshTargetProjectionInput, ModelMeshUsageCostProjection, SourceRequirements,
    TargetDimension, TargetRequest, TargetResolution, TargetSelector, adapt_actor_scope,
    build_model_mesh_continuity_context, project_model_mesh_continuity, project_model_mesh_target,
    project_model_mesh_why_blocked, project_reviewer_continuity,
};
use crate::store::StoredWorkflowActorBinding;
use std::path::PathBuf;

fn target() -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "actor-destination",
        "session-destination",
        "WORKER",
        RuntimeKind::Claude,
        TargetDimension::Exact(ExactProviderId::new("anthropic").unwrap()),
        TargetDimension::Exact(ExactModelId::new("claude-sonnet").unwrap()),
    )
    .unwrap()
}

fn request() -> TargetRequest {
    TargetRequest::new(target(), TargetSelector::Human).unwrap()
}

fn claim(
    dimension: IdentityDimension,
    value: Option<&str>,
    source: IdentitySourceClass,
) -> IdentityClaim {
    IdentityClaim::new(dimension, value, source, Some("t120 fixture observation")).unwrap()
}

fn drift(overall: DriftApplicability) -> ModelMeshDriftEvaluation {
    ModelMeshDriftEvaluation {
        runtime: DriftApplicability::Applicable,
        provider: DriftApplicability::Applicable,
        model: DriftApplicability::Applicable,
        native_session: DriftApplicability::Unproven,
        scope: DriftApplicability::Applicable,
        baseline: ModelMeshBaselineDrift {
            applicability: DriftApplicability::Applicable,
            evaluations: vec![],
        },
        approval: DriftApplicability::Applicable,
        current_authority: DriftApplicability::Applicable,
        overall,
    }
}

fn freshness(value: DriftApplicability) -> ModelMeshEvidenceFreshnessProjection {
    ModelMeshEvidenceFreshnessProjection {
        candidate: value,
        artifact: value,
        evidence: value,
    }
}

fn target_projection(
    resolution: TargetResolution,
    evidence: DriftApplicability,
) -> crate::model_mesh::ModelMeshTargetProjection {
    target_projection_with(
        resolution,
        evidence,
        AuthenticationTruth::Unknown,
        ApprovalApplicability::Exact,
        ModelMeshProjectionTruth::Yes,
        CurrentAuthorityTruth::Allowed,
    )
}

fn target_projection_with(
    resolution: TargetResolution,
    evidence: DriftApplicability,
    authentication: AuthenticationTruth,
    approval: ApprovalApplicability,
    approval_digest_match: ModelMeshProjectionTruth,
    current_authority: CurrentAuthorityTruth,
) -> crate::model_mesh::ModelMeshTargetProjection {
    let request = request();
    let claims = vec![
        claim(
            IdentityDimension::Runtime,
            Some("CLAUDE"),
            IdentitySourceClass::WindsLocallyObserved,
        ),
        claim(
            IdentityDimension::Provider,
            Some("anthropic"),
            IdentitySourceClass::VendorDeclared,
        ),
        claim(
            IdentityDimension::Model,
            Some("claude-sonnet"),
            IdentitySourceClass::AgentReported,
        ),
    ];
    project_model_mesh_target(&ModelMeshTargetProjectionInput {
        request: &request,
        claims: &claims,
        resolution,
        drift: &drift(DriftApplicability::Applicable),
        authentication,
        approval,
        approval_digest_match,
        current_authority,
        evidence_freshness: freshness(evidence),
        execution_authorized: ModelMeshProjectionTruth::No,
        continuity_proven: ModelMeshProjectionTruth::Unknown,
        verified: ModelMeshProjectionTruth::No,
        human_accepted: ModelMeshProjectionTruth::No,
        landed: ModelMeshProjectionTruth::No,
    })
}

fn workflow() -> WorkflowRunIdentity {
    WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap()
}

fn stage() -> StageRunIdentity {
    StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap()
}

fn session(id: &str) -> WindsSessionRecord {
    WindsSessionRecord {
        session_id: id.to_owned(),
        workstream_id: "workstream-1".to_owned(),
        display_name: format!("T120 {id}"),
        created_unix_ms: 1,
        updated_unix_ms: 1,
    }
}

fn binding(
    id: &str,
    session_id: &str,
    runtime: RuntimeKind,
    native: &str,
) -> RuntimeSessionBinding {
    let path = PathBuf::from(format!("/tmp/t120-{id}"));
    RuntimeSessionBinding {
        binding_id: id.to_owned(),
        session_id: session_id.to_owned(),
        runtime,
        executable: RuntimeExecutableIdentity {
            observed_path: path.clone(),
            canonical_path: path,
            byte_len: 120,
            sha256: match runtime {
                RuntimeKind::Codex => "a".repeat(64),
                RuntimeKind::Claude => "b".repeat(64),
            },
        },
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("t120-runtime-1".to_owned()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        native_session_id: Some(native.to_owned()),
        ownership: RuntimeBindingOwnership::Unproven,
        bound_unix_ms: 1,
        ownership_observed_unix_ms: None,
    }
}

fn actor(
    actor_id: &str,
    session_id: &str,
    runtime: RuntimeKind,
    runtime_binding_id: &str,
    native: &str,
) -> ModelMeshContinuityActorV1 {
    let runtime_binding = binding(runtime_binding_id, session_id, runtime, native);
    let stored = StoredWorkflowActorBinding {
        binding_id: actor_id.to_owned(),
        stage_run_id: "stage-1".to_owned(),
        winds_session_id: session_id.to_owned(),
        runtime_binding_id: Some(runtime_binding_id.to_owned()),
        continuation: WorkflowContinuationClass::Unproven,
        bound_unix_ms: 1,
        reconstruction_report: None,
    };
    let scope = adapt_actor_scope(
        &workflow(),
        &stage(),
        &session(session_id),
        &stored,
        Some(&runtime_binding),
    )
    .unwrap();
    ModelMeshContinuityActorV1::from_actor_scope(&scope, &runtime_binding).unwrap()
}

fn continuity_context() -> crate::model_mesh::ModelMeshContinuityContextV1 {
    let target = target();
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&target).unwrap();
    let source = actor(
        "actor-source",
        "session-source",
        RuntimeKind::Codex,
        "binding-source",
        "native-source",
    );
    let destination = actor(
        "actor-destination",
        "session-destination",
        RuntimeKind::Claude,
        "binding-destination",
        "native-destination",
    );
    build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target: &target,
        continuity_class: ContinuityClass::Handoff,
        source_actor: Some(&source),
        destination_actor: Some(&destination),
        candidate_references: &[],
        artifact_references: &[],
        evidence_references: &[],
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Allowed,
        reconstruction_report: None,
        markers: &[],
    })
    .unwrap()
}

#[test]
fn t120_target_projection_preserves_scope_sources_and_separate_outcomes() {
    let projection =
        target_projection(TargetResolution::ExactMatch, DriftApplicability::Applicable);
    assert_eq!(projection.workflow_run_id, "workflow-1");
    assert_eq!(projection.stage_run_id, "stage-1");
    assert_eq!(projection.actor_binding_id, "actor-destination");
    assert_eq!(projection.winds_session_id, "session-destination");
    assert_eq!(projection.actor_role, "WORKER");
    assert_eq!(projection.requested_runtime, "CLAUDE");
    assert_eq!(projection.identity_claims[1].source, "VENDOR_DECLARED");
    assert_eq!(projection.identity_claims[2].source, "AGENT_REPORTED");
    assert_eq!(
        projection.outcomes.target_match,
        ModelMeshProjectionTruth::Yes
    );
    assert_eq!(
        projection.outcomes.execution_authorized,
        ModelMeshProjectionTruth::No
    );
    assert_eq!(
        projection.outcomes.continuity_proven,
        ModelMeshProjectionTruth::Unknown
    );
    assert_eq!(projection.outcomes.verified, ModelMeshProjectionTruth::No);
    assert_eq!(
        projection.outcomes.human_accepted,
        ModelMeshProjectionTruth::No
    );
    assert_eq!(projection.outcomes.landed, ModelMeshProjectionTruth::No);
    assert_eq!(projection.usage_cost, ModelMeshUsageCostProjection::Unknown);
}

#[test]
fn t120_why_blocked_keeps_unknown_unavailable_ambiguous_conflict_stale_and_denied_distinct() {
    for (resolution, blocker) in [
        (
            TargetResolution::Unknown,
            ModelMeshBlockerCategory::TargetUnknown,
        ),
        (
            TargetResolution::Unavailable,
            ModelMeshBlockerCategory::TargetUnavailable,
        ),
        (
            TargetResolution::Ambiguous,
            ModelMeshBlockerCategory::TargetAmbiguous,
        ),
        (
            TargetResolution::Conflict,
            ModelMeshBlockerCategory::TargetConflict,
        ),
        (
            TargetResolution::Stale,
            ModelMeshBlockerCategory::StaleIdentity,
        ),
    ] {
        let projection = target_projection(resolution, DriftApplicability::Applicable);
        assert_eq!(project_model_mesh_why_blocked(&projection).primary, blocker);
    }

    let denied = target_projection_with(
        TargetResolution::AuthorityDenied,
        DriftApplicability::Applicable,
        AuthenticationTruth::Ready,
        ApprovalApplicability::Exact,
        ModelMeshProjectionTruth::Yes,
        CurrentAuthorityTruth::Denied,
    );
    assert_eq!(
        project_model_mesh_why_blocked(&denied).primary,
        ModelMeshBlockerCategory::AuthorityDenied
    );
}

#[test]
fn t120_why_blocked_preserves_auth_approval_native_and_evidence_blockers() {
    let mut projection = target_projection(TargetResolution::ExactMatch, DriftApplicability::Stale);
    projection.approval = ApprovalApplicability::Missing;
    projection.approval_digest_match = ModelMeshProjectionTruth::No;
    projection.current_authority = CurrentAuthorityTruth::Denied;
    let blocked = project_model_mesh_why_blocked(&projection);
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthenticationUnknown)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::ApprovalMissing)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::NativeSessionUnproven)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthorityDenied)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::CandidateStale)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::ArtifactStale)
    );
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::EvidenceStale)
    );
}

#[test]
fn t120_digest_unknown_or_stale_never_manufactures_authority_denial() {
    for digest_truth in [
        ModelMeshProjectionTruth::Unknown,
        ModelMeshProjectionTruth::Stale,
    ] {
        let mut projection =
            target_projection(TargetResolution::ExactMatch, DriftApplicability::Applicable);
        projection.authentication = AuthenticationTruth::Ready;
        projection.native_session_freshness = DriftApplicability::NotApplicable;
        projection.approval = ApprovalApplicability::Exact;
        projection.approval_digest_match = digest_truth;
        projection.current_authority = CurrentAuthorityTruth::Allowed;
        let blocked = project_model_mesh_why_blocked(&projection);
        assert!(
            !blocked
                .blockers
                .contains(&ModelMeshBlockerCategory::AuthorityDenied)
        );
    }

    let mut mismatch =
        target_projection(TargetResolution::ExactMatch, DriftApplicability::Applicable);
    mismatch.authentication = AuthenticationTruth::Ready;
    mismatch.native_session_freshness = DriftApplicability::NotApplicable;
    mismatch.approval = ApprovalApplicability::Exact;
    mismatch.approval_digest_match = ModelMeshProjectionTruth::No;
    mismatch.current_authority = CurrentAuthorityTruth::Allowed;
    let blocked = project_model_mesh_why_blocked(&mismatch);
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::ApprovalMismatch)
    );
    assert!(
        !blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthorityDenied)
    );
}

#[test]
fn t120_authority_resolution_is_disambiguated_by_independent_projection_truth() {
    let approval_missing = target_projection_with(
        TargetResolution::AuthorityDenied,
        DriftApplicability::Applicable,
        AuthenticationTruth::Ready,
        ApprovalApplicability::Missing,
        ModelMeshProjectionTruth::Unknown,
        CurrentAuthorityTruth::Allowed,
    );
    let blocked = project_model_mesh_why_blocked(&approval_missing);
    assert_eq!(
        approval_missing.blocker,
        ModelMeshBlockerCategory::ApprovalMissing
    );
    assert!(
        !blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthorityDenied)
    );

    let denied = target_projection_with(
        TargetResolution::AuthorityDenied,
        DriftApplicability::Applicable,
        AuthenticationTruth::Ready,
        ApprovalApplicability::Exact,
        ModelMeshProjectionTruth::Yes,
        CurrentAuthorityTruth::Denied,
    );
    let blocked = project_model_mesh_why_blocked(&denied);
    assert_eq!(denied.blocker, ModelMeshBlockerCategory::AuthorityDenied);
    assert!(
        blocked
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthorityDenied)
    );

    let inconsistent = target_projection_with(
        TargetResolution::AuthorityDenied,
        DriftApplicability::Applicable,
        AuthenticationTruth::Ready,
        ApprovalApplicability::Exact,
        ModelMeshProjectionTruth::Yes,
        CurrentAuthorityTruth::Allowed,
    );
    assert_eq!(
        inconsistent.blocker,
        ModelMeshBlockerCategory::InconsistentTruth
    );
    assert!(
        !project_model_mesh_why_blocked(&inconsistent)
            .blockers
            .contains(&ModelMeshBlockerCategory::AuthorityDenied)
    );
}

#[test]
fn t120_target_match_preserves_uncertainty_and_staleness() {
    for resolution in [
        TargetResolution::Unknown,
        TargetResolution::Unavailable,
        TargetResolution::Ambiguous,
        TargetResolution::Conflict,
        TargetResolution::AuthenticationUnknown,
        TargetResolution::CapabilityUnavailable,
        TargetResolution::AuthorityDenied,
        TargetResolution::PolicyNotAuthorized,
    ] {
        let projection = target_projection(resolution, DriftApplicability::Applicable);
        assert_eq!(
            projection.outcomes.target_match,
            ModelMeshProjectionTruth::Unknown
        );
    }
    let stale = target_projection(TargetResolution::Stale, DriftApplicability::Stale);
    assert_eq!(stale.outcomes.target_match, ModelMeshProjectionTruth::Stale);
}

#[test]
fn t120_non_stale_freshness_is_not_relabelled_stale() {
    for freshness in [
        DriftApplicability::Missing,
        DriftApplicability::Unproven,
        DriftApplicability::Ambiguous,
        DriftApplicability::Conflict,
        DriftApplicability::Denied,
    ] {
        let mut projection = target_projection(TargetResolution::ExactMatch, freshness);
        projection.authentication = AuthenticationTruth::Ready;
        projection.runtime_freshness = freshness;
        projection.native_session_freshness = freshness;
        projection.approval = ApprovalApplicability::Exact;
        projection.approval_digest_match = ModelMeshProjectionTruth::Yes;
        projection.current_authority = CurrentAuthorityTruth::Allowed;
        let blocked = project_model_mesh_why_blocked(&projection);
        assert_eq!(blocked.runtime_freshness, freshness);
        assert_eq!(blocked.native_session_freshness, freshness);
        assert_eq!(blocked.evidence_freshness.candidate, freshness);
        assert!(
            !blocked
                .blockers
                .contains(&ModelMeshBlockerCategory::StaleIdentity)
        );
        assert!(
            !blocked
                .blockers
                .contains(&ModelMeshBlockerCategory::CandidateStale)
        );
        assert!(
            !blocked
                .blockers
                .contains(&ModelMeshBlockerCategory::ArtifactStale)
        );
        assert!(
            !blocked
                .blockers
                .contains(&ModelMeshBlockerCategory::EvidenceStale)
        );
        if freshness == DriftApplicability::Unproven {
            assert!(
                blocked
                    .blockers
                    .contains(&ModelMeshBlockerCategory::NativeSessionUnproven)
            );
        }
    }
}

#[test]
fn t120_continuity_projection_preserves_actor_session_runtime_native_and_identity_truth() {
    let context = continuity_context();
    let source_claims = vec![claim(
        IdentityDimension::Provider,
        None,
        IdentitySourceClass::Unavailable,
    )];
    let destination_claims = vec![claim(
        IdentityDimension::Model,
        Some("claude-sonnet"),
        IdentitySourceClass::VendorDeclared,
    )];
    let projection = project_model_mesh_continuity(&ModelMeshContinuityProjectionInput {
        context: &context,
        source_identity_claims: &source_claims,
        destination_identity_claims: &destination_claims,
        evidence_freshness: freshness(DriftApplicability::Applicable),
        approval: ApprovalApplicability::Exact,
        approval_digest_match: ModelMeshProjectionTruth::Yes,
        current_authority: CurrentAuthorityTruth::Allowed,
        execution_authorized: ModelMeshProjectionTruth::Yes,
        continuity_proven: ModelMeshProjectionTruth::Yes,
        verified: ModelMeshProjectionTruth::No,
        human_accepted: ModelMeshProjectionTruth::No,
        landed: ModelMeshProjectionTruth::No,
    });
    assert_eq!(projection.continuity_class, "HANDOFF");
    assert_eq!(
        projection.source_actor_binding_id.as_deref(),
        Some("actor-source")
    );
    assert_eq!(
        projection.destination_actor_binding_id.as_deref(),
        Some("actor-destination")
    );
    assert_eq!(
        projection.source_winds_session_id.as_deref(),
        Some("session-source")
    );
    assert_eq!(
        projection.destination_winds_session_id.as_deref(),
        Some("session-destination")
    );
    assert_eq!(projection.source_runtime.as_deref(), Some("CODEX"));
    assert_eq!(projection.destination_runtime.as_deref(), Some("CLAUDE"));
    assert_eq!(
        projection.source_native_session_id.as_deref(),
        Some("native-source")
    );
    assert_eq!(
        projection.destination_native_session_id.as_deref(),
        Some("native-destination")
    );
    assert_eq!(projection.source_identity_claims[0].source, "UNAVAILABLE");
    assert_eq!(
        projection.destination_identity_claims[0].source,
        "VENDOR_DECLARED"
    );
    assert_eq!(
        projection.outcomes.execution_authorized,
        ModelMeshProjectionTruth::Yes
    );
    assert_eq!(projection.outcomes.verified, ModelMeshProjectionTruth::No);
}

#[test]
fn t120_reviewer_projection_is_freshness_sensitive_and_contains_no_winner_or_persuasion_surface() {
    let context = continuity_context();
    let continuity = project_model_mesh_continuity(&ModelMeshContinuityProjectionInput {
        context: &context,
        source_identity_claims: &[],
        destination_identity_claims: &[],
        evidence_freshness: ModelMeshEvidenceFreshnessProjection {
            candidate: DriftApplicability::Applicable,
            artifact: DriftApplicability::Applicable,
            evidence: DriftApplicability::Stale,
        },
        approval: ApprovalApplicability::Exact,
        approval_digest_match: ModelMeshProjectionTruth::Yes,
        current_authority: CurrentAuthorityTruth::Allowed,
        execution_authorized: ModelMeshProjectionTruth::Yes,
        continuity_proven: ModelMeshProjectionTruth::Yes,
        verified: ModelMeshProjectionTruth::No,
        human_accepted: ModelMeshProjectionTruth::No,
        landed: ModelMeshProjectionTruth::No,
    });
    let reviewer = project_reviewer_continuity(&continuity);
    assert!(!reviewer.reviewer_context_fresh);
    let json = serde_json::to_string(&reviewer).unwrap();
    assert!(!json.to_ascii_lowercase().contains("winner"));
    assert!(!json.to_ascii_lowercase().contains("recommend"));
    assert!(!json.to_ascii_lowercase().contains("persuasion"));
}

#[test]
fn t120_projection_is_pure_and_does_not_mutate_source_truth() {
    let request = request();
    let claims = vec![claim(
        IdentityDimension::Provider,
        Some("anthropic"),
        IdentitySourceClass::CatalogDeclared,
    )];
    let before_request = request.target_descriptor_digest().to_owned();
    let before_claims = claims.clone();
    let _ = project_model_mesh_target(&ModelMeshTargetProjectionInput {
        request: &request,
        claims: &claims,
        resolution: TargetResolution::Unknown,
        drift: &drift(DriftApplicability::Missing),
        authentication: AuthenticationTruth::Unknown,
        approval: ApprovalApplicability::Missing,
        approval_digest_match: ModelMeshProjectionTruth::Unknown,
        current_authority: CurrentAuthorityTruth::Denied,
        evidence_freshness: freshness(DriftApplicability::Missing),
        execution_authorized: ModelMeshProjectionTruth::No,
        continuity_proven: ModelMeshProjectionTruth::Unknown,
        verified: ModelMeshProjectionTruth::Unknown,
        human_accepted: ModelMeshProjectionTruth::Unknown,
        landed: ModelMeshProjectionTruth::Unknown,
    });
    assert_eq!(request.target_descriptor_digest(), before_request);
    assert_eq!(claims, before_claims);
}

#[test]
fn t120_source_requirements_remain_winds_observed_without_projection_side_effect() {
    let requirements = SourceRequirements::winds_observed();
    assert_eq!(
        requirements.runtime,
        IdentitySourceClass::WindsLocallyObserved
    );
    assert_eq!(
        requirements.provider,
        IdentitySourceClass::WindsLocallyObserved
    );
    assert_eq!(
        requirements.model,
        IdentitySourceClass::WindsLocallyObserved
    );
}
