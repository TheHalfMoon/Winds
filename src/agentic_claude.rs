use crate::agentic_runtime::{RuntimeKind, RuntimeResumeResolution};
use serde_json::Value;
use std::error::Error;
use std::fmt;

pub(super) const MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES: usize = 1024 * 1024;
pub(super) const MAX_CLAUDE_STREAM_LINE_BYTES: usize = 64 * 1024;
const MAX_NATIVE_SESSION_ID_BYTES: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaudeOutputFormat {
    Json,
    StreamJson,
}

impl ClaudeOutputFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::StreamJson => "stream-json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaudeContinuity {
    Reconstructed,
    RevalidatedResumeCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaudeRestrictionEnforcement {
    Unavailable,
}

impl ClaudeRestrictionEnforcement {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClaudeEvidenceClass {
    AgentReported,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ClaudeSessionSelection<'a> {
    New,
    RevalidatedResume(&'a RuntimeResumeResolution),
    ContinueMostRecent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ClaudeInvocation {
    pub args: Vec<String>,
    pub output_format: ClaudeOutputFormat,
    pub continuity: ClaudeContinuity,
    pub expected_native_session_id: Option<String>,
    pub restriction_enforcement: ClaudeRestrictionEnforcement,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ClaudeStructuredOutput {
    pub native_session_id: String,
    pub continuity: ClaudeContinuity,
    pub evidence: ClaudeEvidenceClass,
    pub terminal: Value,
    pub event_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ClaudeStructuredError {
    ContinueIsNotCanonical,
    ResumeUnavailable,
    ResumeStale,
    ResumeAmbiguous,
    ResumeRuntimeMismatch,
    ResumeMissingNativeSessionId,
    InvalidNativeSessionId,
    UnsafeConstruction,
    MalformedOutput,
    TruncatedOutput,
    OversizedOutput,
    NativeSessionMismatch,
}

impl fmt::Display for ClaudeStructuredError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ContinueIsNotCanonical => {
                "Claude --continue cannot establish canonical Winds continuation"
            }
            Self::ResumeUnavailable => "Claude exact resume binding is unavailable",
            Self::ResumeStale => "Claude exact resume binding is stale",
            Self::ResumeAmbiguous => "Claude exact resume binding is ambiguous",
            Self::ResumeRuntimeMismatch => "Claude resume binding belongs to a different runtime",
            Self::ResumeMissingNativeSessionId => {
                "Claude resume binding is missing an exact native session id"
            }
            Self::InvalidNativeSessionId => "Claude native session id is invalid",
            Self::UnsafeConstruction => "Claude invocation construction is not accepted",
            Self::MalformedOutput => "Claude structured output is malformed",
            Self::TruncatedOutput => "Claude structured output is truncated",
            Self::OversizedOutput => "Claude structured output exceeds its bounded limit",
            Self::NativeSessionMismatch => {
                "Claude structured output does not match the exact native session id"
            }
        };
        formatter.write_str(message)
    }
}

impl Error for ClaudeStructuredError {}

pub(super) fn build_claude_structured_invocation(
    output_format: ClaudeOutputFormat,
    selection: ClaudeSessionSelection<'_>,
) -> Result<ClaudeInvocation, ClaudeStructuredError> {
    let mut args = vec![
        "--print".to_owned(),
        "--output-format".to_owned(),
        output_format.as_str().to_owned(),
    ];

    let (continuity, expected_native_session_id) = match selection {
        ClaudeSessionSelection::New => (ClaudeContinuity::Reconstructed, None),
        ClaudeSessionSelection::ContinueMostRecent => {
            return Err(ClaudeStructuredError::ContinueIsNotCanonical);
        }
        ClaudeSessionSelection::RevalidatedResume(resolution) => match resolution {
            RuntimeResumeResolution::Unavailable => {
                return Err(ClaudeStructuredError::ResumeUnavailable);
            }
            RuntimeResumeResolution::Stale => return Err(ClaudeStructuredError::ResumeStale),
            RuntimeResumeResolution::Ambiguous(_) => {
                return Err(ClaudeStructuredError::ResumeAmbiguous);
            }
            RuntimeResumeResolution::Candidate(binding) => {
                if binding.runtime != RuntimeKind::Claude {
                    return Err(ClaudeStructuredError::ResumeRuntimeMismatch);
                }
                let native_session_id = binding
                    .native_session_id
                    .as_deref()
                    .ok_or(ClaudeStructuredError::ResumeMissingNativeSessionId)?;
                validate_native_session_id(native_session_id)?;
                args.push("--resume".to_owned());
                args.push(native_session_id.to_owned());
                (
                    ClaudeContinuity::RevalidatedResumeCandidate,
                    Some(native_session_id.to_owned()),
                )
            }
        },
    };

    validate_accepted_args(&args)?;
    Ok(ClaudeInvocation {
        args,
        output_format,
        continuity,
        expected_native_session_id,
        restriction_enforcement: ClaudeRestrictionEnforcement::Unavailable,
    })
}

pub(super) fn parse_claude_structured_output(
    invocation: &ClaudeInvocation,
    output: &[u8],
) -> Result<ClaudeStructuredOutput, ClaudeStructuredError> {
    if output.len() > MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES {
        return Err(ClaudeStructuredError::OversizedOutput);
    }
    if output.is_empty() {
        return Err(ClaudeStructuredError::TruncatedOutput);
    }

    match invocation.output_format {
        ClaudeOutputFormat::Json => parse_json_output(invocation, output),
        ClaudeOutputFormat::StreamJson => parse_stream_json_output(invocation, output),
    }
}

fn parse_json_output(
    invocation: &ClaudeInvocation,
    output: &[u8],
) -> Result<ClaudeStructuredOutput, ClaudeStructuredError> {
    let terminal = parse_json_value(output)?;
    finish_terminal_output(invocation, terminal, 1)
}

fn parse_stream_json_output(
    invocation: &ClaudeInvocation,
    output: &[u8],
) -> Result<ClaudeStructuredOutput, ClaudeStructuredError> {
    if !output.ends_with(b"\n") {
        return Err(ClaudeStructuredError::TruncatedOutput);
    }

    let payload = &output[..output.len() - 1];
    if payload.is_empty() {
        return Err(ClaudeStructuredError::TruncatedOutput);
    }

    let mut events = Vec::new();
    for line in payload.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            return Err(ClaudeStructuredError::MalformedOutput);
        }
        if line.len() > MAX_CLAUDE_STREAM_LINE_BYTES {
            return Err(ClaudeStructuredError::OversizedOutput);
        }
        let value = parse_json_value(line)?;
        if !value.is_object() {
            return Err(ClaudeStructuredError::MalformedOutput);
        }
        events.push(value);
    }

    let terminal = events
        .last()
        .ok_or(ClaudeStructuredError::TruncatedOutput)?;
    if event_type(terminal) != Some("result") {
        return Err(ClaudeStructuredError::TruncatedOutput);
    }
    if events[..events.len() - 1]
        .iter()
        .any(|event| event_type(event) == Some("result"))
    {
        return Err(ClaudeStructuredError::MalformedOutput);
    }

    let terminal_session_id = required_result_session_id(terminal)?;
    for event in &events {
        if let Some(value) = event.get("session_id") {
            let session_id = value
                .as_str()
                .ok_or(ClaudeStructuredError::MalformedOutput)?;
            validate_native_session_id(session_id)?;
            if session_id != terminal_session_id {
                return Err(ClaudeStructuredError::NativeSessionMismatch);
            }
        }
    }

    finish_terminal_output(invocation, terminal.clone(), events.len())
}

fn finish_terminal_output(
    invocation: &ClaudeInvocation,
    terminal: Value,
    event_count: usize,
) -> Result<ClaudeStructuredOutput, ClaudeStructuredError> {
    let native_session_id = required_result_session_id(&terminal)?.to_owned();
    if let Some(expected) = invocation.expected_native_session_id.as_deref()
        && native_session_id != expected
    {
        return Err(ClaudeStructuredError::NativeSessionMismatch);
    }

    Ok(ClaudeStructuredOutput {
        native_session_id,
        continuity: invocation.continuity,
        evidence: ClaudeEvidenceClass::AgentReported,
        terminal,
        event_count,
    })
}

fn required_result_session_id(value: &Value) -> Result<&str, ClaudeStructuredError> {
    if !value.is_object() || event_type(value) != Some("result") {
        return Err(ClaudeStructuredError::MalformedOutput);
    }
    let session_id = value
        .get("session_id")
        .and_then(Value::as_str)
        .ok_or(ClaudeStructuredError::MalformedOutput)?;
    validate_native_session_id(session_id)?;
    Ok(session_id)
}

fn event_type(value: &Value) -> Option<&str> {
    value.get("type").and_then(Value::as_str)
}

fn parse_json_value(bytes: &[u8]) -> Result<Value, ClaudeStructuredError> {
    serde_json::from_slice(bytes).map_err(|error| {
        if error.is_eof() {
            ClaudeStructuredError::TruncatedOutput
        } else {
            ClaudeStructuredError::MalformedOutput
        }
    })
}

fn validate_native_session_id(value: &str) -> Result<(), ClaudeStructuredError> {
    if value.is_empty()
        || value.len() > MAX_NATIVE_SESSION_ID_BYTES
        || value.trim() != value
        || value.starts_with('-')
        || value.chars().any(char::is_control)
    {
        return Err(ClaudeStructuredError::InvalidNativeSessionId);
    }
    Ok(())
}

fn validate_accepted_args(args: &[String]) -> Result<(), ClaudeStructuredError> {
    for (index, arg) in args.iter().enumerate() {
        if matches!(
            arg.as_str(),
            "--dangerously-skip-permissions"
                | "--allow-dangerously-skip-permissions"
                | "--continue"
                | "-c"
        ) {
            return Err(ClaudeStructuredError::UnsafeConstruction);
        }
        if arg == "--permission-mode"
            && args.get(index + 1).map(String::as_str) == Some("bypassPermissions")
        {
            return Err(ClaudeStructuredError::UnsafeConstruction);
        }
    }
    Ok(())
}
