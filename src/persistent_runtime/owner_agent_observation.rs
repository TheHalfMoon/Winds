use super::{OwnerConnectionSession, OwnerError, OwnerResult, PersistentOwner};
use crate::multiplexer::agent_state::{ObservationCandidate, ObservationError};
use crate::multiplexer::domain::MultiplexerWorkspaceId;
use crate::persistent_runtime::domain::{EventSequence, LocalControlErrorKind};
use crate::persistent_runtime::protocol::{
    AgentObservationEventV2, MultiplexerEventSubscriptionAckV2, MultiplexerSubscriptionBoundaryV2,
    MultiplexerSubscriptionStreamV2, ProtocolMessage, ProtocolPayload, ProtocolResult,
};

impl PersistentOwner {
    pub(crate) fn record_agent_observation_candidate(
        &mut self,
        candidate: ObservationCandidate,
    ) -> OwnerResult<()> {
        let events = {
            let topology = self.multiplexer_service.topology();
            self.agent_observation_store
                .record(topology, self.generation_id, candidate)
                .map_err(map_agent_observation_owner_error)?
        };
        if events.is_empty() {
            return Ok(());
        }

        let Some(mut session) = self.session.take() else {
            return Ok(());
        };
        let result = self.queue_agent_observation_events(&mut session, &events);
        self.session = Some(session);
        result
    }

    pub(super) fn dispatch_agent_observation_list(
        &mut self,
        request: &ProtocolMessage,
        response_sequence: EventSequence,
    ) -> ProtocolResult<ProtocolMessage> {
        let ProtocolPayload::ListAgentObservations { request: list } = &request.payload else {
            return Err(LocalControlErrorKind::UnsupportedOperation);
        };

        let snapshot = {
            let topology = self.multiplexer_service.topology();
            self.agent_observation_store
                .invalidate_against(topology)
                .map_err(map_agent_observation_protocol_error)?;

            if let Some(cursor) = &list.cursor {
                if cursor.owner_generation_id != self.generation_id {
                    return Err(LocalControlErrorKind::StaleOwnerGeneration);
                }
                if cursor.snapshot_revision != self.agent_observation_store.snapshot_revision() {
                    return Err(LocalControlErrorKind::OutcomeUnknown);
                }
            }

            self.agent_observation_store
                .snapshot(topology, list)
                .map_err(map_agent_observation_protocol_error)?
        };

        self.respond_simple(
            request,
            response_sequence,
            ProtocolPayload::AgentObservationSnapshot { snapshot },
        )
    }

    pub(super) fn dispatch_agent_observation_subscription(
        &mut self,
        session: &mut OwnerConnectionSession,
        request: &ProtocolMessage,
        response_sequence: EventSequence,
    ) -> ProtocolResult<ProtocolMessage> {
        let ProtocolPayload::SubscribeMultiplexerEvents {
            request: subscription,
        } = &request.payload
        else {
            return Err(LocalControlErrorKind::UnsupportedOperation);
        };
        if subscription.stream != MultiplexerSubscriptionStreamV2::AgentObservations {
            return Err(LocalControlErrorKind::UnsupportedOperation);
        }

        let snapshot_revision = {
            let topology = self.multiplexer_service.topology();
            self.agent_observation_store
                .invalidate_against(topology)
                .map_err(map_agent_observation_protocol_error)?;
            self.agent_observation_store.snapshot_revision()
        };
        session.agent_observation_subscription =
            Some((subscription.multiplexer_workspace_id, snapshot_revision));

        self.respond_simple(
            request,
            response_sequence,
            ProtocolPayload::MultiplexerEventSubscriptionAck {
                ack: MultiplexerEventSubscriptionAckV2 {
                    stream: MultiplexerSubscriptionStreamV2::AgentObservations,
                    multiplexer_workspace_id: subscription.multiplexer_workspace_id,
                    boundary: MultiplexerSubscriptionBoundaryV2::AgentObservations {
                        snapshot_revision,
                    },
                },
            },
        )
    }

    pub(super) fn queue_agent_observation_events_after_response(
        &mut self,
        session: &mut OwnerConnectionSession,
        request: &ProtocolMessage,
        response: &ProtocolMessage,
    ) -> OwnerResult<()> {
        if !matches!(
            request.payload,
            ProtocolPayload::ApplyTopologyOperation { .. }
        ) {
            return Ok(());
        }
        let ProtocolPayload::MultiplexerSnapshot {
            snapshot:
                crate::persistent_runtime::protocol::MultiplexerSnapshotV2::MutationResult {
                    result,
                    ..
                },
        } = &response.payload
        else {
            return Ok(());
        };
        if result.outcome
            != crate::persistent_runtime::protocol::TopologyMutationOutcomeV2::Accepted
        {
            return Ok(());
        }

        let events = {
            let topology = self.multiplexer_service.topology();
            self.agent_observation_store
                .invalidate_against(topology)
                .map_err(map_agent_observation_owner_error)?
        };
        self.queue_agent_observation_events(session, &events)
    }

    pub(super) fn pump_agent_observation_subscription_gap(
        &mut self,
        session: &mut OwnerConnectionSession,
    ) -> OwnerResult<()> {
        let Some((_, last_snapshot_revision)) = session.agent_observation_subscription else {
            return Ok(());
        };

        let events = {
            let topology = self.multiplexer_service.topology();
            self.agent_observation_store
                .invalidate_against(topology)
                .map_err(map_agent_observation_owner_error)?
        };
        if !events.is_empty() {
            return self.queue_agent_observation_events(session, &events);
        }

        let current_revision = self.agent_observation_store.snapshot_revision();
        if current_revision == last_snapshot_revision {
            return Ok(());
        }

        session.agent_observation_subscription = None;
        self.queue_agent_observation_event(
            session,
            AgentObservationEventV2::HistoryGap {
                last_known_snapshot_revision: last_snapshot_revision,
            },
        )
    }

    fn queue_agent_observation_events(
        &self,
        session: &mut OwnerConnectionSession,
        events: &[AgentObservationEventV2],
    ) -> OwnerResult<()> {
        let Some((filter, last_snapshot_revision)) = session.agent_observation_subscription else {
            return Ok(());
        };
        let Some(first_revision) = events.first().and_then(agent_event_revision) else {
            return Ok(());
        };
        if events
            .iter()
            .any(|event| agent_event_revision(event) != Some(first_revision))
        {
            session.agent_observation_subscription = None;
            return self.queue_agent_observation_event(
                session,
                AgentObservationEventV2::HistoryGap {
                    last_known_snapshot_revision: last_snapshot_revision,
                },
            );
        }

        let expected_revision = last_snapshot_revision.checked_add(1).ok_or_else(|| {
            OwnerError::Endpoint("agent observation subscription revision exhausted".to_owned())
        })?;
        if first_revision != expected_revision {
            session.agent_observation_subscription = None;
            return self.queue_agent_observation_event(
                session,
                AgentObservationEventV2::HistoryGap {
                    last_known_snapshot_revision: last_snapshot_revision,
                },
            );
        }

        if let Some(subscription) = session.agent_observation_subscription.as_mut() {
            subscription.1 = first_revision;
        }
        for event in events {
            if agent_event_matches_workspace(event, filter) {
                self.queue_agent_observation_event(session, event.clone())?;
            }
        }
        Ok(())
    }

    fn queue_agent_observation_event(
        &self,
        session: &mut OwnerConnectionSession,
        event: AgentObservationEventV2,
    ) -> OwnerResult<()> {
        let sequence = session
            .next_sequence()
            .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;
        let message = ProtocolMessage::new(
            session.connection_id.clone(),
            sequence,
            None,
            self.generation_id,
            None,
            ProtocolPayload::AgentObservationEvent { event },
        )
        .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;
        self.queue_session_message(session, message)
    }
}

fn agent_event_revision(event: &AgentObservationEventV2) -> Option<u64> {
    match event {
        AgentObservationEventV2::Upsert {
            snapshot_revision, ..
        }
        | AgentObservationEventV2::Removed {
            snapshot_revision, ..
        } => Some(*snapshot_revision),
        AgentObservationEventV2::HistoryGap { .. } => None,
    }
}

fn agent_event_matches_workspace(
    event: &AgentObservationEventV2,
    filter: Option<MultiplexerWorkspaceId>,
) -> bool {
    let Some(expected) = filter else {
        return true;
    };
    match event {
        AgentObservationEventV2::Upsert { observation, .. } => {
            observation.multiplexer_workspace_id == expected
        }
        AgentObservationEventV2::Removed {
            multiplexer_workspace_id,
            ..
        } => *multiplexer_workspace_id == expected,
        AgentObservationEventV2::HistoryGap { .. } => true,
    }
}

fn map_agent_observation_protocol_error(error: ObservationError) -> LocalControlErrorKind {
    match error {
        ObservationError::StaleOwnerGeneration => LocalControlErrorKind::StaleOwnerGeneration,
        ObservationError::Unrepresentable | ObservationError::ObservationIdentityReuse => {
            LocalControlErrorKind::MalformedFrame
        }
        ObservationError::SnapshotRevisionExhausted => LocalControlErrorKind::OutcomeUnknown,
        ObservationError::Topology(_) => LocalControlErrorKind::OutcomeUnknown,
    }
}

fn map_agent_observation_owner_error(error: ObservationError) -> OwnerError {
    OwnerError::Runtime(format!("agent observation failed: {error:?}"))
}
