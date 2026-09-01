use crate::agentic_codex::{
    CodexInbound, CodexProtocolClient, CodexProtocolError, EvidenceClass,
    MAX_CODEX_JSONL_FRAME_BYTES, NativeThreadId, RpcId, ServerRequestDisposition,
    T079_PROOF_PROMPT,
};
use crate::agentic_runtime::{
    EvidenceSource, RuntimeDiscovery, RuntimeDiscoveryState, RuntimeExecutableIdentity,
    RuntimeIdentityRevalidation, RuntimeKind, RuntimeVersionState, SafeVersionObservation,
    discover_runtime_from_safe_observations, revalidate_runtime_identity,
};
use serde_json::{Value, json};
#[cfg(target_os = "linux")]
use sha2::{Digest, Sha256};
use std::env;
#[cfg(target_os = "linux")]
use std::error::Error;
#[cfg(target_os = "linux")]
use std::ffi::CString;
use std::fs;
#[cfg(target_os = "linux")]
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(target_os = "linux")]
use std::io::{Seek, SeekFrom};
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
#[cfg(target_os = "linux")]
#[allow(clippy::items_after_test_module)]
mod process_scope {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/process_scope.rs"));

    impl OwnedProcess {
        pub(super) fn terminate_direct_t079(
            &mut self,
            deadline: Instant,
            label: &str,
        ) -> Result<()> {
            match self.child.kill() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::InvalidInput => {}
                Err(error) => {
                    return Err(format!(
                        "{label} failed to terminate its T079 direct child: {error}"
                    )
                    .into());
                }
            }

            loop {
                match self.child.try_wait().map_err(|error| {
                    format!("{label} failed while reaping its T079 direct child: {error}")
                })? {
                    Some(_) => {
                        self.disarm_unix_process_group();
                        return Ok(());
                    }
                    None => {
                        let now = Instant::now();
                        if now >= deadline {
                            return Err(format!(
                                "{label} T079 direct child could not be proven reaped inside the bounded cleanup window"
                            )
                            .into());
                        }
                        thread::sleep(POLL_INTERVAL.min(deadline.saturating_duration_since(now)));
                    }
                }
            }
        }
    }
}
#[cfg(target_os = "linux")]
use process_scope::{OwnedProcess, spawn_owned_process};

const LIVE_PROOF_TIMEOUT: Duration = Duration::from_secs(120);
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(3);
const GRACEFUL_CHILD_EXIT: Duration = Duration::from_millis(250);
const MAX_CONNECTED_BYTES: usize = 1024 * 1024;
const MAX_CONNECTED_FRAMES: usize = 256;
const MAX_QUEUED_FRAMES: usize = 8;
const MAX_VERSION_BYTES: usize = 4096;
const BLOCKED_CODEX_CONFIG_FILES: &[&str] = &[
    "config.toml",
    "managed_config.toml",
    "requirements.toml",
    "hooks.json",
];
#[cfg(windows)]
const SAFE_CODEX_CHILD_ENV_KEYS: &[&str] = &["SystemRoot", "WINDIR", "TEMP", "TMP"];
#[cfg(not(windows))]
const SAFE_CODEX_CHILD_ENV_KEYS: &[&str] = &[];
const T079_REMOTE_CONTROL_DISABLED_ENV_VAR: &str =
    "CODEX_INTERNAL_APP_SERVER_REMOTE_CONTROL_DISABLED";
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

type ProofResult<T> = std::result::Result<T, String>;
type FrameResult = std::result::Result<Vec<u8>, String>;

fn validate_t079_connected_proof_platform(os: &str, arch: &str) -> ProofResult<()> {
    if os != "linux" {
        return Err(format!(
            "T079 live connected proof supports only Linux/WSL2; observed platform {os}/{arch}"
        ));
    }
    match arch {
        "x86_64" | "aarch64" => Ok(()),
        _ => Err(format!(
            "T079 live connected proof does not support Linux architecture {arch}; supported architectures are x86_64 and aarch64"
        )),
    }
}

const T079_CODEX_CONFIG_COMPAT_VERSION: &str = "codex-cli 0.149.0";

const T079_CODEX_AUTHORITY_REDUCTION_ARGS: &[&str] = &[
    "-c",
    "agents.enabled=false",
    "-c",
    "features.multi_agent=false",
    "-c",
    "features.multi_agent_v2=false",
    "-c",
    "features.auth_elicitation=false",
    "-c",
    "features.apps=false",
    "-c",
    "features.remote_plugin=false",
    "-c",
    "features.tool_suggest=false",
    "-c",
    "features.shell_tool=false",
    "-c",
    "include_apps_instructions=false",
    "-c",
    "include_collaboration_mode_instructions=false",
];

const T079_CODEX_SESSION_ORIGIN_PATHS: &[&str] = &[
    "agents.enabled",
    "features.auth_elicitation",
    "features.apps",
    "features.multi_agent",
    "features.multi_agent_v2.enabled",
    "features.remote_plugin",
    "features.tool_suggest",
    "features.shell_tool",
    "include_apps_instructions",
    "include_collaboration_mode_instructions",
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
    OwnedScopeQuiescent,
    OwnedScopeTerminatedAndQuiescent,
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

struct BoundCodexExecutable {
    #[cfg(target_os = "linux")]
    // Sealed anonymous snapshot retained so /proc/self/fd/<fd> resolves to
    // the exact verified bytes even if the original pathname is later mutated.
    file: File,
    launch_path: PathBuf,
}

impl BoundCodexExecutable {
    fn launch_path(&self) -> &Path {
        &self.launch_path
    }
}

#[cfg(target_os = "linux")]
struct BoundCodexHome {
    directory: File,
    launch_path: PathBuf,
    watcher: OwnedFd,
}

#[cfg(target_os = "linux")]
impl BoundCodexHome {
    fn launch_path(&self) -> &Path {
        &self.launch_path
    }

    fn assert_stable(&self) -> ProofResult<()> {
        let self_event_mask =
            libc::IN_Q_OVERFLOW | libc::IN_MOVE_SELF | libc::IN_DELETE_SELF | libc::IN_IGNORED;
        let mut buffer = [0_u8; 4096];

        loop {
            let read = unsafe {
                libc::read(
                    self.watcher.as_raw_fd(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                )
            };
            if read < 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::EAGAIN) {
                    break;
                }
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(format!(
                    "T079 could not inspect bound CODEX_HOME mutation evidence: {error}"
                ));
            }
            if read == 0 {
                break;
            }

            let read = read as usize;
            let header = std::mem::size_of::<libc::inotify_event>();
            let mut offset = 0usize;
            while offset < read {
                if read - offset < header {
                    return Err("T079 bound CODEX_HOME mutation evidence was truncated".to_owned());
                }
                let event = unsafe {
                    std::ptr::read_unaligned(
                        buffer.as_ptr().add(offset).cast::<libc::inotify_event>(),
                    )
                };
                let name_len = event.len as usize;
                let record_len = header
                    .checked_add(name_len)
                    .ok_or_else(|| "T079 CODEX_HOME watch record overflowed".to_owned())?;
                if record_len > read - offset {
                    return Err("T079 bound CODEX_HOME mutation evidence was malformed".to_owned());
                }
                if event.mask & self_event_mask != 0 {
                    return Err(
                        "T079 bound CODEX_HOME directory identity changed before proof completion"
                            .to_owned(),
                    );
                }
                if name_len > 0 {
                    let name = &buffer[offset + header..offset + record_len];
                    let name = &name[..name
                        .iter()
                        .position(|byte| *byte == 0)
                        .unwrap_or(name.len())];
                    if BLOCKED_CODEX_CONFIG_FILES
                        .iter()
                        .any(|blocked| name == blocked.as_bytes())
                    {
                        return Err(
                            "T079 bound CODEX_HOME configuration surface changed before proof completion"
                                .to_owned(),
                        );
                    }
                }
                offset += record_len;
            }
        }

        for name in BLOCKED_CODEX_CONFIG_FILES {
            reject_config_surface(
                &self.launch_path.join(name),
                "a bound local CODEX_HOME configuration surface",
            )?;
        }
        Ok(())
    }
}

fn parse_outbound(line: &str) -> Value {
    assert!(line.ends_with('\n'));
    serde_json::from_str(line.trim_end()).expect("T079 outbound JSONL must be valid JSON")
}

fn t079_diagnostic_shape(value: Option<&Value>) -> &'static str {
    match value {
        Some(Value::Object(_)) => "OBJECT",
        Some(Value::Array(_)) => "ARRAY",
        Some(Value::String(_)) => "STRING",
        Some(Value::Number(_)) => "NUMBER",
        Some(Value::Bool(_)) => "BOOL",
        Some(Value::Null) => "NULL",
        None => "ABSENT",
    }
}

fn t079_diagnostic_object_key_count(value: Option<&Value>) -> u16 {
    value
        .and_then(Value::as_object)
        .map(|object| u16::try_from(object.len()).unwrap_or(u16::MAX))
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum T079RejectedMethodClass {
    KnownConfigWarning,
    KnownThreadStarted,
    KnownThreadStatusChanged,
    KnownTurnStarted,
    KnownTurnCompleted,
    KnownItemStarted,
    KnownItemCompleted,
    KnownAgentMessageDelta,
    KnownPlanDelta,
    KnownReasoningSummaryTextDelta,
    KnownReasoningSummaryPartAdded,
    KnownReasoningTextDelta,
    KnownTokenUsageUpdated,
    KnownModelRerouted,
    KnownModelVerification,
    KnownModelSafetyBufferingUpdated,
    KnownTurnModerationMetadata,
    KnownErrorNotification,
    KnownWarning,
    KnownGuardianWarning,
    KnownTerminalInteraction,
    UnknownMethod,
}

impl T079RejectedMethodClass {
    fn from_method(value: Option<&Value>) -> Self {
        match value.and_then(Value::as_str) {
            Some("configWarning") => Self::KnownConfigWarning,
            Some("thread/started") => Self::KnownThreadStarted,
            Some("thread/status/changed") => Self::KnownThreadStatusChanged,
            Some("turn/started") => Self::KnownTurnStarted,
            Some("turn/completed") => Self::KnownTurnCompleted,
            Some("item/started") => Self::KnownItemStarted,
            Some("item/completed") => Self::KnownItemCompleted,
            Some("item/agentMessage/delta") => Self::KnownAgentMessageDelta,
            Some("item/plan/delta") => Self::KnownPlanDelta,
            Some("item/reasoning/summaryTextDelta") => Self::KnownReasoningSummaryTextDelta,
            Some("item/reasoning/summaryPartAdded") => Self::KnownReasoningSummaryPartAdded,
            Some("item/reasoning/textDelta") => Self::KnownReasoningTextDelta,
            Some("thread/tokenUsage/updated") => Self::KnownTokenUsageUpdated,
            Some("model/rerouted") => Self::KnownModelRerouted,
            Some("model/verification") => Self::KnownModelVerification,
            Some("model/safetyBuffering/updated") => Self::KnownModelSafetyBufferingUpdated,
            Some("turn/moderationMetadata") => Self::KnownTurnModerationMetadata,
            Some("error") => Self::KnownErrorNotification,
            Some("warning") => Self::KnownWarning,
            Some("guardianWarning") => Self::KnownGuardianWarning,
            Some("item/commandExecution/terminalInteraction") => Self::KnownTerminalInteraction,
            _ => Self::UnknownMethod,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::KnownConfigWarning => "KNOWN_CONFIG_WARNING",
            Self::KnownThreadStarted => "KNOWN_THREAD_STARTED",
            Self::KnownThreadStatusChanged => "KNOWN_THREAD_STATUS_CHANGED",
            Self::KnownTurnStarted => "KNOWN_TURN_STARTED",
            Self::KnownTurnCompleted => "KNOWN_TURN_COMPLETED",
            Self::KnownItemStarted => "KNOWN_ITEM_STARTED",
            Self::KnownItemCompleted => "KNOWN_ITEM_COMPLETED",
            Self::KnownAgentMessageDelta => "KNOWN_AGENT_MESSAGE_DELTA",
            Self::KnownPlanDelta => "KNOWN_PLAN_DELTA",
            Self::KnownReasoningSummaryTextDelta => "KNOWN_REASONING_SUMMARY_TEXT_DELTA",
            Self::KnownReasoningSummaryPartAdded => "KNOWN_REASONING_SUMMARY_PART_ADDED",
            Self::KnownReasoningTextDelta => "KNOWN_REASONING_TEXT_DELTA",
            Self::KnownTokenUsageUpdated => "KNOWN_TOKEN_USAGE_UPDATED",
            Self::KnownModelRerouted => "KNOWN_MODEL_REROUTED",
            Self::KnownModelVerification => "KNOWN_MODEL_VERIFICATION",
            Self::KnownModelSafetyBufferingUpdated => "KNOWN_MODEL_SAFETY_BUFFERING_UPDATED",
            Self::KnownTurnModerationMetadata => "KNOWN_TURN_MODERATION_METADATA",
            Self::KnownErrorNotification => "KNOWN_ERROR_NOTIFICATION",
            Self::KnownWarning => "KNOWN_WARNING",
            Self::KnownGuardianWarning => "KNOWN_GUARDIAN_WARNING",
            Self::KnownTerminalInteraction => "KNOWN_TERMINAL_INTERACTION",
            Self::UnknownMethod => "UNKNOWN_METHOD",
        }
    }

    fn known_param_keys(self) -> &'static [&'static str] {
        match self {
            Self::KnownConfigWarning => &["details", "summary"],
            Self::KnownThreadStarted => &["thread"],
            Self::KnownThreadStatusChanged => &["status", "threadId"],
            Self::KnownTurnStarted | Self::KnownTurnCompleted => &["threadId", "turn"],
            Self::KnownItemStarted => &["item", "startedAtMs", "threadId", "turnId"],
            Self::KnownItemCompleted => &["completedAtMs", "item", "threadId", "turnId"],
            Self::KnownAgentMessageDelta | Self::KnownPlanDelta => {
                &["delta", "itemId", "threadId", "turnId"]
            }
            Self::KnownReasoningSummaryTextDelta => {
                &["delta", "itemId", "summaryIndex", "threadId", "turnId"]
            }
            Self::KnownReasoningSummaryPartAdded => {
                &["itemId", "summaryIndex", "threadId", "turnId"]
            }
            Self::KnownReasoningTextDelta => {
                &["contentIndex", "delta", "itemId", "threadId", "turnId"]
            }
            Self::KnownTokenUsageUpdated => &["threadId", "tokenUsage", "turnId"],
            Self::KnownModelRerouted => &["threadId", "turnId", "fromModel", "toModel", "reason"],
            Self::KnownModelVerification => &["threadId", "turnId", "verifications"],
            Self::KnownModelSafetyBufferingUpdated => &[
                "threadId",
                "turnId",
                "model",
                "useCases",
                "reasons",
                "showBufferingUi",
                "fasterModel",
            ],
            Self::KnownTurnModerationMetadata => &["threadId", "turnId", "metadata"],
            Self::KnownErrorNotification => &["error", "willRetry", "threadId", "turnId"],
            Self::KnownWarning | Self::KnownGuardianWarning => &["threadId", "message"],
            Self::KnownTerminalInteraction => {
                &["itemId", "processId", "stdin", "threadId", "turnId"]
            }
            Self::UnknownMethod => &[],
        }
    }
}

fn t079_diagnostic_known_unknown_key_counts(
    params: Option<&Value>,
    method_class: T079RejectedMethodClass,
) -> (u16, u16) {
    let Some(params) = params.and_then(Value::as_object) else {
        return (0, 0);
    };
    let known_keys = method_class.known_param_keys();
    let known = params
        .keys()
        .filter(|key| known_keys.contains(&key.as_str()))
        .count();
    let unknown = params.len().saturating_sub(known);
    (
        u16::try_from(known).unwrap_or(u16::MAX),
        u16::try_from(unknown).unwrap_or(u16::MAX),
    )
}

std::thread_local! {
    static T079_REJECTION_METADATA_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn t079_rejection_metadata(frame: &[u8], phase: &str) -> String {
    T079_REJECTION_METADATA_CALLS.with(|calls| calls.set(calls.get().saturating_add(1)));
    let Ok(value) = serde_json::from_slice::<Value>(frame) else {
        return format!("phase={phase};frame=UNPARSEABLE_JSON");
    };
    let params = value.get("params");
    let method_class = T079RejectedMethodClass::from_method(value.get("method"));
    let (known_key_count, unknown_key_count) =
        t079_diagnostic_known_unknown_key_counts(params, method_class);

    let mut fields = vec![
        format!("phase={phase}"),
        format!(
            "method_shape={}",
            t079_diagnostic_shape(value.get("method"))
        ),
        format!("METHOD_CLASS={}", method_class.label()),
        format!("params_shape={}", t079_diagnostic_shape(params)),
        format!(
            "param_key_count={}",
            t079_diagnostic_object_key_count(params)
        ),
        format!("KNOWN_KEY_COUNT={known_key_count}"),
        format!("UNKNOWN_KEY_COUNT={unknown_key_count}"),
    ];

    for key in ["thread", "turn", "item", "status", "tokenUsage"] {
        let nested = params.and_then(|params| params.get(key));
        if nested.is_some_and(Value::is_object) {
            fields.push(format!(
                "{key}_key_count={}",
                t079_diagnostic_object_key_count(nested)
            ));
        }
    }

    fields.join(";")
}

fn ingest_t079_frame_with_rejection_metadata(
    client: &mut CodexProtocolClient,
    frame: &[u8],
    phase: &str,
) -> ProofResult<CodexInbound> {
    match client.ingest_jsonl_frame(frame) {
        Ok(inbound) => Ok(inbound),
        Err(CodexProtocolError::UnexpectedT079Notification) => {
            let metadata = t079_rejection_metadata(frame, phase);
            Err(format!(
                "T079 Codex protocol failure: rejected notification metadata: {metadata}"
            ))
        }
        Err(error) => Err(format!("T079 Codex protocol failure: {error}")),
    }
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

fn expected_t079_codex_0_149_config() -> Value {
    serde_json::from_str(
        r###"{
        "agents": {
            "default_subagent_model": null,
            "default_subagent_reasoning_effort": null,
            "enabled": false,
            "interrupt_message": null,
            "job_max_runtime_seconds": null,
            "max_concurrent_threads_per_session": null,
            "max_depth": null
        },
        "allow_login_shell": true,
        "analytics": null,
        "approval_policy": null,
        "approvals_reviewer": null,
        "apps": null,
        "apps_mcp_product_sku": null,
        "audio": null,
        "auto_review": null,
        "background_terminal_max_timeout": 300000,
        "chatgpt_base_url": "https://chatgpt.com/backend-api/",
        "check_for_update_on_startup": null,
        "cli_auth_credentials_store": "file",
        "compact_prompt": null,
        "default_permissions": null,
        "desktop": null,
        "developer_instructions": null,
        "disable_paste_burst": null,
        "experimental_compact_prompt_file": null,
        "experimental_realtime_start_instructions": null,
        "experimental_realtime_webrtc_call_base_url": null,
        "experimental_realtime_ws_backend_prompt": null,
        "experimental_realtime_ws_base_url": null,
        "experimental_realtime_ws_model": null,
        "experimental_realtime_ws_startup_context": null,
        "experimental_thread_store": null,
        "experimental_thread_store_endpoint": null,
        "experimental_use_unified_exec_tool": null,
        "features": {
            "apps": false,
            "auth_elicitation": false,
            "mcp_2026_07_28": false,
            "memories": false,
            "mentions_v2": true,
            "multi_agent": false,
            "multi_agent_v2": false,
            "network_proxy": null,
            "remote_control": false,
            "remote_plugin": false,
            "shell_tool": false,
            "tool_suggest": false
        },
        "feedback": null,
        "file_opener": "vscode",
        "forced_chatgpt_workspace_id": null,
        "forced_login_method": null,
        "ghost_snapshot": null,
        "goals": null,
        "hide_agent_reasoning": false,
        "history": {
            "max_bytes": null,
            "persistence": "save-all"
        },
        "hooks": null,
        "include_apps_instructions": false,
        "include_collaboration_mode_instructions": false,
        "include_environment_context": true,
        "include_permissions_instructions": true,
        "instructions": null,
        "js_repl_node_module_dirs": null,
        "js_repl_node_path": null,
        "log_dir": null,
        "marketplaces": {},
        "mcp_oauth_callback_port": null,
        "mcp_oauth_callback_url": null,
        "mcp_oauth_credentials_store": "auto",
        "mcp_servers": {},
        "memories": null,
        "model": null,
        "model_auto_compact_token_limit": null,
        "model_auto_compact_token_limit_scope": null,
        "model_catalog_json": null,
        "model_context_window": null,
        "model_instructions_file": null,
        "model_provider": null,
        "model_providers": {},
        "model_reasoning_effort": null,
        "model_reasoning_summary": null,
        "model_verbosity": null,
        "notice": null,
        "notify": null,
        "openai_base_url": null,
        "orchestrator": null,
        "oss_provider": null,
        "otel": null,
        "permissions": null,
        "personality": null,
        "plan_mode_reasoning_effort": null,
        "plugins": {},
        "profile": null,
        "profiles": {},
        "project_doc_fallback_filenames": [],
        "project_doc_max_bytes": 32768,
        "project_root_markers": [".git"],
        "projects": null,
        "realtime": null,
        "responses_api_metadata": null,
        "review_model": null,
        "sandbox_mode": null,
        "sandbox_workspace_write": null,
        "service_tier": null,
        "shell_environment_policy": {
            "exclude": null,
            "experimental_use_profile": null,
            "filters": null,
            "ignore_default_excludes": null,
            "include_only": null,
            "inherit": null,
            "set": null
        },
        "show_raw_agent_reasoning": null,
        "skills": null,
        "sqlite_home": null,
        "suppress_unstable_features_warning": null,
        "tool_output_token_limit": null,
        "tool_suggest": null,
        "tools": null,
        "tui": null,
        "web_search": null,
        "windows": null
    }"###,
    )
    .expect("static T079 Codex 0.149 config snapshot must be valid JSON")
}

fn valid_config_layer_version(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };

    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_t079_session_origins(origins: &serde_json::Map<String, Value>) -> ProofResult<()> {
    let mut actual_keys = origins.keys().map(String::as_str).collect::<Vec<_>>();
    actual_keys.sort_unstable();

    let mut expected_keys = T079_CODEX_SESSION_ORIGIN_PATHS.to_vec();
    expected_keys.sort_unstable();

    if actual_keys != expected_keys {
        return Err("T079 Codex 0.149 session-origin surface changed".to_owned());
    }

    let mut shared_version: Option<&str> = None;

    for key in T079_CODEX_SESSION_ORIGIN_PATHS {
        let metadata = origins
            .get(*key)
            .and_then(Value::as_object)
            .ok_or_else(|| format!("T079 session origin metadata is malformed: {key}"))?;

        let mut metadata_keys = metadata.keys().map(String::as_str).collect::<Vec<_>>();
        metadata_keys.sort_unstable();

        if metadata_keys != vec!["name", "version"] {
            return Err(format!("T079 session origin metadata shape changed: {key}"));
        }

        let name = metadata
            .get("name")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("T079 session origin source is malformed: {key}"))?;

        if name.len() != 1 || name.get("type").and_then(Value::as_str) != Some("sessionFlags") {
            return Err(format!(
                "T079 refuses non-SessionFlags authority-reduction origin: {key}"
            ));
        }

        let version = metadata
            .get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("T079 session origin version is malformed: {key}"))?;

        if !valid_config_layer_version(version) {
            return Err(format!(
                "T079 session origin version is not canonical SHA-256 evidence: {key}"
            ));
        }

        match shared_version {
            Some(expected) if version != expected => {
                return Err(
                    "T079 authority-reduction origins do not share one SessionFlags layer version"
                        .to_owned(),
                );
            }
            None => shared_version = Some(version),
            _ => {}
        }
    }

    Ok(())
}

fn validate_t079_exact_config_surface(config: &serde_json::Map<String, Value>) -> ProofResult<()> {
    let expected = expected_t079_codex_0_149_config();
    let expected = expected
        .as_object()
        .expect("T079 expected config fixture must be an object");

    let mut actual_keys = config.keys().map(String::as_str).collect::<Vec<_>>();
    actual_keys.sort_unstable();

    let mut expected_keys = expected.keys().map(String::as_str).collect::<Vec<_>>();
    expected_keys.sort_unstable();

    if actual_keys != expected_keys {
        return Err("T079 Codex 0.149 effective config key set changed".to_owned());
    }

    for (key, expected_value) in expected {
        if config.get(key) != Some(expected_value) {
            return Err(format!(
                "T079 Codex 0.149 effective config evidence changed at key: {key}"
            ));
        }
    }

    Ok(())
}

fn validate_effective_config(result: &Value, observed_version: &str) -> ProofResult<()> {
    if observed_version != T079_CODEX_CONFIG_COMPAT_VERSION {
        return Err(format!(
            "T079 config/read compatibility is qualified only for {}; observed {observed_version}",
            T079_CODEX_CONFIG_COMPAT_VERSION
        ));
    }

    let config = result
        .get("config")
        .and_then(Value::as_object)
        .ok_or_else(|| "T079 config/read response is missing effective config".to_owned())?;

    let origins = result
        .get("origins")
        .and_then(Value::as_object)
        .ok_or_else(|| "T079 config/read response is missing config origins".to_owned())?;

    validate_t079_session_origins(origins)?;
    validate_t079_exact_config_surface(config)?;

    Ok(())
}

fn validate_thread_start_result(result: &Value, expected_cwd: &str) -> ProofResult<NativeThreadId> {
    let thread = result
        .get("thread")
        .ok_or_else(|| "T079 thread/start response is missing thread".to_owned())?;
    if thread.get("cwd").and_then(Value::as_str) != Some(expected_cwd) {
        return Err("T079 Codex thread cwd does not match disposable proof root".to_owned());
    }
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

fn completed_final_answer_text(params: &Value) -> ProofResult<Option<&str>> {
    let Some(item) = params.get("item") else {
        return Ok(None);
    };
    if item.get("type").and_then(Value::as_str) != Some("agentMessage")
        || item.get("phase").and_then(Value::as_str) != Some("final_answer")
    {
        return Ok(None);
    }
    item.get("text")
        .and_then(Value::as_str)
        .map(Some)
        .ok_or_else(|| "T079 completed final-answer agent message is missing text".to_owned())
}

fn validate_exact_text(value: &str, label: &str) -> ProofResult<()> {
    if value.trim().is_empty()
        || value != value.trim()
        || value.len() > super::MAX_PROTOCOL_TEXT_BYTES
        || value.chars().any(char::is_control)
    {
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

fn prepare_bound_codex_version_observation(
    executable: &Path,
) -> ProofResult<(RuntimeExecutableIdentity, BoundCodexExecutable)> {
    validate_live_codex_candidate_path(executable)?;
    let pre_version = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .map_err(|error| error.to_string())?;
    let identity = pre_version.executable.ok_or_else(|| {
        "T079 Codex executable became unavailable before version binding".to_owned()
    })?;
    let bound = bind_verified_native_codex_executable(&identity)?;
    Ok((identity, bound))
}

fn discover_codex_from_bound_version(
    executable: &Path,
    expected: &RuntimeExecutableIdentity,
    version: String,
) -> ProofResult<RuntimeDiscovery> {
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        executable,
        SafeVersionObservation::Observed(version),
        &[],
        &[],
    )
    .map_err(|error| error.to_string())?;
    if discovery.executable.as_ref() != Some(expected) {
        return Err(
            "T079 Codex executable identity changed between pre-version binding and discovery"
                .to_owned(),
        );
    }
    Ok(discovery)
}

fn reject_config_surface(path: &Path, label: &str) -> ProofResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!(
            "T079 refuses {label} before Codex launch: {}",
            path.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "T079 could not prove {label} absent at {}: {error}",
            path.display()
        )),
    }
}

fn validate_preexisting_isolated_codex_home(path: &Path) -> ProofResult<PathBuf> {
    let canonical = canonical_directory_outside_primary_checkout(path, "WINDS_T079_CODEX_HOME")?;
    for name in BLOCKED_CODEX_CONFIG_FILES {
        reject_config_surface(
            &canonical.join(name),
            "a local CODEX_HOME configuration surface",
        )?;
    }
    Ok(canonical)
}

#[cfg(target_os = "linux")]
fn bind_preexisting_isolated_codex_home(path: &Path) -> ProofResult<BoundCodexHome> {
    let canonical = canonical_directory_outside_primary_checkout(path, "WINDS_T079_CODEX_HOME")?;
    let directory = File::open(&canonical)
        .map_err(|error| format!("T079 could not open isolated CODEX_HOME directory: {error}"))?;
    let fd = directory.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(format!(
            "T079 could not inspect CODEX_HOME descriptor flags: {}",
            std::io::Error::last_os_error()
        ));
    }
    if flags & libc::FD_CLOEXEC == 0 {
        return Err(
            "T079 requires the bound CODEX_HOME descriptor to remain close-on-exec in the parent process"
                .to_owned(),
        );
    }

    let launch_path = PathBuf::from(format!("/proc/self/fd/{fd}"));
    let bound_target = launch_path
        .canonicalize()
        .map_err(|error| format!("T079 could not resolve bound CODEX_HOME descriptor: {error}"))?;
    if bound_target != canonical {
        return Err("T079 CODEX_HOME identity changed during handle binding".to_owned());
    }
    let checkout = canonical_primary_checkout_root()?;
    ensure_path_outside_primary_checkout(&bound_target, &checkout, "bound CODEX_HOME")?;

    let raw_watcher = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
    if raw_watcher < 0 {
        return Err(format!(
            "T079 could not create CODEX_HOME mutation watch: {}",
            std::io::Error::last_os_error()
        ));
    }
    let watcher = unsafe { OwnedFd::from_raw_fd(raw_watcher) };
    let watch_path = CString::new(launch_path.as_os_str().as_bytes())
        .map_err(|_| "T079 bound CODEX_HOME path contains NUL".to_owned())?;
    let watch_mask = libc::IN_CREATE
        | libc::IN_MOVED_TO
        | libc::IN_DELETE
        | libc::IN_MOVED_FROM
        | libc::IN_CLOSE_WRITE
        | libc::IN_ATTRIB
        | libc::IN_MOVE_SELF
        | libc::IN_DELETE_SELF;
    if unsafe { libc::inotify_add_watch(watcher.as_raw_fd(), watch_path.as_ptr(), watch_mask) } < 0
    {
        return Err(format!(
            "T079 could not bind CODEX_HOME mutation watch: {}",
            std::io::Error::last_os_error()
        ));
    }

    if launch_path
        .canonicalize()
        .map_err(|error| format!("T079 could not revalidate bound CODEX_HOME: {error}"))?
        != canonical
    {
        return Err("T079 CODEX_HOME identity changed while binding mutation watch".to_owned());
    }

    let bound = BoundCodexHome {
        directory,
        launch_path,
        watcher,
    };
    bound.assert_stable()?;
    Ok(bound)
}

#[cfg(target_os = "linux")]
fn configure_bound_codex_home_inheritance(
    command: &mut Command,
    codex_home: &BoundCodexHome,
) -> ProofResult<()> {
    use std::os::unix::process::CommandExt;

    let fd = codex_home.directory.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 || flags & libc::FD_CLOEXEC == 0 {
        return Err(
            "T079 could not prove parent CODEX_HOME descriptor is close-on-exec".to_owned(),
        );
    }

    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn validate_no_system_codex_config() -> ProofResult<()> {
    Err(
        "T079 refuses macOS live proof because com.openai.codex managed preferences (config_toml_base64 / requirements_toml_base64) are a pre-launch configuration surface that this harness intentionally does not read"
            .to_owned(),
    )
}

#[cfg(all(unix, not(target_os = "macos")))]
fn validate_no_system_codex_config() -> ProofResult<()> {
    let base = Path::new("/etc/codex");
    for name in BLOCKED_CODEX_CONFIG_FILES {
        reject_config_surface(&base.join(name), "a system Codex configuration surface")?;
    }
    Ok(())
}

#[cfg(windows)]
fn validate_no_system_codex_config() -> ProofResult<()> {
    let program_data = env::var_os("ProgramData")
        .ok_or_else(|| "T079 cannot prove Windows system Codex config location".to_owned())?;
    let program_data = PathBuf::from(program_data);
    if !program_data.is_absolute() {
        return Err("T079 Windows ProgramData path is not absolute".to_owned());
    }
    let base = program_data.join("OpenAI").join("Codex");
    for name in BLOCKED_CODEX_CONFIG_FILES {
        reject_config_surface(&base.join(name), "a system Codex configuration surface")?;
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn validate_no_system_codex_config() -> ProofResult<()> {
    Err("T079 live proof does not support this platform".to_owned())
}

fn configure_isolated_codex_environment(command: &mut Command, codex_home: Option<&Path>) {
    command.env_clear();
    command.env(T079_REMOTE_CONTROL_DISABLED_ENV_VAR, "1");
    if let Some(codex_home) = codex_home {
        command.env("CODEX_HOME", codex_home);
    }
    for key in SAFE_CODEX_CHILD_ENV_KEYS {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
}

fn configure_t079_codex_authority_reduction(command: &mut Command) {
    command.args(T079_CODEX_AUTHORITY_REDUCTION_ARGS);
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
fn install_t079_no_process_descendants_filter() -> std::io::Result<()> {
    const BPF_LD_W_ABS: u16 = 0x20;
    const BPF_ALU_AND_K: u16 = 0x54;
    const BPF_JMP_JEQ_K: u16 = 0x15;
    const BPF_RET_K: u16 = 0x06;

    const SECCOMP_RET_KILL_THREAD: u32 = 0x0000_0000;
    const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
    const SECCOMP_MODE_FILTER: libc::c_ulong = 2;
    const PR_SET_SECCOMP: libc::c_int = 22;
    const PR_SET_NO_NEW_PRIVS: libc::c_int = 38;

    #[cfg(target_arch = "x86_64")]
    const AUDIT_ARCH: u32 = 0xc000_003e;
    #[cfg(target_arch = "aarch64")]
    const AUDIT_ARCH: u32 = 0xc000_00b7;

    #[cfg(target_arch = "x86_64")]
    const SYS_CLONE: u32 = 56;
    #[cfg(target_arch = "x86_64")]
    const SYS_FORK: u32 = 57;
    #[cfg(target_arch = "x86_64")]
    const SYS_VFORK: u32 = 58;
    #[cfg(target_arch = "aarch64")]
    const SYS_CLONE: u32 = 220;
    #[cfg(target_arch = "aarch64")]
    const SYS_FORK: u32 = u32::MAX - 1;
    #[cfg(target_arch = "aarch64")]
    const SYS_VFORK: u32 = u32::MAX - 2;
    const SYS_CLONE3: u32 = 435;

    const SECCOMP_DATA_NR_OFFSET: u32 = 0;
    const SECCOMP_DATA_ARCH_OFFSET: u32 = 4;
    const SECCOMP_DATA_ARGS0_OFFSET: u32 = 16;
    const X32_SYSCALL_BIT_CLEAR_MASK: u32 = 0xbfff_ffff;
    const CLONE_THREAD_FLAG: u32 = 0x0001_0000;

    const fn statement(code: u16, k: u32) -> libc::sock_filter {
        libc::sock_filter {
            code,
            jt: 0,
            jf: 0,
            k,
        }
    }

    const fn jump(code: u16, k: u32, jt: u8, jf: u8) -> libc::sock_filter {
        libc::sock_filter { code, jt, jf, k }
    }

    let deny_process = SECCOMP_RET_ERRNO | (libc::EPERM as u32 & 0x0000_ffff);
    let clone3_fallback = SECCOMP_RET_ERRNO | (libc::ENOSYS as u32 & 0x0000_ffff);
    let mut filter = [
        statement(BPF_LD_W_ABS, SECCOMP_DATA_ARCH_OFFSET),
        jump(BPF_JMP_JEQ_K, AUDIT_ARCH, 1, 0),
        statement(BPF_RET_K, SECCOMP_RET_KILL_THREAD),
        statement(BPF_LD_W_ABS, SECCOMP_DATA_NR_OFFSET),
        statement(BPF_ALU_AND_K, X32_SYSCALL_BIT_CLEAR_MASK),
        jump(BPF_JMP_JEQ_K, SYS_FORK, 0, 1),
        statement(BPF_RET_K, deny_process),
        jump(BPF_JMP_JEQ_K, SYS_VFORK, 0, 1),
        statement(BPF_RET_K, deny_process),
        jump(BPF_JMP_JEQ_K, SYS_CLONE3, 0, 1),
        statement(BPF_RET_K, clone3_fallback),
        jump(BPF_JMP_JEQ_K, SYS_CLONE, 0, 4),
        statement(BPF_LD_W_ABS, SECCOMP_DATA_ARGS0_OFFSET),
        statement(BPF_ALU_AND_K, CLONE_THREAD_FLAG),
        jump(BPF_JMP_JEQ_K, CLONE_THREAD_FLAG, 1, 0),
        statement(BPF_RET_K, deny_process),
        statement(BPF_RET_K, SECCOMP_RET_ALLOW),
    ];
    let mut program = libc::sock_fprog {
        len: filter.len() as u16,
        filter: filter.as_mut_ptr(),
    };

    if unsafe { libc::prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe {
        libc::prctl(
            PR_SET_SECCOMP,
            SECCOMP_MODE_FILTER,
            &mut program as *mut libc::sock_fprog,
        )
    } == -1
    {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(all(
    target_os = "linux",
    not(any(target_arch = "x86_64", target_arch = "aarch64"))
))]
fn install_t079_no_process_descendants_filter() -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "T079 process-descendant denial is not implemented for this Linux architecture",
    ))
}

#[cfg(target_os = "linux")]
fn configure_t079_process_descendant_denial(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    // This hook is registered before process_scope::spawn_owned_process adds its
    // own hook. It blocks process creation but deliberately permits setsid/prctl,
    // so the later owned-scope hook can still establish the session boundary and
    // its independent anti-escape filter. clone3 returns ENOSYS so libc thread
    // creation can fall back to clone; clone is accepted only with CLONE_THREAD.
    unsafe {
        command.pre_exec(install_t079_no_process_descendants_filter);
    }
}

#[cfg(target_os = "linux")]
fn require_linux_native_elf(file: &mut File) -> ProofResult<()> {
    let mut magic = [0_u8; 4];
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("T079 could not seek Codex executable: {error}"))?;
    file.read_exact(&mut magic)
        .map_err(|error| format!("T079 could not read Codex executable header: {error}"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("T079 could not rewind Codex executable: {error}"))?;
    if magic != [0x7f, b'E', b'L', b'F'] {
        return Err(
            "T079 requires WINDS_T079_CODEX_PATH to resolve to a Linux-native ELF Codex binary; interpreter wrappers are refused because PATH is intentionally absent"
                .to_owned(),
        );
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_live_codex_candidate_path(path: &Path) -> ProofResult<()> {
    let mut file = File::open(path)
        .map_err(|error| format!("T079 could not open candidate Codex executable: {error}"))?;
    require_linux_native_elf(&mut file)
}

#[cfg(not(target_os = "linux"))]
fn validate_live_codex_candidate_path(path: &Path) -> ProofResult<()> {
    let _ = path;
    Err(
        "T079 first connected proof currently requires Linux/WSL2 so executable launch can be bound to an already-open verified file descriptor"
            .to_owned(),
    )
}

#[cfg(target_os = "linux")]
fn bind_verified_native_codex_executable(
    expected: &RuntimeExecutableIdentity,
) -> ProofResult<BoundCodexExecutable> {
    let mut source = File::open(&expected.canonical_path)
        .map_err(|error| format!("T079 could not open verified Codex executable: {error}"))?;
    let metadata = source
        .metadata()
        .map_err(|error| format!("T079 could not inspect verified Codex executable: {error}"))?;
    if !metadata.is_file() || metadata.len() != expected.byte_len {
        return Err("T079 Codex executable metadata changed before handle binding".to_owned());
    }
    require_linux_native_elf(&mut source)?;

    let snapshot_name =
        CString::new("winds-t079-codex-executable").expect("static memfd name contains no NUL");
    let snapshot_fd = unsafe {
        libc::memfd_create(
            snapshot_name.as_ptr(),
            libc::MFD_CLOEXEC | libc::MFD_ALLOW_SEALING,
        )
    };
    if snapshot_fd < 0 {
        return Err(format!(
            "T079 could not create anonymous executable snapshot: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut snapshot = unsafe { File::from_raw_fd(snapshot_fd) };

    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut total_read = 0_u64;
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| format!("T079 could not hash bound Codex executable: {error}"))?;
        if read == 0 {
            break;
        }
        total_read = total_read
            .checked_add(read as u64)
            .ok_or_else(|| "T079 bound Codex executable byte count overflowed".to_owned())?;
        if total_read > expected.byte_len {
            return Err("T079 bound Codex executable exceeded expected byte length".to_owned());
        }
        digest.update(&buffer[..read]);
        snapshot
            .write_all(&buffer[..read])
            .map_err(|error| format!("T079 could not copy executable snapshot: {error}"))?;
    }
    if total_read != expected.byte_len {
        return Err("T079 bound Codex executable byte length changed".to_owned());
    }
    let sha256: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if sha256 != expected.sha256 {
        return Err("T079 bound Codex executable digest does not match discovery".to_owned());
    }

    if unsafe { libc::fchmod(snapshot_fd, 0o500) } < 0 {
        return Err(format!(
            "T079 could not mark executable snapshot executable: {}",
            std::io::Error::last_os_error()
        ));
    }

    let required_seals =
        libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(snapshot_fd, libc::F_ADD_SEALS, required_seals) } < 0 {
        return Err(format!(
            "T079 could not seal executable snapshot: {}",
            std::io::Error::last_os_error()
        ));
    }
    let observed_seals = unsafe { libc::fcntl(snapshot_fd, libc::F_GET_SEALS) };
    if observed_seals < 0 || observed_seals & required_seals != required_seals {
        return Err("T079 executable snapshot seal evidence is incomplete".to_owned());
    }

    // Preserve path provenance through the end of snapshot construction. After
    // this check, launch no longer depends on pathname contents: the sealed
    // memfd holds the already-hashed bytes.
    if revalidate_runtime_identity(expected).map_err(|error| error.to_string())?
        != RuntimeIdentityRevalidation::Match
    {
        return Err("T079 Codex executable path changed while binding launch identity".to_owned());
    }

    let flags = unsafe { libc::fcntl(snapshot_fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(format!(
            "T079 could not inspect Codex executable snapshot descriptor flags: {}",
            std::io::Error::last_os_error()
        ));
    }
    if flags & libc::FD_CLOEXEC == 0 {
        return Err(
            "T079 requires the sealed executable snapshot descriptor to remain close-on-exec in the parent process"
                .to_owned(),
        );
    }

    let launch_path = PathBuf::from(format!("/proc/self/fd/{snapshot_fd}"));
    let launch_metadata = fs::metadata(&launch_path).map_err(|error| {
        format!("T079 could not prove sealed executable snapshot path: {error}")
    })?;
    if !launch_metadata.is_file() || launch_metadata.len() != expected.byte_len {
        return Err(
            "T079 sealed executable snapshot does not preserve executable identity".to_owned(),
        );
    }

    Ok(BoundCodexExecutable {
        file: snapshot,
        launch_path,
    })
}

#[cfg(not(target_os = "linux"))]
fn bind_verified_native_codex_executable(
    expected: &RuntimeExecutableIdentity,
) -> ProofResult<BoundCodexExecutable> {
    let _ = expected;
    Err(
        "T079 first connected proof currently requires Linux/WSL2 handle-bound executable launch"
            .to_owned(),
    )
}

#[cfg(target_os = "linux")]
fn set_nonblocking_stdout(stdout: &std::process::ChildStdout) -> ProofResult<()> {
    let fd = stdout.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(format!(
            "T079 could not inspect Codex stdout descriptor flags: {}",
            std::io::Error::last_os_error()
        ));
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(format!(
            "T079 could not make Codex stdout nonblocking: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn finish_t079_process(
    mut child: OwnedProcess,
    cleanup_deadline: Instant,
    label: &str,
) -> ProofResult<CleanupEvidence> {
    let graceful_deadline = std::cmp::min(Instant::now() + GRACEFUL_CHILD_EXIT, cleanup_deadline);
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("T079 could not inspect {label}: {error}"))?
        {
            Some(_) => {
                return Ok(CleanupEvidence::OwnedScopeQuiescent);
            }
            None if Instant::now() < graceful_deadline => thread::sleep(Duration::from_millis(10)),
            None => break,
        }
    }

    child
        .terminate_direct_t079(cleanup_deadline, label)
        .map_err(|error| error.to_string())?;
    Ok(CleanupEvidence::OwnedScopeTerminatedAndQuiescent)
}

#[cfg(target_os = "linux")]
fn cleanup_version_failure(child: OwnedProcess, message: String) -> String {
    let cleanup_deadline = Instant::now() + CLEANUP_TIMEOUT;
    match finish_t079_process(child, cleanup_deadline, "T079 Codex --version") {
        Ok(_) => message,
        Err(cleanup) => format!("{message}; version cleanup evidence: {cleanup}"),
    }
}

#[cfg(target_os = "linux")]
fn observe_version_bounded(executable: &Path) -> ProofResult<String> {
    let root = disposable_root()?;
    let _root_guard = EmptyDisposableRootGuard { root: root.clone() };
    let result = (|| -> ProofResult<String> {
        let mut command = Command::new(executable);
        configure_isolated_codex_environment(&mut command, None);
        command
            .arg("--version")
            .current_dir(&root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        configure_t079_process_descendant_denial(&mut command);
        let mut child = spawn_owned_process(&mut command, "T079 Codex --version")
            .map_err(|error| format!("T079 could not execute Codex --version: {error}"))?;
        let mut stdout = match child.take_stdout() {
            Some(stdout) => stdout,
            None => {
                return Err(cleanup_version_failure(
                    child,
                    "T079 Codex --version stdout was unavailable".to_owned(),
                ));
            }
        };
        if let Err(error) = set_nonblocking_stdout(&stdout) {
            return Err(cleanup_version_failure(child, error));
        }

        let deadline = Instant::now() + VERSION_TIMEOUT;
        let mut bytes = Vec::new();
        let mut stdout_eof = false;
        let mut exit_status = None;
        let mut chunk = [0_u8; 512];

        loop {
            if !stdout_eof {
                loop {
                    match stdout.read(&mut chunk) {
                        Ok(0) => {
                            stdout_eof = true;
                            break;
                        }
                        Ok(read) => {
                            let new_len = match bytes.len().checked_add(read) {
                                Some(new_len) => new_len,
                                None => {
                                    return Err(cleanup_version_failure(
                                        child,
                                        "T079 Codex --version byte count overflowed".to_owned(),
                                    ));
                                }
                            };
                            if new_len > MAX_VERSION_BYTES {
                                return Err(cleanup_version_failure(
                                    child,
                                    "T079 Codex --version output exceeded bounded size".to_owned(),
                                ));
                            }
                            bytes.extend_from_slice(&chunk[..read]);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(error) => {
                            return Err(cleanup_version_failure(
                                child,
                                format!("T079 could not read Codex --version: {error}"),
                            ));
                        }
                    }
                }
            }

            if exit_status.is_none() {
                match child.try_wait() {
                    Ok(Some(status)) => exit_status = Some(status),
                    Ok(None) => {}
                    Err(error) => {
                        return Err(cleanup_version_failure(
                            child,
                            format!("T079 could not inspect Codex --version: {error}"),
                        ));
                    }
                }
            }

            if stdout_eof && exit_status.is_some() {
                break;
            }
            if Instant::now() >= deadline {
                return Err(cleanup_version_failure(
                    child,
                    "T079 Codex --version exceeded bounded timeout".to_owned(),
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }

        let status = exit_status.expect("T079 version loop exits only with status");
        if !status.success() {
            return Err(format!("T079 Codex --version failed with status {status}"));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| "T079 Codex --version output is not UTF-8".to_owned())?;
        let version = text.trim_end_matches(['\r', '\n']);
        validate_exact_text(version, "Codex version")?;
        if version.contains('\r') || version.contains('\n') {
            return Err("T079 Codex --version must be exactly one line".to_owned());
        }
        Ok(version.to_owned())
    })();

    let root_check = ensure_disposable_root_unchanged(&root);
    match (result, root_check) {
        (Ok(version), Ok(())) => Ok(version),
        (result, root_check) => {
            let mut failures = Vec::new();
            if let Err(error) = result {
                failures.push(format!("version={error}"));
            }
            if let Err(error) = root_check {
                failures.push(format!("root_cleanup={error}"));
            }
            Err(format!(
                "T079 Codex --version proof/cleanup failure: {}",
                failures.join("; ")
            ))
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn observe_version_bounded(executable: &Path) -> ProofResult<String> {
    let _ = executable;
    Err("T079 bounded Codex version observation currently requires Linux/WSL2".to_owned())
}

fn canonical_primary_checkout_root() -> ProofResult<PathBuf> {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"));
    let metadata = fs::metadata(checkout)
        .map_err(|error| format!("T079 could not inspect primary checkout root: {error}"))?;
    if !metadata.is_dir() {
        return Err("T079 primary checkout root is not a directory".to_owned());
    }
    checkout
        .canonicalize()
        .map_err(|error| format!("T079 could not canonicalize primary checkout root: {error}"))
}

fn ensure_path_outside_primary_checkout(
    candidate: &Path,
    checkout: &Path,
    label: &str,
) -> ProofResult<()> {
    if candidate == checkout || candidate.starts_with(checkout) {
        return Err(format!(
            "T079 refuses {label} inside primary checkout: {}",
            candidate.display()
        ));
    }
    Ok(())
}

fn canonical_directory_outside_primary_checkout(path: &Path, label: &str) -> ProofResult<PathBuf> {
    if !path.is_absolute() {
        return Err(format!("T079 requires {label} to be an absolute path"));
    }
    let metadata =
        fs::metadata(path).map_err(|error| format!("T079 could not inspect {label}: {error}"))?;
    if !metadata.is_dir() {
        return Err(format!("T079 requires {label} to be an existing directory"));
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("T079 could not canonicalize {label}: {error}"))?;
    let checkout = canonical_primary_checkout_root()?;
    ensure_path_outside_primary_checkout(&canonical, &checkout, label)?;
    Ok(canonical)
}

fn disposable_root() -> ProofResult<PathBuf> {
    let parent =
        canonical_directory_outside_primary_checkout(&env::temp_dir(), "temporary parent")?;
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("T079 clock error: {error}"))?
        .as_nanos();
    let root = parent.join(format!(
        "winds-t079-{}-{sequence}-{nanos}",
        std::process::id()
    ));
    fs::create_dir(&root)
        .map_err(|error| format!("T079 could not create disposable root: {error}"))?;
    let canonical = root
        .canonicalize()
        .map_err(|error| format!("T079 could not canonicalize disposable root: {error}"))?;
    if canonical.parent() != Some(parent.as_path()) {
        return Err("T079 disposable root escaped its canonical temporary parent".to_owned());
    }
    let checkout = canonical_primary_checkout_root()?;
    ensure_path_outside_primary_checkout(&canonical, &checkout, "disposable root")?;
    Ok(canonical)
}

fn read_bounded_jsonl_frame<R: BufRead>(reader: &mut R) -> ProofResult<Option<Vec<u8>>> {
    let mut frame = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|error| format!("T079 Codex stdout read failed: {error}"))?;
        if available.is_empty() {
            return if frame.is_empty() {
                Ok(None)
            } else {
                Ok(Some(frame))
            };
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        let new_len = frame
            .len()
            .checked_add(take)
            .ok_or_else(|| "T079 Codex frame byte count overflowed".to_owned())?;
        if new_len > MAX_CODEX_JSONL_FRAME_BYTES {
            return Err("T079 Codex frame exceeded bounded size".to_owned());
        }
        frame.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            return Ok(Some(frame));
        }
    }
}

fn spawn_frame_reader_with_sender(
    stdout: std::process::ChildStdout,
    sender: SyncSender<FrameResult>,
    done: SyncSender<()>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            match read_bounded_jsonl_frame(&mut reader) {
                Ok(Some(frame)) => {
                    if sender.send(Ok(frame)).is_err() {
                        break;
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
        let _ = done.send(());
    })
}

fn record_connected_frame(
    frame: &[u8],
    total_bytes: &mut usize,
    frame_count: &mut usize,
) -> ProofResult<()> {
    *frame_count = frame_count
        .checked_add(1)
        .ok_or_else(|| "T079 connected frame count overflowed".to_owned())?;
    *total_bytes = total_bytes
        .checked_add(frame.len())
        .ok_or_else(|| "T079 connected output byte count overflowed".to_owned())?;
    if *frame_count > MAX_CONNECTED_FRAMES || *total_bytes > MAX_CONNECTED_BYTES {
        return Err("T079 connected output exceeded bounded transcript limits".to_owned());
    }
    Ok(())
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
    record_connected_frame(&frame, total_bytes, frame_count)?;
    Ok(frame)
}

fn drain_post_terminal_frames(
    client: &mut CodexProtocolClient,
    receiver: &Receiver<FrameResult>,
    deadline: Instant,
    total_bytes: &mut usize,
    frame_count: &mut usize,
) -> (ProofResult<()>, bool) {
    let mut failure: Option<String> = None;
    loop {
        let now = Instant::now();
        if now >= deadline {
            let timeout = "T079 stdout reader did not reach EOF inside bounded post-terminal drain";
            return (
                Err(match failure {
                    Some(failure) => format!("{failure}; {timeout}"),
                    None => timeout.to_owned(),
                }),
                false,
            );
        }

        match receiver.recv_timeout(deadline.saturating_duration_since(now)) {
            Ok(Ok(frame)) => {
                if let Err(error) = record_connected_frame(&frame, total_bytes, frame_count) {
                    if failure.is_none() {
                        failure = Some(error);
                    }
                    continue;
                }
                if failure.is_none() {
                    let _ =
                        ingest_t079_frame_with_rejection_metadata(client, &frame, "post-terminal");
                    failure = Some(t079_bounded_protocol_failure(
                        "post-terminal",
                        "UNEXPECTED_POST_TERMINAL_FRAME",
                    ));
                }
            }
            Ok(Err(_)) => {
                if failure.is_none() {
                    failure = Some(t079_bounded_protocol_failure(
                        "post-terminal",
                        "STDOUT_READER_FAILURE",
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return (failure.map_or(Ok(()), Err), true);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let timeout =
                    "T079 stdout reader did not reach EOF inside bounded post-terminal drain";
                return (
                    Err(match failure {
                        Some(failure) => format!("{failure}; {timeout}"),
                        None => timeout.to_owned(),
                    }),
                    false,
                );
            }
        }
    }
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

fn send_line<W: Write>(stdin: &mut W, line: &str) -> ProofResult<()> {
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

fn t079_bounded_protocol_failure(phase: &'static str, category: &'static str) -> String {
    format!("T079 Codex protocol failure: phase={phase};category={category}")
}

fn handle_server_request<W: Write>(
    client: &CodexProtocolClient,
    stdin: &mut W,
    id: &RpcId,
    method: &str,
    phase: &'static str,
) -> ProofResult<()> {
    if matches!(
        method,
        "item/commandExecution/requestApproval" | "item/fileChange/requestApproval"
    ) {
        let decline = client.t079_decline(id).map_err(|error| error.to_string())?;
        send_line(stdin, &decline)?;
        return Err(t079_bounded_protocol_failure(
            phase,
            "UNEXPECTED_AUTHORITY_REQUEST_DECLINED",
        ));
    }
    Err(t079_bounded_protocol_failure(
        phase,
        "UNEXPECTED_SERVER_REQUEST",
    ))
}

#[derive(Debug, Clone, Copy)]
struct T079ResponseWait {
    expected_id: u64,
    phase: &'static str,
}

fn wait_for_response<W: Write>(
    client: &mut CodexProtocolClient,
    stdin: &mut W,
    receiver: &Receiver<FrameResult>,
    wait: T079ResponseWait,
    deadline: Instant,
    total_bytes: &mut usize,
    frame_count: &mut usize,
) -> ProofResult<Value> {
    let T079ResponseWait { expected_id, phase } = wait;
    loop {
        let frame = receive_frame(receiver, deadline, total_bytes, frame_count)?;
        match ingest_t079_frame_with_rejection_metadata(client, &frame, phase)? {
            CodexInbound::Response {
                id: RpcId::Number(id),
                result,
                evidence: EvidenceClass::AgentRuntimeEvidence,
            } if id == expected_id => return Ok(result),
            CodexInbound::ErrorResponse { .. } => {
                return Err(t079_bounded_protocol_failure(phase, "ERROR_RESPONSE"));
            }
            CodexInbound::Notification { method, params, .. } => {
                if is_forbidden_activity(&method, &params) {
                    return Err(t079_bounded_protocol_failure(
                        phase,
                        "FORBIDDEN_RUNTIME_ACTIVITY",
                    ));
                }
            }
            CodexInbound::ServerRequest {
                id,
                method,
                disposition: ServerRequestDisposition::RequiresExternalDecision,
                ..
            } => handle_server_request(client, stdin, &id, &method, phase)?,
            _ => {
                return Err(t079_bounded_protocol_failure(phase, "UNEXPECTED_INBOUND"));
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

#[cfg(target_os = "linux")]
fn cleanup_setup_failure(child: OwnedProcess, root: &Path, message: &str) -> String {
    let cleanup_deadline = Instant::now() + CLEANUP_TIMEOUT;
    let cleanup = finish_t079_process(child, cleanup_deadline, "T079 Codex App Server");
    let root_check = ensure_disposable_root_unchanged(root);
    match (cleanup, root_check) {
        (Ok(_), Ok(())) => message.to_owned(),
        (cleanup, root_check) => {
            format!("{message}; setup cleanup evidence: child={cleanup:?}; root={root_check:?}")
        }
    }
}

#[cfg(target_os = "linux")]
fn run_connected_proof(
    discovery: &RuntimeDiscovery,
    winds_session_id: &str,
    codex_home: &Path,
) -> ProofResult<T079Receipt> {
    validate_t079_connected_proof_platform(env::consts::OS, env::consts::ARCH)?;
    validate_exact_text(winds_session_id, "Winds session id")?;
    let bound_codex_home = bind_preexisting_isolated_codex_home(codex_home)?;
    validate_no_system_codex_config()?;
    validate_discovery(discovery)?;
    let executable = discovery
        .executable
        .as_ref()
        .ok_or_else(|| "T079 discovery lost executable identity".to_owned())?;
    let bound_executable = bind_verified_native_codex_executable(executable)?;
    let version = observe_version_bounded(bound_executable.launch_path())?;
    if discovery.version.value.as_deref() != Some(version.as_str()) {
        return Err("T079 Codex version changed after discovery".to_owned());
    }
    if revalidate_runtime_identity(executable).map_err(|error| error.to_string())?
        != RuntimeIdentityRevalidation::Match
    {
        return Err("T079 Codex executable path changed after handle binding".to_owned());
    }
    bound_codex_home.assert_stable()?;
    validate_no_system_codex_config()?;

    let root = disposable_root()?;
    let _root_guard = EmptyDisposableRootGuard { root: root.clone() };
    let cwd = root
        .to_str()
        .ok_or_else(|| "T079 disposable root is not UTF-8".to_owned())?
        .to_owned();
    let (mut stdin, child_stdin) = UnixStream::pair()
        .map_err(|error| format!("T079 could not create owned Codex stdin channel: {error}"))?;
    let child_stdin = OwnedFd::from(child_stdin);
    let mut command = Command::new(bound_executable.launch_path());
    configure_isolated_codex_environment(&mut command, Some(bound_codex_home.launch_path()));
    configure_t079_codex_authority_reduction(&mut command);
    command
        .args(["app-server", "--stdio"])
        .current_dir(&root)
        .stdin(Stdio::from(child_stdin))
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    configure_bound_codex_home_inheritance(&mut command, &bound_codex_home)?;
    configure_t079_process_descendant_denial(&mut command);
    bound_codex_home.assert_stable()?;
    let mut child = spawn_owned_process(&mut command, "T079 Codex App Server")
        .map_err(|error| format!("T079 could not start owned Codex App Server: {error}"))?;
    if let Err(error) = bound_codex_home.assert_stable() {
        drop(stdin);
        let message = cleanup_setup_failure(child, &root, &error);
        return Err(message);
    }
    let stdout = match child.take_stdout() {
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

    let proof = (|| -> ProofResult<(String, String, String, CodexProtocolClient)> {
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
            T079ResponseWait {
                expected_id: config_id,
                phase: "config/read",
            },
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        validate_effective_config(&config, &version)?;
        bound_codex_home.assert_stable()?;

        let (thread_id, thread_request) = client
            .t079_thread_start(&cwd)
            .map_err(|error| error.to_string())?;
        send_line(&mut stdin, &thread_request)?;
        let thread_result = wait_for_response(
            &mut client,
            &mut stdin,
            &receiver,
            T079ResponseWait {
                expected_id: thread_id,
                phase: "thread/start",
            },
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        let native_thread_id = validate_thread_start_result(&thread_result, &cwd)?;
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
            T079ResponseWait {
                expected_id: turn_request_id,
                phase: "turn/start",
            },
            deadline,
            &mut total_bytes,
            &mut frame_count,
        )?;
        let turn_id = turn_id_from_start_result(&turn_result)?;
        let mut final_agent_message: Option<String> = None;

        loop {
            let frame = receive_frame(&receiver, deadline, &mut total_bytes, &mut frame_count)?;
            match ingest_t079_frame_with_rejection_metadata(&mut client, &frame, "turn/runtime")? {
                CodexInbound::Notification {
                    method,
                    params,
                    evidence: EvidenceClass::AgentRuntimeEvidence,
                } => {
                    if is_forbidden_activity(&method, &params) {
                        return Err(t079_bounded_protocol_failure(
                            "turn/runtime",
                            "FORBIDDEN_RUNTIME_ACTIVITY",
                        ));
                    }
                    if method == "item/completed"
                        && let Some(text) = completed_final_answer_text(&params)?
                    {
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
                            "T079 completed without a bounded final-answer agent message".to_owned()
                        })?;
                        let status = parse_structured_agent_message(text)?;
                        bound_codex_home.assert_stable()?;
                        return Ok((
                            native_thread_id.as_str().to_owned(),
                            turn_id,
                            status,
                            client,
                        ));
                    }
                }
                CodexInbound::ServerRequest {
                    id,
                    method,
                    disposition: ServerRequestDisposition::RequiresExternalDecision,
                    ..
                } => handle_server_request(&client, &mut stdin, &id, &method, "turn/runtime")?,
                CodexInbound::ErrorResponse { .. } => {
                    return Err(t079_bounded_protocol_failure(
                        "turn/runtime",
                        "ERROR_RESPONSE",
                    ));
                }
                CodexInbound::Response { .. } | CodexInbound::InitializeAccepted => {
                    return Err("T079 received an unexpected response after turn/start".to_owned());
                }
            }
        }
    })();

    drop(stdin);
    let cleanup_deadline = Instant::now() + CLEANUP_TIMEOUT;
    let cleanup = finish_t079_process(child, cleanup_deadline, "T079 Codex App Server");
    let (proof, reader_result) = match proof {
        Ok((native_thread_id, turn_id, status, mut client)) => {
            let reader_deadline = Instant::now() + CLEANUP_TIMEOUT;
            let (post_terminal, reader_closed) = drain_post_terminal_frames(
                &mut client,
                &receiver,
                reader_deadline,
                &mut total_bytes,
                &mut frame_count,
            );
            let home_check = bound_codex_home.assert_stable();
            let proof = match (post_terminal, home_check) {
                (Ok(()), Ok(())) => Ok((native_thread_id, turn_id, status)),
                (post_terminal, home_check) => {
                    let mut failures = Vec::new();
                    if let Err(error) = post_terminal {
                        failures.push(format!("post_terminal={error}"));
                    }
                    if let Err(error) = home_check {
                        failures.push(format!("codex_home={error}"));
                    }
                    Err(format!(
                        "T079 terminal proof failure: {}",
                        failures.join("; ")
                    ))
                }
            };
            let reader_result = if reader_closed {
                reader
                    .join()
                    .map_err(|_| "T079 stdout reader panicked during bounded cleanup".to_owned())
            } else {
                Err("T079 stdout reader did not terminate inside bounded cleanup".to_owned())
            };
            (proof, reader_result)
        }
        Err(error) => {
            drop(receiver);
            let reader_result = match wait_for_reader_completion(&done_receiver, cleanup_deadline) {
                Ok(()) => reader
                    .join()
                    .map_err(|_| "T079 stdout reader panicked during bounded cleanup".to_owned()),
                Err(reader_error) => Err(reader_error),
            };
            (Err(error), reader_result)
        }
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

#[cfg(not(target_os = "linux"))]
fn run_connected_proof(
    _discovery: &RuntimeDiscovery,
    _winds_session_id: &str,
    _codex_home: &Path,
) -> ProofResult<T079Receipt> {
    Err(
        "T079 first connected proof currently requires Linux/WSL2 owned-process containment"
            .to_owned(),
    )
}

#[test]
fn t079_connected_proof_platform_preflight_accepts_only_seccomp_supported_linux_architectures() {
    assert!(validate_t079_connected_proof_platform("linux", "x86_64").is_ok());
    assert!(validate_t079_connected_proof_platform("linux", "aarch64").is_ok());

    let unsupported_arch = validate_t079_connected_proof_platform("linux", "riscv64").unwrap_err();
    assert!(unsupported_arch.contains("riscv64"));
    assert!(unsupported_arch.contains("x86_64"));
    assert!(unsupported_arch.contains("aarch64"));

    let unsupported_os = validate_t079_connected_proof_platform("windows", "x86_64").unwrap_err();
    assert!(unsupported_os.contains("Linux/WSL2"));
}

#[test]
fn t079_isolated_codex_environment_disables_remote_control_startup() {
    let mut command = Command::new("codex");
    configure_isolated_codex_environment(&mut command, None);

    let remote_control_disabled = command
        .get_envs()
        .find(|(key, _)| *key == std::ffi::OsStr::new(T079_REMOTE_CONTROL_DISABLED_ENV_VAR))
        .and_then(|(_, value)| value);

    assert_eq!(remote_control_disabled, Some(std::ffi::OsStr::new("1")));
}

#[test]
fn t079_requests_are_fixed_ephemeral_read_only_and_non_authorizing() {
    let mut generic = CodexProtocolClient::default();
    let generic_initialize = generic
        .initialize_request("winds", "Winds", "0.1.0")
        .expect("generic initialize request");
    let generic_initialize = parse_outbound(&generic_initialize);
    assert_eq!(generic_initialize["method"], "initialize");
    assert!(
        generic_initialize["params"].get("capabilities").is_none(),
        "generic initialize must not inherit T079 notification opt-outs"
    );

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
        initialize["params"]["capabilities"]["optOutNotificationMethods"],
        json!(["remoteControl/status/changed"])
    );
    assert_eq!(
        initialize["params"]["capabilities"]
            .as_object()
            .expect("capabilities object")
            .len(),
        2
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

    assert!(matches!(
        client
            .ingest_jsonl_frame(br#"{"id":2,"result":{"thread":{"id":"thr_t079_fixture"}}}"#)
            .expect("thread response"),
        CodexInbound::Response {
            id: RpcId::Number(2),
            evidence: EvidenceClass::AgentRuntimeEvidence,
            ..
        }
    ));

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
fn effective_config_preflight_requires_exact_codex_0_149_surface_and_authority_reduction() {
    let layer_version = format!("sha256:{}", "a".repeat(64));

    let origins = || {
        json!({
            "agents.enabled": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.auth_elicitation": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.apps": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.multi_agent": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.multi_agent_v2.enabled": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.remote_plugin": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.tool_suggest": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "features.shell_tool": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "include_apps_instructions": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            },
            "include_collaboration_mode_instructions": {
                "name": { "type": "sessionFlags" },
                "version": layer_version
            }
        })
    };

    validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": origins()
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .expect("exact Codex 0.149 reduced-authority surface");

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": origins()
        }),
        "codex-cli 0.150.0",
    )
    .unwrap_err();
    assert!(error.contains("qualified only"));

    let expected = expected_t079_codex_0_149_config();
    let keys = expected
        .as_object()
        .expect("expected object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut config = expected.clone();
        config.as_object_mut().expect("config object").remove(&key);

        let error = validate_effective_config(
            &json!({
                "config": config,
                "origins": origins()
            }),
            T079_CODEX_CONFIG_COMPAT_VERSION,
        )
        .unwrap_err();

        assert!(error.contains("key set changed"), "missing {key}: {error}");
    }

    let secret_config_key = "SECRET_CONFIG_KEY_DO_NOT_PRINT";
    let mut config = expected_t079_codex_0_149_config();
    config[secret_config_key] = Value::Null;
    let error = validate_effective_config(
        &json!({
            "config": config,
            "origins": origins()
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("key set changed"));
    assert!(!error.contains(secret_config_key));

    for (label, config) in [
        {
            let mut value = expected_t079_codex_0_149_config();
            value["agents"]["enabled"] = json!(true);
            ("agents.enabled", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["agents"]["researcher"] = json!({
                "description": "unexpected role"
            });
            ("agents role", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["apps"] = json!(true);
            ("apps", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["multi_agent"] = json!(true);
            ("multi_agent", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["multi_agent_v2"] = json!(true);
            ("multi_agent_v2", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["auth_elicitation"] = json!(true);
            ("auth_elicitation", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["remote_plugin"] = json!(true);
            ("remote_plugin", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["tool_suggest"] = json!(true);
            ("tool_suggest", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["features"]["shell_tool"] = json!(true);
            ("shell_tool", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["include_apps_instructions"] = json!(true);
            ("include_apps_instructions", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["include_collaboration_mode_instructions"] = json!(true);
            ("include_collaboration_mode_instructions", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["chatgpt_base_url"] = json!("https://example.invalid/");
            ("chatgpt_base_url", value)
        },
        {
            let mut value = expected_t079_codex_0_149_config();
            value["mcp_servers"] = json!({
                "unexpected": { "command": "x" }
            });
            ("mcp_servers", value)
        },
    ] {
        let error = validate_effective_config(
            &json!({
                "config": config,
                "origins": origins()
            }),
            T079_CODEX_CONFIG_COMPAT_VERSION,
        )
        .unwrap_err();

        assert!(
            error.contains("effective config evidence changed"),
            "{label}: {error}"
        );
    }

    let mut missing_origin = origins();
    missing_origin
        .as_object_mut()
        .expect("origins object")
        .remove("features.apps");

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": missing_origin
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("session-origin surface changed"));

    let secret_origin_key = "SECRET_ORIGIN_KEY_DO_NOT_PRINT";
    let mut extra_origin = origins();
    extra_origin[secret_origin_key] = json!({
        "name": { "type": "sessionFlags" },
        "version": layer_version
    });

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": extra_origin
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("session-origin surface changed"));
    assert!(!error.contains(secret_origin_key));

    let mut wrong_source = origins();
    wrong_source["features.apps"]["name"]["type"] = json!("user");

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": wrong_source
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("non-SessionFlags"));

    let mut malformed_version = origins();
    malformed_version["features.apps"]["version"] = json!("fixture");

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": malformed_version
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("canonical SHA-256"));

    let mut split_layer = origins();
    split_layer["features.apps"]["version"] = json!(format!("sha256:{}", "b".repeat(64)));

    let error = validate_effective_config(
        &json!({
            "config": expected_t079_codex_0_149_config(),
            "origins": split_layer
        }),
        T079_CODEX_CONFIG_COMPAT_VERSION,
    )
    .unwrap_err();
    assert!(error.contains("one SessionFlags layer version"));
}

#[test]
fn t079_codex_launch_applies_fixed_authority_reduction_before_app_server() {
    let mut command = Command::new("codex");
    configure_t079_codex_authority_reduction(&mut command);
    command.args(["app-server", "--stdio"]);

    let args = command
        .get_args()
        .map(|arg| arg.to_str().expect("UTF-8 fixed T079 argument"))
        .collect::<Vec<_>>();

    assert_eq!(
        args,
        vec![
            "-c",
            "agents.enabled=false",
            "-c",
            "features.multi_agent=false",
            "-c",
            "features.multi_agent_v2=false",
            "-c",
            "features.auth_elicitation=false",
            "-c",
            "features.apps=false",
            "-c",
            "features.remote_plugin=false",
            "-c",
            "features.tool_suggest=false",
            "-c",
            "features.shell_tool=false",
            "-c",
            "include_apps_instructions=false",
            "-c",
            "include_collaboration_mode_instructions=false",
            "app-server",
            "--stdio",
        ]
    );
    assert_eq!(
        args.iter()
            .filter(|argument| **argument == "features.apps=false")
            .count(),
        1,
        "T079 must disable Codex Apps exactly once through SessionFlags"
    );
}

#[test]
fn isolated_codex_home_rejects_local_config_without_reading_credentials() {
    let root = disposable_root().expect("isolated home fixture");
    for name in ["config.toml", "hooks.json"] {
        let surface = root.join(name);
        fs::write(&surface, b"fixture\n").expect("write config fixture");
        let error = validate_preexisting_isolated_codex_home(&root).unwrap_err();
        fs::remove_file(&surface).expect("remove config fixture");
        assert!(error.contains(name));
    }
    let canonical = validate_preexisting_isolated_codex_home(&root).expect("clean isolated home");
    fs::remove_dir(&root).expect("remove isolated home fixture");
    assert_eq!(canonical, root);
}

#[cfg(target_os = "linux")]
#[test]
fn t079_bound_codex_home_rejects_path_replacement_and_blocked_config_mutation() {
    let root = disposable_root().expect("bound home path replacement fixture");
    let moved = root.with_extension("bound-original");
    let bound = bind_preexisting_isolated_codex_home(&root).expect("bind isolated home");
    assert!(
        bound
            .launch_path()
            .to_string_lossy()
            .starts_with("/proc/self/fd/")
    );
    fs::rename(&root, &moved).expect("move original isolated home");
    fs::create_dir(&root).expect("replace original isolated home pathname");
    assert_eq!(
        bound
            .launch_path()
            .canonicalize()
            .expect("bound target after rename"),
        moved
    );
    let error = bound.assert_stable().unwrap_err();
    assert!(error.contains("directory identity changed"));
    fs::remove_dir(&root).expect("remove replacement home");
    drop(bound);
    fs::remove_dir(&moved).expect("remove original bound home");

    let config_root = disposable_root().expect("bound home config mutation fixture");
    let bound =
        bind_preexisting_isolated_codex_home(&config_root).expect("bind config fixture home");
    fs::write(config_root.join("config.toml"), b"fixture\n").expect("inject blocked config");
    let error = bound.assert_stable().unwrap_err();
    assert!(error.contains("configuration surface changed"));
    fs::remove_file(config_root.join("config.toml")).expect("remove blocked config");
    drop(bound);
    fs::remove_dir(config_root).expect("remove config fixture home");
}

#[test]
fn isolated_codex_home_rejects_primary_checkout_and_descendants() {
    let checkout = canonical_primary_checkout_root().expect("canonical primary checkout");
    let descendant = checkout.join("src");

    for candidate in [&checkout, &descendant] {
        let error = validate_preexisting_isolated_codex_home(candidate).unwrap_err();
        assert!(error.contains("WINDS_T079_CODEX_HOME"));
        assert!(error.contains("inside primary checkout"));
    }
}

#[test]
fn t079_checkout_containment_rejects_primary_checkout_and_descendants() {
    let checkout = canonical_primary_checkout_root().expect("canonical primary checkout");
    assert!(ensure_path_outside_primary_checkout(&checkout, &checkout, "fixture root").is_err());
    assert!(
        ensure_path_outside_primary_checkout(&checkout.join("tmp"), &checkout, "fixture root")
            .is_err()
    );
}

#[test]
fn t079_disposable_root_is_canonical_and_outside_primary_checkout() {
    let root = disposable_root().expect("safe disposable root");
    let checkout = canonical_primary_checkout_root().expect("canonical primary checkout");
    assert!(root.is_absolute());
    assert!(!root.starts_with(&checkout));
    fs::remove_dir(root).expect("remove safe disposable root");
}

#[cfg(target_os = "macos")]
#[test]
fn t079_macos_managed_preferences_are_fail_closed_without_reading_them() {
    let error = validate_no_system_codex_config().unwrap_err();
    assert!(error.contains("com.openai.codex"));
    assert!(error.contains("config_toml_base64"));
    assert!(error.contains("requirements_toml_base64"));
}

#[test]
fn codex_launch_environment_is_explicit_allowlist_only() {
    let root = disposable_root().expect("isolated home fixture");
    let mut command = Command::new("codex");
    configure_isolated_codex_environment(&mut command, Some(&root));
    let explicit: Vec<_> = command
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|value| {
                (
                    key.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
        })
        .collect();
    let codex_home = explicit
        .iter()
        .find(|(key, _)| key == "CODEX_HOME")
        .expect("CODEX_HOME must be explicit");
    assert_eq!(PathBuf::from(&codex_home.1), root);
    for (key, _) in &explicit {
        assert!(
            key == "CODEX_HOME"
                || key == T079_REMOTE_CONTROL_DISABLED_ENV_VAR
                || SAFE_CODEX_CHILD_ENV_KEYS.contains(&key.as_str())
        );
    }
    for forbidden in [
        "HOME",
        "PATH",
        "OPENAI_API_KEY",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
    ] {
        assert!(!explicit.iter().any(|(key, _)| key == forbidden));
    }
    #[cfg(not(windows))]
    assert!(
        !explicit.iter().any(|(key, _)| key == "TMPDIR"),
        "non-Windows T079 launch must not inherit host TMPDIR"
    );
    fs::remove_dir(root).expect("remove isolated home fixture");
}

#[cfg(target_os = "linux")]
#[test]
fn t079_linux_launch_binding_rejects_wrappers_and_holds_verified_descriptor() {
    use std::os::unix::fs::PermissionsExt;

    let root = disposable_root().expect("launch binding fixture");
    let executable = root.join("codex");
    fs::write(&executable, b"#!/usr/bin/env node\n").expect("write wrapper fixture");
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();
    let error = validate_live_codex_candidate_path(&executable).unwrap_err();
    assert!(error.contains("Linux-native ELF"));

    fs::write(&executable, b"\x7fELFfixture-codex-v1\n").expect("write ELF fixture");
    let (expected, bound) = prepare_bound_codex_version_observation(&executable)
        .expect("pre-version identity bound to sealed descriptor");
    let discovery =
        discover_codex_from_bound_version(&executable, &expected, "codex-cli fixture".to_owned())
            .expect("unchanged source reconciles with pre-version identity");
    assert_eq!(discovery.executable.as_ref(), Some(&expected));
    assert!(
        bound
            .launch_path()
            .to_string_lossy()
            .starts_with("/proc/self/fd/")
    );
    let flags = unsafe { libc::fcntl(bound.file.as_raw_fd(), libc::F_GETFD) };
    assert!(flags >= 0);
    assert_ne!(flags & libc::FD_CLOEXEC, 0);

    let required_seals =
        libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    let seals = unsafe { libc::fcntl(bound.file.as_raw_fd(), libc::F_GET_SEALS) };
    assert!(seals >= 0);
    assert_eq!(seals & required_seals, required_seals);

    let verified_snapshot = fs::read(bound.launch_path()).expect("read sealed verified snapshot");
    fs::write(&executable, b"\x7fELFmutated-after-binding\n")
        .expect("mutate original path after binding");
    assert_ne!(
        fs::read(&executable).expect("read mutated source"),
        verified_snapshot
    );
    assert_eq!(
        fs::read(bound.launch_path()).expect("re-read sealed snapshot"),
        verified_snapshot
    );
    let error =
        discover_codex_from_bound_version(&executable, &expected, "codex-cli fixture".to_owned())
            .unwrap_err();
    assert!(error.contains("identity changed between pre-version binding and discovery"));

    drop(bound);
    fs::remove_file(executable).expect("remove launch fixture");
    fs::remove_dir(root).expect("remove launch fixture root");
}

#[cfg(target_os = "linux")]
#[test]
fn t079_version_observation_uses_disposable_working_directory() {
    use std::os::unix::fs::PermissionsExt;

    let fixture_root = disposable_root().expect("version fixture root");
    let executable = fixture_root.join("codex-version-fixture");
    fs::write(&executable, b"#!/bin/sh\nprintf '%s\\n' \"$PWD\"\n").expect("write version fixture");
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();

    let observed = observe_version_bounded(&executable).expect("bounded version fixture");
    let observed_root = PathBuf::from(observed);
    let checkout = canonical_primary_checkout_root().expect("canonical primary checkout");
    assert!(observed_root.is_absolute());
    assert!(!observed_root.starts_with(&checkout));
    assert!(!observed_root.exists());

    fs::remove_file(executable).expect("remove version fixture");
    fs::remove_dir(fixture_root).expect("remove version fixture root");
}

#[test]
fn jsonl_frame_reader_rejects_newline_free_oversize_input_at_the_cap() {
    let source = std::io::repeat(b'x').take((MAX_CODEX_JSONL_FRAME_BYTES + 1) as u64);
    let mut reader = BufReader::new(source);
    let error = read_bounded_jsonl_frame(&mut reader).unwrap_err();
    assert!(error.contains("frame exceeded bounded size"));
}

#[cfg(target_os = "linux")]
#[test]
fn t079_linux_seccomp_filter_denies_process_descendants() {
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "( : ) & wait"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_t079_process_descendant_denial(&mut command);
    let mut child = spawn_owned_process(&mut command, "T079 descendant-denial regression")
        .expect("spawn filter regression child");
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        match child.try_wait().expect("inspect filter regression child") {
            Some(status) => break status,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            None => {
                child
                    .terminate_direct_t079(
                        Instant::now() + CLEANUP_TIMEOUT,
                        "T079 filter regression",
                    )
                    .expect("terminate filter regression child");
                panic!("T079 descendant-denial regression child did not exit");
            }
        }
    };
    assert!(
        !status.success(),
        "T079 seccomp filter unexpectedly allowed a shell background process"
    );
}

#[test]
fn thread_and_result_validation_preserve_exact_provenance_and_non_authority() {
    let cwd = "/tmp/winds-t079-fixture";
    let native = validate_thread_start_result(
        &json!({
            "thread": { "id": "thr_exact", "cwd": cwd, "ephemeral": true, "path": null },
            "approvalPolicy": "never",
            "sandbox": { "type": "readOnly", "networkAccess": false },
            "runtimeWorkspaceRoots": [],
            "instructionSources": []
        }),
        cwd,
    )
    .expect("exact T079 thread evidence");
    assert_eq!(native.as_str(), "thr_exact");
    assert!(
        validate_thread_start_result(
            &json!({
                "thread": {
                    "id": "thr_exact",
                    "cwd": "/tmp/winds-t079-other",
                    "ephemeral": true,
                    "path": null
                },
                "approvalPolicy": "never",
                "sandbox": { "type": "readOnly", "networkAccess": false },
                "runtimeWorkspaceRoots": [],
                "instructionSources": []
            }),
            cwd,
        )
        .is_err()
    );
    assert!(
        validate_thread_start_result(
            &json!({
                "thread": { "id": "thr_exact", "cwd": cwd, "ephemeral": false, "path": "/persisted" },
                "approvalPolicy": "never",
                "sandbox": { "type": "readOnly", "networkAccess": false },
                "runtimeWorkspaceRoots": [],
                "instructionSources": []
            }),
            cwd,
        )
        .is_err()
    );
    assert!(
        validate_thread_start_result(
            &json!({
                "thread": { "id": "thr_exact", "cwd": cwd, "ephemeral": true, "path": null },
                "approvalPolicy": "never",
                "sandbox": { "type": "readOnly", "networkAccess": false },
                "runtimeWorkspaceRoots": [],
                "instructionSources": ["/home/user/AGENTS.md"]
            }),
            cwd,
        )
        .is_err()
    );
    assert!(
        validate_thread_start_result(
            &json!({
                "thread": { "id": "thr_exact", "cwd": cwd, "ephemeral": true, "path": null },
                "approvalPolicy": "never",
                "sandbox": { "type": "readOnly", "networkAccess": false },
                "runtimeWorkspaceRoots": ["/tmp/winds-t079-fixture"],
                "instructionSources": []
            }),
            cwd,
        )
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
fn t079_only_final_answer_phase_can_populate_structured_proof_result() {
    for phase in [Value::Null, json!("commentary")] {
        let params = json!({
            "item": {
                "type": "agentMessage",
                "phase": phase,
                "text": "{\"status\":\"WINDS_T079_OK\"}"
            }
        });
        assert_eq!(completed_final_answer_text(&params).unwrap(), None);
    }

    let params = json!({
        "item": {
            "type": "agentMessage",
            "phase": "final_answer",
            "text": "{\"status\":\"WINDS_T079_OK\"}"
        }
    });
    assert_eq!(
        completed_final_answer_text(&params).unwrap(),
        Some("{\"status\":\"WINDS_T079_OK\"}")
    );
}

#[test]
fn t079_post_terminal_drain_rejects_queued_frames_instead_of_discarding_them() {
    let (sender, receiver) = mpsc::sync_channel(1);
    sender
        .send(Ok(
            br#"{"method":"future/postTerminal","params":{}}"#.to_vec()
        ))
        .expect("queue post-terminal fixture");
    drop(sender);

    let mut client = initialized_client();
    let mut total_bytes = 0usize;
    let mut frame_count = 0usize;
    let (result, reader_closed) = drain_post_terminal_frames(
        &mut client,
        &receiver,
        Instant::now() + Duration::from_secs(1),
        &mut total_bytes,
        &mut frame_count,
    );
    let error = result.unwrap_err();
    assert!(reader_closed);
    assert!(error.contains("UNEXPECTED_POST_TERMINAL_FRAME"));
    assert_eq!(frame_count, 1);
    assert!(total_bytes > 0);
}

#[test]
fn t079_exact_text_identity_is_bounded_by_protocol_limit() {
    let at_limit = "x".repeat(super::MAX_PROTOCOL_TEXT_BYTES);
    validate_exact_text(&at_limit, "fixture identity").expect("at-limit identity");

    let oversized = "x".repeat(super::MAX_PROTOCOL_TEXT_BYTES + 1);
    let error = validate_exact_text(&oversized, "fixture identity").unwrap_err();
    assert!(error.contains("exact safe text identity"));
    assert!(
        turn_id_from_start_result(&json!({ "turn": { "id": oversized } })).is_err(),
        "server-provided turn ids must share the same protocol text bound"
    );
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
fn t079_server_request_failure_diagnostics_do_not_echo_app_server_values() {
    let client = initialized_client();
    let secret_id_text = "SECRET_RPC_ID_DO_NOT_PRINT";

    for (method, category) in [
        (
            "SECRET_SERVER_METHOD_DO_NOT_PRINT",
            "UNEXPECTED_SERVER_REQUEST",
        ),
        (
            "item/commandExecution/requestApproval",
            "UNEXPECTED_AUTHORITY_REQUEST_DECLINED",
        ),
    ] {
        let mut protocol_output = Vec::new();
        let error = handle_server_request(
            &client,
            &mut protocol_output,
            &RpcId::Text(secret_id_text.to_owned()),
            method,
            "turn/runtime",
        )
        .unwrap_err();

        assert!(error.contains("phase=turn/runtime"));
        assert!(error.contains(&format!("category={category}")));
        assert!(!error.contains(secret_id_text));
        assert!(!error.contains(method));
    }
}

#[test]
fn t079_wait_for_response_error_diagnostics_do_not_echo_error_payload() {
    let mut client = initialized_client();
    let (request_id, _) = client
        .t079_config_read("/tmp/winds-t079-fixture")
        .expect("pending config/read fixture");
    let (sender, receiver) = mpsc::sync_channel(1);
    sender
        .send(Ok(serde_json::to_vec(&json!({
            "id": request_id,
            "error": {
                "code": -32000,
                "message": "SECRET_ERROR_MESSAGE_DO_NOT_PRINT",
                "data": {
                    "SECRET_ERROR_KEY_DO_NOT_PRINT": "SECRET_ERROR_OBJECT_DO_NOT_PRINT",
                    "scalar": "SECRET_ERROR_SCALAR_DO_NOT_PRINT"
                }
            }
        }))
        .expect("serialize error response fixture")))
        .expect("queue error response fixture");
    drop(sender);

    let mut stdin = Vec::new();
    let mut total_bytes = 0usize;
    let mut frame_count = 0usize;
    let error = wait_for_response(
        &mut client,
        &mut stdin,
        &receiver,
        T079ResponseWait {
            expected_id: request_id,
            phase: "config/read",
        },
        Instant::now() + Duration::from_secs(1),
        &mut total_bytes,
        &mut frame_count,
    )
    .unwrap_err();

    assert!(error.contains("phase=config/read"));
    assert!(error.contains("category=ERROR_RESPONSE"));
    for forbidden in [
        "SECRET_ERROR_MESSAGE_DO_NOT_PRINT",
        "SECRET_ERROR_KEY_DO_NOT_PRINT",
        "SECRET_ERROR_OBJECT_DO_NOT_PRINT",
        "SECRET_ERROR_SCALAR_DO_NOT_PRINT",
    ] {
        assert!(!error.contains(forbidden), "diagnostic leaked {forbidden}");
    }
}

#[test]
fn t079_wait_for_response_server_request_diagnostics_do_not_echo_request_payload() {
    let mut client = initialized_client();
    let (request_id, _) = client
        .t079_config_read("/tmp/winds-t079-fixture")
        .expect("pending config/read fixture");
    let secret_id = "SECRET_SERVER_REQUEST_ID_DO_NOT_PRINT";
    let secret_method = "SECRET_SERVER_REQUEST_METHOD_DO_NOT_PRINT";
    let secret_key = "SECRET_SERVER_REQUEST_KEY_DO_NOT_PRINT";
    let secret_value = "SECRET_SERVER_REQUEST_VALUE_DO_NOT_PRINT";
    let (sender, receiver) = mpsc::sync_channel(1);
    sender
        .send(Ok(serde_json::to_vec(&json!({
            "id": secret_id,
            "method": secret_method,
            "params": { secret_key: secret_value }
        }))
        .expect("serialize server request fixture")))
        .expect("queue server request fixture");
    drop(sender);

    let mut stdin = Vec::new();
    let mut total_bytes = 0usize;
    let mut frame_count = 0usize;
    let error = wait_for_response(
        &mut client,
        &mut stdin,
        &receiver,
        T079ResponseWait {
            expected_id: request_id,
            phase: "config/read",
        },
        Instant::now() + Duration::from_secs(1),
        &mut total_bytes,
        &mut frame_count,
    )
    .unwrap_err();

    assert!(error.contains("phase=config/read"));
    assert!(error.contains("category=UNEXPECTED_SERVER_REQUEST"));
    for forbidden in [secret_id, secret_method, secret_key, secret_value] {
        assert!(!error.contains(forbidden), "diagnostic leaked {forbidden}");
    }
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
fn t079_rejection_metadata_exposes_shape_without_untrusted_identifiers_or_scalar_values() {
    let frame = br#"{"method":"apiKeySecret","params":{"credentialToken":"DO_NOT_PRINT","thread":{"privateIdentifier":"secret-thread","secretPath":"/secret/path"},"item":{"apiSecretType":"secretType","privateText":"secret-text"}}}"#;
    let metadata = t079_rejection_metadata(frame, "fixture-phase");

    assert!(metadata.contains("phase=fixture-phase"));
    assert!(metadata.contains("method_shape=STRING"));
    assert!(metadata.contains("METHOD_CLASS=UNKNOWN_METHOD"));
    assert!(metadata.contains("params_shape=OBJECT"));
    assert!(metadata.contains("param_key_count=3"));
    assert!(metadata.contains("KNOWN_KEY_COUNT=0"));
    assert!(metadata.contains("UNKNOWN_KEY_COUNT=3"));
    assert!(metadata.contains("thread_key_count=2"));
    assert!(metadata.contains("item_key_count=2"));

    for forbidden in [
        "apiKeySecret",
        "credentialToken",
        "privateIdentifier",
        "secretPath",
        "apiSecretType",
        "privateText",
        "DO_NOT_PRINT",
        "secret-thread",
        "/secret/path",
        "secretType",
        "secret-text",
    ] {
        assert!(
            !metadata.contains(forbidden),
            "diagnostic leaked untrusted identifier or scalar value: {forbidden}"
        );
    }
}

#[test]
fn t079_model_rerouted_is_classified_but_remains_fail_closed() {
    let frame = serde_json::to_vec(&json!({
        "method": "model/rerouted",
        "params": {
            "threadId": "thr_fixture",
            "turnId": "turn_fixture",
            "fromModel": "fixture-a",
            "toModel": "fixture-b",
            "reason": "highRiskCyberActivity"
        }
    }))
    .expect("serialize model/rerouted fixture");

    let mut direct = initialized_client();
    assert_eq!(
        direct.ingest_jsonl_frame(&frame),
        Err(CodexProtocolError::UnexpectedT079Notification)
    );

    let mut classified = initialized_client();
    let error = ingest_t079_frame_with_rejection_metadata(&mut classified, &frame, "turn/start")
        .unwrap_err();
    assert!(error.contains("METHOD_CLASS=KNOWN_MODEL_REROUTED"));
    assert!(error.contains("KNOWN_KEY_COUNT=5"));
    assert!(error.contains("UNKNOWN_KEY_COUNT=0"));
    for forbidden in [
        "model/rerouted",
        "thr_fixture",
        "turn_fixture",
        "fixture-a",
        "fixture-b",
        "highRiskCyberActivity",
    ] {
        assert!(!error.contains(forbidden), "diagnostic leaked {forbidden}");
    }
}

#[test]
fn t079_terminal_interaction_is_classified_but_remains_fail_closed() {
    assert_eq!(
        T079RejectedMethodClass::KnownTerminalInteraction.known_param_keys(),
        &["itemId", "processId", "stdin", "threadId", "turnId"]
    );

    let frame = serde_json::to_vec(&json!({
        "method": "item/commandExecution/terminalInteraction",
        "params": {
            "itemId": "item-secret",
            "processId": "process-secret",
            "stdin": "private-stdin",
            "threadId": "thread-secret",
            "turnId": "turn-secret"
        }
    }))
    .expect("serialize terminal interaction fixture");

    let mut direct = initialized_client();
    assert_eq!(
        direct.ingest_jsonl_frame(&frame),
        Err(CodexProtocolError::UnexpectedT079Notification),
        "terminal interaction unexpectedly became admissible"
    );

    let mut classified = initialized_client();
    let error = ingest_t079_frame_with_rejection_metadata(&mut classified, &frame, "turn/start")
        .unwrap_err();
    assert!(error.contains("METHOD_CLASS=KNOWN_TERMINAL_INTERACTION"));
    assert!(error.contains("param_key_count=5"));
    assert!(error.contains("KNOWN_KEY_COUNT=5"));
    assert!(error.contains("UNKNOWN_KEY_COUNT=0"));

    for forbidden in [
        "item/commandExecution/terminalInteraction",
        "item-secret",
        "process-secret",
        "private-stdin",
        "thread-secret",
        "turn-secret",
    ] {
        assert!(!error.contains(forbidden), "diagnostic leaked {forbidden}");
    }
}

#[test]
fn t079_rejected_known_codex_0149_notifications_stay_rejected() {
    let cases = [
        (
            "model/verification",
            json!({
                "threadId": "SECRET_TOKEN_DO_NOT_PRINT",
                "turnId": "turn-secret",
                "verifications": []
            }),
            "KNOWN_MODEL_VERIFICATION",
        ),
        (
            "model/safetyBuffering/updated",
            json!({
                "threadId": "SECRET_TOKEN_DO_NOT_PRINT",
                "turnId": "turn-secret",
                "model": "model-secret-name",
                "useCases": ["private-use-case"],
                "reasons": ["private-reason"],
                "showBufferingUi": true,
                "fasterModel": null
            }),
            "KNOWN_MODEL_SAFETY_BUFFERING_UPDATED",
        ),
        (
            "turn/moderationMetadata",
            json!({
                "threadId": "SECRET_TOKEN_DO_NOT_PRINT",
                "turnId": "turn-secret",
                "metadata": {"credentialToken": "private-value"}
            }),
            "KNOWN_TURN_MODERATION_METADATA",
        ),
        (
            "error",
            json!({
                "error": {"message": "SECRET_TOKEN_DO_NOT_PRINT"},
                "willRetry": false,
                "threadId": "thread-secret",
                "turnId": "turn-secret"
            }),
            "KNOWN_ERROR_NOTIFICATION",
        ),
        (
            "warning",
            json!({
                "threadId": null,
                "message": "SECRET_TOKEN_DO_NOT_PRINT"
            }),
            "KNOWN_WARNING",
        ),
        (
            "guardianWarning",
            json!({
                "threadId": "thread-secret",
                "message": "SECRET_TOKEN_DO_NOT_PRINT"
            }),
            "KNOWN_GUARDIAN_WARNING",
        ),
    ];

    for (method, params, expected_class) in cases {
        let frame = serde_json::to_vec(&json!({"method": method, "params": params}))
            .expect("serialize rejected known notification fixture");
        let mut client = initialized_client();
        assert_eq!(
            client.ingest_jsonl_frame(&frame),
            Err(CodexProtocolError::UnexpectedT079Notification),
            "{method} unexpectedly became admissible"
        );
        let metadata = t079_rejection_metadata(&frame, "fixture-phase");
        assert!(metadata.contains(&format!("METHOD_CLASS={expected_class}")));
        assert!(metadata.contains("UNKNOWN_KEY_COUNT=0"));
        assert!(!metadata.contains(method));
        for forbidden in [
            "SECRET_TOKEN_DO_NOT_PRINT",
            "private-value",
            "model-secret-name",
            "credentialToken",
            "thread-secret",
            "turn-secret",
        ] {
            assert!(
                !metadata.contains(forbidden),
                "diagnostic leaked {forbidden}"
            );
        }
    }
}

#[test]
fn t079_known_method_extra_key_is_counted_without_key_or_value_leakage() {
    let frame = serde_json::to_vec(&json!({
        "method": "model/rerouted",
        "params": {
            "threadId": "thread-secret",
            "turnId": "turn-secret",
            "fromModel": "model-secret-name",
            "toModel": "other-secret-model",
            "reason": "highRiskCyberActivity",
            "unknownSecretField": "SECRET_TOKEN_DO_NOT_PRINT"
        }
    }))
    .expect("serialize extra-key fixture");

    let mut client = initialized_client();
    assert_eq!(
        client.ingest_jsonl_frame(&frame),
        Err(CodexProtocolError::UnexpectedT079Notification)
    );
    let metadata = t079_rejection_metadata(&frame, "turn/start");
    assert!(metadata.contains("METHOD_CLASS=KNOWN_MODEL_REROUTED"));
    assert!(metadata.contains("KNOWN_KEY_COUNT=5"));
    assert!(metadata.contains("UNKNOWN_KEY_COUNT=1"));
    for forbidden in [
        "unknownSecretField",
        "SECRET_TOKEN_DO_NOT_PRINT",
        "thread-secret",
        "turn-secret",
        "model-secret-name",
        "other-secret-model",
        "highRiskCyberActivity",
    ] {
        assert!(
            !metadata.contains(forbidden),
            "diagnostic leaked {forbidden}"
        );
    }
}

#[test]
fn t079_five_key_candidates_receive_distinct_static_classes() {
    for (method, params, expected_class) in [
        (
            "model/rerouted",
            json!({
                "threadId": "thread-secret",
                "turnId": "turn-secret",
                "fromModel": "model-a",
                "toModel": "model-b",
                "reason": "private-reason"
            }),
            "KNOWN_MODEL_REROUTED",
        ),
        (
            "item/reasoning/summaryTextDelta",
            json!({
                "threadId": "thread-secret",
                "turnId": "turn-secret",
                "itemId": "item-secret",
                "delta": "private-delta",
                "summaryIndex": 0
            }),
            "KNOWN_REASONING_SUMMARY_TEXT_DELTA",
        ),
        (
            "item/reasoning/textDelta",
            json!({
                "threadId": "thread-secret",
                "turnId": "turn-secret",
                "itemId": "item-secret",
                "delta": "private-delta",
                "contentIndex": 0
            }),
            "KNOWN_REASONING_TEXT_DELTA",
        ),
    ] {
        let frame = serde_json::to_vec(&json!({"method": method, "params": params}))
            .expect("serialize five-key fixture");
        let metadata = t079_rejection_metadata(&frame, "turn/start");
        assert!(metadata.contains(&format!("METHOD_CLASS={expected_class}")));
        assert!(metadata.contains("KNOWN_KEY_COUNT=5"));
        assert!(metadata.contains("UNKNOWN_KEY_COUNT=0"));
        assert!(!metadata.contains(method));
        for forbidden in [
            "thread-secret",
            "turn-secret",
            "item-secret",
            "private-delta",
            "model-a",
            "model-b",
            "private-reason",
        ] {
            assert!(
                !metadata.contains(forbidden),
                "diagnostic leaked {forbidden}"
            );
        }
    }
}

#[test]
fn t079_unexpected_notification_error_reports_only_rejection_metadata() {
    let mut client = initialized_client();
    let frame = br#"{"method":"apiKeySecret","params":{"credentialToken":"PRIVATE_VALUE"}}"#;

    let error =
        ingest_t079_frame_with_rejection_metadata(&mut client, frame, "fixture-wait").unwrap_err();

    assert!(error.contains("rejected notification metadata"));
    assert!(error.contains("phase=fixture-wait"));
    assert!(error.contains("method_shape=STRING"));
    assert!(error.contains("METHOD_CLASS=UNKNOWN_METHOD"));
    assert!(error.contains("params_shape=OBJECT"));
    assert!(error.contains("param_key_count=1"));
    assert!(error.contains("KNOWN_KEY_COUNT=0"));
    assert!(error.contains("UNKNOWN_KEY_COUNT=1"));
    for forbidden in ["apiKeySecret", "credentialToken", "PRIVATE_VALUE"] {
        assert!(
            !error.contains(forbidden),
            "enriched error leaked untrusted identifier or scalar value: {forbidden}"
        );
    }
}

#[test]
fn t079_rejection_diagnostics_do_not_add_protocol_state_mutation() {
    let frame = br#"{"method":"apiKeySecret","params":{"credentialToken":"PRIVATE_VALUE"}}"#;
    let mut direct = initialized_client();
    let mut wrapped = initialized_client();

    assert_eq!(
        direct.ingest_jsonl_frame(frame),
        Err(CodexProtocolError::UnexpectedT079Notification)
    );
    let _ = ingest_t079_frame_with_rejection_metadata(&mut wrapped, frame, "fixture-wait")
        .expect_err("wrapped rejection remains fail closed");

    assert_eq!(direct.state, wrapped.state);
    assert_eq!(direct.next_request_id, wrapped.next_request_id);
    assert_eq!(direct.t079_mode, wrapped.t079_mode);
    assert_eq!(direct.t079_requests, wrapped.t079_requests);
    assert_eq!(
        direct.t079_thread_start_issued,
        wrapped.t079_thread_start_issued
    );
    assert_eq!(direct.t079_thread_id, wrapped.t079_thread_id);
    assert_eq!(direct.t079_turn_id, wrapped.t079_turn_id);
}

#[test]
fn t079_accepted_path_does_not_compute_rejection_metadata() {
    let mut client = initialized_client();
    let _ = client
        .t079_config_read("/tmp/winds-t079-fixture")
        .expect("pending config/read fixture");
    let frame = serde_json::to_vec(&json!({
        "method": "configWarning",
        "params": {
            "summary": super::T079_MISSING_SYSTEM_BWRAP_WARNING,
            "details": null
        }
    }))
    .expect("serialize admitted configWarning fixture");

    T079_REJECTION_METADATA_CALLS.with(|calls| calls.set(0));
    assert!(matches!(
        ingest_t079_frame_with_rejection_metadata(&mut client, &frame, "config/read")
            .expect("exact config warning remains admitted"),
        CodexInbound::Notification { method, .. } if method == "configWarning"
    ));
    T079_REJECTION_METADATA_CALLS.with(|calls| assert_eq!(calls.get(), 0));

    let mut rejected = initialized_client();
    let rejected_frame = br#"{"method":"future/unknown","params":{}}"#;
    let _ = ingest_t079_frame_with_rejection_metadata(
        &mut rejected,
        rejected_frame,
        "fixture-rejection",
    )
    .expect_err("unknown notification remains rejected");
    T079_REJECTION_METADATA_CALLS.with(|calls| assert_eq!(calls.get(), 1));
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
        cleanup: CleanupEvidence::OwnedScopeQuiescent,
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
#[ignore = "T079 live proof: requires Linux/WSL2 x86_64 or aarch64, a pre-existing locally authenticated native Codex binary, and isolated CODEX_HOME; no install/auth/terms/credential automation"]
fn t079_real_codex_one_bounded_prompt() {
    validate_t079_connected_proof_platform(env::consts::OS, env::consts::ARCH)
        .expect("supported T079 connected-proof platform");
    let executable = PathBuf::from(
        env::var("WINDS_T079_CODEX_PATH")
            .expect("set WINDS_T079_CODEX_PATH to an existing Linux-native Codex executable"),
    );
    let codex_home =
        PathBuf::from(env::var_os("WINDS_T079_CODEX_HOME").expect(
            "set WINDS_T079_CODEX_HOME to a pre-existing isolated authenticated Codex home",
        ));
    let winds_session_id = env::var("WINDS_T079_WINDS_SESSION_ID")
        .expect("set WINDS_T079_WINDS_SESSION_ID to the exact canonical Winds session id");
    let codex_home = validate_preexisting_isolated_codex_home(&codex_home)
        .expect("pre-existing isolated CODEX_HOME without local config surfaces");
    validate_no_system_codex_config().expect("no unvalidated system Codex config surfaces");
    let (pre_version_identity, bound_version_executable) =
        prepare_bound_codex_version_observation(&executable)
            .expect("static Codex identity bound before first version observation");
    let version = observe_version_bounded(bound_version_executable.launch_path())
        .expect("bounded Codex version observation through sealed snapshot");
    let discovery = discover_codex_from_bound_version(&executable, &pre_version_identity, version)
        .expect("exact Codex discovery matches pre-version static identity");
    drop(bound_version_executable);
    let receipt = run_connected_proof(&discovery, &winds_session_id, &codex_home)
        .expect("bounded T079 proof");

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
            "launch_environment": "ENV_CLEAR_EXPLICIT_ALLOWLIST",
            "launch_executable": "LINUX_NATIVE_VERIFIED_OPEN_FD",
            "process_descendants": "KERNEL_DENIED_SECCOMP",
            "codex_home": "PREEXISTING_ISOLATED_BOUND_OPEN_FD_NOT_READ_OR_COPIED_BY_WINDS",
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
