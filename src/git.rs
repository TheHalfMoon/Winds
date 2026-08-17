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
#[cfg(test)]
#[path = "t059_negative_tests.rs"]
mod t059_negative_tests;
#[cfg(all(test, unix))]
#[path = "t060_fault_tests.rs"]
mod t060_fault_tests;
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

fn observed_status_bytes(repo: &Repo) -> Result<Vec<u8>> {
    let output = git_command(repo.root())
        .env("GIT_OPTIONAL_LOCKS", "0")
        .args([
            "status",
            "--porcelain=v2",
            "--branch",
            "--no-ahead-behind",
            "-z",
            "--untracked-files=all",
            "--ignore-submodules=none",
            "--no-renames",
        ])
        .output()?;
    if output.status.success() {
        return Ok(output.stdout);
    }
    Err(format!(
        "failed to inspect workspace Git state: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )
    .into())
}

fn parse_worktree_status(bytes: &[u8]) -> Result<WorktreeStateObservation> {
    let mut head_oid: Option<Option<String>> = None;
    let mut branch: Option<Option<String>> = None;
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
        if field.starts_with(b"# ") {
            continue;
        }

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
    use super::{GIT_WORKTREE_STATE_FORMAT, parse_worktree_status};

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
}
