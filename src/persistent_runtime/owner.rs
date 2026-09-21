use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::TerminalSize;
use crate::multiplexer::domain::navigation::MultiplexerTopology;
use crate::multiplexer::domain::service::{MultiplexerService, MultiplexerServiceError};
use crate::multiplexer::domain::{
    ClientSurfaceCapability, MultiplexerAuthority, MultiplexerErrorKind, TopologyGeneration,
};
use crate::persistent_runtime::controller::{
    ControllerDisposition, ControllerRegistry, ControllerStateSnapshot, ControllerTransition,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, OwnerGenerationId, RuntimeAlias, RuntimeNamespaceId,
};
use crate::persistent_runtime::protocol::{ProtocolMessage, validate_candidate_topology_v2};
use crate::persistent_runtime::replay::ObserverHandle;
use crate::persistent_runtime::runtime::{
    PersistentRuntimeShutdownReport, PersistentTerminalAttachment, PersistentTerminalRegistry,
    PersistentTerminalSnapshot,
};
use crate::store::Store;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub(crate) const INTERNAL_OWNER_COMMAND: &str = "__winds-internal-owner-v1";
pub(crate) const OWNER_IDLE_GRACE_MS: u64 = 300_000;
const OWNER_POLL_INTERVAL_MS: u64 = 250;

pub(crate) type OwnerResult<T> = Result<T, OwnerError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnerError {
    HomeMustBeAbsolute,
    HomeUnavailable,
    HomeNotCanonical,
    SingletonAlreadyActive,
    SingletonSecurityMismatch,
    SingletonIo(String),
    Entropy(String),
    Store(String),
    Endpoint(String),
    ClockUnavailable,
    ActivityUnderflow,
    Runtime(String),
}

impl fmt::Display for OwnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HomeMustBeAbsolute => {
                formatter.write_str("persistent owner home must be absolute")
            }
            Self::HomeUnavailable => formatter.write_str("persistent owner home is unavailable"),
            Self::HomeNotCanonical => {
                formatter.write_str("persistent owner home could not be canonicalized")
            }
            Self::SingletonAlreadyActive => {
                formatter.write_str("persistent owner singleton is already active")
            }
            Self::SingletonSecurityMismatch => formatter.write_str(
                "persistent owner singleton security facts do not match the accepted principal",
            ),
            Self::SingletonIo(message) => write!(
                formatter,
                "persistent owner singleton I/O failed: {message}"
            ),
            Self::Entropy(message) => write!(
                formatter,
                "persistent owner generation entropy failed: {message}"
            ),
            Self::Store(message) => write!(
                formatter,
                "persistent owner store startup failed: {message}"
            ),
            Self::Endpoint(message) => write!(
                formatter,
                "persistent owner endpoint startup failed: {message}"
            ),
            Self::ClockUnavailable => formatter.write_str("persistent owner clock is unavailable"),
            Self::ActivityUnderflow => {
                formatter.write_str("persistent owner client activity underflow")
            }
            Self::Runtime(message) => {
                write!(formatter, "persistent owner runtime failed: {message}")
            }
        }
    }
}

impl Error for OwnerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerStartupPhase {
    SingletonAcquired,
    GenerationCreated,
    Reconciled,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OwnerActivity {
    connected_clients: usize,
    live_runtimes: usize,
    idle_since_monotonic_ms: Option<u64>,
}

impl OwnerActivity {
    pub(crate) fn new(now_monotonic_ms: u64) -> Self {
        Self {
            connected_clients: 0,
            live_runtimes: 0,
            idle_since_monotonic_ms: Some(now_monotonic_ms),
        }
    }

    pub(crate) fn client_connected(&mut self) {
        self.connected_clients = self.connected_clients.saturating_add(1);
        self.idle_since_monotonic_ms = None;
    }

    pub(crate) fn client_disconnected(&mut self, now_monotonic_ms: u64) -> OwnerResult<()> {
        if self.connected_clients == 0 {
            return Err(OwnerError::ActivityUnderflow);
        }
        self.connected_clients -= 1;
        self.refresh_idle_start(now_monotonic_ms);
        Ok(())
    }

    pub(crate) fn set_live_runtime_count(&mut self, count: usize, now_monotonic_ms: u64) {
        self.live_runtimes = count;
        if count > 0 {
            self.idle_since_monotonic_ms = None;
        } else {
            self.refresh_idle_start(now_monotonic_ms);
        }
    }

    pub(crate) fn should_exit(&self, now_monotonic_ms: u64) -> bool {
        if self.connected_clients != 0 || self.live_runtimes != 0 {
            return false;
        }
        self.idle_since_monotonic_ms.is_some_and(|idle_since| {
            now_monotonic_ms.saturating_sub(idle_since) >= OWNER_IDLE_GRACE_MS
        })
    }

    fn refresh_idle_start(&mut self, now_monotonic_ms: u64) {
        if self.connected_clients == 0 && self.live_runtimes == 0 {
            self.idle_since_monotonic_ms.get_or_insert(now_monotonic_ms);
        }
    }
}

pub(crate) struct PersistentOwner {
    _singleton: OwnerSingleton,
    generation_id: OwnerGenerationId,
    store: Store,
    _endpoint: OwnerEndpoint,
    activity: OwnerActivity,
    reconciled_runtime_count: usize,
    ready_unix_ms: i64,
    startup_phase: OwnerStartupPhase,
    runtime_registry: PersistentTerminalRegistry,
    controller_registry: ControllerRegistry,
    multiplexer_service: MultiplexerService,
}

impl PersistentOwner {
    pub(crate) fn start(home: &Path, now_unix_ms: i64) -> OwnerResult<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Self::start_with_posix_endpoint(home, now_unix_ms, None)
        }
        #[cfg(windows)]
        {
            Self::start_with_windows_endpoint(home, now_unix_ms)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        {
            let _ = (home, now_unix_ms);
            Err(OwnerError::Endpoint(
                "unsupported owner platform".to_owned(),
            ))
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn start_with_posix_endpoint(
        home: &Path,
        now_unix_ms: i64,
        runtime_directory: Option<&Path>,
    ) -> OwnerResult<Self> {
        use crate::persistent_runtime::transport::unix::{
            BoundUnixListener, generate_owner_generation_id,
        };

        let canonical_home = prepare_owner_home(home)?;
        let singleton = OwnerSingleton::acquire(&canonical_home)?;
        let _phase = OwnerStartupPhase::SingletonAcquired;
        let generation_id = generate_owner_generation_id()
            .map_err(|error| OwnerError::Entropy(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::GenerationCreated;
        let (store, reconciled_runtime_count) =
            reconcile_store(&canonical_home, generation_id, now_unix_ms)?;
        let multiplexer_service = MultiplexerService::restore(&store)
            .map_err(|error| OwnerError::Store(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::Reconciled;
        let listener = match runtime_directory {
            Some(path) => BoundUnixListener::bind(path),
            None => BoundUnixListener::bind_default(),
        }
        .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;

        Ok(Self {
            _singleton: singleton,
            generation_id,
            store,
            _endpoint: OwnerEndpoint::Posix(listener),
            activity: OwnerActivity::new(0),
            reconciled_runtime_count,
            ready_unix_ms: now_unix_ms,
            startup_phase: OwnerStartupPhase::Ready,
            runtime_registry: PersistentTerminalRegistry::new(generation_id),
            controller_registry: ControllerRegistry::new(generation_id),
            multiplexer_service,
        })
    }

    #[cfg(windows)]
    fn start_with_windows_endpoint(home: &Path, now_unix_ms: i64) -> OwnerResult<Self> {
        use crate::persistent_runtime::transport::windows::{
            WindowsNamedPipeServer, generate_owner_generation_id,
        };

        let canonical_home = prepare_owner_home(home)?;
        let singleton = OwnerSingleton::acquire(&canonical_home)?;
        let _phase = OwnerStartupPhase::SingletonAcquired;
        let generation_id = generate_owner_generation_id()
            .map_err(|error| OwnerError::Entropy(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::GenerationCreated;
        let (store, reconciled_runtime_count) =
            reconcile_store(&canonical_home, generation_id, now_unix_ms)?;
        let multiplexer_service = MultiplexerService::restore(&store)
            .map_err(|error| OwnerError::Store(format!("{error:?}")))?;
        let _phase = OwnerStartupPhase::Reconciled;
        let server = WindowsNamedPipeServer::bind(generation_id)
            .map_err(|error| OwnerError::Endpoint(format!("{error:?}")))?;

        Ok(Self {
            _singleton: singleton,
            generation_id,
            store,
            _endpoint: OwnerEndpoint::Windows(server),
            activity: OwnerActivity::new(0),
            reconciled_runtime_count,
            ready_unix_ms: now_unix_ms,
            startup_phase: OwnerStartupPhase::Ready,
            runtime_registry: PersistentTerminalRegistry::new(generation_id),
            controller_registry: ControllerRegistry::new(generation_id),
            multiplexer_service,
        })
    }

    #[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
    pub(crate) fn start_for_test(
        home: &Path,
        runtime_directory: &Path,
        now_unix_ms: i64,
    ) -> OwnerResult<Self> {
        Self::start_with_posix_endpoint(home, now_unix_ms, Some(runtime_directory))
    }

    pub(crate) fn generation_id(&self) -> OwnerGenerationId {
        self.generation_id
    }

    pub(crate) fn is_ready(&self) -> bool {
        self.startup_phase == OwnerStartupPhase::Ready
    }

    pub(crate) fn reconciled_runtime_count(&self) -> usize {
        self.reconciled_runtime_count
    }

    pub(crate) fn multiplexer_topology(&self) -> &MultiplexerTopology {
        self.multiplexer_service.topology()
    }

    pub(crate) fn multiplexer_authority(
        &self,
        client_connection_id: &ClientConnectionId,
    ) -> MultiplexerAuthority {
        self.multiplexer_service.authority(client_connection_id)
    }

    pub(crate) fn request_multiplexer_write(
        &mut self,
        client_connection_id: ClientConnectionId,
        capability: ClientSurfaceCapability,
    ) -> Result<MultiplexerAuthority, MultiplexerErrorKind> {
        self.multiplexer_service
            .request_write(client_connection_id, capability)
    }

    pub(crate) fn release_multiplexer_write(
        &mut self,
        client_connection_id: &ClientConnectionId,
    ) -> MultiplexerAuthority {
        self.multiplexer_service.release_write(client_connection_id)
    }

    pub(crate) fn disconnect_multiplexer_client(
        &mut self,
        client_connection_id: &ClientConnectionId,
    ) {
        self.multiplexer_service
            .disconnect_client(client_connection_id);
    }

    pub(crate) fn mutate_multiplexer_topology<F>(
        &mut self,
        client_connection_id: &ClientConnectionId,
        expected_generation: TopologyGeneration,
        now_unix_ms: i64,
        mutation: F,
    ) -> Result<TopologyGeneration, MultiplexerServiceError>
    where
        F: FnOnce(
            &mut MultiplexerTopology,
            TopologyGeneration,
        ) -> Result<TopologyGeneration, MultiplexerErrorKind>,
    {
        let owner_generation_id = self.generation_id;
        self.multiplexer_service.mutate(
            &mut self.store,
            client_connection_id,
            expected_generation,
            now_unix_ms,
            |candidate, expected| {
                let accepted = mutation(candidate, expected)?;
                validate_candidate_topology_v2(candidate, owner_generation_id)?;
                Ok(accepted)
            },
        )
    }

    pub(crate) fn ready_unix_ms(&self) -> i64 {
        self.ready_unix_ms
    }

    pub(crate) fn activity_mut(&mut self) -> &mut OwnerActivity {
        &mut self.activity
    }

    pub(crate) fn start_terminal_runtime(
        &mut self,
        runtime_alias: RuntimeAlias,
        profile: &ShellProfile,
        cwd: &Path,
        terminal_size: TerminalSize,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalAttachment> {
        let attachment = self
            .runtime_registry
            .start_shell(
                &self.store,
                runtime_alias,
                profile,
                cwd,
                terminal_size,
                now_unix_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.controller_registry
            .register_runtime(attachment.runtime_namespace_id());
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(attachment)
    }

    pub(crate) fn reattach_terminal_runtime(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        expected_owner_generation_id: OwnerGenerationId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalAttachment> {
        let attachment = self
            .runtime_registry
            .reattach(
                &self.store,
                runtime_namespace_id,
                expected_owner_generation_id,
                now_unix_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(attachment)
    }

    pub(crate) fn terminal_runtime_snapshot(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let snapshot = self
            .runtime_registry
            .snapshot(&self.store, attachment, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(snapshot)
    }

    pub(crate) fn send_terminal_runtime_input(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        bytes: &[u8],
    ) -> OwnerResult<()> {
        self.runtime_registry
            .send_input(attachment, bytes)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn read_terminal_runtime_output(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        buffer: &mut [u8],
    ) -> OwnerResult<usize> {
        self.runtime_registry
            .read_output(attachment, buffer)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn resize_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        terminal_size: TerminalSize,
    ) -> OwnerResult<()> {
        self.runtime_registry
            .resize(attachment, terminal_size)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn terminal_runtime_size(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> OwnerResult<TerminalSize> {
        self.runtime_registry
            .current_size(attachment)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn interrupt_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> OwnerResult<()> {
        self.runtime_registry
            .interrupt(attachment)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn terminate_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let runtime_result = self
            .runtime_registry
            .terminate(&self.store, attachment, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()));
        let controller_result = self.revoke_controller_for_runtime(
            attachment.runtime_namespace_id(),
            now_unix_ms,
            now_monotonic_ms,
        );
        self.sync_runtime_activity(now_monotonic_ms);
        combine_runtime_and_controller_result(runtime_result, controller_result)
    }

    pub(crate) fn close_terminal_runtime(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        let runtime_result = self
            .runtime_registry
            .close(&self.store, attachment, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()));
        let controller_result = self.revoke_controller_for_runtime(
            attachment.runtime_namespace_id(),
            now_unix_ms,
            now_monotonic_ms,
        );
        self.sync_runtime_activity(now_monotonic_ms);
        combine_runtime_and_controller_result(runtime_result, controller_result)
    }

    pub(crate) fn poll_terminal_runtimes(
        &mut self,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<usize> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        let observed = self
            .runtime_registry
            .poll_exits(&self.store, now_unix_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.revoke_dead_runtime_controllers(now_unix_ms, now_monotonic_ms)?;
        self.sync_runtime_activity(now_monotonic_ms);
        Ok(observed)
    }

    pub(crate) fn live_terminal_runtime_count(&self) -> usize {
        self.runtime_registry.live_count()
    }

    pub(crate) fn request_terminal_control(
        &mut self,
        client_connection_id: ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<ControllerStateSnapshot> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        if !self.runtime_registry.is_live_runtime(runtime_namespace_id) {
            return Err(OwnerError::Runtime(
                "controller request requires a live owned runtime".to_owned(),
            ));
        }
        if let Some(transition) = self
            .controller_registry
            .request_control(
                runtime_namespace_id,
                client_connection_id.clone(),
                now_monotonic_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))?
        {
            self.record_controller_transition(&transition, now_unix_ms)?;
        }
        self.controller_registry
            .control_state(
                runtime_namespace_id,
                &client_connection_id,
                now_monotonic_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn renew_terminal_control(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<ControllerStateSnapshot> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        self.controller_registry
            .renew_control(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.controller_registry
            .control_state(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn release_terminal_control(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<ControllerStateSnapshot> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        let transition = self
            .controller_registry
            .release_control(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.record_controller_transition(&transition, now_unix_ms)?;
        self.controller_registry
            .control_state(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn terminal_control_state(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<ControllerStateSnapshot> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        self.controller_registry
            .control_state(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn disconnect_terminal_controller(
        &mut self,
        client_connection_id: &ClientConnectionId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<usize> {
        let transitions = self
            .controller_registry
            .disconnect_client(client_connection_id, now_monotonic_ms);
        for transition in &transitions {
            self.record_controller_transition(transition, now_unix_ms)?;
        }
        Ok(transitions.len())
    }

    pub(crate) fn controller_send_terminal_input(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        bytes: &[u8],
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        self.authorize_controller_mutation(
            client_connection_id,
            runtime_namespace_id,
            now_unix_ms,
            now_monotonic_ms,
        )?;
        let attachment = self
            .runtime_registry
            .attachment_for_runtime(runtime_namespace_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.runtime_registry
            .send_input(&attachment, bytes)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn controller_resize_terminal(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        terminal_size: TerminalSize,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        self.authorize_controller_mutation(
            client_connection_id,
            runtime_namespace_id,
            now_unix_ms,
            now_monotonic_ms,
        )?;
        let attachment = self
            .runtime_registry
            .attachment_for_runtime(runtime_namespace_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.runtime_registry
            .resize(&attachment, terminal_size)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn controller_interrupt_terminal(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        self.authorize_controller_mutation(
            client_connection_id,
            runtime_namespace_id,
            now_unix_ms,
            now_monotonic_ms,
        )?;
        let attachment = self
            .runtime_registry
            .attachment_for_runtime(runtime_namespace_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.runtime_registry
            .interrupt(&attachment)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn controller_stop_terminal(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<PersistentTerminalSnapshot> {
        self.authorize_controller_mutation(
            client_connection_id,
            runtime_namespace_id,
            now_unix_ms,
            now_monotonic_ms,
        )?;
        let attachment = self
            .runtime_registry
            .attachment_for_runtime(runtime_namespace_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        self.terminate_terminal_runtime(&attachment, now_unix_ms, now_monotonic_ms)
    }

    pub(crate) fn attach_terminal_observer(
        &mut self,
        connection_id: ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> OwnerResult<ObserverHandle> {
        self.runtime_registry
            .attach_observer(connection_id, runtime_namespace_id)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn fill_terminal_observer_queue(
        &mut self,
        handle: &ObserverHandle,
    ) -> OwnerResult<usize> {
        self.runtime_registry
            .fill_observer_queue(handle)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn drain_terminal_observer(
        &mut self,
        handle: &ObserverHandle,
    ) -> OwnerResult<Vec<ProtocolMessage>> {
        self.runtime_registry
            .drain_observer(handle)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn detach_terminal_observer(&mut self, handle: &ObserverHandle) -> OwnerResult<()> {
        self.runtime_registry
            .detach_observer(handle)
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    fn authorize_controller_mutation(
        &mut self,
        client_connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        self.reap_expired_controller_leases(now_unix_ms, now_monotonic_ms)?;
        self.controller_registry
            .authorize_mutation(runtime_namespace_id, client_connection_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        Ok(())
    }

    fn reap_expired_controller_leases(
        &mut self,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<usize> {
        let transitions = self
            .controller_registry
            .expire_leases(now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        for transition in &transitions {
            self.record_controller_transition(transition, now_unix_ms)?;
        }
        Ok(transitions.len())
    }

    fn revoke_dead_runtime_controllers(
        &mut self,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        let runtime_ids = self
            .controller_registry
            .active_runtime_ids(now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?;
        for runtime_namespace_id in runtime_ids {
            if !self.runtime_registry.is_live_runtime(runtime_namespace_id) {
                self.revoke_controller_for_runtime(
                    runtime_namespace_id,
                    now_unix_ms,
                    now_monotonic_ms,
                )?;
            }
        }
        Ok(())
    }

    fn revoke_controller_for_runtime(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> OwnerResult<()> {
        if let Some(transition) = self
            .controller_registry
            .owner_revoke(runtime_namespace_id, now_monotonic_ms)
            .map_err(|error| OwnerError::Runtime(error.to_string()))?
        {
            self.record_controller_transition(&transition, now_unix_ms)?;
        }
        Ok(())
    }

    fn record_controller_transition(
        &mut self,
        transition: &ControllerTransition,
        now_unix_ms: i64,
    ) -> OwnerResult<()> {
        let controller_client_id = match transition.disposition {
            ControllerDisposition::Granted => {
                Some(transition.identity.controller_client_id.clone())
            }
            ControllerDisposition::Released
            | ControllerDisposition::Disconnected
            | ControllerDisposition::Expired
            | ControllerDisposition::OwnerRevoked => None,
        };
        self.runtime_registry
            .record_controller_changed(
                transition.identity.runtime_namespace_id,
                controller_client_id,
                now_unix_ms,
            )
            .map_err(|error| OwnerError::Runtime(error.to_string()))
    }

    pub(crate) fn shutdown_terminal_runtimes(
        &mut self,
        now_unix_ms: i64,
        now_monotonic_ms: u64,
    ) -> Vec<PersistentRuntimeShutdownReport> {
        let reports = self.runtime_registry.shutdown_all(&self.store, now_unix_ms);
        self.sync_runtime_activity(now_monotonic_ms);
        reports
    }

    fn sync_runtime_activity(&mut self, now_monotonic_ms: u64) {
        self.activity
            .set_live_runtime_count(self.runtime_registry.live_count(), now_monotonic_ms);
    }

    pub(crate) fn should_exit(&self, now_monotonic_ms: u64) -> bool {
        self.activity.should_exit(now_monotonic_ms)
    }
}

fn combine_runtime_and_controller_result<T>(
    runtime_result: OwnerResult<T>,
    controller_result: OwnerResult<()>,
) -> OwnerResult<T> {
    match (runtime_result, controller_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(runtime_error), Ok(())) => Err(runtime_error),
        (Ok(_), Err(controller_error)) => Err(controller_error),
        (Err(runtime_error), Err(controller_error)) => Err(OwnerError::Runtime(format!(
            "{runtime_error}; controller revocation also failed: {controller_error}"
        ))),
    }
}

fn reconcile_store(
    canonical_home: &Path,
    generation_id: OwnerGenerationId,
    now_unix_ms: i64,
) -> OwnerResult<(Store, usize)> {
    let store =
        Store::open(canonical_home).map_err(|error| OwnerError::Store(error.to_string()))?;
    store
        .record_persistent_runtime_owner_generation(generation_id, now_unix_ms)
        .map_err(|error| OwnerError::Store(error.to_string()))?;
    let reconciled = store
        .reconcile_persistent_runtime_records(Some(generation_id), now_unix_ms)
        .map_err(|error| OwnerError::Store(error.to_string()))?;
    Ok((store, reconciled))
}

fn prepare_owner_home(home: &Path) -> OwnerResult<PathBuf> {
    if !home.is_absolute() {
        return Err(OwnerError::HomeMustBeAbsolute);
    }
    fs::create_dir_all(home).map_err(|_| OwnerError::HomeUnavailable)?;
    let canonical = home
        .canonicalize()
        .map_err(|_| OwnerError::HomeNotCanonical)?;
    if !canonical.is_absolute() {
        return Err(OwnerError::HomeNotCanonical);
    }
    Ok(canonical)
}

pub(crate) fn run_internal_owner(home: &Path) -> OwnerResult<()> {
    let mut owner = PersistentOwner::start(home, system_unix_ms()?)?;
    let monotonic_origin = Instant::now();
    loop {
        let now_monotonic_ms = monotonic_elapsed_ms(monotonic_origin);
        owner.poll_terminal_runtimes(system_unix_ms()?, now_monotonic_ms)?;
        if owner.should_exit(now_monotonic_ms) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(OWNER_POLL_INTERVAL_MS));
    }
}

fn monotonic_elapsed_ms(origin: Instant) -> u64 {
    u64::try_from(origin.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn system_unix_ms() -> OwnerResult<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OwnerError::ClockUnavailable)?;
    i64::try_from(elapsed.as_millis()).map_err(|_| OwnerError::ClockUnavailable)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
enum OwnerEndpoint {
    Posix(crate::persistent_runtime::transport::unix::BoundUnixListener),
}

#[cfg(windows)]
enum OwnerEndpoint {
    Windows(crate::persistent_runtime::transport::windows::WindowsNamedPipeServer),
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
enum OwnerEndpoint {}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct OwnerSingleton {
    _file: std::fs::File,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl OwnerSingleton {
    fn acquire(home: &Path) -> OwnerResult<Self> {
        use crate::persistent_runtime::peer::current_effective_uid;
        use std::fs::{DirBuilder, OpenOptions, Permissions};
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};

        const DIRECTORY_MODE: u32 = 0o700;
        const LOCK_MODE: u32 = 0o600;
        let directory = home.join("persistent-runtime");
        match fs::symlink_metadata(&directory) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink()
                    || !metadata.is_dir()
                    || metadata.uid() != current_effective_uid()
                {
                    return Err(OwnerError::SingletonSecurityMismatch);
                }
                if metadata.mode() & 0o777 != DIRECTORY_MODE {
                    fs::set_permissions(&directory, Permissions::from_mode(DIRECTORY_MODE))
                        .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let mut builder = DirBuilder::new();
                builder.mode(DIRECTORY_MODE);
                match builder.create(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(OwnerError::SingletonIo(error.to_string())),
                }
                let metadata = fs::symlink_metadata(&directory)
                    .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
                if metadata.file_type().is_symlink()
                    || !metadata.is_dir()
                    || metadata.uid() != current_effective_uid()
                    || metadata.mode() & 0o777 != DIRECTORY_MODE
                {
                    return Err(OwnerError::SingletonSecurityMismatch);
                }
            }
            Err(error) => return Err(OwnerError::SingletonIo(error.to_string())),
        }

        let lock_path = directory.join("owner.lock");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(LOCK_MODE)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&lock_path)
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        let file_metadata = file
            .metadata()
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        let path_metadata = fs::symlink_metadata(&lock_path)
            .map_err(|error| OwnerError::SingletonIo(error.to_string()))?;
        if !file_metadata.is_file()
            || path_metadata.file_type().is_symlink()
            || !path_metadata.is_file()
            || file_metadata.uid() != current_effective_uid()
            || file_metadata.mode() & 0o777 != LOCK_MODE
            || file_metadata.dev() != path_metadata.dev()
            || file_metadata.ino() != path_metadata.ino()
        {
            return Err(OwnerError::SingletonSecurityMismatch);
        }

        // SAFETY: file owns a valid descriptor for the private regular lock file. The kernel
        // releases this advisory lock automatically when the File is dropped.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            let error = std::io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN
            ) {
                return Err(OwnerError::SingletonAlreadyActive);
            }
            return Err(OwnerError::SingletonIo(error.to_string()));
        }

        Ok(Self { _file: file })
    }
}

#[cfg(windows)]
struct OwnerSingleton {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl OwnerSingleton {
    fn acquire(home: &Path) -> OwnerResult<Self> {
        use crate::persistent_runtime::peer::{
            current_process_user_sid, validate_pipe_owner_and_dacl,
        };
        use std::ffi::c_void;
        use std::mem::size_of;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::null_mut;
        use windows_sys::Win32::Foundation::{
            CloseHandle, ERROR_SHARING_VIOLATION, GENERIC_READ, GENERIC_WRITE, GetLastError,
            INVALID_HANDLE_VALUE,
        };
        use windows_sys::Win32::Security::Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        };
        use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
        use windows_sys::Win32::Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, OPEN_ALWAYS, READ_CONTROL,
        };

        struct Descriptor(PSECURITY_DESCRIPTOR);
        impl Drop for Descriptor {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    // SAFETY: the descriptor was allocated by the SDDL conversion API via LocalAlloc.
                    let _ = unsafe { windows_sys::Win32::Foundation::LocalFree(self.0) };
                }
            }
        }

        let user_sid = current_process_user_sid()
            .map_err(|error| OwnerError::SingletonIo(format!("{error:?}")))?;
        let sid = user_sid
            .to_sddl_string()
            .map_err(|error| OwnerError::SingletonIo(format!("{error:?}")))?;
        let sddl = format!("O:{sid}D:P(A;;FA;;;{sid})");
        let mut sddl_wide: Vec<u16> = sddl.encode_utf16().collect();
        sddl_wide.push(0);
        let mut raw_descriptor: PSECURITY_DESCRIPTOR = null_mut();
        // SAFETY: sddl_wide is NUL-terminated and raw_descriptor is a valid writable output pointer.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl_wide.as_ptr(),
                SDDL_REVISION_1,
                &mut raw_descriptor,
                null_mut(),
            )
        } == 0
            || raw_descriptor.is_null()
        {
            return Err(OwnerError::SingletonSecurityMismatch);
        }
        let descriptor = Descriptor(raw_descriptor);
        let security = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor.0.cast::<c_void>(),
            bInheritHandle: 0,
        };
        let lock_path = home.join("persistent-owner.lock");
        let mut wide: Vec<u16> = lock_path.as_os_str().encode_wide().collect();
        if wide.contains(&0) {
            return Err(OwnerError::SingletonIo(
                "singleton path contains NUL".to_owned(),
            ));
        }
        wide.push(0);
        // SAFETY: wide is NUL-terminated, security points to a live descriptor, share mode zero
        // gives the singleton kernel ownership property, and template handle is null.
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ | GENERIC_WRITE | READ_CONTROL,
                0,
                &security,
                OPEN_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            // SAFETY: GetLastError has no preconditions.
            let code = unsafe { GetLastError() };
            if code == ERROR_SHARING_VIOLATION {
                return Err(OwnerError::SingletonAlreadyActive);
            }
            return Err(OwnerError::SingletonIo(format!("CreateFileW error {code}")));
        }
        if validate_pipe_owner_and_dacl(handle, &user_sid).is_err() {
            // SAFETY: handle is owned by this scope and has not been transferred.
            let _ = unsafe { CloseHandle(handle) };
            return Err(OwnerError::SingletonSecurityMismatch);
        }
        Ok(Self { handle })
    }
}

#[cfg(windows)]
impl Drop for OwnerSingleton {
    fn drop(&mut self) {
        if !self.handle.is_null()
            && self.handle != windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE
        {
            // SAFETY: this guard owns the singleton file handle and closes it exactly once.
            let _ = unsafe { windows_sys::Win32::Foundation::CloseHandle(self.handle) };
        }
    }
}

#[cfg(test)]
#[path = "../t151_persistent_owner_shell_tests.rs"]
mod t151_persistent_owner_shell_tests;

#[cfg(test)]
#[path = "../t152_persistent_terminal_tests.rs"]
mod t152_persistent_terminal_tests;

#[cfg(test)]
#[path = "../t154_controller_owner_tests.rs"]
mod t154_controller_owner_tests;

#[cfg(test)]
#[path = "../t157_owner_recovery_tests.rs"]
mod t157_owner_recovery_tests;

#[cfg(test)]
#[path = "../t165_multiplexer_owner_service_tests.rs"]
mod t165_multiplexer_owner_service_tests;
