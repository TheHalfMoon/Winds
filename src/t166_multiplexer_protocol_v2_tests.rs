use super::*;
use crate::multiplexer::domain::navigation::{
    LayoutNode, MultiplexerTopology, SplitAxis as DomainSplitAxis, SplitRatioBps, TabState,
    WorkspaceState,
};
use crate::multiplexer::domain::{
    AgentObservationId, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId, RuntimeNamespaceId,
};
use crate::persistent_runtime::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationCursorV2, AgentObservationEventV2,
    AgentObservationFreshnessV2, AgentObservationSnapshotV2, AgentObservationSourceV2,
    AgentObservationV2, ApplyTopologyOperationV2, ApplyWorktreeOperationV2, AttentionEventV2,
    AttentionItemV2, AttentionKindV2, AttentionSnapshotV2, LEGACY_PROTOCOL_VERSION,
    ListAgentObservationsV2, ListWorktreesV2, MAX_CONTROL_FRAME_BYTES,
    MAX_INBOUND_CONTROL_FRAME_BYTES, MAX_V2_AGENT_OBSERVATIONS_PER_PAGE, MAX_V2_ALIAS_BYTES,
    MAX_V2_ATTENTION_ITEMS_PER_PAGE, MAX_V2_BRANCH_BYTES, MAX_V2_DETAIL_BYTES,
    MAX_V2_EVIDENCE_SUMMARY_BYTES, MAX_V2_GIT_WORKSPACE_ID_BYTES, MAX_V2_PANES_PER_TAB,
    MAX_V2_PATH_BYTES, MAX_V2_PROVIDER_SESSION_ID_BYTES, MAX_V2_REPOSITORY_IDENTITY_BYTES,
    MAX_V2_TABS_PER_WORKSPACE, MAX_V2_WORKTREES_PER_PAGE, MessageAuthorityClass, MessageKind,
    MultiplexerEventV2, MultiplexerSnapshotV2, PROTOCOL_VERSION, ProtocolLayoutNodeV2,
    ProtocolPanePlacement, ProtocolSplitAxis, ProtocolTabSnapshotV2, ProtocolWorkspaceSnapshotV2,
    TopologyMutationOutcomeV2, TopologyMutationResultV2, TopologyOperationV2, WorktreeCursorV2,
    WorktreeMembershipV2, WorktreeObservationV2, WorktreeOperationOutcomeV2,
    WorktreeOperationResultV2, WorktreeOperationV2, decode_frame, encode_frame,
    inactive_v2_domain_response, validate_candidate_topology_v2, validate_owner_v2_handshake,
    validate_response_binding,
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
fn t166_typed_worktree_page_is_bounded_and_single_frame_safe() {
    let result = WorktreeOperationResultV2 {
        outcome: WorktreeOperationOutcomeV2::Accepted,
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
                multiplexer_workspace_id: Some(workspace(13)),
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
fn t166_inactive_future_domains_return_typed_unsupported_without_side_effect_path() {
    let request = ProtocolMessage::new(
        connection("t166-inactive"),
        sequence(50),
        None,
        generation(35),
        None,
        ProtocolPayload::ListWorktrees {
            request: ListWorktreesV2 {
                multiplexer_workspace_id: Some(workspace(35)),
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
        receive: VecDeque::from([Err(WireError::LegacyProtocol)]),
    };
    let error = RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), None)
        .err()
        .unwrap();
    assert_eq!(error, LocalControlClientError::BlockedLegacyOwner);
}
