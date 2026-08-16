use super::Result;
use super::terminal::{TerminalSession, TerminalSize};
use super::wsl::WslDistribution;
#[cfg(windows)]
use super::wsl::discover_wsl_distributions;
#[cfg(windows)]
use super::{Repo, run_git_text, strip_git_line_ending};
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WslTerminalProfile {
    pub profile_id: String,
    pub display_name: String,
    pub execution_domain: WslExecutionDomain,
    pub launcher_executable: String,
    pub shell_executable: String,
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
    if plan.profile.shell_executable != WSL_SHELL_EXECUTABLE {
        return Err("WSL terminal profile shell executable is unsupported or stale".into());
    }
    let expected_profile = build_profile(launcher_text, &distribution);
    if expected_profile.profile_id != plan.profile.profile_id {
        return Err("WSL terminal profile identity does not match its launch data".into());
    }

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
    let arguments = build_launch_arguments(&distribution.name, linux_cwd, WSL_SHELL_EXECUTABLE)?;
    let mut session = TerminalSession::start_exact_launch(
        &plan.profile.profile_id,
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
        let repo = Repo::open(Path::new(windows_workspace_root))?;
        match attest_workspace(&launcher, &distribution, linux_workspace_root, &repo) {
            Ok(attestation)
                if attestation.linux_workspace_root == *linux_workspace_root
                    && attestation.linux_git_common_dir == *linux_git_common_dir
                    && attestation.git_head_oid == *git_head_oid => {}
            Ok(_) => {
                let _ = session.terminate();
                return Err(
                    "WSL mapped workspace identity changed after terminal launch; session terminated"
                        .into(),
                );
            }
            Err(error) => {
                let _ = session.terminate();
                return Err(format!(
                    "WSL mapped workspace could not be revalidated after terminal launch; session terminated: {error}"
                )
                .into());
            }
        }
    }

    Ok(WslLaunchedTerminal {
        session,
        profile: plan.profile.clone(),
        cwd_resolution: plan.cwd_resolution.clone(),
    })
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
    let profile_id = stable_profile_id(&execution_domain, launcher, WSL_SHELL_EXECUTABLE);
    WslTerminalProfile {
        profile_id,
        display_name: format!("WSL: {} / {}", distribution.name, WSL_SHELL_EXECUTABLE),
        execution_domain,
        launcher_executable: launcher.to_owned(),
        shell_executable: WSL_SHELL_EXECUTABLE.to_owned(),
    }
}

fn stable_profile_id(domain: &WslExecutionDomain, launcher: &str, shell: &str) -> String {
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
    let hex: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("wsl-terminal-profile-{hex}")
}

fn build_launch_arguments(distribution: &str, linux_cwd: &str, shell: &str) -> Result<Vec<String>> {
    if distribution.is_empty() {
        return Err("WSL distribution identity cannot be empty".into());
    }
    require_absolute_linux_path(linux_cwd, "WSL terminal cwd")?;
    require_absolute_linux_path(shell, "WSL shell executable")?;
    Ok(vec![
        "--distribution".to_owned(),
        distribution.to_owned(),
        "--cd".to_owned(),
        linux_cwd.to_owned(),
        "--exec".to_owned(),
        shell.to_owned(),
    ])
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
    use std::ffi::OsString;

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
    use std::io::{Read, Result as IoResult};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    const CAP: usize = 256 * 1024;
    const TIMEOUT: Duration = Duration::from_secs(30);

    fn read_capped<R: Read>(mut reader: R) -> IoResult<(Vec<u8>, bool)> {
        let mut bytes = Vec::new();
        let mut truncated = false;
        let mut buffer = [0_u8; 8192];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                return Ok((bytes, truncated));
            }
            let remaining = CAP.saturating_sub(bytes.len());
            let keep = remaining.min(count);
            bytes.extend_from_slice(&buffer[..keep]);
            if keep < count {
                truncated = true;
            }
        }
    }

    let mut process = Command::new(launcher);
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
    let stdout = child
        .stdout
        .take()
        .ok_or("failed to capture WSL command stdout")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("failed to capture WSL command stderr")?;
    let (stdout_tx, stdout_rx) = mpsc::sync_channel(1);
    let (stderr_tx, stderr_rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = stdout_tx.send(read_capped(stdout));
    });
    thread::spawn(move || {
        let _ = stderr_tx.send(read_capped(stderr));
    });

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return Err("selected WSL command exceeded the 30 second safety timeout".into());
        }
        thread::sleep(Duration::from_millis(25));
    };

    let (stdout, stdout_truncated) = stdout_rx
        .recv_timeout(Duration::from_secs(1))
        .map_err(|_| "WSL command stdout did not close")??;
    let (stderr, stderr_truncated) = stderr_rx
        .recv_timeout(Duration::from_secs(1))
        .map_err(|_| "WSL command stderr did not close")??;
    if stdout_truncated || stderr_truncated {
        return Err("selected WSL command exceeded the 256 KiB per-stream safety bound".into());
    }
    if !status.success() {
        let stderr = decode_wsl_text(&stderr)
            .unwrap_or_else(|_| String::from_utf8_lossy(&stderr).into_owned());
        return Err(format!(
            "selected WSL command failed with status {status}: {}",
            stderr.trim()
        )
        .into());
    }
    Ok(stdout)
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
        WslExecutionDomain, build_launch_arguments, parse_single_linux_path, stable_profile_id,
    };

    #[test]
    fn launch_arguments_bind_distribution_cwd_and_exact_shell_without_shell_parsing() {
        let args =
            build_launch_arguments("Ubuntu Dev", "/mnt/c/work space/repo", "/bin/sh").unwrap();
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
        assert!(build_launch_arguments("Ubuntu", "mnt/c/repo", "/bin/sh").is_err());
        assert!(build_launch_arguments("Ubuntu", "/mnt/c/repo\n/tmp", "/bin/sh").is_err());
        assert!(build_launch_arguments("Ubuntu", "/mnt/c/repo", "bin/sh").is_err());
    }

    #[test]
    fn wsl_profile_identity_binds_domain_launcher_and_shell() {
        let domain = WslExecutionDomain {
            host_os: "windows".to_owned(),
            host_arch: "x86_64".to_owned(),
            distribution: "Ubuntu Dev".to_owned(),
            version: 2,
        };
        let first = stable_profile_id(&domain, r"C:\Windows\System32\wsl.exe", "/bin/sh");
        let same = stable_profile_id(&domain, r"C:\Windows\System32\wsl.exe", "/bin/sh");
        let changed = stable_profile_id(&domain, r"C:\Windows\System32\wsl.exe", "/bin/bash");
        assert_eq!(first, same);
        assert_ne!(first, changed);
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
}
