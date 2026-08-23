#[cfg(test)]
#[path = "t079_codex_connected_tests.rs"]
mod t079_codex_connected_tests;

use serde_json::{Map, Value, json};
use std::error::Error;
use std::fmt;

pub(super) const MAX_CODEX_JSONL_FRAME_BYTES: usize = 64 * 1024;
const MAX_PROTOCOL_TEXT_BYTES: usize = 1024;
const INITIALIZE_REQUEST_ID: u64 = 0;
#[cfg(test)]
pub(super) const T079_PROOF_PROMPT: &str = "Return only JSON matching the supplied schema with status WINDS_T079_OK. Do not run commands, use tools, modify files, request permissions, or access workspace contents.";

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
    #[cfg(test)]
    UnexpectedT079Notification,
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
            #[cfg(test)]
            Self::UnexpectedT079Notification => {
                "T079 received a notification outside its exact phase-bound allowlist"
            }
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

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum T079RequestKind {
    ConfigRead,
    ThreadStart,
    TurnStart,
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
    ErrorResponse {
        id: RpcId,
        error: Value,
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
    #[cfg(test)]
    t079_mode: bool,
    #[cfg(test)]
    t079_requests: Vec<(u64, T079RequestKind)>,
    #[cfg(test)]
    t079_thread_id: Option<String>,
    #[cfg(test)]
    t079_turn_id: Option<String>,
}

impl Default for CodexProtocolClient {
    fn default() -> Self {
        Self {
            state: HandshakeState::Fresh,
            next_request_id: 1,
            #[cfg(test)]
            t079_mode: false,
            #[cfg(test)]
            t079_requests: Vec::new(),
            #[cfg(test)]
            t079_thread_id: None,
            #[cfg(test)]
            t079_turn_id: None,
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
        self.initialize_request_with_experimental_api(
            client_name,
            client_title,
            client_version,
            false,
        )
    }

    #[cfg(test)]
    /// T079 opts into the experimental API only to send explicit empty environment/tool roots.
    pub(super) fn t079_initialize_request(
        &mut self,
        client_name: &str,
        client_title: &str,
        client_version: &str,
    ) -> Result<String, CodexProtocolError> {
        let request = self.initialize_request_with_experimental_api(
            client_name,
            client_title,
            client_version,
            true,
        )?;
        self.t079_mode = true;
        Ok(request)
    }

    fn initialize_request_with_experimental_api(
        &mut self,
        client_name: &str,
        client_title: &str,
        client_version: &str,
        experimental_api: bool,
    ) -> Result<String, CodexProtocolError> {
        match self.state {
            HandshakeState::Fresh => {}
            HandshakeState::Failed => return Err(CodexProtocolError::ClientFailed),
            _ => return Err(CodexProtocolError::AlreadyStarted),
        }
        validate_nonempty_exact(client_name)?;
        validate_nonempty_exact(client_title)?;
        validate_nonempty_exact(client_version)?;

        let params = if experimental_api {
            json!({
                "clientInfo": {
                    "name": client_name,
                    "title": client_title,
                    "version": client_version
                },
                "capabilities": {
                    "experimentalApi": true
                }
            })
        } else {
            json!({
                "clientInfo": {
                    "name": client_name,
                    "title": client_title,
                    "version": client_version
                }
            })
        };
        let line = encode_jsonl(&json!({
            "method": "initialize",
            "id": INITIALIZE_REQUEST_ID,
            "params": params
        }))?;
        self.state = HandshakeState::InitializeSent;
        Ok(line)
    }

    pub(super) fn initialized_notification(&mut self) -> Result<String, CodexProtocolError> {
        match self.state {
            HandshakeState::InitializeAccepted => {}
            HandshakeState::Failed => return Err(CodexProtocolError::ClientFailed),
            _ => return Err(CodexProtocolError::InitializedOutOfOrder),
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

    #[cfg(test)]
    /// Builds the read-only T079 effective-config preflight after the mandatory handshake.
    pub(super) fn t079_config_read(
        &mut self,
        cwd: &str,
    ) -> Result<(u64, String), CodexProtocolError> {
        validate_nonempty_exact(cwd)?;
        self.t079_request(
            T079RequestKind::ConfigRead,
            "config/read",
            json!({ "cwd": cwd, "includeLayers": false }),
        )
    }

    #[cfg(test)]
    /// Builds the only fresh thread shape accepted by the bounded T079 connected proof.
    pub(super) fn t079_thread_start(
        &mut self,
        cwd: &str,
    ) -> Result<(u64, String), CodexProtocolError> {
        validate_nonempty_exact(cwd)?;
        self.t079_request(
            T079RequestKind::ThreadStart,
            "thread/start",
            json!({
                "cwd": cwd,
                "runtimeWorkspaceRoots": [],
                "approvalPolicy": "never",
                "sandbox": "read-only",
                "ephemeral": true,
                "environments": [],
                "dynamicTools": [],
                "selectedCapabilityRoots": []
            }),
        )
    }

    #[cfg(test)]
    /// Builds the single fixed T079 turn. The caller cannot inject a model, prompt, tool, or policy.
    pub(super) fn t079_turn_start(
        &mut self,
        native_thread_id: &NativeThreadId,
        cwd: &str,
    ) -> Result<(u64, String), CodexProtocolError> {
        validate_nonempty_exact(cwd)?;
        self.t079_request(
            T079RequestKind::TurnStart,
            "turn/start",
            json!({
                "threadId": native_thread_id.as_str(),
                "input": [{ "type": "text", "text": T079_PROOF_PROMPT }],
                "cwd": cwd,
                "runtimeWorkspaceRoots": [],
                "environments": [],
                "approvalPolicy": "never",
                "sandboxPolicy": {
                    "type": "readOnly",
                    "networkAccess": false
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "const": "WINDS_T079_OK" }
                    },
                    "required": ["status"],
                    "additionalProperties": false
                }
            }),
        )
    }

    #[cfg(test)]
    /// Produces a denial response for command/file approval requests. It never grants authority.
    pub(super) fn t079_decline(&self, id: &RpcId) -> Result<String, CodexProtocolError> {
        encode_jsonl(&json!({
            "id": rpc_id_value(id),
            "result": { "decision": "decline" }
        }))
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

        let value: Value = match serde_json::from_slice(frame) {
            Ok(value) => value,
            Err(_) => return self.fail(CodexProtocolError::MalformedFrame),
        };
        let Some(object) = value.as_object() else {
            return self.fail(CodexProtocolError::MalformedFrame);
        };
        let id = match object.get("id") {
            Some(value) => Some(parse_rpc_id(value).or_else(|error| self.fail(error))?),
            None => None,
        };
        let method = match object.get("method") {
            Some(Value::String(method)) => Some(method.as_str()),
            Some(_) => return self.fail(CodexProtocolError::MalformedFrame),
            None => None,
        };
        let has_response_payload = object.contains_key("result") || object.contains_key("error");

        match (id, method, has_response_payload) {
            (Some(id), None, _) => self.ingest_response(id, object),
            (Some(id), Some(method), false) => self.ingest_server_request(id, method, object),
            (None, Some(method), false) => self.ingest_notification(method, object),
            _ => self.fail(CodexProtocolError::MalformedFrame),
        }
    }

    pub(super) fn on_server_eof(&mut self) -> Result<(), CodexProtocolError> {
        if self.state == HandshakeState::Failed {
            return Err(CodexProtocolError::ClientFailed);
        }
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
        self.request(method, params).map(|(_, line)| line)
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<(u64, String), CodexProtocolError> {
        match self.state {
            HandshakeState::Ready => {}
            HandshakeState::Failed => return Err(CodexProtocolError::ClientFailed),
            _ => return Err(CodexProtocolError::HandshakeIncomplete),
        }
        validate_method(method)?;
        let id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(CodexProtocolError::MalformedFrame)?;
        let line = encode_jsonl(&json!({ "method": method, "id": id, "params": params }))?;
        Ok((id, line))
    }

    #[cfg(test)]
    fn t079_request(
        &mut self,
        kind: T079RequestKind,
        method: &str,
        params: Value,
    ) -> Result<(u64, String), CodexProtocolError> {
        let (id, line) = self.request(method, params)?;
        self.t079_requests.push((id, kind));
        Ok((id, line))
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
            HandshakeState::Ready => match (result, error) {
                (Some(result), None) => {
                    #[cfg(test)]
                    if self.t079_mode
                        && let Err(error) = self.record_t079_response(&id, result)
                    {
                        return self.fail(error);
                    }
                    Ok(CodexInbound::Response {
                        id,
                        result: result.clone(),
                        evidence: EvidenceClass::AgentRuntimeEvidence,
                    })
                }
                (None, Some(error)) => {
                    #[cfg(test)]
                    if self.t079_mode
                        && let Err(record_error) = self.record_t079_error(&id)
                    {
                        return self.fail(record_error);
                    }
                    Ok(CodexInbound::ErrorResponse {
                        id,
                        error: error.clone(),
                        evidence: EvidenceClass::AgentRuntimeEvidence,
                    })
                }
                _ => self.fail(CodexProtocolError::MalformedFrame),
            },
            _ => self.fail(CodexProtocolError::MalformedFrame),
        }
    }

    #[cfg(test)]
    fn record_t079_response(
        &mut self,
        id: &RpcId,
        result: &Value,
    ) -> Result<(), CodexProtocolError> {
        let RpcId::Number(id) = id else {
            return Err(CodexProtocolError::MalformedFrame);
        };
        let Some(index) = self
            .t079_requests
            .iter()
            .position(|(request_id, _)| request_id == id)
        else {
            return Err(CodexProtocolError::MalformedFrame);
        };
        let (_, kind) = self.t079_requests.remove(index);

        match kind {
            T079RequestKind::ConfigRead => Ok(()),
            T079RequestKind::ThreadStart => {
                let thread_id = result
                    .get("thread")
                    .and_then(|thread| thread.get("id"))
                    .and_then(Value::as_str)
                    .ok_or(CodexProtocolError::MalformedFrame)?;
                validate_native_thread_id(thread_id)?;
                match self.t079_thread_id.as_deref() {
                    Some(bound_thread_id) if bound_thread_id != thread_id => {
                        return Err(CodexProtocolError::MalformedFrame);
                    }
                    Some(_) => {}
                    None => self.t079_thread_id = Some(thread_id.to_owned()),
                }
                Ok(())
            }
            T079RequestKind::TurnStart => {
                let turn_id = result
                    .get("turn")
                    .and_then(|turn| turn.get("id"))
                    .and_then(Value::as_str)
                    .ok_or(CodexProtocolError::MalformedFrame)?;
                validate_nonempty_exact(turn_id)?;
                match self.t079_turn_id.as_deref() {
                    Some(bound_turn_id) if bound_turn_id != turn_id => {
                        return Err(CodexProtocolError::MalformedFrame);
                    }
                    Some(_) => {}
                    None => self.t079_turn_id = Some(turn_id.to_owned()),
                }
                Ok(())
            }
        }
    }

    #[cfg(test)]
    fn record_t079_error(&mut self, id: &RpcId) -> Result<(), CodexProtocolError> {
        let RpcId::Number(id) = id else {
            return Err(CodexProtocolError::MalformedFrame);
        };
        let Some(index) = self
            .t079_requests
            .iter()
            .position(|(request_id, _)| request_id == id)
        else {
            return Err(CodexProtocolError::MalformedFrame);
        };
        let (_, kind) = self.t079_requests.remove(index);
        match kind {
            T079RequestKind::ConfigRead => {}
            T079RequestKind::ThreadStart => {
                self.t079_thread_id = None;
                self.t079_turn_id = None;
            }
            T079RequestKind::TurnStart => {
                self.t079_turn_id = None;
            }
        }
        Ok(())
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
        let params = protocol_params(object).or_else(|error| self.fail(error))?;
        #[cfg(test)]
        if self.t079_mode && !self.t079_notification_allowed(method, &params) {
            return self.fail(CodexProtocolError::UnexpectedT079Notification);
        }
        Ok(CodexInbound::Notification {
            method: method.to_owned(),
            params,
            evidence: EvidenceClass::AgentRuntimeEvidence,
        })
    }

    #[cfg(test)]
    fn t079_notification_allowed(&mut self, method: &str, params: &Value) -> bool {
        let Some(params) = params.as_object() else {
            return false;
        };

        if method == "thread/started" {
            if !exact_object_keys(params, &["thread"]) {
                return false;
            }
            let Some(thread) = params.get("thread") else {
                return false;
            };
            if let Some(thread_id) = self.t079_thread_id.as_deref() {
                return t079_thread_allowed(thread, thread_id);
            }
            if !self
                .t079_requests
                .iter()
                .any(|(_, kind)| *kind == T079RequestKind::ThreadStart)
            {
                return false;
            }
            let Some(thread_id) = thread.get("id").and_then(Value::as_str) else {
                return false;
            };
            if validate_native_thread_id(thread_id).is_err()
                || !t079_thread_allowed(thread, thread_id)
            {
                return false;
            }
            self.t079_thread_id = Some(thread_id.to_owned());
            return true;
        }

        let Some(thread_id) = self.t079_thread_id.clone() else {
            return false;
        };

        if method == "thread/status/changed" {
            return exact_object_keys(params, &["status", "threadId"])
                && params.get("threadId").and_then(Value::as_str) == Some(thread_id.as_str())
                && params.get("status").is_some_and(t079_thread_status_allowed);
        }

        if method == "turn/started" && self.t079_turn_id.is_none() {
            if !self
                .t079_requests
                .iter()
                .any(|(_, kind)| *kind == T079RequestKind::TurnStart)
                || !exact_object_keys(params, &["threadId", "turn"])
                || params.get("threadId").and_then(Value::as_str) != Some(thread_id.as_str())
            {
                return false;
            }
            let Some(turn) = params.get("turn") else {
                return false;
            };
            let Some(turn_id) = turn.get("id").and_then(Value::as_str) else {
                return false;
            };
            if validate_nonempty_exact(turn_id).is_err()
                || !t079_turn_allowed(turn, turn_id, "inProgress")
            {
                return false;
            }
            self.t079_turn_id = Some(turn_id.to_owned());
            return true;
        }

        let Some(turn_id) = self.t079_turn_id.as_deref() else {
            return false;
        };

        match method {
            "turn/started" => {
                exact_object_keys(params, &["threadId", "turn"])
                    && params.get("threadId").and_then(Value::as_str) == Some(thread_id.as_str())
                    && params
                        .get("turn")
                        .is_some_and(|turn| t079_turn_allowed(turn, turn_id, "inProgress"))
            }
            "turn/completed" => {
                exact_object_keys(params, &["threadId", "turn"])
                    && params.get("threadId").and_then(Value::as_str) == Some(thread_id.as_str())
                    && params
                        .get("turn")
                        .is_some_and(|turn| t079_turn_allowed(turn, turn_id, "completed"))
            }
            "item/started" => {
                exact_object_keys(params, &["item", "startedAtMs", "threadId", "turnId"])
                    && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("startedAtMs").is_some_and(Value::is_number)
                    && params.get("item").is_some_and(t079_passive_item)
            }
            "item/completed" => {
                exact_object_keys(params, &["completedAtMs", "item", "threadId", "turnId"])
                    && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("completedAtMs").is_some_and(Value::is_number)
                    && params.get("item").is_some_and(t079_passive_item)
            }
            "item/agentMessage/delta" | "item/plan/delta" => {
                exact_object_keys(params, &["delta", "itemId", "threadId", "turnId"])
                    && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("itemId").and_then(Value::as_str).is_some()
                    && params.get("delta").and_then(Value::as_str).is_some()
            }
            "item/reasoning/summaryTextDelta" => {
                exact_object_keys(
                    params,
                    &["delta", "itemId", "summaryIndex", "threadId", "turnId"],
                ) && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("itemId").and_then(Value::as_str).is_some()
                    && params.get("delta").and_then(Value::as_str).is_some()
                    && params.get("summaryIndex").is_some_and(Value::is_number)
            }
            "item/reasoning/summaryPartAdded" => {
                exact_object_keys(params, &["itemId", "summaryIndex", "threadId", "turnId"])
                    && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("itemId").and_then(Value::as_str).is_some()
                    && params.get("summaryIndex").is_some_and(Value::is_number)
            }
            "item/reasoning/textDelta" => {
                exact_object_keys(
                    params,
                    &["contentIndex", "delta", "itemId", "threadId", "turnId"],
                ) && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params.get("itemId").and_then(Value::as_str).is_some()
                    && params.get("delta").and_then(Value::as_str).is_some()
                    && params.get("contentIndex").is_some_and(Value::is_number)
            }
            "thread/tokenUsage/updated" => {
                exact_object_keys(params, &["threadId", "tokenUsage", "turnId"])
                    && t079_notification_identity_matches(params, thread_id.as_str(), turn_id)
                    && params
                        .get("tokenUsage")
                        .is_some_and(t079_thread_token_usage_allowed)
            }
            _ => false,
        }
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
            params: protocol_params(object).or_else(|error| self.fail(error))?,
            disposition: ServerRequestDisposition::RequiresExternalDecision,
            evidence: EvidenceClass::AgentRuntimeEvidence,
        })
    }

    fn fail<T>(&mut self, error: CodexProtocolError) -> Result<T, CodexProtocolError> {
        self.state = HandshakeState::Failed;
        Err(error)
    }
}

#[cfg(test)]
fn exact_object_keys(object: &Map<String, Value>, expected: &[&str]) -> bool {
    if object.len() != expected.len() {
        return false;
    }
    expected.iter().all(|key| object.contains_key(*key))
}

#[cfg(test)]
fn object_keys_within(object: &Map<String, Value>, allowed: &[&str]) -> bool {
    object.keys().all(|key| allowed.contains(&key.as_str()))
}

#[cfg(test)]
fn t079_notification_identity_matches(
    params: &Map<String, Value>,
    thread_id: &str,
    turn_id: &str,
) -> bool {
    params.get("threadId").and_then(Value::as_str) == Some(thread_id)
        && params.get("turnId").and_then(Value::as_str) == Some(turn_id)
}

#[cfg(test)]
fn t079_thread_status_allowed(value: &Value) -> bool {
    let Some(status) = value.as_object() else {
        return false;
    };
    match status.get("type").and_then(Value::as_str) {
        Some("notLoaded" | "idle" | "systemError") => exact_object_keys(status, &["type"]),
        Some("active") => {
            exact_object_keys(status, &["activeFlags", "type"])
                && status
                    .get("activeFlags")
                    .and_then(Value::as_array)
                    .is_some_and(|flags| flags.is_empty())
        }
        _ => false,
    }
}

#[cfg(test)]
fn t079_thread_allowed(value: &Value, thread_id: &str) -> bool {
    const THREAD_KEYS: &[&str] = &[
        "agentNickname",
        "agentRole",
        "canAcceptDirectInput",
        "cliVersion",
        "createdAt",
        "cwd",
        "ephemeral",
        "extra",
        "forkedFromId",
        "gitInfo",
        "historyMode",
        "id",
        "modelProvider",
        "name",
        "parentThreadId",
        "path",
        "preview",
        "projectId",
        "recencyAt",
        "section",
        "sectionEnteredAt",
        "sessionId",
        "source",
        "status",
        "threadSource",
        "turns",
        "updatedAt",
    ];

    let Some(thread) = value.as_object() else {
        return false;
    };
    if !object_keys_within(thread, THREAD_KEYS)
        || thread.get("id").and_then(Value::as_str) != Some(thread_id)
    {
        return false;
    }
    if let Some(status) = thread.get("status")
        && !t079_thread_status_allowed(status)
    {
        return false;
    }
    if let Some(extra) = thread.get("extra")
        && !(extra.is_null() || extra.as_object().is_some_and(|extra| extra.is_empty()))
    {
        return false;
    }
    true
}

#[cfg(test)]
fn t079_passive_item(value: &Value) -> bool {
    let Some(item) = value.as_object() else {
        return false;
    };
    let Some(kind) = item.get("type").and_then(Value::as_str) else {
        return false;
    };
    if item.get("id").and_then(Value::as_str).is_none() {
        return false;
    }
    match kind {
        "userMessage" => object_keys_within(item, &["clientId", "content", "id", "type"]),
        "agentMessage" => object_keys_within(
            item,
            &["delivery", "id", "memoryCitation", "phase", "text", "type"],
        ),
        "plan" => object_keys_within(item, &["id", "text", "type"]),
        "reasoning" => object_keys_within(item, &["content", "id", "summary", "type"]),
        "contextCompaction" => exact_object_keys(item, &["id", "type"]),
        _ => false,
    }
}

#[cfg(test)]
fn t079_turn_allowed(value: &Value, turn_id: &str, expected_status: &str) -> bool {
    const TURN_KEYS: &[&str] = &[
        "completedAt",
        "durationMs",
        "error",
        "id",
        "items",
        "itemsView",
        "startedAt",
        "status",
    ];

    let Some(turn) = value.as_object() else {
        return false;
    };
    if !object_keys_within(turn, TURN_KEYS)
        || turn.get("id").and_then(Value::as_str) != Some(turn_id)
        || turn.get("status").and_then(Value::as_str) != Some(expected_status)
    {
        return false;
    }
    if let Some(items) = turn.get("items")
        && !items
            .as_array()
            .is_some_and(|items| items.iter().all(t079_passive_item))
    {
        return false;
    }
    true
}

#[cfg(test)]
fn t079_token_usage_breakdown_allowed(value: &Value) -> bool {
    let Some(usage) = value.as_object() else {
        return false;
    };
    exact_object_keys(
        usage,
        &[
            "cacheWriteInputTokens",
            "cachedInputTokens",
            "inputTokens",
            "outputTokens",
            "reasoningOutputTokens",
            "totalTokens",
        ],
    ) && usage.values().all(Value::is_number)
}

#[cfg(test)]
fn t079_thread_token_usage_allowed(value: &Value) -> bool {
    let Some(usage) = value.as_object() else {
        return false;
    };
    exact_object_keys(usage, &["last", "modelContextWindow", "total"])
        && usage
            .get("last")
            .is_some_and(t079_token_usage_breakdown_allowed)
        && usage
            .get("total")
            .is_some_and(t079_token_usage_breakdown_allowed)
        && usage
            .get("modelContextWindow")
            .is_some_and(|value| value.is_null() || value.is_number())
}

fn rpc_id_value(id: &RpcId) -> Value {
    match id {
        RpcId::Number(value) => json!(value),
        RpcId::Text(value) => json!(value),
    }
}

fn parse_rpc_id(value: &Value) -> Result<RpcId, CodexProtocolError> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .map(RpcId::Number)
            .ok_or(CodexProtocolError::MalformedFrame),
        Value::String(text)
            if !text.trim().is_empty()
                && text.len() <= MAX_PROTOCOL_TEXT_BYTES
                && !text.chars().any(char::is_control) =>
        {
            Ok(RpcId::Text(text.clone()))
        }
        _ => Err(CodexProtocolError::MalformedFrame),
    }
}

fn protocol_params(object: &Map<String, Value>) -> Result<Value, CodexProtocolError> {
    match object.get("params") {
        None => Ok(json!({})),
        Some(value @ Value::Object(_)) => Ok(value.clone()),
        Some(_) => Err(CodexProtocolError::MalformedFrame),
    }
}

fn validate_native_thread_id(value: &str) -> Result<(), CodexProtocolError> {
    if value.trim().is_empty()
        || value.len() > MAX_PROTOCOL_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(CodexProtocolError::InvalidNativeThreadId);
    }
    Ok(())
}

fn validate_nonempty_exact(value: &str) -> Result<(), CodexProtocolError> {
    if value.trim().is_empty()
        || value.len() > MAX_PROTOCOL_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
        return Err(CodexProtocolError::MalformedFrame);
    }
    Ok(())
}

fn validate_method(method: &str) -> Result<(), CodexProtocolError> {
    if method.trim().is_empty()
        || method.len() > MAX_PROTOCOL_TEXT_BYTES
        || method.chars().any(char::is_control)
    {
        return Err(CodexProtocolError::MalformedFrame);
    }
    Ok(())
}

fn strip_jsonl_terminator(frame: &[u8]) -> &[u8] {
    frame
        .strip_suffix(b"\r\n")
        .or_else(|| frame.strip_suffix(b"\n"))
        .unwrap_or(frame)
}

fn encode_jsonl(value: &Value) -> Result<String, CodexProtocolError> {
    let mut encoded =
        serde_json::to_string(value).map_err(|_| CodexProtocolError::MalformedFrame)?;
    if encoded.len() + 1 > MAX_CODEX_JSONL_FRAME_BYTES {
        return Err(CodexProtocolError::OversizedFrame);
    }
    encoded.push('\n');
    Ok(encoded)
}

#[cfg(test)]
mod t079_notification_regression_tests {
    use super::*;

    fn ready_t079_client() -> CodexProtocolClient {
        let mut client = CodexProtocolClient::default();
        client
            .t079_initialize_request("winds", "Winds", "0.1.0")
            .expect("T079 initialize");
        assert_eq!(
            client
                .ingest_jsonl_frame(br#"{"id":0,"result":{"userAgent":"fixture"}}"#)
                .expect("initialize response"),
            CodexInbound::InitializeAccepted
        );
        client
            .initialized_notification()
            .expect("initialized notification");

        let (config_id, _) = client
            .t079_config_read("/tmp/winds-t079-fixture")
            .expect("config request");
        client
            .ingest_jsonl_frame(
                format!(r#"{{"id":{config_id},"result":{{"config":{{}}}}}}"#).as_bytes(),
            )
            .expect("config response");
        client
    }

    fn active_t079_client() -> CodexProtocolClient {
        let mut client = ready_t079_client();

        let (thread_id, _) = client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{thread_id},"result":{{"thread":{{"id":"thr_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("thread response");

        let native = NativeThreadId::parse("thr_t079_fixture").expect("native thread id");
        let (turn_request_id, _) = client
            .t079_turn_start(&native, "/tmp/winds-t079-fixture")
            .expect("turn request");
        client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{turn_request_id},"result":{{"turn":{{"id":"turn_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("turn response");
        client
    }

    #[test]
    fn t079_unknown_notifications_fail_closed_even_with_empty_or_plausible_params() {
        for frame in [
            br#"{"method":"future/authorityChanged","params":{}}"#.as_slice(),
            br#"{"method":"future/authorityChanged","params":{"threadId":"thr_t079_fixture","turnId":"turn_t079_fixture"}}"#.as_slice(),
            br#"{"method":"item/futurePassiveDelta","params":{"threadId":"thr_t079_fixture","turnId":"turn_t079_fixture","itemId":"item-1","delta":"x"}}"#.as_slice(),
        ] {
            let mut client = active_t079_client();
            assert_eq!(
                client.ingest_jsonl_frame(frame),
                Err(CodexProtocolError::UnexpectedT079Notification)
            );
        }
    }

    #[test]
    fn t079_notifications_are_phase_and_identity_bound() {
        let mut before_thread = CodexProtocolClient::default();
        before_thread
            .t079_initialize_request("winds", "Winds", "0.1.0")
            .expect("T079 initialize");
        before_thread
            .ingest_jsonl_frame(br#"{"id":0,"result":{"userAgent":"fixture"}}"#)
            .expect("initialize response");
        before_thread
            .initialized_notification()
            .expect("initialized notification");
        assert_eq!(
            before_thread.ingest_jsonl_frame(
                br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_t079_fixture","status":"inProgress"}}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );

        let mut client = active_t079_client();
        assert!(matches!(
            client
                .ingest_jsonl_frame(
                    br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_t079_fixture","status":"inProgress"}}}"#
                )
                .expect("allowed turn start"),
            CodexInbound::Notification { method, .. } if method == "turn/started"
        ));

        let mut wrong_identity = active_t079_client();
        assert_eq!(
            wrong_identity.ingest_jsonl_frame(
                br#"{"method":"item/agentMessage/delta","params":{"threadId":"thr_t079_fixture","turnId":"turn_other","itemId":"item-1","delta":"x"}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );
    }

    #[test]
    fn t079_nested_notification_objects_reject_unknown_keys() {
        for frame in [
            br#"{"method":"thread/started","params":{"thread":{"id":"thr_t079_fixture","futureAuthority":true}}}"#.as_slice(),
            br#"{"method":"thread/status/changed","params":{"threadId":"thr_t079_fixture","status":{"type":"idle","futureAuthority":true}}}"#.as_slice(),
            br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_t079_fixture","status":"inProgress","futureAuthority":true}}}"#.as_slice(),
            br#"{"method":"item/started","params":{"threadId":"thr_t079_fixture","turnId":"turn_t079_fixture","startedAtMs":1,"item":{"type":"agentMessage","id":"item-1","futureAuthority":true}}}"#.as_slice(),
        ] {
            let mut client = active_t079_client();
            assert_eq!(
                client.ingest_jsonl_frame(frame),
                Err(CodexProtocolError::UnexpectedT079Notification)
            );
        }

        let mut usage_client = active_t079_client();
        assert_eq!(
            usage_client.ingest_jsonl_frame(
                br#"{"method":"thread/tokenUsage/updated","params":{"threadId":"thr_t079_fixture","turnId":"turn_t079_fixture","tokenUsage":{"total":{"totalTokens":1,"inputTokens":1,"cachedInputTokens":0,"cacheWriteInputTokens":0,"outputTokens":0,"reasoningOutputTokens":0},"last":{"totalTokens":1,"inputTokens":1,"cachedInputTokens":0,"cacheWriteInputTokens":0,"outputTokens":0,"reasoningOutputTokens":0},"modelContextWindow":128000,"futureAuthority":true}}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );
    }

    #[test]
    fn t079_started_notifications_can_bind_identity_before_matching_response() {
        let mut client = ready_t079_client();
        let (thread_request_id, _) = client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        assert!(matches!(
            client
                .ingest_jsonl_frame(
                    br#"{"method":"thread/started","params":{"thread":{"id":"thr_t079_fixture"}}}"#
                )
                .expect("pre-response thread/started"),
            CodexInbound::Notification { method, .. } if method == "thread/started"
        ));
        assert_eq!(client.t079_thread_id.as_deref(), Some("thr_t079_fixture"));
        client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{thread_request_id},"result":{{"thread":{{"id":"thr_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("matching thread response");

        let native = NativeThreadId::parse("thr_t079_fixture").expect("native thread id");
        let (turn_request_id, _) = client
            .t079_turn_start(&native, "/tmp/winds-t079-fixture")
            .expect("turn request");
        assert!(matches!(
            client
                .ingest_jsonl_frame(
                    br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_t079_fixture","status":"inProgress"}}}"#
                )
                .expect("pre-response turn/started"),
            CodexInbound::Notification { method, .. } if method == "turn/started"
        ));
        assert_eq!(client.t079_turn_id.as_deref(), Some("turn_t079_fixture"));
        client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{turn_request_id},"result":{{"turn":{{"id":"turn_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("matching turn response");
    }

    #[test]
    fn t079_pre_response_identity_mismatch_fails_closed() {
        let mut thread_client = ready_t079_client();
        let (thread_request_id, _) = thread_client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        thread_client
            .ingest_jsonl_frame(
                br#"{"method":"thread/started","params":{"thread":{"id":"thr_from_notification"}}}"#,
            )
            .expect("pre-response thread/started");
        assert_eq!(
            thread_client.ingest_jsonl_frame(
                format!(
                    r#"{{"id":{thread_request_id},"result":{{"thread":{{"id":"thr_from_response"}}}}}}"#
                )
                .as_bytes(),
            ),
            Err(CodexProtocolError::MalformedFrame)
        );

        let mut turn_client = ready_t079_client();
        let (thread_request_id, _) = turn_client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        turn_client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{thread_request_id},"result":{{"thread":{{"id":"thr_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("thread response");
        let native = NativeThreadId::parse("thr_t079_fixture").expect("native thread id");
        let (turn_request_id, _) = turn_client
            .t079_turn_start(&native, "/tmp/winds-t079-fixture")
            .expect("turn request");
        turn_client
            .ingest_jsonl_frame(
                br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_from_notification","status":"inProgress"}}}"#,
            )
            .expect("pre-response turn/started");
        assert_eq!(
            turn_client.ingest_jsonl_frame(
                format!(
                    r#"{{"id":{turn_request_id},"result":{{"turn":{{"id":"turn_from_response"}}}}}}"#
                )
                .as_bytes(),
            ),
            Err(CodexProtocolError::MalformedFrame)
        );
    }

    #[test]
    fn t079_pre_response_status_does_not_introduce_thread_identity() {
        let mut client = ready_t079_client();
        client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        assert_eq!(
            client.ingest_jsonl_frame(
                br#"{"method":"thread/status/changed","params":{"threadId":"thr_t079_fixture","status":{"type":"idle"}}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );
    }

    #[test]
    fn t079_error_responses_clear_pending_phase_and_bound_identity() {
        let mut thread_client = ready_t079_client();
        let (thread_request_id, _) = thread_client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        thread_client
            .ingest_jsonl_frame(
                br#"{"method":"thread/started","params":{"thread":{"id":"thr_from_notification"}}}"#,
            )
            .expect("pre-response thread/started");
        assert_eq!(
            thread_client
                .ingest_jsonl_frame(
                    format!(
                        r#"{{"id":{thread_request_id},"error":{{"code":-32000,"message":"fixture"}}}}"#
                    )
                    .as_bytes(),
                )
                .expect("thread error response"),
            CodexInbound::ErrorResponse {
                id: RpcId::Number(thread_request_id),
                error: json!({"code": -32000, "message": "fixture"}),
                evidence: EvidenceClass::AgentRuntimeEvidence,
            }
        );
        assert!(thread_client.t079_requests.is_empty());
        assert_eq!(thread_client.t079_thread_id, None);
        assert_eq!(thread_client.t079_turn_id, None);
        assert_eq!(
            thread_client.ingest_jsonl_frame(
                br#"{"method":"thread/started","params":{"thread":{"id":"thr_after_error"}}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );

        let mut turn_client = ready_t079_client();
        let (thread_request_id, _) = turn_client
            .t079_thread_start("/tmp/winds-t079-fixture")
            .expect("thread request");
        turn_client
            .ingest_jsonl_frame(
                format!(
                    r#"{{"id":{thread_request_id},"result":{{"thread":{{"id":"thr_t079_fixture"}}}}}}"#
                )
                .as_bytes(),
            )
            .expect("thread response");
        let native = NativeThreadId::parse("thr_t079_fixture").expect("native thread id");
        let (turn_request_id, _) = turn_client
            .t079_turn_start(&native, "/tmp/winds-t079-fixture")
            .expect("turn request");
        turn_client
            .ingest_jsonl_frame(
                br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_from_notification","status":"inProgress"}}}"#,
            )
            .expect("pre-response turn/started");
        assert_eq!(
            turn_client
                .ingest_jsonl_frame(
                    format!(
                        r#"{{"id":{turn_request_id},"error":{{"code":-32000,"message":"fixture"}}}}"#
                    )
                    .as_bytes(),
                )
                .expect("turn error response"),
            CodexInbound::ErrorResponse {
                id: RpcId::Number(turn_request_id),
                error: json!({"code": -32000, "message": "fixture"}),
                evidence: EvidenceClass::AgentRuntimeEvidence,
            }
        );
        assert!(turn_client.t079_requests.is_empty());
        assert_eq!(turn_client.t079_thread_id.as_deref(), Some("thr_t079_fixture"));
        assert_eq!(turn_client.t079_turn_id, None);
        assert_eq!(
            turn_client.ingest_jsonl_frame(
                br#"{"method":"turn/started","params":{"threadId":"thr_t079_fixture","turn":{"id":"turn_after_error","status":"inProgress"}}}"#
            ),
            Err(CodexProtocolError::UnexpectedT079Notification)
        );
    }
}
