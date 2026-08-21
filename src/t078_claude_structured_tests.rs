use crate::agentic_claude::{
    ClaudeContinuity, ClaudeEvidenceClass, ClaudeOutputFormat, ClaudeRestrictionEnforcement,
    ClaudeSessionSelection, ClaudeStructuredError, MAX_CLAUDE_STREAM_LINE_BYTES,
    MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES, build_claude_structured_invocation,
    parse_claude_structured_output,
};
use crate::agentic_runtime::{
    EvidenceSource, RuntimeBindingOwnership, RuntimeExecutableIdentity, RuntimeKind,
    RuntimeResumeResolution, RuntimeSessionBinding, RuntimeVersionEvidence, RuntimeVersionState,
};
use std::path::PathBuf;

fn fixture_binding(runtime: RuntimeKind, native_session_id: Option<&str>) -> RuntimeSessionBinding {
    RuntimeSessionBinding {
        binding_id: "binding-fixture".to_owned(),
        session_id: "winds-session-fixture".to_owned(),
        runtime,
        executable: RuntimeExecutableIdentity {
            observed_path: PathBuf::from("/fixture/claude"),
            canonical_path: PathBuf::from("/fixture/claude"),
            byte_len: 7,
            sha256: "a".repeat(64),
        },
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("2.1.0-fixture".to_owned()),
            source: EvidenceSource::WindsLocallyObserved,
        },
        native_session_id: native_session_id.map(str::to_owned),
        ownership: RuntimeBindingOwnership::Unproven,
        bound_unix_ms: 10,
        ownership_observed_unix_ms: None,
    }
}

fn claude_resume(native_session_id: &str) -> RuntimeResumeResolution {
    RuntimeResumeResolution::Candidate(Box::new(fixture_binding(
        RuntimeKind::Claude,
        Some(native_session_id),
    )))
}

#[test]
fn new_json_construction_is_structured_and_permission_safe() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::Json,
        ClaudeSessionSelection::New,
    )
    .expect("new structured invocation");

    assert_eq!(
        invocation.args,
        vec!["--print", "--output-format", "json"]
    );
    assert_eq!(invocation.continuity, ClaudeContinuity::Reconstructed);
    assert_eq!(invocation.expected_native_session_id, None);
    assert_eq!(
        invocation.restriction_enforcement,
        ClaudeRestrictionEnforcement::Unavailable
    );
    assert_eq!(invocation.restriction_enforcement.as_str(), "UNAVAILABLE");
    assert!(!invocation.args.iter().any(|arg| arg.contains("dangerously")));
    assert!(!invocation.args.iter().any(|arg| arg == "--continue"));
    assert!(!invocation.args.iter().any(|arg| arg == "-c"));
}

#[test]
fn stream_json_construction_uses_exact_structured_output_flag() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::StreamJson,
        ClaudeSessionSelection::New,
    )
    .expect("stream structured invocation");

    assert_eq!(
        invocation.args,
        vec!["--print", "--output-format", "stream-json"]
    );
    assert_eq!(invocation.output_format, ClaudeOutputFormat::StreamJson);
}

#[test]
fn exact_resume_requires_a_revalidated_claude_binding() {
    let resolution = claude_resume("550e8400-e29b-41d4-a716-446655440000");
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::Json,
        ClaudeSessionSelection::RevalidatedResume(&resolution),
    )
    .expect("revalidated Claude resume");

    assert_eq!(
        invocation.args,
        vec![
            "--print",
            "--output-format",
            "json",
            "--resume",
            "550e8400-e29b-41d4-a716-446655440000"
        ]
    );
    assert_eq!(
        invocation.continuity,
        ClaudeContinuity::RevalidatedResumeCandidate
    );
    assert_eq!(
        invocation.expected_native_session_id.as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
}

#[test]
fn stale_unavailable_ambiguous_wrong_runtime_and_continue_fail_closed() {
    for resolution in [
        RuntimeResumeResolution::Unavailable,
        RuntimeResumeResolution::Stale,
        RuntimeResumeResolution::Ambiguous(vec![
            fixture_binding(RuntimeKind::Claude, Some("session-a")),
            fixture_binding(RuntimeKind::Claude, Some("session-b")),
        ]),
    ] {
        let error = build_claude_structured_invocation(
            ClaudeOutputFormat::Json,
            ClaudeSessionSelection::RevalidatedResume(&resolution),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ClaudeStructuredError::ResumeUnavailable
                | ClaudeStructuredError::ResumeStale
                | ClaudeStructuredError::ResumeAmbiguous
        ));
    }

    let wrong_runtime = RuntimeResumeResolution::Candidate(Box::new(fixture_binding(
        RuntimeKind::Codex,
        Some("codex-native"),
    )));
    assert_eq!(
        build_claude_structured_invocation(
            ClaudeOutputFormat::Json,
            ClaudeSessionSelection::RevalidatedResume(&wrong_runtime),
        )
        .unwrap_err(),
        ClaudeStructuredError::ResumeRuntimeMismatch
    );

    assert_eq!(
        build_claude_structured_invocation(
            ClaudeOutputFormat::Json,
            ClaudeSessionSelection::ContinueMostRecent,
        )
        .unwrap_err(),
        ClaudeStructuredError::ContinueIsNotCanonical
    );
}

#[test]
fn unsafe_native_resume_identifier_is_rejected_without_option_injection() {
    for native_session_id in [
        "--dangerously-skip-permissions",
        "--continue",
        " session-with-space-edge",
        "session-with-space-edge ",
        "session\nnewline",
    ] {
        let resolution = claude_resume(native_session_id);
        assert_eq!(
            build_claude_structured_invocation(
                ClaudeOutputFormat::Json,
                ClaudeSessionSelection::RevalidatedResume(&resolution),
            )
            .unwrap_err(),
            ClaudeStructuredError::InvalidNativeSessionId
        );
    }
}

#[test]
fn json_output_is_agent_reported_and_new_session_is_reconstructed() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::Json,
        ClaudeSessionSelection::New,
    )
    .expect("new json invocation");
    let parsed = parse_claude_structured_output(
        &invocation,
        br#"{"type":"result","subtype":"success","session_id":"session-new","result":"fixture"}"#,
    )
    .expect("valid fixture JSON output");

    assert_eq!(parsed.native_session_id, "session-new");
    assert_eq!(parsed.continuity, ClaudeContinuity::Reconstructed);
    assert_eq!(parsed.evidence, ClaudeEvidenceClass::AgentReported);
    assert_eq!(parsed.event_count, 1);
    assert_eq!(parsed.terminal["result"], "fixture");
}

#[test]
fn resume_output_must_match_the_exact_bound_native_session() {
    let resolution = claude_resume("session-exact");
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::Json,
        ClaudeSessionSelection::RevalidatedResume(&resolution),
    )
    .expect("resume invocation");

    let parsed = parse_claude_structured_output(
        &invocation,
        br#"{"type":"result","subtype":"success","session_id":"session-exact","result":"fixture"}"#,
    )
    .expect("matching fixture output");
    assert_eq!(
        parsed.continuity,
        ClaudeContinuity::RevalidatedResumeCandidate
    );

    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            br#"{"type":"result","subtype":"success","session_id":"session-other","result":"fixture"}"#,
        )
        .unwrap_err(),
        ClaudeStructuredError::NativeSessionMismatch
    );
}

#[test]
fn malformed_truncated_and_oversized_json_fail_truthfully() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::Json,
        ClaudeSessionSelection::New,
    )
    .expect("new json invocation");

    assert_eq!(
        parse_claude_structured_output(&invocation, b"{").unwrap_err(),
        ClaudeStructuredError::TruncatedOutput
    );
    assert_eq!(
        parse_claude_structured_output(&invocation, b"not-json").unwrap_err(),
        ClaudeStructuredError::MalformedOutput
    );
    assert_eq!(
        parse_claude_structured_output(&invocation, br#"{"type":"assistant"}"#).unwrap_err(),
        ClaudeStructuredError::MalformedOutput
    );

    let oversized = vec![b'x'; MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES + 1];
    assert_eq!(
        parse_claude_structured_output(&invocation, &oversized).unwrap_err(),
        ClaudeStructuredError::OversizedOutput
    );
}

#[test]
fn stream_json_requires_terminal_result_and_consistent_session_identity() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::StreamJson,
        ClaudeSessionSelection::New,
    )
    .expect("new stream invocation");

    let parsed = parse_claude_structured_output(
        &invocation,
        b"{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"stream-session\"}\n{\"type\":\"assistant\",\"session_id\":\"stream-session\"}\n{\"type\":\"result\",\"subtype\":\"success\",\"session_id\":\"stream-session\",\"result\":\"done\"}\n",
    )
    .expect("valid stream-json fixture");
    assert_eq!(parsed.native_session_id, "stream-session");
    assert_eq!(parsed.event_count, 3);
    assert_eq!(parsed.continuity, ClaudeContinuity::Reconstructed);

    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            b"{\"type\":\"system\",\"session_id\":\"one\"}\n{\"type\":\"result\",\"session_id\":\"two\"}\n",
        )
        .unwrap_err(),
        ClaudeStructuredError::NativeSessionMismatch
    );
}

#[test]
fn truncated_malformed_and_oversized_stream_json_fail_truthfully() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::StreamJson,
        ClaudeSessionSelection::New,
    )
    .expect("new stream invocation");

    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            b"{\"type\":\"result\",\"session_id\":\"session\"}",
        )
        .unwrap_err(),
        ClaudeStructuredError::TruncatedOutput
    );
    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            b"{\"type\":\"system\",\"session_id\":\"session\"}\n",
        )
        .unwrap_err(),
        ClaudeStructuredError::TruncatedOutput
    );
    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            b"{not-json}\n{\"type\":\"result\",\"session_id\":\"session\"}\n",
        )
        .unwrap_err(),
        ClaudeStructuredError::MalformedOutput
    );

    let mut oversized_line = vec![b'a'; MAX_CLAUDE_STREAM_LINE_BYTES + 1];
    oversized_line.push(b'\n');
    assert_eq!(
        parse_claude_structured_output(&invocation, &oversized_line).unwrap_err(),
        ClaudeStructuredError::OversizedOutput
    );
}

#[test]
fn stream_result_must_be_terminal() {
    let invocation = build_claude_structured_invocation(
        ClaudeOutputFormat::StreamJson,
        ClaudeSessionSelection::New,
    )
    .expect("new stream invocation");

    assert_eq!(
        parse_claude_structured_output(
            &invocation,
            b"{\"type\":\"result\",\"session_id\":\"session\"}\n{\"type\":\"assistant\",\"session_id\":\"session\"}\n",
        )
        .unwrap_err(),
        ClaudeStructuredError::TruncatedOutput
    );
}

#[test]
fn t078_surface_contains_no_real_process_launch_or_prompt_execution() {
    let source = include_str!("agentic_claude.rs");
    assert!(!source.contains("Command::new"));
    assert!(!source.contains("std::process::Command"));
    assert!(!source.contains("tokio::process"));

    for format in [ClaudeOutputFormat::Json, ClaudeOutputFormat::StreamJson] {
        let invocation = build_claude_structured_invocation(format, ClaudeSessionSelection::New)
            .expect("accepted fixture construction");
        assert_eq!(invocation.args[0], "--print");
        assert!(!invocation.args.iter().any(|arg| arg == "--continue"));
        assert!(!invocation.args.iter().any(|arg| arg == "-c"));
        assert!(!invocation.args.iter().any(|arg| arg.contains("dangerously")));
        assert!(!invocation.args.iter().any(|arg| arg == "bypassPermissions"));
        assert!(!invocation.args.iter().any(|arg| arg.contains("mcp")));
        assert!(!invocation.args.iter().any(|arg| arg.contains("remote")));
    }
}
