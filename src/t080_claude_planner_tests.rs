use super::{
    ClaudeContinuity, ClaudeEvidenceClass, ClaudeRestrictionEnforcement, ClaudeSessionSelection,
    ClaudeStructuredError, MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES, T080_MAX_AGENTIC_TURNS,
    T080_MCP_DENY_RULE, T080_PLANNER_PROMPT, T080_PLANNER_TOOLS, build_t080_planner_invocation,
    parse_claude_structured_output,
};
use crate::agentic_runtime::{
    EvidenceSource, RuntimeBindingOwnership, RuntimeExecutableIdentity, RuntimeIdentityRevalidation,
    RuntimeKind, RuntimeResumeResolution, RuntimeSessionBinding, RuntimeVersionEvidence,
    RuntimeVersionState, revalidate_runtime_identity,
};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LIVE_TIMEOUT: Duration = Duration::from_secs(120);
const VERSION_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_VERSION_BYTES: usize = 256;
const MIN_RESTRICTED_VERSION: (u64, u64, u64) = (2, 1, 248);

fn fixture_binding(runtime: RuntimeKind, native_session_id: Option<&str>) -> RuntimeSessionBinding {
    RuntimeSessionBinding {
        binding_id: "binding-t080-fixture".to_owned(),
        session_id: "winds-session-t080-fixture".to_owned(),
        runtime,
        executable: RuntimeExecutableIdentity {
            observed_path: PathBuf::from("/fixture/claude"),
            canonical_path: PathBuf::from("/fixture/claude"),
            byte_len: 7,
            sha256: "a".repeat(64),
        },
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some("2.1.248-fixture".to_owned()),
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
fn t080_planner_invocation_is_bounded_read_plan_only() {
    let planner = build_t080_planner_invocation(
        ClaudeSessionSelection::New,
        RuntimeIdentityRevalidation::Match,
        "/fixture/empty-mcp.json",
    )
    .expect("accepted T080 planner invocation");

    assert_eq!(
        planner.invocation.args,
        vec![
            "--print",
            "--output-format",
            "json",
            "--restricted",
            "--permission-mode",
            "plan",
            "--tools",
            T080_PLANNER_TOOLS,
            "--disallowedTools",
            T080_MCP_DENY_RULE,
            "--strict-mcp-config",
            "--mcp-config",
            "/fixture/empty-mcp.json",
            "--disable-slash-commands",
            "--no-chrome",
            "--max-turns",
            "4",
        ]
    );
    assert_eq!(T080_MAX_AGENTIC_TURNS, 4);
    assert_eq!(planner.prompt, T080_PLANNER_PROMPT);
    assert_eq!(
        planner.invocation.continuity,
        ClaudeContinuity::Reconstructed
    );
    assert_eq!(
        planner.invocation.restriction_enforcement,
        ClaudeRestrictionEnforcement::AgentNativeEnforced
    );
    assert_eq!(
        planner.invocation.restriction_enforcement.as_str(),
        "AGENT_NATIVE_ENFORCED"
    );
    for forbidden in [
        "--continue",
        "-c",
        "--dangerously-skip-permissions",
        "--allow-dangerously-skip-permissions",
        "bypassPermissions",
        "--cloud",
        "--remote",
        "--remote-control",
        "--chrome",
        "--bg",
        "--exec",
    ] {
        assert!(!planner.invocation.args.iter().any(|arg| arg == forbidden));
    }
}

#[test]
fn t080_requires_fresh_exact_runtime_identity_match() {
    for (identity, expected) in [
        (
            RuntimeIdentityRevalidation::Changed,
            ClaudeStructuredError::RuntimeIdentityChanged,
        ),
        (
            RuntimeIdentityRevalidation::Unavailable,
            ClaudeStructuredError::RuntimeIdentityUnavailable,
        ),
    ] {
        assert_eq!(
            build_t080_planner_invocation(
                ClaudeSessionSelection::New,
                identity,
                "/fixture/empty-mcp.json",
            )
            .unwrap_err(),
            expected
        );
    }
}

#[test]
fn t080_exact_resume_preserves_native_session_provenance() {
    let resolution = claude_resume("550e8400-e29b-41d4-a716-446655440000");
    let planner = build_t080_planner_invocation(
        ClaudeSessionSelection::RevalidatedResume(&resolution),
        RuntimeIdentityRevalidation::Match,
        "/fixture/empty-mcp.json",
    )
    .expect("accepted exact resume planner");

    assert_eq!(
        planner.invocation.continuity,
        ClaudeContinuity::RevalidatedResumeCandidate
    );
    assert_eq!(
        planner.invocation.expected_native_session_id.as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
    assert!(
        planner
            .invocation
            .args
            .windows(2)
            .any(|pair| pair == ["--resume", "550e8400-e29b-41d4-a716-446655440000"])
    );
}

#[test]
fn t080_invalid_empty_mcp_config_paths_fail_closed() {
    for path in [
        "",
        " relative-leading-space",
        "relative-trailing-space ",
        "--settings",
        "bad\npath",
    ] {
        assert_eq!(
            build_t080_planner_invocation(
                ClaudeSessionSelection::New,
                RuntimeIdentityRevalidation::Match,
                path,
            )
            .unwrap_err(),
            ClaudeStructuredError::InvalidPlannerConfigPath
        );
    }
}

#[test]
fn t080_planner_result_remains_agent_reported_not_acceptance() {
    let planner = build_t080_planner_invocation(
        ClaudeSessionSelection::New,
        RuntimeIdentityRevalidation::Match,
        "/fixture/empty-mcp.json",
    )
    .expect("accepted T080 planner invocation");
    let parsed = parse_claude_structured_output(
        &planner.invocation,
        br#"{"type":"result","subtype":"success","session_id":"t080-native-session","result":"Read-only fixture plan"}"#,
    )
    .expect("valid T080 planner fixture result");

    assert_eq!(parsed.native_session_id, "t080-native-session");
    assert_eq!(parsed.continuity, ClaudeContinuity::Reconstructed);
    assert_eq!(parsed.evidence, ClaudeEvidenceClass::AgentReported);
    assert_eq!(parsed.event_count, 1);
    assert_eq!(parsed.terminal["result"], "Read-only fixture plan");
}

#[test]
fn t080_contract_denies_ambient_extension_and_write_surfaces() {
    let planner = build_t080_planner_invocation(
        ClaudeSessionSelection::New,
        RuntimeIdentityRevalidation::Match,
        "/fixture/empty-mcp.json",
    )
    .expect("accepted T080 planner invocation");

    assert!(
        planner
            .invocation
            .args
            .windows(2)
            .any(|pair| pair == ["--permission-mode", "plan"])
    );
    assert!(
        planner
            .invocation
            .args
            .windows(2)
            .any(|pair| pair == ["--tools", "Read,Glob,Grep"])
    );
    assert!(
        planner
            .invocation
            .args
            .windows(2)
            .any(|pair| pair == ["--disallowedTools", "mcp__*"])
    );
    assert!(
        planner
            .invocation
            .args
            .iter()
            .any(|arg| arg == "--strict-mcp-config")
    );
    assert!(
        planner
            .invocation
            .args
            .iter()
            .any(|arg| arg == "--restricted")
    );
    assert!(
        planner
            .invocation
            .args
            .iter()
            .any(|arg| arg == "--disable-slash-commands")
    );
    assert!(
        planner
            .invocation
            .args
            .iter()
            .any(|arg| arg == "--no-chrome")
    );
}

#[test]
#[ignore = "requires an explicitly governed real Claude Code runtime and sends one real T080 Planner prompt"]
fn t080_live_planner_read_plan_proof() {
    assert_eq!(
        env::var("WINDS_T080_LIVE").as_deref(),
        Ok("1"),
        "WINDS_T080_LIVE=1 is required for the explicitly governed live proof"
    );

    let executable = PathBuf::from(
        env::var("WINDS_T080_CLAUDE_EXECUTABLE")
            .expect("WINDS_T080_CLAUDE_EXECUTABLE must name the exact pre-observed executable"),
    );
    assert!(
        executable.is_absolute(),
        "T080 Claude executable must be absolute"
    );
    let canonical_executable = executable
        .canonicalize()
        .expect("T080 Claude executable must remain available");
    let expected_sha = env::var("WINDS_T080_EXPECTED_SHA256")
        .expect("WINDS_T080_EXPECTED_SHA256 must bind the pre-observed executable bytes");
    assert!(
        expected_sha.len() == 64 && expected_sha.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "T080 expected SHA-256 must be exactly 64 hex characters"
    );
    let metadata = fs::metadata(&canonical_executable).expect("T080 executable metadata");
    assert!(metadata.is_file(), "T080 executable must remain a file");

    let expected_identity = RuntimeExecutableIdentity {
        observed_path: executable.clone(),
        canonical_path: canonical_executable.clone(),
        byte_len: metadata.len(),
        sha256: expected_sha.to_ascii_lowercase(),
    };
    let identity = revalidate_runtime_identity(&expected_identity)
        .expect("T080 runtime identity revalidation must complete");
    assert_eq!(
        identity,
        RuntimeIdentityRevalidation::Match,
        "T080 executable identity must still match the pre-observed exact bytes"
    );

    let expected_version =
        env::var("WINDS_T080_EXPECTED_VERSION").expect("T080 exact version evidence is required");
    let fixture = create_t080_fixture();
    let version_output = run_bounded(
        Command::new(&canonical_executable).arg("--version"),
        &fixture,
        VERSION_TIMEOUT,
        MAX_VERSION_BYTES,
    )
    .expect("bounded Claude version observation");
    let observed_version =
        String::from_utf8(version_output).expect("Claude version output must be UTF-8");
    assert_eq!(
        observed_version.trim(),
        expected_version,
        "T080 exact Claude version changed after preflight"
    );
    let semver = first_semver_triplet(&observed_version)
        .expect("T080 Claude version output must contain a semantic version");
    assert!(
        semver >= MIN_RESTRICTED_VERSION,
        "T080 live proof requires Claude Code >= 2.1.248 for --restricted"
    );

    let empty_mcp = fixture.join("empty-mcp.json");
    let planner = build_t080_planner_invocation(
        ClaudeSessionSelection::New,
        RuntimeIdentityRevalidation::Match,
        empty_mcp
            .to_str()
            .expect("T080 fixture path must be representable as UTF-8"),
    )
    .expect("T080 planner invocation must be accepted");

    let before = fixture_entries(&fixture);
    let mut command = Command::new(&canonical_executable);
    command.args(&planner.invocation.args).arg(planner.prompt);
    let output = run_bounded(
        &mut command,
        &fixture,
        LIVE_TIMEOUT,
        MAX_CLAUDE_STRUCTURED_OUTPUT_BYTES,
    )
    .expect("bounded T080 real Claude Planner proof");
    let parsed = parse_claude_structured_output(&planner.invocation, &output)
        .expect("T080 real Claude output must satisfy the structured result contract");

    assert_eq!(parsed.evidence, ClaudeEvidenceClass::AgentReported);
    assert_eq!(parsed.continuity, ClaudeContinuity::Reconstructed);
    assert_eq!(
        planner.invocation.restriction_enforcement,
        ClaudeRestrictionEnforcement::AgentNativeEnforced
    );
    assert_eq!(
        fs::read(fixture.join("PLANNING.md")).expect("read T080 fixture after proof"),
        T080_FIXTURE_TEXT.as_bytes(),
        "T080 Planner must not mutate the planning fixture"
    );
    assert_eq!(
        fixture_entries(&fixture),
        before,
        "T080 Planner must not add or remove fixture-root entries"
    );

    fs::remove_dir_all(&fixture).expect("remove disposable T080 fixture");
}

const T080_FIXTURE_TEXT: &str =
    "# T080 fixture\n\nGoal: propose a read-only plan for adding a deterministic status command.\nConstraints: no edits, no shell, no network, no MCP, no acceptance claim.\n";

fn create_t080_fixture() -> PathBuf {
    let primary = env::current_dir()
        .expect("current directory")
        .canonicalize()
        .expect("canonical current directory");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let fixture = env::temp_dir().join(format!("winds-t080-{}-{nonce}", std::process::id()));
    fs::create_dir(&fixture).expect("create disposable T080 fixture");
    let fixture = fixture.canonicalize().expect("canonical T080 fixture");
    assert!(
        !fixture.starts_with(&primary) && !primary.starts_with(&fixture),
        "T080 disposable fixture must be outside the primary checkout tree"
    );
    fs::write(fixture.join("PLANNING.md"), T080_FIXTURE_TEXT).expect("write T080 planning fixture");
    fs::write(fixture.join("empty-mcp.json"), b"{\"mcpServers\":{}}\n")
        .expect("write exact empty T080 MCP config");
    fixture
}

fn fixture_entries(root: &Path) -> BTreeSet<String> {
    fs::read_dir(root)
        .expect("read T080 fixture directory")
        .map(|entry| {
            entry
                .expect("T080 fixture entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn run_bounded(
    command: &mut Command,
    cwd: &Path,
    timeout: Duration,
    max_stdout_bytes: usize,
) -> Result<Vec<u8>, String> {
    command
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let mut child = command
        .spawn()
        .map_err(|error| format!("T080 process launch failed: {error}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "T080 process stdout pipe was unavailable".to_owned())?;
    let reader = thread::spawn(move || {
        let mut output = Vec::new();
        stdout
            .by_ref()
            .take((max_stdout_bytes + 1) as u64)
            .read_to_end(&mut output)
            .map(|_| output)
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child
            .try_wait()
            .map_err(|error| format!("T080 process wait failed: {error}"))?
        {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err("T080 bounded process deadline expired".to_owned());
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    };

    let output = reader
        .join()
        .map_err(|_| "T080 stdout reader thread panicked".to_owned())?
        .map_err(|error| format!("T080 stdout read failed: {error}"))?;
    if output.len() > max_stdout_bytes {
        return Err("T080 stdout exceeded its bounded limit".to_owned());
    }
    if !status.success() {
        return Err(format!(
            "T080 process exited unsuccessfully with status {status}"
        ));
    }
    Ok(output)
}

fn first_semver_triplet(value: &str) -> Option<(u64, u64, u64)> {
    value.split_whitespace().find_map(|token| {
        let token = token.trim_matches(|character: char| {
            !character.is_ascii_digit() && character != '.'
        });
        let mut parts = token.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch_text = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        let patch = patch_text
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>()
            .parse()
            .ok()?;
        Some((major, minor, patch))
    })
}
