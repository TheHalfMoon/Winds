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

    pub fn remove_worktree(&self, path: &Path) -> Result<()> {
        let registered = run_git_bytes(&self.root, ["worktree", "list", "--porcelain", "-z"])?;
        let expected = path.canonicalize()?;
        let owned = registered.split(|byte| *byte == 0).any(|entry| {
            let Some(value) = entry.strip_prefix(b"worktree ") else {
                return false;
            };
            let candidate = PathBuf::from(OsString::from_vec(value.to_vec()));
            candidate
                .canonicalize()
                .is_ok_and(|canonical| canonical == expected)
        });
        if !owned {
            return Err(format!(
                "refusing to remove unregistered candidate worktree: {}",
                path.display()
            )
            .into());
        }

        let status = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_ALTERNATE_OBJECT_DIRECTORIES")
            .env_remove("GIT_SHALLOW_FILE")
            .env_remove("GIT_NAMESPACE")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_PREFIX")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .arg("-C")
            .arg(&self.root)
            .args(["worktree", "remove", "--"])
            .arg(&expected)
            .status()?;
        if !status.success() {
            return Err(format!(
                "system Git refused candidate worktree removal: {}",
                expected.display()
            )
            .into());
        }
        Ok(())
    }
}

pub(crate) fn git_command(cwd: &Path) -> Command {
    let mut command = Command::new("git");
    for variable in GIT_CONTEXT_ENV_VARS {
        command.env_remove(variable);
    }
    command
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .arg("-c")
        .arg("core.hooksPath=/dev/null")
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
    if !output.status.success() {
        return Err(format!(
            "system Git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(output.stdout)
}

pub(crate) fn run_git_text<I, S>(cwd: &Path, args: I) -> Result<String>
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
    let output = git_command(cwd).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "system Git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(())
}

pub(crate) fn strip_git_line_ending(value: &str) -> &str {
    value.strip_suffix("\r\n").or_else(|| value.strip_suffix('\n')).unwrap_or(value)
}
