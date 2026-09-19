use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId,
    RuntimeLifecycleEvent, RuntimeNamespaceId, RuntimeTruth,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Read, Write};

pub(crate) const PROTOCOL_VERSION: u16 = 1;
pub(crate) const MAX_INBOUND_CONTROL_FRAME_BYTES: usize = 256 * 1024;
pub(crate) const MAX_OUTPUT_EVENT_CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const MAX_INPUT_BYTES: usize = 16 * 1024;

pub(crate) type ProtocolResult<T> = Result<T, LocalControlErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum MessageKind {
    Hello,
    HelloAck,
    Ping,
    Pong,
    ListRuntimes,
    RuntimeSnapshot,
    AttachObserver,
    Detach,
    RuntimeEvent,
    OutputEvent,
    HistoryGap,
    OwnerStatus,
    Error,
    ControlState,
    RequestControl,
    ReleaseControl,
    Input,
    Resize,
    Interrupt,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageDirection {
    ClientToOwner,
    OwnerToClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MessageAuthorityClass {
    ConnectionObserverSafe,
    OwnerToClientStateOnly,
    BoundedAuthorityTransition,
    ActiveControllerRevocation,
    ControllerOnly,
}

impl MessageKind {
    pub(crate) fn authority_class(self) -> MessageAuthorityClass {
        match self {
            Self::Hello
            | Self::Ping
            | Self::Pong
            | Self::ListRuntimes
            | Self::RuntimeSnapshot
            | Self::AttachObserver
            | Self::Detach
            | Self::RuntimeEvent
            | Self::OutputEvent
            | Self::HistoryGap
            | Self::OwnerStatus => MessageAuthorityClass::ConnectionObserverSafe,
            Self::HelloAck | Self::Error | Self::ControlState => {
                MessageAuthorityClass::OwnerToClientStateOnly
            }
            Self::RequestControl => MessageAuthorityClass::BoundedAuthorityTransition,
            Self::ReleaseControl => MessageAuthorityClass::ActiveControllerRevocation,
            Self::Input | Self::Resize | Self::Interrupt | Self::Stop => {
                MessageAuthorityClass::ControllerOnly
            }
        }
    }

    pub(crate) fn direction(self) -> MessageDirection {
        match self {
            Self::Hello
            | Self::Ping
            | Self::ListRuntimes
            | Self::AttachObserver
            | Self::Detach
            | Self::RequestControl
            | Self::ReleaseControl
            | Self::Input
            | Self::Resize
            | Self::Interrupt
            | Self::Stop => MessageDirection::ClientToOwner,
            Self::HelloAck
            | Self::Pong
            | Self::RuntimeSnapshot
            | Self::RuntimeEvent
            | Self::OutputEvent
            | Self::HistoryGap
            | Self::OwnerStatus
            | Self::Error
            | Self::ControlState => MessageDirection::OwnerToClient,
        }
    }

    pub(crate) fn is_consequential_mutation(self) -> bool {
        matches!(
            self,
            Self::RequestControl
                | Self::ReleaseControl
                | Self::Input
                | Self::Resize
                | Self::Interrupt
                | Self::Stop
        )
    }

    fn runtime_binding(self) -> RuntimeBinding {
        match self {
            Self::Hello
            | Self::HelloAck
            | Self::Ping
            | Self::Pong
            | Self::ListRuntimes
            | Self::OwnerStatus => RuntimeBinding::Forbidden,
            Self::Error => RuntimeBinding::Optional,
            Self::RuntimeSnapshot
            | Self::AttachObserver
            | Self::Detach
            | Self::RuntimeEvent
            | Self::OutputEvent
            | Self::HistoryGap
            | Self::ControlState
            | Self::RequestControl
            | Self::ReleaseControl
            | Self::Input
            | Self::Resize
            | Self::Interrupt
            | Self::Stop => RuntimeBinding::Required,
        }
    }

    fn correlation_requirement(self) -> CorrelationRequirement {
        match self.direction() {
            MessageDirection::ClientToOwner => CorrelationRequirement::Forbidden,
            MessageDirection::OwnerToClient => match self {
                Self::HelloAck
                | Self::Pong
                | Self::RuntimeSnapshot
                | Self::Error
                | Self::ControlState => CorrelationRequirement::Required,
                Self::RuntimeEvent | Self::OutputEvent | Self::HistoryGap | Self::OwnerStatus => {
                    CorrelationRequirement::Forbidden
                }
                _ => CorrelationRequirement::Forbidden,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeBinding {
    Required,
    Optional,
    Forbidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorrelationRequirement {
    Required,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProtocolPayload {
    Hello {
        minimum_protocol_version: u16,
        maximum_protocol_version: u16,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    },
    HelloAck,
    Ping,
    Pong,
    ListRuntimes,
    RuntimeSnapshot {
        truth: RuntimeTruth,
    },
    AttachObserver,
    Detach,
    RuntimeEvent {
        event: RuntimeLifecycleEvent,
    },
    OutputEvent {
        chunk: Vec<u8>,
    },
    HistoryGap {
        first_available_sequence: EventSequence,
        last_dropped_sequence: EventSequence,
    },
    OwnerStatus {
        ready: bool,
    },
    Error {
        kind: LocalControlErrorKind,
    },
    ControlState {
        authority: ClientAuthority,
        controller_client_id: Option<ClientConnectionId>,
    },
    RequestControl,
    ReleaseControl,
    Input {
        data: Vec<u8>,
    },
    Resize {
        columns: u16,
        rows: u16,
    },
    Interrupt,
    Stop,
}

impl ProtocolPayload {
    pub(crate) fn kind(&self) -> MessageKind {
        match self {
            Self::Hello { .. } => MessageKind::Hello,
            Self::HelloAck => MessageKind::HelloAck,
            Self::Ping => MessageKind::Ping,
            Self::Pong => MessageKind::Pong,
            Self::ListRuntimes => MessageKind::ListRuntimes,
            Self::RuntimeSnapshot { .. } => MessageKind::RuntimeSnapshot,
            Self::AttachObserver => MessageKind::AttachObserver,
            Self::Detach => MessageKind::Detach,
            Self::RuntimeEvent { .. } => MessageKind::RuntimeEvent,
            Self::OutputEvent { .. } => MessageKind::OutputEvent,
            Self::HistoryGap { .. } => MessageKind::HistoryGap,
            Self::OwnerStatus { .. } => MessageKind::OwnerStatus,
            Self::Error { .. } => MessageKind::Error,
            Self::ControlState { .. } => MessageKind::ControlState,
            Self::RequestControl => MessageKind::RequestControl,
            Self::ReleaseControl => MessageKind::ReleaseControl,
            Self::Input { .. } => MessageKind::Input,
            Self::Resize { .. } => MessageKind::Resize,
            Self::Interrupt => MessageKind::Interrupt,
            Self::Stop => MessageKind::Stop,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProtocolMessage {
    pub(crate) connection_id: Option<ClientConnectionId>,
    pub(crate) sequence: EventSequence,
    pub(crate) runtime_namespace_id: Option<RuntimeNamespaceId>,
    pub(crate) owner_generation_id: Option<OwnerGenerationId>,
    pub(crate) correlation_sequence: Option<EventSequence>,
    pub(crate) payload: ProtocolPayload,
}

impl ProtocolMessage {
    pub(crate) fn hello(
        sequence: EventSequence,
        minimum_protocol_version: u16,
        maximum_protocol_version: u16,
        expected_owner_generation_id: Option<OwnerGenerationId>,
    ) -> ProtocolResult<Self> {
        let message = Self {
            connection_id: None,
            sequence,
            runtime_namespace_id: None,
            owner_generation_id: None,
            correlation_sequence: None,
            payload: ProtocolPayload::Hello {
                minimum_protocol_version,
                maximum_protocol_version,
                expected_owner_generation_id,
            },
        };
        validate_message(&message)?;
        Ok(message)
    }

    pub(crate) fn new(
        connection_id: ClientConnectionId,
        sequence: EventSequence,
        runtime_namespace_id: Option<RuntimeNamespaceId>,
        owner_generation_id: OwnerGenerationId,
        correlation_sequence: Option<EventSequence>,
        payload: ProtocolPayload,
    ) -> ProtocolResult<Self> {
        let message = Self {
            connection_id: Some(connection_id),
            sequence,
            runtime_namespace_id,
            owner_generation_id: Some(owner_generation_id),
            correlation_sequence,
            payload,
        };
        validate_message(&message)?;
        Ok(message)
    }

    pub(crate) fn kind(&self) -> MessageKind {
        self.payload.kind()
    }

    pub(crate) fn direction(&self) -> MessageDirection {
        self.kind().direction()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnvelope {
    protocol_version: u16,
    message_kind: MessageKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    connection_id: Option<ClientConnectionId>,
    sequence: EventSequence,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_namespace_id: Option<RuntimeNamespaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_generation_id: Option<OwnerGenerationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_sequence: Option<EventSequence>,
    body: Value,
}

#[derive(Debug, Deserialize)]
struct ProtocolVersionProbe {
    protocol_version: u16,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HelloBody {
    minimum_protocol_version: u16,
    maximum_protocol_version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_owner_generation_id: Option<OwnerGenerationId>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyBody {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeSnapshotBody {
    truth: RuntimeTruth,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeEventBody {
    event: RuntimeLifecycleEvent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputEventBody {
    chunk_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryGapBody {
    first_available_sequence: EventSequence,
    last_dropped_sequence: EventSequence,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerStatusBody {
    ready: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorBody {
    kind: LocalControlErrorKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlStateBody {
    authority: ClientAuthority,
    #[serde(skip_serializing_if = "Option::is_none")]
    controller_client_id: Option<ClientConnectionId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputBody {
    data_hex: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResizeBody {
    columns: u16,
    rows: u16,
}

pub(crate) fn encode_frame(message: &ProtocolMessage) -> ProtocolResult<Vec<u8>> {
    validate_message(message)?;
    let body = encode_body(&message.payload)?;
    let envelope = WireEnvelope {
        protocol_version: PROTOCOL_VERSION,
        message_kind: message.kind(),
        connection_id: message.connection_id.clone(),
        sequence: message.sequence,
        runtime_namespace_id: message.runtime_namespace_id,
        owner_generation_id: message.owner_generation_id,
        correlation_sequence: message.correlation_sequence,
        body,
    };
    let payload =
        serde_json::to_vec(&envelope).map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    if payload.is_empty() {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if payload.len() > MAX_INBOUND_CONTROL_FRAME_BYTES {
        return Err(LocalControlErrorKind::OversizedFrame);
    }
    let length = u32::try_from(payload.len()).map_err(|_| LocalControlErrorKind::OversizedFrame)?;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&length.to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

pub(crate) fn write_frame<W: Write>(
    writer: &mut W,
    message: &ProtocolMessage,
) -> ProtocolResult<()> {
    let frame = encode_frame(message)?;
    writer
        .write_all(&frame)
        .map_err(|_| LocalControlErrorKind::MalformedFrame)
}

pub(crate) fn decode_frame(frame: &[u8]) -> ProtocolResult<ProtocolMessage> {
    let mut cursor = std::io::Cursor::new(frame);
    let message = read_frame(&mut cursor)?;
    if cursor.position() != frame.len() as u64 {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(message)
}

pub(crate) fn read_frame<R: Read>(reader: &mut R) -> ProtocolResult<ProtocolMessage> {
    let mut length_bytes = [0_u8; 4];
    reader
        .read_exact(&mut length_bytes)
        .map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    let claimed = u32::from_le_bytes(length_bytes) as usize;
    if claimed == 0 {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if claimed > MAX_INBOUND_CONTROL_FRAME_BYTES {
        return Err(LocalControlErrorKind::OversizedFrame);
    }

    let mut payload = vec![0_u8; claimed];
    reader
        .read_exact(&mut payload)
        .map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    let text = std::str::from_utf8(&payload).map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    decode_payload(text)
}

fn decode_payload(text: &str) -> ProtocolResult<ProtocolMessage> {
    let probe: ProtocolVersionProbe =
        serde_json::from_str(text).map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    if probe.protocol_version != PROTOCOL_VERSION {
        return Err(LocalControlErrorKind::ProtocolMismatch);
    }

    let envelope: WireEnvelope =
        serde_json::from_str(text).map_err(|_| LocalControlErrorKind::MalformedFrame)?;
    let payload = decode_body(envelope.message_kind, envelope.body)?;
    let message = ProtocolMessage {
        connection_id: envelope.connection_id,
        sequence: envelope.sequence,
        runtime_namespace_id: envelope.runtime_namespace_id,
        owner_generation_id: envelope.owner_generation_id,
        correlation_sequence: envelope.correlation_sequence,
        payload,
    };
    validate_message(&message)?;
    Ok(message)
}

fn encode_body(payload: &ProtocolPayload) -> ProtocolResult<Value> {
    match payload {
        ProtocolPayload::HelloAck
        | ProtocolPayload::Ping
        | ProtocolPayload::Pong
        | ProtocolPayload::ListRuntimes
        | ProtocolPayload::AttachObserver
        | ProtocolPayload::Detach
        | ProtocolPayload::RequestControl
        | ProtocolPayload::ReleaseControl
        | ProtocolPayload::Interrupt
        | ProtocolPayload::Stop => to_value(&EmptyBody::default()),
        ProtocolPayload::Hello {
            minimum_protocol_version,
            maximum_protocol_version,
            expected_owner_generation_id,
        } => to_value(&HelloBody {
            minimum_protocol_version: *minimum_protocol_version,
            maximum_protocol_version: *maximum_protocol_version,
            expected_owner_generation_id: *expected_owner_generation_id,
        }),
        ProtocolPayload::RuntimeSnapshot { truth } => to_value(&RuntimeSnapshotBody {
            truth: truth.clone(),
        }),
        ProtocolPayload::RuntimeEvent { event } => to_value(&RuntimeEventBody {
            event: event.clone(),
        }),
        ProtocolPayload::OutputEvent { chunk } => to_value(&OutputEventBody {
            chunk_hex: encode_hex_bytes(chunk),
        }),
        ProtocolPayload::HistoryGap {
            first_available_sequence,
            last_dropped_sequence,
        } => to_value(&HistoryGapBody {
            first_available_sequence: *first_available_sequence,
            last_dropped_sequence: *last_dropped_sequence,
        }),
        ProtocolPayload::OwnerStatus { ready } => to_value(&OwnerStatusBody { ready: *ready }),
        ProtocolPayload::Error { kind } => to_value(&ErrorBody { kind: *kind }),
        ProtocolPayload::ControlState {
            authority,
            controller_client_id,
        } => to_value(&ControlStateBody {
            authority: *authority,
            controller_client_id: controller_client_id.clone(),
        }),
        ProtocolPayload::Input { data } => to_value(&InputBody {
            data_hex: encode_hex_bytes(data),
        }),
        ProtocolPayload::Resize { columns, rows } => to_value(&ResizeBody {
            columns: *columns,
            rows: *rows,
        }),
    }
}

fn decode_body(kind: MessageKind, body: Value) -> ProtocolResult<ProtocolPayload> {
    match kind {
        MessageKind::Hello => {
            let value: HelloBody = from_value(body)?;
            Ok(ProtocolPayload::Hello {
                minimum_protocol_version: value.minimum_protocol_version,
                maximum_protocol_version: value.maximum_protocol_version,
                expected_owner_generation_id: value.expected_owner_generation_id,
            })
        }
        MessageKind::HelloAck => from_empty(body).map(|_| ProtocolPayload::HelloAck),
        MessageKind::Ping => from_empty(body).map(|_| ProtocolPayload::Ping),
        MessageKind::Pong => from_empty(body).map(|_| ProtocolPayload::Pong),
        MessageKind::ListRuntimes => from_empty(body).map(|_| ProtocolPayload::ListRuntimes),
        MessageKind::RuntimeSnapshot => {
            let value: RuntimeSnapshotBody = from_value(body)?;
            Ok(ProtocolPayload::RuntimeSnapshot { truth: value.truth })
        }
        MessageKind::AttachObserver => from_empty(body).map(|_| ProtocolPayload::AttachObserver),
        MessageKind::Detach => from_empty(body).map(|_| ProtocolPayload::Detach),
        MessageKind::RuntimeEvent => {
            let value: RuntimeEventBody = from_value(body)?;
            Ok(ProtocolPayload::RuntimeEvent { event: value.event })
        }
        MessageKind::OutputEvent => {
            let value: OutputEventBody = from_value(body)?;
            Ok(ProtocolPayload::OutputEvent {
                chunk: decode_hex_bytes(&value.chunk_hex, MAX_OUTPUT_EVENT_CHUNK_BYTES)?,
            })
        }
        MessageKind::HistoryGap => {
            let value: HistoryGapBody = from_value(body)?;
            Ok(ProtocolPayload::HistoryGap {
                first_available_sequence: value.first_available_sequence,
                last_dropped_sequence: value.last_dropped_sequence,
            })
        }
        MessageKind::OwnerStatus => {
            let value: OwnerStatusBody = from_value(body)?;
            Ok(ProtocolPayload::OwnerStatus { ready: value.ready })
        }
        MessageKind::Error => {
            let value: ErrorBody = from_value(body)?;
            Ok(ProtocolPayload::Error { kind: value.kind })
        }
        MessageKind::ControlState => {
            let value: ControlStateBody = from_value(body)?;
            Ok(ProtocolPayload::ControlState {
                authority: value.authority,
                controller_client_id: value.controller_client_id,
            })
        }
        MessageKind::RequestControl => from_empty(body).map(|_| ProtocolPayload::RequestControl),
        MessageKind::ReleaseControl => from_empty(body).map(|_| ProtocolPayload::ReleaseControl),
        MessageKind::Input => {
            let value: InputBody = from_value(body)?;
            Ok(ProtocolPayload::Input {
                data: decode_hex_bytes(&value.data_hex, MAX_INPUT_BYTES)?,
            })
        }
        MessageKind::Resize => {
            let value: ResizeBody = from_value(body)?;
            Ok(ProtocolPayload::Resize {
                columns: value.columns,
                rows: value.rows,
            })
        }
        MessageKind::Interrupt => from_empty(body).map(|_| ProtocolPayload::Interrupt),
        MessageKind::Stop => from_empty(body).map(|_| ProtocolPayload::Stop),
    }
}

fn to_value<T: Serialize>(value: &T) -> ProtocolResult<Value> {
    serde_json::to_value(value).map_err(|_| LocalControlErrorKind::MalformedFrame)
}

fn from_value<T: for<'de> Deserialize<'de>>(value: Value) -> ProtocolResult<T> {
    serde_json::from_value(value).map_err(|_| LocalControlErrorKind::MalformedFrame)
}

fn from_empty(value: Value) -> ProtocolResult<EmptyBody> {
    from_value(value)
}

fn encode_hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_hex_bytes(encoded: &str, max_decoded_bytes: usize) -> ProtocolResult<Vec<u8>> {
    if encoded.len() > max_decoded_bytes * 2 {
        return Err(LocalControlErrorKind::OversizedFrame);
    }
    if !encoded.len().is_multiple_of(2) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = decode_hex_nibble(pair[0]).ok_or(LocalControlErrorKind::MalformedFrame)?;
        let low = decode_hex_nibble(pair[1]).ok_or(LocalControlErrorKind::MalformedFrame)?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn decode_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn validate_message(message: &ProtocolMessage) -> ProtocolResult<()> {
    let kind = message.kind();
    match kind {
        MessageKind::Hello => {
            if message.connection_id.is_some() || message.owner_generation_id.is_some() {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
        }
        _ => {
            if message.connection_id.is_none() || message.owner_generation_id.is_none() {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
        }
    }
    match (kind.runtime_binding(), message.runtime_namespace_id) {
        (RuntimeBinding::Required, None) | (RuntimeBinding::Forbidden, Some(_)) => {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        _ => {}
    }
    match (kind.correlation_requirement(), message.correlation_sequence) {
        (CorrelationRequirement::Required, None) | (CorrelationRequirement::Forbidden, Some(_)) => {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        _ => {}
    }

    match &message.payload {
        ProtocolPayload::Hello {
            minimum_protocol_version,
            maximum_protocol_version,
            ..
        } => {
            if *minimum_protocol_version == 0
                || minimum_protocol_version > maximum_protocol_version
                || PROTOCOL_VERSION < *minimum_protocol_version
                || PROTOCOL_VERSION > *maximum_protocol_version
            {
                return Err(LocalControlErrorKind::ProtocolMismatch);
            }
        }
        ProtocolPayload::RuntimeEvent { event } => {
            if Some(event.runtime_namespace_id) != message.runtime_namespace_id
                || event.sequence != message.sequence
            {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            if Some(event.owner_generation_id) != message.owner_generation_id {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::OutputEvent { chunk } => {
            if chunk.len() > MAX_OUTPUT_EVENT_CHUNK_BYTES {
                return Err(LocalControlErrorKind::OversizedFrame);
            }
        }
        ProtocolPayload::Input { data } => {
            if data.is_empty() {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            if data.len() > MAX_INPUT_BYTES {
                return Err(LocalControlErrorKind::OversizedFrame);
            }
        }
        ProtocolPayload::HistoryGap {
            first_available_sequence,
            last_dropped_sequence,
        } => {
            if first_available_sequence.get() <= last_dropped_sequence.get() {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
        }
        ProtocolPayload::ControlState {
            authority,
            controller_client_id,
        } => match (*authority, controller_client_id) {
            (ClientAuthority::Controller, None) | (ClientAuthority::Observer, Some(_)) => {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            _ => {}
        },
        ProtocolPayload::Resize { columns, rows } if *columns == 0 || *rows == 0 => {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn validate_response_binding(
    request: &ProtocolMessage,
    response: &ProtocolMessage,
) -> ProtocolResult<()> {
    if request.direction() != MessageDirection::ClientToOwner
        || response.direction() != MessageDirection::OwnerToClient
    {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if !response_kind_is_valid_for_request(request.kind(), response.kind()) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if response.correlation_sequence != Some(request.sequence) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if request.kind() == MessageKind::Hello {
        if response.connection_id.is_none() || response.owner_generation_id.is_none() {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if let ProtocolPayload::Hello {
            expected_owner_generation_id: Some(expected),
            ..
        } = &request.payload
            && response.owner_generation_id != Some(*expected)
        {
            return Err(LocalControlErrorKind::StaleOwnerGeneration);
        }
    } else {
        if response.connection_id != request.connection_id {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if response.owner_generation_id != request.owner_generation_id {
            return Err(LocalControlErrorKind::StaleOwnerGeneration);
        }
    }
    match (request.runtime_namespace_id, response.runtime_namespace_id) {
        (Some(expected), Some(actual)) if expected == actual => {}
        (Some(_), _) => return Err(LocalControlErrorKind::UnknownRuntime),
        (None, Some(_)) if response.kind() == MessageKind::RuntimeSnapshot => {}
        (None, None) => {}
        (None, Some(_)) => return Err(LocalControlErrorKind::MalformedFrame),
    }
    Ok(())
}

fn response_kind_is_valid_for_request(request: MessageKind, response: MessageKind) -> bool {
    if response == MessageKind::Error {
        return true;
    }
    matches!(
        (request, response),
        (MessageKind::Hello, MessageKind::HelloAck)
            | (MessageKind::Ping, MessageKind::Pong)
            | (MessageKind::ListRuntimes, MessageKind::RuntimeSnapshot)
            | (MessageKind::AttachObserver, MessageKind::RuntimeSnapshot)
            | (MessageKind::AttachObserver, MessageKind::ControlState)
            | (MessageKind::Detach, MessageKind::ControlState)
            | (MessageKind::RequestControl, MessageKind::ControlState)
            | (MessageKind::ReleaseControl, MessageKind::ControlState)
            | (MessageKind::Input, MessageKind::ControlState)
            | (MessageKind::Resize, MessageKind::ControlState)
            | (MessageKind::Interrupt, MessageKind::ControlState)
            | (MessageKind::Stop, MessageKind::ControlState)
    )
}

pub(crate) fn validate_event_binding(
    expected_connection_id: &ClientConnectionId,
    expected_owner_generation_id: OwnerGenerationId,
    expected_runtime_namespace_id: Option<RuntimeNamespaceId>,
    event: &ProtocolMessage,
) -> ProtocolResult<()> {
    if event.direction() != MessageDirection::OwnerToClient || event.correlation_sequence.is_some()
    {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if !matches!(
        event.kind(),
        MessageKind::RuntimeEvent
            | MessageKind::OutputEvent
            | MessageKind::HistoryGap
            | MessageKind::OwnerStatus
    ) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if event.connection_id.as_ref() != Some(expected_connection_id) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    if event.owner_generation_id != Some(expected_owner_generation_id) {
        return Err(LocalControlErrorKind::StaleOwnerGeneration);
    }
    if event.runtime_namespace_id != expected_runtime_namespace_id {
        return Err(LocalControlErrorKind::UnknownRuntime);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RequestSequenceGuard {
    connection_id: ClientConnectionId,
    owner_generation_id: OwnerGenerationId,
    last_accepted_sequence: Option<EventSequence>,
}

impl RequestSequenceGuard {
    pub(crate) fn new(
        connection_id: ClientConnectionId,
        owner_generation_id: OwnerGenerationId,
    ) -> Self {
        Self {
            connection_id,
            owner_generation_id,
            last_accepted_sequence: None,
        }
    }

    pub(crate) fn accept(&mut self, message: &ProtocolMessage) -> ProtocolResult<()> {
        if message.direction() != MessageDirection::ClientToOwner {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if message.connection_id.as_ref() != Some(&self.connection_id) {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if message.owner_generation_id != Some(self.owner_generation_id) {
            return Err(LocalControlErrorKind::StaleOwnerGeneration);
        }
        if self
            .last_accepted_sequence
            .is_some_and(|last| message.sequence.get() <= last.get())
        {
            return Err(LocalControlErrorKind::DuplicateOrOutOfOrderRequest);
        }
        self.last_accepted_sequence = Some(message.sequence);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingMutation {
    request_kind: MessageKind,
    sequence: EventSequence,
    connection_id: ClientConnectionId,
    owner_generation_id: OwnerGenerationId,
    runtime_namespace_id: RuntimeNamespaceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MutationOutcomeTracker {
    pending: Option<PendingMutation>,
    uncertain_runtime_namespace_id: Option<RuntimeNamespaceId>,
}

impl MutationOutcomeTracker {
    pub(crate) fn new() -> Self {
        Self {
            pending: None,
            uncertain_runtime_namespace_id: None,
        }
    }

    pub(crate) fn begin(&mut self, message: &ProtocolMessage) -> ProtocolResult<()> {
        if !message.kind().is_consequential_mutation()
            || message.direction() != MessageDirection::ClientToOwner
        {
            return Err(LocalControlErrorKind::UnsupportedOperation);
        }
        if self.uncertain_runtime_namespace_id.is_some() || self.pending.is_some() {
            return Err(LocalControlErrorKind::OutcomeUnknown);
        }
        let runtime_namespace_id = message
            .runtime_namespace_id
            .ok_or(LocalControlErrorKind::UnknownRuntime)?;
        self.pending = Some(PendingMutation {
            request_kind: message.kind(),
            sequence: message.sequence,
            connection_id: message
                .connection_id
                .clone()
                .ok_or(LocalControlErrorKind::MalformedFrame)?,
            owner_generation_id: message
                .owner_generation_id
                .ok_or(LocalControlErrorKind::MalformedFrame)?,
            runtime_namespace_id,
        });
        Ok(())
    }

    pub(crate) fn acknowledge(&mut self, response: &ProtocolMessage) -> ProtocolResult<()> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(LocalControlErrorKind::MalformedFrame)?;
        if response.direction() != MessageDirection::OwnerToClient
            || !response_kind_is_valid_for_request(pending.request_kind, response.kind())
            || response.correlation_sequence != Some(pending.sequence)
            || response.connection_id.as_ref() != Some(&pending.connection_id)
        {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if response.owner_generation_id != Some(pending.owner_generation_id) {
            return Err(LocalControlErrorKind::StaleOwnerGeneration);
        }
        if response.runtime_namespace_id != Some(pending.runtime_namespace_id) {
            return Err(LocalControlErrorKind::UnknownRuntime);
        }
        if matches!(
            response.payload,
            ProtocolPayload::Error {
                kind: LocalControlErrorKind::OutcomeUnknown
            }
        ) {
            let runtime_namespace_id = pending.runtime_namespace_id;
            self.pending = None;
            self.uncertain_runtime_namespace_id = Some(runtime_namespace_id);
            return Err(LocalControlErrorKind::OutcomeUnknown);
        }
        self.pending = None;
        Ok(())
    }

    pub(crate) fn connection_lost(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.uncertain_runtime_namespace_id = Some(pending.runtime_namespace_id);
        }
    }

    pub(crate) fn reconcile_state(
        &mut self,
        request: &ProtocolMessage,
        response: &ProtocolMessage,
    ) -> ProtocolResult<()> {
        let uncertain_runtime_namespace_id = self
            .uncertain_runtime_namespace_id
            .ok_or(LocalControlErrorKind::MalformedFrame)?;

        let valid_state_query = matches!(
            (request.kind(), response.kind()),
            (MessageKind::ListRuntimes, MessageKind::RuntimeSnapshot)
                | (MessageKind::AttachObserver, MessageKind::RuntimeSnapshot)
                | (MessageKind::AttachObserver, MessageKind::ControlState)
        );
        if !valid_state_query {
            return Err(LocalControlErrorKind::UnsupportedOperation);
        }
        validate_response_binding(request, response)?;
        if response.runtime_namespace_id != Some(uncertain_runtime_namespace_id) {
            return Err(LocalControlErrorKind::UnknownRuntime);
        }

        self.uncertain_runtime_namespace_id = None;
        Ok(())
    }

    pub(crate) fn is_outcome_unknown(&self) -> bool {
        self.uncertain_runtime_namespace_id.is_some()
    }
}

#[cfg(test)]
#[path = "../t148_persistent_runtime_protocol_tests.rs"]
mod t148_persistent_runtime_protocol_tests;
