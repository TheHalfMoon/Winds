use crate::agentic_codex::{
    CodexInbound, CodexProtocolClient, EvidenceClass, MAX_CODEX_JSONL_FRAME_BYTES, NativeThreadId,
    RpcId, ServerRequestDisposition, T079_PROOF_PROMPT,
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
use std::fs;
#[cfg(target_os = "linux")]
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(target_os = "linux")]
use std::io::{Seek, SeekFrom};
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, OwnedFd};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "linux")]
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
#[cfg(target_os = "linux")]
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
const SAFE_CODEX_CHILD_ENV_KEYS: &[&str] = &["TMPDIR"];
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

type ProofResult<T> = std::result::Result<T, String>;
type FrameResult = std::result::Result<Vec<u8>, String>;

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
    file: File,
    launch_path: PathBuf,
}

impl BoundCodexExecutable {
    fn launch_path(&self) -> &Path {
        &self.launch_path
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

    if config.get("model_provider").and_then(Value::as_str) != Some("openai") {
        return Err(
            "T079 requires effective model_provider=openai for the connected proof".to_owned(),
        );
    }

    for (key, value) in config {
        if !meaningful(value) {
            continue;
        }
        if !ALLOWED_EFFECTIVE_CONFIG_KEYS.contains(&key.as_str()) {
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
    if !path.is_absolute() {
        return Err("T079 requires WINDS_T079_CODEX_HOME to be an absolute path".to_owned());
    }
    let metadata = fs::metadata(path)
        .map_err(|error| format!("T079 could not inspect pre-existing CODEX_HOME: {error}"))?;
    if !metadata.is_dir() {
        return Err("T079 requires WINDS_T079_CODEX_HOME to name an existing directory".to_owned());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("T079 could not canonicalize pre-existing CODEX_HOME: {error}"))?;
    for name in BLOCKED_CODEX_CONFIG_FILES {
        reject_config_surface(
            &canonical.join(name),
            "a local CODEX_HOME configuration surface",
        )?;
    }
    Ok(canonical)
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
    if let Some(codex_home) = codex_home {
        command.env("CODEX_HOME", codex_home);
    }
    for key in SAFE_CODEX_CHILD_ENV_KEYS {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
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
    let mut file = File::open(&expected.canonical_path)
        .map_err(|error| format!("T079 could not open verified Codex executable: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("T079 could not inspect verified Codex executable: {error}"))?;
    if !metadata.is_file() || metadata.len() != expected.byte_len {
        return Err("T079 Codex executable metadata changed before handle binding".to_owned());
    }
    require_linux_native_elf(&mut file)?;

    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut total_read = 0_u64;
    loop {
        let read = file
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
    if revalidate_runtime_identity(expected).map_err(|error| error.to_string())?
        != RuntimeIdentityRevalidation::Match
    {
        return Err("T079 Codex executable path changed while binding launch identity".to_owned());
    }

    let fd = file.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(format!(
            "T079 could not inspect Codex executable descriptor flags: {}",
            std::io::Error::last_os_error()
        ));
    }
    if flags & libc::FD_CLOEXEC == 0 {
        return Err(
            "T079 requires the bound Codex executable descriptor to remain close-on-exec in the parent process"
                .to_owned(),
        );
    }

    let launch_path = PathBuf::from(format!("/proc/self/fd/{fd}"));
    let launch_metadata = fs::metadata(&launch_path)
        .map_err(|error| format!("T079 could not prove bound Codex descriptor path: {error}"))?;
    if !launch_metadata.is_file() || launch_metadata.len() != expected.byte_len {
        return Err("T079 bound Codex descriptor does not preserve executable identity".to_owned());
    }

    Ok(BoundCodexExecutable { file, launch_path })
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
                // The T079 seccomp filter rejects fork/vfork and every clone
                // that is not CLONE_THREAD, while clone3 is forced through the
                // libc fallback path. A reaped direct child therefore implies
                // there cannot be independently running process descendants.
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
    let mut command = Command::new(executable);
    configure_isolated_codex_environment(&mut command, None);
    command
        .arg("--version")
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
}

#[cfg(not(target_os = "linux"))]
fn observe_version_bounded(executable: &Path) -> ProofResult<String> {
    let _ = executable;
    Err("T079 bounded Codex version observation currently requires Linux/WSL2".to_owned())
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

fn handle_server_request<W: Write>(
    client: &CodexProtocolClient,
    stdin: &mut W,
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

fn wait_for_response<W: Write>(
    client: &mut CodexProtocolClient,
    stdin: &mut W,
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
    validate_exact_text(winds_session_id, "Winds session id")?;
    let codex_home = validate_preexisting_isolated_codex_home(codex_home)?;
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
    let revalidated_codex_home = validate_preexisting_isolated_codex_home(&codex_home)?;
    if revalidated_codex_home != codex_home {
        return Err("T079 CODEX_HOME identity changed before launch".to_owned());
    }
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
    configure_isolated_codex_environment(&mut command, Some(&codex_home));
    command
        .args(["app-server", "--stdio"])
        .current_dir(&root)
        .stdin(Stdio::from(child_stdin))
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    configure_t079_process_descendant_denial(&mut command);
    let mut child = spawn_owned_process(&mut command, "T079 Codex App Server")
        .map_err(|error| format!("T079 could not start owned Codex App Server: {error}"))?;
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
    let cleanup = finish_t079_process(child, cleanup_deadline, "T079 Codex App Server");
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
            "model_provider": "openai",
            "mcp_servers": null,
            "hooks": [],
            "apps": null,
            "instructions": null,
            "developer_instructions": null,
            "tools": null,
            "web_search": "disabled"
        }
    }))
    .expect("allowed OpenAI model fields plus empty/disabled surfaces are acceptable");

    for config in [
        json!({ "config": {} }),
        json!({ "config": { "model_provider": null } }),
    ] {
        let error = validate_effective_config(&config).unwrap_err();
        assert!(error.contains("model_provider"));
    }

    for provider in ["ollama", "azure"] {
        let error = validate_effective_config(&json!({
            "config": { "model_provider": provider }
        }))
        .unwrap_err();
        assert!(error.contains("model_provider"));
    }

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
        config.insert("model_provider".to_owned(), json!("openai"));
        config.insert(key.to_owned(), value);
        let error = validate_effective_config(&json!({ "config": config })).unwrap_err();
        assert!(error.contains(key));
    }
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
        assert!(key == "CODEX_HOME" || SAFE_CODEX_CHILD_ENV_KEYS.contains(&key.as_str()));
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
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("codex-cli fixture".to_owned()),
        &[],
        &[],
    )
    .expect("native fixture discovery");
    let expected = discovery.executable.as_ref().expect("fixture executable");
    let bound = bind_verified_native_codex_executable(expected).expect("bound verified descriptor");
    assert!(
        bound
            .launch_path()
            .to_string_lossy()
            .starts_with("/proc/self/fd/")
    );
    let flags = unsafe { libc::fcntl(bound.file.as_raw_fd(), libc::F_GETFD) };
    assert!(flags >= 0);
    assert_ne!(flags & libc::FD_CLOEXEC, 0);

    drop(bound);
    fs::remove_file(executable).expect("remove launch fixture");
    fs::remove_dir(root).expect("remove launch fixture root");
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
                    .terminate_direct_t079(Instant::now() + CLEANUP_TIMEOUT, "T079 filter regression")
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
        ("item/started", json!({"item": {"type": "hookPrompt"}}),
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
#[ignore = "T079 live proof: requires Linux/WSL2, a pre-existing locally authenticated native Codex binary, and isolated CODEX_HOME; no install/auth/terms/credential automation"]
fn t079_real_codex_one_bounded_prompt() {
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
    validate_live_codex_candidate_path(&executable)
        .expect("Linux-native Codex executable required for handle-bound launch");
    let version = observe_version_bounded(&executable).expect("bounded Codex version observation");
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed(version),
        &[],
        &[],
    )
    .expect("exact Codex discovery");
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
            "codex_home": "PREEXISTING_ISOLATED_NOT_READ_OR_COPIED_BY_WINDS",
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
