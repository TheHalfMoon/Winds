use super::*;
use crate::persistent_runtime::client::{
    ClientEventProjection, ClientResponseProjection, LocalControlClientError,
};
use crate::persistent_runtime::domain::{
    ClientAuthority, ContinuityClass, EndpointAvailability, EventSequence, OwnershipState,
    ProcessLiveness, RuntimeAlias, RuntimeNamespaceId, RuntimeTruth,
};

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn sequence(value: u64) -> EventSequence {
    EventSequence::new(value).unwrap()
}

fn truth(
    ownership: OwnershipState,
    liveness: ProcessLiveness,
    continuity: ContinuityClass,
) -> RuntimeTruth {
    RuntimeTruth {
        ownership,
        process_liveness: liveness,
        endpoint_availability: EndpointAvailability::Available,
        continuity,
    }
}

fn input(
    runtime_namespace_id: RuntimeNamespaceId,
    runtime_truth: RuntimeTruth,
    truth_source: RuntimeTruthSource,
) -> RuntimePresentationInput {
    RuntimePresentationInput {
        runtime_namespace_id,
        display_alias: Some(RuntimeAlias::new("planner").unwrap()),
        truth: Some(runtime_truth),
        truth_source,
        authority: Some(ClientAuthority::Observer),
        replay_state: RuntimeReplayStateInput::Complete,
        client_condition: RuntimeClientCondition::Ready,
    }
}

#[test]
fn t156_live_retained_process_requires_winds_observed_live_owned_running_truth() {
    let runtime_id = runtime(1);
    let projection = project_runtime_truth(&input(
        runtime_id,
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    ));

    assert_eq!(
        projection.continuity,
        RuntimeContinuityPresentation::LiveRetainedProcess
    );
    assert_eq!(
        projection.ownership,
        RuntimeOwnershipPresentation::LiveOwned
    );
    assert_eq!(projection.liveness, RuntimeLivenessPresentation::Running);
    assert_eq!(
        project_workbench_runtime(&projection).headline,
        "Live retained process"
    );
}

#[test]
fn t156_canonical_metadata_cannot_recreate_live_ownership_or_running_truth() {
    let projection = project_runtime_truth(&input(
        runtime(18),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::AcceptedCanonical,
    ));
    assert_eq!(projection.ownership, RuntimeOwnershipPresentation::Unknown);
    assert_eq!(projection.liveness, RuntimeLivenessPresentation::Unknown);
    assert_eq!(
        projection.continuity,
        RuntimeContinuityPresentation::Unknown
    );
    assert_eq!(projection.health, RuntimeHealthPresentation::ProcessUnknown);
}

#[test]
fn t156_native_resume_is_distinct_from_retained_process_and_requires_accepted_source() {
    let runtime_id = runtime(2);
    let accepted = project_runtime_truth(&input(
        runtime_id,
        truth(
            OwnershipState::Unowned,
            ProcessLiveness::Exited,
            ContinuityClass::ProviderNativeResume,
        ),
        RuntimeTruthSource::AcceptedCanonical,
    ));
    assert_eq!(
        accepted.continuity,
        RuntimeContinuityPresentation::ProviderNativeResume
    );
    assert_ne!(
        accepted.continuity,
        RuntimeContinuityPresentation::LiveRetainedProcess
    );

    let agent_claim = project_runtime_truth(&input(
        runtime_id,
        truth(
            OwnershipState::Unowned,
            ProcessLiveness::Exited,
            ContinuityClass::ProviderNativeResume,
        ),
        RuntimeTruthSource::AgentReported,
    ));
    assert_eq!(
        agent_claim.continuity,
        RuntimeContinuityPresentation::ClaimedProviderNativeResume
    );
    assert_eq!(agent_claim.truth_source, RuntimeTruthSource::AgentReported);
    assert_eq!(
        project_cli_runtime(&agent_claim).continuity,
        "CLAIMED_PROVIDER_NATIVE_RESUME"
    );
}

#[test]
fn t156_winds_reconstruction_and_fresh_process_never_render_as_native_resume() {
    for (byte, continuity, expected) in [
        (
            3,
            ContinuityClass::WindsReconstruction,
            RuntimeContinuityPresentation::WindsReconstruction,
        ),
        (
            4,
            ContinuityClass::FreshProcess,
            RuntimeContinuityPresentation::FreshProcess,
        ),
    ] {
        let projection = project_runtime_truth(&input(
            runtime(byte),
            truth(
                OwnershipState::Unowned,
                ProcessLiveness::Running,
                continuity,
            ),
            RuntimeTruthSource::WindsObserved,
        ));
        assert_eq!(projection.continuity, expected);
        assert_ne!(
            projection.continuity,
            RuntimeContinuityPresentation::ProviderNativeResume
        );
    }
}

#[test]
fn t156_ownership_loss_is_never_hidden_by_a_generic_connected_state() {
    let projection = project_runtime_truth(&input(
        runtime(5),
        truth(
            OwnershipState::OwnershipLost,
            ProcessLiveness::Unknown,
            ContinuityClass::Unknown,
        ),
        RuntimeTruthSource::WindsObserved,
    ));

    assert_eq!(projection.health, RuntimeHealthPresentation::OwnershipLost);
    assert_eq!(project_cli_runtime(&projection).health, "OWNERSHIP_LOST");
    assert_eq!(
        project_workbench_runtime(&projection).headline,
        "Ownership lost"
    );
}

#[test]
fn t156_unknown_and_unavailable_process_liveness_are_explicit() {
    let unknown = project_runtime_truth(&input(
        runtime(6),
        truth(
            OwnershipState::Unowned,
            ProcessLiveness::Unknown,
            ContinuityClass::Unknown,
        ),
        RuntimeTruthSource::Unknown,
    ));
    assert_eq!(unknown.health, RuntimeHealthPresentation::ProcessUnknown);

    let unavailable = project_runtime_truth(&input(
        runtime(7),
        truth(
            OwnershipState::Unowned,
            ProcessLiveness::Unavailable,
            ContinuityClass::Unavailable,
        ),
        RuntimeTruthSource::WindsObserved,
    ));
    assert_eq!(
        unavailable.health,
        RuntimeHealthPresentation::ProcessUnavailable
    );
    assert_eq!(
        unavailable.continuity,
        RuntimeContinuityPresentation::Unavailable
    );
}

#[test]
fn t156_protocol_generation_and_outcome_unknown_are_distinct_client_states() {
    let base = input(
        runtime(8),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    );

    for (condition, expected) in [
        (
            RuntimeClientCondition::ProtocolIncompatible,
            RuntimeHealthPresentation::ProtocolIncompatible,
        ),
        (
            RuntimeClientCondition::OwnerGenerationMismatch,
            RuntimeHealthPresentation::OwnerGenerationMismatch,
        ),
        (
            RuntimeClientCondition::OutcomeUnknown,
            RuntimeHealthPresentation::OutcomeUnknown,
        ),
    ] {
        let mut input = base.clone();
        input.client_condition = condition;
        assert_eq!(project_runtime_truth(&input).health, expected);
    }
}

#[test]
fn t156_observer_and_controller_states_are_explicit_and_focus_never_implies_control() {
    let mut observer_input = input(
        runtime(9),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    );
    observer_input.authority = Some(ClientAuthority::Observer);
    let observer = project_runtime_truth(&observer_input);
    assert_eq!(observer.authority, RuntimeAuthorityPresentation::Observer);

    observer_input.authority = Some(ClientAuthority::Controller);
    let controller = project_runtime_truth(&observer_input);
    assert_eq!(
        controller.authority,
        RuntimeAuthorityPresentation::Controller
    );
}

#[test]
fn t156_replay_gap_surfaces_exact_truncation_bounds_across_cli_and_desktop() {
    let mut runtime_input = input(
        runtime(10),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    );
    runtime_input.replay_state = RuntimeReplayStateInput::Gap(RuntimeReplayGap {
        first_available_sequence: sequence(51),
        last_dropped_sequence: sequence(50),
    });

    let projection = project_runtime_truth(&runtime_input);
    assert_eq!(
        projection.health,
        RuntimeHealthPresentation::ReplayTruncated
    );
    assert!(matches!(
        projection.replay,
        RuntimeReplayPresentation::Truncated(_)
    ));

    let cli = project_cli_runtime(&projection);
    assert_eq!(cli.replay, "TRUNCATED_GAP");
    assert_eq!(cli.replay_first_available_sequence, Some(51));
    assert_eq!(cli.replay_last_dropped_sequence, Some(50));

    let desktop = project_desktop_runtime(&projection);
    assert_eq!(desktop.replay, "TRUNCATED_GAP");
    assert_eq!(desktop.replay_first_available_sequence, Some(51));
    assert_eq!(desktop.replay_last_dropped_sequence, Some(50));
}

#[test]
fn t156_client_error_mapping_preserves_protocol_generation_outcome_and_disconnect_distinctions() {
    assert_eq!(
        client_condition_from_error(&LocalControlClientError::ProtocolMismatch),
        RuntimeClientCondition::ProtocolIncompatible
    );
    assert_eq!(
        client_condition_from_error(&LocalControlClientError::StaleOwnerGeneration),
        RuntimeClientCondition::OwnerGenerationMismatch
    );
    assert_eq!(
        client_condition_from_error(&LocalControlClientError::OutcomeUnknown(runtime(11))),
        RuntimeClientCondition::OutcomeUnknown
    );
    assert_eq!(
        client_condition_from_error(&LocalControlClientError::Disconnected),
        RuntimeClientCondition::Disconnected
    );
}

#[test]
fn t156_client_response_and_event_adapters_preserve_exact_runtime_identity() {
    let runtime_id = runtime(12);
    let runtime_truth = truth(
        OwnershipState::LiveOwned,
        ProcessLiveness::Running,
        ContinuityClass::RetainedLiveProcess,
    );
    let snapshot = ClientResponseProjection::RuntimeSnapshot {
        runtime_namespace_id: runtime_id,
        truth: runtime_truth.clone(),
    };
    assert_eq!(
        runtime_truth_from_response(&snapshot),
        Some((runtime_id, runtime_truth))
    );

    let control = ClientResponseProjection::ControlState {
        runtime_namespace_id: runtime_id,
        authority: ClientAuthority::Controller,
        controller_client_id: None,
    };
    assert_eq!(
        authority_from_response(&control),
        Some((runtime_id, ClientAuthority::Controller))
    );

    let gap = ClientEventProjection::HistoryGap {
        runtime_namespace_id: runtime_id,
        first_available_sequence: sequence(101),
        last_dropped_sequence: sequence(100),
    };
    assert_eq!(
        replay_gap_from_event(&gap),
        Some((
            runtime_id,
            RuntimeReplayGap {
                first_available_sequence: sequence(101),
                last_dropped_sequence: sequence(100),
            }
        ))
    );
}

#[test]
fn t156_display_alias_never_replaces_immutable_runtime_target() {
    let runtime_id = runtime(13);
    let projection = project_runtime_truth(&input(
        runtime_id,
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    ));

    let cli = project_cli_runtime(&projection);
    let workbench = project_workbench_runtime(&projection);
    let desktop = project_desktop_runtime(&projection);
    let desktop_bridge = crate::desktop::desktop_bridge_persistent_runtime_truth(desktop.clone());

    assert_eq!(cli.runtime_namespace_id, runtime_id.to_string());
    assert_eq!(workbench.immutable_runtime_id, runtime_id.to_string());
    assert_eq!(desktop.runtime_namespace_id, runtime_id.to_string());
    assert_eq!(desktop_bridge.runtime_namespace_id, runtime_id.to_string());
    assert_eq!(cli.display_alias.as_deref(), Some("planner"));
    assert_ne!(cli.runtime_namespace_id, "planner");
}

#[test]
fn t156_process_exit_never_projects_verification_or_human_acceptance() {
    let projection = project_runtime_truth(&input(
        runtime(14),
        truth(
            OwnershipState::Unowned,
            ProcessLiveness::Exited,
            ContinuityClass::FreshProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    ));
    assert_eq!(projection.verification, RuntimeOutcomeClaim::NotProjected);
    assert_eq!(
        projection.human_acceptance,
        RuntimeOutcomeClaim::NotProjected
    );

    let cli = project_cli_runtime(&projection);
    assert_eq!(cli.verification, "NOT_PROJECTED");
    assert_eq!(cli.human_acceptance, "NOT_PROJECTED");
}

#[test]
fn t156_agent_reported_retained_process_cannot_become_winds_observed_live_continuity() {
    let projection = project_runtime_truth(&input(
        runtime(15),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::AgentReported,
    ));

    assert_eq!(
        projection.continuity,
        RuntimeContinuityPresentation::Unknown
    );
    assert_eq!(projection.ownership, RuntimeOwnershipPresentation::Unknown);
    assert_eq!(projection.liveness, RuntimeLivenessPresentation::Unknown);
    assert_eq!(projection.health, RuntimeHealthPresentation::ProcessUnknown);
    assert_eq!(projection.truth_source, RuntimeTruthSource::AgentReported);
    assert_eq!(
        project_cli_runtime(&projection).truth_source,
        "AGENT_REPORTED"
    );
}

#[test]
fn t156_absent_replay_evidence_is_unknown_not_optimistically_complete() {
    let mut runtime_input = input(
        runtime(17),
        truth(
            OwnershipState::LiveOwned,
            ProcessLiveness::Running,
            ContinuityClass::RetainedLiveProcess,
        ),
        RuntimeTruthSource::WindsObserved,
    );
    runtime_input.replay_state = RuntimeReplayStateInput::Unknown;
    let projection = project_runtime_truth(&runtime_input);
    assert_eq!(projection.replay, RuntimeReplayPresentation::Unknown);
    assert_eq!(project_cli_runtime(&projection).replay, "UNKNOWN");
}

#[test]
fn t156_desktop_projection_is_bounded_typed_data_without_generic_connected_or_authority_claims() {
    let projection = project_runtime_truth(&input(
        runtime(16),
        truth(
            OwnershipState::OwnershipLost,
            ProcessLiveness::Unknown,
            ContinuityClass::Unknown,
        ),
        RuntimeTruthSource::WindsObserved,
    ));
    let desktop = project_desktop_runtime(&projection);
    let value = serde_json::to_value(&desktop).unwrap();
    let text = value.to_string();

    assert_eq!(desktop.health, "OWNERSHIP_LOST");
    assert!(!text.contains("\"connected\""));
    assert!(!text.contains("VERIFIED"));
    assert!(!text.contains("ACCEPTED"));
    assert!(text.len() < 4096);
}
