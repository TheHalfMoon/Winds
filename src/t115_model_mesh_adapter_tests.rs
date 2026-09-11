use crate::agentic_authority::{StoredApproval, revalidate_loaded_model_mesh_approval};
use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, AuthReadinessEvidence, EvidenceSource,
    RuntimeBindingOwnership, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity,
    RuntimeKind, RuntimeSessionBinding, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::WindsSessionRecord;
use crate::domain::workflow::{StageRunIdentity, WorkflowContinuationClass, WorkflowRunIdentity};
use crate::model_mesh::{
    ApprovalApplicability, AuthenticationTruth, CapabilityTruth, ContextDigest, ContinuityClass,
    CurrentAuthorityTruth, ExactProviderId, IdentityClaim, IdentityDimension, IdentitySourceClass,
    ModelMeshAuthorityEnvelopeV1, ModelMeshContinuityPermissionDescriptorV1,
    ModelMeshTargetDescriptorV1, SourceRequirements, TargetDimension, TargetRequest,
    TargetResolution, TargetResolverInput, TargetSelector, adapt_actor_scope, adapt_runtime_truth,
    resolve_target,
};
use crate::store::StoredWorkflowActorBinding;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn executable(hash_byte: char) -> RuntimeExecutableIdentity {
    RuntimeExecutableIdentity {
        observed_path: PathBuf::from("/opt/winds/bin/runtime"),
        canonical_path: PathBuf::from("/opt/winds/bin/runtime"),
        byte_len: 42,
        sha256: hash_byte.to_string().repeat(64),
    }
}

fn discovery(runtime: RuntimeKind) -> RuntimeDiscovery {
    RuntimeDiscovery {
        runtime,
        state: RuntimeDiscoveryState::Present,
        executable: Some(executable('a')),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("1.0.0".into()),
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

fn runtime_binding(runtime: RuntimeKind) -> RuntimeSessionBinding {
    RuntimeSessionBinding {
        binding_id: "runtime-binding-1".into(),
        session_id: "session-1".into(),
        runtime,
        executable: executable('a'),
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("1.0.0".into()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        native_session_id: Some("native-1".into()),
        ownership: RuntimeBindingOwnership::Unproven,
        bound_unix_ms: 10,
        ownership_observed_unix_ms: None,
    }
}

fn workflow() -> WorkflowRunIdentity {
    WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap()
}

fn stage() -> StageRunIdentity {
    StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap()
}

fn session() -> WindsSessionRecord {
    WindsSessionRecord {
        session_id: "session-1".into(),
        workstream_id: "workstream-1".into(),
        display_name: "Session".into(),
        created_unix_ms: 1,
        updated_unix_ms: 1,
    }
}

fn actor() -> StoredWorkflowActorBinding {
    StoredWorkflowActorBinding {
        binding_id: "actor-binding-1".into(),
        stage_run_id: "stage-1".into(),
        winds_session_id: "session-1".into(),
        runtime_binding_id: Some("runtime-binding-1".into()),
        continuation: WorkflowContinuationClass::Unproven,
        bound_unix_ms: 10,
        reconstruction_report: None,
    }
}

fn target_descriptor() -> ModelMeshTargetDescriptorV1 {
    adapt_actor_scope(
        &workflow(),
        &stage(),
        &session(),
        &actor(),
        Some(&runtime_binding(RuntimeKind::Codex)),
    )
    .unwrap()
    .target_descriptor(
        "worker",
        RuntimeKind::Codex,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
    .unwrap()
}

fn target_envelope(descriptor: &ModelMeshTargetDescriptorV1) -> ModelMeshAuthorityEnvelopeV1 {
    ModelMeshAuthorityEnvelopeV1::new_target_selection(
        descriptor.workspace_id(),
        descriptor.workstream_id(),
        descriptor.winds_session_id(),
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        descriptor.actor_binding_id(),
        descriptor.actor_role(),
        &descriptor.digest().unwrap(),
    )
    .unwrap()
}

fn stored_for(envelope: &ModelMeshAuthorityEnvelopeV1) -> StoredApproval {
    let canonical = envelope.canonical_json().unwrap();
    StoredApproval {
        approval_id: "approval-1".into(),
        workstream_id: envelope.workstream_id().into(),
        session_id: envelope.session_id().into(),
        workspace_id: envelope.workspace_id().into(),
        content_digest: sha256_hex(&canonical),
        canonical_content_json: canonical,
        approved_unix_ms: 20,
    }
}

#[test]
fn t115_runtime_adapter_never_infers_provider_or_model_from_codex_or_claude() {
    for runtime in [RuntimeKind::Codex, RuntimeKind::Claude] {
        let binding = runtime_binding(runtime);
        let observed = adapt_runtime_truth(&discovery(runtime), Some(&binding)).unwrap();
        assert_eq!(observed.runtime_claim.dimension, IdentityDimension::Runtime);
        assert_eq!(
            observed.runtime_claim.source,
            IdentitySourceClass::WindsLocallyObserved
        );
        assert_eq!(
            observed.runtime_claim.value.as_deref(),
            Some(runtime.as_str())
        );
        assert!(observed.provider_claim.is_none());
        assert!(observed.model_claim.is_none());
        assert_eq!(observed.authentication, AuthenticationTruth::Unknown);
        assert!(!observed.binding_stale);
    }
}

#[test]
fn t115_explicit_provider_stays_unknown_without_an_accepted_provider_observation() {
    let scope = adapt_actor_scope(
        &workflow(),
        &stage(),
        &session(),
        &actor(),
        Some(&runtime_binding(RuntimeKind::Codex)),
    )
    .unwrap();
    let descriptor = scope
        .target_descriptor(
            "worker",
            RuntimeKind::Codex,
            TargetDimension::Exact(ExactProviderId::new("openai").unwrap()),
            TargetDimension::Unspecified,
        )
        .unwrap();
    let request = TargetRequest::new(descriptor, TargetSelector::Human).unwrap();
    let observed = adapt_runtime_truth(
        &discovery(RuntimeKind::Codex),
        Some(&runtime_binding(RuntimeKind::Codex)),
    )
    .unwrap();
    let claims = [observed.runtime_claim];

    assert_eq!(
        resolve_target(&TargetResolverInput {
            request: &request,
            claims: &claims,
            source_requirements: SourceRequirements::winds_observed(),
            stale: observed.binding_stale,
            authentication: AuthenticationTruth::Ready,
            capability: CapabilityTruth::Available,
            approval: ApprovalApplicability::Exact,
            current_authority: CurrentAuthorityTruth::Allowed,
        }),
        TargetResolution::Unknown
    );
}

#[test]
fn t115_runtime_binding_movement_is_stale_without_manufacturing_identity() {
    let discovery = discovery(RuntimeKind::Codex);
    let mut binding = runtime_binding(RuntimeKind::Codex);
    binding.executable = executable('b');
    let observed = adapt_runtime_truth(&discovery, Some(&binding)).unwrap();
    assert!(observed.binding_stale);
    assert!(observed.provider_claim.is_none());
    assert!(observed.model_claim.is_none());
}

#[test]
fn t115_unavailable_runtime_remains_unavailable_and_authentication_unknown() {
    let unavailable = RuntimeDiscovery {
        runtime: RuntimeKind::Claude,
        state: RuntimeDiscoveryState::Unavailable,
        executable: None,
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Unavailable,
            value: None,
            source: EvidenceSource::Unavailable,
        },
        capabilities: Vec::new(),
        auth_readiness: AuthReadinessEvidence {
            readiness: AuthReadiness::Unknown,
            source: EvidenceSource::Unavailable,
        },
        agent_execution: AgentExecutionObservation::NotPerformed,
    };
    let observed = adapt_runtime_truth(&unavailable, None).unwrap();
    assert_eq!(
        observed.runtime_claim.source,
        IdentitySourceClass::Unavailable
    );
    assert_eq!(observed.runtime_claim.value, None);
    assert_eq!(observed.authentication, AuthenticationTruth::Unknown);
}

#[test]
fn t115_actor_scope_reconstructs_only_exact_canonical_work_session_and_binding_truth() {
    let binding = runtime_binding(RuntimeKind::Codex);
    let scope =
        adapt_actor_scope(&workflow(), &stage(), &session(), &actor(), Some(&binding)).unwrap();
    let descriptor = scope
        .target_descriptor(
            "worker",
            RuntimeKind::Codex,
            TargetDimension::Unspecified,
            TargetDimension::Unspecified,
        )
        .unwrap();
    assert_eq!(descriptor.workspace_id(), "workspace-1");
    assert_eq!(descriptor.workstream_id(), "workstream-1");
    assert_eq!(descriptor.workflow_run_id(), "workflow-1");
    assert_eq!(descriptor.stage_run_id(), "stage-1");
    assert_eq!(descriptor.actor_binding_id(), "actor-binding-1");
    assert_eq!(descriptor.winds_session_id(), "session-1");
    assert_eq!(descriptor.actor_role(), "WORKER");
}

#[test]
fn t115_actor_scope_cannot_relabel_a_bound_codex_actor_as_claude() {
    let binding = runtime_binding(RuntimeKind::Codex);
    let scope =
        adapt_actor_scope(&workflow(), &stage(), &session(), &actor(), Some(&binding)).unwrap();
    assert!(
        scope
            .target_descriptor(
                "worker",
                RuntimeKind::Claude,
                TargetDimension::Unspecified,
                TargetDimension::Unspecified,
            )
            .is_err()
    );
}

#[test]
fn t115_actor_scope_rejects_cross_stage_workstream_session_and_runtime_binding_confusion() {
    let binding = runtime_binding(RuntimeKind::Codex);

    let wrong_stage = StageRunIdentity::new("stage-1", "other-workflow", "build", 1, None).unwrap();
    assert!(
        adapt_actor_scope(
            &workflow(),
            &wrong_stage,
            &session(),
            &actor(),
            Some(&binding)
        )
        .is_err()
    );

    let mut wrong_session = session();
    wrong_session.workstream_id = "other-workstream".into();
    assert!(
        adapt_actor_scope(
            &workflow(),
            &stage(),
            &wrong_session,
            &actor(),
            Some(&binding)
        )
        .is_err()
    );

    let mut wrong_binding = binding.clone();
    wrong_binding.binding_id = "runtime-binding-2".into();
    assert!(
        adapt_actor_scope(
            &workflow(),
            &stage(),
            &session(),
            &actor(),
            Some(&wrong_binding)
        )
        .is_err()
    );
}

#[test]
fn t115_exact_model_mesh_approval_revalidates_without_expanding_authority() {
    let descriptor = target_descriptor();
    let envelope = target_envelope(&descriptor);
    let stored = stored_for(&envelope);
    let approval = revalidate_loaded_model_mesh_approval(&stored, &envelope);
    assert_eq!(approval, ApprovalApplicability::Exact);

    let request = TargetRequest::new(descriptor, TargetSelector::Human).unwrap();
    let claims = [IdentityClaim::new(
        IdentityDimension::Runtime,
        Some("CODEX"),
        IdentitySourceClass::WindsLocallyObserved,
        Some("accepted runtime discovery"),
    )
    .unwrap()];
    assert_eq!(
        resolve_target(&TargetResolverInput {
            request: &request,
            claims: &claims,
            source_requirements: SourceRequirements::winds_observed(),
            stale: false,
            authentication: AuthenticationTruth::Ready,
            capability: CapabilityTruth::Available,
            approval,
            current_authority: CurrentAuthorityTruth::Denied,
        }),
        TargetResolution::AuthorityDenied
    );
}

#[test]
fn t115_approval_for_another_exact_target_cannot_authorize_this_target() {
    let descriptor = target_descriptor();
    let expected = target_envelope(&descriptor);
    let other_descriptor = ModelMeshTargetDescriptorV1::new(
        descriptor.workspace_id(),
        descriptor.workstream_id(),
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        descriptor.actor_binding_id(),
        descriptor.winds_session_id(),
        "planner",
        RuntimeKind::Codex,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
    .unwrap();
    let stored = stored_for(&target_envelope(&other_descriptor));
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&stored, &expected),
        ApprovalApplicability::Mismatch
    );
}

#[test]
fn t115_stale_digest_malformed_generic_and_wrong_schema_approvals_fail_closed() {
    let envelope = target_envelope(&target_descriptor());

    let mut stale = stored_for(&envelope);
    stale.content_digest = "0".repeat(64);
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&stale, &envelope),
        ApprovalApplicability::Stale
    );

    for canonical in [
        "not-json".to_owned(),
        r#"{"schema_version":2,"purpose":"TARGET_SELECTION","workspace_id":"workspace-1","workstream_id":"workstream-1","session_id":"session-1","workflow_run_id":"workflow-1","stage_run_id":"stage-1","actor_binding_id":"actor-binding-1","actor_role":"WORKER","target_descriptor_digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","continuity_permission_digest":null}"#.to_owned(),
        r#"{"schema_version":1,"workspace_id":"workspace-1","workstream_id":"workstream-1","session_id":"session-1","runtime_kind":"CODEX","target":{"capability":"run","resource":"repo"}}"#.to_owned(),
    ] {
        let stored = StoredApproval {
            approval_id: "generic".into(),
            workstream_id: "workstream-1".into(),
            session_id: "session-1".into(),
            workspace_id: "workspace-1".into(),
            content_digest: sha256_hex(&canonical),
            canonical_content_json: canonical,
            approved_unix_ms: 20,
        };
        assert_eq!(
            revalidate_loaded_model_mesh_approval(&stored, &envelope),
            ApprovalApplicability::Mismatch
        );
    }
}

#[test]
fn t115_stored_approval_audit_scope_must_match_the_canonical_envelope() {
    let envelope = target_envelope(&target_descriptor());
    let mut stored = stored_for(&envelope);
    stored.session_id = "other-session".into();
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&stored, &envelope),
        ApprovalApplicability::Mismatch
    );
}

#[test]
fn t115_continuity_authority_is_derived_from_exact_permission_scope_and_target() {
    let descriptor = target_descriptor();
    let target_digest = descriptor.digest().unwrap();
    let permission = ModelMeshContinuityPermissionDescriptorV1::new(
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        Some("actor-binding-source"),
        Some(descriptor.actor_binding_id()),
        ContinuityClass::Handoff,
        &target_digest,
        ContextDigest::exact(&"a".repeat(64)).unwrap(),
    )
    .unwrap();
    let envelope = ModelMeshAuthorityEnvelopeV1::new_continuity_permission(
        descriptor.workspace_id(),
        descriptor.workstream_id(),
        descriptor.winds_session_id(),
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        descriptor.actor_binding_id(),
        descriptor.actor_role(),
        &target_digest,
        &permission,
    )
    .unwrap();
    assert_eq!(
        envelope.continuity_permission_digest(),
        Some(permission.digest().unwrap().as_str())
    );

    let wrong_workflow = ModelMeshContinuityPermissionDescriptorV1::new(
        "workflow-other",
        descriptor.stage_run_id(),
        None,
        Some(descriptor.actor_binding_id()),
        ContinuityClass::Handoff,
        &target_digest,
        ContextDigest::NotApplicable,
    )
    .unwrap();
    assert!(
        ModelMeshAuthorityEnvelopeV1::new_continuity_permission(
            descriptor.workspace_id(),
            descriptor.workstream_id(),
            descriptor.winds_session_id(),
            descriptor.workflow_run_id(),
            descriptor.stage_run_id(),
            descriptor.actor_binding_id(),
            descriptor.actor_role(),
            &target_digest,
            &wrong_workflow,
        )
        .is_err()
    );

    let wrong_stage = ModelMeshContinuityPermissionDescriptorV1::new(
        descriptor.workflow_run_id(),
        "stage-other",
        None,
        Some(descriptor.actor_binding_id()),
        ContinuityClass::Handoff,
        &target_digest,
        ContextDigest::NotApplicable,
    )
    .unwrap();
    assert!(
        ModelMeshAuthorityEnvelopeV1::new_continuity_permission(
            descriptor.workspace_id(),
            descriptor.workstream_id(),
            descriptor.winds_session_id(),
            descriptor.workflow_run_id(),
            descriptor.stage_run_id(),
            descriptor.actor_binding_id(),
            descriptor.actor_role(),
            &target_digest,
            &wrong_stage,
        )
        .is_err()
    );

    let wrong_target = ModelMeshContinuityPermissionDescriptorV1::new(
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        None,
        Some(descriptor.actor_binding_id()),
        ContinuityClass::Handoff,
        &"b".repeat(64),
        ContextDigest::NotApplicable,
    )
    .unwrap();
    assert!(
        ModelMeshAuthorityEnvelopeV1::new_continuity_permission(
            descriptor.workspace_id(),
            descriptor.workstream_id(),
            descriptor.winds_session_id(),
            descriptor.workflow_run_id(),
            descriptor.stage_run_id(),
            descriptor.actor_binding_id(),
            descriptor.actor_role(),
            &target_digest,
            &wrong_target,
        )
        .is_err()
    );
}

#[test]
fn t115_loaded_continuity_approval_cannot_swap_in_an_unrelated_permission_digest() {
    let descriptor = target_descriptor();
    let target_digest = descriptor.digest().unwrap();
    let permission = ModelMeshContinuityPermissionDescriptorV1::new(
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        None,
        Some(descriptor.actor_binding_id()),
        ContinuityClass::Handoff,
        &target_digest,
        ContextDigest::NotApplicable,
    )
    .unwrap();
    let expected = ModelMeshAuthorityEnvelopeV1::new_continuity_permission(
        descriptor.workspace_id(),
        descriptor.workstream_id(),
        descriptor.winds_session_id(),
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        descriptor.actor_binding_id(),
        descriptor.actor_role(),
        &target_digest,
        &permission,
    )
    .unwrap();
    let swapped_json = format!(
        r#"{{"schema_version":1,"purpose":"CONTINUITY_PERMISSION","workspace_id":"{}","workstream_id":"{}","session_id":"{}","workflow_run_id":"{}","stage_run_id":"{}","actor_binding_id":"{}","actor_role":"{}","target_descriptor_digest":"{}","continuity_permission_digest":"{}"}}"#,
        descriptor.workspace_id(),
        descriptor.workstream_id(),
        descriptor.winds_session_id(),
        descriptor.workflow_run_id(),
        descriptor.stage_run_id(),
        descriptor.actor_binding_id(),
        descriptor.actor_role(),
        target_digest,
        "c".repeat(64),
    );
    let stored = StoredApproval {
        approval_id: "approval-swapped".into(),
        workstream_id: descriptor.workstream_id().into(),
        session_id: descriptor.winds_session_id().into(),
        workspace_id: descriptor.workspace_id().into(),
        content_digest: sha256_hex(&swapped_json),
        canonical_content_json: swapped_json,
        approved_unix_ms: 20,
    };
    assert_eq!(
        revalidate_loaded_model_mesh_approval(&stored, &expected),
        ApprovalApplicability::Mismatch
    );
}
