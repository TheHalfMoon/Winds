use crate::multiplexer::domain::{
    MultiplexerErrorKind, MultiplexerWorkspaceId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{LocalControlErrorKind, OwnerGenerationId};
use crate::persistent_runtime::protocol::{
    MultiplexerEventSubscriptionAckV2, MultiplexerEventV2, MultiplexerSnapshotV2,
    MultiplexerSubscriptionBoundaryV2, MultiplexerSubscriptionStreamV2,
    SubscribeMultiplexerEventsV2,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopologyProjectionFreshness {
    Unsubscribed,
    NeedsSnapshot,
    Current,
    NeedsResubscribe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopologyProjectionError {
    Protocol(LocalControlErrorKind),
    Multiplexer(MultiplexerErrorKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopologyProjection {
    owner_generation_id: OwnerGenerationId,
    subscription: Option<SubscribeMultiplexerEventsV2>,
    subscription_boundary: Option<TopologyGeneration>,
    observed_generation: Option<TopologyGeneration>,
    trusted_snapshot: Option<MultiplexerSnapshotV2>,
    freshness: TopologyProjectionFreshness,
}

impl TopologyProjection {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            subscription: None,
            subscription_boundary: None,
            observed_generation: None,
            trusted_snapshot: None,
            freshness: TopologyProjectionFreshness::Unsubscribed,
        }
    }

    pub(crate) fn reset_for_connection(&mut self, owner_generation_id: OwnerGenerationId) {
        *self = Self::new(owner_generation_id);
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }

    pub(crate) fn freshness(&self) -> TopologyProjectionFreshness {
        self.freshness
    }

    pub(crate) fn subscription(&self) -> Option<&SubscribeMultiplexerEventsV2> {
        self.subscription.as_ref()
    }

    pub(crate) fn subscription_filter(&self) -> Option<Option<MultiplexerWorkspaceId>> {
        self.subscription
            .as_ref()
            .map(|subscription| subscription.multiplexer_workspace_id)
    }

    pub(crate) fn observed_generation(&self) -> Option<TopologyGeneration> {
        self.observed_generation
    }

    pub(crate) fn trusted_snapshot(&self) -> Option<&MultiplexerSnapshotV2> {
        (self.freshness == TopologyProjectionFreshness::Current)
            .then_some(self.trusted_snapshot.as_ref())
            .flatten()
    }

    pub(crate) fn accept_subscription_ack(
        &mut self,
        request: &SubscribeMultiplexerEventsV2,
        ack: &MultiplexerEventSubscriptionAckV2,
    ) -> Result<TopologyGeneration, TopologyProjectionError> {
        if request.stream != MultiplexerSubscriptionStreamV2::Topology
            || ack.stream != MultiplexerSubscriptionStreamV2::Topology
            || ack.multiplexer_workspace_id != request.multiplexer_workspace_id
        {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ));
        }
        let MultiplexerSubscriptionBoundaryV2::Topology {
            topology_generation,
        } = ack.boundary
        else {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        };

        self.subscription = Some(request.clone());
        self.subscription_boundary = Some(topology_generation);
        self.observed_generation = Some(topology_generation);
        self.trusted_snapshot = None;
        self.freshness = TopologyProjectionFreshness::NeedsSnapshot;
        Ok(topology_generation)
    }

    pub(crate) fn accept_snapshot(
        &mut self,
        snapshot: &MultiplexerSnapshotV2,
    ) -> Result<TopologyGeneration, TopologyProjectionError> {
        if self.freshness == TopologyProjectionFreshness::NeedsResubscribe {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }
        let subscription = self
            .subscription
            .as_ref()
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ))?;
        let generation = match (subscription.multiplexer_workspace_id, snapshot) {
            (
                None,
                MultiplexerSnapshotV2::WorkspaceList {
                    topology_generation,
                    ..
                },
            ) => *topology_generation,
            (Some(expected_workspace_id), MultiplexerSnapshotV2::Workspace { snapshot })
                if snapshot.multiplexer_workspace_id == expected_workspace_id =>
            {
                snapshot.topology_generation
            }
            _ => {
                return Err(TopologyProjectionError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ));
            }
        };

        let boundary = self
            .subscription_boundary
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ))?;
        let observed = self.observed_generation.unwrap_or(boundary);
        if generation < boundary || generation < observed {
            self.trusted_snapshot = None;
            self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        self.observed_generation = Some(generation);
        self.trusted_snapshot = Some(snapshot.clone());
        self.freshness = TopologyProjectionFreshness::Current;
        Ok(generation)
    }

    pub(crate) fn accept_event(
        &mut self,
        event: &MultiplexerEventV2,
    ) -> Result<(), TopologyProjectionError> {
        let subscription = self
            .subscription
            .as_ref()
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ))?;
        if self.freshness == TopologyProjectionFreshness::NeedsResubscribe {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        let (workspace_id, generation) = match event {
            MultiplexerEventV2::TopologyChanged {
                multiplexer_workspace_id,
                topology_generation,
            }
            | MultiplexerEventV2::PaneCleared {
                multiplexer_workspace_id,
                topology_generation,
                ..
            } => (Some(*multiplexer_workspace_id), Some(*topology_generation)),
            MultiplexerEventV2::HistoryGap {
                last_known_topology_generation,
            } => {
                self.observed_generation = Some(*last_known_topology_generation);
                self.trusted_snapshot = None;
                self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
                return Ok(());
            }
        };

        if let (Some(expected), Some(actual)) =
            (subscription.multiplexer_workspace_id, workspace_id)
            && expected != actual
        {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }
        let generation = generation.ok_or(TopologyProjectionError::Protocol(
            LocalControlErrorKind::MalformedFrame,
        ))?;
        let previous = self
            .observed_generation
            .or(self.subscription_boundary)
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ))?;
        if generation <= previous {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        if subscription.multiplexer_workspace_id.is_none() {
            let expected = previous.checked_next().map_err(|_| {
                TopologyProjectionError::Protocol(LocalControlErrorKind::MalformedFrame)
            })?;
            if generation != expected {
                self.observed_generation = Some(generation);
                self.trusted_snapshot = None;
                self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
                return Ok(());
            }
        }

        self.observed_generation = Some(generation);
        self.trusted_snapshot = None;
        self.freshness = TopologyProjectionFreshness::NeedsSnapshot;
        Ok(())
    }
}
