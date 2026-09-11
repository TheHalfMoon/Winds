use super::agentic_runtime::RuntimeKind;
use super::model_mesh::{
    ApprovalApplicability, AuthenticationTruth, CapabilityTruth, ContextDigest, ContinuityClass,
    CurrentAuthorityTruth, ExactModelId, ExactProviderId, IdentityClaim, IdentityDimension,
    IdentitySourceClass, ModelMeshAuthorityEnvelopeV1, ModelMeshAuthorityPurpose,
    ModelMeshContinuityPermissionDescriptorV1, ModelMeshTargetDescriptorV1, SourceRequirements,
    TargetDimension, TargetRequest, TargetResolution, TargetResolverInput, TargetSelector,
    resolve_target,
};

#[allow(clippy::too_many_arguments)]
fn descriptor(
    workspace: &str,
    workstream: &str,
    workflow: &str,
    stage: &str,
    binding: &str,
    session: &str,
    role: &str,
    runtime: RuntimeKind,
    provider: TargetDimension<ExactProviderId>,
    model: TargetDimension<ExactModelId>,
) -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        workspace, workstream, workflow, stage, binding, session, role, runtime, provider, model,
    )
    .expect("valid Model Mesh descriptor")
}

fn basic_descriptor() -> ModelMeshTargetDescriptorV1 {
    descriptor(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "binding-1",
        "winds-session-1",
        "planner",
        RuntimeKind::Codex,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
}

fn runtime_claim(value: &str, source: IdentitySourceClass) -> IdentityClaim {
    IdentityClaim::new(
        IdentityDimension::Runtime,
        Some(value),
        source,
        Some("fixture"),
    )
    .expect("valid claim")
}

fn provider_claim(value: &str, source: IdentitySourceClass) -> IdentityClaim {
    IdentityClaim::new(
        IdentityDimension::Provider,
        Some(value),
        source,
        Some("fixture"),
    )
    .expect("valid claim")
}

fn model_claim(value: &str, source: IdentitySourceClass) -> IdentityClaim {
    IdentityClaim::new(
        IdentityDimension::Model,
        Some(value),
        source,
        Some("fixture"),
    )
    .expect("valid claim")
}

#[allow(clippy::too_many_arguments)]
fn resolve(
    request: &TargetRequest,
    claims: &[IdentityClaim],
    source_requirements: SourceRequirements,
    stale: bool,
    authentication: AuthenticationTruth,
    capability: CapabilityTruth,
    approval: ApprovalApplicability,
    current_authority: CurrentAuthorityTruth,
) -> TargetResolution {
    resolve_target(&TargetResolverInput {
        request,
        claims,
        source_requirements,
        stale,
        authentication,
        capability,
        approval,
        current_authority,
    })
}

#[test]
fn t114_target_descriptor_canonicalization_is_normalized_and_stable() {
    let left = descriptor(
        " workspace-1 ",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "binding-1",
        "winds-session-1",
        " planner ",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new(" openai ").unwrap()),
        TargetDimension::Exact(ExactModelId::new(" gpt-x ").unwrap()),
    );
    let right = descriptor(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        "binding-1",
        "winds-session-1",
        "PLANNER",
        RuntimeKind::Codex,
        TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
        TargetDimension::Exact(ExactModelId::new("gpt-x").unwrap()),
    );
    assert_eq!(
        left.canonical_json().unwrap(),
        right.canonical_json().unwrap()
    );
    assert_eq!(left.digest().unwrap(), right.digest().unwrap());
}

#[test]
fn t114_every_material_target_scope_dimension_moves_the_digest() {
    let base = basic_descriptor().digest().unwrap();
    let cases = [
        descriptor(
            "workspace-2",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-2",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-2",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-2",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-2",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-2",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "worker",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Claude,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Unspecified,
        ),
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Exact(ExactModelId::new("gpt-x").unwrap()),
        ),
    ];
    for candidate in cases {
        assert_ne!(base, candidate.digest().unwrap());
    }
}

#[test]
fn t114_unspecified_provider_and_model_stay_explicit_and_are_not_inferred_from_runtime() {
    let descriptor = basic_descriptor();
    let json = descriptor.canonical_json().unwrap();
    assert!(json.contains("\"provider\":{\"kind\":\"UNSPECIFIED\"}"));
    assert!(json.contains("\"model\":{\"kind\":\"UNSPECIFIED\"}"));
    assert!(!json.to_ascii_lowercase().contains("openai"));
    assert!(!json.to_ascii_lowercase().contains("anthropic"));
}

#[test]
fn t114_agent_report_cannot_satisfy_winds_observed_identity_requirement() {
    let request = TargetRequest::new(
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Codex,
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Unspecified,
        ),
        TargetSelector::Human,
    )
    .unwrap();
    let claims = [
        runtime_claim("CODEX", IdentitySourceClass::WindsLocallyObserved),
        provider_claim("openai", IdentitySourceClass::AgentReported),
    ];
    assert_eq!(
        resolve(
            &request,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
        ),
        TargetResolution::Unknown
    );
}

#[test]
fn t114_unavailable_ambiguous_and_conflicting_identity_never_selects_a_winner() {
    let request = TargetRequest::new(basic_descriptor(), TargetSelector::Human).unwrap();
    let unavailable = [IdentityClaim::new(
        IdentityDimension::Runtime,
        None,
        IdentitySourceClass::Unavailable,
        Some("not discovered"),
    )
    .unwrap()];
    assert_eq!(
        resolve(
            &request,
            &unavailable,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::Unavailable
    );

    let ambiguous = [
        runtime_claim("CODEX", IdentitySourceClass::WindsLocallyObserved),
        runtime_claim("CLAUDE", IdentitySourceClass::WindsLocallyObserved),
    ];
    assert_eq!(
        resolve(
            &request,
            &ambiguous,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::Ambiguous
    );

    let conflict = [
        runtime_claim("CODEX", IdentitySourceClass::WindsLocallyObserved),
        runtime_claim("CLAUDE", IdentitySourceClass::VendorDeclared),
    ];
    assert_eq!(
        resolve(
            &request,
            &conflict,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::Conflict
    );
}

#[test]
fn t114_explicit_policy_is_always_fail_closed_in_the_first_slice() {
    let request = TargetRequest::new(basic_descriptor(), TargetSelector::ExplicitPolicy).unwrap();
    let claims = [runtime_claim(
        "CODEX",
        IdentitySourceClass::WindsLocallyObserved,
    )];
    assert_eq!(
        resolve(
            &request,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::PolicyNotAuthorized
    );
}

#[test]
fn t114_negative_state_matrix_is_deterministic_and_fail_closed() {
    let request = TargetRequest::new(basic_descriptor(), TargetSelector::Human).unwrap();
    let exact = [runtime_claim(
        "CODEX",
        IdentitySourceClass::WindsLocallyObserved,
    )];
    let missing: [IdentityClaim; 0] = [];

    let cases = [
        (
            &missing[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::Unknown,
        ),
        (
            &exact[..],
            true,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::Stale,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Unknown,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::AuthenticationUnknown,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Unavailable,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::CapabilityUnavailable,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Missing,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::AuthorityDenied,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Mismatch,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::AuthorityDenied,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Stale,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::Stale,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Denied,
            TargetResolution::AuthorityDenied,
        ),
        (
            &exact[..],
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
            TargetResolution::ExactMatch,
        ),
    ];

    for (claims, stale, auth, capability, approval, authority, expected) in cases {
        let first = resolve(
            &request,
            claims,
            SourceRequirements::winds_observed(),
            stale,
            auth,
            capability,
            approval,
            authority,
        );
        let second = resolve(
            &request,
            claims,
            SourceRequirements::winds_observed(),
            stale,
            auth,
            capability,
            approval,
            authority,
        );
        assert_eq!(first, expected);
        assert_eq!(second, expected);
    }
}

#[test]
fn t114_exact_provider_and_model_require_their_qualified_claims() {
    let request = TargetRequest::new(
        descriptor(
            "workspace-1",
            "workstream-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "winds-session-1",
            "planner",
            RuntimeKind::Claude,
            TargetDimension::Exact(ExactProviderId::new("provider-a").unwrap()),
            TargetDimension::Exact(ExactModelId::new("model-a").unwrap()),
        ),
        TargetSelector::Human,
    )
    .unwrap();
    let claims = [
        runtime_claim("CLAUDE", IdentitySourceClass::WindsLocallyObserved),
        provider_claim("provider-a", IdentitySourceClass::WindsLocallyObserved),
        model_claim("model-a", IdentitySourceClass::WindsLocallyObserved),
    ];
    assert_eq!(
        resolve(
            &request,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::ExactMatch
    );
}

#[test]
fn t114_continuity_permission_and_authority_envelopes_are_content_bound() {
    let target = TargetRequest::new(basic_descriptor(), TargetSelector::Human).unwrap();
    let permission = ModelMeshContinuityPermissionDescriptorV1::new(
        "workflow-1",
        "stage-1",
        Some("binding-source"),
        Some("binding-1"),
        ContinuityClass::Handoff,
        target.target_descriptor_digest(),
        ContextDigest::exact(&"a".repeat(64)).unwrap(),
    )
    .unwrap();
    let permission_digest = permission.digest().unwrap();
    let envelope = ModelMeshAuthorityEnvelopeV1::new(
        ModelMeshAuthorityPurpose::ContinuityPermission,
        "workspace-1",
        "workstream-1",
        "winds-session-1",
        "workflow-1",
        "stage-1",
        "binding-1",
        "planner",
        target.target_descriptor_digest(),
        Some(&permission_digest),
    )
    .unwrap();
    assert_eq!(permission.digest().unwrap(), permission_digest);
    assert_eq!(envelope.digest().unwrap().len(), 64);

    let different_permission = ModelMeshContinuityPermissionDescriptorV1::new(
        "workflow-1",
        "stage-1",
        Some("binding-source"),
        Some("binding-1"),
        ContinuityClass::Reconstructed,
        target.target_descriptor_digest(),
        ContextDigest::exact(&"a".repeat(64)).unwrap(),
    )
    .unwrap();
    assert_ne!(permission_digest, different_permission.digest().unwrap());
}

#[test]
fn t114_authority_envelope_rejects_purpose_digest_shape_mismatch() {
    let target_digest = basic_descriptor().digest().unwrap();
    assert!(
        ModelMeshAuthorityEnvelopeV1::new(
            ModelMeshAuthorityPurpose::TargetSelection,
            "workspace-1",
            "workstream-1",
            "winds-session-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "planner",
            &target_digest,
            Some(&"b".repeat(64)),
        )
        .is_err()
    );
    assert!(
        ModelMeshAuthorityEnvelopeV1::new(
            ModelMeshAuthorityPurpose::ContinuityPermission,
            "workspace-1",
            "workstream-1",
            "winds-session-1",
            "workflow-1",
            "stage-1",
            "binding-1",
            "planner",
            &target_digest,
            None,
        )
        .is_err()
    );
}

#[test]
fn t114_inputs_are_bounded_and_reject_control_or_malformed_digest_content() {
    assert!(ExactProviderId::new("").is_err());
    assert!(ExactModelId::new("model\nname").is_err());
    assert!(
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("x\0y"),
            IdentitySourceClass::VendorDeclared,
            None
        )
        .is_err()
    );
    assert!(
        IdentityClaim::new(
            IdentityDimension::Provider,
            Some("x"),
            IdentitySourceClass::Unavailable,
            None
        )
        .is_err()
    );
    assert!(ContextDigest::exact("not-a-digest").is_err());
}

#[test]
fn t114_exact_match_is_only_target_resolution_not_verification_acceptance_or_landing() {
    let request = TargetRequest::new(basic_descriptor(), TargetSelector::Human).unwrap();
    let claims = [runtime_claim(
        "CODEX",
        IdentitySourceClass::WindsLocallyObserved,
    )];
    assert_eq!(
        resolve(
            &request,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed
        ),
        TargetResolution::ExactMatch
    );
    assert_eq!(request.selector(), TargetSelector::Human);
}

#[test]
fn t114_resolver_rejects_descriptor_digest_mismatch_even_for_otherwise_exact_truth() {
    let descriptor = basic_descriptor();
    let mismatched = TargetRequest::from_untrusted_parts_for_test(
        descriptor,
        &"f".repeat(64),
        TargetSelector::Human,
    )
    .expect("syntactically valid untrusted fixture");
    let claims = [runtime_claim(
        "CODEX",
        IdentitySourceClass::WindsLocallyObserved,
    )];

    assert_eq!(
        resolve(
            &mismatched,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
        ),
        TargetResolution::Stale
    );
}

#[test]
fn t114_descriptor_integrity_precedes_explicit_policy_resolution() {
    let mismatched = TargetRequest::from_untrusted_parts_for_test(
        basic_descriptor(),
        &"f".repeat(64),
        TargetSelector::ExplicitPolicy,
    )
    .expect("syntactically valid untrusted fixture");
    let claims = [runtime_claim(
        "CODEX",
        IdentitySourceClass::WindsLocallyObserved,
    )];

    assert_eq!(
        resolve(
            &mismatched,
            &claims,
            SourceRequirements::winds_observed(),
            false,
            AuthenticationTruth::Ready,
            CapabilityTruth::Available,
            ApprovalApplicability::Exact,
            CurrentAuthorityTruth::Allowed,
        ),
        TargetResolution::Stale
    );
}

#[test]
fn t114_native_session_identity_is_closed_vocabulary_but_not_a_target_dimension() {
    let native = IdentityClaim::new(
        IdentityDimension::NativeSession,
        Some("native-session-1"),
        IdentitySourceClass::VendorDeclared,
        Some("structured fixture"),
    )
    .unwrap();
    assert_eq!(native.dimension, IdentityDimension::NativeSession);
}

#[test]
fn t114_continuity_class_vocabulary_keeps_handoff_resume_and_reconstruction_distinct() {
    assert_ne!(ContinuityClass::Handoff, ContinuityClass::NativeResume);
    assert_ne!(
        ContinuityClass::Reconstructed,
        ContinuityClass::NativeResume
    );
    assert_ne!(ContinuityClass::Reassigned, ContinuityClass::NativeResume);
    assert_ne!(ContinuityClass::OwnershipLost, ContinuityClass::Unproven);
    assert_ne!(ContinuityClass::Unavailable, ContinuityClass::Unproven);
    assert_eq!(ContextDigest::NotApplicable, ContextDigest::NotApplicable);
}
