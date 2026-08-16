use super::Result;
use super::terminal::{TerminalSession, TerminalSize};
use super::wsl::WslDistribution;
#[cfg(windows)]
use super::wsl::discover_wsl_distributions;
#[cfg(windows)]
use super::{GIT_CONTEXT_ENV_VARS, Repo, run_git_text, strip_git_line_ending};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;

const WSL_SHELL_EXECUTABLE: &str = "/bin/sh";

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
    let windows_head = run_git_text(repo.root(), ["rev-parse", "--verify", "HEAD^{commit}"])?;
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
    use super::wsl::decode_wsl_text;
    use std::ffi::c_void;
    use std::io::{Read, Result as IoResult};
    use std::os::windows::io::AsRawHandle;
    use std::process::{ChildStderr, ChildStdout, Command, Stdio};
    use std::ptr;
    use std::thread;
    use std::time::{Duration, Instant};

    const CAP: usize = 256 * 1024;
    const TIMEOUT: Duration = Duration::from_secs(30);
    const ERROR_BROKEN_PIPE: i32 = 109;
    const ERROR_NO_DATA: i32 = 232;
    const ERROR_PIPE_NOT_CONNECTED: i32 = 233;

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

    fn read_available<R: Read + AsRawHandle>(
        reader: &mut R,
        captured: &mut Vec<u8>,
        truncated: &mut bool,
    ) -> IoResult<bool> {
        let mut available = 0_u32;
        // SAFETY: `reader` owns a valid pipe handle for this call. No output buffer is
        // supplied; PeekNamedPipe only reports the number of bytes immediately readable.
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
        stdout_truncated: &mut bool,
        stderr_truncated: &mut bool,
    ) -> IoResult<bool> {
        let stdout_progress = read_available(stdout, stdout_bytes, stdout_truncated)?;
        let stderr_progress = read_available(stderr, stderr_bytes, stderr_truncated)?;
        Ok(stdout_progress || stderr_progress)
    }

    fn diagnostic_text(bytes: &[u8]) -> String {
        let decoded =
            decode_wsl_text(bytes).unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned());
        let trimmed = decoded.trim();
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

    let mut process = Command::new(launcher);
    for key in GIT_CONTEXT_ENV_VARS {
        process.env_remove(key);
    }
    process.arg("--distribution").arg(distribution);
    if let Some(cwd) = cwd {
        process.arg("--cd").arg(cwd);
    }
    process.arg("--exec").arg(command).args(command_args);
    let mut child = process
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to execute selected WSL distribution: {error}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or("failed to capture WSL command stdout")?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or("failed to capture WSL command stderr")?;
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;

    let started = Instant::now();
    let status = loop {
        let progressed = drain_pair(
            &mut stdout,
            &mut stderr,
            &mut stdout_bytes,
            &mut stderr_bytes,
            &mut stdout_truncated,
            &mut stderr_truncated,
        )?;
        if let Some(status) = child.try_wait()? {
            while drain_pair(
                &mut stdout,
                &mut stderr,
                &mut stdout_bytes,
                &mut stderr_bytes,
                &mut stdout_truncated,
                &mut stderr_truncated,
            )? {}
            break status;
        }
        if started.elapsed() >= TIMEOUT {
            let cleanup = match child.kill() {
                Ok(()) => match child.wait() {
                    Ok(_) => "Windows WSL launcher process terminated".to_owned(),
                    Err(error) => format!(
                        "Windows WSL launcher termination wait could not be proven: {error}"
                    ),
                },
                Err(kill_error) => match child.try_wait() {
                    Ok(Some(_)) => "Windows WSL launcher had already exited".to_owned(),
                    Ok(None) => format!(
                        "Windows WSL launcher termination could not be proven: {kill_error}"
                    ),
                    Err(wait_error) => format!(
                        "Windows WSL launcher termination could not be proven: {kill_error}; status check failed: {wait_error}"
                    ),
                },
            };
            return Err(format!(
                "selected WSL command exceeded the 30 second safety timeout; {cleanup}"
            )
            .into());
        }
        if !progressed {
            thread::sleep(Duration::from_millis(10));
        }
    };

    let stderr_diagnostic = diagnostic_text(&stderr_bytes);
    let suffix = truncation_suffix(stdout_truncated, stderr_truncated);
    if !status.success() {
        return Err(format!(
            "selected WSL command failed with status {status}: {stderr_diagnostic}{suffix}"
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
            "selected WSL command exceeded the 256 KiB per-stream safety bound{diagnostic}"
        )
        .into());
    }
    Ok(stdout_bytes)
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
