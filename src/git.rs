use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "workspace.rs"]
pub(crate) mod workspace;
#[path = "workspace_clone.rs"]
pub(crate) mod workspace_clone;
#[path = "workspace_inventory.rs"]
pub(crate) mod workspace_inventory;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

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
                paths.push(PathBuf::from(OsString::from_vec(path.to_vec())));
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
