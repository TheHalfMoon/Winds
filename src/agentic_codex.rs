use serde_json::{Map, Value, json};
use std::error::Error;
use std::fmt;

pub(super) const MAX_CODEX_JSONL_FRAME_BYTES: usize = 64 * 1024;
const MAX_NATIVE_ID_BYTES: usize = 1024;
const INITIALIZE_REQUEST_ID: u64 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CodexProtocolError {
    AlreadyStarted,
    HandshakeIncomplete,
    InitializeResponseIdMismatch,
    InitializeRejected,
    InitializedOutOfOrder,
    MalformedFrame,
    OversizedFrame,
    InvalidNativeThreadId,
    ServerExitedDuringHandshake,
    ServerExited,
    ClientFailed,
}

impl fmt::Display for CodexProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::AlreadyStarted => "Codex protocol initialization has already started",
            Self::HandshakeIncomplete => "Codex protocol handshake is incomplete",
            Self::InitializeResponseIdMismatch => {
                "Codex initialize response did not match the initialize request id"
            }
            Self::InitializeRejected => "Codex initialize request was rejected",
            Self::InitializedOutOfOrder => {
                "Codex initialized notification requires a successful initialize response first"
            }
            Self::MalformedFrame => "Codex JSONL frame is malformed or structurally invalid",
            Self::OversizedFrame => "Codex JSONL frame exceeds the bounded frame limit",
            Self::InvalidNativeThreadId => "Codex native thread id is invalid",
            Self::ServerExitedDuringHandshake => {
                "Codex fake server exited before the required handshake completed"
            }
            Self::ServerExited => "Codex fake server exited",
            Self::ClientFailed => "Codex protocol client is already in a failed state",
        };
        formatter.write_str(message)
    }
}

impl Error for CodexProtocolError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HandshakeState {
    Fresh,
    InitializeSent,
    InitializeAccepted,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RpcId {
    Number(u64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EvidenceClass {
    AgentRuntimeEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ServerRequestDisposition {
    RequiresExternalDecision,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum CodexInbound {
    InitializeAccepted,
    Response {
        id: RpcId,
        result: Value,
        evidence: EvidenceClass,
    },
    Notification {
        method: String,
        params: Value,
        evidence: EvidenceClass,
    },
    ServerRequest {
        id: RpcId,
        method: String,
        params: Value,
        disposition: ServerRequestDisposition,
        evidence: EvidenceClass,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NativeThreadId(String);

impl NativeThreadId {
    pub(super) fn parse(value: &str) -> Result<Self, CodexProtocolError> {
        validate_native_thread_id(value)?;
        Ok(Self(value.to_owned()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn from_thread_result(result: &Value) -> Result<Self, CodexProtocolError> {
        let thread_id = result
            .get("thread")
            .and_then(|thread| thread.get("id"))
            .and_then(Value::as_str)
            .ok_or(CodexProtocolError::MalformedFrame)?;
        Self::parse(thread_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProcessOwnership {
    ProvenOwnedChild,
    Unproven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CleanupDecision {
    TerminateProvenOwnedChild,
    PreserveUnprovenProcess,
}

pub(super) fn cleanup_decision(ownership: ProcessOwnership) -> CleanupDecision {
    match ownership {
        ProcessOwnership::ProvenOwnedChild => CleanupDecision::TerminateProvenOwnedChild,
        ProcessOwnership::Unproven => CleanupDecision::PreserveUnprovenProcess,
    }
}

#[derive(Debug)]
pub(super) struct CodexProtocolClient {
    state: HandshakeState,
    next_request_id: u64,
}

impl Default for CodexProtocolClient {
    fn default() -> Self {
        Self {
            state: HandshakeState::Fresh,
            next_request_id: 1,
        }
    }
}

impl CodexProtocolClient {
    pub(super) fn initialize_request(
        &mut self,
        client_name: &str,
        client_title: &str,
        client_version: &str,
    ) -> Result<String, CodexProtocolError> {
        if self.state != HandshakeState::Fresh {
            return Err(CodexProtocolError::AlreadyStarted);
        }
        validate_nonempty_exact(client_name)?;
        validate_nonempty_exact(client_title)?;
        validate_nonempty_exact(client_version)?;

        self.state = HandshakeState::InitializeSent;
        encode_jsonl(&json!({
            "method": "initialize",
            "id": INITIALIZE_REQUEST_ID,
            "params": {
                "clientInfo": {
                    "name": client_name,
                    "title": client_title,
                    "version": client_version
                }
            }
        }))
    }

    pub(super) fn initialized_notification(&mut self) -> Result<String, CodexProtocolError> {
        if self.state != HandshakeState::InitializeAccepted {
            return Err(CodexProtocolError::InitializedOutOfOrder);
        }
        let line = encode_jsonl(&json!({ "method": "initialized", "params": {} }))?;
        self.state = HandshakeState::Ready;
        Ok(line)
    }

    pub(super) fn thread_start(&mut self) -> Result<String, CodexProtocolError> {
        self.thread_request("thread/start", json!({}))
    }

    pub(super) fn thread_resume(
        &mut self,
        native_thread_id: &NativeThreadId,
    ) -> Result<String, CodexProtocolError> {
        self.thread_request(
            "thread/resume",
            json!({ "threadId": native_thread_id.as_str() }),
        )
    }

    pub(super) fn thread_fork(
        &mut self,
        native_thread_id: &NativeThreadId,
    ) -> Result<String, CodexProtocolError> {
        self.thread_request(
            "thread/fork",
            json!({ "threadId": native_thread_id.as_str() }),
        )
    }

    pub(super) fn ingest_jsonl_frame(
        &mut self,
        frame: &[u8],
    ) -> Result<CodexInbound, CodexProtocolError> {
        if self.state == HandshakeState::Failed {
            return Err(CodexProtocolError::ClientFailed);
        }
        if frame.len() > MAX_CODEX_JSONL_FRAME_BYTES {
            return self.fail(CodexProtocolError::OversizedFrame);
        }

        let frame = strip_jsonl_terminator(frame);
        if frame.is_empty() || frame.iter().any(|byte| matches!(byte, b'\n' | b'\r')) {
            return self.fail(CodexProtocolError::MalformedFrame);
        }
        let value: Value = serde_json::from_slice(frame)
            .map_err(|_| CodexProtocolError::MalformedFrame)
            .or_else(|error| self.fail(error))?;
        let object = value
            .as_object()
            .ok_or(CodexProtocolError::MalformedFrame)
            .or_else(|error| self.fail(error))?;

        let id = object.get("id").map(parse_rpc_id).transpose().or_else(|error| self.fail(error))?;
        let method = object.get("method").map(Value::as_str).transpose().ok_or(CodexProtocolError::MalformedFrame).or_else(|error| self.fail(error))?;

        match (id, method) {
            (Some(id), None) => self.ingest_response(id, object),
            (Some(id), Some(method)) => self.ingest_server_request(id, method, object),
            (None, Some(method)) => self.ingest_notification(method, object),
            (None, None) => self.fail(CodexProtocolError::MalformedFrame),
        }
    }

    pub(super) fn on_server_eof(&mut self) -> Result<(), CodexProtocolError> {
        let error = if self.state == HandshakeState::Ready {
            CodexProtocolError::ServerExited
        } else {
            CodexProtocolError::ServerExitedDuringHandshake
        };
        self.fail(error)
    }

    pub(super) fn is_ready(&self) -> bool {
        self.state == HandshakeState::Ready
    }

    fn thread_request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<String, CodexProtocolError> {
        if self.state != HandshakeState::Ready {
            return Err(CodexProtocolError::HandshakeIncomplete);
        }
        let id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(CodexProtocolError::MalformedFrame)?;
        encode_jsonl(&json!({ "method": method, "id": id, "params": params }))
    }

    fn ingest_response(
        &mut self,
        id: RpcId,
        object: &Map<String, Value>,
    ) -> Result<CodexInbound, CodexProtocolError> {
        let result = object.get("result");
        let error = object.get("error");
        if result.is_some() == error.is_some() {
            return self.fail(CodexProtocolError::MalformedFrame);
        }

        match self.state {
            HandshakeState::InitializeSent => {
                if id != RpcId::Number(INITIALIZE_REQUEST_ID) {
                    return self.fail(CodexProtocolError::InitializeResponseIdMismatch);
                }
                if error.is_some() {
                    return self.fail(CodexProtocolError::InitializeRejected);
                }
                self.state = HandshakeState::InitializeAccepted;
                Ok(CodexInbound::InitializeAccepted)
            }
            HandshakeState::Ready => {
                if error.is_some() {
                    return Ok(CodexInbound::Response {
                        id,
                        result: json!({ "error": error.cloned().unwrap_or(Value::Null) }),
                        evidence: EvidenceClass::AgentRuntimeEvidence,
                    });
                }
                Ok(CodexInbound::Response {
                    id,
                    result: result.cloned().unwrap_or(Value::Null),
                    evidence: EvidenceClass::AgentRuntimeEvidence,
                })
            }
            _ => self.fail(CodexProtocolError::MalformedFrame),
        }
    }

    fn ingest_notification(
        &mut self,
        method: &str,
        object: &Map<String, Value>,
    ) -> Result<CodexInbound, CodexProtocolError> {
        if self.state != HandshakeState::Ready {
            return self.fail(CodexProtocolError::HandshakeIncomplete);
        }
        validate_method(method).or_else(|error| self.fail(error))?;
        Ok(CodexInbound::Notification {
            method: method.to_owned(),
            params: object.get("params").cloned().unwrap_or_else(|| json!({})),
            evidence: EvidenceClass::AgentRuntimeEvidence,
        })
    }

    fn ingest_server_request(
        &mut self,
        id: RpcId,
        method: &str,
        object: &Map<String, Value>,
    ) -> Result<CodexInbound, CodexProtocolError> {
        if self.state != HandshakeState::Ready {
            return self.fail(CodexProtocolError::HandshakeIncomplete);
        }
        validate_method(method).or_else(|error| self.fail(error))?;
        Ok(CodexInbound::ServerRequest {
            id,
            method: method.to_owned(),
            params: object.get("params").cloned().unwrap_or_else(|| json!({})),
            disposition: ServerRequestDisposition::RequiresExternalDecision,
            evidence: EvidenceClass::AgentRuntimeEvidence,
        })
    }

    fn fail<T>(&mut self, error: CodexProtocolError) -> Result<T, CodexProtocolError> {
        self.state = HandshakeState::Failed;
        Err(error)
    }
}

fn parse_rpc_id(value: &Value) -> Result<RpcId, CodexProtocolError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .map(RpcId::Number)
            .ok_or(CodexProtocolError::MalformedFrame),
        Value::String(text) if !text.is_empty() && text.len() <= MAX_NATIVE_ID_BYTES => {
            Ok(RpcId::Text(text.clone()))
        }
        _ => Err(CodexProtocolError::MalformedFrame),
    }
}

fn validate_native_thread_id(value: &str) -> Result<(), CodexProtocolError> {
    if value.is_empty()
        || value.len() > MAX_NATIVE_ID_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(CodexProtocolError::InvalidNativeThreadId);
    }
    Ok(())
}

fn validate_nonempty_exact(value: &str) -> Result<(), CodexProtocolError> {
    if value.is_empty() || value.len() > MAX_NATIVE_ID_BYTES || value.chars().any(char::is_control) {
        return Err(CodexProtocolError::MalformedFrame);
    }
    Ok(())
}

fn validate_method(method: &str) -> Result<(), CodexProtocolError> {
    if method.is_empty() || method.len() > MAX_NATIVE_ID_BYTES || method.chars().any(char::is_control)
    {
        return Err(CodexProtocolError::MalformedFrame);
    }
    Ok(())
}

fn strip_jsonl_terminator(frame: &[u8]) -> &[u8] {
    let without_lf = frame.strip_suffix(b"\n").unwrap_or(frame);
    without_lf.strip_suffix(b"\r").unwrap_or(without_lf)
}

fn encode_jsonl(value: &Value) -> Result<String, CodexProtocolError> {
    let mut encoded = serde_json::to_string(value).map_err(|_| CodexProtocolError::MalformedFrame)?;
    if encoded.len() + 1 > MAX_CODEX_JSONL_FRAME_BYTES {
        return Err(CodexProtocolError::OversizedFrame);
    }
    encoded.push('\n');
    Ok(encoded)
}
