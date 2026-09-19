use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, ControllerLeaseIdentity, OwnerGenerationId,
    RuntimeNamespaceId,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub(crate) const CONTROLLER_HEARTBEAT_TARGET_MS: u64 = 10_000;
pub(crate) const CONTROLLER_LEASE_EXPIRY_MS: u64 = 30_000;

pub(crate) type ControllerResult<T> = Result<T, ControllerError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControllerError {
    UnknownRuntime,
    ControllerConflict {
        active_controller: ClientConnectionId,
    },
    NotActiveController,
    LeaseExpired,
    MonotonicTimeRegression,
}

impl fmt::Display for ControllerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRuntime => formatter.write_str("controller runtime is unknown"),
            Self::ControllerConflict { active_controller } => write!(
                formatter,
                "controller lease is already held by {}",
                active_controller.as_str()
            ),
            Self::NotActiveController => {
                formatter.write_str("client does not hold the active controller lease")
            }
            Self::LeaseExpired => formatter.write_str("controller lease has expired"),
            Self::MonotonicTimeRegression => {
                formatter.write_str("controller monotonic clock regressed")
            }
        }
    }
}

impl Error for ControllerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControllerDisposition {
    Granted,
    Released,
    Disconnected,
    Expired,
    OwnerRevoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControllerTransition {
    pub(crate) identity: ControllerLeaseIdentity,
    pub(crate) disposition: ControllerDisposition,
    pub(crate) observed_monotonic_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControllerLeaseSnapshot {
    pub(crate) identity: ControllerLeaseIdentity,
    pub(crate) acquired_monotonic_ms: u64,
    pub(crate) last_heartbeat_monotonic_ms: u64,
    pub(crate) expires_at_monotonic_ms: u64,
}

impl ControllerLeaseSnapshot {
    pub(crate) fn heartbeat_due(&self, now_monotonic_ms: u64) -> ControllerResult<bool> {
        if now_monotonic_ms < self.last_heartbeat_monotonic_ms {
            return Err(ControllerError::MonotonicTimeRegression);
        }
        Ok(
            now_monotonic_ms.saturating_sub(self.last_heartbeat_monotonic_ms)
                >= CONTROLLER_HEARTBEAT_TARGET_MS,
        )
    }

    fn is_expired(&self, now_monotonic_ms: u64) -> ControllerResult<bool> {
        if now_monotonic_ms < self.last_heartbeat_monotonic_ms {
            return Err(ControllerError::MonotonicTimeRegression);
        }
        Ok(now_monotonic_ms >= self.expires_at_monotonic_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ControllerStateSnapshot {
    pub(crate) authority: ClientAuthority,
    pub(crate) controller_client_id: Option<ClientConnectionId>,
    pub(crate) lease: Option<ControllerLeaseSnapshot>,
}

#[derive(Debug, Default)]
struct RuntimeControllerState {
    lease: Option<ControllerLeaseSnapshot>,
}

pub(crate) struct ControllerRegistry {
    owner_generation_id: OwnerGenerationId,
    runtimes: BTreeMap<RuntimeNamespaceId, RuntimeControllerState>,
}

impl ControllerRegistry {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            runtimes: BTreeMap::new(),
        }
    }

    pub(crate) fn register_runtime(&mut self, runtime_namespace_id: RuntimeNamespaceId) {
        self.runtimes.entry(runtime_namespace_id).or_default();
    }

    pub(crate) fn unregister_runtime(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Option<ControllerTransition>> {
        let state = self
            .runtimes
            .remove(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        Ok(state.lease.map(|lease| ControllerTransition {
            identity: lease.identity,
            disposition: ControllerDisposition::OwnerRevoked,
            observed_monotonic_ms: now_monotonic_ms,
        }))
    }

    pub(crate) fn request_control(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        client_connection_id: ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Option<ControllerTransition>> {
        let state = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;

        if let Some(active) = &state.lease {
            if active.is_expired(now_monotonic_ms)? {
                return Err(ControllerError::LeaseExpired);
            }
            if active.identity.controller_client_id == client_connection_id {
                return Ok(None);
            }
            return Err(ControllerError::ControllerConflict {
                active_controller: active.identity.controller_client_id.clone(),
            });
        }

        let identity = ControllerLeaseIdentity {
            owner_generation_id: self.owner_generation_id,
            runtime_namespace_id,
            controller_client_id: client_connection_id,
        };
        state.lease = Some(ControllerLeaseSnapshot {
            identity: identity.clone(),
            acquired_monotonic_ms: now_monotonic_ms,
            last_heartbeat_monotonic_ms: now_monotonic_ms,
            expires_at_monotonic_ms: now_monotonic_ms.saturating_add(CONTROLLER_LEASE_EXPIRY_MS),
        });
        Ok(Some(ControllerTransition {
            identity,
            disposition: ControllerDisposition::Granted,
            observed_monotonic_ms: now_monotonic_ms,
        }))
    }

    pub(crate) fn renew_control(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        client_connection_id: &ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<ControllerLeaseSnapshot> {
        let state = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        let lease = state
            .lease
            .as_mut()
            .ok_or(ControllerError::NotActiveController)?;
        if lease.is_expired(now_monotonic_ms)? {
            return Err(ControllerError::LeaseExpired);
        }
        if &lease.identity.controller_client_id != client_connection_id {
            return Err(ControllerError::NotActiveController);
        }
        lease.last_heartbeat_monotonic_ms = now_monotonic_ms;
        lease.expires_at_monotonic_ms = now_monotonic_ms.saturating_add(CONTROLLER_LEASE_EXPIRY_MS);
        Ok(lease.clone())
    }

    pub(crate) fn release_control(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        client_connection_id: &ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<ControllerTransition> {
        let state = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        let lease = state
            .lease
            .as_ref()
            .ok_or(ControllerError::NotActiveController)?;
        if lease.is_expired(now_monotonic_ms)? {
            return Err(ControllerError::LeaseExpired);
        }
        if &lease.identity.controller_client_id != client_connection_id {
            return Err(ControllerError::NotActiveController);
        }
        let identity = state
            .lease
            .take()
            .expect("active controller lease was validated")
            .identity;
        Ok(ControllerTransition {
            identity,
            disposition: ControllerDisposition::Released,
            observed_monotonic_ms: now_monotonic_ms,
        })
    }

    pub(crate) fn owner_revoke(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Option<ControllerTransition>> {
        let state = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        Ok(state.lease.take().map(|lease| ControllerTransition {
            identity: lease.identity,
            disposition: ControllerDisposition::OwnerRevoked,
            observed_monotonic_ms: now_monotonic_ms,
        }))
    }

    pub(crate) fn disconnect_client(
        &mut self,
        client_connection_id: &ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> Vec<ControllerTransition> {
        let mut transitions = Vec::new();
        for state in self.runtimes.values_mut() {
            let matches = state
                .lease
                .as_ref()
                .is_some_and(|lease| &lease.identity.controller_client_id == client_connection_id);
            if matches {
                let identity = state
                    .lease
                    .take()
                    .expect("matching controller lease was checked")
                    .identity;
                transitions.push(ControllerTransition {
                    identity,
                    disposition: ControllerDisposition::Disconnected,
                    observed_monotonic_ms: now_monotonic_ms,
                });
            }
        }
        transitions
    }

    pub(crate) fn expire_leases(
        &mut self,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Vec<ControllerTransition>> {
        let mut transitions = Vec::new();
        for state in self.runtimes.values_mut() {
            let expired = match state.lease.as_ref() {
                Some(lease) => lease.is_expired(now_monotonic_ms)?,
                None => false,
            };
            if expired {
                let identity = state
                    .lease
                    .take()
                    .expect("expired controller lease was checked")
                    .identity;
                transitions.push(ControllerTransition {
                    identity,
                    disposition: ControllerDisposition::Expired,
                    observed_monotonic_ms: now_monotonic_ms,
                });
            }
        }
        Ok(transitions)
    }

    pub(crate) fn authorize_mutation(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
        client_connection_id: &ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<ControllerLeaseIdentity> {
        let state = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        let lease = state
            .lease
            .as_ref()
            .ok_or(ControllerError::NotActiveController)?;
        if lease.is_expired(now_monotonic_ms)? {
            return Err(ControllerError::LeaseExpired);
        }
        if &lease.identity.controller_client_id != client_connection_id {
            return Err(ControllerError::NotActiveController);
        }
        Ok(lease.identity.clone())
    }

    pub(crate) fn control_state(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
        client_connection_id: &ClientConnectionId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<ControllerStateSnapshot> {
        let state = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        let lease = match state.lease.as_ref() {
            Some(lease) if lease.is_expired(now_monotonic_ms)? => None,
            Some(lease) => Some(lease.clone()),
            None => None,
        };
        let controller_client_id = lease
            .as_ref()
            .map(|lease| lease.identity.controller_client_id.clone());
        let authority = if controller_client_id.as_ref() == Some(client_connection_id) {
            ClientAuthority::Controller
        } else {
            ClientAuthority::Observer
        };
        Ok(ControllerStateSnapshot {
            authority,
            controller_client_id,
            lease,
        })
    }

    pub(crate) fn active_runtime_ids(
        &self,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Vec<RuntimeNamespaceId>> {
        let mut runtime_ids = Vec::new();
        for (runtime_namespace_id, state) in &self.runtimes {
            if let Some(lease) = &state.lease
                && !lease.is_expired(now_monotonic_ms)?
            {
                runtime_ids.push(*runtime_namespace_id);
            }
        }
        Ok(runtime_ids)
    }

    pub(crate) fn active_lease(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
        now_monotonic_ms: u64,
    ) -> ControllerResult<Option<ControllerLeaseSnapshot>> {
        let state = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(ControllerError::UnknownRuntime)?;
        match state.lease.as_ref() {
            Some(lease) if lease.is_expired(now_monotonic_ms)? => Ok(None),
            Some(lease) => Ok(Some(lease.clone())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[path = "../t154_controller_lease_tests.rs"]
mod t154_controller_lease_tests;
