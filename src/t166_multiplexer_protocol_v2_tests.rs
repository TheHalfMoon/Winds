use super::*;
use crate::multiplexer::domain::{
    AgentObservationId, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationCursorV2,
    AgentObservationFreshnessV2, AgentObservationSnapshotV2, AgentObservationSourceV2,
    AgentObservationV2, ApplyTopologyOperationV2, ApplyWorktreeOperationV2,
    LEGACY_PROTOCOL_VERSION, ListAgentObservationsV2, ListWorktreesV2,
    MAX_CONTROL_FRAME_BYTES, MAX_INBOUND_CONTROL_FRAME_BYTES, MAX_V2_BRANCH_BYTES,
    MAX_V2_PATH_BYTES, MAX_V2_REPOSITORY_IDENTITY_BYTES, MAX_V2_WORKTREES_PER_PAGE,
    MessageAuthorityClass, MessageKind, PROTOCOL_VERSION, ProtocolPanePlacement,
    ProtocolSplitAxis, TopologyOperationV2, WorktreeCursorV2, WorktreeMembershipV2,
    WorktreeObservationV2, WorktreeOperationOutcomeV2, WorktreeOperationResultV2,
    WorktreeOperationV2, decode_frame, encode_frame,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId,
    RuntimeNamespaceId,
};
use crate::persistent_runtime::protocol::{ProtocolMessage, ProtocolPayload};
use std::collections::VecDeque;

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

fn observation(byte: u8) -> AgentObservationId {
    AgentObservationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn topology_generation(value: u64) -> TopologyGeneration {
    TopologyGeneration::new(value).unwrap()
}

fn connection(value: &str) -> ClientConnectionId {
    ClientConnectionId::new(value.to_owned()).unwrap()
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
        git_workspace_id: format!("git-{index:03}-{}", "g".repeat(240)),
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
    let current_frame = encode_frame(&current).unwrap();
    let current_json = std::str::from_utf8(&current_frame[4..]).unwrap();
    assert!(current_json.contains(r#""protocol_version":2"#));
    assert_eq!(decode_frame(&current_frame).unwrap(), current);

    let legacy = ProtocolMessage::hello(sequence(2), 1, 1, None).unwrap();
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
fn t166_agent_observation_schema_keeps_workspace_domains_unambiguous() {
    let payload = ProtocolPayload::AgentObservationSnapshot {
        snapshot: AgentObservationSnapshotV2 {
            snapshot_revision: 1,
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
fn t166_typed_worktree_page_is_bounded_and_single_frame_safe() {
    let result = WorktreeOperationResultV2 {
        outcome: WorktreeOperationOutcomeV2::UnsupportedOperation,
        worktrees: (0..MAX_V2_WORKTREES_PER_PAGE).map(worktree).collect(),
        next_cursor: Some(WorktreeCursorV2 {
            owner_generation_id: generation(13),
            snapshot_revision: 1,
            offset: MAX_V2_WORKTREES_PER_PAGE as u16,
        }),
    };
    let response = ProtocolMessage::new(
        connection("t166-client"),
        sequence(14),
        None,
        generation(13),
        Some(sequence(7)),
        ProtocolPayload::WorktreeOperationResult { result },
    )
    .unwrap();
    let frame = encode_frame(&response).unwrap();
    assert!(frame.len() <= MAX_CONTROL_FRAME_BYTES);
    assert_eq!(decode_frame(&frame).unwrap(), response);

    let too_many = WorktreeOperationResultV2 {
        outcome: WorktreeOperationOutcomeV2::UnsupportedOperation,
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
fn t166_cursor_revision_mismatch_fails_closed() {
    let error = ProtocolMessage::new(
        connection("t166-client"),
        sequence(16),
        None,
        generation(15),
        Some(sequence(7)),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                snapshot_revision: 10,
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
fn t166_future_domain_mutation_is_typed_without_runtime_controller_authority() {
    let worktree = v2_message(ProtocolPayload::ApplyWorktreeOperation {
        request: ApplyWorktreeOperationV2 {
            operation: WorktreeOperationV2::Open {
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
            multiplexer_workspace_id: Some(workspace(4)),
            cursor: None,
        },
    });
    assert_eq!(worktrees.runtime_namespace_id, None);
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
        receive: VecDeque::from([Err(WireError::Protocol(
            LocalControlErrorKind::ProtocolMismatch,
        ))]),
    };
    let error =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), None).unwrap_err();
    assert_eq!(error, LocalControlClientError::BlockedLegacyOwner);
}
