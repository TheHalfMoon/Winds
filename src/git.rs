use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct Repo {
    root: PathBuf,
    common_dir: PathBuf,
}

impl Repo {
    pub fn open(path: &Path) -> Result<Self> {
        let root = run_git_text(path, ["rev-parse", "--show-toplevel"])?;
        let root = PathBuf::from(root.trim()).canonicalize()?;
        let common_dir = run_git_text(
            &root,
            ["rev-parse", "--path-format=absolute", "--git-common-dir"],
        )?;
        let common_dir = PathBuf::from(common_dir.trim()).canonicalize()?;
        Ok(Self { root, common_dir })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn acquire_mutation_lock(&self) -> Result<File> {
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.common_dir.join("winds.lock"))?;
        lock.lock()?;
        Ok(lock)
    }

    pub fn require_clean_primary(&self) -> Result<()> {
        let status = run_git_bytes(&self.root, ["status", "--porcelain=v1", "-z"])?;
        if !status.is_empty() {
            return Err("primary checkout is dirty; Winds refuses to provision a candidate".into());
        }
        Ok(())
    }

    pub fn resolve_commit(&self, value: &str) -> Result<String> {
        let spec = format!("{value}^{{commit}}");
        Ok(
            run_git_text(&self.root, ["rev-parse", "--verify", spec.as_str()])?
                .trim()
                .to_owned(),
        )
    }

    pub fn tree_oid(&self, commit_oid: &str) -> Result<String> {
        let spec = format!("{commit_oid}^{{tree}}");
        Ok(
            run_git_text(&self.root, ["rev-parse", "--verify", spec.as_str()])?
                .trim()
                .to_owned(),
        )
    }

    pub fn add_locked_worktree(
        &self,
        path: &Path,
        commit_oid: &str,
        branch: &str,
        reason: &str,
    ) -> Result<()> {
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
                OsStr::new("-b"),
                OsStr::new(branch),
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
        Ok(run_git_bytes(path, ["status", "--porcelain=v1", "-z"])?.is_empty())
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
        let existing = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["show-ref", "--verify", "--hash", full_ref.as_str()])
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

        run_git_text(&self.root, ["branch", branch, commit_oid])?;
        Ok(())
    }
}

fn run_git_bytes<I, S>(cwd: &Path, args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git").arg("-C").arg(cwd).args(args).output()?;
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
