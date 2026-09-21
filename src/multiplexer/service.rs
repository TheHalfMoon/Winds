use super::navigation::MultiplexerTopology;
use super::persistence::{MULTIPLEXER_MAX_WORKSPACES, TopologySnapshotV1};
use super::{
    ClientSurfaceCapability, MultiplexerAuthority, MultiplexerErrorKind, TopologyGeneration,
};
use crate::persistent_runtime::domain::ClientConnectionId;
use crate::store::Store;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MultiplexerServiceError {
    Domain(MultiplexerErrorKind),
    Persistence(String),
}

impl From<MultiplexerErrorKind> for MultiplexerServiceError {
    fn from(value: MultiplexerErrorKind) -> Self {
        Self::Domain(value)
    }
}

pub(crate) struct MultiplexerService {
    topology: MultiplexerTopology,
    writers: BTreeSet<ClientConnectionId>,
}

impl MultiplexerService {
    pub(crate) fn restore(store: &Store) -> Result<Self, MultiplexerServiceError> {
        let snapshots = store
            .load_multiplexer_topology_snapshots()
            .map_err(|error| MultiplexerServiceError::Persistence(error.to_string()))?;
        Ok(Self {
            topology: restore_topology(snapshots)?,
            writers: BTreeSet::new(),
        })
    }

    pub(crate) fn topology(&self) -> &MultiplexerTopology {
        &self.topology
    }

    pub(crate) fn authority(&self, client: &ClientConnectionId) -> MultiplexerAuthority {
        if self.writers.contains(client) {
            MultiplexerAuthority::MultiplexerWrite
        } else {
            MultiplexerAuthority::Observer
        }
    }

    pub(crate) fn request_write(
        &mut self,
        client: ClientConnectionId,
        capability: ClientSurfaceCapability,
    ) -> Result<MultiplexerAuthority, MultiplexerErrorKind> {
        if capability == ClientSurfaceCapability::NonInteractiveObserver {
            return Err(MultiplexerErrorKind::CapabilityUnavailable);
        }
        self.writers.insert(client);
        Ok(MultiplexerAuthority::MultiplexerWrite)
    }

    pub(crate) fn release_write(&mut self, client: &ClientConnectionId) -> MultiplexerAuthority {
        self.writers.remove(client);
        MultiplexerAuthority::Observer
    }

    pub(crate) fn disconnect_client(&mut self, client: &ClientConnectionId) {
        self.writers.remove(client);
    }

    pub(crate) fn mutate<F>(
        &mut self,
        store: &mut Store,
        client: &ClientConnectionId,
        expected: TopologyGeneration,
        now_unix_ms: i64,
        mutation: F,
    ) -> Result<TopologyGeneration, MultiplexerServiceError>
    where
        F: FnOnce(
            &mut MultiplexerTopology,
            TopologyGeneration,
        ) -> Result<TopologyGeneration, MultiplexerErrorKind>,
    {
        if self.authority(client) != MultiplexerAuthority::MultiplexerWrite {
            return Err(MultiplexerErrorKind::MultiplexerWriteRequired.into());
        }
        if self.topology.generation() != expected {
            return Err(MultiplexerErrorKind::StaleTopologyGeneration.into());
        }

        let required = expected.checked_next()?;
        let mut candidate = self.topology.clone();
        let accepted = mutation(&mut candidate, expected)?;
        if accepted != required || candidate.generation() != required {
            return Err(MultiplexerErrorKind::OutcomeUnknown.into());
        }
        if candidate.workspaces().len() > MULTIPLEXER_MAX_WORKSPACES {
            return Err(MultiplexerErrorKind::SnapshotLimitExceeded.into());
        }

        let snapshots =
            snapshots_from_topology(&candidate).map_err(MultiplexerServiceError::Persistence)?;
        store
            .persist_multiplexer_topology_snapshots(&snapshots, now_unix_ms)
            .map_err(|error| MultiplexerServiceError::Persistence(error.to_string()))?;
        self.topology = candidate;
        Ok(accepted)
    }
}

fn snapshots_from_topology(
    topology: &MultiplexerTopology,
) -> Result<Vec<TopologySnapshotV1>, String> {
    topology
        .workspaces()
        .iter()
        .enumerate()
        .map(|(ordinal, workspace)| {
            TopologySnapshotV1::from_workspace_with_ordinal(
                workspace,
                topology.generation(),
                topology.focused_workspace_id() == Some(workspace.id),
                ordinal,
            )
        })
        .collect()
}

fn restore_topology(
    mut snapshots: Vec<TopologySnapshotV1>,
) -> Result<MultiplexerTopology, MultiplexerServiceError> {
    if snapshots.len() > MULTIPLEXER_MAX_WORKSPACES {
        return Err(MultiplexerServiceError::Persistence(
            "persisted multiplexer workspace count exceeds the accepted limit".to_owned(),
        ));
    }
    if snapshots.is_empty() {
        return Ok(MultiplexerTopology::empty());
    }

    let with_ordinal = snapshots
        .iter()
        .filter(|snapshot| snapshot.workspace_ordinal().is_some())
        .count();
    if with_ordinal != 0 && with_ordinal != snapshots.len() {
        return Err(MultiplexerServiceError::Persistence(
            "persisted topology mixes ordinal-aware and legacy workspace snapshots".to_owned(),
        ));
    }
    if with_ordinal == snapshots.len() {
        snapshots.sort_by_key(|snapshot| snapshot.workspace_ordinal());
        for (expected, snapshot) in snapshots.iter().enumerate() {
            if snapshot.workspace_ordinal().map(usize::from) != Some(expected) {
                return Err(MultiplexerServiceError::Persistence(
                    "persisted workspace ordinals must be unique and contiguous".to_owned(),
                ));
            }
        }
    }

    let generation = snapshots[0].topology_generation();
    let mut focused = None;
    let mut workspaces = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        if snapshot.topology_generation() != generation {
            return Err(MultiplexerServiceError::Persistence(
                "persisted topology mixes topology generations".to_owned(),
            ));
        }
        if snapshot.is_focused() && focused.replace(snapshot.workspace_id()).is_some() {
            return Err(MultiplexerServiceError::Persistence(
                "persisted topology has multiple focused workspaces".to_owned(),
            ));
        }
        workspaces.push(
            snapshot
                .to_workspace_state()
                .map_err(MultiplexerServiceError::Persistence)?,
        );
    }

    MultiplexerTopology::restore_presentation(generation, workspaces, focused).map_err(|error| {
        MultiplexerServiceError::Persistence(format!(
            "persisted topology violates presentation invariants: {error:?}"
        ))
    })
}
