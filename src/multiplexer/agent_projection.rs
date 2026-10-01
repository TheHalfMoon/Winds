//! Client-side trust projection for bounded agent observations.
//!
//! The projection deliberately treats observation events as invalidation signals
//! rather than mutating a trusted cache incrementally. A fresh authoritative
//! snapshot is required after every accepted upsert/removal event. This keeps a
//! multi-event revision, duplicate transport delivery, or filtered global
//! revision movement from becoming cache authority. A history gap is stronger:
//! it requires a fresh subscription boundary before any snapshot can be trusted.

use crate::multiplexer::domain::{AgentObservationId, MultiplexerWorkspaceId};
use crate::persistent_runtime::domain::{LocalControlErrorKind, OwnerGenerationId};
use crate::persistent_runtime::protocol::{
    AgentObservationEventV2, AgentObservationSnapshotV2, AgentObservationV2,
    MultiplexerEventSubscriptionAckV2, MultiplexerSubscriptionBoundaryV2,
    MultiplexerSubscriptionStreamV2, SubscribeMultiplexerEventsV2,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentObservationProjectionFreshness {
    Unsubscribed,
    NeedsSnapshot,
    Current,
    NeedsResubscribe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentObservationProjectionError {
    Protocol(LocalControlErrorKind),
    StaleSnapshotRevision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentObservationProjection {
    owner_generation_id: OwnerGenerationId,
    subscription: Option<SubscribeMultiplexerEventsV2>,
    subscription_boundary: Option<u64>,
    observed_revision: Option<u64>,
    hydration_revision: Option<u64>,
    next_page_offset: u16,
    records: BTreeMap<AgentObservationId, AgentObservationV2>,
    freshness: AgentObservationProjectionFreshness,
}

impl AgentObservationProjection {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            subscription: None,
            subscription_boundary: None,
            observed_revision: None,
            hydration_revision: None,
            next_page_offset: 0,
            records: BTreeMap::new(),
            freshness: AgentObservationProjectionFreshness::Unsubscribed,
        }
    }

    pub(crate) fn reset_for_connection(&mut self, owner_generation_id: OwnerGenerationId) {
        *self = Self::new(owner_generation_id);
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }

    pub(crate) fn freshness(&self) -> AgentObservationProjectionFreshness {
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

    pub(crate) fn observed_revision(&self) -> Option<u64> {
        self.observed_revision
    }

    pub(crate) fn trusted_observations(&self) -> Option<Vec<AgentObservationV2>> {
        (self.freshness == AgentObservationProjectionFreshness::Current)
            .then(|| self.records.values().cloned().collect())
    }

    pub(crate) fn accept_subscription_ack(
        &mut self,
        request: &SubscribeMultiplexerEventsV2,
        ack: &MultiplexerEventSubscriptionAckV2,
    ) -> Result<u64, AgentObservationProjectionError> {
        if request.stream != MultiplexerSubscriptionStreamV2::AgentObservations
            || ack.stream != MultiplexerSubscriptionStreamV2::AgentObservations
            || ack.multiplexer_workspace_id != request.multiplexer_workspace_id
        {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ));
        }
        let MultiplexerSubscriptionBoundaryV2::AgentObservations { snapshot_revision } = ack.boundary
        else {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        };
        if snapshot_revision == 0 {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }

        self.subscription = Some(request.clone());
        self.subscription_boundary = Some(snapshot_revision);
        self.observed_revision = Some(snapshot_revision);
        self.hydration_revision = None;
        self.next_page_offset = 0;
        self.records.clear();
        self.freshness = AgentObservationProjectionFreshness::NeedsSnapshot;
        Ok(snapshot_revision)
    }

    pub(crate) fn accept_snapshot(
        &mut self,
        snapshot: &AgentObservationSnapshotV2,
    ) -> Result<u64, AgentObservationProjectionError> {
        if self.freshness == AgentObservationProjectionFreshness::NeedsResubscribe {
            return Err(AgentObservationProjectionError::StaleSnapshotRevision);
        }
        let subscription = self.subscription.as_ref().ok_or(
            AgentObservationProjectionError::Protocol(LocalControlErrorKind::UnsupportedOperation),
        )?;
        if snapshot.filter_multiplexer_workspace_id != subscription.multiplexer_workspace_id
            || snapshot.snapshot_revision == 0
        {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }
        if let Some(workspace_id) = subscription.multiplexer_workspace_id
            && snapshot
                .observations
                .iter()
                .any(|observation| observation.multiplexer_workspace_id != workspace_id)
        {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }

        let boundary = self.subscription_boundary.ok_or(
            AgentObservationProjectionError::Protocol(LocalControlErrorKind::MalformedFrame),
        )?;
        let observed = self.observed_revision.unwrap_or(boundary);
        if snapshot.snapshot_revision < boundary || snapshot.snapshot_revision < observed {
            self.invalidate_for_resubscribe(snapshot.snapshot_revision);
            return Err(AgentObservationProjectionError::StaleSnapshotRevision);
        }

        match self.hydration_revision {
            None => {
                if snapshot.page_offset != 0 {
                    return Err(AgentObservationProjectionError::Protocol(
                        LocalControlErrorKind::MalformedFrame,
                    ));
                }
                self.records.clear();
                self.hydration_revision = Some(snapshot.snapshot_revision);
                self.next_page_offset = 0;
            }
            Some(revision)
                if revision != snapshot.snapshot_revision
                    || snapshot.page_offset != self.next_page_offset =>
            {
                self.invalidate_for_resubscribe(snapshot.snapshot_revision);
                return Err(AgentObservationProjectionError::StaleSnapshotRevision);
            }
            Some(_) => {}
        }

        for observation in &snapshot.observations {
            if observation.owner_generation_id != self.owner_generation_id
                || self
                    .records
                    .insert(observation.observation_id, observation.clone())
                    .is_some()
            {
                self.invalidate_for_resubscribe(snapshot.snapshot_revision);
                return Err(AgentObservationProjectionError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ));
            }
        }

        let consumed = u16::try_from(snapshot.observations.len()).map_err(|_| {
            AgentObservationProjectionError::Protocol(LocalControlErrorKind::MalformedFrame)
        })?;
        let expected_next = snapshot.page_offset.checked_add(consumed).ok_or(
            AgentObservationProjectionError::Protocol(LocalControlErrorKind::MalformedFrame),
        )?;

        if let Some(cursor) = &snapshot.next_cursor {
            if cursor.owner_generation_id != self.owner_generation_id
                || cursor.snapshot_revision != snapshot.snapshot_revision
                || cursor.offset != expected_next
            {
                self.invalidate_for_resubscribe(snapshot.snapshot_revision);
                return Err(AgentObservationProjectionError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ));
            }
            self.next_page_offset = cursor.offset;
            self.observed_revision = Some(snapshot.snapshot_revision);
            self.freshness = AgentObservationProjectionFreshness::NeedsSnapshot;
            return Ok(snapshot.snapshot_revision);
        }

        self.hydration_revision = None;
        self.next_page_offset = 0;
        self.observed_revision = Some(snapshot.snapshot_revision);
        self.freshness = AgentObservationProjectionFreshness::Current;
        Ok(snapshot.snapshot_revision)
    }

    pub(crate) fn accept_event(
        &mut self,
        event: &AgentObservationEventV2,
    ) -> Result<(), AgentObservationProjectionError> {
        let subscription = self.subscription.as_ref().ok_or(
            AgentObservationProjectionError::Protocol(LocalControlErrorKind::UnsupportedOperation),
        )?;
        if self.freshness == AgentObservationProjectionFreshness::NeedsResubscribe {
            return Err(AgentObservationProjectionError::StaleSnapshotRevision);
        }

        let (workspace_id, revision) = match event {
            AgentObservationEventV2::Upsert {
                snapshot_revision,
                observation,
            } => (Some(observation.multiplexer_workspace_id), *snapshot_revision),
            AgentObservationEventV2::Removed {
                snapshot_revision,
                multiplexer_workspace_id,
                ..
            } => (Some(*multiplexer_workspace_id), *snapshot_revision),
            AgentObservationEventV2::HistoryGap {
                last_known_snapshot_revision,
            } => {
                self.invalidate_for_resubscribe(*last_known_snapshot_revision);
                return Ok(());
            }
        };
        if revision == 0 {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }
        if let (Some(expected), Some(actual)) =
            (subscription.multiplexer_workspace_id, workspace_id)
            && expected != actual
        {
            return Err(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }

        let previous = self
            .observed_revision
            .or(self.subscription_boundary)
            .ok_or(AgentObservationProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ))?;
        if revision < previous
            || (revision == previous
                && self.freshness != AgentObservationProjectionFreshness::NeedsSnapshot)
        {
            return Err(AgentObservationProjectionError::StaleSnapshotRevision);
        }

        if subscription.multiplexer_workspace_id.is_none() && revision > previous {
            let expected = previous.checked_add(1).ok_or(
                AgentObservationProjectionError::Protocol(LocalControlErrorKind::MalformedFrame),
            )?;
            if revision != expected {
                self.invalidate_for_resubscribe(previous);
                return Ok(());
            }
        }

        self.observed_revision = Some(revision);
        self.hydration_revision = None;
        self.next_page_offset = 0;
        self.records.clear();
        self.freshness = AgentObservationProjectionFreshness::NeedsSnapshot;
        Ok(())
    }

    fn invalidate_for_resubscribe(&mut self, last_known_revision: u64) {
        self.observed_revision = Some(last_known_revision);
        self.hydration_revision = None;
        self.next_page_offset = 0;
        self.records.clear();
        self.freshness = AgentObservationProjectionFreshness::NeedsResubscribe;
    }
}
