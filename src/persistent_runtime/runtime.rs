use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::{TerminalExit, TerminalSession, TerminalSessionId, TerminalSize};
use crate::persistent_runtime::domain::{
    ClientConnectionId, ContinuityClass, EndpointAvailability, LifecycleProofClass,
    OwnerGenerationId, OwnershipState, ProcessLiveness, RuntimeAlias, RuntimeLifecycleEventKind,
    RuntimeNamespaceId, RuntimeTruth,
};
use crate::persistent_runtime::persistence::{
    PersistentRuntimeRecordInput, PersistentRuntimeRecoveryReason,
};
use crate::persistent_runtime::protocol::ProtocolMessage;
use crate::persistent_runtime::replay::{ObserverHandle, ReplayCoordinator, ReplayError};
use crate::store::Store;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TrySendError};
use std::sync::{Arc, Mutex, TryLockError};
use std::thread;
use std::time::Duration;

pub(crate) type RuntimeResult<T> = Result<T, PersistentTerminalRuntimeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PersistentTerminalRuntimeError {
    Entropy(String),
    Terminal(String),
    Store(String),
    UnknownRuntime,
    StaleOwnerGeneration,
    RuntimeNotLive,
    RuntimeNamespaceCollision,
    OutputGap,
    Replay(String),
}

impl fmt::Display for PersistentTerminalRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entropy(message) => {
                write!(formatter, "runtime namespace entropy failed: {message}")
            }
            Self::Terminal(message) => {
                write!(formatter, "persistent terminal operation failed: {message}")
            }
            Self::Store(message) => {
                write!(
                    formatter,
                    "persistent terminal store update failed: {message}"
                )
            }
            Self::UnknownRuntime => formatter.write_str("persistent terminal runtime is unknown"),
            Self::StaleOwnerGeneration => formatter.write_str(
                "persistent terminal attachment does not match the live owner generation",
            ),
            Self::RuntimeNotLive => {
                formatter.write_str("persistent terminal runtime is not live-owned")
            }
            Self::RuntimeNamespaceCollision => formatter.write_str(
                "generated persistent terminal runtime namespace already exists in this owner",
            ),
            Self::OutputGap => formatter.write_str(
                "persistent terminal output exceeded the bounded T152 direct-output queue; the direct stream is incomplete",
            ),
            Self::Replay(message) => write!(formatter, "persistent terminal replay failed: {message}"),
        }
    }
}

impl Error for PersistentTerminalRuntimeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistentTerminalAttachment {
    runtime_namespace_id: RuntimeNamespaceId,
    owner_generation_id: OwnerGenerationId,
}

impl PersistentTerminalAttachment {
    pub(crate) fn runtime_namespace_id(&self) -> RuntimeNamespaceId {
        self.runtime_namespace_id
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistentTerminalSnapshot {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) runtime_alias: RuntimeAlias,
    pub(crate) truth: RuntimeTruth,
    pub(crate) terminal_session_id: TerminalSessionId,
    pub(crate) terminal_size: TerminalSize,
    pub(crate) exit: Option<TerminalExit>,
    pub(crate) last_lifecycle_event_kind: RuntimeLifecycleEventKind,
    pub(crate) last_observed_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PersistentRuntimeShutdownDisposition {
    Stopped,
    AlreadyTerminal,
    OwnershipLost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersistentRuntimeShutdownReport {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) disposition: PersistentRuntimeShutdownDisposition,
    pub(crate) snapshot: PersistentTerminalSnapshot,
    pub(crate) error: Option<String>,
}

const OUTPUT_PUMP_CHUNK_BYTES: usize = 4 * 1024;
const OUTPUT_PUMP_QUEUE_CHUNKS: usize = 64;
const OUTPUT_PUMP_POLL_MS: u64 = 25;

type SharedReplay = Arc<Mutex<ReplayCoordinator>>;

struct RuntimeOutputPump {
    receiver: Receiver<Vec<u8>>,
    closed: Arc<AtomicBool>,
    error: Arc<Mutex<Option<String>>>,
    output_gap: Arc<AtomicBool>,
    replay_source_gap: Arc<AtomicBool>,
    replay_error: Arc<Mutex<Option<String>>>,
    pending: VecDeque<u8>,
}

impl RuntimeOutputPump {
    fn start(
        mut reader: Box<dyn Read + Send>,
        runtime_namespace_id: RuntimeNamespaceId,
        replay: SharedReplay,
    ) -> RuntimeResult<Self> {
        let (sender, receiver) = mpsc::sync_channel(OUTPUT_PUMP_QUEUE_CHUNKS);
        let closed = Arc::new(AtomicBool::new(false));
        let error = Arc::new(Mutex::new(None));
        let output_gap = Arc::new(AtomicBool::new(false));
        let replay_source_gap = Arc::new(AtomicBool::new(false));
        let replay_error = Arc::new(Mutex::new(None));
        let thread_closed = Arc::clone(&closed);
        let thread_error = Arc::clone(&error);
        let thread_output_gap = Arc::clone(&output_gap);
        let thread_replay_source_gap = Arc::clone(&replay_source_gap);
        let thread_replay_error = Arc::clone(&replay_error);
        let thread_replay = Arc::clone(&replay);

        thread::Builder::new()
            .name("winds-persistent-pty-output".to_owned())
            .spawn(move || {
                let mut buffer = [0_u8; OUTPUT_PUMP_CHUNK_BYTES];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            thread_closed.store(true, Ordering::Release);
                            return;
                        }
                        Ok(count) => {
                            let chunk = buffer[..count].to_vec();
                            match thread_replay.try_lock() {
                                Ok(mut coordinator) => {
                                    if thread_replay_source_gap.swap(false, Ordering::AcqRel)
                                        && let Err(error) =
                                            coordinator.note_source_gap(runtime_namespace_id)
                                    {
                                        store_replay_error(&thread_replay_error, error);
                                        thread_replay_source_gap.store(true, Ordering::Release);
                                    }
                                    if let Err(error) =
                                        coordinator.append_output(runtime_namespace_id, &chunk)
                                    {
                                        store_replay_error(&thread_replay_error, error);
                                        thread_replay_source_gap.store(true, Ordering::Release);
                                    }
                                }
                                Err(TryLockError::WouldBlock) => {
                                    thread_replay_source_gap.store(true, Ordering::Release);
                                }
                                Err(TryLockError::Poisoned(_)) => {
                                    if let Ok(mut slot) = thread_replay_error.lock() {
                                        *slot = Some(
                                            "persistent replay coordinator lock was poisoned"
                                                .to_owned(),
                                        );
                                    }
                                    thread_replay_source_gap.store(true, Ordering::Release);
                                }
                            }

                            match sender.try_send(chunk) {
                                Ok(()) => {}
                                Err(TrySendError::Full(_)) => {
                                    thread_output_gap.store(true, Ordering::Release);
                                }
                                Err(TrySendError::Disconnected(_)) => {
                                    thread_closed.store(true, Ordering::Release);
                                    return;
                                }
                            }
                        }
                        Err(read_error) => {
                            if let Ok(mut slot) = thread_error.lock() {
                                *slot = Some(read_error.to_string());
                            }
                            thread_closed.store(true, Ordering::Release);
                            return;
                        }
                    }
                }
            })
            .map_err(|error| {
                PersistentTerminalRuntimeError::Terminal(format!(
                    "persistent terminal output pump could not start: {error}"
                ))
            })?;

        Ok(Self {
            receiver,
            closed,
            error,
            output_gap,
            replay_source_gap,
            replay_error,
            pending: VecDeque::new(),
        })
    }

    fn take_replay_source_gap(&self) -> bool {
        self.replay_source_gap.swap(false, Ordering::AcqRel)
    }

    fn replay_error(&self) -> RuntimeResult<()> {
        let error = self
            .replay_error
            .lock()
            .map_err(|_| {
                PersistentTerminalRuntimeError::Replay(
                    "persistent replay error state was poisoned".to_owned(),
                )
            })?
            .clone();
        if let Some(error) = error {
            return Err(PersistentTerminalRuntimeError::Replay(error));
        }
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> RuntimeResult<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        loop {
            if self.output_gap.load(Ordering::Acquire) {
                return Err(PersistentTerminalRuntimeError::OutputGap);
            }

            if !self.pending.is_empty() {
                let count = buffer.len().min(self.pending.len());
                for slot in &mut buffer[..count] {
                    *slot = self
                        .pending
                        .pop_front()
                        .expect("pending output length was checked");
                }
                return Ok(count);
            }

            match self
                .receiver
                .recv_timeout(Duration::from_millis(OUTPUT_PUMP_POLL_MS))
            {
                Ok(chunk) => self.pending.extend(chunk),
                Err(RecvTimeoutError::Timeout) => {
                    if self.closed.load(Ordering::Acquire) {
                        if let Some(error) = self
                            .error
                            .lock()
                            .map_err(|_| {
                                PersistentTerminalRuntimeError::Terminal(
                                    "persistent terminal output pump error state was poisoned"
                                        .to_owned(),
                                )
                            })?
                            .clone()
                        {
                            return Err(PersistentTerminalRuntimeError::Terminal(error));
                        }
                        return Ok(0);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    if let Some(error) = self
                        .error
                        .lock()
                        .map_err(|_| {
                            PersistentTerminalRuntimeError::Terminal(
                                "persistent terminal output pump error state was poisoned"
                                    .to_owned(),
                            )
                        })?
                        .clone()
                    {
                        return Err(PersistentTerminalRuntimeError::Terminal(error));
                    }
                    return Ok(0);
                }
            }
        }
    }
}

fn store_replay_error(slot: &Arc<Mutex<Option<String>>>, error: ReplayError) {
    if let Ok(mut stored) = slot.lock() {
        *stored = Some(error.to_string());
    }
}

struct OwnedTerminalRuntime {
    runtime_namespace_id: RuntimeNamespaceId,
    owner_generation_id: OwnerGenerationId,
    runtime_alias: RuntimeAlias,
    session: TerminalSession,
    output_pump: RuntimeOutputPump,
    terminal_size: TerminalSize,
    truth: RuntimeTruth,
    exit: Option<TerminalExit>,
    last_lifecycle_event_kind: RuntimeLifecycleEventKind,
    created_unix_ms: i64,
    updated_unix_ms: i64,
    last_observed_unix_ms: Option<i64>,
    ownership_lost_unix_ms: Option<i64>,
    recovery_reason: Option<PersistentRuntimeRecoveryReason>,
    persistence_dirty: bool,
}

impl OwnedTerminalRuntime {
    fn snapshot(&self) -> PersistentTerminalSnapshot {
        PersistentTerminalSnapshot {
            runtime_namespace_id: self.runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
            runtime_alias: self.runtime_alias.clone(),
            truth: self.truth.clone(),
            terminal_session_id: self.session.session_id(),
            terminal_size: self.terminal_size,
            exit: self.exit.clone(),
            last_lifecycle_event_kind: self.last_lifecycle_event_kind,
            last_observed_unix_ms: self.last_observed_unix_ms,
        }
    }

    fn is_live(&self) -> bool {
        self.truth.ownership == OwnershipState::LiveOwned
            && self.truth.process_liveness == ProcessLiveness::Running
    }

    fn mark_final(
        &mut self,
        exit: TerminalExit,
        event_kind: RuntimeLifecycleEventKind,
        observed_unix_ms: i64,
    ) {
        self.truth = RuntimeTruth {
            ownership: OwnershipState::Unowned,
            process_liveness: ProcessLiveness::Exited,
            endpoint_availability: EndpointAvailability::Available,
            continuity: ContinuityClass::RetainedLiveProcess,
        };
        self.exit = Some(exit);
        self.last_lifecycle_event_kind = event_kind;
        self.updated_unix_ms = observed_unix_ms;
        self.last_observed_unix_ms = Some(observed_unix_ms);
        self.ownership_lost_unix_ms = None;
        self.recovery_reason = None;
        self.persistence_dirty = true;
    }

    fn mark_cleanup_unproven(&mut self, observed_unix_ms: i64) {
        self.session.suppress_drop_cleanup_after_ownership_loss();
        self.truth = RuntimeTruth {
            ownership: OwnershipState::OwnershipLost,
            process_liveness: ProcessLiveness::Unknown,
            endpoint_availability: EndpointAvailability::Unknown,
            continuity: ContinuityClass::Unknown,
        };
        self.exit = None;
        self.last_lifecycle_event_kind = RuntimeLifecycleEventKind::OwnershipLost;
        self.updated_unix_ms = observed_unix_ms;
        self.last_observed_unix_ms = Some(observed_unix_ms);
        self.ownership_lost_unix_ms = Some(observed_unix_ms);
        self.recovery_reason = Some(PersistentRuntimeRecoveryReason::OwnerGenerationUnproven);
        self.persistence_dirty = true;
    }
}

pub(crate) struct PersistentTerminalRegistry {
    owner_generation_id: OwnerGenerationId,
    runtimes: HashMap<RuntimeNamespaceId, OwnedTerminalRuntime>,
    replay: SharedReplay,
}

impl PersistentTerminalRegistry {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            runtimes: HashMap::new(),
            replay: Arc::new(Mutex::new(ReplayCoordinator::new(owner_generation_id))),
        }
    }

    pub(crate) fn live_count(&self) -> usize {
        self.runtimes
            .values()
            .filter(|runtime| runtime.is_live())
            .count()
    }

    pub(crate) fn runtime_ids(&self) -> Vec<RuntimeNamespaceId> {
        let mut runtime_ids = self.runtimes.keys().copied().collect::<Vec<_>>();
        runtime_ids.sort();
        runtime_ids
    }

    pub(crate) fn shutdown_all(
        &mut self,
        store: &Store,
        now_unix_ms: i64,
    ) -> Vec<PersistentRuntimeShutdownReport> {
        let runtime_ids = self.runtime_ids();
        let mut reports = Vec::with_capacity(runtime_ids.len());
        for runtime_namespace_id in runtime_ids {
            let before = self
                .runtimes
                .get(&runtime_namespace_id)
                .expect("runtime id came from the registry")
                .snapshot();

            if before.truth.ownership == OwnershipState::OwnershipLost {
                reports.push(PersistentRuntimeShutdownReport {
                    runtime_namespace_id,
                    disposition: PersistentRuntimeShutdownDisposition::OwnershipLost,
                    snapshot: before,
                    error: None,
                });
                continue;
            }
            if before.truth.process_liveness == ProcessLiveness::Exited
                || before.truth.ownership == OwnershipState::Unowned
            {
                reports.push(PersistentRuntimeShutdownReport {
                    runtime_namespace_id,
                    disposition: PersistentRuntimeShutdownDisposition::AlreadyTerminal,
                    snapshot: before,
                    error: None,
                });
                continue;
            }

            let attachment = PersistentTerminalAttachment {
                runtime_namespace_id,
                owner_generation_id: self.owner_generation_id,
            };
            match self.terminate(store, &attachment, now_unix_ms) {
                Ok(snapshot) => reports.push(PersistentRuntimeShutdownReport {
                    runtime_namespace_id,
                    disposition: PersistentRuntimeShutdownDisposition::Stopped,
                    snapshot,
                    error: None,
                }),
                Err(error) => {
                    let snapshot = self
                        .runtimes
                        .get(&runtime_namespace_id)
                        .expect("failed shutdown retains its runtime record")
                        .snapshot();
                    reports.push(PersistentRuntimeShutdownReport {
                        runtime_namespace_id,
                        disposition: if snapshot.truth.ownership == OwnershipState::OwnershipLost {
                            PersistentRuntimeShutdownDisposition::OwnershipLost
                        } else {
                            PersistentRuntimeShutdownDisposition::AlreadyTerminal
                        },
                        snapshot,
                        error: Some(error.to_string()),
                    });
                }
            }
        }
        reports
    }

    pub(crate) fn is_live_runtime(&self, runtime_namespace_id: RuntimeNamespaceId) -> bool {
        self.runtimes
            .get(&runtime_namespace_id)
            .is_some_and(OwnedTerminalRuntime::is_live)
    }

    pub(crate) fn attachment_for_runtime(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> RuntimeResult<PersistentTerminalAttachment> {
        let runtime = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if !runtime.is_live() {
            return Err(PersistentTerminalRuntimeError::RuntimeNotLive);
        }
        Ok(PersistentTerminalAttachment {
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
        })
    }

    pub(crate) fn record_controller_changed(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        controller_client_id: Option<ClientConnectionId>,
        observed_unix_ms: i64,
    ) -> RuntimeResult<()> {
        self.reconcile_replay_ingress(runtime_namespace_id)?;
        self.with_replay_mut(|replay| {
            replay.append_lifecycle(
                runtime_namespace_id,
                RuntimeLifecycleEventKind::ControllerChanged,
                LifecycleProofClass::WindsObserved,
                controller_client_id,
                u64::try_from(observed_unix_ms).ok(),
            )?;
            Ok(())
        })
    }

    pub(crate) fn attach_observer(
        &mut self,
        connection_id: ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> RuntimeResult<ObserverHandle> {
        self.reconcile_replay_ingress(runtime_namespace_id)?;
        self.with_replay_mut(|replay| replay.attach_observer(connection_id, runtime_namespace_id))
    }

    pub(crate) fn detach_observer(&mut self, handle: &ObserverHandle) -> RuntimeResult<()> {
        self.with_replay_mut(|replay| replay.detach_observer(handle))
    }

    pub(crate) fn fill_observer_queue(&mut self, handle: &ObserverHandle) -> RuntimeResult<usize> {
        self.reconcile_replay_ingress(handle.runtime_namespace_id())?;
        self.with_replay_mut(|replay| replay.fill_observer_queue(handle))
    }

    pub(crate) fn drain_observer(
        &mut self,
        handle: &ObserverHandle,
    ) -> RuntimeResult<Vec<ProtocolMessage>> {
        self.with_replay_mut(|replay| replay.drain_observer(handle))
    }

    pub(crate) fn start_shell(
        &mut self,
        store: &Store,
        runtime_alias: RuntimeAlias,
        profile: &ShellProfile,
        cwd: &Path,
        terminal_size: TerminalSize,
        now_unix_ms: i64,
    ) -> RuntimeResult<PersistentTerminalAttachment> {
        let runtime_namespace_id = generate_runtime_namespace_id()?;
        if self.runtimes.contains_key(&runtime_namespace_id) {
            return Err(PersistentTerminalRuntimeError::RuntimeNamespaceCollision);
        }

        let mut session = TerminalSession::start(profile, cwd, terminal_size)
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))?;
        let output_reader = match session.take_output_reader() {
            Ok(reader) => reader,
            Err(error) => {
                let cleanup = session.terminate();
                let suffix = cleanup
                    .err()
                    .map(|cleanup_error| format!("; cleanup also failed: {cleanup_error}"))
                    .unwrap_or_default();
                return Err(PersistentTerminalRuntimeError::Terminal(format!(
                    "terminal output ownership could not move under the persistent owner: {error}{suffix}"
                )));
            }
        };

        self.with_replay_mut(|replay| {
            replay.register_runtime(runtime_namespace_id);
            Ok(())
        })?;
        if let Err(error) = self.with_replay_mut(|replay| {
            replay.append_lifecycle(
                runtime_namespace_id,
                RuntimeLifecycleEventKind::OwnershipEstablished,
                LifecycleProofClass::WindsObserved,
                None,
                u64::try_from(now_unix_ms).ok(),
            )?;
            Ok(())
        }) {
            let _ = self.with_replay_mut(|replay| replay.unregister_runtime(runtime_namespace_id));
            let cleanup = session.terminate();
            if let Err(cleanup_error) = cleanup {
                return Err(PersistentTerminalRuntimeError::Terminal(format!(
                    "{error}; bounded cleanup after replay initialization failure also failed: {cleanup_error}"
                )));
            }
            return Err(error);
        }

        let output_pump = match RuntimeOutputPump::start(
            output_reader,
            runtime_namespace_id,
            Arc::clone(&self.replay),
        ) {
            Ok(pump) => pump,
            Err(error) => {
                let _ =
                    self.with_replay_mut(|replay| replay.unregister_runtime(runtime_namespace_id));
                let cleanup = session.terminate();
                let suffix = cleanup
                    .err()
                    .map(|cleanup_error| format!("; cleanup also failed: {cleanup_error}"))
                    .unwrap_or_default();
                return Err(PersistentTerminalRuntimeError::Terminal(format!(
                    "{error}{suffix}"
                )));
            }
        };

        let runtime = OwnedTerminalRuntime {
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
            runtime_alias,
            session,
            output_pump,
            terminal_size,
            truth: RuntimeTruth {
                ownership: OwnershipState::LiveOwned,
                process_liveness: ProcessLiveness::Running,
                endpoint_availability: EndpointAvailability::Available,
                continuity: ContinuityClass::RetainedLiveProcess,
            },
            exit: None,
            last_lifecycle_event_kind: RuntimeLifecycleEventKind::OwnershipEstablished,
            created_unix_ms: now_unix_ms,
            updated_unix_ms: now_unix_ms,
            last_observed_unix_ms: Some(now_unix_ms),
            ownership_lost_unix_ms: None,
            recovery_reason: None,
            persistence_dirty: false,
        };

        if let Err(error) = persist_runtime(store, &runtime) {
            let mut runtime = runtime;
            let cleanup = runtime.session.terminate();
            let _ = self.with_replay_mut(|replay| replay.unregister_runtime(runtime_namespace_id));
            if let Err(cleanup_error) = cleanup {
                return Err(PersistentTerminalRuntimeError::Terminal(format!(
                    "{error}; bounded cleanup after persistence failure also failed: {cleanup_error}"
                )));
            }
            return Err(error);
        }

        let attachment = PersistentTerminalAttachment {
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
        };
        self.runtimes.insert(runtime_namespace_id, runtime);
        Ok(attachment)
    }

    pub(crate) fn reattach(
        &mut self,
        store: &Store,
        runtime_namespace_id: RuntimeNamespaceId,
        expected_owner_generation_id: OwnerGenerationId,
        now_unix_ms: i64,
    ) -> RuntimeResult<PersistentTerminalAttachment> {
        if expected_owner_generation_id != self.owner_generation_id {
            return Err(PersistentTerminalRuntimeError::StaleOwnerGeneration);
        }
        self.observe_one(store, runtime_namespace_id, now_unix_ms)?;
        let runtime = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if !runtime.is_live() {
            return Err(PersistentTerminalRuntimeError::RuntimeNotLive);
        }
        Ok(PersistentTerminalAttachment {
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
        })
    }

    pub(crate) fn snapshot(
        &mut self,
        store: &Store,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
    ) -> RuntimeResult<PersistentTerminalSnapshot> {
        self.validate_attachment(attachment)?;
        self.observe_one(store, attachment.runtime_namespace_id, now_unix_ms)?;
        Ok(self
            .runtimes
            .get(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?
            .snapshot())
    }

    pub(crate) fn send_input(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        bytes: &[u8],
    ) -> RuntimeResult<()> {
        self.live_runtime_mut(attachment)?
            .session
            .send_input(bytes)
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))
    }

    pub(crate) fn read_output(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        buffer: &mut [u8],
    ) -> RuntimeResult<usize> {
        self.validate_attachment(attachment)?;
        self.runtimes
            .get_mut(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?
            .output_pump
            .read(buffer)
    }

    pub(crate) fn resize(
        &mut self,
        attachment: &PersistentTerminalAttachment,
        terminal_size: TerminalSize,
    ) -> RuntimeResult<()> {
        let runtime = self.live_runtime_mut(attachment)?;
        runtime
            .session
            .resize(terminal_size)
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))?;
        runtime.terminal_size = terminal_size;
        Ok(())
    }

    pub(crate) fn current_size(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> RuntimeResult<TerminalSize> {
        let runtime = self.live_runtime_mut(attachment)?;
        runtime
            .session
            .current_size()
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))
    }

    pub(crate) fn interrupt(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> RuntimeResult<()> {
        self.live_runtime_mut(attachment)?
            .session
            .interrupt()
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))
    }

    pub(crate) fn terminate(
        &mut self,
        store: &Store,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
    ) -> RuntimeResult<PersistentTerminalSnapshot> {
        self.finish_owned(
            store,
            attachment,
            now_unix_ms,
            RuntimeLifecycleEventKind::RuntimeStopped,
            |session| session.terminate(),
        )
    }

    pub(crate) fn close(
        &mut self,
        store: &Store,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
    ) -> RuntimeResult<PersistentTerminalSnapshot> {
        self.finish_owned(
            store,
            attachment,
            now_unix_ms,
            RuntimeLifecycleEventKind::RuntimeStopped,
            |session| session.close(),
        )
    }

    pub(crate) fn poll_exits(&mut self, store: &Store, now_unix_ms: i64) -> RuntimeResult<usize> {
        let runtime_ids: Vec<_> = self.runtimes.keys().copied().collect();
        let mut observed = 0_usize;
        for runtime_namespace_id in runtime_ids {
            if self.observe_one(store, runtime_namespace_id, now_unix_ms)? {
                observed = observed.saturating_add(1);
            }
        }
        Ok(observed)
    }

    fn finish_owned<F>(
        &mut self,
        store: &Store,
        attachment: &PersistentTerminalAttachment,
        now_unix_ms: i64,
        event_kind: RuntimeLifecycleEventKind,
        operation: F,
    ) -> RuntimeResult<PersistentTerminalSnapshot>
    where
        F: FnOnce(&mut TerminalSession) -> crate::git::Result<TerminalExit>,
    {
        self.validate_attachment(attachment)?;
        self.flush_dirty(store, attachment.runtime_namespace_id)?;
        if !self
            .runtimes
            .get(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?
            .is_live()
        {
            return Ok(self
                .runtimes
                .get(&attachment.runtime_namespace_id)
                .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?
                .snapshot());
        }

        let exit = {
            let runtime = self
                .runtimes
                .get_mut(&attachment.runtime_namespace_id)
                .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
            match operation(&mut runtime.session) {
                Ok(exit) => exit,
                Err(error) => {
                    let cleanup_error = error.to_string();
                    runtime.mark_cleanup_unproven(now_unix_ms);
                    let _ = runtime;
                    let persistence_result =
                        self.flush_dirty(store, attachment.runtime_namespace_id);
                    let lifecycle_result = self.append_lifecycle_if_changed(
                        attachment.runtime_namespace_id,
                        RuntimeLifecycleEventKind::OwnershipLost,
                        now_unix_ms,
                    );
                    let mut message = format!(
                        "cleanup outcome is unproven; live ownership was revoked: {cleanup_error}"
                    );
                    if let Err(persist_error) = persistence_result {
                        message.push_str(&format!(
                            "; ownership-loss persistence also failed: {persist_error}"
                        ));
                    }
                    if let Err(lifecycle_error) = lifecycle_result {
                        message.push_str(&format!(
                            "; ownership-loss replay event also failed: {lifecycle_error}"
                        ));
                    }
                    return Err(PersistentTerminalRuntimeError::Terminal(message));
                }
            }
        };
        {
            let runtime = self
                .runtimes
                .get_mut(&attachment.runtime_namespace_id)
                .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
            runtime.mark_final(exit, event_kind, now_unix_ms);
        }
        self.flush_dirty(store, attachment.runtime_namespace_id)?;
        self.append_lifecycle_if_changed(attachment.runtime_namespace_id, event_kind, now_unix_ms)?;
        Ok(self
            .runtimes
            .get(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?
            .snapshot())
    }

    fn observe_one(
        &mut self,
        store: &Store,
        runtime_namespace_id: RuntimeNamespaceId,
        now_unix_ms: i64,
    ) -> RuntimeResult<bool> {
        self.flush_dirty(store, runtime_namespace_id)?;
        let runtime = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if runtime.owner_generation_id != self.owner_generation_id {
            return Err(PersistentTerminalRuntimeError::StaleOwnerGeneration);
        }
        if !runtime.is_live() {
            return Ok(false);
        }
        let exit = runtime
            .session
            .try_wait()
            .map_err(|error| PersistentTerminalRuntimeError::Terminal(error.to_string()))?;
        let Some(exit) = exit else {
            return Ok(false);
        };
        runtime.mark_final(
            exit,
            RuntimeLifecycleEventKind::ProcessStateObserved,
            now_unix_ms,
        );
        let _ = runtime;
        self.flush_dirty(store, runtime_namespace_id)?;
        self.append_lifecycle_if_changed(
            runtime_namespace_id,
            RuntimeLifecycleEventKind::ProcessStateObserved,
            now_unix_ms,
        )?;
        Ok(true)
    }

    fn flush_dirty(
        &mut self,
        store: &Store,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> RuntimeResult<()> {
        let runtime = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if runtime.persistence_dirty {
            persist_runtime(store, runtime)?;
            runtime.persistence_dirty = false;
        }
        Ok(())
    }

    fn reconcile_replay_ingress(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> RuntimeResult<()> {
        let (source_gap, replay_error) = {
            let runtime = self
                .runtimes
                .get(&runtime_namespace_id)
                .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
            let source_gap = runtime.output_pump.take_replay_source_gap();
            let replay_error = runtime.output_pump.replay_error().err();
            (source_gap, replay_error)
        };
        if let Some(error) = replay_error {
            return Err(error);
        }
        if source_gap {
            self.with_replay_mut(|replay| {
                replay.note_source_gap(runtime_namespace_id)?;
                Ok(())
            })?;
        }
        Ok(())
    }

    fn append_lifecycle_if_changed(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        kind: RuntimeLifecycleEventKind,
        observed_unix_ms: i64,
    ) -> RuntimeResult<()> {
        self.reconcile_replay_ingress(runtime_namespace_id)?;
        self.with_replay_mut(|replay| {
            replay.append_lifecycle_if_changed(
                runtime_namespace_id,
                kind,
                LifecycleProofClass::WindsObserved,
                u64::try_from(observed_unix_ms).ok(),
            )?;
            Ok(())
        })
    }

    fn with_replay_mut<T, F>(&self, operation: F) -> RuntimeResult<T>
    where
        F: FnOnce(&mut ReplayCoordinator) -> Result<T, ReplayError>,
    {
        let mut replay = self.replay.lock().map_err(|_| {
            PersistentTerminalRuntimeError::Replay(
                "persistent replay coordinator lock was poisoned".to_owned(),
            )
        })?;
        operation(&mut replay)
            .map_err(|error| PersistentTerminalRuntimeError::Replay(error.to_string()))
    }

    fn validate_attachment(&self, attachment: &PersistentTerminalAttachment) -> RuntimeResult<()> {
        if attachment.owner_generation_id != self.owner_generation_id {
            return Err(PersistentTerminalRuntimeError::StaleOwnerGeneration);
        }
        let runtime = self
            .runtimes
            .get(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if runtime.owner_generation_id != self.owner_generation_id {
            return Err(PersistentTerminalRuntimeError::StaleOwnerGeneration);
        }
        Ok(())
    }

    fn live_runtime_mut(
        &mut self,
        attachment: &PersistentTerminalAttachment,
    ) -> RuntimeResult<&mut OwnedTerminalRuntime> {
        self.validate_attachment(attachment)?;
        let runtime = self
            .runtimes
            .get_mut(&attachment.runtime_namespace_id)
            .ok_or(PersistentTerminalRuntimeError::UnknownRuntime)?;
        if !runtime.is_live() {
            return Err(PersistentTerminalRuntimeError::RuntimeNotLive);
        }
        Ok(runtime)
    }
}

fn persist_runtime(store: &Store, runtime: &OwnedTerminalRuntime) -> RuntimeResult<()> {
    store
        .persist_persistent_runtime_record(&PersistentRuntimeRecordInput {
            runtime_namespace_id: runtime.runtime_namespace_id,
            runtime_alias: &runtime.runtime_alias,
            workspace_id: None,
            session_id: None,
            terminal_execution_id: None,
            owner_generation_id: Some(runtime.owner_generation_id),
            truth: &runtime.truth,
            last_lifecycle_event_kind: runtime.last_lifecycle_event_kind,
            created_unix_ms: runtime.created_unix_ms,
            updated_unix_ms: runtime.updated_unix_ms,
            last_observed_unix_ms: runtime.last_observed_unix_ms,
            ownership_lost_unix_ms: runtime.ownership_lost_unix_ms,
            recovery_reason: runtime.recovery_reason,
        })
        .map_err(|error| PersistentTerminalRuntimeError::Store(error.to_string()))
}

fn generate_runtime_namespace_id() -> RuntimeResult<RuntimeNamespaceId> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        crate::persistent_runtime::transport::unix::generate_runtime_namespace_id()
            .map_err(|error| PersistentTerminalRuntimeError::Entropy(format!("{error:?}")))
    }
    #[cfg(windows)]
    {
        crate::persistent_runtime::transport::windows::generate_runtime_namespace_id()
            .map_err(|error| PersistentTerminalRuntimeError::Entropy(format!("{error:?}")))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    {
        Err(PersistentTerminalRuntimeError::Entropy(
            "unsupported runtime platform".to_owned(),
        ))
    }
}

#[cfg(test)]
#[path = "../t157_runtime_recovery_tests.rs"]
mod t157_runtime_recovery_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Instant;

    fn replay_fixture(byte: u8) -> (RuntimeNamespaceId, SharedReplay) {
        let runtime_namespace_id = RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap();
        let owner_generation_id = OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap();
        let replay = Arc::new(Mutex::new(ReplayCoordinator::new(owner_generation_id)));
        replay
            .lock()
            .unwrap()
            .register_runtime(runtime_namespace_id);
        (runtime_namespace_id, replay)
    }

    #[test]
    fn t152_output_pump_saturation_is_bounded_nonblocking_and_fail_closed() {
        let payload = vec![
            b'x';
            OUTPUT_PUMP_CHUNK_BYTES
                .checked_mul(OUTPUT_PUMP_QUEUE_CHUNKS + 8)
                .expect("bounded test payload size")
        ];
        let (runtime_namespace_id, replay) = replay_fixture(0x31);
        let mut pump =
            RuntimeOutputPump::start(Box::new(Cursor::new(payload)), runtime_namespace_id, replay)
                .expect("pump must start");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump.closed.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }

        assert!(
            pump.closed.load(Ordering::Acquire),
            "bounded pump must drain the finite source without blocking on a full queue"
        );
        assert!(
            pump.output_gap.load(Ordering::Acquire),
            "queue saturation must be recorded explicitly"
        );

        let mut output = [0_u8; 32];
        assert_eq!(
            pump.read(&mut output),
            Err(PersistentTerminalRuntimeError::OutputGap)
        );
    }

    #[test]
    fn t153_output_pump_captures_replay_without_presentation_reads() {
        let payload = b"detached persistent replay output".to_vec();
        let (runtime_namespace_id, replay) = replay_fixture(0x32);
        let pump = RuntimeOutputPump::start(
            Box::new(Cursor::new(payload.clone())),
            runtime_namespace_id,
            Arc::clone(&replay),
        )
        .expect("pump must start");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump.closed.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(pump.closed.load(Ordering::Acquire));

        let mut coordinator = replay.lock().unwrap();
        let handle = coordinator
            .attach_observer(
                ClientConnectionId::new("runtime-pump-observer").unwrap(),
                runtime_namespace_id,
            )
            .unwrap();
        coordinator.fill_observer_queue(&handle).unwrap();
        let messages = coordinator.drain_observer(&handle).unwrap();
        let captured: Vec<u8> = messages
            .iter()
            .filter_map(|message| match &message.payload {
                crate::persistent_runtime::protocol::ProtocolPayload::OutputEvent { chunk } => {
                    Some(chunk.as_slice())
                }
                _ => None,
            })
            .flatten()
            .copied()
            .collect();
        assert_eq!(captured, payload);
    }

    #[test]
    fn t153_replay_lock_contention_never_blocks_pty_drain_and_marks_gap() {
        let payload = vec![b'g'; OUTPUT_PUMP_CHUNK_BYTES * 8];
        let (runtime_namespace_id, replay) = replay_fixture(0x33);
        let guard = replay.lock().unwrap();
        let pump = RuntimeOutputPump::start(
            Box::new(Cursor::new(payload)),
            runtime_namespace_id,
            Arc::clone(&replay),
        )
        .expect("pump must start");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump.closed.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            pump.closed.load(Ordering::Acquire),
            "replay coordination must never block the PTY reader"
        );
        assert!(
            pump.replay_source_gap.load(Ordering::Acquire),
            "contention must become an explicit replay source gap"
        );
        drop(guard);
    }
}
