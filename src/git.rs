use sha2::{Digest, Sha256};
use std::error::Error;
use std::ffi::OsStr;
#[cfg(unix)]
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "shell_profiles.rs"]
pub(crate) mod shell_profiles;
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

pub(crate) const GIT_WORKTREE_STATE_FORMAT: &str = "GIT_STATUS_PORCELAIN_V1_Z_SHA256_V1";

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

    let branch = observed_branch_name(&repo)?;
    let head_oid = observed_head_oid(&repo, branch.as_deref())?;
    let detached = branch.is_none();
    let status = observed_status_bytes(&repo)?;
    let dirty = !status.is_empty();
    let worktree_state_sha256 = sha256_hex(&status);

    Ok(WorktreeStateObservation {
        head_oid,
        branch,
        detached,
        dirty,
        worktree_state_sha256,
    })
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

#[derive(Debug, Clone)]
pub struct Repo {
    root: PathBuf,
    common_dir: PathBuf,
}

impl Repo {
    pub fn open(path: &Path) -> Result<Self> {
        let root = run_git_text(path, ["rev-parse", "--show-toplevel"])?;
        let root = PathBuf::from(strip_git_line_ending(&root)).canonicalize()?;
        let common_dir = run_git_text(
            &root,
            ["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?;
        let common_dir = PathBuf::from(strip_git_line_ending(&common_dir)).canonicalize()?;
        Ok(Self { root, common_dir })
    }

    pub fn root(&self) -> &Path {
        &self.root
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
        let status = run_git_bytes(
            &self.root,
            ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?;
        if !status.is_empty() {
            return Err("primary checkout is dirty; Winds refuses to provision a candidate".into());
        }
        Ok(())
    }

    pub fn resolve_commit(&self, value: &str) -> Result<String> {
        let spec = format!("{value}^{{commit}}");
        Ok(run_git_text(
            &self.root,
            ["rev-parse", "--verify", "--end-of-options", spec.as_str()],
        )?
        .trim()
        .to_owned())
    }

    pub fn tree_oid(&self, commit_oid: &str) -> Result<String> {
        let spec = format!("{commit_oid}^{{tree}}");
        Ok(run_git_text(
            &self.root,
            ["rev-parse", "--verify", "--end-of-options", spec.as_str()],
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

        run_git_os(
            &self.root,
            [
                OsStr::new("worktree"),
                OsStr::new("add"),
                OsStr::new("--detach"),
                path.as_os_str(),
                OsStr::new(commit_oid),
            ],
        )?;

        run_git_os(
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
        Ok(run_git_text(path, ["rev-parse", "HEAD"])?.trim().to_owned())
    }

    pub fn worktree_is_clean(&self, path: &Path) -> Result<bool> {
        Ok(run_git_bytes(
            path,
            ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )?
        .is_empty())
    }

    pub fn worktree_paths(&self) -> Result<Vec<PathBuf>> {
        let output = run_git_bytes(&self.root, ["worktree", "list", "--porcelain", "-z"])?;
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
        let existing = git_command(&self.root)
            .args([
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                spec.as_str(),
            ])
            .output()?;

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

        run_git_text(&self.root, ["branch", branch, commit_oid])?;
        Ok(())
    }
}

fn observed_branch_name(repo: &Repo) -> Result<Option<String>> {
    let output = git_command(repo.root())
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .output()?;
    match output.status.code() {
        Some(0) => {
            let branch = String::from_utf8(output.stdout)?;
            let branch = strip_git_line_ending(&branch);
            if branch.is_empty() {
                return Err("Git returned an empty branch name".into());
            }
            Ok(Some(branch.to_owned()))
        }
        Some(1) => Ok(None),
        _ => Err(format!(
            "failed to determine workspace branch state: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into()),
    }
}

fn observed_head_oid(repo: &Repo, branch: Option<&str>) -> Result<Option<String>> {
    let output = git_command(repo.root())
        .args(["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])
        .output()?;
    if output.status.success() {
        let head = String::from_utf8(output.stdout)?;
        let head = strip_git_line_ending(&head);
        if head.is_empty() {
            return Err("Git returned an empty HEAD object id".into());
        }
        return Ok(Some(head.to_owned()));
    }
    if output.status.code() != Some(1) {
        return Err(format!(
            "failed to resolve workspace HEAD: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    let Some(branch) = branch else {
        return Err("detached workspace HEAD does not resolve to a commit".into());
    };
    let full_ref = format!("refs/heads/{branch}");
    let ref_status = git_command(repo.root())
        .args(["show-ref", "--verify", "--quiet", full_ref.as_str()])
        .status()?;
    match ref_status.code() {
        Some(1) => Ok(None),
        Some(0) => Err(format!(
            "workspace HEAD branch exists but does not resolve to a commit: {full_ref}"
        )
        .into()),
        _ => Err(format!("failed to verify workspace HEAD branch: {full_ref}").into()),
    }
}

fn observed_status_bytes(repo: &Repo) -> Result<Vec<u8>> {
    let output = git_command(repo.root())
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
        ])
        .output()?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(format!(
        "failed to inspect workspace dirty state: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
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

fn run_git_bytes<I, S>(cwd: &Path, args: I) -> Result<Vec<u8>>
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

fn run_git_text<I, S>(cwd: &Path, args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(String::from_utf8(run_git_bytes(cwd, args)?)?)
}

fn run_git_os<I, S>(cwd: &Path, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_git_bytes(cwd, args).map(|_| ())
}

fn strip_git_line_ending(value: &str) -> &str {
    let value = value.strip_suffix('\n').unwrap_or(value);
    value.strip_suffix('\r').unwrap_or(value)
}

#[cfg(test)]
mod git_observation_tests {
    use super::{GIT_WORKTREE_STATE_FORMAT, sha256_hex};

    #[test]
    fn worktree_state_digest_is_sha256_over_exact_porcelain_bytes() {
        assert_eq!(GIT_WORKTREE_STATE_FORMAT, "GIT_STATUS_PORCELAIN_V1_Z_SHA256_V1");
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_ne!(sha256_hex(b" M tracked.txt\0"), sha256_hex(b""));
    }
}
