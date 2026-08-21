use crate::agentic_codex::{
    CodexInbound, CodexProtocolClient, EvidenceClass, MAX_CODEX_JSONL_FRAME_BYTES, NativeThreadId,
    RpcId, ServerRequestDisposition, T079_PROOF_PROMPT,
};
use crate::agentic_runtime::{
    EvidenceSource, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeIdentityRevalidation,
    RuntimeKind, RuntimeVersionState, SafeVersionObservation,
    discover_runtime_from_safe_observations, revalidate_runtime_identity,
};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LIVE_PROOF_TIMEOUT: Duration = Duration::from_secs(120);
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(3);
const GRACEFUL_CHILD_EXIT: Duration = Duration::from_millis(250);
const MAX_CONNECTED_BYTES: usize = 1024 * 1024;
const MAX_CONNECTED_FRAMES: usize = 256;
const MAX_QUEUED_FRAMES: usize = 8;
const MAX_VERSION_BYTES: usize = 4096;
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

type ProofResult<T> = Result<T, String>;
type FrameResult = Result<Vec<u8>, String>;

const ALLOWED_EFFECTIVE_CONFIG_KEYS: &[&str] = &[
    "model",
    "review_model",
    "model_context_window",
    "model_auto_compact_token_limit",
    "model_auto_compact_token_limit_scope",
    "model_provider",
    "approval_policy",
    "approvals_reviewer",
    "sandbox_mode",
    "forced_chatgpt_workspace_id",
    "forced_login_method",
    "model_reasoning_effort",
    "model_reasoning_summary",
    "model_verbosity",
    "service_tier",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResultAuthority {
    AgentRuntimeEvidenceNotVerifiedOrAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestrictionEvidence {
    AgentNativeEnforced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanupEvidence {
    DirectChildReaped,
    DirectChildTerminatedAndReaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct T079Receipt {
    winds_session_id: String,
    native_thread_id: String,
    turn_id: String,
    version: String,
    status: String,
    authority: ResultAuthority,
    restrictions: RestrictionEvidence,
    cleanup: CleanupEvidence,
}

struct EmptyDisposableRootGuard {
    root: PathBuf,
}

impl Drop for EmptyDisposableRootGuard {
    fn drop(&mut self) {
        let Ok(mut entries) = fs::read_dir(&self.root) else {
            return;
        };
        if entries.next().is_none() {
            let _ = fs::remove_dir(&self.root);
        }
    }
}

struct FixtureRootGuard(PathBuf);

impl Drop for FixtureRootGuard {
    fn drop(&mut self) {
        let expected_prefix = format!("winds-t079-discovery-{}-", std::process::id());
        let Ok(temp_root) = env::temp_dir().canonicalize() else {
            return;
        };
        let Ok(root) = self.0.canonicalize() else {
            return;
        };
        if root.parent() != Some(temp_root.as_path())
            || !root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&expected_prefix))
        {
            return;
        }
        let executable = root.join(if cfg!(windows) { "codex.exe" } else { "codex" });
        let _ = fs::remove_file(executable);
        let _ = fs::remove_dir(root);
    }
}

fn parse_outbound(line: &str) -> Value {
    assert!(line.ends_with('\n'));
    serde_json::from_str(line.trim_end()).expect("T079 outbound JSONL must be valid JSON")
}

fn initialized_client() -> CodexProtocolClient {
    let mut client = CodexProtocolClient::default();
    client
        .t079_initialize_request("winds", "Winds", "0.1.0")
        .expect("T079 initialize request");
    assert_eq!(
        client
            .ingest_jsonl_frame(br#"{"id":0,"result":{"userAgent":"fixture"}}"#)
            .expect("initialize response"),
        CodexInbound::InitializeAccepted
    );
    client
        .initialized_notification()
        .expect("initialized notification");
    assert!(client.is_ready());
    client
}

fn meaningful(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(false) => false,
        Value::String(text) => {
            let normalized = text.trim().to_ascii_lowercase();
            !normalized.is_empty() && normalized != "disabled" && normalized != "off"
        }
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Bool(true) | Value::Number(_) => true,
    }
}

fn validate_effective_config(result: &Value) -> ProofResult<()> {
    let config = result
        .get("config")
        .and_then(Value::as_object)
        .ok_or_else(|| "T079 config/read response is missing effective config".to_owned())?;

    for (key, value) in config {
        if meaningful(value) && !ALLOWED_EFFECTIVE_CONFIG_KEYS.contains(&key.as_str()) {
            return Err(format!(
                "T079 refuses connected proof with active or ambiguous effective config: {key}"
            ));
        }
    }
    Ok(())
}

fn validate_thread_start_result(result: &Value) -> ProofResult<NativeThreadId> {
    let thread = result
        .get("thread")
        .ok_or_else(|| "T079 thread/start response is missing thread".to_owned())?;
    if thread.get("ephemeral").and_then(Value::as_bool) != Some(true) {
        return Err("T079 requires an ephemeral Codex thread".to_owned());
    }
    if !thread.get("path").is_some_and(Value::is_null) {
        return Err("T079 ephemeral Codex thread unexpectedly has a persisted path".to_owned());
    }
    if result.get("approvalPolicy").and_then(Value::as_str) != Some("never") {
        return Err("T079 Codex thread did not confirm approvalPolicy=never".to_owned());
    }
    let sandbox = result
        .get("sandbox")
        .and_then(Value::as_object)
        .ok_or_else(|| "T079 thread/start response is missing sandbox evidence".to_owned())?;
    if sandbox.get("type").and_then(Value::as_str) != Some("readOnly")
        || sandbox.get("networkAccess").and_then(Value::as_bool) != Some(false)
    {
        return Err("T079 Codex thread did not confirm read-only/no-network sandbox".to_owned());
    }
    let runtime_workspace_roots = result
        .get("runtimeWorkspaceRoots")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "T079 thread/start response is missing runtime workspace roots".to_owned()
        })?;
    if !runtime_workspace_roots.is_empty() {
        return Err("T079 Codex thread retained runtime workspace roots".to_owned());
    }
    let instruction_sources = result
        .get("instructionSources")
        .and_then(Value::as_array)
        .ok_or_else(|| "T079 thread/start response is missing instruction sources".to_owned())?;
    if !instruction_sources.is_empty() {
        return Err(
            "T079 Codex thread loaded instruction sources into the bounded proof".to_owned(),
        );
    }
    NativeThreadId::from_thread_result(result).map_err(|error| error.to_string())
}

fn turn_id_from_start_result(result: &Value) -> ProofResult<String> {
    let turn_id = result
        .get("turn")
        .and_then(|turn| turn.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "T079 turn/start response is missing turn id".to_owned())?;
    validate_exact_text(turn_id, "Codex native turn id")?;
    Ok(turn_id.to_owned())
}

fn parse_structured_agent_message(text: &str) -> ProofResult<String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| format!("T079 final agent message is not JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "T079 final agent message must be a JSON object".to_owned())?;
    if object.len() != 1 || object.get("status").and_then(Value::as_str) != Some("WINDS_T079_OK") {
        return Err("T079 final agent message does not match the fixed output contract".to_owned());
    }
    Ok("WINDS_T079_OK".to_owned())
}

fn validate_exact_text(value: &str, label: &str) -> ProofResult<()> {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        return Err(format!("{label} is not an exact safe text identity"));
    }
    Ok(())
}

fn validate_discovery(discovery: &RuntimeDiscovery) -> ProofResult<()> {
    if discovery.runtime != RuntimeKind::Codex {
        return Err("T079 requires a Codex runtime discovery".to_owned());
    }
    if discovery.state != RuntimeDiscoveryState::Present {
        return Err("T079 requires a present supported Codex runtime".to_owned());
    }
    if discovery.version.state != RuntimeVersionState::Observed
        || discovery.version.source != EvidenceSource::WindsLocallyObserved
        || discovery.version.value.as_deref().is_none()
    {
        return Err("T079 requires exact locally observed Codex version evidence".to_owned());
    }
    let executable = discovery
        .executable
        .as_ref()
        .ok_or_else(|| "T079 present Codex discovery is missing executable identity".to_owned())?;
    match revalidate_runtime_identity(executable).map_err(|error| error.to_string())? {
        RuntimeIdentityRevalidation::Match => Ok(()),
        RuntimeIdentityRevalidation::Changed => {
            Err("T079 Codex executable identity changed after discovery".to_owned())
        }
        RuntimeIdentityRevalidation::Unavailable => {
            Err("T079 Codex executable became unavailable after discovery".to_owned())
        }
    }
}

fn observe_version_bounded(executable: &Path) -> ProofResult<String> {
    let mut child = Command::new(executable)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("T079 could not execute Codex --version: {error}"))?;
    let deadline = Instant::now() + VERSION_TIMEOUT;
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("T079 could not inspect Codex --version: {error}"))?
        {
            Some(status) => {
                if !status.success() {
                    return Err(format!("T079 Codex --version failed with status {status}"));
                }
                break;
            }
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("T079 Codex --version exceeded bounded timeout".to_owned());
            }
        }
    }

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "T079 Codex --version stdout was unavailable".to_owned())?
        .take((MAX_VERSION_BYTES + 1) as u64);
    let mut bytes = Vec::new();
    stdout
        .read_to_end(&mut bytes)
        .map_err(|error| format!("T079 could not read Codex --version: {error}"))?;
    if bytes.len() > MAX_VERSION_BYTES {
        return Err("T079 Codex --version output exceeded bounded size".to_owned());
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| "T079 Codex --version output is not UTF-8".to_owned())?;
    let version = text.trim_end_matches(['\r', '\n']);
    validate_exact_text(version, "Codex version")?;
    if version.contains('\r') || version.contains('\n') {
        return Err("T079 Codex --version must be exactly one line".to_owned());
    }
    Ok(version.to_owned())
}

fn disposable_root() -> ProofResult<PathBuf> {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("T079 clock error: {error}"))?
        .as_nanos();
    let root = env::temp_dir().join(format!(
        "winds-t079-{}-{sequence}-{nanos}",
        std::process::id()
    ));
    fs::create_dir(&root)
        .map_err(|error| format!("T079 could not create disposable root: {error}"))?;
    root.canonicalize()
        .map_err(|error| format!("T079 could not canonicalize disposable root: {error}"))
}

fn spawn_frame_reader_with_sender(
    stdout: std::process::ChildStdout,
    sender: SyncSender<FrameResult>,
    done: SyncSender<()>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut frame = Vec::new();
            match reader.read_until(b'\n', &mut frame) {
                Ok(0) => break,
                Ok(_) if frame.len() > MAX_CODEX_JSONL_FRAME_BYTES => {
                    let _ = sender.send(Err("T079 Codex frame exceeded bounded size".to_owned()));
                    break;
                }
                Ok(_) => {
                    if sender.send(Ok(frame)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(format!("T079 Codex stdout read failed: {error}")));
                    break;
                }
            }
        }
        let _ = done.send(());
    })
}

fn receive_frame(
    receiver: &Receiver<FrameResult>,
    deadline: Instant,
    total_bytes: &mut usize,
    frame_count: &mut usize,
) -> ProofResult<Vec<u8>> {
    let now = Instant::now();
    if now >= deadline {
        return Err("T079 connected proof timed out".to_owned());
    }
    let frame = receiver
        .recv_timeout(deadline.saturating_duration_since(now))
        .map_err(|error| format!("T079 Codex frame unavailable: {error}"))??;
    *frame_count += 1;
    *total_bytes = total_bytes
        .checked_add(frame.len())
        .ok_or_else(|| "T079 connected output byte count overflowed".to_owned())?;
    if *frame_count > MAX_CONNECTED_FRAMES || *total_bytes > MAX_CONNECTED_BYTES {
        return Err("T079 connected output exceeded bounded transcript limits".to_owned());
    }
    Ok(frame)
}

fn wait_for_reader_completion(receiver: &Receiver<()>, deadline: Instant) -> ProofResult<()> {
    let now = Instant::now();
    if now >= deadline {
        return Err("T079 stdout reader did not terminate inside bounded cleanup".to_owned());
    }
    receiver
        .recv_timeout(deadline.saturating_duration_since(now))
        .map_err(|_| "T079 stdout reader did not terminate inside bounded cleanup".to_owned())
}

fn send_line(stdin: &mut std::process::ChildStdin, line: &str) -> ProofResult<()> {
    stdin
        .write_all(line.as_bytes())
        .and_then(|_| stdin.flush())
        .map_err(|error| format!("T079 could not write Codex JSONL request: {error}"))
}

fn is_forbidden_activity(method: &str, params: &Value) -> bool {
    let method_lower = method.to_ascii_lowercase();
    if method_lower.contains("commandexecution")
        || method_lower.contains("filechange")
        || method_lower.contains("mcp")
        || method_lower.contains("tool")
        || method_lower.contains("hook")
        || method_lower.contains("websearch")
        || method_lower.contains("imagegeneration")
        || method_lower.contains("collabagent")
        || method_lower.contains("subagent")
        || method_lower.contains("turn/diff")
    {
        return true;
    }

    params
        .get("item")
        .and_then(|item| item.get("type"))
        .and_then(Value::as_str)
        .is_some_and(|kind| {
            !matches!(
                kind,
                "userMessage" | "agentMessage" | "plan" | "reasoning" | "contextCompaction"
            )
        })
}

fn handle_server_request(
    client: &CodexProtocolClient,
    stdin: &mut std::process::ChildStdin,
    id: &RpcId,
    method: &str,
) -> ProofResult<()> {
    if matches!(
        method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
    ) {
        let decline = client.t079_decline(id).map_err(|error| error.to_string())?;
        send_line(stdin, &decline)?;
        return Err(format!(
            "T079 Codex requested unexpected authority and was explicitly declined: {method}"
        ));
    }
    Err(format!(
        "T079 refuses unexpected server-initiated request without granting it: {method}"
    ))
}

fn wait_for_response(
    client: &mut CodexProtocolClient,
    stdin: &mut std::process::ChildStdin,
    receiver: &Receiver<FrameResult>,
    expected_id: u64,
    deadline: Instant,
    total_bytes: &mut usize,
    frame_count: &mut usize,
) -> ProofResult<Value> {
    loop {
        let frame = receive_frame(receiver, deadline, total_bytes, frame_count)?;
        match client
            .ingest_jsonl_frame(&frame)
            .map_err(|error| format!("T079 Codex protocol failure: {error}"))?
        {
            CodexInbound::Response {
                id: RpcId::Number(id),
                result,
                evidence: EvidenceClass::AgentRuntimeEvidence,
            } if id == expected_id => return Ok(result),
            CodexInbound::ErrorResponse { id, error, .. } => {
                return Err(format!("T079 Codex request {id:?} failed: {error}"));
            }
            CodexInbound::Notification { method, params, .. } => {
                if is_forbidden_activity(&method, &params) {
                    return Err(format!(
                        "T079 observed forbidden runtime activity: {method}"
                    ));
                }
            }
            CodexInbound::ServerRequest {
                id,
                method,
                disposition: ServerRequestDisposition::RequiresExternalDecision,
                ..
            } => handle_server_request(client, stdin, &id, &method)?,
            other => {
                return Err(format!(
                    "T079 received response/event that cannot satisfy request {expected_id}: {other:?}"
                ));
            }
        }
    }
}

fn hand_off_child_reap(mut child: Child) -> ProofResult<()> {
    thread::Builder::new()
        .name("winds-t079-child-reaper".to_owned())
        .spawn(move || {
            let _ = child.wait();
        })
        .map(|_| ())
        .map_err(|error| {
            format!("T079 could not hand terminated child to background reaper: {error}")
        })
}

fn finish_child(mut child: Child, cleanup_deadline: Instant) -> ProofResult<CleanupEvidence> {
    let graceful_deadline = std::cmp::min(Instant::now() + GRACEFUL_CHILD_EXIT, cleanup_deadline);
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("T079 could not inspect owned Codex child: {error}"))?
        {
            Some(_) => return Ok(CleanupEvidence::DirectChildReaped),
            None if Instant::now() < graceful_deadline => thread::sleep(Duration::from_millis(10)),
            None => break,
        }
    }

    if let Err(kill_error) = child.kill() {
        return match child.try_wait() {
            Ok(Some(_)) => Ok(CleanupEvidence::DirectChildReaped),
            Ok(None) => {
                let message = format!(
                    "T079 could not terminate owned Codex child: {kill_error}; direct-child cleanup remains unproven"
                );
                hand_off_child_reap(child)
                    .map_err(|reaper_error| format!("{message}; {reaper_error}"))?;
                Err(format!("{message}; direct-child reap handed off"))
            }
            Err(wait_error) => {
                let message = format!(
                    "T079 could not terminate owned Codex child: {kill_error}; reap state is also unproven: {wait_error}"
                );
                hand_off_child_reap(child)
                    .map_err(|reaper_error| format!("{message}; {reaper_error}"))?;
                Err(format!("{message}; direct-child reap handed off"))
            }
        };
    }

    loop {
        match child
            .try_wait()
            .map_err(|error| format!("T079 could not reap terminated Codex child: {error}"))?
        {
            Some(_) => return Ok(CleanupEvidence::DirectChildTerminatedAndReaped),
            None if Instant::now() < cleanup_deadline => thread::sleep(Duration::from_millis(10)),
            None => {
                let message = "T079 terminated the owned Codex child but could not prove reap inside bounded cleanup";
                hand_off_child_reap(child)
                    .map_err(|reaper_error| format!("{message}; {reaper_error}"))?;
                return Err(format!("{message}; direct-child reap handed off"));
            }
        }
    }
}

fn ensure_disposable_root_unchanged(root: &Path) -> ProofResult<()> {
    let mut entries = fs::read_dir(root)
        .map_err(|error| format!("T079 could not inspect disposable root: {error}"))?;
    if entries
        .next()
        .transpose()
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err(format!(
            "T079 disposable proof context was mutated; preserved for inspection at {}",
            root.display()
        ));
    }
    fs::remove_dir(root)
        .map_err(|error| format!("T079 could not remove unchanged disposable root: {error}"))
}

fn reconcile_proof_cleanup<T>(
    proof: ProofResult<T>,
    cleanup: ProofResult<CleanupEvidence>,
    reader_result: ProofResult<()>,
    root_check: ProofResult<()>,
) -> ProofResult<(T, CleanupEvidence)> {
    match (proof, cleanup, reader_result, root_check) {
        (Ok(value), Ok(cleanup), Ok(()), Ok(())) => Ok((value, cleanup)),
        (proof, cleanup, reader_result, root_check) => {
            let mut failures = Vec::new();
            if let Err(error) = proof {
                failures.push(format!("proof={error}"));
            }
            if let Err(error) = cleanup {
                failures.push(format!("child_cleanup={error}"));
            }
            if let Err(error) = reader_result {
                failures.push(format!("reader_cleanup={error}"));
            }
            if let Err(error) = root_check {
                failures.push(format!("root_cleanup={error}"));
            }
            Err(format!(
                "T079 proof/cleanup failure: {}",
                failures.join("; ")
            ))
        }
    }
}

fn cleanup_setup_failure(child: Child, root: &Path, message: &str) -> String {
    let cleanup_deadline = Instant::now() + CLEANUP_TIMEOUT;
    let cleanup = finish_child(child, cleanup_deadline);
    let root_check = ensure_disposable_root_unchanged(root);
    match (cleanup, root_check) {
        (Ok(_), Ok(())) => message.to_owned(),
        (cleanup, root_check) => {
            format!("{message}; setup cleanup evidence: child={cleanup:?}; root={root_check:?}")
        }
    }
}

fn run_connected_proof(
    discovery: &RuntimeDiscovery,
    winds_session_id: &str,
) -> ProofResult<T079Receipt> {
    validate_exact_text(winds_session_id, "Winds session id")?;
    validate_discovery(discovery)?;
    let executable = discovery
        .executable
        .as_ref()
        .ok_or_else(|| "T079 discovery lost executable identity".to_owned())?;
    let version = observe_version_bounded(&executable.observed_path)?;
    if discovery.version.value.as_deref() != Some(version.as_str()) {
        return Err("T079 Codex version changed after discovery".to_owned());
    }
    if revalidate_runtime_identity(executable).map_err(|error| error.to_string())?
        != RuntimeIdentityRevalidation::Match
    {
        return Err("T079 Codex executable changed immediately before launch".to_owned());
    }

    let root = disposable_root()?;
    let _root_guard = EmptyDisposableRootGuard { root: root.clone() };
    let cwd = root
        .to_str()
        .ok_or_else(|| "T079 disposable root is not UTF-8".to_owned())?
        .to_owned();
    let mut child = Command::new(&executable.observed_path)
        .args(["app-server", "--stdio"])
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("T079 could not start owned Codex App Server: {error}"))?;
    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            let message = cleanup_setup_failure(child, &root, "T079 Codex child stdin unavailable");
            return Err(message);
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            drop(stdin);
            let message =
                cleanup_setup_failure(child, &root, "T079 Codex child stdout unavailable");
            return Err(message);
        }
    };
    let (sender, receiver) = mpsc::sync_channel(MAX_QUEUED_FRAMES);
    let (done_sender, done_receiver) = mpsc::sync_channel(1);
    let reader = spawn_frame_reader_with_sender(stdout, sender, done_sender);
    let deadline = Instant::now() + LIVE_PROOF_TIMEOUT;
    let mut total_bytes = 0usize;
    let mut frame_count = 0usize;

    let proof = (|| -> ProofResult<(String, String, String)> {
        let mut client = CodexProtocolClient::default();
        let initialize = client
            .t079_initialize_request("winds", "Winds", "0.1.0")
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &initialize)?;
        let frame = receive_frame(&receiver, deadline, &mut total_bytes, &mut frame_count)?;
        if client
            .ingest_jsonl_frame(&frame)
            .map_err(|error| error.to_string())?
            != CodexInbound::InitializeAccepted
        {
            return Err("T079 did not receive the required initialize response".to_owned());
        }
        let initialized = client
            .initialized_notification()
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &initialized)?;

        let (config_id, config_request) = client
            .t079_config_read(&cwd)
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &config_request)?;
        let config = wait_for_response(
            &mut client,
            &mut stdin,
            &receiver,
            config_id,
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        validate_effective_config(&config)?;

        let (thread_id, thread_request) = client
            .t079_thread_start(&cwd)
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &thread_request)?;
        let thread_result = wait_for_response(
            &mut client,
            &mut stdin,
            &receiver,
            thread_id,
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        let native_thread_id = validate_thread_start_result(&thread_result)?;
        if native_thread_id.as_str() == winds_session_id {
            return Err(
                "T079 native thread id must not alias canonical Winds session id".to_owned(),
            );
        }

        let (turn_request_id, turn_request) = client
            .t079_turn_start(&native_thread_id, &cwd)
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &turn_request)?;
        let turn_result = wait_for_response(
            &mut client,
            &mut stdin,
            &receiver,
            turn_request_id,
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        let turn_id = turn_id_from_start_result(&turn_result)?;
        let mut final_agent_message: Option<String> = None;

        loop {
            let frame = receive_frame(&receiver, deadline, &mut total_bytes, &mut frame_count)?;
            match client
                .ingest_jsonl_frame(&frame)
                .map_err(|error| format!("T079 Codex protocol failure: {error}"))?
            {
                CodexInbound::Notification {
                    method,
                    params,
                    evidence: EvidenceClass::AgentRuntimeEvidence,
                } => {
                    if is_forbidden_activity(&method, &params) {
                        return Err(format!(
                            "T079 observed forbidden runtime activity: {method}"
                        ));
                    }
                    if method == "item/completed"
                        && let Some(item) = params.get("item")
                        && item.get("type").and_then(Value::as_str) == Some("agentMessage")
                    {
                        let text = item.get("text").and_then(Value::as_str).ok_or_else(|| {
                            "T079 completed agent message is missing text".to_owned()
                        })?;
                        final_agent_message = Some(text.to_owned());
                    }
                    if method == "turn/completed" {
                        let turn = params
                            .get("turn")
                            .ok_or_else(|| "T079 turn/completed is missing turn".to_owned())?;
                        if turn.get("id").and_then(Value::as_str) != Some(turn_id.as_str()) {
                            return Err("T079 turn/completed identity mismatch".to_owned());
                        }
                        if turn.get("status").and_then(Value::as_str) != Some("completed") {
                            return Err("T079 Codex turn did not complete successfully".to_owned());
                        }
                        let text = final_agent_message.as_deref().ok_or_else(|| {
                            "T079 completed without a bounded final agent message".to_owned()
                        })?;
                        let status = parse_structured_agent_message(text)?;
                        return Ok((native_thread_id.as_str().to_owned(), turn_id, status));
                    }
                }
                CodexInbound::ServerRequest {
                    id,
                    method,
                    disposition: ServerRequestDisposition::RequiresExternalDecision,
                    ..
                } => handle_server_request(&client, &mut stdin, &id, &method)?,
                CodexInbound::ErrorResponse { id, error, .. } => {
                    return Err(format!("T079 Codex runtime error {id:?}: {error}"));
                }
                CodexInbound::Response { .. } | CodexInbound::InitializeAccepted => {
                    return Err("T079 received an unexpected response after turn/start".to_owned());
                }
            }
        }
    })();

    drop(stdin);
    drop(receiver);
    let cleanup_deadline = Instant::now() + CLEANUP_TIMEOUT;
    let cleanup = finish_child(child, cleanup_deadline);
    let reader_result = match wait_for_reader_completion(&done_receiver, cleanup_deadline) {
        Ok(()) => reader
            .join()
            .map_err(|_| "T079 stdout reader panicked during bounded cleanup".to_owned()),
        Err(error) => Err(error),
    };
    let root_check = ensure_disposable_root_unchanged(&root);

    let ((native_thread_id, turn_id, status), cleanup) =
        reconcile_proof_cleanup(proof, cleanup, reader_result, root_check)?;
    Ok(T079Receipt {
        winds_session_id: winds_session_id.to_owned(),
        native_thread_id,
        turn_id,
        version,
        status,
        authority: ResultAuthority::AgentRuntimeEvidenceNotVerifiedOrAccepted,
        restrictions: RestrictionEvidence::AgentNativeEnforced,
        cleanup,
    })
}

#[test]
fn t079_requests_are_fixed_ephemeral_read_only_and_non_authorizing() {
    let mut handshake = CodexProtocolClient::default();
    let initialize = handshake
        .t079_initialize_request("winds", "Winds", "0.1.0")
        .expect("T079 initialize request");
    let initialize = parse_outbound(&initialize);
    assert_eq!(initialize["method"], "initialize");
    assert_eq!(
        initialize["params"]["capabilities"]["experimentalApi"],
        true
    );
    assert_eq!(
        initialize["params"]["capabilities"]
            .as_object()
            .expect("capabilities object")
            .len(),
        1
    );

    let mut client = initialized_client();
    let cwd = "/tmp/winds-t079-fixture";

    let (config_id, config) = client.t079_config_read(cwd).expect("config request");
    assert_eq!(config_id, 1);
    assert_eq!(
        parse_outbound(&config),
        json!({
            "method": "config/read",
            "id": 1,
            "params": { "cwd": cwd, "includeLayers": false }
        })
    );

    let (thread_id, thread) = client.t079_thread_start(cwd).expect("thread request");
    assert_eq!(thread_id, 2);
    assert_eq!(
        parse_outbound(&thread),
        json!({
            "method": "thread/start",
            "id": 2,
            "params": {
                "cwd": cwd,
                "runtimeWorkspaceRoots": [],
                "approvalPolicy": "never",
                "sandbox": "read-only",
                "ephemeral": true,
                "environments": [],
                "dynamicTools": [],
                "selectedCapabilityRoots": []
            }
        })
    );

    let native = NativeThreadId::parse("thr_t079_fixture").expect("native id");
    let (turn_id, turn) = client.t079_turn_start(&native, cwd).expect("turn request");
    assert_eq!(turn_id, 3);
    let turn = parse_outbound(&turn);
    assert_eq!(turn["method"], "turn/start");
    assert_eq!(turn["id"], 3);
    assert_eq!(turn["params"]["threadId"], "thr_t079_fixture");
    assert_eq!(turn["params"]["input"][0]["text"], T079_PROOF_PROMPT);
    assert_eq!(turn["params"]["runtimeWorkspaceRoots"], json!([]));
    assert_eq!(turn["params"]["environments"], json!([]));
    assert_eq!(turn["params"]["approvalPolicy"], "never");
    assert_eq!(turn["params"]["sandboxPolicy"]["type"], "readOnly");
    assert_eq!(turn["params"]["sandboxPolicy"]["networkAccess"], false);
    assert_eq!(
        turn["params"]["outputSchema"]["properties"]["status"]["const"],
        "WINDS_T079_OK"
    );
    let serialized = turn.to_string();
    assert!(!serialized.contains("dangerFullAccess"));
    assert!(!serialized.contains("workspaceWrite"));
    assert!(!serialized.contains("mcp"));
    assert!(!serialized.contains("model\""));
}

#[test]
fn effective_config_preflight_rejects_side_channel_surfaces() {
    validate_effective_config(&json!({
        "config": {
            "model": "gpt-fixture",
            "mcp_servers": null,
            "hooks": [],
            "apps": null,
            "instructions": null,
            "developer_instructions": null,
            "tools": null,
            "web_search": "disabled"
        }
    }))
    .expect("allowed model fields plus empty/disabled surfaces are acceptable");

    for (key, value) in [
        ("mcp_servers", json!({"server": {"command": "x"}})),
        ("mcpServers", json!({"server": {"command": "x"}})),
        ("hooks", json!([{"event": "SessionStart"}])),
        ("apps", json!({"demo": {"enabled": true}})),
        ("instructions", json!("use tools")),
        ("developer_instructions", json!("run a command")),
        ("tools", json!({"web_search": {"enabled": true}})),
        ("web_search", json!("live")),
        ("future_side_channel", json!({"enabled": true})),
    ] {
        let mut config = serde_json::Map::new();
        config.insert(key.to_owned(), value);
        let error = validate_effective_config(&json!({ "config": config })).unwrap_err();
        assert!(error.contains(key));
    }
}

#[test]
fn thread_and_result_validation_preserve_exact_provenance_and_non_authority() {
    let native = validate_thread_start_result(&json!({
        "thread": { "id": "thr_exact", "ephemeral": true, "path": null },
        "approvalPolicy": "never",
        "sandbox": { "type": "readOnly", "networkAccess": false },
        "runtimeWorkspaceRoots": [],
        "instructionSources": []
    }))
    .expect("exact T079 thread evidence");
    assert_eq!(native.as_str(), "thr_exact");
    assert!(
        validate_thread_start_result(&json!({
            "thread": { "id": "thr_exact", "ephemeral": false, "path": "/persisted" },
            "approvalPolicy": "never",
            "sandbox": { "type": "readOnly", "networkAccess": false },
            "runtimeWorkspaceRoots": [],
            "instructionSources": []
        }))
        .is_err()
    );
    assert!(
        validate_thread_start_result(&json!({
            "thread": { "id": "thr_exact", "ephemeral": true, "path": null },
            "approvalPolicy": "never",
            "sandbox": { "type": "readOnly", "networkAccess": false },
            "runtimeWorkspaceRoots": [],
            "instructionSources": ["/home/user/AGENTS.md"]
        }))
        .is_err()
    );
    assert!(
        validate_thread_start_result(&json!({
            "thread": { "id": "thr_exact", "ephemeral": true, "path": null },
            "approvalPolicy": "never",
            "sandbox": { "type": "readOnly", "networkAccess": false },
            "runtimeWorkspaceRoots": ["/tmp/winds-t079-fixture"],
            "instructionSources": []
        }))
        .is_err()
    );
    assert_eq!(
        parse_structured_agent_message(r#"{"status":"WINDS_T079_OK"}"#).unwrap(),
        "WINDS_T079_OK"
    );
    assert!(
        parse_structured_agent_message(r#"{"status":"WINDS_T079_OK","verified":true}"#).is_err()
    );
    assert!(parse_structured_agent_message(r#"{"status":"OTHER"}"#).is_err());
}

#[test]
fn approval_response_is_decline_only_and_unknown_requests_never_authorize() {
    let client = initialized_client();
    let decline = parse_outbound(
        &client
            .t079_decline(&RpcId::Text("approval-1".to_owned()))
            .expect("decline response"),
    );
    assert_eq!(
        decline,
        json!({ "id": "approval-1", "result": { "decision": "decline" } })
    );
    assert!(!decline.to_string().contains("accept"));
    assert!(!decline.to_string().contains("approve"));
}

#[test]
fn forbidden_runtime_activity_is_detected_fail_closed() {
    for (method, params) in [
        ("item/commandExecution/outputDelta", json!({})),
        ("item/fileChange/outputDelta", json!({})),
        ("mcp/tool/call", json!({})),
        ("turn/diff/updated", json!({})),
        ("hook/started", json!({})),
        (
            "item/completed",
            json!({"item": {"type": "dynamicToolCall"}}),
        ),
        (
            "item/started",
            json!({"item": {"type": "collabAgentToolCall"}}),
        ),
        ("item/started", json!({"item": {"type": "webSearch"}})),
        ("item/started", json!({"item": {"type": "hookPrompt"}})),
        (
            "item/started",
            json!({"item": {"type": "futureUnknownToolSurface"}}),
        ),
    ] {
        assert!(is_forbidden_activity(method, &params), "{method}");
    }
    for kind in [
        "userMessage",
        "agentMessage",
        "plan",
        "reasoning",
        "contextCompaction",
    ] {
        assert!(!is_forbidden_activity(
            "item/completed",
            &json!({"item": {"type": kind, "text": "{}"}})
        ));
    }
}

#[test]
fn reader_completion_wait_is_bounded_and_fail_closed() {
    let (_sender, receiver) = mpsc::sync_channel::<()>(1);
    let deadline = Instant::now() + Duration::from_millis(20);
    let error = wait_for_reader_completion(&receiver, deadline).unwrap_err();
    assert!(error.contains("stdout reader"));
}

#[test]
fn proof_error_preserves_cleanup_failures() {
    let error = reconcile_proof_cleanup::<()>(
        Err("primary proof failure".to_owned()),
        Err("child cleanup failure".to_owned()),
        Err("reader cleanup failure".to_owned()),
        Err("root cleanup failure".to_owned()),
    )
    .unwrap_err();
    for expected in [
        "primary proof failure",
        "child cleanup failure",
        "reader cleanup failure",
        "root cleanup failure",
    ] {
        assert!(error.contains(expected));
    }
}

#[test]
fn runtime_identity_must_match_exact_codex_discovery_before_launch() {
    let root = env::temp_dir().join(format!(
        "winds-t079-discovery-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let _root_guard = FixtureRootGuard(root.clone());
    let executable = root.join(if cfg!(windows) { "codex.exe" } else { "codex" });
    fs::write(&executable, b"fixture-codex-v1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).unwrap();
    }

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("codex-cli fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();
    validate_discovery(&discovery).expect("exact identity");

    fs::write(&executable, b"fixture-codex-x1\n").unwrap();
    assert!(validate_discovery(&discovery).is_err());
}

#[test]
fn native_thread_identity_never_aliases_winds_session_identity_in_receipt_contract() {
    let receipt = T079Receipt {
        winds_session_id: "winds-session-1".to_owned(),
        native_thread_id: "thr_native_1".to_owned(),
        turn_id: "turn_native_1".to_owned(),
        version: "codex-cli fixture".to_owned(),
        status: "WINDS_T079_OK".to_owned(),
        authority: ResultAuthority::AgentRuntimeEvidenceNotVerifiedOrAccepted,
        restrictions: RestrictionEvidence::AgentNativeEnforced,
        cleanup: CleanupEvidence::DirectChildReaped,
    };
    assert_ne!(receipt.winds_session_id, receipt.native_thread_id);
    assert_eq!(
        receipt.authority,
        ResultAuthority::AgentRuntimeEvidenceNotVerifiedOrAccepted
    );
    assert_eq!(
        receipt.restrictions,
        RestrictionEvidence::AgentNativeEnforced
    );
}

#[test]
#[ignore = "T079 live proof: requires a pre-existing locally authenticated Codex executable; no install/auth/terms automation"]
fn t079_real_codex_one_bounded_prompt() {
    let executable = PathBuf::from(
        env::var("WINDS_T079_CODEX_PATH")
            .expect("set WINDS_T079_CODEX_PATH to an existing Codex executable"),
    );
    let winds_session_id = env::var("WINDS_T079_WINDS_SESSION_ID")
        .expect("set WINDS_T079_WINDS_SESSION_ID to the exact canonical Winds session id");
    let version = observe_version_bounded(&executable).expect("bounded Codex version observation");
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed(version),
        &[],
        &[],
    )
    .expect("exact Codex discovery");
    let receipt = run_connected_proof(&discovery, &winds_session_id).expect("bounded T079 proof");

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "T079_REAL_CODEX_PROOF": "PASS",
            "winds_session_id": receipt.winds_session_id,
            "native_thread_id": receipt.native_thread_id,
            "turn_id": receipt.turn_id,
            "version": receipt.version,
            "structured_status": receipt.status,
            "authority": "AGENT_RUNTIME_EVIDENCE_NOT_VERIFIED_OR_ACCEPTED",
            "restrictions": "AGENT_NATIVE_ENFORCED",
            "cleanup": format!("{:?}", receipt.cleanup),
            "experimental_api_opt_in": true,
            "environment_access": "disabled",
            "runtime_workspace_roots": 0,
            "instruction_sources": 0,
            "prompt_sent": true,
            "turns": 1,
            "primary_checkout_mutation": false,
            "credential_or_terms_automation": false,
            "mcp_runtime": false,
            "automatic_landing": false
        }))
        .unwrap()
    );
}
