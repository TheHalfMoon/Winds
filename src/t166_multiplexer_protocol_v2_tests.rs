use super::*;
use crate::multiplexer::domain::navigation::{
    LayoutNode, MultiplexerTopology, SplitAxis as DomainSplitAxis, SplitRatioBps, TabState,
    WorkspaceState,
};
use crate::multiplexer::domain::{
    AgentObservationId, ClientSurfaceCapability, MultiplexerAuthority, MultiplexerErrorKind,
    MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId, RuntimeNamespaceId,
};
use crate::persistent_runtime::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationCursorV2, AgentObservationEventV2,
    AgentObservationFreshnessV2, AgentObservationSnapshotV2, AgentObservationSourceV2,
    AgentObservationV2, ApplyTopologyOperationV2, ApplyWorktreeOperationV2, AttentionEventV2,
    AttentionItemV2, AttentionKindV2, AttentionSnapshotV2, LEGACY_PROTOCOL_VERSION,
    ListAgentObservationsV2, ListMultiplexerWorkspacesV2, ListWorktreesV2, MAX_CONTROL_FRAME_BYTES,
    MAX_INBOUND_CONTROL_FRAME_BYTES, MAX_V2_AGENT_OBSERVATIONS_PER_PAGE, MAX_V2_ALIAS_BYTES,
    MAX_V2_ATTENTION_ITEMS_PER_PAGE, MAX_V2_BRANCH_BYTES, MAX_V2_DETAIL_BYTES,
    MAX_V2_EVIDENCE_SUMMARY_BYTES, MAX_V2_GIT_WORKSPACE_ID_BYTES, MAX_V2_PANES_PER_TAB,
    MAX_V2_PATH_BYTES, MAX_V2_PROVIDER_SESSION_ID_BYTES, MAX_V2_REPOSITORY_IDENTITY_BYTES,
    MAX_V2_TABS_PER_WORKSPACE, MAX_V2_WORKSPACES, MAX_V2_WORKTREES_PER_PAGE, MessageAuthorityClass,
    MessageKind, MultiplexerEventSubscriptionAckV2, MultiplexerEventV2, MultiplexerSnapshotV2,
    MultiplexerSubscriptionBoundaryV2, MultiplexerSubscriptionStreamV2, MultiplexerWriteStateV2,
    PROTOCOL_VERSION, ProtocolLayoutNodeV2, ProtocolPanePlacement, ProtocolSplitAxis,
    ProtocolTabSnapshotV2, ProtocolWorkspaceSnapshotV2, ProtocolWorkspaceSummaryV2,
    RequestMultiplexerWriteV2, SubscribeMultiplexerEventsV2, TopologyMutationOutcomeV2,
    TopologyMutationResultV2, TopologyOperationV2, WorktreeCursorV2, WorktreeMembershipV2,
    WorktreeObservationV2, WorktreeOperationOutcomeV2, WorktreeOperationResultV2,
    WorktreeOperationTargetV2, WorktreeOperationV2, apply_topology_operation_v2, decode_frame,
    encode_frame, inactive_v2_domain_response, is_legacy_protocol_frame,
    multiplexer_snapshot_for_request_v2, validate_candidate_topology_v2,
    validate_multiplexer_subscription_event_binding, validate_owner_v2_handshake,
    validate_response_binding,
};
use crate::persistent_runtime::protocol::{ProtocolMessage, ProtocolPayload};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn sequence(value: u64) -> EventSequence {
    EventSequence::new(value).unwrap()
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane_index(value: u16) -> PaneId {
    let mut bytes = [0_u8; 16];
    bytes[..2].copy_from_slice(&value.to_be_bytes());
    bytes[15] = 1;
    PaneId::from_entropy_bytes(bytes).unwrap()
}

fn tab_index(value: u16) -> TabId {
    let mut bytes = [0_u8; 16];
    bytes[..2].copy_from_slice(&value.to_be_bytes());
    bytes[15] = 2;
    TabId::from_entropy_bytes(bytes).unwrap()
}

fn observation_index(value: u16) -> AgentObservationId {
    let mut bytes = [0_u8; 16];
    bytes[..2].copy_from_slice(&value.to_be_bytes());
    bytes[15] = 3;
    AgentObservationId::from_entropy_bytes(bytes).unwrap()
}

fn observation(byte: u8) -> AgentObservationId {
    AgentObservationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn topology_generation(value: u64) -> TopologyGeneration {
    TopologyGeneration::new(value).unwrap()
}

fn connection(value: &str) -> ClientConnectionId {
    ClientConnectionId::new(value).unwrap()
}

fn framed_json(json: &str) -> Vec<u8> {
    let bytes = json.as_bytes();
    let mut frame = Vec::with_capacity(4 + bytes.len());
    frame.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    frame.extend_from_slice(bytes);
    frame
}

fn v2_message(payload: ProtocolPayload) -> ProtocolMessage {
    ProtocolMessage::new(
        connection("t166-client"),
        sequence(7),
        None,
        generation(2),
        None,
        payload,
    )
    .unwrap()
}

fn worktree(index: usize) -> WorktreeObservationV2 {
    WorktreeObservationV2 {
        git_workspace_id: format!(
            "{index:03}{}",
            "g".repeat(MAX_V2_GIT_WORKSPACE_ID_BYTES.saturating_sub(3))
        ),
        repository_identity: "r".repeat(MAX_V2_REPOSITORY_IDENTITY_BYTES),
        canonical_path: format!("/{}", "p".repeat(MAX_V2_PATH_BYTES - 1)),
        branch: Some("b".repeat(MAX_V2_BRANCH_BYTES)),
        head_oid: "a".repeat(64),
        dirty: false,
        membership: WorktreeMembershipV2::ExplicitMember,
    }
}

#[test]
fn t166_exact_v2_handshake_preserves_legacy_fixture_without_downgrade() {
    assert_eq!(PROTOCOL_VERSION, 2);
    assert_eq!(LEGACY_PROTOCOL_VERSION, 1);

    let current = ProtocolMessage::hello(sequence(1), 2, 2, None).unwrap();
    validate_owner_v2_handshake(&current).unwrap();
    let current_frame = encode_frame(&current).unwrap();
    let current_json = std::str::from_utf8(&current_frame[4..]).unwrap();
    assert!(current_json.contains(r#""protocol_version":2"#));
    assert_eq!(decode_frame(&current_frame).unwrap(), current);

    let legacy = ProtocolMessage::hello(sequence(2), 1, 1, None).unwrap();
    assert_eq!(
        validate_owner_v2_handshake(&legacy).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );
    let legacy_frame = encode_frame(&legacy).unwrap();
    let legacy_json = std::str::from_utf8(&legacy_frame[4..]).unwrap();
    assert!(legacy_json.contains(r#""protocol_version":1"#));
    assert_eq!(decode_frame(&legacy_frame).unwrap(), legacy);

    assert_eq!(
        ProtocolMessage::hello(sequence(3), 1, 2, None).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );

    let legacy_ping = framed_json(
        r#"{"protocol_version":1,"message_kind":"PING","connection_id":"legacy-client","sequence":4,"owner_generation_id":"02020202020202020202020202020202","body":{}}"#,
    );
    assert_eq!(
        decode_frame(&legacy_ping).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );
}

/// The exact owner response a live protocol-v1 owner returns for a v2 HELLO: a typed
/// protocol-mismatch error correlated to the rejected request.
fn owner_protocol_mismatch_response() -> ProtocolMessage {
    ProtocolMessage::new(
        connection("legacy-owner"),
        sequence(9),
        None,
        generation(2),
        Some(sequence(8)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::ProtocolMismatch,
        },
    )
    .unwrap()
}

/// Re-encodes `message` at the legacy wire version, reproducing the exact frame shape a
/// still-running protocol-v1 owner emits: the shared envelope schema carrying
/// `protocol_version: 1`.
fn reencode_at_legacy_version(message: &ProtocolMessage) -> Vec<u8> {
    let frame = encode_frame(message).unwrap();
    let json = std::str::from_utf8(&frame[4..]).unwrap().replacen(
        &format!(r#""protocol_version":{PROTOCOL_VERSION}"#),
        &format!(r#""protocol_version":{LEGACY_PROTOCOL_VERSION}"#),
        1,
    );
    framed_json(&json)
}

#[test]
fn t166_live_v1_owner_wire_frame_derives_blocked_legacy_owner_before_downgrade() {
    let legacy_owner_response = reencode_at_legacy_version(&owner_protocol_mismatch_response());

    // A live v1 owner answer is decoded as a protocol mismatch on the v2 client...
    assert_eq!(
        decode_frame(&legacy_owner_response).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );
    // ...and that mismatch is attributed to a live legacy owner, not to a malformed peer.
    assert!(is_legacy_protocol_frame(&legacy_owner_response));
    assert!(matches!(
        decode_platform_frame(&legacy_owner_response),
        Err(WireError::LegacyProtocol)
    ));
    assert_eq!(
        map_wire_error(WireError::LegacyProtocol),
        LocalControlClientError::BlockedLegacyOwner
    );

    // The preserved v1 HELLO fixture still resolves through the decoder itself, so it is
    // never misattributed to legacy-owner blocking.
    let legacy_hello =
        encode_frame(&ProtocolMessage::hello(sequence(2), 1, 1, None).unwrap()).unwrap();
    assert!(is_legacy_protocol_frame(&legacy_hello));
    assert!(decode_platform_frame(&legacy_hello).is_ok());
}

#[test]
fn t166_legacy_owner_detection_is_limited_to_structurally_valid_legacy_frames() {
    // A current-version frame is never reported as legacy, and it decodes normally.
    let current = encode_frame(&owner_protocol_mismatch_response()).unwrap();
    assert!(!is_legacy_protocol_frame(&current));
    assert!(decode_platform_frame(&current).is_ok());

    // Every structurally invalid shape stays a malformed/oversized transport concern
    // rather than being promoted to a legacy-owner block.
    assert!(!is_legacy_protocol_frame(&[]));
    assert!(!is_legacy_protocol_frame(&4_u32.to_le_bytes()));

    let mut truncated_payload = 32_u32.to_le_bytes().to_vec();
    truncated_payload.extend_from_slice(br#"{"protocol_version":1}"#);
    assert!(!is_legacy_protocol_frame(&truncated_payload));

    let mut lying_length = 4_096_u32.to_le_bytes().to_vec();
    lying_length.extend_from_slice(br#"{"protocol_version":1}"#);
    assert!(!is_legacy_protocol_frame(&lying_length));

    assert!(!is_legacy_protocol_frame(&framed_json("not-json")));
    assert!(!is_legacy_protocol_frame(&framed_json(
        r#"{"protocol_version":3,"message_kind":"PING"}"#
    )));
}

#[test]
fn t166_total_frame_ceiling_is_exactly_256_kib_including_prefix() {
    assert_eq!(MAX_CONTROL_FRAME_BYTES, 262_144);
    assert_eq!(MAX_INBOUND_CONTROL_FRAME_BYTES, 262_140);

    let legal = vec![b' '; MAX_INBOUND_CONTROL_FRAME_BYTES];
    let mut legal_frame = Vec::with_capacity(MAX_CONTROL_FRAME_BYTES);
    legal_frame.extend_from_slice(&(legal.len() as u32).to_le_bytes());
    legal_frame.extend_from_slice(&legal);
    assert_eq!(legal_frame.len(), MAX_CONTROL_FRAME_BYTES);
    assert_eq!(
        decode_frame(&legal_frame).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let oversized = vec![b' '; MAX_INBOUND_CONTROL_FRAME_BYTES + 1];
    let mut oversized_frame = Vec::with_capacity(MAX_CONTROL_FRAME_BYTES + 1);
    oversized_frame.extend_from_slice(&(oversized.len() as u32).to_le_bytes());
    oversized_frame.extend_from_slice(&oversized);
    assert_eq!(
        decode_frame(&oversized_frame).unwrap_err(),
        LocalControlErrorKind::OversizedFrame
    );
}

#[test]
fn t166_closed_v2_message_vocabulary_has_no_generic_rpc_method() {
    let kinds = [
        MessageKind::ListMultiplexerWorkspaces,
        MessageKind::MultiplexerSnapshot,
        MessageKind::RequestMultiplexerWrite,
        MessageKind::ReleaseMultiplexerWrite,
        MessageKind::MultiplexerWriteState,
        MessageKind::ApplyTopologyOperation,
        MessageKind::MultiplexerEvent,
        MessageKind::SubscribeMultiplexerEvents,
        MessageKind::MultiplexerEventSubscriptionAck,
        MessageKind::ListAgentObservations,
        MessageKind::AgentObservationSnapshot,
        MessageKind::AgentObservationEvent,
        MessageKind::ListWorktrees,
        MessageKind::ApplyWorktreeOperation,
        MessageKind::WorktreeOperationResult,
        MessageKind::AttentionSnapshot,
        MessageKind::AttentionEvent,
    ];
    let encoded = serde_json::to_string(&kinds).unwrap();
    for expected in [
        "LIST_MULTIPLEXER_WORKSPACES",
        "MULTIPLEXER_SNAPSHOT",
        "REQUEST_MULTIPLEXER_WRITE",
        "RELEASE_MULTIPLEXER_WRITE",
        "MULTIPLEXER_WRITE_STATE",
        "APPLY_TOPOLOGY_OPERATION",
        "MULTIPLEXER_EVENT",
        "SUBSCRIBE_MULTIPLEXER_EVENTS",
        "MULTIPLEXER_EVENT_SUBSCRIPTION_ACK",
        "LIST_AGENT_OBSERVATIONS",
        "AGENT_OBSERVATION_SNAPSHOT",
        "AGENT_OBSERVATION_EVENT",
        "LIST_WORKTREES",
        "APPLY_WORKTREE_OPERATION",
        "WORKTREE_OPERATION_RESULT",
        "ATTENTION_SNAPSHOT",
        "ATTENTION_EVENT",
    ] {
        assert!(encoded.contains(expected));
    }
    assert!(!encoded.contains("method"));
}

#[test]
fn t166_topology_operation_is_exact_generation_and_target_bound() {
    let payload = ProtocolPayload::ApplyTopologyOperation {
        request: ApplyTopologyOperationV2 {
            expected_topology_generation: topology_generation(8),
            operation: TopologyOperationV2::SplitPane {
                multiplexer_workspace_id: workspace(3),
                tab_id: tab(4),
                target_pane_id: pane(5),
                new_pane_id: pane(6),
                axis: ProtocolSplitAxis::Vertical,
                placement: ProtocolPanePlacement::After,
                ratio_basis_points: 5_000,
            },
        },
    };
    let message = v2_message(payload);
    let decoded = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
    assert_eq!(
        message.kind().authority_class(),
        MessageAuthorityClass::MultiplexerWriteOnly
    );
    assert_eq!(message.runtime_namespace_id, None);
}

#[test]
fn t166_response_binding_rejects_topology_target_and_generation_substitution() {
    let request = ProtocolMessage::new(
        connection("t166-topology-bind"),
        sequence(18),
        None,
        generation(18),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: topology_generation(8),
                operation: TopologyOperationV2::RenameWorkspace {
                    multiplexer_workspace_id: workspace(18),
                    alias: "renamed".to_owned(),
                },
            },
        },
    )
    .unwrap();

    let response = |workspace_id, accepted_generation| {
        ProtocolMessage::new(
            connection("t166-topology-bind"),
            sequence(19),
            None,
            generation(18),
            Some(sequence(18)),
            ProtocolPayload::MultiplexerSnapshot {
                snapshot: MultiplexerSnapshotV2::MutationResult {
                    result: TopologyMutationResultV2 {
                        outcome: TopologyMutationOutcomeV2::Accepted,
                        accepted_topology_generation: Some(accepted_generation),
                        multiplexer_workspace_id: Some(workspace_id),
                        tab_id: None,
                        pane_id: None,
                        secondary_tab_id: None,
                        secondary_pane_id: None,
                    },
                    snapshot: None,
                },
            },
        )
        .unwrap()
    };

    assert_eq!(
        validate_response_binding(&request, &response(workspace(19), topology_generation(9)))
            .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        validate_response_binding(&request, &response(workspace(18), topology_generation(10)))
            .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    validate_response_binding(&request, &response(workspace(18), topology_generation(9))).unwrap();
}

#[test]
fn t166_response_binding_preserves_complete_multi_target_ids() {
    let split_request = ProtocolMessage::new(
        connection("t166-split-bind"),
        sequence(20),
        None,
        generation(18),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: topology_generation(8),
                operation: TopologyOperationV2::SplitPane {
                    multiplexer_workspace_id: workspace(18),
                    tab_id: tab(18),
                    target_pane_id: pane(18),
                    new_pane_id: pane(19),
                    axis: ProtocolSplitAxis::Vertical,
                    placement: ProtocolPanePlacement::After,
                    ratio_basis_points: 5_000,
                },
            },
        },
    )
    .unwrap();
    let split_response = |secondary_pane_id| {
        ProtocolMessage::new(
            connection("t166-split-bind"),
            sequence(21),
            None,
            generation(18),
            Some(sequence(20)),
            ProtocolPayload::MultiplexerSnapshot {
                snapshot: MultiplexerSnapshotV2::MutationResult {
                    result: TopologyMutationResultV2 {
                        outcome: TopologyMutationOutcomeV2::Accepted,
                        accepted_topology_generation: Some(topology_generation(9)),
                        multiplexer_workspace_id: Some(workspace(18)),
                        tab_id: Some(tab(18)),
                        pane_id: Some(pane(18)),
                        secondary_tab_id: None,
                        secondary_pane_id: Some(secondary_pane_id),
                    },
                    snapshot: None,
                },
            },
        )
        .unwrap()
    };
    validate_response_binding(&split_request, &split_response(pane(19))).unwrap();
    assert_eq!(
        validate_response_binding(&split_request, &split_response(pane(20))).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let move_request = ProtocolMessage::new(
        connection("t166-move-bind"),
        sequence(22),
        None,
        generation(18),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: topology_generation(9),
                operation: TopologyOperationV2::MovePane {
                    multiplexer_workspace_id: workspace(18),
                    source_tab_id: tab(18),
                    pane_id: pane(18),
                    destination_tab_id: tab(19),
                    destination_pane_id: pane(21),
                    axis: ProtocolSplitAxis::Horizontal,
                    placement: ProtocolPanePlacement::Before,
                    ratio_basis_points: 4_000,
                },
            },
        },
    )
    .unwrap();
    let move_response = |secondary_tab_id, secondary_pane_id| {
        ProtocolMessage::new(
            connection("t166-move-bind"),
            sequence(23),
            None,
            generation(18),
            Some(sequence(22)),
            ProtocolPayload::MultiplexerSnapshot {
                snapshot: MultiplexerSnapshotV2::MutationResult {
                    result: TopologyMutationResultV2 {
                        outcome: TopologyMutationOutcomeV2::Accepted,
                        accepted_topology_generation: Some(topology_generation(10)),
                        multiplexer_workspace_id: Some(workspace(18)),
                        tab_id: Some(tab(18)),
                        pane_id: Some(pane(18)),
                        secondary_tab_id: Some(secondary_tab_id),
                        secondary_pane_id: Some(secondary_pane_id),
                    },
                    snapshot: None,
                },
            },
        )
        .unwrap()
    };
    validate_response_binding(&move_request, &move_response(tab(19), pane(21))).unwrap();
    assert_eq!(
        validate_response_binding(&move_request, &move_response(tab(20), pane(21))).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        validate_response_binding(&move_request, &move_response(tab(19), pane(22))).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t166_agent_observation_schema_keeps_workspace_domains_unambiguous() {
    let payload = ProtocolPayload::AgentObservationSnapshot {
        snapshot: AgentObservationSnapshotV2 {
            filter_multiplexer_workspace_id: None,
            snapshot_revision: 1,
            page_offset: 0,
            observations: vec![AgentObservationV2 {
                observation_id: observation(7),
                family: AgentFamilyV2::Claude,
                source_class: AgentObservationSourceV2::OwnedProcessMetadata,
                confidence_class: AgentObservationConfidenceV2::Strong,
                freshness: AgentObservationFreshnessV2::Current,
                multiplexer_workspace_id: workspace(8),
                git_workspace_id: Some("workspace-git-8".to_owned()),
                tab_id: tab(9),
                pane_id: pane(10),
                runtime_namespace_id: Some(runtime(11)),
                provider_native_session_id: Some("provider-session".to_owned()),
                owner_generation_id: generation(12),
                observed_unix_ms: 42,
                structured_evidence_summary: "structured process metadata".to_owned(),
            }],
            next_cursor: None,
        },
    };
    let message = ProtocolMessage::new(
        connection("t166-client"),
        sequence(9),
        None,
        generation(12),
        Some(sequence(7)),
        payload,
    )
    .unwrap();
    let frame = encode_frame(&message).unwrap();
    let json = std::str::from_utf8(&frame[4..]).unwrap();
    assert!(json.contains("multiplexer_workspace_id"));
    assert!(json.contains("git_workspace_id"));
    assert!(!json.contains(r#""workspace_id":"#));
    assert_eq!(decode_frame(&frame).unwrap(), message);
}

#[test]
fn t166_snapshot_rejects_cross_tab_focus_and_zoom_bindings() {
    let base = ProtocolWorkspaceSnapshotV2 {
        multiplexer_workspace_id: workspace(20),
        alias: "workspace".to_owned(),
        topology_generation: topology_generation(1),
        focused_tab_id: tab(20),
        tabs: vec![
            ProtocolTabSnapshotV2 {
                tab_id: tab(20),
                alias: "first".to_owned(),
                root: ProtocolLayoutNodeV2::Pane { pane_id: pane(20) },
                focused_pane_id: pane(20),
                zoomed_pane_id: None,
            },
            ProtocolTabSnapshotV2 {
                tab_id: tab(21),
                alias: "second".to_owned(),
                root: ProtocolLayoutNodeV2::Pane { pane_id: pane(21) },
                focused_pane_id: pane(20),
                zoomed_pane_id: None,
            },
        ],
    };
    let invalid_focus = ProtocolMessage::new(
        connection("t166-cross-tab-focus"),
        sequence(20),
        None,
        generation(20),
        Some(sequence(19)),
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::Workspace {
                snapshot: base.clone(),
            },
        },
    )
    .unwrap_err();
    assert_eq!(invalid_focus, LocalControlErrorKind::MalformedFrame);

    let mut invalid_zoom = base;
    invalid_zoom.tabs[1].focused_pane_id = pane(21);
    invalid_zoom.tabs[1].zoomed_pane_id = Some(pane(20));
    let invalid_zoom = ProtocolMessage::new(
        connection("t166-cross-tab-zoom"),
        sequence(21),
        None,
        generation(20),
        Some(sequence(19)),
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::Workspace {
                snapshot: invalid_zoom,
            },
        },
    )
    .unwrap_err();
    assert_eq!(invalid_zoom, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_workspace_list_rejects_mixed_generation_and_multiple_focus() {
    let summary = |id: u8, generation_value: u64, is_focused: bool| ProtocolWorkspaceSummaryV2 {
        multiplexer_workspace_id: workspace(id),
        alias: format!("workspace-{id}"),
        topology_generation: topology_generation(generation_value),
        is_focused,
    };

    for workspaces in [
        vec![summary(24, 2, true), summary(25, 3, false)],
        vec![summary(24, 2, true), summary(25, 2, true)],
    ] {
        let error = ProtocolMessage::new(
            connection("t166-workspace-list"),
            sequence(24),
            None,
            generation(24),
            Some(sequence(23)),
            ProtocolPayload::MultiplexerSnapshot {
                snapshot: MultiplexerSnapshotV2::WorkspaceList {
                    topology_generation: topology_generation(2),
                    workspaces,
                },
            },
        )
        .unwrap_err();
        assert_eq!(error, LocalControlErrorKind::MalformedFrame);
    }
}

#[test]
fn t166_max_workspace_list_frame_is_bounded() {
    let workspaces = (0..MAX_V2_WORKSPACES)
        .map(|index| ProtocolWorkspaceSummaryV2 {
            multiplexer_workspace_id: workspace((index + 1) as u8),
            alias: "w".repeat(MAX_V2_ALIAS_BYTES),
            topology_generation: topology_generation(2),
            is_focused: index == 0,
        })
        .collect();
    let message = ProtocolMessage::new(
        connection("t166-max-workspace-list"),
        sequence(25),
        None,
        generation(25),
        Some(sequence(24)),
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::WorkspaceList {
                topology_generation: topology_generation(2),
                workspaces,
            },
        },
    )
    .unwrap();
    let frame = encode_frame(&message).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), message);
}

#[test]
fn t166_typed_worktree_page_is_bounded_and_single_frame_safe() {
    let result = WorktreeOperationResultV2 {
        outcome: WorktreeOperationOutcomeV2::Accepted,
        multiplexer_workspace_id: workspace(13),
        repository_identity: None,
        git_workspace_id: None,
        operation_target: None,
        snapshot_revision: 1,
        page_offset: 0,
        worktrees: (0..MAX_V2_WORKTREES_PER_PAGE).map(worktree).collect(),
        next_cursor: Some(WorktreeCursorV2 {
            owner_generation_id: generation(13),
            snapshot_revision: 1,
            offset: MAX_V2_WORKTREES_PER_PAGE as u16,
        }),
    };
    let request = ProtocolMessage::new(
        connection("t166-client"),
        sequence(7),
        None,
        generation(13),
        None,
        ProtocolPayload::ListWorktrees {
            request: ListWorktreesV2 {
                multiplexer_workspace_id: workspace(13),
                cursor: None,
            },
        },
    )
    .unwrap();
    let response = ProtocolMessage::new(
        connection("t166-client"),
        sequence(14),
        None,
        generation(13),
        Some(sequence(7)),
        ProtocolPayload::WorktreeOperationResult { result },
    )
    .unwrap();
    validate_response_binding(&request, &response).unwrap();
    let frame = encode_frame(&response).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), response);

    let too_many = WorktreeOperationResultV2 {
        outcome: WorktreeOperationOutcomeV2::Accepted,
        multiplexer_workspace_id: workspace(13),
        repository_identity: None,
        git_workspace_id: None,
        operation_target: None,
        snapshot_revision: 1,
        page_offset: 0,
        worktrees: (0..=MAX_V2_WORKTREES_PER_PAGE).map(worktree).collect(),
        next_cursor: None,
    };
    assert_eq!(
        ProtocolMessage::new(
            connection("t166-client"),
            sequence(15),
            None,
            generation(13),
            Some(sequence(7)),
            ProtocolPayload::WorktreeOperationResult { result: too_many },
        )
        .unwrap_err(),
        LocalControlErrorKind::OversizedFrame
    );
}

#[test]
fn t166_agent_observation_rejects_negative_observation_time() {
    let error = ProtocolMessage::new(
        connection("t166-negative-observation-time"),
        sequence(26),
        None,
        generation(26),
        Some(sequence(25)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 1,
                page_offset: 0,
                observations: vec![AgentObservationV2 {
                    observation_id: observation(26),
                    family: AgentFamilyV2::Claude,
                    source_class: AgentObservationSourceV2::OwnedProcessMetadata,
                    confidence_class: AgentObservationConfidenceV2::Strong,
                    freshness: AgentObservationFreshnessV2::Current,
                    multiplexer_workspace_id: workspace(26),
                    git_workspace_id: None,
                    tab_id: tab(26),
                    pane_id: pane(26),
                    runtime_namespace_id: None,
                    provider_native_session_id: None,
                    owner_generation_id: generation(26),
                    observed_unix_ms: -1,
                    structured_evidence_summary: "evidence".to_owned(),
                }],
                next_cursor: None,
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_paged_collections_reject_duplicate_item_identities() {
    let owner_generation_id = generation(14);
    let duplicate_observation = AgentObservationV2 {
        observation_id: observation(14),
        family: AgentFamilyV2::Claude,
        source_class: AgentObservationSourceV2::OwnedProcessMetadata,
        confidence_class: AgentObservationConfidenceV2::Strong,
        freshness: AgentObservationFreshnessV2::Current,
        multiplexer_workspace_id: workspace(14),
        git_workspace_id: None,
        tab_id: tab(14),
        pane_id: pane(14),
        runtime_namespace_id: None,
        provider_native_session_id: None,
        owner_generation_id,
        observed_unix_ms: 1,
        structured_evidence_summary: "evidence".to_owned(),
    };
    let duplicate_agents = ProtocolMessage::new(
        connection("t166-duplicate-agent"),
        sequence(22),
        None,
        owner_generation_id,
        Some(sequence(21)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 1,
                page_offset: 0,
                observations: vec![duplicate_observation.clone(), duplicate_observation],
                next_cursor: None,
            },
        },
    )
    .unwrap_err();
    assert_eq!(duplicate_agents, LocalControlErrorKind::MalformedFrame);

    let duplicate_worktree = WorktreeObservationV2 {
        git_workspace_id: "duplicate-worktree".to_owned(),
        repository_identity: "repo".to_owned(),
        canonical_path: "/tmp/duplicate-worktree".to_owned(),
        branch: Some("main".to_owned()),
        head_oid: "a".repeat(40),
        dirty: false,
        membership: WorktreeMembershipV2::ExplicitMember,
    };
    let duplicate_worktrees = ProtocolMessage::new(
        connection("t166-duplicate-worktree"),
        sequence(23),
        None,
        owner_generation_id,
        Some(sequence(21)),
        ProtocolPayload::WorktreeOperationResult {
            result: WorktreeOperationResultV2 {
                outcome: WorktreeOperationOutcomeV2::Accepted,
                multiplexer_workspace_id: workspace(14),
                repository_identity: None,
                git_workspace_id: None,
                operation_target: None,
                snapshot_revision: 1,
                page_offset: 0,
                worktrees: vec![duplicate_worktree.clone(), duplicate_worktree],
                next_cursor: None,
            },
        },
    )
    .unwrap_err();
    assert_eq!(duplicate_worktrees, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_cursor_revision_mismatch_fails_closed() {
    let error = ProtocolMessage::new(
        connection("t166-client"),
        sequence(16),
        None,
        generation(15),
        Some(sequence(7)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 10,
                page_offset: 0,
                observations: Vec::new(),
                next_cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id: generation(15),
                    snapshot_revision: 9,
                    offset: 1,
                }),
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, LocalControlErrorKind::MalformedFrame);
}

fn max_agent_observation(index: u16, owner_generation_id: OwnerGenerationId) -> AgentObservationV2 {
    AgentObservationV2 {
        observation_id: observation_index(index),
        family: AgentFamilyV2::Claude,
        source_class: AgentObservationSourceV2::ProviderStructuredMetadata,
        confidence_class: AgentObservationConfidenceV2::Strong,
        freshness: AgentObservationFreshnessV2::Current,
        multiplexer_workspace_id: workspace(22),
        git_workspace_id: Some("g".repeat(MAX_V2_GIT_WORKSPACE_ID_BYTES)),
        tab_id: tab(23),
        pane_id: pane_index(index.saturating_add(1)),
        runtime_namespace_id: Some(runtime(24)),
        provider_native_session_id: Some("s".repeat(MAX_V2_PROVIDER_SESSION_ID_BYTES)),
        owner_generation_id,
        observed_unix_ms: i64::MAX,
        structured_evidence_summary: "e".repeat(MAX_V2_EVIDENCE_SUMMARY_BYTES),
    }
}

fn domain_pane_chain(start: u16, count: u16) -> LayoutNode {
    let mut node = LayoutNode::Pane(pane_index(start));
    for offset in 1..count {
        node = LayoutNode::Split {
            axis: DomainSplitAxis::Horizontal,
            ratio_bps: SplitRatioBps::new(5_000).unwrap(),
            first: Box::new(node),
            second: Box::new(LayoutNode::Pane(pane_index(start + offset))),
        };
    }
    node
}

fn topology_with_panes(tab_counts: &[u16]) -> MultiplexerTopology {
    let workspace_id = workspace(40);
    let mut next_pane = 1_u16;
    let tabs = tab_counts
        .iter()
        .enumerate()
        .map(|(index, pane_count)| {
            let first_pane = next_pane;
            let tab_state = TabState {
                id: tab_index(index as u16 + 1),
                alias: String::new(),
                root: domain_pane_chain(first_pane, *pane_count),
                focused_pane_id: pane_index(first_pane),
                zoomed_pane_id: None,
            };
            next_pane += *pane_count;
            tab_state
        })
        .collect::<Vec<_>>();
    MultiplexerTopology::restore_presentation(
        topology_generation(1),
        vec![WorkspaceState {
            id: workspace_id,
            alias: String::new(),
            focused_tab_id: tab_index(1),
            tabs,
        }],
        Some(workspace_id),
    )
    .unwrap()
}

fn pane_chain(start: u16, count: u16) -> ProtocolLayoutNodeV2 {
    let mut node = ProtocolLayoutNodeV2::Pane {
        pane_id: pane_index(start),
    };
    for offset in 1..count {
        node = ProtocolLayoutNodeV2::Split {
            axis: ProtocolSplitAxis::Horizontal,
            ratio_basis_points: 5_000,
            first: Box::new(node),
            second: Box::new(ProtocolLayoutNodeV2::Pane {
                pane_id: pane_index(start + offset),
            }),
        };
    }
    node
}

#[test]
fn t166_owner_preflight_accepts_256_panes_and_rejects_257() {
    let accepted = topology_with_panes(&[
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
    ]);
    validate_candidate_topology_v2(&accepted, generation(40)).unwrap();

    let rejected = topology_with_panes(&[
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
        MAX_V2_PANES_PER_TAB as u16,
        1,
    ]);
    assert_eq!(
        validate_candidate_topology_v2(&rejected, generation(40)),
        Err(crate::multiplexer::domain::MultiplexerErrorKind::SnapshotLimitExceeded)
    );
    assert_eq!(
        multiplexer_snapshot_for_request_v2(&rejected, generation(40), None),
        Err(crate::multiplexer::domain::MultiplexerErrorKind::SnapshotLimitExceeded)
    );
}

#[test]
fn t166_max_topology_snapshot_fits_single_frame_without_truncation() {
    let panes_per_tab = 8_u16;
    let tabs = (0..MAX_V2_TABS_PER_WORKSPACE as u16)
        .map(|index| {
            let first_pane = index * panes_per_tab + 1;
            ProtocolTabSnapshotV2 {
                tab_id: tab_index(index + 1),
                alias: "a".repeat(MAX_V2_ALIAS_BYTES),
                root: pane_chain(first_pane, panes_per_tab),
                focused_pane_id: pane_index(first_pane),
                zoomed_pane_id: Some(pane_index(first_pane + panes_per_tab - 1)),
            }
        })
        .collect::<Vec<_>>();
    let message = ProtocolMessage::new(
        connection("t166-max-topology"),
        sequence(30),
        None,
        generation(30),
        Some(sequence(29)),
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::Workspace {
                snapshot: ProtocolWorkspaceSnapshotV2 {
                    multiplexer_workspace_id: workspace(30),
                    alias: "w".repeat(MAX_V2_ALIAS_BYTES),
                    topology_generation: topology_generation(30),
                    focused_tab_id: tab_index(1),
                    tabs,
                },
            },
        },
    )
    .unwrap();
    let frame = encode_frame(&message).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), message);
}

#[test]
fn t166_max_agent_page_fits_and_next_cursor_is_exact() {
    let owner_generation_id = generation(31);
    let observations = (0..MAX_V2_AGENT_OBSERVATIONS_PER_PAGE as u16)
        .map(|index| max_agent_observation(index + 1, owner_generation_id))
        .collect::<Vec<_>>();
    let message = ProtocolMessage::new(
        connection("t166-max-agent"),
        sequence(31),
        None,
        owner_generation_id,
        Some(sequence(30)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 9,
                page_offset: 0,
                observations,
                next_cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id,
                    snapshot_revision: 9,
                    offset: MAX_V2_AGENT_OBSERVATIONS_PER_PAGE as u16,
                }),
            },
        },
    )
    .unwrap();
    let frame = encode_frame(&message).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), message);
}

#[test]
fn t166_max_attention_snapshot_is_single_frame_and_owner_bound() {
    let owner_generation_id = generation(32);
    let items = (0..MAX_V2_ATTENTION_ITEMS_PER_PAGE)
        .map(|index| AttentionItemV2 {
            source_domain: "d".repeat(MAX_V2_DETAIL_BYTES),
            source_event_id: format!(
                "{index:03}{}",
                "e".repeat(MAX_V2_DETAIL_BYTES.saturating_sub(3))
            ),
            kind: AttentionKindV2::ExplicitAgentAttention,
            multiplexer_workspace_id: workspace(32),
            tab_id: Some(tab(32)),
            pane_id: Some(pane(32)),
            runtime_namespace_id: Some(runtime(32)),
            owner_generation_id,
            detail: "x".repeat(MAX_V2_DETAIL_BYTES),
            stale: false,
        })
        .collect::<Vec<_>>();
    let message = ProtocolMessage::new(
        connection("t166-max-attention"),
        sequence(32),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::AttentionSnapshot {
            snapshot: AttentionSnapshotV2 {
                snapshot_revision: 11,
                items,
            },
        },
    )
    .unwrap();
    let frame = encode_frame(&message).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), message);
}

#[test]
fn t166_collection_cursor_owner_and_offset_binding_fail_closed() {
    let owner_generation_id = generation(33);
    let request = ProtocolMessage::new(
        connection("t166-page"),
        sequence(40),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::ListAgentObservations {
            request: ListAgentObservationsV2 {
                multiplexer_workspace_id: Some(workspace(33)),
                cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id,
                    snapshot_revision: 12,
                    offset: 64,
                }),
            },
        },
    )
    .unwrap();

    let wrong_offset = ProtocolMessage::new(
        connection("t166-page"),
        sequence(41),
        None,
        owner_generation_id,
        Some(sequence(40)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 12,
                page_offset: 63,
                observations: Vec::new(),
                next_cursor: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_offset).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let stale_owner = ProtocolMessage::new(
        connection("t166-page"),
        sequence(42),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::ListAgentObservations {
            request: ListAgentObservationsV2 {
                multiplexer_workspace_id: Some(workspace(33)),
                cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id: generation(34),
                    snapshot_revision: 12,
                    offset: 64,
                }),
            },
        },
    )
    .unwrap_err();
    assert_eq!(stale_owner, LocalControlErrorKind::StaleOwnerGeneration);
}

#[test]
fn t166_empty_page_cursor_must_be_absent_and_nonempty_cursor_must_advance() {
    let owner_generation_id = generation(34);
    let empty = ProtocolMessage::new(
        connection("t166-empty-terminal"),
        sequence(42),
        None,
        owner_generation_id,
        Some(sequence(40)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 12,
                page_offset: 64,
                observations: Vec::new(),
                next_cursor: None,
            },
        },
    )
    .unwrap();
    assert_eq!(decode_frame(&encode_frame(&empty).unwrap()).unwrap(), empty);

    let empty_with_cursor = ProtocolMessage::new(
        connection("t166-empty-cursor"),
        sequence(43),
        None,
        owner_generation_id,
        Some(sequence(40)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 12,
                page_offset: 64,
                observations: Vec::new(),
                next_cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id,
                    snapshot_revision: 12,
                    offset: 64,
                }),
            },
        },
    )
    .unwrap_err();
    assert_eq!(empty_with_cursor, LocalControlErrorKind::MalformedFrame);

    let nonadvancing = ProtocolMessage::new(
        connection("t166-nonadvancing"),
        sequence(44),
        None,
        owner_generation_id,
        Some(sequence(40)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: Some(workspace(33)),
                snapshot_revision: 12,
                page_offset: 64,
                observations: vec![max_agent_observation(1, owner_generation_id)],
                next_cursor: Some(AgentObservationCursorV2 {
                    owner_generation_id,
                    snapshot_revision: 12,
                    offset: 64,
                }),
            },
        },
    )
    .unwrap_err();
    assert_eq!(nonadvancing, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_inactive_future_domains_return_typed_unsupported_without_side_effect_path() {
    let request = ProtocolMessage::new(
        connection("t166-inactive"),
        sequence(50),
        None,
        generation(35),
        None,
        ProtocolPayload::ListWorktrees {
            request: ListWorktreesV2 {
                multiplexer_workspace_id: workspace(35),
                cursor: None,
            },
        },
    )
    .unwrap();
    let response = inactive_v2_domain_response(&request, sequence(51)).unwrap();
    assert_eq!(
        response.payload,
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::UnsupportedOperation,
        }
    );
    assert_eq!(response.correlation_sequence, Some(sequence(50)));
    assert_eq!(response.owner_generation_id, Some(generation(35)));
    validate_response_binding(&request, &response).unwrap();

    let future_topology = ProtocolMessage::new(
        connection("t166-inactive"),
        sequence(52),
        None,
        generation(35),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: topology_generation(2),
                operation: TopologyOperationV2::ClearPane {
                    multiplexer_workspace_id: workspace(35),
                    tab_id: tab(35),
                    pane_id: pane(35),
                    presentation_epoch: 1,
                },
            },
        },
    )
    .unwrap();
    let future_response = inactive_v2_domain_response(&future_topology, sequence(53)).unwrap();
    assert_eq!(
        future_response.payload,
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::UnsupportedOperation,
        }
    );
}

#[test]
fn t166_stale_topology_and_history_gap_are_explicit_wire_outcomes() {
    let stale = ProtocolMessage::new(
        connection("t166-stale"),
        sequence(52),
        None,
        generation(36),
        Some(sequence(51)),
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::MutationResult {
                result: TopologyMutationResultV2 {
                    outcome: TopologyMutationOutcomeV2::Rejected {
                        error: crate::multiplexer::domain::MultiplexerErrorKind::StaleTopologyGeneration,
                    },
                    accepted_topology_generation: None,
                    multiplexer_workspace_id: Some(workspace(36)),
                    tab_id: None,
                    pane_id: None,
                    secondary_tab_id: None,
                    secondary_pane_id: None,
                },
                snapshot: None,
            },
        },
    )
    .unwrap();
    assert_eq!(decode_frame(&encode_frame(&stale).unwrap()).unwrap(), stale);

    for payload in [
        ProtocolPayload::MultiplexerEvent {
            event: MultiplexerEventV2::HistoryGap {
                last_known_topology_generation: topology_generation(7),
            },
        },
        ProtocolPayload::AgentObservationEvent {
            event: AgentObservationEventV2::HistoryGap {
                last_known_snapshot_revision: 7,
            },
        },
        ProtocolPayload::AttentionEvent {
            event: AttentionEventV2::HistoryGap {
                last_known_snapshot_revision: 7,
            },
        },
    ] {
        let message = ProtocolMessage::new(
            connection("t166-gap"),
            sequence(53),
            None,
            generation(36),
            None,
            payload,
        )
        .unwrap();
        assert_eq!(
            decode_frame(&encode_frame(&message).unwrap()).unwrap(),
            message
        );
    }
}

#[test]
fn t166_terminal_text_shaped_like_protocol_remains_untrusted_input() {
    let suspicious =
        br#"{"message_kind":"APPLY_TOPOLOGY_OPERATION","accepted":true,"VERIFIED":true}"#.to_vec();
    let message = ProtocolMessage::new(
        connection("t166-client"),
        sequence(17),
        Some(runtime(18)),
        generation(19),
        None,
        ProtocolPayload::Input {
            data: suspicious.clone(),
        },
    )
    .unwrap();
    let decoded = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded.payload, ProtocolPayload::Input { data: suspicious });
}
#[test]
fn t166_subscription_ack_is_closed_exact_bound_and_inactive() {
    let owner_generation_id = generation(37);
    for (stream, boundary) in [
        (
            MultiplexerSubscriptionStreamV2::Topology,
            MultiplexerSubscriptionBoundaryV2::Topology {
                topology_generation: topology_generation(3),
            },
        ),
        (
            MultiplexerSubscriptionStreamV2::AgentObservations,
            MultiplexerSubscriptionBoundaryV2::AgentObservations {
                snapshot_revision: 4,
            },
        ),
        (
            MultiplexerSubscriptionStreamV2::Attention,
            MultiplexerSubscriptionBoundaryV2::Attention {
                snapshot_revision: 5,
            },
        ),
    ] {
        let request = ProtocolMessage::new(
            connection("t166-subscribe"),
            sequence(100),
            None,
            owner_generation_id,
            None,
            ProtocolPayload::SubscribeMultiplexerEvents {
                request: SubscribeMultiplexerEventsV2 {
                    stream,
                    multiplexer_workspace_id: Some(workspace(37)),
                },
            },
        )
        .unwrap();
        let response = ProtocolMessage::new(
            connection("t166-subscribe"),
            sequence(101),
            None,
            owner_generation_id,
            Some(sequence(100)),
            ProtocolPayload::MultiplexerEventSubscriptionAck {
                ack: MultiplexerEventSubscriptionAckV2 {
                    stream,
                    multiplexer_workspace_id: Some(workspace(37)),
                    boundary,
                },
            },
        )
        .unwrap();
        validate_response_binding(&request, &response).unwrap();
        let unsupported = inactive_v2_domain_response(&request, sequence(102)).unwrap();
        validate_response_binding(&request, &unsupported).unwrap();
    }
}

#[test]
fn t166_abbreviated_worktree_oid_fails_closed() {
    let error = ProtocolMessage::new(
        connection("t166-short-oid"),
        sequence(61),
        None,
        generation(38),
        None,
        ProtocolPayload::ApplyWorktreeOperation {
            request: ApplyWorktreeOperationV2 {
                multiplexer_workspace_id: workspace(38),
                repository_identity: "repo".to_owned(),
                trust_record_revision: 1,
                operation: WorktreeOperationV2::Create {
                    destination_path: "/tmp/winds-worktree".to_owned(),
                    base_commit_oid: "abcdef1".to_owned(),
                    new_branch_name: None,
                },
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_worktree_operation_requires_trust_revision_and_exact_result_target() {
    let zero_revision = ProtocolMessage::new(
        connection("t166-zero-trust"),
        sequence(62),
        None,
        generation(39),
        None,
        ProtocolPayload::ApplyWorktreeOperation {
            request: ApplyWorktreeOperationV2 {
                multiplexer_workspace_id: workspace(39),
                repository_identity: "repo".to_owned(),
                trust_record_revision: 0,
                operation: WorktreeOperationV2::Open {
                    git_workspace_id: "git-worktree-39".to_owned(),
                    canonical_path: "/tmp/winds-worktree".to_owned(),
                },
            },
        },
    )
    .unwrap_err();
    assert_eq!(zero_revision, LocalControlErrorKind::MalformedFrame);

    let request = ProtocolMessage::new(
        connection("t166-worktree-bind"),
        sequence(63),
        None,
        generation(39),
        None,
        ProtocolPayload::ApplyWorktreeOperation {
            request: ApplyWorktreeOperationV2 {
                multiplexer_workspace_id: workspace(39),
                repository_identity: "repo".to_owned(),
                trust_record_revision: 7,
                operation: WorktreeOperationV2::Open {
                    git_workspace_id: "git-worktree-39".to_owned(),
                    canonical_path: "/tmp/winds-worktree".to_owned(),
                },
            },
        },
    )
    .unwrap();
    let wrong_target = ProtocolMessage::new(
        connection("t166-worktree-bind"),
        sequence(64),
        None,
        generation(39),
        Some(sequence(63)),
        ProtocolPayload::WorktreeOperationResult {
            result: WorktreeOperationResultV2 {
                outcome: WorktreeOperationOutcomeV2::Accepted,
                multiplexer_workspace_id: workspace(40),
                repository_identity: Some("repo".to_owned()),
                git_workspace_id: Some("git-worktree-39".to_owned()),
                operation_target: Some(WorktreeOperationTargetV2 {
                    operation: WorktreeOperationV2::Open {
                        git_workspace_id: "git-worktree-39".to_owned(),
                        canonical_path: "/tmp/winds-worktree".to_owned(),
                    },
                    trust_record_revision: 7,
                }),
                snapshot_revision: 1,
                page_offset: 0,
                worktrees: Vec::new(),
                next_cursor: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_target).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t166_empty_optional_worktree_branch_name_fails_closed() {
    let error = ProtocolMessage::new(
        connection("t166-empty-branch"),
        sequence(60),
        None,
        generation(37),
        None,
        ProtocolPayload::ApplyWorktreeOperation {
            request: ApplyWorktreeOperationV2 {
                multiplexer_workspace_id: workspace(37),
                repository_identity: "repo".to_owned(),
                trust_record_revision: 1,
                operation: WorktreeOperationV2::Create {
                    destination_path: "/tmp/winds-worktree".to_owned(),
                    base_commit_oid: "a".repeat(40),
                    new_branch_name: Some(String::new()),
                },
            },
        },
    )
    .unwrap_err();
    assert_eq!(error, LocalControlErrorKind::MalformedFrame);
}

#[test]
fn t166_future_domain_mutation_is_typed_without_runtime_controller_authority() {
    let worktree = v2_message(ProtocolPayload::ApplyWorktreeOperation {
        request: ApplyWorktreeOperationV2 {
            multiplexer_workspace_id: workspace(4),
            repository_identity: "repo".to_owned(),
            trust_record_revision: 1,
            operation: WorktreeOperationV2::Open {
                git_workspace_id: "git-worktree-4".to_owned(),
                canonical_path: "/tmp/winds-worktree".to_owned(),
            },
        },
    });
    assert_eq!(
        worktree.kind().authority_class(),
        MessageAuthorityClass::FutureDomainMutation
    );
    assert_eq!(worktree.runtime_namespace_id, None);

    let list = v2_message(ProtocolPayload::ListAgentObservations {
        request: ListAgentObservationsV2 {
            multiplexer_workspace_id: Some(workspace(4)),
            cursor: None,
        },
    });
    assert_eq!(list.runtime_namespace_id, None);

    let worktrees = v2_message(ProtocolPayload::ListWorktrees {
        request: ListWorktreesV2 {
            multiplexer_workspace_id: workspace(4),
            cursor: None,
        },
    });
    assert_eq!(worktrees.runtime_namespace_id, None);
}

#[test]
fn t166_schema_freeze_repaired_bindings_fail_closed() {
    let owner_generation_id = generation(41);
    let agent_request = ProtocolMessage::new(
        connection("t166-filter"),
        sequence(120),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::ListAgentObservations {
            request: ListAgentObservationsV2 {
                multiplexer_workspace_id: Some(workspace(41)),
                cursor: None,
            },
        },
    )
    .unwrap();
    let filtered_empty = ProtocolMessage::new(
        connection("t166-filter"),
        sequence(121),
        None,
        owner_generation_id,
        Some(sequence(120)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: Some(workspace(41)),
                snapshot_revision: 1,
                page_offset: 0,
                observations: Vec::new(),
                next_cursor: None,
            },
        },
    )
    .unwrap();
    validate_response_binding(&agent_request, &filtered_empty).unwrap();
    let undeclared_filter = ProtocolMessage::new(
        connection("t166-filter"),
        sequence(122),
        None,
        owner_generation_id,
        Some(sequence(120)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 1,
                page_offset: 0,
                observations: Vec::new(),
                next_cursor: None,
            },
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&agent_request, &undeclared_filter).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let attention_removed = ProtocolMessage::new(
        connection("t166-attention-remove"),
        sequence(123),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::AttentionEvent {
            event: AttentionEventV2::Removed {
                snapshot_revision: 2,
                source_domain: "worktree".to_owned(),
                source_event_id: "event-41".to_owned(),
                kind: AttentionKindV2::WorktreeTrustRequired,
                multiplexer_workspace_id: workspace(41),
                tab_id: Some(tab(41)),
                pane_id: Some(pane(41)),
                runtime_namespace_id: Some(runtime(41)),
                owner_generation_id,
            },
        },
    )
    .unwrap();
    assert_eq!(
        decode_frame(&encode_frame(&attention_removed).unwrap()).unwrap(),
        attention_removed
    );

    let operation = WorktreeOperationV2::Open {
        git_workspace_id: "git-41".to_owned(),
        canonical_path: "/tmp/git-41".to_owned(),
    };
    let worktree_request = ProtocolMessage::new(
        connection("t166-worktree-target"),
        sequence(124),
        None,
        owner_generation_id,
        None,
        ProtocolPayload::ApplyWorktreeOperation {
            request: ApplyWorktreeOperationV2 {
                multiplexer_workspace_id: workspace(41),
                repository_identity: "repo-41".to_owned(),
                trust_record_revision: 7,
                operation: operation.clone(),
            },
        },
    )
    .unwrap();
    let exact_worktree_result = |operation: WorktreeOperationV2, trust| {
        ProtocolMessage::new(
            connection("t166-worktree-target"),
            sequence(125),
            None,
            owner_generation_id,
            Some(sequence(124)),
            ProtocolPayload::WorktreeOperationResult {
                result: WorktreeOperationResultV2 {
                    outcome: WorktreeOperationOutcomeV2::Accepted,
                    multiplexer_workspace_id: workspace(41),
                    repository_identity: Some("repo-41".to_owned()),
                    git_workspace_id: Some("git-41".to_owned()),
                    operation_target: Some(WorktreeOperationTargetV2 {
                        operation,
                        trust_record_revision: trust,
                    }),
                    snapshot_revision: 1,
                    page_offset: 0,
                    worktrees: Vec::new(),
                    next_cursor: None,
                },
            },
        )
        .unwrap()
    };
    validate_response_binding(
        &worktree_request,
        &exact_worktree_result(operation.clone(), 7),
    )
    .unwrap();

    assert_eq!(
        validate_response_binding(
            &worktree_request,
            &exact_worktree_result(
                WorktreeOperationV2::Open {
                    git_workspace_id: "git-41".to_owned(),
                    canonical_path: "/tmp/substituted".to_owned(),
                },
                7,
            ),
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        validate_response_binding(&worktree_request, &exact_worktree_result(operation, 8),)
            .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let subscription = SubscribeMultiplexerEventsV2 {
        stream: MultiplexerSubscriptionStreamV2::Attention,
        multiplexer_workspace_id: Some(workspace(41)),
    };
    validate_multiplexer_subscription_event_binding(
        &connection("t166-attention-remove"),
        owner_generation_id,
        &subscription,
        &attention_removed,
    )
    .unwrap();
    assert_eq!(
        validate_multiplexer_subscription_event_binding(
            &connection("t166-attention-remove"),
            owner_generation_id,
            &SubscribeMultiplexerEventsV2 {
                stream: MultiplexerSubscriptionStreamV2::Attention,
                multiplexer_workspace_id: Some(workspace(42)),
            },
            &attention_removed,
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

struct Spec011V2RoundTripWire {
    owner_generation_id: OwnerGenerationId,
    connection_id: ClientConnectionId,
    sent: Arc<Mutex<Vec<ProtocolMessage>>>,
    receive: VecDeque<Result<ProtocolMessage, WireError>>,
}

impl Spec011V2RoundTripWire {
    fn new(owner_generation_id: OwnerGenerationId) -> (Self, Arc<Mutex<Vec<ProtocolMessage>>>) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                owner_generation_id,
                connection_id: connection("t166-spec011-v2"),
                sent: Arc::clone(&sent),
                receive: VecDeque::new(),
            },
            sent,
        )
    }

    fn response_for(&self, request: &ProtocolMessage) -> Result<ProtocolMessage, WireError> {
        let response_sequence = sequence(request.sequence.get().saturating_add(1_000));
        let payload = match &request.payload {
            ProtocolPayload::Hello { .. } => ProtocolPayload::HelloAck,
            ProtocolPayload::AttachObserver
            | ProtocolPayload::Detach
            | ProtocolPayload::ReleaseControl => ProtocolPayload::ControlState {
                authority: ClientAuthority::Observer,
                controller_client_id: None,
            },
            ProtocolPayload::RequestControl
            | ProtocolPayload::Input { .. }
            | ProtocolPayload::Resize { .. }
            | ProtocolPayload::Interrupt
            | ProtocolPayload::Stop => ProtocolPayload::ControlState {
                authority: ClientAuthority::Controller,
                controller_client_id: Some(self.connection_id.clone()),
            },
            _ => {
                return Err(WireError::Protocol(
                    LocalControlErrorKind::UnsupportedOperation,
                ));
            }
        };
        ProtocolMessage::new(
            self.connection_id.clone(),
            response_sequence,
            request.runtime_namespace_id,
            self.owner_generation_id,
            Some(request.sequence),
            payload,
        )
        .map_err(WireError::Protocol)
    }
}

impl LocalControlWire for Spec011V2RoundTripWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        let frame = encode_frame(message).map_err(WireError::Protocol)?;
        let json = std::str::from_utf8(&frame[4..])
            .map_err(|_| WireError::Protocol(LocalControlErrorKind::MalformedFrame))?;
        if !json.contains(r#""protocol_version":2"#) {
            return Err(WireError::Protocol(LocalControlErrorKind::ProtocolMismatch));
        }
        let decoded = decode_frame(&frame).map_err(WireError::Protocol)?;
        self.sent.lock().unwrap().push(decoded.clone());
        let response = self.response_for(&decoded)?;
        let response = decode_frame(&encode_frame(&response).map_err(WireError::Protocol)?)
            .map_err(WireError::Protocol)?;
        self.receive.push_back(Ok(response));
        Ok(())
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        self.receive
            .pop_front()
            .unwrap_or_else(|| Err(WireError::Transport("script exhausted".to_owned())))
    }
}

#[test]
fn t166_all_existing_spec011_client_operations_round_trip_under_v2() {
    let owner_generation_id = generation(48);
    let runtime_namespace_id = runtime(48);
    let target = ResolvedRuntimeTarget::exact(runtime_namespace_id);
    let (wire, sent) = Spec011V2RoundTripWire::new(owner_generation_id);
    let mut client = RustLocalControlClient::connect_with_wire_for_test(
        Box::new(wire),
        Some(owner_generation_id),
    )
    .unwrap();

    assert!(matches!(
        client.attach_observer(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Observer,
            ..
        }
    ));
    assert!(matches!(
        client.request_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.send_input(target, b"v2".to_vec()).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.resize(target, 120, 40).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.interrupt(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.release_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Observer,
            ..
        }
    ));
    assert!(matches!(
        client.request_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.stop(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
    assert!(matches!(
        client.detach_observer(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Observer,
            ..
        }
    ));

    let sent = sent.lock().unwrap();
    let kinds = sent.iter().map(ProtocolMessage::kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            MessageKind::Hello,
            MessageKind::AttachObserver,
            MessageKind::RequestControl,
            MessageKind::Input,
            MessageKind::Resize,
            MessageKind::Interrupt,
            MessageKind::ReleaseControl,
            MessageKind::RequestControl,
            MessageKind::Stop,
            MessageKind::Detach,
        ]
    );
    assert!(
        sent[1..]
            .iter()
            .all(|message| message.runtime_namespace_id == Some(runtime_namespace_id))
    );
}

struct LegacyMismatchWire {
    sent: Vec<ProtocolMessage>,
    receive: VecDeque<Result<ProtocolMessage, WireError>>,
}

impl LocalControlWire for LegacyMismatchWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        self.sent.push(message.clone());
        Ok(())
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        self.receive
            .pop_front()
            .unwrap_or_else(|| Err(WireError::Transport("script exhausted".to_owned())))
    }
}

#[test]
fn t166_live_v1_mismatch_maps_to_blocked_legacy_owner_without_handoff() {
    let wire = LegacyMismatchWire {
        sent: Vec::new(),
        receive: VecDeque::from([Err(WireError::LegacyProtocol)]),
    };
    let error = RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), None)
        .err()
        .unwrap();
    assert_eq!(error, LocalControlClientError::BlockedLegacyOwner);
}

#[test]
fn t166_request_multiplexer_write_carries_client_surface_capability() {
    let request = v2_message(ProtocolPayload::RequestMultiplexerWrite {
        request: RequestMultiplexerWriteV2 {
            client_surface_capability: ClientSurfaceCapability::TrustedDesktopTerminalSurface,
        },
    });
    let frame = encode_frame(&request).unwrap();
    let json = std::str::from_utf8(&frame[4..]).unwrap();
    assert!(json.contains("TRUSTED_DESKTOP_TERMINAL_SURFACE"));
    assert_eq!(decode_frame(&frame).unwrap(), request);
}

#[test]
fn t166_topology_helpers_execute_only_currently_authorized_operations() {
    let mut topology = MultiplexerTopology::empty();
    let create = TopologyOperationV2::CreateWorkspace {
        multiplexer_workspace_id: workspace(70),
        alias: "dispatch".to_owned(),
        first_tab_id: tab(71),
        first_tab_alias: "main".to_owned(),
        first_pane_id: pane(72),
    };
    let accepted =
        apply_topology_operation_v2(&mut topology, topology_generation(1), &create).unwrap();
    assert_eq!(accepted, topology_generation(2));
    assert_eq!(topology.workspace(workspace(70)).unwrap().alias, "dispatch");

    let deferred = TopologyOperationV2::ClearPane {
        multiplexer_workspace_id: workspace(70),
        tab_id: tab(71),
        pane_id: pane(72),
        presentation_epoch: 1,
    };
    assert_eq!(
        apply_topology_operation_v2(&mut topology, accepted, &deferred).unwrap_err(),
        MultiplexerErrorKind::UnsupportedOperation
    );
}

#[test]
fn t166_owner_dispatch_preflights_capability_and_applies_exact_topology() {
    use crate::persistent_runtime::owner::PersistentOwner;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_T166_DISPATCH_ROOT: AtomicU64 = AtomicU64::new(1);

    fn test_root(label: &str) -> PathBuf {
        let id = NEXT_T166_DISPATCH_ROOT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("winds-t166-{label}-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    fn short_runtime_root() -> PathBuf {
        let id = NEXT_T166_DISPATCH_ROOT.fetch_add(1, Ordering::Relaxed);
        let base = if cfg!(any(target_os = "linux", target_os = "macos")) {
            PathBuf::from("/tmp")
        } else {
            std::env::temp_dir()
        };
        let path = base.join(format!("w166r{id}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn start_owner(home: &Path, runtime_root: &Path) -> PersistentOwner {
        let runtime_directory = runtime_root.join("r");
        crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
            .unwrap();
        PersistentOwner::start_for_test(home, &runtime_directory, 100).unwrap()
    }

    #[cfg(windows)]
    fn start_owner(home: &Path, _runtime_root: &Path) -> PersistentOwner {
        PersistentOwner::start(home, 100).unwrap()
    }

    let home = test_root("dispatch-home");
    let runtime_root = short_runtime_root();
    let mut owner = start_owner(&home, &runtime_root);
    let client = connection("t166-owner-dispatch");

    let observer_request = ProtocolMessage::new(
        client.clone(),
        sequence(90),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::RequestMultiplexerWrite {
            request: RequestMultiplexerWriteV2 {
                client_surface_capability: ClientSurfaceCapability::NonInteractiveObserver,
            },
        },
    )
    .unwrap();
    let observer_response = owner
        .dispatch_multiplexer_protocol_v2(client.clone(), &observer_request, sequence(91), 101)
        .unwrap();
    assert!(matches!(
        observer_response.payload,
        ProtocolPayload::MultiplexerWriteState {
            state: MultiplexerWriteStateV2 {
                authority: MultiplexerAuthority::Observer,
                error: Some(MultiplexerErrorKind::CapabilityUnavailable),
            }
        }
    ));
    assert_eq!(
        owner.multiplexer_authority(&client),
        MultiplexerAuthority::Observer
    );

    let write_request = ProtocolMessage::new(
        client.clone(),
        sequence(92),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::RequestMultiplexerWrite {
            request: RequestMultiplexerWriteV2 {
                client_surface_capability: ClientSurfaceCapability::ControllingTerminal,
            },
        },
    )
    .unwrap();
    let write_response = owner
        .dispatch_multiplexer_protocol_v2(client.clone(), &write_request, sequence(93), 102)
        .unwrap();
    assert!(matches!(
        write_response.payload,
        ProtocolPayload::MultiplexerWriteState {
            state: MultiplexerWriteStateV2 {
                authority: MultiplexerAuthority::MultiplexerWrite,
                error: None,
            }
        }
    ));

    let expected = owner.multiplexer_topology().generation();
    let mutation_request = ProtocolMessage::new(
        client.clone(),
        sequence(94),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: expected,
                operation: TopologyOperationV2::CreateWorkspace {
                    multiplexer_workspace_id: workspace(76),
                    alias: "wire-dispatch".to_owned(),
                    first_tab_id: tab(77),
                    first_tab_alias: "main".to_owned(),
                    first_pane_id: pane(78),
                },
            },
        },
    )
    .unwrap();
    let mutation_response = owner
        .dispatch_multiplexer_protocol_v2(client.clone(), &mutation_request, sequence(95), 103)
        .unwrap();
    assert!(matches!(
        mutation_response.payload,
        ProtocolPayload::MultiplexerSnapshot {
            snapshot: MultiplexerSnapshotV2::MutationResult {
                result: TopologyMutationResultV2 {
                    outcome: TopologyMutationOutcomeV2::Accepted,
                    accepted_topology_generation: Some(_),
                    multiplexer_workspace_id: Some(_),
                    ..
                },
                snapshot: Some(_),
            }
        }
    ));
    assert_eq!(
        owner
            .multiplexer_topology()
            .workspace(workspace(76))
            .unwrap()
            .alias,
        "wire-dispatch"
    );
    assert_eq!(
        owner.multiplexer_topology().generation(),
        expected.checked_next().unwrap()
    );

    assert_eq!(
        owner
            .dispatch_multiplexer_protocol_v2(client.clone(), &mutation_request, sequence(96), 104,)
            .unwrap_err(),
        LocalControlErrorKind::DuplicateOrOutOfOrderRequest
    );
    assert_eq!(
        owner.multiplexer_topology().generation(),
        expected.checked_next().unwrap()
    );

    drop(owner);
    let _ = fs::remove_dir_all(home);
    let _ = fs::remove_dir_all(runtime_root);
}

#[cfg(test)]
mod real_endpoint_tests {
    use super::*;
    use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
    use crate::git::terminal::TerminalSize;
    use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
    use crate::persistent_runtime::client::{
        ClientResponseProjection, ResolvedRuntimeTarget, RustLocalControlClient,
    };
    use crate::persistent_runtime::domain::{ClientAuthority, RuntimeAlias};
    use crate::persistent_runtime::owner::PersistentOwner;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    fn root(label: &str) -> PathBuf {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t166-real-{label}-{}-{serial}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    fn short_root() -> PathBuf {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let base = if cfg!(windows) {
            std::env::temp_dir()
        } else {
            PathBuf::from("/tmp")
        };
        let path = base.join(format!("w166r{serial}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    fn shell_profile(cwd: &Path) -> ShellProfile {
        let candidate = if cfg!(windows) {
            std::env::var("COMSPEC").expect("Windows CI must provide COMSPEC")
        } else {
            "/bin/sh".to_owned()
        };
        let inventory = WorkspaceEnvironmentInventory {
            host_os: std::env::consts::OS.to_owned(),
            host_arch: std::env::consts::ARCH.to_owned(),
            canonical_worktree_root: cwd.to_string_lossy().into_owned(),
            git_common_dir: cwd.to_string_lossy().into_owned(),
            shell_candidates: vec![candidate.clone()],
            detected_manifests: Vec::new(),
        };
        discover_native_shell_profiles(&inventory)
            .unwrap()
            .into_iter()
            .find(|profile| profile.executable == candidate)
            .unwrap()
    }

    fn start_owner(home: &Path, runtime_root: &Path, now: i64) -> PersistentOwner {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let runtime_directory = runtime_root.join("r");
            crate::persistent_runtime::transport::unix::prepare_runtime_directory(
                &runtime_directory,
            )
            .unwrap();
            PersistentOwner::start_for_test(home, &runtime_directory, now).unwrap()
        }
        #[cfg(windows)]
        {
            PersistentOwner::start(home, now).unwrap()
        }
    }

    fn connect_client(
        runtime_root: &Path,
        generation: OwnerGenerationId,
    ) -> RustLocalControlClient {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            RustLocalControlClient::connect_runtime_directory_for_test(
                &runtime_root.join("r"),
                Some(generation),
            )
            .unwrap()
        }
        #[cfg(windows)]
        {
            RustLocalControlClient::connect(None, Some(generation)).unwrap()
        }
    }

    #[test]
    fn t166_real_private_endpoint_preserves_spec011_and_v2_then_allows_next_client() {
        let home = root("home");
        let runtime_root = short_root();
        let mut owner = start_owner(&home, &runtime_root, 100);
        let attachment = owner
            .start_terminal_runtime(
                RuntimeAlias::new("t166-real").unwrap(),
                &shell_profile(&home),
                &home,
                TerminalSize { rows: 24, cols: 80 },
                101,
                0,
            )
            .unwrap();
        let runtime_id = attachment.runtime_namespace_id();
        let generation = owner.generation_id();
        let done = Arc::new(AtomicBool::new(false));
        let done_for_thread = Arc::clone(&done);
        let client_runtime_root = runtime_root.clone();

        let client_thread = thread::spawn(move || {
            let mut client = connect_client(&client_runtime_root, generation);
            let target = ResolvedRuntimeTarget::exact(runtime_id);
            assert!(matches!(
                client.attach_observer(target).unwrap(),
                ClientResponseProjection::RuntimeSnapshot { .. }
            ));
            assert!(matches!(
                client.request_control(target).unwrap(),
                ClientResponseProjection::ControlState {
                    authority: ClientAuthority::Controller,
                    ..
                }
            ));
            for result in [
                client
                    .send_input(target, b"printf ready\n".to_vec())
                    .unwrap(),
                client.resize(target, 100, 30).unwrap(),
                client.interrupt(target).unwrap(),
            ] {
                assert!(matches!(
                    result,
                    ClientResponseProjection::ControlState {
                        authority: ClientAuthority::Controller,
                        ..
                    }
                ));
            }
            assert!(matches!(
                client.release_control(target).unwrap(),
                ClientResponseProjection::ControlState {
                    authority: ClientAuthority::Observer,
                    ..
                }
            ));
            assert!(matches!(
                client.request_control(target).unwrap(),
                ClientResponseProjection::ControlState {
                    authority: ClientAuthority::Controller,
                    ..
                }
            ));
            assert!(matches!(
                client.stop(target).unwrap(),
                ClientResponseProjection::ControlState {
                    authority: ClientAuthority::Observer,
                    ..
                }
            ));
            let _ = client.detach_observer(target);
            client
                .transact(
                    None,
                    ProtocolPayload::ListMultiplexerWorkspaces {
                        request: ListMultiplexerWorkspacesV2 {
                            multiplexer_workspace_id: None,
                        },
                    },
                    false,
                )
                .unwrap();
            client
                .transact(
                    None,
                    ProtocolPayload::RequestMultiplexerWrite {
                        request: RequestMultiplexerWriteV2 {
                            client_surface_capability: ClientSurfaceCapability::ControllingTerminal,
                        },
                    },
                    false,
                )
                .unwrap();
            client
                .transact(
                    None,
                    ProtocolPayload::ApplyTopologyOperation {
                        request: ApplyTopologyOperationV2 {
                            expected_topology_generation: topology_generation(1),
                            operation: TopologyOperationV2::CreateWorkspace {
                                multiplexer_workspace_id: workspace(201),
                                alias: "real-endpoint".to_owned(),
                                first_tab_id: tab(201),
                                first_tab_alias: "main".to_owned(),
                                first_pane_id: pane(201),
                            },
                        },
                    },
                    false,
                )
                .unwrap();
            done_for_thread.store(true, Ordering::Release);
        });

        let deadline = Instant::now() + Duration::from_secs(15);
        while !done.load(Ordering::Acquire) && Instant::now() < deadline {
            owner.service_endpoint_once_for_test(102, 1).unwrap();
            owner.poll_terminal_runtimes(102, 1).unwrap();
            thread::sleep(Duration::from_millis(2));
        }
        client_thread.join().unwrap();
        assert!(done.load(Ordering::Acquire));
        for _ in 0..100 {
            owner.service_endpoint_once_for_test(103, 2).unwrap();
            if !owner.has_active_session_for_test() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert!(!owner.has_active_session_for_test());
        assert!(
            owner
                .terminal_control_state(&connection("unrelated"), runtime_id, 104, 3,)
                .is_err()
        );
        drop(owner);
        let _ = fs::remove_dir_all(home);
        let _ = fs::remove_dir_all(runtime_root);

        let home = root("next");
        let runtime_root = short_root();
        let mut owner = start_owner(&home, &runtime_root, 200);
        let generation = owner.generation_id();
        let connected = Arc::new(AtomicBool::new(false));
        let connected_for_thread = Arc::clone(&connected);
        let second_runtime_root = runtime_root.clone();
        let second = thread::spawn(move || {
            let client = connect_client(&second_runtime_root, generation);
            assert_eq!(client.owner_generation_id(), generation);
            connected_for_thread.store(true, Ordering::Release);
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while !connected.load(Ordering::Acquire) && Instant::now() < deadline {
            owner.service_endpoint_once_for_test(201, 4).unwrap();
            thread::sleep(Duration::from_millis(2));
        }
        second.join().unwrap();
        assert!(connected.load(Ordering::Acquire));
        for _ in 0..100 {
            owner.service_endpoint_once_for_test(202, 5).unwrap();
            if !owner.has_active_session_for_test() {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert!(!owner.has_active_session_for_test());
        drop(owner);
        let _ = fs::remove_dir_all(home);
        let _ = fs::remove_dir_all(runtime_root);
    }
}
