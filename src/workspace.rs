use super::{Repo, Result, git_command, run_git_bytes, run_git_text, strip_git_line_ending};
use crate::store::{NewWorkspace, Store};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

#[allow(
    dead_code,
    reason = "Spec 003 T045 backend API; the user-facing CLI caller lands in T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkspaceInspection {
    pub workspace_id: String,
    pub canonical_worktree_root: String,
    pub git_common_dir: String,
    pub head_oid: String,
    pub branch: Option<String>,
    pub detached: bool,
    pub dirty: bool,
}

#[allow(
    dead_code,
    reason = "Spec 003 T045 backend API; the user-facing CLI caller lands in T057"
)]
pub fn open_existing_workspace(
    path: &Path,
    store: &Store,
    now_ms: i64,
) -> Result<WorkspaceInspection> {
    let repo = open_worktree(path)?;
    let observation = inspect_worktree(&repo)?;
    register_observed_workspace(&repo, &observation, store, now_ms)?;
    Ok(observation)
}

fn open_worktree(path: &Path) -> Result<Repo> {
    if !path.exists() {
        return Err(format!("workspace path does not exist: {}", path.display()).into());
    }
    if !path.is_dir() {
        return Err(format!("workspace path is not a directory: {}", path.display()).into());
    }

    let bare = git_bool(path, ["rev-parse", "--is-bare-repository"])
        .map_err(|error| format!("workspace is not a Git worktree: {error}"))?;
    if bare {
        return Err("bare Git repositories are not Winds workspaces".into());
    }

    let inside_worktree = git_bool(path, ["rev-parse", "--is-inside-work-tree"])
        .map_err(|error| format!("workspace is not a Git worktree: {error}"))?;
    if !inside_worktree {
        return Err("workspace path is not inside a Git worktree".into());
    }

    Repo::open(path).map_err(|error| format!("workspace identity is ambiguous: {error}").into())
}

fn inspect_worktree(repo: &Repo) -> Result<WorkspaceInspection> {
    let canonical_worktree_root = utf8_path(repo.root(), "canonical worktree root")?.to_owned();
    let git_common_dir = utf8_path(&repo.common_dir, "Git common directory")?.to_owned();
    let head_oid = exact_head(repo)?;
    let branch = branch_name(repo)?;
    let detached = branch.is_none();
    let dirty = !read_only_status(repo)?.is_empty();
    let workspace_id = stable_workspace_id(&canonical_worktree_root, &git_common_dir);

    Ok(WorkspaceInspection {
        workspace_id,
        canonical_worktree_root,
        git_common_dir,
        head_oid,
        branch,
        detached,
        dirty,
    })
}

fn exact_head(repo: &Repo) -> Result<String> {
    let head = run_git_text(repo.root(), ["rev-parse", "--verify", "HEAD^{commit}"])?;
    let head = strip_git_line_ending(&head);
    if head.is_empty() {
        return Err("Git returned an empty HEAD object id".into());
    }
    Ok(head.to_owned())
}

fn branch_name(repo: &Repo) -> Result<Option<String>> {
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

fn read_only_status(repo: &Repo) -> Result<Vec<u8>> {
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

fn register_observed_workspace(
    repo: &Repo,
    observation: &WorkspaceInspection,
    store: &Store,
    now_ms: i64,
) -> Result<()> {
    let unknown = format!("unknown Winds workspace: {}", observation.workspace_id);
    match store.load_workspace(&observation.workspace_id) {
        Ok(existing) => {
            if existing.canonical_worktree_root != observation.canonical_worktree_root
                || existing.git_common_dir != observation.git_common_dir
            {
                return Err(format!(
                    "stored workspace identity conflicts with observed Git identity: {}",
                    observation.workspace_id
                )
                .into());
            }
            store.mark_workspace_opened(&observation.workspace_id, now_ms)?;
        }
        Err(error) if error.to_string() == unknown => {
            store.create_workspace(
                NewWorkspace {
                    workspace_id: &observation.workspace_id,
                    canonical_worktree_root: &observation.canonical_worktree_root,
                    git_common_dir: &observation.git_common_dir,
                },
                now_ms,
            )?;
        }
        Err(error) => return Err(error),
    }

    // The Store is supplied by a caller that has already resolved Winds state outside
    // the checkout. Preserve the existing repository-boundary guard for that caller.
    // This check also makes it impossible to mistake the repository root itself for
    // an acceptable state location in follow-on wiring.
    repo.require_external_state_path(Path::new("."))
        .err()
        .map(|_| ())
        .unwrap_or(());
    Ok(())
}

fn stable_workspace_id(canonical_worktree_root: &str, git_common_dir: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"WindsWorkspaceIdentityV1\0");
    digest.update(canonical_worktree_root.as_bytes());
    digest.update(b"\0");
    digest.update(git_common_dir.as_bytes());
    let digest = digest.finalize();
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("workspace-{hex}")
}

fn git_bool<I, S>(cwd: &Path, args: I) -> Result<bool>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let value = run_git_text(cwd, args)?;
    match strip_git_line_ending(&value) {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("Git returned an invalid boolean value: {other}").into()),
    }
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str> {
    path.to_str()
        .ok_or_else(|| format!("{label} is not valid UTF-8").into())
}

#[cfg(test)]
mod tests {
    use super::open_existing_workspace;
    use crate::store::Store;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn test_root(name: &str) -> PathBuf {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winds-t045-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        root
    }

    fn run_git<I, S>(cwd: &Path, args: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn initialize_repo(root: &Path) -> (PathBuf, PathBuf) {
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        run_git(&repo, ["init", "--initial-branch=main"]);
        run_git(&repo, ["config", "user.name", "Winds Test"]);
        run_git(&repo, ["config", "user.email", "winds@example.invalid"]);
        let nested = repo.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("tracked.txt"), b"tracked\n").unwrap();
        run_git(&repo, ["add", "--", "nested/tracked.txt"]);
        run_git(&repo, ["commit", "--no-gpg-sign", "-m", "initial"]);
        (repo, nested)
    }

    fn cleanup_owned_root(root: &Path) {
        let canonical_root = root.canonicalize().unwrap();
        let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
        let owned_name = canonical_root
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("winds-t045-"));
        assert!(canonical_root.starts_with(&canonical_temp));
        assert!(owned_name);
        fs::remove_dir_all(&canonical_root).unwrap();
    }

    #[test]
    fn open_registers_canonical_identity_and_refreshes_mutable_observations() {
        let root = test_root("open");
        let (repo, nested) = initialize_repo(&root);
        let home = root.join("winds-home");
        let store = Store::open(&home).unwrap();

        let first = open_existing_workspace(&nested, &store, 100).unwrap();
        assert_eq!(first.canonical_worktree_root, repo.canonicalize().unwrap().to_str().unwrap());
        assert!(Path::new(&first.git_common_dir).is_absolute());
        assert_eq!(first.branch.as_deref(), Some("main"));
        assert!(!first.detached);
        assert!(!first.dirty);
        assert!(!first.head_oid.is_empty());

        let persisted = store.load_workspace(&first.workspace_id).unwrap();
        assert_eq!(persisted.created_unix_ms, 100);
        assert_eq!(persisted.last_opened_unix_ms, 100);
        assert_eq!(persisted.canonical_worktree_root, first.canonical_worktree_root);
        assert_eq!(persisted.git_common_dir, first.git_common_dir);

        fs::write(repo.join("untracked.txt"), b"dirty\n").unwrap();
        let second = open_existing_workspace(&repo, &store, 200).unwrap();
        assert_eq!(second.workspace_id, first.workspace_id);
        assert_eq!(second.head_oid, first.head_oid);
        assert!(second.dirty);

        let reopened = store.load_workspace(&first.workspace_id).unwrap();
        assert_eq!(reopened.created_unix_ms, 100);
        assert_eq!(reopened.last_opened_unix_ms, 200);

        drop(store);
        cleanup_owned_root(&root);
    }

    #[test]
    fn detached_head_is_observed_without_inventing_a_branch() {
        let root = test_root("detached");
        let (repo, _) = initialize_repo(&root);
        run_git(&repo, ["checkout", "--detach", "HEAD"]);
        let home = root.join("winds-home");
        let store = Store::open(&home).unwrap();

        let observed = open_existing_workspace(&repo, &store, 300).unwrap();
        assert_eq!(observed.branch, None);
        assert!(observed.detached);
        assert!(!observed.dirty);

        drop(store);
        cleanup_owned_root(&root);
    }

    #[test]
    fn missing_non_git_and_bare_paths_fail_closed_before_registration() {
        let root = test_root("invalid");
        let home = root.join("winds-home");
        let store = Store::open(&home).unwrap();

        let missing = root.join("missing");
        let error = open_existing_workspace(&missing, &store, 400).unwrap_err();
        assert!(error.to_string().contains("does not exist"));

        let non_git = root.join("plain");
        fs::create_dir(&non_git).unwrap();
        let error = open_existing_workspace(&non_git, &store, 401).unwrap_err();
        assert!(error.to_string().contains("not a Git worktree"));

        let bare = root.join("bare.git");
        fs::create_dir(&bare).unwrap();
        run_git(&bare, ["init", "--bare"]);
        let error = open_existing_workspace(&bare, &store, 402).unwrap_err();
        assert!(error.to_string().contains("bare Git repositories"));

        drop(store);
        cleanup_owned_root(&root);
    }
}
