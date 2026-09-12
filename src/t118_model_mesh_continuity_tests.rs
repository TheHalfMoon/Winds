use crate::agentic_runtime::{
    EvidenceSource, RuntimeBindingOwnership, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeSessionBinding, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::WindsSessionRecord;
use crate::domain::workflow::{
    ReconstructionCategory, ReconstructionContentState, ReconstructionItem,
    ReconstructionSourceClass, ReconstructionTransferState, StageRunIdentity,
    WorkflowContinuationClass, WorkflowRunIdentity, build_reconstruction_preview,
};
use crate::model_mesh::{
    ContinuityClass, ContinuityContextCompleteness, ContinuityMaterialState, CurrentAuthorityTruth,
    ModelMeshAuthorityEnvelopeV1, ModelMeshContinuityActorV1, ModelMeshContinuityClassifierInput,
    ModelMeshContinuityContextInput, ModelMeshContinuityMarkerV1, ModelMeshContinuityReferenceV1,
    ModelMeshTargetDescriptorV1, TargetDimension, adapt_actor_scope,
    build_model_mesh_continuity_context, classify_model_mesh_continuity,
};
use crate::store::StoredWorkflowActorBinding;
use std::path::PathBuf;

fn target(runtime: RuntimeKind, actor: &str, session: &str) -> ModelMeshTargetDescriptorV1 {
    ModelMeshTargetDescriptorV1::new(
        "workspace-1",
        "workstream-1",
        "workflow-1",
        "stage-1",
        actor,
        session,
        "WORKER",
        runtime,
        TargetDimension::Unspecified,
        TargetDimension::Unspecified,
    )
    .unwrap()
}

fn runtime_binding(
    id: &str,
    session: &str,
    runtime: RuntimeKind,
    native: &str,
    ownership: RuntimeBindingOwnership,
) -> RuntimeSessionBinding {
    let executable = match runtime {
        RuntimeKind::Codex => PathBuf::from("/tmp/winds-t118-codex"),
        RuntimeKind::Claude => PathBuf::from("/tmp/winds-t118-claude"),
    };
    RuntimeSessionBinding {
        binding_id: id.to_owned(),
        session_id: session.to_owned(),
        runtime,
        executable: RuntimeExecutableIdentity {
            observed_path: executable.clone(),
            canonical_path: executable,
            byte_len: 118,
            sha256: match runtime {
                RuntimeKind::Codex => "a".repeat(64),
                RuntimeKind::Claude => "b".repeat(64),
            },
        },
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("t118-runtime-1".to_owned()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        native_session_id: Some(native.to_owned()),
        ownership,
        bound_unix_ms: 1,
        ownership_observed_unix_ms: match ownership {
            RuntimeBindingOwnership::Unproven => None,
            RuntimeBindingOwnership::OwnershipLost => Some(2),
        },
    }
}

fn workflow() -> WorkflowRunIdentity {
    WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap()
}

fn stage() -> StageRunIdentity {
    StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap()
}

fn winds_session(session_id: &str) -> WindsSessionRecord {
    WindsSessionRecord {
        session_id: session_id.to_owned(),
        workstream_id: "workstream-1".to_owned(),
        display_name: format!("T118 {session_id}"),
        created_unix_ms: 1,
        updated_unix_ms: 1,
    }
}

fn stored_actor(
    actor_id: &str,
    session_id: &str,
    runtime_binding_id: &str,
    continuation: WorkflowContinuationClass,
) -> StoredWorkflowActorBinding {
    StoredWorkflowActorBinding {
        binding_id: actor_id.to_owned(),
        stage_run_id: "stage-1".to_owned(),
        winds_session_id: session_id.to_owned(),
        runtime_binding_id: Some(runtime_binding_id.to_owned()),
        continuation,
        bound_unix_ms: 1,
        reconstruction_report: None,
    }
}

fn continuity_actor(
    actor_id: &str,
    session_id: &str,
    runtime: RuntimeKind,
    runtime_binding_id: &str,
    native: &str,
    ownership: RuntimeBindingOwnership,
    continuation: WorkflowContinuationClass,
) -> ModelMeshContinuityActorV1 {
    let binding = runtime_binding(runtime_binding_id, session_id, runtime, native, ownership);
    let scope = adapt_actor_scope(
        &workflow(),
        &stage(),
        &winds_session(session_id),
        &stored_actor(actor_id, session_id, runtime_binding_id, continuation),
        Some(&binding),
    )
    .unwrap();
    ModelMeshContinuityActorV1::from_actor_scope(&scope, &binding).unwrap()
}

fn reconstruction(actor: &str) -> crate::domain::workflow::ReconstructionReport {
    reconstruction_with_decision_state(
        actor,
        ReconstructionTransferState::PreservedReference,
        ReconstructionContentState::Full,
    )
}

fn reconstruction_with_decision_state(
    actor: &str,
    decision_transfer: ReconstructionTransferState,
    decision_content: ReconstructionContentState,
) -> crate::domain::workflow::ReconstructionReport {
    let mut items = Vec::new();
    for (category, reference) in [
        (
            ReconstructionCategory::CanonicalWorkContext,
            "canonical-work:1",
        ),
        (
            ReconstructionCategory::ObjectiveConstraints,
            "constraints:1",
        ),
        (
            ReconstructionCategory::CandidateEvidence,
            "candidate-evidence:1",
        ),
        (ReconstructionCategory::PriorStageOutputs, "prior-stage:1"),
        (
            ReconstructionCategory::RuntimeNativeContext,
            "runtime-native:1",
        ),
    ] {
        items.push(
            ReconstructionItem::new(
                category,
                ReconstructionSourceClass::StoredCanonicalReference,
                reference,
                ReconstructionTransferState::PreservedReference,
                ReconstructionContentState::Full,
            )
            .unwrap(),
        );
    }
    items.push(
        ReconstructionItem::new(
            ReconstructionCategory::Decisions,
            if matches!(
                decision_transfer,
                ReconstructionTransferState::Unavailable
                    | ReconstructionTransferState::NoLongerTransferable
            ) {
                ReconstructionSourceClass::Unavailable
            } else {
                ReconstructionSourceClass::StoredCanonicalReference
            },
            "decisions:1",
            decision_transfer,
            decision_content,
        )
        .unwrap(),
    );
    items.push(
        ReconstructionItem::new(
            ReconstructionCategory::ProviderPrivateState,
            ReconstructionSourceClass::Unavailable,
            "provider-private-state:unavailable",
            ReconstructionTransferState::Unavailable,
            ReconstructionContentState::Unavailable,
        )
        .unwrap(),
    );
    build_reconstruction_preview(actor, "stage-1", &items)
        .unwrap()
        .report
}

fn reference(id: &str, exact: &str) -> ModelMeshContinuityReferenceV1 {
    ModelMeshContinuityReferenceV1::new(id, exact).unwrap()
}

fn default_references() -> (
    Vec<ModelMeshContinuityReferenceV1>,
    Vec<ModelMeshContinuityReferenceV1>,
    Vec<ModelMeshContinuityReferenceV1>,
) {
    (
        vec![reference(
            "candidate.current",
            "oid:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/tree:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )],
        vec![reference(
            "artifact.binary",
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )],
        vec![reference(
            "evidence.quality",
            "run:1227/head:32e35a66cefca708ef9320a0191cfbbcf9ce3756",
        )],
    )
}

fn complete_context(
    target: &ModelMeshTargetDescriptorV1,
    continuity_class: ContinuityClass,
    source_actor: Option<&ModelMeshContinuityActorV1>,
    destination_actor: Option<&ModelMeshContinuityActorV1>,
    report: Option<&crate::domain::workflow::ReconstructionReport>,
    authority: CurrentAuthorityTruth,
) -> crate::model_mesh::ModelMeshContinuityContextV1 {
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(target).unwrap();
    let (candidates, artifacts, evidence) = default_references();
    build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target,
        continuity_class,
        source_actor,
        destination_actor,
        candidate_references: &candidates,
        artifact_references: &artifacts,
        evidence_references: &evidence,
        selection_approval: &approval,
        current_authority: authority,
        reconstruction_report: report,
        markers: &[],
    })
    .unwrap()
}

#[test]
fn t118_cross_runtime_directions_are_handoff_never_native_resume() {
    for (source_kind, destination_kind) in [
        (RuntimeKind::Codex, RuntimeKind::Claude),
        (RuntimeKind::Claude, RuntimeKind::Codex),
    ] {
        let descriptor = target(destination_kind, "actor-destination", "session-destination");
        let source = continuity_actor(
            "actor-source",
            "session-source",
            source_kind,
            "runtime-source",
            "native-same-looking-id",
            RuntimeBindingOwnership::Unproven,
            WorkflowContinuationClass::Unproven,
        );
        let destination = continuity_actor(
            "actor-destination",
            "session-destination",
            destination_kind,
            "runtime-destination",
            "native-same-looking-id",
            RuntimeBindingOwnership::Unproven,
            WorkflowContinuationClass::Reconstructed,
        );
        let report = reconstruction("actor-destination");
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&source),
            destination_actor: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Reconstructed,
            reconstruction_report: Some(&report),
        })
        .unwrap();
        assert_eq!(class, ContinuityClass::Handoff);
        assert_ne!(class, ContinuityClass::NativeResume);
    }
}

#[test]
fn t118_same_runtime_native_id_coincidence_and_resumed_label_remain_unproven() {
    for runtime in [RuntimeKind::Codex, RuntimeKind::Claude] {
        let descriptor = target(runtime, "actor-1", "session-1");
        let actor = continuity_actor(
            "actor-1",
            "session-1",
            runtime,
            "runtime-1",
            "native-id-1",
            RuntimeBindingOwnership::Unproven,
            WorkflowContinuationClass::Resumed,
        );
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&actor),
            destination_actor: Some(&actor),
            destination_continuation: WorkflowContinuationClass::Resumed,
            reconstruction_report: None,
        })
        .unwrap();
        assert_eq!(class, ContinuityClass::Unproven);
    }
}

#[test]
fn t118_reassignment_reconstruction_ownership_loss_and_unavailable_remain_distinct() {
    let descriptor = target(
        RuntimeKind::Codex,
        "actor-destination",
        "session-destination",
    );
    let source = continuity_actor(
        "actor-source",
        "session-source",
        RuntimeKind::Codex,
        "runtime-source",
        "native-source",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let destination = continuity_actor(
        "actor-destination",
        "session-destination",
        RuntimeKind::Codex,
        "runtime-destination",
        "native-destination",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&source),
            destination_actor: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unproven,
            reconstruction_report: None,
        })
        .unwrap(),
        ContinuityClass::Reassigned
    );

    let reconstructed = continuity_actor(
        "actor-destination",
        "session-destination",
        RuntimeKind::Codex,
        "runtime-destination",
        "native-destination",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Reconstructed,
    );
    let report = reconstruction("actor-destination");
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&reconstructed),
            destination_actor: Some(&reconstructed),
            destination_continuation: WorkflowContinuationClass::Reconstructed,
            reconstruction_report: Some(&report),
        })
        .unwrap(),
        ContinuityClass::Reconstructed
    );

    let lost = continuity_actor(
        "actor-source",
        "session-source",
        RuntimeKind::Codex,
        "runtime-source-lost",
        "native-source",
        RuntimeBindingOwnership::OwnershipLost,
        WorkflowContinuationClass::OwnershipLost,
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&lost),
            destination_actor: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unproven,
            reconstruction_report: None,
        })
        .unwrap(),
        ContinuityClass::OwnershipLost
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: None,
            destination_actor: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unavailable,
            reconstruction_report: None,
        })
        .unwrap(),
        ContinuityClass::Unavailable
    );
}

#[test]
fn t118_all_four_runtime_directions_project_deterministically_with_exact_provenance() {
    for (source_kind, destination_kind) in [
        (RuntimeKind::Codex, RuntimeKind::Claude),
        (RuntimeKind::Claude, RuntimeKind::Codex),
        (RuntimeKind::Codex, RuntimeKind::Codex),
        (RuntimeKind::Claude, RuntimeKind::Claude),
    ] {
        let cross_runtime = source_kind != destination_kind;
        let actor_id = if cross_runtime {
            "actor-destination"
        } else {
            "actor-same"
        };
        let descriptor = target(destination_kind, actor_id, "session-destination");
        let destination = continuity_actor(
            actor_id,
            "session-destination",
            destination_kind,
            "runtime-destination",
            "native-destination",
            RuntimeBindingOwnership::Unproven,
            WorkflowContinuationClass::Unproven,
        );
        let source = if cross_runtime {
            continuity_actor(
                "actor-source",
                "session-source",
                source_kind,
                "runtime-source",
                "native-source",
                RuntimeBindingOwnership::Unproven,
                WorkflowContinuationClass::Unproven,
            )
        } else {
            destination.clone()
        };
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor: Some(&source),
            destination_actor: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unproven,
            reconstruction_report: None,
        })
        .unwrap();
        assert_eq!(
            class,
            if cross_runtime {
                ContinuityClass::Handoff
            } else {
                ContinuityClass::Unproven
            }
        );
        let first = complete_context(
            &descriptor,
            class,
            Some(&source),
            Some(&destination),
            None,
            CurrentAuthorityTruth::Allowed,
        );
        let second = complete_context(
            &descriptor,
            class,
            Some(&source),
            Some(&destination),
            None,
            CurrentAuthorityTruth::Allowed,
        );
        assert_eq!(
            first.canonical_json().unwrap(),
            second.canonical_json().unwrap()
        );
        assert_eq!(first.digest().unwrap(), second.digest().unwrap());
        assert_eq!(first.continuity_class(), class.as_str());
        assert_eq!(first.source_runtime(), Some(source_kind.as_str()));
        assert_eq!(first.destination_runtime(), Some(destination_kind.as_str()));
        assert_eq!(
            first.completeness(),
            ContinuityContextCompleteness::Complete
        );
    }
}

#[test]
fn t118_context_digest_moves_on_material_structured_context_movement() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let baseline_actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let baseline = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some(&baseline_actor),
        Some(&baseline_actor),
        None,
        CurrentAuthorityTruth::Allowed,
    );
    let denied = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some(&baseline_actor),
        Some(&baseline_actor),
        None,
        CurrentAuthorityTruth::Denied,
    );
    assert_ne!(baseline.digest().unwrap(), denied.digest().unwrap());

    let moved_actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-2",
        "native-2",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let moved = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some(&moved_actor),
        Some(&moved_actor),
        None,
        CurrentAuthorityTruth::Allowed,
    );
    assert_ne!(baseline.digest().unwrap(), moved.digest().unwrap());
}

#[test]
fn t118_missing_redacted_and_material_loss_context_are_explicit_not_filled_from_prose() {
    let descriptor = target(RuntimeKind::Claude, "actor-1", "session-1");
    let actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Claude,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&descriptor).unwrap();
    let (candidates, artifacts, _) = default_references();
    let redacted = vec![
        ModelMeshContinuityMarkerV1::new(
            "required-decision",
            ContinuityMaterialState::Redacted,
            true,
        )
        .unwrap(),
    ];
    let context = build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target: &descriptor,
        continuity_class: ContinuityClass::Unproven,
        source_actor: Some(&actor),
        destination_actor: Some(&actor),
        candidate_references: &candidates,
        artifact_references: &artifacts,
        evidence_references: &[],
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Allowed,
        reconstruction_report: None,
        markers: &redacted,
    })
    .unwrap();
    assert_eq!(
        context.completeness(),
        ContinuityContextCompleteness::Redacted
    );

    let material_loss = vec![
        ModelMeshContinuityMarkerV1::new(
            "required-canonical-output",
            ContinuityMaterialState::MaterialLoss,
            true,
        )
        .unwrap(),
    ];
    let context = build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target: &descriptor,
        continuity_class: ContinuityClass::Unproven,
        source_actor: Some(&actor),
        destination_actor: Some(&actor),
        candidate_references: &candidates,
        artifact_references: &artifacts,
        evidence_references: &[],
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Allowed,
        reconstruction_report: None,
        markers: &material_loss,
    })
    .unwrap();
    assert_eq!(
        context.completeness(),
        ContinuityContextCompleteness::MaterialLoss
    );
}

#[test]
fn t118_provider_private_state_is_explicitly_unavailable_without_lowering_complete_context() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let context = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some(&actor),
        Some(&actor),
        None,
        CurrentAuthorityTruth::Allowed,
    );
    assert_eq!(
        context.completeness(),
        ContinuityContextCompleteness::Complete
    );
    assert!(context.markers().iter().any(|marker| {
        marker.item_id == "provider-private-state"
            && marker.state == ContinuityMaterialState::Unavailable
            && !marker.required
    }));
    let json = context.canonical_json().unwrap();
    assert!(!json.contains("private_hidden_state_reasoning"));
    assert!(!json.contains("winner_recommendation"));
    assert!(!json.contains("confidence"));
}

#[test]
fn t118_context_rejects_false_native_resume_wrong_direction_and_wrong_approval() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&descriptor).unwrap();
    let (candidates, artifacts, evidence) = default_references();
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-request-1",
            target: &descriptor,
            continuity_class: ContinuityClass::NativeResume,
            source_actor: Some(&actor),
            destination_actor: Some(&actor),
            candidate_references: &candidates,
            artifact_references: &artifacts,
            evidence_references: &evidence,
            selection_approval: &approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        })
        .is_err()
    );

    let claude_same_actor = continuity_actor(
        "actor-1",
        "session-source",
        RuntimeKind::Claude,
        "runtime-claude",
        "native-claude",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Unproven,
    );
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-request-1",
            target: &descriptor,
            continuity_class: ContinuityClass::Handoff,
            source_actor: Some(&claude_same_actor),
            destination_actor: Some(&actor),
            candidate_references: &candidates,
            artifact_references: &artifacts,
            evidence_references: &evidence,
            selection_approval: &approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        })
        .is_err()
    );

    let other = target(RuntimeKind::Codex, "actor-other", "session-other");
    let wrong_approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&other).unwrap();
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-request-1",
            target: &descriptor,
            continuity_class: ContinuityClass::Unproven,
            source_actor: Some(&actor),
            destination_actor: Some(&actor),
            candidate_references: &candidates,
            artifact_references: &artifacts,
            evidence_references: &evidence,
            selection_approval: &wrong_approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        })
        .is_err()
    );
}

#[test]
fn t118_reconstruction_source_payload_is_digest_bound_but_not_exposed_in_context() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Reconstructed,
    );
    let mut report = reconstruction("actor-1");
    report.items[0].source_reference = "sensitive-reconstruction-source-reference".to_owned();
    let context = complete_context(
        &descriptor,
        ContinuityClass::Reconstructed,
        Some(&actor),
        Some(&actor),
        Some(&report),
        CurrentAuthorityTruth::Allowed,
    );
    let json = context.canonical_json().unwrap();
    assert!(!json.contains("sensitive-reconstruction-source-reference"));
    assert!(json.contains("reconstruction_report_digest"));
    assert!(json.contains("reconstruction_summary"));
}

#[test]
fn t118_reference_input_rejects_secret_private_and_persuasive_prose() {
    for value in [
        "Authorization: Bearer abc",
        "provider-private memory blob",
        "token=abc",
        "confidence=0.99",
        "winner recommendation: model-x",
        "persuasion: trust me",
    ] {
        assert!(ModelMeshContinuityReferenceV1::new("unsafe", value).is_err());
    }
}

#[test]
fn t118_actor_runtime_binding_pairing_is_canonical_and_cannot_be_swapped() {
    let canonical_binding = runtime_binding(
        "runtime-canonical",
        "session-1",
        RuntimeKind::Codex,
        "native-1",
        RuntimeBindingOwnership::Unproven,
    );
    let scope = adapt_actor_scope(
        &workflow(),
        &stage(),
        &winds_session("session-1"),
        &stored_actor(
            "actor-1",
            "session-1",
            "runtime-canonical",
            WorkflowContinuationClass::Unproven,
        ),
        Some(&canonical_binding),
    )
    .unwrap();
    let swapped_binding = runtime_binding(
        "runtime-swapped",
        "session-1",
        RuntimeKind::Codex,
        "native-1",
        RuntimeBindingOwnership::Unproven,
    );
    assert!(ModelMeshContinuityActorV1::from_actor_scope(&scope, &swapped_binding).is_err());
    assert!(ModelMeshContinuityActorV1::from_actor_scope(&scope, &canonical_binding).is_ok());
}

#[test]
fn t118_reconstruction_loss_states_prevent_complete_context() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let actor = continuity_actor(
        "actor-1",
        "session-1",
        RuntimeKind::Codex,
        "runtime-1",
        "native-1",
        RuntimeBindingOwnership::Unproven,
        WorkflowContinuationClass::Reconstructed,
    );
    for (transfer, content, expected) in [
        (
            ReconstructionTransferState::PreservedReference,
            ReconstructionContentState::Redacted,
            ContinuityContextCompleteness::Redacted,
        ),
        (
            ReconstructionTransferState::Omitted,
            ReconstructionContentState::Omitted,
            ContinuityContextCompleteness::Omitted,
        ),
        (
            ReconstructionTransferState::Unavailable,
            ReconstructionContentState::Unavailable,
            ContinuityContextCompleteness::Unavailable,
        ),
        (
            ReconstructionTransferState::NoLongerTransferable,
            ReconstructionContentState::Unavailable,
            ContinuityContextCompleteness::MaterialLoss,
        ),
    ] {
        let report = reconstruction_with_decision_state("actor-1", transfer, content);
        let context = complete_context(
            &descriptor,
            ContinuityClass::Reconstructed,
            Some(&actor),
            Some(&actor),
            Some(&report),
            CurrentAuthorityTruth::Allowed,
        );
        assert_eq!(context.completeness(), expected);
        assert_ne!(
            context.completeness(),
            ContinuityContextCompleteness::Complete
        );
    }
}
