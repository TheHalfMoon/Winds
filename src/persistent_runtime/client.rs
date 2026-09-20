use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId,
    RuntimeAlias, RuntimeLifecycleEvent, RuntimeNamespaceId, RuntimeTruth,
};
use crate::persistent_runtime::protocol::{
    MAX_INBOUND_CONTROL_FRAME_BYTES, MessageKind, MutationOutcomeTracker, PROTOCOL_VERSION,
    ProtocolMessage, ProtocolPayload, decode_frame, encode_frame, validate_event_binding,
    validate_response_binding,
};
use std::collections::{BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

pub(crate) const MAX_CLIENT_QUEUED_EVENT_BYTES: usize = 4 * 1024 * 1024;

pub(crate) type ClientResult<T> = Result<T, LocalControlClientError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LocalControlClientError {
    Transport(String),
    Protocol(LocalControlErrorKind),
    ProtocolMismatch,
    StaleOwnerGeneration,
    RemoteEndpointRejected,
    ExpectedOwnerGenerationRequired,
    Disconnected,
    SequenceExhausted,
    UnknownAlias(String),
    AmbiguousAlias { alias: String, matches: usize },
    OutcomeUnknown(RuntimeNamespaceId),
    ClientEventBackpressure,
    UnexpectedResponse(MessageKind),
}

impl fmt::Display for LocalControlClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(message) => write!(formatter, "local-control transport failed: {message}"),
            Self::Protocol(kind) => write!(formatter, "local-control protocol failed: {kind:?}"),
            Self::ProtocolMismatch => formatter.write_str("local-control protocol version mismatch"),
            Self::StaleOwnerGeneration => {
                formatter.write_str("local-control owner generation is stale or mismatched")
            }
            Self::RemoteEndpointRejected => formatter.write_str(
                "local-control endpoint override rejected; only the private local endpoint is accepted",
            ),
            Self::ExpectedOwnerGenerationRequired => formatter.write_str(
                "this local-control transport requires an exact expected owner generation",
            ),
            Self::Disconnected => formatter.write_str("local-control client is disconnected"),
            Self::SequenceExhausted => {
                formatter.write_str("local-control request sequence is exhausted")
            }
            Self::UnknownAlias(alias) => write!(formatter, "runtime alias is unknown: {alias}"),
            Self::AmbiguousAlias { alias, matches } => write!(
                formatter,
                "runtime alias is ambiguous and requires explicit ID disambiguation: {alias} ({matches} matches)"
            ),
            Self::OutcomeUnknown(runtime_namespace_id) => write!(
                formatter,
                "mutation outcome is unknown for runtime {runtime_namespace_id}; reconcile state before a new consequential action"
            ),
            Self::ClientEventBackpressure => formatter.write_str(
                "local-control client event queue reached its bounded 4 MiB ceiling",
            ),
            Self::UnexpectedResponse(kind) => {
                write!(formatter, "unexpected local-control response kind: {kind:?}")
            }
        }
    }
}

impl Error for LocalControlClientError {}

fn map_protocol_error(kind: LocalControlErrorKind) -> LocalControlClientError {
    match kind {
        LocalControlErrorKind::ProtocolMismatch => LocalControlClientError::ProtocolMismatch,
        LocalControlErrorKind::StaleOwnerGeneration => {
            LocalControlClientError::StaleOwnerGeneration
        }
        other => LocalControlClientError::Protocol(other),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeInventoryEntry {
    pub(crate) runtime_namespace_id: RuntimeNamespaceId,
    pub(crate) runtime_alias: RuntimeAlias,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeTargetSelector {
    Exact(RuntimeNamespaceId),
    Alias(RuntimeAlias),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedRuntimeTarget {
    runtime_namespace_id: RuntimeNamespaceId,
}

impl ResolvedRuntimeTarget {
    pub(crate) fn exact(runtime_namespace_id: RuntimeNamespaceId) -> Self {
        Self {
            runtime_namespace_id,
        }
    }

    pub(crate) fn runtime_namespace_id(self) -> RuntimeNamespaceId {
        self.runtime_namespace_id
    }
}

pub(crate) fn resolve_runtime_target(
    inventory: &[RuntimeInventoryEntry],
    selector: &RuntimeTargetSelector,
) -> ClientResult<ResolvedRuntimeTarget> {
    match selector {
        RuntimeTargetSelector::Exact(runtime_namespace_id) => {
            Ok(ResolvedRuntimeTarget::exact(*runtime_namespace_id))
        }
        RuntimeTargetSelector::Alias(alias) => {
            let mut matches = inventory
                .iter()
                .filter(|entry| entry.runtime_alias == *alias)
                .map(|entry| entry.runtime_namespace_id);
            let Some(first) = matches.next() else {
                return Err(LocalControlClientError::UnknownAlias(
                    alias.as_str().to_owned(),
                ));
            };
            let additional = matches.count();
            if additional > 0 {
                return Err(LocalControlClientError::AmbiguousAlias {
                    alias: alias.as_str().to_owned(),
                    matches: additional.saturating_add(1),
                });
            }
            Ok(ResolvedRuntimeTarget::exact(first))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClientResponseProjection {
    RuntimeSnapshot {
        runtime_namespace_id: RuntimeNamespaceId,
        truth: RuntimeTruth,
    },
    ControlState {
        runtime_namespace_id: RuntimeNamespaceId,
        authority: ClientAuthority,
        controller_client_id: Option<ClientConnectionId>,
    },
    Pong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClientEventProjection {
    RuntimeEvent {
        runtime_namespace_id: RuntimeNamespaceId,
        event: RuntimeLifecycleEvent,
    },
    Output {
        runtime_namespace_id: RuntimeNamespaceId,
        chunk: Vec<u8>,
    },
    HistoryGap {
        runtime_namespace_id: RuntimeNamespaceId,
        first_available_sequence: EventSequence,
        last_dropped_sequence: EventSequence,
    },
    OwnerStatus {
        ready: bool,
    },
}

#[derive(Debug)]
enum WireError {
    Transport(String),
    Protocol(LocalControlErrorKind),
}

trait LocalControlWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError>;
    fn receive(&mut self) -> Result<ProtocolMessage, WireError>;
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct PlatformWire {
    stream: std::os::unix::net::UnixStream,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl PlatformWire {
    fn connect_default(
        _expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        let stream = crate::persistent_runtime::transport::unix::connect_default_same_user()
            .map_err(|error| LocalControlClientError::Transport(format!("{error:?}")))?;
        Ok(Self { stream })
    }

    #[cfg(test)]
    fn connect_runtime_directory(runtime_directory: &std::path::Path) -> ClientResult<Self> {
        let stream =
            crate::persistent_runtime::transport::unix::connect_same_user(runtime_directory)
                .map_err(|error| LocalControlClientError::Transport(format!("{error:?}")))?;
        Ok(Self { stream })
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl LocalControlWire for PlatformWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        use std::io::Write;
        let frame = encode_frame(message).map_err(WireError::Protocol)?;
        self.stream
            .write_all(&frame)
            .map_err(|error| WireError::Transport(error.to_string()))
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        use std::io::Read;
        let mut length_bytes = [0_u8; 4];
        self.stream
            .read_exact(&mut length_bytes)
            .map_err(|error| WireError::Transport(error.to_string()))?;
        let claimed = u32::from_le_bytes(length_bytes) as usize;
        if claimed == 0 {
            return Err(WireError::Protocol(LocalControlErrorKind::MalformedFrame));
        }
        if claimed > MAX_INBOUND_CONTROL_FRAME_BYTES {
            return Err(WireError::Protocol(LocalControlErrorKind::OversizedFrame));
        }
        let mut payload = vec![0_u8; claimed];
        self.stream
            .read_exact(&mut payload)
            .map_err(|error| WireError::Transport(error.to_string()))?;
        let mut frame = Vec::with_capacity(4 + claimed);
        frame.extend_from_slice(&length_bytes);
        frame.extend_from_slice(&payload);
        decode_frame(&frame).map_err(WireError::Protocol)
    }
}

#[cfg(windows)]
struct PlatformWire {
    client: crate::persistent_runtime::transport::windows::WindowsNamedPipeClient,
}

#[cfg(windows)]
impl PlatformWire {
    fn connect_default(
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        let expected = expected_owner_generation_id
            .ok_or(LocalControlClientError::ExpectedOwnerGenerationRequired)?;
        let client =
            crate::persistent_runtime::transport::windows::WindowsNamedPipeClient::connect(
                expected,
            )
            .map_err(|error| LocalControlClientError::Transport(format!("{error:?}")))?;
        Ok(Self { client })
    }
}

#[cfg(windows)]
impl LocalControlWire for PlatformWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        let frame = encode_frame(message).map_err(WireError::Protocol)?;
        self.client
            .write_all(&frame)
            .map_err(|error| WireError::Transport(format!("{error:?}")))
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        let mut length_bytes = [0_u8; 4];
        self.client
            .read_exact(&mut length_bytes)
            .map_err(|error| WireError::Transport(format!("{error:?}")))?;
        let claimed = u32::from_le_bytes(length_bytes) as usize;
        if claimed == 0 {
            return Err(WireError::Protocol(LocalControlErrorKind::MalformedFrame));
        }
        if claimed > MAX_INBOUND_CONTROL_FRAME_BYTES {
            return Err(WireError::Protocol(LocalControlErrorKind::OversizedFrame));
        }
        let mut payload = vec![0_u8; claimed];
        self.client
            .read_exact(&mut payload)
            .map_err(|error| WireError::Transport(format!("{error:?}")))?;
        let mut frame = Vec::with_capacity(4 + claimed);
        frame.extend_from_slice(&length_bytes);
        frame.extend_from_slice(&payload);
        decode_frame(&frame).map_err(WireError::Protocol)
    }
}

pub(crate) struct RustLocalControlClient {
    wire: Option<Box<dyn LocalControlWire>>,
    connection_id: ClientConnectionId,
    owner_generation_id: OwnerGenerationId,
    next_sequence: u64,
    mutation_outcomes: MutationOutcomeTracker,
    attached_runtimes: BTreeSet<RuntimeNamespaceId>,
    pending_events: VecDeque<ProtocolMessage>,
    pending_event_bytes: usize,
}

impl RustLocalControlClient {
    pub(crate) fn connect(
        endpoint_hint: Option<&str>,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        validate_local_endpoint_hint(endpoint_hint)?;
        let wire = PlatformWire::connect_default(expected_owner_generation_id)?;
        Self::handshake(Box::new(wire), expected_owner_generation_id)
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }

    pub(crate) fn connection_id(&self) -> &ClientConnectionId {
        &self.connection_id
    }

    pub(crate) fn is_outcome_unknown(&self) -> bool {
        self.mutation_outcomes.is_outcome_unknown()
    }

    pub(crate) fn queued_event_bytes(&self) -> usize {
        self.pending_event_bytes
    }

    pub(crate) fn reconnect(&mut self) -> ClientResult<()> {
        let wire = PlatformWire::connect_default(Some(self.owner_generation_id))?;
        self.replace_after_handshake(Box::new(wire))
    }

    pub(crate) fn attach_observer(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        let runtime_namespace_id = target.runtime_namespace_id();
        let newly_attached = self.attached_runtimes.insert(runtime_namespace_id);
        let result = self
            .transact(
                Some(runtime_namespace_id),
                ProtocolPayload::AttachObserver,
                false,
            )
            .and_then(|(_, response)| project_response(&response));
        if result.is_err() && newly_attached {
            self.attached_runtimes.remove(&runtime_namespace_id);
        }
        result
    }

    pub(crate) fn detach_observer(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        let runtime_namespace_id = target.runtime_namespace_id();
        let (_, response) =
            self.transact(Some(runtime_namespace_id), ProtocolPayload::Detach, false)?;
        let projection = project_response(&response)?;
        self.attached_runtimes.remove(&runtime_namespace_id);
        Ok(projection)
    }

    pub(crate) fn request_control(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::RequestControl)
    }

    pub(crate) fn release_control(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::ReleaseControl)
    }

    pub(crate) fn send_input(
        &mut self,
        target: ResolvedRuntimeTarget,
        data: Vec<u8>,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::Input { data })
    }

    pub(crate) fn resize(
        &mut self,
        target: ResolvedRuntimeTarget,
        columns: u16,
        rows: u16,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::Resize { columns, rows })
    }

    pub(crate) fn interrupt(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::Interrupt)
    }

    pub(crate) fn stop(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        self.mutate(target, ProtocolPayload::Stop)
    }

    pub(crate) fn reconcile_runtime(
        &mut self,
        target: ResolvedRuntimeTarget,
    ) -> ClientResult<ClientResponseProjection> {
        let runtime_namespace_id = target.runtime_namespace_id();
        let newly_attached = self.attached_runtimes.insert(runtime_namespace_id);
        let result = (|| {
            let (request, response) = self.transact(
                Some(runtime_namespace_id),
                ProtocolPayload::AttachObserver,
                false,
            )?;
            self.mutation_outcomes
                .reconcile_state(&request, &response)
                .map_err(map_protocol_error)?;
            project_response(&response)
        })();
        if result.is_err() && newly_attached {
            self.attached_runtimes.remove(&runtime_namespace_id);
        }
        result
    }

    pub(crate) fn pop_event(&mut self) -> ClientResult<Option<ClientEventProjection>> {
        let Some(message) = self.pending_events.pop_front() else {
            return Ok(None);
        };
        let frame_bytes = encode_frame(&message).map_err(map_protocol_error)?.len();
        self.pending_event_bytes = self.pending_event_bytes.saturating_sub(frame_bytes);
        project_event(message).map(Some)
    }

    fn mutate(
        &mut self,
        target: ResolvedRuntimeTarget,
        payload: ProtocolPayload,
    ) -> ClientResult<ClientResponseProjection> {
        let runtime_namespace_id = target.runtime_namespace_id();
        if self.mutation_outcomes.is_outcome_unknown() {
            return Err(LocalControlClientError::OutcomeUnknown(
                runtime_namespace_id,
            ));
        }
        let (_, response) = self.transact(Some(runtime_namespace_id), payload, true)?;
        project_response(&response)
    }

    fn handshake(
        mut wire: Box<dyn LocalControlWire>,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        let sequence =
            EventSequence::new(1).map_err(|_| LocalControlClientError::SequenceExhausted)?;
        let hello = ProtocolMessage::hello(
            sequence,
            PROTOCOL_VERSION,
            PROTOCOL_VERSION,
            expected_owner_generation_id,
        )
        .map_err(map_protocol_error)?;
        wire.send(&hello).map_err(map_wire_error)?;
        let response = wire.receive().map_err(map_wire_error)?;
        validate_response_binding(&hello, &response).map_err(map_protocol_error)?;
        match response.payload {
            ProtocolPayload::HelloAck => {}
            ProtocolPayload::Error { kind } => return Err(map_protocol_error(kind)),
            _ => return Err(LocalControlClientError::UnexpectedResponse(response.kind())),
        }
        let connection_id =
            response
                .connection_id
                .clone()
                .ok_or(LocalControlClientError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ))?;
        let owner_generation_id =
            response
                .owner_generation_id
                .ok_or(LocalControlClientError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ))?;
        Ok(Self {
            wire: Some(wire),
            connection_id,
            owner_generation_id,
            next_sequence: 2,
            mutation_outcomes: MutationOutcomeTracker::new(),
            attached_runtimes: BTreeSet::new(),
            pending_events: VecDeque::new(),
            pending_event_bytes: 0,
        })
    }

    fn replace_after_handshake(&mut self, wire: Box<dyn LocalControlWire>) -> ClientResult<()> {
        let replacement = Self::handshake(wire, Some(self.owner_generation_id))?;
        if replacement.owner_generation_id != self.owner_generation_id {
            return Err(LocalControlClientError::StaleOwnerGeneration);
        }
        self.wire = replacement.wire;
        self.connection_id = replacement.connection_id;
        self.next_sequence = replacement.next_sequence;
        self.attached_runtimes.clear();
        self.pending_events.clear();
        self.pending_event_bytes = 0;
        Ok(())
    }

    fn transact(
        &mut self,
        runtime_namespace_id: Option<RuntimeNamespaceId>,
        payload: ProtocolPayload,
        consequential_mutation: bool,
    ) -> ClientResult<(ProtocolMessage, ProtocolMessage)> {
        let sequence = self.next_request_sequence()?;
        let request = ProtocolMessage::new(
            self.connection_id.clone(),
            sequence,
            runtime_namespace_id,
            self.owner_generation_id,
            None,
            payload,
        )
        .map_err(map_protocol_error)?;

        if consequential_mutation {
            self.mutation_outcomes
                .begin(&request)
                .map_err(map_protocol_error)?;
        }

        let send_result = self
            .wire
            .as_mut()
            .ok_or(LocalControlClientError::Disconnected)?
            .send(&request);
        if let Err(error) = send_result {
            return self.fail_inflight(request, consequential_mutation, error);
        }

        loop {
            let received = match self
                .wire
                .as_mut()
                .ok_or(LocalControlClientError::Disconnected)?
                .receive()
            {
                Ok(message) => message,
                Err(error) => {
                    return self.fail_inflight(request, consequential_mutation, error);
                }
            };

            if received.correlation_sequence == Some(request.sequence) {
                if let Err(kind) = validate_response_binding(&request, &received) {
                    if consequential_mutation {
                        self.mutation_outcomes.connection_lost();
                        self.wire = None;
                        return Err(LocalControlClientError::OutcomeUnknown(
                            request
                                .runtime_namespace_id
                                .expect("consequential mutations are runtime-bound"),
                        ));
                    }
                    return Err(map_protocol_error(kind));
                }
                if consequential_mutation
                    && let Err(kind) = self.mutation_outcomes.acknowledge(&received)
                {
                    if kind == LocalControlErrorKind::OutcomeUnknown {
                        return Err(LocalControlClientError::OutcomeUnknown(
                            request
                                .runtime_namespace_id
                                .expect("consequential mutations are runtime-bound"),
                        ));
                    }
                    return Err(map_protocol_error(kind));
                }
                if let ProtocolPayload::Error { kind } = received.payload {
                    return Err(map_protocol_error(kind));
                }
                return Ok((request, received));
            }

            if received.correlation_sequence.is_some() {
                if consequential_mutation {
                    self.mutation_outcomes.connection_lost();
                    self.wire = None;
                    return Err(LocalControlClientError::OutcomeUnknown(
                        request
                            .runtime_namespace_id
                            .expect("consequential mutations are runtime-bound"),
                    ));
                }
                return Err(LocalControlClientError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ));
            }

            if let Err(error) = self.queue_event(received) {
                if consequential_mutation {
                    self.mutation_outcomes.connection_lost();
                    self.wire = None;
                    return Err(LocalControlClientError::OutcomeUnknown(
                        request
                            .runtime_namespace_id
                            .expect("consequential mutations are runtime-bound"),
                    ));
                }
                return Err(error);
            }
        }
    }

    fn fail_inflight(
        &mut self,
        request: ProtocolMessage,
        consequential_mutation: bool,
        error: WireError,
    ) -> ClientResult<(ProtocolMessage, ProtocolMessage)> {
        self.wire = None;
        if consequential_mutation {
            self.mutation_outcomes.connection_lost();
            return Err(LocalControlClientError::OutcomeUnknown(
                request
                    .runtime_namespace_id
                    .expect("consequential mutations are runtime-bound"),
            ));
        }
        Err(map_wire_error(error))
    }

    fn queue_event(&mut self, event: ProtocolMessage) -> ClientResult<()> {
        match event.runtime_namespace_id {
            Some(runtime_namespace_id) => {
                if !self.attached_runtimes.contains(&runtime_namespace_id) {
                    return Err(LocalControlClientError::Protocol(
                        LocalControlErrorKind::UnknownRuntime,
                    ));
                }
                validate_event_binding(
                    &self.connection_id,
                    self.owner_generation_id,
                    Some(runtime_namespace_id),
                    &event,
                )
                .map_err(map_protocol_error)?;
            }
            None => {
                validate_event_binding(&self.connection_id, self.owner_generation_id, None, &event)
                    .map_err(map_protocol_error)?;
            }
        }
        let frame_bytes = encode_frame(&event).map_err(map_protocol_error)?.len();
        if self.pending_event_bytes.saturating_add(frame_bytes) > MAX_CLIENT_QUEUED_EVENT_BYTES {
            self.wire = None;
            return Err(LocalControlClientError::ClientEventBackpressure);
        }
        self.pending_event_bytes = self.pending_event_bytes.saturating_add(frame_bytes);
        self.pending_events.push_back(event);
        Ok(())
    }

    fn next_request_sequence(&mut self) -> ClientResult<EventSequence> {
        let current = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(LocalControlClientError::SequenceExhausted)?;
        EventSequence::new(current).map_err(|_| LocalControlClientError::SequenceExhausted)
    }

    #[cfg(test)]
    fn connect_with_wire_for_test(
        wire: Box<dyn LocalControlWire>,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        Self::handshake(wire, expected_owner_generation_id)
    }

    #[cfg(test)]
    fn reconnect_with_wire_for_test(
        &mut self,
        wire: Box<dyn LocalControlWire>,
    ) -> ClientResult<()> {
        self.replace_after_handshake(wire)
    }

    #[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
    fn connect_runtime_directory_for_test(
        runtime_directory: &std::path::Path,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ClientResult<Self> {
        let wire = PlatformWire::connect_runtime_directory(runtime_directory)?;
        Self::handshake(Box::new(wire), expected_owner_generation_id)
    }
}

pub(crate) fn validate_local_endpoint_hint(endpoint_hint: Option<&str>) -> ClientResult<()> {
    match endpoint_hint.map(str::trim) {
        None | Some("") | Some("default") | Some("local") => Ok(()),
        Some(_) => Err(LocalControlClientError::RemoteEndpointRejected),
    }
}

fn map_wire_error(error: WireError) -> LocalControlClientError {
    match error {
        WireError::Transport(message) => LocalControlClientError::Transport(message),
        WireError::Protocol(kind) => map_protocol_error(kind),
    }
}

fn project_response(message: &ProtocolMessage) -> ClientResult<ClientResponseProjection> {
    match &message.payload {
        ProtocolPayload::RuntimeSnapshot { truth } => {
            Ok(ClientResponseProjection::RuntimeSnapshot {
                runtime_namespace_id: message.runtime_namespace_id.ok_or(
                    LocalControlClientError::Protocol(LocalControlErrorKind::UnknownRuntime),
                )?,
                truth: truth.clone(),
            })
        }
        ProtocolPayload::ControlState {
            authority,
            controller_client_id,
        } => Ok(ClientResponseProjection::ControlState {
            runtime_namespace_id: message.runtime_namespace_id.ok_or(
                LocalControlClientError::Protocol(LocalControlErrorKind::UnknownRuntime),
            )?,
            authority: *authority,
            controller_client_id: controller_client_id.clone(),
        }),
        ProtocolPayload::Pong => Ok(ClientResponseProjection::Pong),
        _ => Err(LocalControlClientError::UnexpectedResponse(message.kind())),
    }
}

fn project_event(message: ProtocolMessage) -> ClientResult<ClientEventProjection> {
    let runtime_namespace_id = message.runtime_namespace_id;
    match message.payload {
        ProtocolPayload::RuntimeEvent { event } => Ok(ClientEventProjection::RuntimeEvent {
            runtime_namespace_id: runtime_namespace_id.ok_or(LocalControlClientError::Protocol(
                LocalControlErrorKind::UnknownRuntime,
            ))?,
            event,
        }),
        ProtocolPayload::OutputEvent { chunk } => Ok(ClientEventProjection::Output {
            runtime_namespace_id: runtime_namespace_id.ok_or(LocalControlClientError::Protocol(
                LocalControlErrorKind::UnknownRuntime,
            ))?,
            chunk,
        }),
        ProtocolPayload::HistoryGap {
            first_available_sequence,
            last_dropped_sequence,
        } => Ok(ClientEventProjection::HistoryGap {
            runtime_namespace_id: runtime_namespace_id.ok_or(LocalControlClientError::Protocol(
                LocalControlErrorKind::UnknownRuntime,
            ))?,
            first_available_sequence,
            last_dropped_sequence,
        }),
        ProtocolPayload::OwnerStatus { ready } => Ok(ClientEventProjection::OwnerStatus { ready }),
        _ => Err(LocalControlClientError::UnexpectedResponse(message.kind())),
    }
}

#[cfg(test)]
#[path = "../t155_local_control_client_tests.rs"]
mod t155_local_control_client_tests;

#[cfg(test)]
#[path = "../t157_client_recovery_tests.rs"]
mod t157_client_recovery_tests;
