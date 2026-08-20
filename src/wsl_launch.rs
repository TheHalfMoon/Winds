use super::Result;
#[cfg(windows)]
use super::process_scope::{OwnedProcess, operation_deadlines, spawn_owned_process};
use super::terminal::{TerminalSession, TerminalSize};
use super::wsl::WslDistribution;
#[cfg(windows)]
use super::wsl::discover_wsl_distributions;
#[cfg(windows)]
use super::{GIT_CONTEXT_ENV_VARS, Repo, run_read_only_git_text, strip_git_line_ending};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::atomic::{AtomicU64, Ordering};

const WSL_SHELL_EXECUTABLE: &str = "/bin/sh";

#[cfg(windows)]
static NEXT_WSL_EXEC_SCOPE_ID: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
const WSL_OWNED_SCOPE_SCRIPT: &str = r#"
token="$1"
timeout_seconds="$2"
shift 2

for required in /usr/bin/setsid /bin/sh /bin/sleep /bin/kill; do
    if [ ! -x "$required" ]; then
        printf '__WINDS_WSL_SCOPE_UNSUPPORTED_%s__:%s\n' "$token" "$required" >&2
        exit 125
    fi
done
if ! /bin/sleep 0.01 2>/dev/null; then
    printf '__WINDS_WSL_SCOPE_UNSUPPORTED_%s__:%s\n' "$token" '/bin/sleep:fractional-seconds' >&2
    exit 125
fi

/usr/bin/setsid /bin/sh -c '
    /bin/sleep 86400 &
    sentinel=$!
    if ! /bin/kill -0 "$sentinel" 2>/dev/null; then
        exit 125
    fi
    exec "$@"
' winds-wsl-target "$@" &
target_leader=$!

/usr/bin/setsid /bin/sh -c '
    /bin/sleep "$1"
    printf "__WINDS_WSL_SCOPE_TIMEOUT_%s__\n" "$2" >&2
    /bin/kill -KILL -- "$3" 2>/dev/null || :
' winds-wsl-watchdog "$timeout_seconds" "$token" "$target_leader" &
watchdog=$!

wait "$target_leader"
target_status=$?

/bin/kill -KILL -- "-$watchdog" 2>/dev/null || :
wait "$watchdog" 2>/dev/null || :

if /bin/kill -0 -- "-$target_leader" 2>/dev/null; then
    if ! /bin/kill -KILL -- "-$target_leader" 2>/dev/null; then
        printf '__WINDS_WSL_SCOPE_UNPROVEN_%s__:group-kill\n' "$token" >&2
        exit 125
    fi
fi

checks=0
while /bin/kill -0 -- "-$target_leader" 2>/dev/null; do
    checks=$((checks + 1))
    if [ "$checks" -ge 100 ]; then
        printf '__WINDS_WSL_SCOPE_UNPROVEN_%s__:quiescence\n' "$token" >&2
        exit 125
    fi
    if ! /bin/sleep 0.01; then
        printf '__WINDS_WSL_SCOPE_UNPROVEN_%s__:sleep-failed\n' "$token" >&2
        exit 125
    fi
done

printf '__WINDS_WSL_SCOPE_CLEAN_%s__:%s\n' "$token" "$target_status" >&2
exit "$target_status"
"#;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WslExecutionDomain {
    pub host_os: String,
    pub host_arch: String,
    pub distribution: String,
    pub version: u8,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WslCwdStrategy {
    MappedWorkspaceOrHomeFallback,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WslTerminalProfile {
    pub profile_id: String,
    pub display_name: String,
    pub execution_domain: WslExecutionDomain,
    pub launcher_executable: String,
    pub shell_executable: String,
    pub shell_arguments: Vec<String>,
    pub cwd_strategy: WslCwdStrategy,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WslCwdResolution {
    MappedWorkspace {
        windows_workspace_root: String,
        linux_workspace_root: String,
        linux_git_common_dir: String,
        git_head_oid: String,
    },
    FallbackHome {
        requested_windows_workspace_root: String,
        linux_home: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WslTerminalLaunchPlan {
    pub profile: WslTerminalProfile,
    pub cwd_resolution: WslCwdResolution,
}

pub struct WslLaunchedTerminal {
    pub session: TerminalSession,
    pub profile: WslTerminalProfile,
    pub cwd_resolution: WslCwdResolution,
}

#[cfg(windows)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct WslWorkspaceAttestation {
    linux_workspace_root: String,
    linux_git_common_dir: String,
    git_head_oid: String,
}

#[cfg(windows)]
pub fn prepare_wsl_terminal_launch(
    workspace: &Path,
    selected_distribution: &str,
) -> Result<WslTerminalLaunchPlan> {
    use super::wsl::system_wsl_executable;

    let distribution = fresh_selected_distribution(selected_distribution)?;
    let launcher = system_wsl_executable()?;
    let launcher_text = launcher
        .to_str()
        .ok_or("WSL launcher path is not valid UTF-8")?
        .to_owned();
    let repo = Repo::open(workspace)?;
    let windows_workspace_root = utf8_windows_path(repo.root(), "canonical workspace root")?;
    let profile = build_profile(&launcher_text, &distribution);

    let cwd_resolution = match map_and_attest_workspace(&launcher, &distribution, &repo) {
        Ok(attestation) => WslCwdResolution::MappedWorkspace {
            windows_workspace_root,
            linux_workspace_root: attestation.linux_workspace_root,
            linux_git_common_dir: attestation.linux_git_common_dir,
            git_head_oid: attestation.git_head_oid,
        },
        Err(mapping_error) => {
            let linux_home = resolve_linux_home(&launcher, &distribution)?;
            WslCwdResolution::FallbackHome {
                requested_windows_workspace_root: windows_workspace_root,
                linux_home,
                reason: format!("workspace mapping could not be proven: {mapping_error}"),
            }
        }
    };

    Ok(WslTerminalLaunchPlan {
        profile,
        cwd_resolution,
    })
}

#[cfg(not(windows))]
pub fn prepare_wsl_terminal_launch(
    _workspace: &Path,
    _selected_distribution: &str,
) -> Result<WslTerminalLaunchPlan> {
    Err("WSL terminal launch is only available on a native Windows host".into())
}

#[cfg(windows)]
pub fn launch_wsl_terminal(
    plan: &WslTerminalLaunchPlan,
    size: TerminalSize,
) -> Result<WslLaunchedTerminal> {
    use super::wsl::system_wsl_executable;

    let distribution = fresh_selected_distribution(&plan.profile.execution_domain.distribution)?;
    if distribution.version != plan.profile.execution_domain.version {
        return Err(format!(
            "WSL distribution version changed after launch preparation: {} was WSL{} and is now WSL{}",
            distribution.name, plan.profile.execution_domain.version, distribution.version
        )
        .into());
    }

    let launcher = system_wsl_executable()?;
    let launcher_text = launcher
        .to_str()
        .ok_or("WSL launcher path is not valid UTF-8")?;
    if launcher_text != plan.profile.launcher_executable {
        return Err("WSL launcher identity changed after launch preparation".into());
    }
    let expected_profile = build_profile(launcher_text, &distribution);
    validate_profile_for_launch(&plan.profile, &expected_profile)?;

    let (windows_workspace_root, linux_cwd) = match &plan.cwd_resolution {
        WslCwdResolution::MappedWorkspace {
            windows_workspace_root,
            linux_workspace_root,
            ..
        } => (windows_workspace_root, linux_workspace_root),
        WslCwdResolution::FallbackHome {
            requested_windows_workspace_root,
            linux_home,
            ..
        } => (requested_windows_workspace_root, linux_home),
    };
    let workspace_path = Path::new(windows_workspace_root);
    let current_repo = Repo::open(workspace_path)?;
    let current_root = utf8_windows_path(current_repo.root(), "canonical workspace root")?;
    if current_root != *windows_workspace_root {
        return Err(
            "WSL launch plan workspace root is no longer the canonical Git worktree root".into(),
        );
    }
    if let WslCwdResolution::FallbackHome { linux_home, .. } = &plan.cwd_resolution {
        let current_home = resolve_linux_home(&launcher, &distribution)?;
        if current_home != *linux_home {
            return Err("WSL fallback home changed after launch preparation".into());
        }
    }

    if let WslCwdResolution::MappedWorkspace {
        linux_workspace_root,
        linux_git_common_dir,
        git_head_oid,
        ..
    } = &plan.cwd_resolution
    {
        let attestation = attest_workspace(
            &launcher,
            &distribution,
            linux_workspace_root,
            &current_repo,
        )?;
        if attestation.linux_workspace_root != *linux_workspace_root
            || attestation.linux_git_common_dir != *linux_git_common_dir
            || attestation.git_head_oid != *git_head_oid
        {
            return Err("WSL mapped workspace identity changed before terminal launch".into());
        }
    }

    let arguments = build_launch_arguments(
        &distribution.name,
        linux_cwd,
        &expected_profile.shell_executable,
        &expected_profile.shell_arguments,
    )?;
    let session = TerminalSession::start_exact_launch(
        &expected_profile.profile_id,
        &launcher,
        &arguments,
        Path::new(windows_workspace_root),
        size,
    )?;

    if let WslCwdResolution::MappedWorkspace {
        linux_workspace_root,
        linux_git_common_dir,
        git_head_oid,
        ..
    } = &plan.cwd_resolution
    {
        let repo = match Repo::open(Path::new(windows_workspace_root)) {
            Ok(repo) => repo,
            Err(error) => {
                return fail_after_started_wsl_session(
                    session,
                    format!(
                        "WSL mapped workspace could not be reopened after terminal launch: {error}"
                    ),
                );
            }
        };
        match attest_workspace(&launcher, &distribution, linux_workspace_root, &repo) {
            Ok(attestation)
                if attestation.linux_workspace_root == *linux_workspace_root
                    && attestation.linux_git_common_dir == *linux_git_common_dir
                    && attestation.git_head_oid == *git_head_oid => {}
            Ok(_) => {
                return fail_after_started_wsl_session(
                    session,
                    "WSL mapped workspace identity changed after terminal launch".to_owned(),
                );
            }
            Err(error) => {
                return fail_after_started_wsl_session(
                    session,
                    format!(
                        "WSL mapped workspace could not be revalidated after terminal launch: {error}"
                    ),
                );
            }
        }
    }

    Ok(WslLaunchedTerminal {
        session,
        profile: expected_profile,
        cwd_resolution: plan.cwd_resolution.clone(),
    })
}

fn validate_profile_for_launch(
    profile: &WslTerminalProfile,
    expected: &WslTerminalProfile,
) -> Result<()> {
    if profile.execution_domain != expected.execution_domain {
        return Err("WSL terminal profile execution domain is unsupported or stale".into());
    }
    if profile.launcher_executable != expected.launcher_executable {
        return Err("WSL terminal profile launcher executable is unsupported or stale".into());
    }
    if profile.shell_executable != expected.shell_executable {
        return Err("WSL terminal profile shell executable is unsupported or stale".into());
    }
    if profile.shell_arguments != expected.shell_arguments {
        return Err("WSL terminal profile shell arguments are unsupported or stale".into());
    }
    if profile.cwd_strategy != expected.cwd_strategy {
        return Err("WSL terminal profile cwd strategy is unsupported or stale".into());
    }
    if profile.profile_id != expected.profile_id {
        return Err("WSL terminal profile identity does not match its launch data".into());
    }
    Ok(())
}

#[cfg(windows)]
fn fail_after_started_wsl_session(
    mut session: TerminalSession,
    reason: String,
) -> Result<WslLaunchedTerminal> {
    match session.terminate() {
        Ok(_) => Err(format!("{reason}; owned session terminated").into()),
        Err(cleanup_error) => Err(format!(
            "{reason}; owned session termination could not be proven: {cleanup_error}"
        )
        .into()),
    }
}

#[cfg(not(windows))]
pub fn launch_wsl_terminal(
    _plan: &WslTerminalLaunchPlan,
    _size: TerminalSize,
) -> Result<WslLaunchedTerminal> {
    Err("WSL terminal launch is only available on a native Windows host".into())
}

fn build_profile(launcher: &str, distribution: &WslDistribution) -> WslTerminalProfile {
    let execution_domain = WslExecutionDomain {
        host_os: "windows".to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        distribution: distribution.name.clone(),
        version: distribution.version,
    };
    let shell_arguments = Vec::new();
    let cwd_strategy = WslCwdStrategy::MappedWorkspaceOrHomeFallback;
    let profile_id = stable_profile_id(
        &execution_domain,
        launcher,
        WSL_SHELL_EXECUTABLE,
        &shell_arguments,
        cwd_strategy,
    );
    WslTerminalProfile {
        profile_id,
        display_name: format!("WSL: {} / {}", distribution.name, WSL_SHELL_EXECUTABLE),
        execution_domain,
        launcher_executable: launcher.to_owned(),
        shell_executable: WSL_SHELL_EXECUTABLE.to_owned(),
        shell_arguments,
        cwd_strategy,
    }
}

fn stable_profile_id(
    domain: &WslExecutionDomain,
    launcher: &str,
    shell: &str,
    shell_arguments: &[String],
    cwd_strategy: WslCwdStrategy,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"WindsWslTerminalProfileV1\0");
    digest.update(domain.host_os.as_bytes());
    digest.update(b"\0");
    digest.update(domain.host_arch.as_bytes());
    digest.update(b"\0");
    digest.update(domain.distribution.as_bytes());
    digest.update(b"\0");
    digest.update([domain.version]);
    digest.update(b"\0");
    digest.update(launcher.as_bytes());
    digest.update(b"\0");
    digest.update(shell.as_bytes());
    digest.update(b"\0");
    for argument in shell_arguments {
        digest.update(argument.as_bytes());
        digest.update(b"\0");
    }
    match cwd_strategy {
        WslCwdStrategy::MappedWorkspaceOrHomeFallback => {
            digest.update(b"MAPPED_WORKSPACE_OR_HOME_FALLBACK\0")
        }
    }
    let hex: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("wsl-terminal-profile-{hex}")
}

fn build_launch_arguments(
    distribution: &str,
    linux_cwd: &str,
    shell: &str,
    shell_arguments: &[String],
) -> Result<Vec<String>> {
    if distribution.is_empty() {
        return Err("WSL distribution identity cannot be empty".into());
    }
    require_absolute_linux_path(linux_cwd, "WSL terminal cwd")?;
    require_absolute_linux_path(shell, "WSL shell executable")?;
    if shell_arguments
        .iter()
        .any(|argument| argument.contains('\0'))
    {
        return Err("WSL shell arguments cannot contain NUL".into());
    }
    let mut arguments = vec![
        "--distribution".to_owned(),
        distribution.to_owned(),
        "--cd".to_owned(),
        linux_cwd.to_owned(),
        "--exec".to_owned(),
        shell.to_owned(),
    ];
    arguments.extend(shell_arguments.iter().cloned());
    Ok(arguments)
}

fn require_absolute_linux_path(value: &str, label: &str) -> Result<()> {
    if value.starts_with('/') && !value.contains('\0') && !value.contains(['\r', '\n']) {
        return Ok(());
    }
    Err(format!("{label} must be one absolute single-line Linux path").into())
}

#[cfg(windows)]
fn fresh_selected_distribution(name: &str) -> Result<WslDistribution> {
    if name.is_empty() {
        return Err("selected WSL distribution name cannot be empty".into());
    }
    discover_wsl_distributions()?
        .into_iter()
        .find(|distribution| distribution.name == name)
        .ok_or_else(|| format!("selected WSL distribution is not installed: {name:?}").into())
}

#[cfg(windows)]
fn map_and_attest_workspace(
    launcher: &Path,
    distribution: &WslDistribution,
    repo: &Repo,
) -> Result<WslWorkspaceAttestation> {
    let windows_workspace_root = repo.root();
    let mapped = wslpath_to_linux(launcher, distribution, windows_workspace_root)?;
    attest_workspace(launcher, distribution, &mapped, repo)
}

#[cfg(windows)]
fn attest_workspace(
    launcher: &Path,
    distribution: &WslDistribution,
    linux_cwd: &str,
    repo: &Repo,
) -> Result<WslWorkspaceAttestation> {
    require_absolute_linux_path(linux_cwd, "mapped WSL workspace cwd")?;

    let effective_cwd = run_linux_path_command(
        launcher,
        distribution,
        Some(linux_cwd),
        "/bin/pwd",
        &["-P"],
        "effective WSL cwd",
    )?;
    let linux_workspace_root = run_linux_path_command(
        launcher,
        distribution,
        Some(linux_cwd),
        "/usr/bin/git",
        &["rev-parse", "--show-toplevel"],
        "WSL Git worktree root",
    )?;
    let linux_git_common_dir = run_linux_path_command(
        launcher,
        distribution,
        Some(linux_cwd),
        "/usr/bin/git",
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        "WSL Git common directory",
    )?;
    let linux_head_oid = run_linux_text_command(
        launcher,
        distribution,
        Some(linux_cwd),
        "/usr/bin/git",
        &["rev-parse", "--verify", "HEAD^{commit}"],
        "WSL Git HEAD",
    )?;
    let windows_head = run_read_only_git_text(
        repo.root(),
        ["rev-parse", "--verify", "HEAD^{commit}"],
        "Windows Git HEAD attestation",
    )?;
    let windows_head_oid = strip_git_line_ending(&windows_head);
    if windows_head_oid.is_empty() {
        return Err("Windows Git returned an empty HEAD object id".into());
    }

    let effective_windows = wslpath_to_windows(launcher, distribution, &effective_cwd)?;
    let root_windows = wslpath_to_windows(launcher, distribution, &linux_workspace_root)?;
    let common_windows = wslpath_to_windows(launcher, distribution, &linux_git_common_dir)?;

    require_same_canonical_windows_path(&effective_windows, repo.root(), "effective WSL cwd")?;
    require_same_canonical_windows_path(&root_windows, repo.root(), "WSL Git worktree root")?;
    require_same_canonical_windows_path(
        &common_windows,
        &repo.common_dir,
        "WSL Git common directory",
    )?;
    if linux_head_oid != windows_head_oid {
        return Err(format!(
            "WSL Git HEAD does not match Windows Git HEAD: WSL {linux_head_oid}, Windows {windows_head_oid}"
        )
        .into());
    }

    Ok(WslWorkspaceAttestation {
        linux_workspace_root,
        linux_git_common_dir,
        git_head_oid: linux_head_oid,
    })
}

#[cfg(windows)]
fn resolve_linux_home(launcher: &Path, distribution: &WslDistribution) -> Result<String> {
    run_linux_path_command(
        launcher,
        distribution,
        Some("~"),
        "/bin/pwd",
        &["-P"],
        "WSL fallback home",
    )
}

#[cfg(windows)]
fn wslpath_to_linux(
    launcher: &Path,
    distribution: &WslDistribution,
    windows_path: &Path,
) -> Result<String> {
    let output = run_wsl_exec(
        launcher,
        &distribution.name,
        None,
        "/usr/bin/wslpath",
        &[windows_path.as_os_str().to_os_string()],
    )?;
    parse_single_linux_path(&output, "wslpath Windows-to-Linux result")
}

#[cfg(windows)]
fn wslpath_to_windows(
    launcher: &Path,
    distribution: &WslDistribution,
    linux_path: &str,
) -> Result<PathBuf> {
    use std::ffi::OsString;

    let output = run_wsl_exec(
        launcher,
        &distribution.name,
        None,
        "/usr/bin/wslpath",
        &[OsString::from("-w"), OsString::from(linux_path)],
    )?;
    let text = parse_single_text(&output, "wslpath Linux-to-Windows result")?;
    let path = PathBuf::from(text);
    if !path.is_absolute() {
        return Err("wslpath Linux-to-Windows result is not absolute".into());
    }
    Ok(path)
}

#[cfg(windows)]
fn run_linux_text_command(
    launcher: &Path,
    distribution: &WslDistribution,
    cwd: Option<&str>,
    command: &str,
    args: &[&str],
    label: &str,
) -> Result<String> {
    use std::ffi::OsString;

    let args: Vec<OsString> = args.iter().map(OsString::from).collect();
    let output = run_wsl_exec(launcher, &distribution.name, cwd, command, &args)?;
    parse_single_text(&output, label)
}

#[cfg(windows)]
fn run_linux_path_command(
    launcher: &Path,
    distribution: &WslDistribution,
    cwd: Option<&str>,
    command: &str,
    args: &[&str],
    label: &str,
) -> Result<String> {
    use std::ffi::OsString;

    let args: Vec<OsString> = args.iter().map(OsString::from).collect();
    let output = run_wsl_exec(launcher, &distribution.name, cwd, command, &args)?;
    parse_single_linux_path(&output, label)
}

#[cfg(windows)]
fn run_wsl_exec(
    launcher: &Path,
    distribution: &str,
    cwd: Option<&str>,
    command: &str,
    command_args: &[std::ffi::OsString],
) -> Result<Vec<u8>> {
    run_wsl_exec_with_limits(
        launcher,
        distribution,
        cwd,
        command,
        command_args,
        20,
        std::time::Duration::from_secs(30),
    )
}

#[cfg(windows)]
fn run_wsl_exec_with_limits(
    launcher: &Path,
    distribution: &str,
    cwd: Option<&str>,
    command: &str,
    command_args: &[std::ffi::OsString],
    linux_scope_timeout_seconds: u64,
    total_timeout: std::time::Duration,
) -> Result<Vec<u8>> {
    use super::wsl::decode_wsl_text;
    use std::ffi::c_void;
    use std::io::{Read, Result as IoResult};
    use std::os::windows::io::AsRawHandle;
    use std::process::{ChildStderr, ChildStdout, Command, Stdio};
    use std::ptr;
    use std::thread;
    use std::time::{Duration, Instant};

    const CAP: usize = 256 * 1024;
    const CONTROL_TAIL_CAP: usize = 16 * 1024;
    const ERROR_BROKEN_PIPE: i32 = 109;
    const ERROR_NO_DATA: i32 = 232;
    const ERROR_PIPE_NOT_CONNECTED: i32 = 233;
    const OWNED_LABEL: &str = "WSL command launcher";

    if linux_scope_timeout_seconds == 0 {
        return Err("WSL-side command scope timeout must be positive".into());
    }
    let minimum_total = Duration::from_secs(linux_scope_timeout_seconds.saturating_add(3));
    if total_timeout <= minimum_total {
        return Err(
            "host WSL timeout must leave a cleanup margin after the Linux-side scope timeout"
                .into(),
        );
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "PeekNamedPipe"]
        fn peek_named_pipe(
            named_pipe: *mut c_void,
            buffer: *mut c_void,
            buffer_size: u32,
            bytes_read: *mut u32,
            total_bytes_available: *mut u32,
            bytes_left_this_message: *mut u32,
        ) -> i32;
    }

    fn append_control_tail(tail: &mut Vec<u8>, bytes: &[u8]) {
        if bytes.len() >= CONTROL_TAIL_CAP {
            tail.clear();
            tail.extend_from_slice(&bytes[bytes.len() - CONTROL_TAIL_CAP..]);
            return;
        }
        let overflow = tail
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(CONTROL_TAIL_CAP);
        if overflow > 0 {
            tail.drain(..overflow);
        }
        tail.extend_from_slice(bytes);
    }

    fn read_available<R: Read + AsRawHandle>(
        reader: &mut R,
        captured: &mut Vec<u8>,
        truncated: &mut bool,
        control_tail: Option<&mut Vec<u8>>,
    ) -> IoResult<bool> {
        let mut available = 0_u32;
        let peeked = unsafe {
            peek_named_pipe(
                reader.as_raw_handle(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut available,
                ptr::null_mut(),
            )
        };
        if peeked == 0 {
            let error = std::io::Error::last_os_error();
            if matches!(
                error.raw_os_error(),
                Some(ERROR_BROKEN_PIPE | ERROR_NO_DATA | ERROR_PIPE_NOT_CONNECTED)
            ) {
                return Ok(false);
            }
            return Err(error);
        }
        if available == 0 {
            return Ok(false);
        }

        let mut buffer = [0_u8; 8192];
        let read_len = buffer.len().min(available as usize);
        let count = reader.read(&mut buffer[..read_len])?;
        if count == 0 {
            return Ok(false);
        }
        if let Some(tail) = control_tail {
            append_control_tail(tail, &buffer[..count]);
        }
        let remaining = CAP.saturating_sub(captured.len());
        let keep = remaining.min(count);
        captured.extend_from_slice(&buffer[..keep]);
        if keep < count {
            *truncated = true;
        }
        Ok(true)
    }

    fn drain_pair(
        stdout: &mut ChildStdout,
        stderr: &mut ChildStderr,
        stdout_bytes: &mut Vec<u8>,
        stderr_bytes: &mut Vec<u8>,
        stderr_control_tail: &mut Vec<u8>,
        stdout_truncated: &mut bool,
        stderr_truncated: &mut bool,
    ) -> IoResult<bool> {
        let stdout_progress = read_available(stdout, stdout_bytes, stdout_truncated, None)?;
        let stderr_progress = read_available(
            stderr,
            stderr_bytes,
            stderr_truncated,
            Some(stderr_control_tail),
        )?;
        Ok(stdout_progress || stderr_progress)
    }

    fn control_value(tail: &[u8], prefix: &str) -> Option<String> {
        String::from_utf8_lossy(tail)
            .lines()
            .rev()
            .find_map(|line| line.strip_prefix(prefix).map(str::to_owned))
    }

    fn has_control_line(tail: &[u8], expected: &str) -> bool {
        String::from_utf8_lossy(tail)
            .lines()
            .any(|line| line == expected)
    }

    fn diagnostic_text(bytes: &[u8], token: &str) -> String {
        let decoded =
            decode_wsl_text(bytes).unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned());
        let filtered = decoded
            .lines()
            .filter(|line| !(line.starts_with("__WINDS_WSL_SCOPE_") && line.contains(token)))
            .collect::<Vec<_>>()
            .join("\n");
        let trimmed = filtered.trim();
        let mut chars = trimmed.chars();
        let mut diagnostic: String = chars.by_ref().take(2048).collect();
        if chars.next().is_some() {
            diagnostic.push_str("...");
        }
        diagnostic
    }

    fn truncation_suffix(stdout_truncated: bool, stderr_truncated: bool) -> &'static str {
        match (stdout_truncated, stderr_truncated) {
            (true, true) => " [stdout truncated] [stderr truncated]",
            (true, false) => " [stdout truncated]",
            (false, true) => " [stderr truncated]",
            (false, false) => "",
        }
    }

    fn fail_without_linux_scope_proof(
        child: &mut OwnedProcess,
        cleanup_deadline: Instant,
        reason: impl std::fmt::Display,
    ) -> Result<Vec<u8>> {
        let windows_cleanup = child.terminate_and_prove(cleanup_deadline, OWNED_LABEL);
        match windows_cleanup {
            Ok(()) => Err(format!(
                "{reason}; WSL-side owned command scope cleanup is unproven because no Linux cleanup marker was observed; Windows launcher process-scope cleanup was proven"
            )
            .into()),
            Err(cleanup_error) => Err(format!(
                "{reason}; WSL-side owned command scope cleanup is unproven because no Linux cleanup marker was observed; Windows launcher process-scope cleanup was also not proven: {cleanup_error}"
            )
            .into()),
        }
    }

    let scope_sequence = NEXT_WSL_EXEC_SCOPE_ID.fetch_add(1, Ordering::Relaxed);
    let scope_token = format!("{:08x}{scope_sequence:016x}", std::process::id());
    let clean_prefix = format!("__WINDS_WSL_SCOPE_CLEAN_{scope_token}__:");
    let timeout_line = format!("__WINDS_WSL_SCOPE_TIMEOUT_{scope_token}__");
    let unproven_prefix = format!("__WINDS_WSL_SCOPE_UNPROVEN_{scope_token}__:");
    let unsupported_prefix = format!("__WINDS_WSL_SCOPE_UNSUPPORTED_{scope_token}__:");

    let mut process = Command::new(launcher);
    for key in GIT_CONTEXT_ENV_VARS {
        process.env_remove(key);
    }
    process.arg("--distribution").arg(distribution);
    if let Some(cwd) = cwd {
        process.arg("--cd").arg(cwd);
    }
    process
        .arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(WSL_OWNED_SCOPE_SCRIPT)
        .arg("winds-wsl-scope")
        .arg(&scope_token)
        .arg(linux_scope_timeout_seconds.to_string())
        .arg(command)
        .args(command_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let started = Instant::now();
    let (command_deadline, cleanup_deadline) = operation_deadlines(started, total_timeout);
    let mut child = spawn_owned_process(&mut process, OWNED_LABEL).map_err(|error| {
        format!(
            "failed to execute selected WSL distribution in an owned Windows process scope: {error}"
        )
    })?;

    let mut stdout = match child.take_stdout() {
        Some(stdout) => stdout,
        None => {
            return fail_without_linux_scope_proof(
                &mut child,
                cleanup_deadline,
                "failed to capture WSL command stdout",
            );
        }
    };
    let mut stderr = match child.take_stderr() {
        Some(stderr) => stderr,
        None => {
            return fail_without_linux_scope_proof(
                &mut child,
                cleanup_deadline,
                "failed to capture WSL command stderr",
            );
        }
    };

    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut stderr_control_tail = Vec::new();
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;

    let status = loop {
        let progressed = match drain_pair(
            &mut stdout,
            &mut stderr,
            &mut stdout_bytes,
            &mut stderr_bytes,
            &mut stderr_control_tail,
            &mut stdout_truncated,
            &mut stderr_truncated,
        ) {
            Ok(progressed) => progressed,
            Err(error) => {
                return fail_without_linux_scope_proof(
                    &mut child,
                    cleanup_deadline,
                    format!("failed reading selected WSL command output: {error}"),
                );
            }
        };

        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= command_deadline => {
                return fail_without_linux_scope_proof(
                    &mut child,
                    cleanup_deadline,
                    format!(
                        "selected WSL command exceeded the bounded host execution phase before its Linux-side cleanup proof (Linux scope deadline: {linux_scope_timeout_seconds}s)"
                    ),
                );
            }
            Ok(None) => {}
            Err(error) => {
                return fail_without_linux_scope_proof(
                    &mut child,
                    cleanup_deadline,
                    format!("failed observing selected WSL command launcher exit: {error}"),
                );
            }
        }

        if !progressed {
            let now = Instant::now();
            if now < command_deadline {
                thread::sleep(
                    Duration::from_millis(10).min(command_deadline.saturating_duration_since(now)),
                );
            }
        }
    };

    // Post-exit pipe draining is cleanup work, but it must not consume the
    // entire reserved cleanup window. Give draining at most half of the
    // remaining cleanup budget so process-scope termination still has time
    // to run if a descendant keeps an inherited pipe continuously writable.
    let post_exit_drain_deadline = {
        let now = Instant::now();
        now + cleanup_deadline.saturating_duration_since(now) / 2
    };
    loop {
        if Instant::now() >= post_exit_drain_deadline {
            let cleanup = child.terminate_and_prove(cleanup_deadline, OWNED_LABEL);
            return Err(format!(
                "selected WSL command launcher exited, but post-exit output draining exceeded the reserved cleanup deadline; WSL-side cleanup proof cannot be trusted; bounded Windows launcher cleanup {}",
                cleanup
                    .map(|()| "was proven".to_owned())
                    .unwrap_or_else(|error| format!("was not proven: {error}"))
            )
            .into());
        }
        match drain_pair(
            &mut stdout,
            &mut stderr,
            &mut stdout_bytes,
            &mut stderr_bytes,
            &mut stderr_control_tail,
            &mut stdout_truncated,
            &mut stderr_truncated,
        ) {
            Ok(true) => continue,
            Ok(false) => break,
            Err(error) => {
                let cleanup = child.terminate_and_prove(cleanup_deadline, OWNED_LABEL);
                return Err(format!(
                    "selected WSL command launcher exited, but output draining failed and WSL-side cleanup proof cannot be trusted: {error}; bounded Windows launcher cleanup {}",
                    cleanup
                        .map(|()| "was proven".to_owned())
                        .unwrap_or_else(|cleanup_error| format!("was not proven: {cleanup_error}"))
                )
                .into());
            }
        }
    }

    // The launcher has exited and output has been drained. Scope quiescence is
    // cleanup work and must consume only the reserved cleanup budget.
    match child.wait_for_scope_quiescence(cleanup_deadline, OWNED_LABEL) {
        Ok(true) => {}
        Ok(false) => {
            let cleanup = child.terminate_and_prove(cleanup_deadline, OWNED_LABEL);
            return Err(format!(
                "Windows WSL launcher direct child exited while its owned Windows process scope remained live; bounded cleanup {}",
                cleanup
                    .map(|()| "was proven".to_owned())
                    .unwrap_or_else(|error| format!("was not proven: {error}"))
            )
            .into());
        }
        Err(error) => {
            let cleanup = child.terminate_and_prove(cleanup_deadline, OWNED_LABEL);
            return Err(format!(
                "Windows WSL launcher process-scope quiescence could not be inspected: {error}; bounded cleanup {}",
                cleanup
                    .map(|()| "was proven".to_owned())
                    .unwrap_or_else(|cleanup_error| format!("was not proven: {cleanup_error}"))
            )
            .into());
        }
    }

    if let Some(required) = control_value(&stderr_control_tail, &unsupported_prefix) {
        return Err(format!(
            "selected WSL distribution lacks a required owned-scope primitive ({required}); command was not admitted"
        )
        .into());
    }
    if let Some(reason) = control_value(&stderr_control_tail, &unproven_prefix) {
        return Err(
            format!("WSL-side owned command scope cleanup could not be proven: {reason}").into(),
        );
    }

    let target_status = control_value(&stderr_control_tail, &clean_prefix)
        .ok_or(
            "WSL-side owned command scope cleanup is unproven: the Linux supervisor exited without its cleanup marker",
        )?
        .parse::<i32>()
        .map_err(|_| "WSL-side cleanup marker contained an invalid target exit status")?;

    if status.code() != Some(target_status) {
        return Err(format!(
            "WSL-side cleanup marker/launcher exit mismatch: Linux target status {target_status}, Windows launcher status {status}; cleanup truth is ambiguous"
        )
        .into());
    }

    let stderr_diagnostic = diagnostic_text(&stderr_bytes, &scope_token);
    let suffix = truncation_suffix(stdout_truncated, stderr_truncated);

    if has_control_line(&stderr_control_tail, &timeout_line) {
        return Err(format!(
            "selected WSL command exceeded the {linux_scope_timeout_seconds} second WSL-side safety timeout; owned Linux process-group cleanup was proven{suffix}"
        )
        .into());
    }
    if stdout_truncated || stderr_truncated {
        let diagnostic = if stderr_diagnostic.is_empty() {
            suffix.to_owned()
        } else {
            format!("{suffix}; stderr: {stderr_diagnostic}")
        };
        return Err(format!(
            "selected WSL command exceeded the 256 KiB per-stream safety bound after WSL-side cleanup was proven{diagnostic}"
        )
        .into());
    }
    if !status.success() {
        return Err(format!(
            "selected WSL command failed with status {status} after WSL-side cleanup was proven: {stderr_diagnostic}"
        )
        .into());
    }

    Ok(stdout_bytes)
}

#[cfg(all(windows, test))]
pub(crate) fn prove_wsl_exec_scope_cleanup_for_test(distribution: &str) -> Result<()> {
    use super::wsl::system_wsl_executable;
    use std::ffi::OsString;
    use std::time::Duration;

    let launcher = system_wsl_executable()?;
    let descendant_script = "/bin/sleep 120 & child=$!; printf '%s\\n' \"$child\"; exit 0";
    let output = run_wsl_exec_with_limits(
        &launcher,
        distribution,
        None,
        "/bin/sh",
        &[OsString::from("-c"), OsString::from(descendant_script)],
        2,
        Duration::from_secs(8),
    )?;
    let descendant_pid = parse_single_text(&output, "WSL scope descendant pid")?;
    if descendant_pid.is_empty() || !descendant_pid.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("WSL scope regression returned an invalid descendant pid".into());
    }

    let absence_script = "if /bin/kill -0 \"$1\" 2>/dev/null; then exit 91; else exit 0; fi";
    run_wsl_exec_with_limits(
        &launcher,
        distribution,
        None,
        "/bin/sh",
        &[
            OsString::from("-c"),
            OsString::from(absence_script),
            OsString::from("winds-wsl-scope-check"),
            OsString::from(&descendant_pid),
        ],
        2,
        Duration::from_secs(8),
    )
    .map_err(|error| {
        format!("WSL-side descendant survived the completed owned attestation scope: {error}")
    })?;

    let timeout_error = run_wsl_exec_with_limits(
        &launcher,
        distribution,
        None,
        "/bin/sleep",
        &[OsString::from("120")],
        1,
        Duration::from_secs(6),
    )
    .unwrap_err()
    .to_string();
    if !timeout_error.contains("1 second WSL-side safety timeout")
        || !timeout_error.contains("cleanup was proven")
    {
        return Err(format!(
            "WSL-side timeout regression did not report proven scope cleanup: {timeout_error}"
        )
        .into());
    }

    // Regression for the post-exit drain bound. This test-only arbitrary
    // command deliberately escapes the tracked Linux process group and keeps
    // stdout continuously writable after the supervised target exits. The
    // production call graph does not expose arbitrary commands through this
    // helper; the fixture exists only to prove host-side draining is bounded.
    let post_exit_writer_script = r#"/usr/bin/setsid /bin/sh -c '/bin/sleep 7 </dev/null >/dev/null 2>&1 & timer=$!; while /bin/kill -0 "$timer" 2>/dev/null; do printf x; done; wait "$timer" 2>/dev/null || :' & exit 0"#;
    let drain_started = std::time::Instant::now();
    let drain_error = run_wsl_exec_with_limits(
        &launcher,
        distribution,
        None,
        "/bin/sh",
        &[
            OsString::from("-c"),
            OsString::from(post_exit_writer_script),
        ],
        1,
        Duration::from_secs(5),
    )
    .unwrap_err()
    .to_string();
    let drain_elapsed = drain_started.elapsed();
    if !drain_error.contains("post-exit output draining exceeded the reserved cleanup deadline")
        || !drain_error.contains("WSL-side cleanup proof cannot be trusted")
        || drain_elapsed > Duration::from_secs(6)
    {
        return Err(format!(
            "WSL post-exit drain regression was not bounded as expected: elapsed={drain_elapsed:?}, error={drain_error}"
        )
        .into());
    }
    Ok(())
}

#[cfg(windows)]
fn require_same_canonical_windows_path(
    observed: &Path,
    expected: &Path,
    label: &str,
) -> Result<()> {
    let observed = observed
        .canonicalize()
        .map_err(|error| format!("{label} cannot be canonicalized on Windows: {error}"))?;
    if observed != expected {
        return Err(format!(
            "{label} does not resolve to the selected Windows workspace identity: observed {}, expected {}",
            observed.display(),
            expected.display()
        )
        .into());
    }
    Ok(())
}

#[cfg(windows)]
fn utf8_windows_path(path: &Path, label: &str) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{label} is not valid UTF-8").into())
}

#[cfg(any(windows, test))]
fn parse_single_text(bytes: &[u8], label: &str) -> Result<String> {
    use super::wsl::decode_wsl_text;

    let text = decode_wsl_text(bytes)?;
    let mut lines = text.lines();
    let value = lines.next().ok_or_else(|| format!("{label} was empty"))?;
    if value.is_empty() || value.contains('\0') {
        return Err(format!("{label} was empty or contained NUL").into());
    }
    if lines.next().is_some() {
        return Err(format!("{label} returned multiple lines").into());
    }
    Ok(value.to_owned())
}

#[cfg(any(windows, test))]
fn parse_single_linux_path(bytes: &[u8], label: &str) -> Result<String> {
    let value = parse_single_text(bytes, label)?;
    require_absolute_linux_path(&value, label)?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        WslCwdStrategy, WslExecutionDomain, WslTerminalProfile, build_launch_arguments,
        parse_single_linux_path, stable_profile_id, validate_profile_for_launch,
    };

    #[test]
    fn launch_arguments_bind_distribution_cwd_and_exact_shell_without_shell_parsing() {
        let args =
            build_launch_arguments("Ubuntu Dev", "/mnt/c/work space/repo", "/bin/sh", &[]).unwrap();
        assert_eq!(
            args,
            [
                "--distribution",
                "Ubuntu Dev",
                "--cd",
                "/mnt/c/work space/repo",
                "--exec",
                "/bin/sh",
            ]
        );
    }

    #[test]
    fn launch_arguments_reject_relative_or_multiline_linux_paths() {
        assert!(build_launch_arguments("Ubuntu", "mnt/c/repo", "/bin/sh", &[]).is_err());
        assert!(build_launch_arguments("Ubuntu", "/mnt/c/repo\n/tmp", "/bin/sh", &[]).is_err());
        assert!(build_launch_arguments("Ubuntu", "/mnt/c/repo", "bin/sh", &[]).is_err());
        assert!(
            build_launch_arguments(
                "Ubuntu",
                "/mnt/c/repo",
                "/bin/sh",
                &["bad\0argument".to_owned()],
            )
            .is_err()
        );
    }

    #[test]
    fn wsl_profile_identity_binds_domain_launcher_and_shell() {
        let domain = WslExecutionDomain {
            host_os: "windows".to_owned(),
            host_arch: "x86_64".to_owned(),
            distribution: "Ubuntu Dev".to_owned(),
            version: 2,
        };
        let strategy = WslCwdStrategy::MappedWorkspaceOrHomeFallback;
        let first = stable_profile_id(
            &domain,
            r"C:\Windows\System32\wsl.exe",
            "/bin/sh",
            &[],
            strategy,
        );
        let same = stable_profile_id(
            &domain,
            r"C:\Windows\System32\wsl.exe",
            "/bin/sh",
            &[],
            strategy,
        );
        let changed_shell = stable_profile_id(
            &domain,
            r"C:\Windows\System32\wsl.exe",
            "/bin/bash",
            &[],
            strategy,
        );
        let changed_arguments = stable_profile_id(
            &domain,
            r"C:\Windows\System32\wsl.exe",
            "/bin/sh",
            &["-i".to_owned()],
            strategy,
        );
        assert_eq!(first, same);
        assert_ne!(first, changed_shell);
        assert_ne!(first, changed_arguments);
    }

    #[test]
    fn parses_exact_single_absolute_linux_path_without_trimming_spaces() {
        assert_eq!(
            parse_single_linux_path(b"/mnt/c/work space/trailing \r\n", "mapped path").unwrap(),
            "/mnt/c/work space/trailing "
        );
        assert!(parse_single_linux_path(b"relative\n", "mapped path").is_err());
        assert!(parse_single_linux_path(b"/one\n/two\n", "mapped path").is_err());
    }

    #[test]
    fn launch_validation_rejects_tampered_host_domain_but_ignores_ux_label() {
        let domain = WslExecutionDomain {
            host_os: "windows".to_owned(),
            host_arch: "x86_64".to_owned(),
            distribution: "Ubuntu Dev".to_owned(),
            version: 2,
        };
        let strategy = WslCwdStrategy::MappedWorkspaceOrHomeFallback;
        let profile_id = stable_profile_id(
            &domain,
            r"C:\Windows\System32\wsl.exe",
            "/bin/sh",
            &[],
            strategy,
        );
        let expected = WslTerminalProfile {
            profile_id,
            display_name: "WSL: Ubuntu Dev / /bin/sh".to_owned(),
            execution_domain: domain,
            launcher_executable: r"C:\Windows\System32\wsl.exe".to_owned(),
            shell_executable: "/bin/sh".to_owned(),
            shell_arguments: Vec::new(),
            cwd_strategy: strategy,
        };

        let mut ux_only = expected.clone();
        ux_only.display_name = "UX label only".to_owned();
        validate_profile_for_launch(&ux_only, &expected).unwrap();

        let mut tampered_os = expected.clone();
        tampered_os.execution_domain.host_os = "linux".to_owned();
        let error = validate_profile_for_launch(&tampered_os, &expected).unwrap_err();
        assert!(error.to_string().contains("execution domain"));

        let mut tampered_arch = expected.clone();
        tampered_arch.execution_domain.host_arch = "aarch64".to_owned();
        let error = validate_profile_for_launch(&tampered_arch, &expected).unwrap_err();
        assert!(error.to_string().contains("execution domain"));
    }
}
