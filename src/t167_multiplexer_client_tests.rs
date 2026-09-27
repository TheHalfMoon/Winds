use super::*;
use crate::multiplexer::domain::{
    MultiplexerErrorKind, MultiplexerWorkspaceId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId,
};
use crate::persistent_runtime::protocol::{
    MultiplexerEventSubscriptionAckV2, MultiplexerEventV2, MultiplexerSnapshotV2,
    MultiplexerSubscriptionBoundaryV2, MultiplexerSubscriptionStreamV2, ProtocolWorkspaceSummaryV2,
    SubscribeMultiplexerEventsV2,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn sequence(value: u64) -> EventSequence {
    EventSequence::new(value).unwrap()
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn topology_generation(value: u64) -> TopologyGeneration {
    TopologyGeneration::new(value).unwrap()
}

fn connection(value: &str) -> ClientConnectionId {
    ClientConnectionId::new(value).unwrap()
}

fn topology_subscription(
    multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
) -> SubscribeMultiplexerEventsV2 {
    SubscribeMultiplexerEventsV2 {
        stream: MultiplexerSubscriptionStreamV2::Topology,
        multiplexer_workspace_id,
    }
}

fn topology_ack(
    multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
    generation: u64,
) -> MultiplexerEventSubscriptionAckV2 {
    MultiplexerEventSubscriptionAckV2 {
        stream: MultiplexerSubscriptionStreamV2::Topology,
        multiplexer_workspace_id,
        boundary: MultiplexerSubscriptionBoundaryV2::Topology {
            topology_generation: topology_generation(generation),
        },
    }
}

fn workspace_list_snapshot(generation: u64) -> MultiplexerSnapshotV2 {
    MultiplexerSnapshotV2::WorkspaceList {
        topology_generation: topology_generation(generation),
        workspaces: Vec::<ProtocolWorkspaceSummaryV2>::new(),
    }
}

#[test]
fn t167_projection_requires_snapshot_after_subscription_and_after_event() {
    let mut projection = TopologyProjection::new(generation(1));
    let subscription = topology_subscription(None);
    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 1))
        .unwrap();
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::NeedsSnapshot
    );
    assert!(projection.trusted_snapshot().is_none());

    projection
        .accept_snapshot(&workspace_list_snapshot(1))
        .unwrap();
    assert_eq!(projection.freshness(), TopologyProjectionFreshness::Current);
    assert!(projection.trusted_snapshot().is_some());

    projection
        .accept_event(&MultiplexerEventV2::TopologyChanged {
            multiplexer_workspace_id: workspace(1),
            topology_generation: topology_generation(2),
        })
        .unwrap();
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::NeedsSnapshot
    );
    assert!(projection.trusted_snapshot().is_none());

    projection
        .accept_snapshot(&workspace_list_snapshot(2))
        .unwrap();
    assert_eq!(projection.freshness(), TopologyProjectionFreshness::Current);
}

#[test]
fn t167_gap_and_unfiltered_generation_discontinuity_require_resubscribe() {
    let mut projection = TopologyProjection::new(generation(2));
    let subscription = topology_subscription(None);
    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 3))
        .unwrap();
    projection
        .accept_snapshot(&workspace_list_snapshot(3))
        .unwrap();

    projection
        .accept_event(&MultiplexerEventV2::TopologyChanged {
            multiplexer_workspace_id: workspace(2),
            topology_generation: topology_generation(5),
        })
        .unwrap();
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::NeedsResubscribe
    );
    assert_eq!(
        projection
            .accept_snapshot(&workspace_list_snapshot(5))
            .unwrap_err(),
        TopologyProjectionError::Multiplexer(MultiplexerErrorKind::StaleTopologyGeneration)
    );

    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 5))
        .unwrap();
    projection
        .accept_snapshot(&workspace_list_snapshot(5))
        .unwrap();
    projection
        .accept_event(&MultiplexerEventV2::HistoryGap {
            last_known_topology_generation: topology_generation(5),
        })
        .unwrap();
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::NeedsResubscribe
    );
}

#[test]
fn t167_workspace_filter_rejects_different_workspace_event() {
    let expected = workspace(3);
    let mut projection = TopologyProjection::new(generation(3));
    projection
        .accept_subscription_ack(
            &topology_subscription(Some(expected)),
            &topology_ack(Some(expected), 7),
        )
        .unwrap();

    assert_eq!(
        projection
            .accept_event(&MultiplexerEventV2::TopologyChanged {
                multiplexer_workspace_id: workspace(4),
                topology_generation: topology_generation(8),
            })
            .unwrap_err(),
        TopologyProjectionError::Protocol(LocalControlErrorKind::MalformedFrame)
    );
}

#[test]
fn t167_reconnect_reset_clears_subscription_and_cache_authority() {
    let mut projection = TopologyProjection::new(generation(4));
    let subscription = topology_subscription(None);
    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 9))
        .unwrap();
    projection
        .accept_snapshot(&workspace_list_snapshot(9))
        .unwrap();
    assert!(projection.trusted_snapshot().is_some());

    projection.reset_for_connection(generation(4));
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::Unsubscribed
    );
    assert!(projection.subscription().is_none());
    assert!(projection.trusted_snapshot().is_none());
}

#[test]
fn t167_multi_client_projections_converge_after_event_and_authoritative_snapshot() {
    let owner_generation = generation(6);
    let subscription = topology_subscription(None);
    let mut first = TopologyProjection::new(owner_generation);
    let mut second = TopologyProjection::new(owner_generation);

    for projection in [&mut first, &mut second] {
        projection
            .accept_subscription_ack(&subscription, &topology_ack(None, 10))
            .unwrap();
        projection
            .accept_snapshot(&workspace_list_snapshot(10))
            .unwrap();
        projection
            .accept_event(&MultiplexerEventV2::TopologyChanged {
                multiplexer_workspace_id: workspace(6),
                topology_generation: topology_generation(11),
            })
            .unwrap();
        assert_eq!(
            projection.freshness(),
            TopologyProjectionFreshness::NeedsSnapshot
        );
        projection
            .accept_snapshot(&workspace_list_snapshot(11))
            .unwrap();
    }

    assert_eq!(first.freshness(), TopologyProjectionFreshness::Current);
    assert_eq!(second.freshness(), TopologyProjectionFreshness::Current);
    assert_eq!(first.observed_generation(), second.observed_generation());
    assert_eq!(first.trusted_snapshot(), second.trusted_snapshot());
}

#[test]
fn t167_owner_restart_invalidates_old_projection_until_fresh_subscription_and_snapshot() {
    let old_generation = generation(7);
    let new_generation = generation(8);
    let subscription = topology_subscription(None);
    let mut projection = TopologyProjection::new(old_generation);

    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 20))
        .unwrap();
    projection
        .accept_snapshot(&workspace_list_snapshot(20))
        .unwrap();
    assert_eq!(projection.freshness(), TopologyProjectionFreshness::Current);

    projection.reset_for_connection(new_generation);
    assert_eq!(projection.owner_generation_id(), new_generation);
    assert_eq!(
        projection.freshness(),
        TopologyProjectionFreshness::Unsubscribed
    );
    assert_eq!(
        projection
            .accept_snapshot(&workspace_list_snapshot(20))
            .unwrap_err(),
        TopologyProjectionError::Protocol(LocalControlErrorKind::UnsupportedOperation)
    );
    assert!(projection.trusted_snapshot().is_none());

    projection
        .accept_subscription_ack(&subscription, &topology_ack(None, 20))
        .unwrap();
    projection
        .accept_snapshot(&workspace_list_snapshot(20))
        .unwrap();
    assert_eq!(projection.freshness(), TopologyProjectionFreshness::Current);
    assert_eq!(projection.owner_generation_id(), new_generation);
}

struct ScriptedWire {
    responses: VecDeque<ProtocolMessage>,
    sent: Arc<Mutex<Vec<MessageKind>>>,
}

impl LocalControlWire for ScriptedWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        self.sent.lock().unwrap().push(message.kind());
        Ok(())
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        self.responses
            .pop_front()
            .ok_or_else(|| WireError::Transport("scripted response exhausted".to_owned()))
    }
}

#[test]
fn t167_read_only_refresh_never_requests_multiplexer_write() {
    let owner_generation = generation(5);
    let connection_id = connection("t167-read-only");
    let responses = VecDeque::from([
        ProtocolMessage::new(
            connection_id.clone(),
            sequence(1),
            None,
            owner_generation,
            Some(sequence(1)),
            ProtocolPayload::HelloAck,
        )
        .unwrap(),
        ProtocolMessage::new(
            connection_id.clone(),
            sequence(2),
            None,
            owner_generation,
            Some(sequence(2)),
            ProtocolPayload::MultiplexerEventSubscriptionAck {
                ack: topology_ack(None, 1),
            },
        )
        .unwrap(),
        ProtocolMessage::new(
            connection_id,
            sequence(3),
            None,
            owner_generation,
            Some(sequence(3)),
            ProtocolPayload::MultiplexerSnapshot {
                snapshot: workspace_list_snapshot(1),
            },
        )
        .unwrap(),
    ]);
    let sent = Arc::new(Mutex::new(Vec::new()));
    let wire = ScriptedWire {
        responses,
        sent: Arc::clone(&sent),
    };
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), None).unwrap();

    let snapshot = client.refresh_topology_projection(None).unwrap();
    assert!(matches!(
        snapshot,
        MultiplexerSnapshotV2::WorkspaceList { .. }
    ));
    assert_eq!(
        client.topology_projection_freshness(),
        TopologyProjectionFreshness::Current
    );
    let sent = sent.lock().unwrap();
    assert_eq!(
        sent.as_slice(),
        &[
            MessageKind::Hello,
            MessageKind::SubscribeMultiplexerEvents,
            MessageKind::ListMultiplexerWorkspaces,
        ]
    );
    assert!(!sent.contains(&MessageKind::RequestMultiplexerWrite));
}
