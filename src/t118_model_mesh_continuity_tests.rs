use crate::agentic_runtime::{
    EvidenceSource, RuntimeBindingOwnership, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeSessionBinding, RuntimeVersionEvidence, RuntimeVersionState,
};
use crate::domain::workflow::{
    ReconstructionCategory, ReconstructionContentState, ReconstructionItem,
    ReconstructionSourceClass, ReconstructionTransferState, WorkflowContinuationClass,
    build_reconstruction_preview,
};
use crate::model_mesh::{
    ContinuityClass, ContinuityContextCompleteness, ContinuityMaterialState, CurrentAuthorityTruth,
    ModelMeshAuthorityEnvelopeV1, ModelMeshContinuityClassifierInput,
    ModelMeshContinuityContextInput, ModelMeshContinuityMarkerV1, ModelMeshContinuityReferenceV1,
    ModelMeshTargetDescriptorV1, TargetDimension, build_model_mesh_continuity_context,
    classify_model_mesh_continuity,
};
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

fn reconstruction(actor: &str) -> crate::domain::workflow::ReconstructionReport {
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
        (ReconstructionCategory::Decisions, "decisions:1"),
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

#[allow(clippy::too_many_arguments)]
fn complete_context(
    target: &ModelMeshTargetDescriptorV1,
    continuity_class: ContinuityClass,
    source_actor: Option<&str>,
    destination_actor: Option<&str>,
    source_runtime: Option<RuntimeKind>,
    destination_runtime: Option<RuntimeKind>,
    report: Option<&crate::domain::workflow::ReconstructionReport>,
    authority: CurrentAuthorityTruth,
) -> crate::model_mesh::ModelMeshContinuityContextV1 {
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(target).unwrap();
    let source_binding = source_runtime.map(|runtime| {
        let session = if source_actor == destination_actor {
            target.winds_session_id()
        } else {
            "session-source-context"
        };
        runtime_binding(
            "context-source-runtime",
            session,
            runtime,
            "context-source-native",
            RuntimeBindingOwnership::Unproven,
        )
    });
    let destination_binding = destination_runtime.map(|runtime| {
        runtime_binding(
            "context-destination-runtime",
            target.winds_session_id(),
            runtime,
            "context-destination-native",
            RuntimeBindingOwnership::Unproven,
        )
    });
    let candidates = vec![reference(
        "candidate.current",
        "oid:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/tree:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )];
    let artifacts = vec![reference(
        "artifact.binary",
        "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    )];
    let evidence = vec![reference(
        "evidence.quality",
        "run:1227/head:32e35a66cefca708ef9320a0191cfbbcf9ce3756",
    )];
    build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target,
        continuity_class,
        source_actor_binding_id: source_actor,
        destination_actor_binding_id: destination_actor,
        source_runtime_binding: source_binding.as_ref(),
        destination_runtime_binding: destination_binding.as_ref(),
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
        let source = runtime_binding(
            "runtime-source",
            "session-source",
            source_kind,
            "native-same-looking-id",
            RuntimeBindingOwnership::Unproven,
        );
        let destination = runtime_binding(
            "runtime-destination",
            "session-destination",
            destination_kind,
            "native-same-looking-id",
            RuntimeBindingOwnership::Unproven,
        );
        let report = reconstruction("actor-destination");
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: Some("actor-source"),
            destination_actor_binding_id: Some("actor-destination"),
            source_runtime_binding: Some(&source),
            destination_runtime_binding: Some(&destination),
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
        let source = runtime_binding(
            "runtime-source",
            "session-1",
            runtime,
            "native-id-1",
            RuntimeBindingOwnership::Unproven,
        );
        let destination = runtime_binding(
            "runtime-destination",
            "session-1",
            runtime,
            "native-id-1",
            RuntimeBindingOwnership::Unproven,
        );
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: Some("actor-1"),
            destination_actor_binding_id: Some("actor-1"),
            source_runtime_binding: Some(&source),
            destination_runtime_binding: Some(&destination),
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
    let source = runtime_binding(
        "runtime-source",
        "session-source",
        RuntimeKind::Codex,
        "native-source",
        RuntimeBindingOwnership::Unproven,
    );
    let destination = runtime_binding(
        "runtime-destination",
        "session-destination",
        RuntimeKind::Codex,
        "native-destination",
        RuntimeBindingOwnership::Unproven,
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: Some("actor-source"),
            destination_actor_binding_id: Some("actor-destination"),
            source_runtime_binding: Some(&source),
            destination_runtime_binding: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unproven,
            reconstruction_report: None,
        })
        .unwrap(),
        ContinuityClass::Reassigned
    );

    let same_actor_descriptor = target(
        RuntimeKind::Codex,
        "actor-destination",
        "session-destination",
    );
    let report = reconstruction("actor-destination");
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &same_actor_descriptor,
            source_actor_binding_id: Some("actor-destination"),
            destination_actor_binding_id: Some("actor-destination"),
            source_runtime_binding: Some(&source),
            destination_runtime_binding: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Reconstructed,
            reconstruction_report: Some(&report),
        })
        .unwrap(),
        ContinuityClass::Reconstructed
    );

    let lost = runtime_binding(
        "runtime-source-lost",
        "session-source",
        RuntimeKind::Codex,
        "native-source",
        RuntimeBindingOwnership::OwnershipLost,
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: Some("actor-source"),
            destination_actor_binding_id: Some("actor-destination"),
            source_runtime_binding: Some(&lost),
            destination_runtime_binding: Some(&destination),
            destination_continuation: WorkflowContinuationClass::Unproven,
            reconstruction_report: None,
        })
        .unwrap(),
        ContinuityClass::OwnershipLost
    );
    assert_eq!(
        classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: None,
            destination_actor_binding_id: Some("actor-destination"),
            source_runtime_binding: None,
            destination_runtime_binding: Some(&destination),
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
        let actor = if cross_runtime {
            "actor-destination"
        } else {
            "actor-same"
        };
        let descriptor = target(destination_kind, actor, "session-destination");
        let source = runtime_binding(
            "runtime-source",
            "session-source",
            source_kind,
            "native-source",
            RuntimeBindingOwnership::Unproven,
        );
        let destination = runtime_binding(
            "runtime-destination",
            "session-destination",
            destination_kind,
            "native-destination",
            RuntimeBindingOwnership::Unproven,
        );
        let source_actor = if cross_runtime { "actor-source" } else { actor };
        let class = classify_model_mesh_continuity(&ModelMeshContinuityClassifierInput {
            target: &descriptor,
            source_actor_binding_id: Some(source_actor),
            destination_actor_binding_id: Some(actor),
            source_runtime_binding: Some(&source),
            destination_runtime_binding: Some(&destination),
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
            Some(source_actor),
            Some(actor),
            Some(source_kind),
            Some(destination_kind),
            None,
            CurrentAuthorityTruth::Allowed,
        );
        let second = complete_context(
            &descriptor,
            class,
            Some(source_actor),
            Some(actor),
            Some(source_kind),
            Some(destination_kind),
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
    let baseline = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some("actor-1"),
        Some("actor-1"),
        Some(RuntimeKind::Codex),
        Some(RuntimeKind::Codex),
        None,
        CurrentAuthorityTruth::Allowed,
    );
    let denied = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some("actor-1"),
        Some("actor-1"),
        Some(RuntimeKind::Codex),
        Some(RuntimeKind::Codex),
        None,
        CurrentAuthorityTruth::Denied,
    );
    assert_ne!(baseline.digest().unwrap(), denied.digest().unwrap());

    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&descriptor).unwrap();
    let candidates = vec![reference("candidate.current", "oid:changed/tree:bbbb")];
    let artifacts = vec![reference("artifact.binary", "sha256:cccc")];
    let evidence = vec![reference("evidence.quality", "run:changed")];
    let changed_source_binding = runtime_binding(
        "changed-source-runtime",
        "session-1",
        RuntimeKind::Codex,
        "changed-source-native",
        RuntimeBindingOwnership::Unproven,
    );
    let changed_destination_binding = runtime_binding(
        "changed-destination-runtime",
        "session-1",
        RuntimeKind::Codex,
        "changed-destination-native",
        RuntimeBindingOwnership::Unproven,
    );
    let changed = build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
        target_request_id: "target-request-1",
        target: &descriptor,
        continuity_class: ContinuityClass::Unproven,
        source_actor_binding_id: Some("actor-1"),
        destination_actor_binding_id: Some("actor-1"),
        source_runtime_binding: Some(&changed_source_binding),
        destination_runtime_binding: Some(&changed_destination_binding),
        candidate_references: &candidates,
        artifact_references: &artifacts,
        evidence_references: &evidence,
        selection_approval: &approval,
        current_authority: CurrentAuthorityTruth::Allowed,
        reconstruction_report: None,
        markers: &[],
    })
    .unwrap();
    assert_ne!(baseline.digest().unwrap(), changed.digest().unwrap());
}

#[test]
fn t118_missing_redacted_and_material_loss_context_are_explicit_not_filled_from_prose() {
    let descriptor = target(RuntimeKind::Claude, "actor-1", "session-1");
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&descriptor).unwrap();
    let candidates = vec![reference("candidate.current", "oid:aaaa/tree:bbbb")];
    let artifacts = vec![reference("artifact.binary", "sha256:cccc")];
    let source_binding = runtime_binding(
        "context-source-runtime",
        "session-1",
        RuntimeKind::Claude,
        "source-native",
        RuntimeBindingOwnership::Unproven,
    );
    let destination_binding = runtime_binding(
        "context-destination-runtime",
        "session-1",
        RuntimeKind::Claude,
        "destination-native",
        RuntimeBindingOwnership::Unproven,
    );
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
        source_actor_binding_id: Some("actor-1"),
        destination_actor_binding_id: Some("actor-1"),
        source_runtime_binding: Some(&source_binding),
        destination_runtime_binding: Some(&destination_binding),
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
        markers: &material_loss,
        evidence_references: &[],
        ..ModelMeshContinuityContextInput {
            target_request_id: "target-request-1",
            target: &descriptor,
            continuity_class: ContinuityClass::Unproven,
            source_actor_binding_id: Some("actor-1"),
            destination_actor_binding_id: Some("actor-1"),
            source_runtime_binding: Some(&source_binding),
            destination_runtime_binding: Some(&destination_binding),
            candidate_references: &candidates,
            artifact_references: &artifacts,
            evidence_references: &[],
            selection_approval: &approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        }
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
    let context = complete_context(
        &descriptor,
        ContinuityClass::Unproven,
        Some("actor-1"),
        Some("actor-1"),
        Some(RuntimeKind::Codex),
        Some(RuntimeKind::Codex),
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
    let approval = ModelMeshAuthorityEnvelopeV1::for_target_selection(&descriptor).unwrap();
    let references = vec![reference("ref", "sha256:aaaa")];
    let codex_source = runtime_binding(
        "context-source-runtime",
        "session-1",
        RuntimeKind::Codex,
        "source-native",
        RuntimeBindingOwnership::Unproven,
    );
    let codex_destination = runtime_binding(
        "context-destination-runtime",
        "session-1",
        RuntimeKind::Codex,
        "destination-native",
        RuntimeBindingOwnership::Unproven,
    );
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-request-1",
            target: &descriptor,
            continuity_class: ContinuityClass::NativeResume,
            source_actor_binding_id: Some("actor-1"),
            destination_actor_binding_id: Some("actor-1"),
            source_runtime_binding: Some(&codex_source),
            destination_runtime_binding: Some(&codex_destination),
            candidate_references: &references,
            artifact_references: &references,
            evidence_references: &references,
            selection_approval: &approval,
            current_authority: CurrentAuthorityTruth::Allowed,
            reconstruction_report: None,
            markers: &[],
        })
        .is_err()
    );

    let claude_source = runtime_binding(
        "context-claude-source",
        "session-source",
        RuntimeKind::Claude,
        "claude-native",
        RuntimeBindingOwnership::Unproven,
    );
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            continuity_class: ContinuityClass::Unproven,
            source_runtime_binding: Some(&claude_source),
            ..ModelMeshContinuityContextInput {
                target_request_id: "target-request-1",
                target: &descriptor,
                continuity_class: ContinuityClass::Unproven,
                source_actor_binding_id: Some("actor-1"),
                destination_actor_binding_id: Some("actor-1"),
                source_runtime_binding: Some(&codex_source),
                destination_runtime_binding: Some(&codex_destination),
                candidate_references: &references,
                artifact_references: &references,
                evidence_references: &references,
                selection_approval: &approval,
                current_authority: CurrentAuthorityTruth::Allowed,
                reconstruction_report: None,
                markers: &[],
            }
        })
        .is_err()
    );

    let claude_destination_target = target(RuntimeKind::Claude, "actor-1", "session-1");
    let claude_destination = runtime_binding(
        "context-claude-destination",
        "session-1",
        RuntimeKind::Claude,
        "claude-destination-native",
        RuntimeBindingOwnership::Unproven,
    );
    let claude_approval =
        ModelMeshAuthorityEnvelopeV1::for_target_selection(&claude_destination_target).unwrap();
    assert!(
        build_model_mesh_continuity_context(&ModelMeshContinuityContextInput {
            target_request_id: "target-request-cross-runtime-same-actor",
            target: &claude_destination_target,
            continuity_class: ContinuityClass::Handoff,
            source_actor_binding_id: Some("actor-1"),
            destination_actor_binding_id: Some("actor-1"),
            source_runtime_binding: Some(&codex_source),
            destination_runtime_binding: Some(&claude_destination),
            candidate_references: &references,
            artifact_references: &references,
            evidence_references: &references,
            selection_approval: &claude_approval,
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
            selection_approval: &wrong_approval,
            ..ModelMeshContinuityContextInput {
                target_request_id: "target-request-1",
                target: &descriptor,
                continuity_class: ContinuityClass::Unproven,
                source_actor_binding_id: Some("actor-1"),
                destination_actor_binding_id: Some("actor-1"),
                source_runtime_binding: Some(&codex_source),
                destination_runtime_binding: Some(&codex_destination),
                candidate_references: &references,
                artifact_references: &references,
                evidence_references: &references,
                selection_approval: &approval,
                current_authority: CurrentAuthorityTruth::Allowed,
                reconstruction_report: None,
                markers: &[],
            }
        })
        .is_err()
    );
}
#[test]
fn t118_reconstruction_source_payload_is_digest_bound_but_not_exposed_in_context() {
    let descriptor = target(RuntimeKind::Codex, "actor-1", "session-1");
    let mut report = reconstruction("actor-1");
    report.items[0].source_reference = "secret-token-like-reconstruction-source".to_owned();
    let context = complete_context(
        &descriptor,
        ContinuityClass::Reconstructed,
        Some("actor-1"),
        Some("actor-1"),
        Some(RuntimeKind::Codex),
        Some(RuntimeKind::Codex),
        Some(&report),
        CurrentAuthorityTruth::Allowed,
    );
    let json = context.canonical_json().unwrap();
    assert!(!json.contains("secret-token-like-reconstruction-source"));
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
