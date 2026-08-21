use crate::agentic_codex::{
    CleanupDecision, CodexInbound, CodexProtocolClient, CodexProtocolError, EvidenceClass,
    MAX_CODEX_JSONL_FRAME_BYTES, NativeThreadId, ProcessOwnership, RpcId,
    ServerRequestDisposition, cleanup_decision,
};
use serde_json::{Value, json};

fn parse_outbound(line: &str) -> Value {
    assert!(line.ends_with('\n'));
    serde_json::from_str(line.trim_end()).expect("outbound JSONL must be valid JSON")
}

fn initialized_client() -> CodexProtocolClient {
    let mut client = CodexProtocolClient::default();
    let initialize = client
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    let initialize = parse_outbound(&initialize);
    assert_eq!(initialize["method"], "initialize");
    assert_eq!(initialize["id"], 0);
    assert_eq!(initialize["params"]["clientInfo"]["name"], "winds");

    assert_eq!(
        client
            .ingest_jsonl_frame(br#"{"id":0,"result":{"userAgent":"fake"}}"#)
            .expect("successful initialize response"),
        CodexInbound::InitializeAccepted
    );
    let initialized = parse_outbound(
        &client
            .initialized_notification()
            .expect("initialized notification"),
    );
    assert_eq!(initialized, json!({ "method": "initialized", "params": {} }));
    assert!(client.is_ready());
    client
}

#[test]
fn mandatory_handshake_blocks_all_thread_methods_until_complete() {
    let mut client = CodexProtocolClient::default();
    let native = NativeThreadId::parse("thr_fixture").expect("native id");

    assert_eq!(
        client.thread_start().unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );
    assert_eq!(
        client.thread_resume(&native).unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );
    assert_eq!(
        client.thread_fork(&native).unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );

    client
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        client.thread_start().unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );
    client
        .ingest_jsonl_frame(br#"{"id":0,"result":{}}"#)
        .expect("initialize response");
    assert_eq!(
        client.thread_start().unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );

    client
        .initialized_notification()
        .expect("initialized notification");
    assert!(client.thread_start().is_ok());
}

#[test]
fn initialized_notification_requires_successful_initialize_response() {
    let mut fresh = CodexProtocolClient::default();
    assert_eq!(
        fresh.initialized_notification().unwrap_err(),
        CodexProtocolError::InitializedOutOfOrder
    );

    let mut sent = CodexProtocolClient::default();
    sent.initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        sent.initialized_notification().unwrap_err(),
        CodexProtocolError::InitializedOutOfOrder
    );
}

#[test]
fn rejected_or_mismatched_initialize_response_fails_closed() {
    let mut mismatch = CodexProtocolClient::default();
    mismatch
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        mismatch
            .ingest_jsonl_frame(br#"{"id":9,"result":{}}"#)
            .unwrap_err(),
        CodexProtocolError::InitializeResponseIdMismatch
    );
    assert_eq!(
        mismatch.thread_start().unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );

    let mut rejected = CodexProtocolClient::default();
    rejected
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        rejected
            .ingest_jsonl_frame(br#"{"id":0,"error":{"code":-1,"message":"no"}}"#)
            .unwrap_err(),
        CodexProtocolError::InitializeRejected
    );
}

#[test]
fn thread_start_resume_and_fork_use_bounded_protocol_requests_only() {
    let mut client = initialized_client();
    let native = NativeThreadId::parse("thr_native_exact").expect("native id");

    let start = parse_outbound(&client.thread_start().expect("thread start"));
    let resume = parse_outbound(&client.thread_resume(&native).expect("thread resume"));
    let fork = parse_outbound(&client.thread_fork(&native).expect("thread fork"));

    assert_eq!(start, json!({ "method": "thread/start", "id": 1, "params": {} }));
    assert_eq!(
        resume,
        json!({ "method": "thread/resume", "id": 2, "params": { "threadId": "thr_native_exact" } })
    );
    assert_eq!(
        fork,
        json!({ "method": "thread/fork", "id": 3, "params": { "threadId": "thr_native_exact" } })
    );
    for request in [start, resume, fork] {
        let serialized = request.to_string();
        assert!(!serialized.contains("turn/start"));
        assert!(!serialized.contains("input"));
        assert!(!serialized.contains("prompt"));
    }
}

#[test]
fn malformed_jsonl_fails_closed_and_cannot_later_authorize() {
    let mut client = initialized_client();
    assert_eq!(
        client.ingest_jsonl_frame(b"{not-json}\n").unwrap_err(),
        CodexProtocolError::MalformedFrame
    );
    assert_eq!(
        client.thread_start().unwrap_err(),
        CodexProtocolError::HandshakeIncomplete
    );
    assert_eq!(
        client.ingest_jsonl_frame(br#"{"method":"turn/started","params":{}}"#)
            .unwrap_err(),
        CodexProtocolError::ClientFailed
    );
}

#[test]
fn oversized_jsonl_is_rejected_before_parsing() {
    let mut client = initialized_client();
    let oversized = vec![b' '; MAX_CODEX_JSONL_FRAME_BYTES + 1];
    assert_eq!(
        client.ingest_jsonl_frame(&oversized).unwrap_err(),
        CodexProtocolError::OversizedFrame
    );
    assert!(!client.is_ready());
}

#[test]
fn structural_unknown_frame_fails_closed() {
    let mut client = initialized_client();
    assert_eq!(
        client.ingest_jsonl_frame(br#"{"wat":true}"#).unwrap_err(),
        CodexProtocolError::MalformedFrame
    );
}

#[test]
fn unknown_notification_remains_runtime_evidence_not_verification() {
    let mut client = initialized_client();
    let event = client
        .ingest_jsonl_frame(br#"{"method":"future/unknown","params":{"done":true}}"#)
        .expect("unknown notification remains observable");

    assert_eq!(
        event,
        CodexInbound::Notification {
            method: "future/unknown".to_owned(),
            params: json!({ "done": true }),
            evidence: EvidenceClass::AgentRuntimeEvidence,
        }
    );
    assert!(client.is_ready());
}

#[test]
fn approval_request_never_self_authorizes() {
    let mut client = initialized_client();
    let request = client
        .ingest_jsonl_frame(
            br#"{"id":"approval-1","method":"item/commandExecution/requestApproval","params":{"threadId":"thr_1","turnId":"turn_1","itemId":"item_1"}}"#,
        )
        .expect("approval request is parsed as external decision input");

    assert_eq!(
        request,
        CodexInbound::ServerRequest {
            id: RpcId::Text("approval-1".to_owned()),
            method: "item/commandExecution/requestApproval".to_owned(),
            params: json!({
                "threadId": "thr_1",
                "turnId": "turn_1",
                "itemId": "item_1"
            }),
            disposition: ServerRequestDisposition::RequiresExternalDecision,
            evidence: EvidenceClass::AgentRuntimeEvidence,
        }
    );
    assert!(client.is_ready());
}

#[test]
fn every_server_initiated_request_requires_external_decision() {
    let mut client = initialized_client();
    let request = client
        .ingest_jsonl_frame(
            br#"{"id":44,"method":"future/serverRequest","params":{"suggested":"allow"}}"#,
        )
        .expect("unknown server request is bounded");
    assert_eq!(
        request,
        CodexInbound::ServerRequest {
            id: RpcId::Number(44),
            method: "future/serverRequest".to_owned(),
            params: json!({ "suggested": "allow" }),
            disposition: ServerRequestDisposition::RequiresExternalDecision,
            evidence: EvidenceClass::AgentRuntimeEvidence,
        }
    );
}

#[test]
fn fake_server_exit_during_handshake_is_truthful_failure() {
    let mut client = CodexProtocolClient::default();
    client
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        client.on_server_eof().unwrap_err(),
        CodexProtocolError::ServerExitedDuringHandshake
    );
    assert!(!client.is_ready());
}

#[test]
fn fake_server_exit_after_handshake_is_not_reported_as_success() {
    let mut client = initialized_client();
    assert_eq!(
        client.on_server_eof().unwrap_err(),
        CodexProtocolError::ServerExited
    );
    assert!(!client.is_ready());
}

#[test]
fn native_thread_identity_never_becomes_winds_identity() {
    let mut client = initialized_client();
    let response = client
        .ingest_jsonl_frame(br#"{"id":1,"result":{"thread":{"id":"thr_runtime_native"}}}"#)
        .expect("thread response");
    let CodexInbound::Response {
        result,
        evidence: EvidenceClass::AgentRuntimeEvidence,
        ..
    } = response
    else {
        panic!("expected runtime response")
    };
    let native = NativeThreadId::from_thread_result(&result).expect("native thread id");
    assert_eq!(native.as_str(), "thr_runtime_native");

    let resume = parse_outbound(&client.thread_resume(&native).expect("resume request"));
    assert_eq!(resume["params"]["threadId"], "thr_runtime_native");
    let serialized = resume.to_string();
    assert!(!serialized.contains("session_id"));
    assert!(!serialized.contains("workstream_id"));
    assert!(!serialized.contains("workspace_id"));
    assert_eq!(
        NativeThreadId::parse("   ").unwrap_err(),
        CodexProtocolError::InvalidNativeThreadId
    );
}

#[test]
fn error_response_is_runtime_evidence_not_authority() {
    let mut client = initialized_client();
    let error = client
        .ingest_jsonl_frame(br#"{"id":8,"error":{"code":123,"message":"fake failure"}}"#)
        .expect("error response stays structured evidence");
    assert_eq!(
        error,
        CodexInbound::ErrorResponse {
            id: RpcId::Number(8),
            error: json!({ "code": 123, "message": "fake failure" }),
            evidence: EvidenceClass::AgentRuntimeEvidence,
        }
    );
}

#[test]
fn cleanup_can_target_only_proven_owned_child() {
    assert_eq!(
        cleanup_decision(ProcessOwnership::ProvenOwnedChild),
        CleanupDecision::TerminateProvenOwnedChild
    );
    assert_eq!(
        cleanup_decision(ProcessOwnership::Unproven),
        CleanupDecision::PreserveUnprovenProcess
    );
}

#[test]
fn jsonl_crlf_terminator_is_accepted_but_embedded_newline_is_not() {
    let mut client = CodexProtocolClient::default();
    client
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("initialize request");
    assert_eq!(
        client
            .ingest_jsonl_frame(b"{\"id\":0,\"result\":{}}\r\n")
            .expect("CRLF frame"),
        CodexInbound::InitializeAccepted
    );

    let mut ready = initialized_client();
    assert_eq!(
        ready
            .ingest_jsonl_frame(b"{\"method\":\"a\"}\n{\"method\":\"b\"}\n")
            .unwrap_err(),
        CodexProtocolError::MalformedFrame
    );
}
