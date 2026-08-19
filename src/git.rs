use sha2::{Digest, Sha256};
use std::error::Error;
use std::ffi::OsStr;
#[cfg(unix)]
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

#[path = "process_scope.rs"]
mod process_scope;
use process_scope::{OwnedProcess, operation_deadlines, spawn_owned_process};

#[path = "shell_profiles.rs"]
pub(crate) mod shell_profiles;
#[cfg(test)]
#[path = "t059_negative_tests.rs"]
mod t059_negative_tests;
#[cfg(all(test, unix))]
#[path = "t060_fault_tests.rs"]
mod t060_fault_tests;
#[cfg(all(test, any(unix, windows)))]
#[path = "t063_soak_tests.rs"]
mod t063_soak_tests;
#[cfg(any(unix, windows))]
#[allow(
    dead_code,
    reason = "Spec 003 T050/T051 backend API; persistence/CLI callers land in T053/T057"
)]
#[path = "terminal.rs"]
pub(crate) mod terminal;
#[path = "workspace.rs"]
pub(crate) mod workspace;
#[path = "workspace_clone.rs"]
pub(crate) mod workspace_clone;
#[path = "workspace_inventory.rs"]
pub(crate) mod workspace_inventory;
#[path = "wsl.rs"]
pub(crate) mod wsl;
#[cfg(any(windows, test))]
#[allow(
    dead_code,
    reason = "Spec 003 T052 backend API; persistence/CLI callers land in T053/T057"
)]
#[path = "wsl_launch.rs"]
pub(crate) mod wsl_launch;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) const GIT_WORKTREE_STATE_FORMAT: &str =
    "GIT_STATUS_PORCELAIN_V2_BRANCH_Z_NO_RENAMES_SHA256_V1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorktreeStateObservation {
    pub head_oid: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub dirty: bool,
    pub worktree_state_sha256: String,
}

pub(crate) fn observe_worktree_state(
    expected_root: &Path,
    expected_common_dir: &Path,
) -> Result<WorktreeStateObservation> {
    let repo = Repo::open(expected_root)?;
    if repo.root != expected_root {
        return Err(format!(
            "registered workspace root no longer matches observed Git root: expected {}, observed {}",
            expected_root.display(),
            repo.root.display()
        )
        .into());
    }
    if repo.common_dir != expected_common_dir {
        return Err(format!(
            "registered Git common directory no longer matches observed Git identity: expected {}, observed {}",
            expected_common_dir.display(),
            repo.common_dir.display()
        )
        .into());
    }

    parse_worktree_status(&observed_status_bytes(&repo)?)
}

const GIT_CONTEXT_ENV_VARS: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_SHALLOW_FILE",
    "GIT_NAMESPACE",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
    "GIT_PREFIX",
];
const OBSERVATION_GIT_OUTPUT_LIMIT: usize = 1024 * 1024;
const OBSERVATION_GIT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(super) struct BoundedGitOutput {
    pub(super) status: ExitStatus,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Repo {
    root: PathBuf,
    common_dir: PathBuf,
}

impl Repo {
    pub fn open(path: &Path) -> Result<Self> {
        let root = run_read_only_git_text(
            path,
            ["rev-parse", "--show-toplevel"],
            "workspace Git root discovery",
        )?;
        let root = PathBuf::from(strip_git_line_ending(&root)).canonicalize()?;
        let common_dir = run_read_only_git_text(
            &root,
            ["rev-parse", "--path-format=absolute", "--git-common-dir"],
            "workspace Git common-directory discovery",
        )?;
        let common_dir = PathBuf::from(strip_git_line_ending(&common_dir)).canonicalize()?;
        Ok(Self { root, common_dir })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn common_dir(&self) -> &Path {
        &self.common_dir
    }

    pub fn require_external_state_path(&self, path: &Path) -> Result<()> {
        if path.starts_with(&self.root) || path.starts_with(&self.common_dir) {
            return Err(
                "Winds state must live outside the source checkout and Git common directory".into(),
            );
        }
        Ok(())
    }

    pub fn acquire_mutation_lock(&self) -> Result<File> {
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.common_dir.join("winds.lock"))?;
        lock.lock()?;
        Ok(lock)
    }

    pub fn require_clean_primary(&self) -> Result<()> {
        let status = run_read_only_git_bytes(
            &self.root,
            ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            "primary checkout cleanliness inspection",
        )?;
        if !status.is_empty() {
            return Err("primary checkout is dirty; Winds refuses to provision a candidate".into());
        }
        Ok(())
    }

    pub fn resolve_commit(&self, value: &str) -> Result<String> {
        let spec = format!("{value}^{{commit}}");
        Ok(run_read_only_git_text(
            &self.root,
            ["rev-parse", "--verify", "--end-of-options", spec.as_str()],
            "commit resolution",
        )?
        .trim()
        .to_owned())
    }

    pub fn tree_oid(&self, commit_oid: &str) -> Result<String> {
        let spec = format!("{commit_oid}^{{tree}}");
        Ok(run_read_only_git_text(
            &self.root,
            ["rev-parse", "--verify", "--end-of-options", spec.as_str()],
            "tree resolution",
        )?
        .trim()
        .to_owned())
    }

    pub fn add_locked_worktree(&self, path: &Path, commit_oid: &str, reason: &str) -> Result<()> {
        if path.exists() {
            return Err(
                format!("candidate worktree path already exists: {}", path.display()).into(),
            );
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        run_mutating_git_os(
            &self.root,
            [
                OsStr::new("worktree"),
                OsStr::new("add"),
                OsStr::new("--detach"),
                path.as_os_str(),
                OsStr::new(commit_oid),
            ],
        )?;

        run_mutating_git_os(
            &self.root,
            [
                OsStr::new("worktree"),
                OsStr::new("lock"),
                OsStr::new("--reason"),
                OsStr::new(reason),
                path.as_os_str(),
            ],
        )?;
        Ok(())
    }

    pub fn worktree_head(&self, path: &Path) -> Result<String> {
        Ok(
            run_read_only_git_text(path, ["rev-parse", "HEAD"], "worktree HEAD inspection")?
                .trim()
                .to_owned(),
        )
    }

    pub fn worktree_is_clean(&self, path: &Path) -> Result<bool> {
        Ok(run_read_only_git_bytes(
            path,
            ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            "candidate worktree cleanliness inspection",
        )?
        .is_empty())
    }

    pub fn worktree_paths(&self) -> Result<Vec<PathBuf>> {
        let output = run_read_only_git_bytes(
            &self.root,
            ["worktree", "list", "--porcelain", "-z"],
            "worktree inventory inspection",
        )?;
        let mut paths = Vec::new();
        for field in output.split(|byte| *byte == 0) {
            if let Some(path) = field.strip_prefix(b"worktree ") {
                paths.push(git_path_from_bytes(path)?);
            }
        }
        Ok(paths)
    }

    pub fn create_selected_branch(&self, branch: &str, commit_oid: &str) -> Result<()> {
        let full_ref = format!("refs/heads/{branch}");
        let spec = format!("{full_ref}^{{commit}}");
        let existing = run_read_only_git_output(
            &self.root,
            [
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                spec.as_str(),
            ],
            "selected branch existence inspection",
        )?;

        if existing.status.success() {
            let current = String::from_utf8(existing.stdout)?.trim().to_owned();
            if current == commit_oid {
                return Ok(());
            }
            return Err(
                format!("selected branch already exists at different commit: {current}").into(),
            );
        }
        if existing.status.code() != Some(1) {
            return Err(format!(
                "failed checking selected branch: {}",
                String::from_utf8_lossy(&existing.stderr).trim()
            )
            .into());
        }

        run_mutating_git_text(&self.root, ["branch", branch, commit_oid])?;
        Ok(())
    }
}

fn observed_status_bytes(repo: &Repo) -> Result<Vec<u8>> {
    let mut command = git_command(repo.root());
    command.env("GIT_OPTIONAL_LOCKS", "0").args([
        "status",
        "--porcelain=v2",
        "--branch",
        "--no-ahead-behind",
        "-z",
        "--untracked-files=all",
        "--ignore-submodules=none",
        "--no-renames",
    ]);
    run_bounded_read_only_git(command, "workspace Git observation")
}

pub(super) fn run_bounded_read_only_git(command: Command, label: &str) -> Result<Vec<u8>> {
    let output = run_bounded_read_only_git_output(command, label)?;
    if !output.status.success() {
        return Err(format!(
            "{label} failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

fn run_bounded_read_only_git_output(mut command: Command, label: &str) -> Result<BoundedGitOutput> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let started = Instant::now();
    let (command_deadline, cleanup_deadline) =
        operation_deadlines(started, OBSERVATION_GIT_TIMEOUT);
    let mut child = spawn_owned_process(&mut command, label)?;

    let stdout = match child.take_stdout() {
        Some(stdout) => stdout,
        None => {
            let cleanup = child.terminate_and_prove(cleanup_deadline, label);
            return Err(format!(
                "{label} could not capture Git stdout; owned cleanup {}",
                cleanup
                    .map(|()| "succeeded".to_owned())
                    .unwrap_or_else(|error| format!("was not proven: {error}"))
            )
            .into());
        }
    };
    let stderr = match child.take_stderr() {
        Some(stderr) => stderr,
        None => {
            let cleanup = child.terminate_and_prove(cleanup_deadline, label);
            return Err(format!(
                "{label} could not capture Git stderr; owned cleanup {}",
                cleanup
                    .map(|()| "succeeded".to_owned())
                    .unwrap_or_else(|error| format!("was not proven: {error}"))
            )
            .into());
        }
    };
    let stdout_reader = spawn_bounded_reader(stdout);
    let stderr_reader = spawn_bounded_reader(stderr);

    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= command_deadline => {
                return fail_bounded_git_observation(
                    &mut child,
                    &stdout_reader,
                    &stderr_reader,
                    cleanup_deadline,
                    label,
                    format!(
                        "{label} exceeded the bounded execution phase of its {} second safety timeout",
                        OBSERVATION_GIT_TIMEOUT.as_secs()
                    ),
                );
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                return fail_bounded_git_observation(
                    &mut child,
                    &stdout_reader,
                    &stderr_reader,
                    cleanup_deadline,
                    label,
                    format!("{label} failed while waiting for Git: {error}"),
                );
            }
        }
    };

    let stdout = match receive_bounded_reader(&stdout_reader, label, "stdout", command_deadline) {
        Ok(output) => output,
        Err(error) => {
            return fail_bounded_git_observation(
                &mut child,
                &stdout_reader,
                &stderr_reader,
                cleanup_deadline,
                label,
                error.to_string(),
            );
        }
    };
    let stderr = match receive_bounded_reader(&stderr_reader, label, "stderr", command_deadline) {
        Ok(output) => output,
        Err(error) => {
            return fail_bounded_git_observation(
                &mut child,
                &stdout_reader,
                &stderr_reader,
                cleanup_deadline,
                label,
                error.to_string(),
            );
        }
    };

    match child.wait_for_scope_quiescence(command_deadline, label) {
        Ok(true) => {}
        Ok(false) => {
            return fail_bounded_git_observation(
                &mut child,
                &stdout_reader,
                &stderr_reader,
                cleanup_deadline,
                label,
                format!("{label} direct Git child exited while owned descendants remained live"),
            );
        }
        Err(error) => {
            return fail_bounded_git_observation(
                &mut child,
                &stdout_reader,
                &stderr_reader,
                cleanup_deadline,
                label,
                format!("{label} could not prove owned process-scope quiescence: {error}"),
            );
        }
    }

    if stdout.truncated || stderr.truncated {
        return Err(format!(
            "{label} output exceeded the {} byte per-stream safety bound",
            OBSERVATION_GIT_OUTPUT_LIMIT
        )
        .into());
    }
    Ok(BoundedGitOutput {
        status,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
    })
}

fn fail_bounded_git_observation<T>(
    child: &mut OwnedProcess,
    stdout_reader: &Receiver<io::Result<BoundedCapture>>,
    stderr_reader: &Receiver<io::Result<BoundedCapture>>,
    cleanup_deadline: Instant,
    label: &str,
    primary_error: String,
) -> Result<T> {
    let mut cleanup_failures = Vec::new();
    if let Err(error) = child.terminate_and_prove(cleanup_deadline, label) {
        cleanup_failures.push(error.to_string());
    }
    if let Err(error) =
        wait_bounded_reader_shutdown(stdout_reader, label, "stdout", cleanup_deadline)
    {
        cleanup_failures.push(error.to_string());
    }
    if let Err(error) =
        wait_bounded_reader_shutdown(stderr_reader, label, "stderr", cleanup_deadline)
    {
        cleanup_failures.push(error.to_string());
    }

    if cleanup_failures.is_empty() {
        Err(primary_error.into())
    } else {
        Err(format!(
            "{primary_error}; owned subprocess cleanup was not proven: {}",
            cleanup_failures.join("; ")
        )
        .into())
    }
}

struct BoundedCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

fn spawn_bounded_reader<R>(reader: R) -> Receiver<io::Result<BoundedCapture>>
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let _ = sender.send(read_bounded(reader));
    });
    receiver
}

fn receive_bounded_reader(
    receiver: &Receiver<io::Result<BoundedCapture>>,
    label: &str,
    stream: &str,
    deadline: Instant,
) -> Result<BoundedCapture> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    match receiver.recv_timeout(remaining) {
        Ok(result) => {
            result.map_err(|error| format!("{label} failed reading Git {stream}: {error}").into())
        }
        Err(RecvTimeoutError::Timeout) => Err(format!(
            "{label} {stream} reader exceeded the bounded execution phase of the overall {} second safety timeout",
            OBSERVATION_GIT_TIMEOUT.as_secs()
        )
        .into()),
        Err(RecvTimeoutError::Disconnected) => {
            Err(format!("{label} {stream} reader terminated without a result").into())
        }
    }
}

fn wait_bounded_reader_shutdown(
    receiver: &Receiver<io::Result<BoundedCapture>>,
    label: &str,
    stream: &str,
    deadline: Instant,
) -> Result<()> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    match receiver.recv_timeout(remaining) {
        Ok(_) | Err(RecvTimeoutError::Disconnected) => Ok(()),
        Err(RecvTimeoutError::Timeout) => Err(format!(
            "{label} {stream} reader shutdown was not proven inside the bounded cleanup window"
        )
        .into()),
    }
}

fn read_bounded<R: Read>(mut reader: R) -> io::Result<BoundedCapture> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let probe_limit = OBSERVATION_GIT_OUTPUT_LIMIT + 1;
        let remaining = probe_limit.saturating_sub(bytes.len());
        let retained = remaining.min(count);
        bytes.extend_from_slice(&buffer[..retained]);
        if bytes.len() > OBSERVATION_GIT_OUTPUT_LIMIT {
            bytes.truncate(OBSERVATION_GIT_OUTPUT_LIMIT);
            return Ok(BoundedCapture {
                bytes,
                truncated: true,
            });
        }
    }
    Ok(BoundedCapture {
        bytes,
        truncated: false,
    })
}

fn parse_worktree_status(bytes: &[u8]) -> Result<WorktreeStateObservation> {
    let mut head_oid: Option<Option<String>> = None;
    let mut branch: Option<Option<String>> = None;
    let mut upstream_seen = false;
    let mut ahead_behind_seen = false;
    let mut dirty = false;
    let mut worktree_hasher = Sha256::new();

    for field in bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
    {
        if let Some(value) = field.strip_prefix(b"# branch.oid ") {
            if head_oid.is_some() {
                return Err("Git status returned duplicate branch.oid headers".into());
            }
            let value = std::str::from_utf8(value)?;
            head_oid = Some(if value == "(initial)" {
                None
            } else if value.is_empty() {
                return Err("Git status returned an empty branch.oid value".into());
            } else {
                Some(value.to_owned())
            });
            continue;
        }
        if let Some(value) = field.strip_prefix(b"# branch.head ") {
            if branch.is_some() {
                return Err("Git status returned duplicate branch.head headers".into());
            }
            let value = std::str::from_utf8(value)?;
            branch = Some(if value == "(detached)" {
                None
            } else if value.is_empty() {
                return Err("Git status returned an empty branch.head value".into());
            } else {
                Some(value.to_owned())
            });
            continue;
        }
        if let Some(value) = field.strip_prefix(b"# branch.upstream ") {
            if upstream_seen || value.is_empty() {
                return Err("Git status returned invalid branch.upstream headers".into());
            }
            upstream_seen = true;
            continue;
        }
        if let Some(value) = field.strip_prefix(b"# branch.ab ") {
            if ahead_behind_seen || value.is_empty() {
                return Err("Git status returned invalid branch.ab headers".into());
            }
            ahead_behind_seen = true;
            continue;
        }
        if field.starts_with(b"# ") {
            return Err("Git status returned an unrecognized porcelain-v2 branch header".into());
        }

        validate_worktree_record(field)?;
        dirty = true;
        worktree_hasher.update(field);
        worktree_hasher.update([0]);
    }

    let head_oid = head_oid.ok_or("Git status omitted the required branch.oid header")?;
    let branch = branch.ok_or("Git status omitted the required branch.head header")?;
    let detached = branch.is_none();
    if detached && head_oid.is_none() {
        return Err("Git status reported detached HEAD without an exact object id".into());
    }
    let worktree_state_sha256 = hex_digest(worktree_hasher.finalize());

    Ok(WorktreeStateObservation {
        head_oid,
        branch,
        detached,
        dirty,
        worktree_state_sha256,
    })
}

fn validate_worktree_record(field: &[u8]) -> Result<()> {
    if let Some(rest) = field.strip_prefix(b"1 ") {
        if fixed_fields_then_path(rest, 7) {
            return Ok(());
        }
        return Err("Git status returned a malformed ordinary changed-entry record".into());
    }
    if let Some(rest) = field.strip_prefix(b"u ") {
        if fixed_fields_then_path(rest, 9) {
            return Ok(());
        }
        return Err("Git status returned a malformed unmerged-entry record".into());
    }
    if let Some(path) = field.strip_prefix(b"? ") {
        if !path.is_empty() {
            return Ok(());
        }
        return Err("Git status returned an empty untracked path".into());
    }
    if field.starts_with(b"2 ") {
        return Err("Git status returned a rename/copy record despite --no-renames".into());
    }
    Err("Git status returned an unrecognized porcelain-v2 worktree record".into())
}

fn fixed_fields_then_path(mut rest: &[u8], fixed_fields: usize) -> bool {
    for _ in 0..fixed_fields {
        let Some(separator) = rest.iter().position(|byte| *byte == b' ') else {
            return false;
        };
        if separator == 0 {
            return false;
        }
        rest = &rest[separator + 1..];
    }
    !rest.is_empty()
}

fn hex_digest(digest: impl AsRef<[u8]>) -> String {
    digest
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(unix)]
fn git_path_from_bytes(path: &[u8]) -> Result<PathBuf> {
    Ok(PathBuf::from(OsString::from_vec(path.to_vec())))
}

#[cfg(windows)]
fn git_path_from_bytes(path: &[u8]) -> Result<PathBuf> {
    let path = std::str::from_utf8(path)
        .map_err(|error| format!("Git returned a non-UTF-8 Windows worktree path: {error}"))?;
    Ok(PathBuf::from(path))
}

fn git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    for key in GIT_CONTEXT_ENV_VARS {
        command.env_remove(key);
    }
    command
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .arg("-c")
        .arg("core.hooksPath=/dev/null")
        .arg("-c")
        .arg("core.fsmonitor=false")
        .arg("-C")
        .arg(cwd);
    command
}

pub(super) fn run_read_only_git_output<I, S>(
    cwd: &Path,
    args: I,
    label: &str,
) -> Result<BoundedGitOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = git_command(cwd);
    command.env("GIT_OPTIONAL_LOCKS", "0").args(args);
    run_bounded_read_only_git_output(command, label)
}

pub(super) fn run_read_only_git_bytes<I, S>(cwd: &Path, args: I, label: &str) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = run_read_only_git_output(cwd, args, label)?;
    if !output.status.success() {
        return Err(format!(
            "{label} failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

pub(super) fn run_read_only_git_text<I, S>(cwd: &Path, args: I, label: &str) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(String::from_utf8(run_read_only_git_bytes(
        cwd, args, label,
    )?)?)
}

// Mutation-capable Git operations intentionally remain outside the read-only
// observation timeout/containment contract. Read-only callers must use the
// bounded helpers above.
fn run_mutating_git_bytes<I, S>(cwd: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = git_command(cwd).args(args).output()?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(format!(
        "git command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

fn run_mutating_git_text<I, S>(cwd: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(String::from_utf8(run_mutating_git_bytes(cwd, args)?)?)
}

fn run_mutating_git_os<I, S>(cwd: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_mutating_git_bytes(cwd, args).map(|_| ())
}

fn strip_git_line_ending(value: &str) -> &str {
    let value = value.strip_suffix('\n').unwrap_or(value);
    value.strip_suffix('\r').unwrap_or(value)
}

#[cfg(test)]
mod git_observation_tests {
    use super::{
        GIT_WORKTREE_STATE_FORMAT, OBSERVATION_GIT_OUTPUT_LIMIT, parse_worktree_status,
        read_bounded, run_bounded_read_only_git_output,
    };
    use std::io::Cursor;
    use std::process::Command;

    #[test]
    fn clean_attached_status_parses_branch_and_empty_state_digest() {
        let observation =
            parse_worktree_status(b"# branch.oid 0123456789abcdef\0# branch.head main\0").unwrap();
        assert_eq!(observation.head_oid.as_deref(), Some("0123456789abcdef"));
        assert_eq!(observation.branch.as_deref(), Some("main"));
        assert!(!observation.detached);
        assert!(!observation.dirty);
        assert_eq!(
            observation.worktree_state_sha256,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            GIT_WORKTREE_STATE_FORMAT,
            "GIT_STATUS_PORCELAIN_V2_BRANCH_Z_NO_RENAMES_SHA256_V1"
        );
    }

    #[test]
    fn status_digest_excludes_branch_headers_but_includes_exact_worktree_records() {
        let main =
            parse_worktree_status(b"# branch.oid abc\0# branch.head main\0? untracked.txt\0")
                .unwrap();
        let other_branch =
            parse_worktree_status(b"# branch.oid abc\0# branch.head other\0? untracked.txt\0")
                .unwrap();
        let different_state =
            parse_worktree_status(b"# branch.oid abc\0# branch.head main\0? different.txt\0")
                .unwrap();
        assert!(main.dirty);
        assert_eq!(
            main.worktree_state_sha256,
            other_branch.worktree_state_sha256
        );
        assert_ne!(
            main.worktree_state_sha256,
            different_state.worktree_state_sha256
        );
    }

    #[test]
    fn unborn_and_detached_headers_remain_explicit() {
        let unborn =
            parse_worktree_status(b"# branch.oid (initial)\0# branch.head main\0").unwrap();
        assert_eq!(unborn.head_oid, None);
        assert_eq!(unborn.branch.as_deref(), Some("main"));
        assert!(!unborn.detached);

        let detached =
            parse_worktree_status(b"# branch.oid deadbeef\0# branch.head (detached)\0").unwrap();
        assert_eq!(detached.head_oid.as_deref(), Some("deadbeef"));
        assert_eq!(detached.branch, None);
        assert!(detached.detached);
    }

    #[test]
    fn missing_or_inconsistent_branch_headers_fail_closed() {
        assert!(parse_worktree_status(b"# branch.head main\0").is_err());
        assert!(parse_worktree_status(b"# branch.oid abc\0").is_err());
        assert!(
            parse_worktree_status(b"# branch.oid (initial)\0# branch.head (detached)\0").is_err()
        );
    }

    #[test]
    fn malformed_or_unexpected_porcelain_v2_records_fail_closed() {
        let prefix = b"# branch.oid abc\0# branch.head main\0";
        for invalid in [
            b"garbage\0".as_slice(),
            b"? \0".as_slice(),
            b"1 MM N... 100644\0".as_slice(),
            b"2 MM N... 100644 100644 100644 abc def R100 new\0old\0".as_slice(),
            b"# unexpected header\0".as_slice(),
        ] {
            let mut bytes = prefix.to_vec();
            bytes.extend_from_slice(invalid);
            assert!(parse_worktree_status(&bytes).is_err());
        }
    }

    #[test]
    fn bounded_reader_stops_at_the_safety_cap() {
        let input = vec![b'x'; OBSERVATION_GIT_OUTPUT_LIMIT + 17];
        let captured = read_bounded(Cursor::new(input)).unwrap();
        assert_eq!(captured.bytes.len(), OBSERVATION_GIT_OUTPUT_LIMIT);
        assert!(captured.truncated);
    }

    #[test]
    fn bounded_read_only_runner_preserves_expected_nonzero_status() {
        let command = if cfg!(windows) {
            let mut command = Command::new("cmd.exe");
            command.args(["/d", "/s", "/c", "exit 7"]);
            command
        } else {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", "exit 7"]);
            command
        };
        let output =
            run_bounded_read_only_git_output(command, "bounded nonzero-status fixture").unwrap();
        assert_eq!(output.status.code(), Some(7));
        assert!(output.stdout.is_empty());
    }
}
