use super::{
    Repo, Result, git_command, run_bounded_read_only_git, run_git_text, strip_git_line_ending,
};
use crate::store::{NewWorkspace, Store};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
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
    pub head_oid: Option<String>,
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
    canonical_state_root: &Path,
    now_ms: i64,
) -> Result<WorkspaceInspection> {
    let observation = inspect_existing_workspace(path, canonical_state_root)?;
    let store = Store::open(canonical_state_root)?;
    register_observed_workspace(&observation, &store, now_ms)?;
    Ok(observation)
}

pub(super) fn inspect_existing_workspace(
    path: &Path,
    canonical_state_root: &Path,
) -> Result<WorkspaceInspection> {
    let repo = open_worktree(path)?;
    let observation = inspect_worktree(&repo)?;
    require_canonical_external_state_root(&repo, canonical_state_root)?;
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
    let git_common_dir = utf8_path(repo.common_dir(), "Git common directory")?.to_owned();
    let branch = branch_name(repo)?;
    let head_oid = exact_head(repo, branch.as_deref())?;
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

fn exact_head(repo: &Repo, branch: Option<&str>) -> Result<Option<String>> {
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
    let mut command = git_command(repo.root());
    command.env("GIT_OPTIONAL_LOCKS", "0").args([
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--ignore-submodules=none",
    ]);
    run_bounded_read_only_git(command, "workspace dirty-state inspection")
}

fn require_canonical_external_state_root(repo: &Repo, state_root: &Path) -> Result<()> {
    if !state_root.is_absolute() {
        return Err("Winds state root must be an absolute canonical path".into());
    }
    let canonical = state_root
        .canonicalize()
        .map_err(|error| format!("Winds state root cannot be canonicalized: {error}"))?;
    if canonical != state_root {
        return Err("Winds state root must be supplied in canonical form".into());
    }
    repo.require_external_state_path(&canonical)
}

fn register_observed_workspace(
    observation: &WorkspaceInspection,
    store: &Store,
    now_ms: i64,
) -> Result<()> {
    let workspace = NewWorkspace {
        workspace_id: &observation.workspace_id,
        canonical_worktree_root: &observation.canonical_worktree_root,
        git_common_dir: &observation.git_common_dir,
    };

    match store.create_workspace(workspace, now_ms) {
        Ok(()) => Ok(()),
        Err(error) if is_sqlite_constraint_violation(error.as_ref()) => {
            let existing = match store.load_workspace(&observation.workspace_id) {
                Ok(existing) => existing,
                Err(_) => return Err(error),
            };
            if existing.canonical_worktree_root != observation.canonical_worktree_root
                || existing.git_common_dir != observation.git_common_dir
            {
                return Err(format!(
                    "stored workspace identity conflicts with observed Git identity: {}",
                    observation.workspace_id
                )
                .into());
            }
            store.mark_workspace_opened(&observation.workspace_id, now_ms)
        }
        Err(error) => Err(error),
    }
}

fn is_sqlite_constraint_violation(error: &(dyn std::error::Error + Send + Sync + 'static)) -> bool {
    error
        .downcast_ref::<rusqlite::Error>()
        .and_then(rusqlite::Error::sqlite_error_code)
        == Some(rusqlite::ErrorCode::ConstraintViolation)
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

    fn initialize_unborn_repo(root: &Path) -> PathBuf {
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        run_git(&repo, ["init", "--initial-branch=main"]);
        repo
    }

    fn initialize_repo(root: &Path) -> (PathBuf, PathBuf) {
        let repo = initialize_unborn_repo(root);
        run_git(&repo, ["config", "user.name", "Winds Test"]);
        run_git(&repo, ["config", "user.email", "winds@example.invalid"]);
        let nested = repo.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("tracked.txt"), b"tracked\n").unwrap();
        run_git(&repo, ["add", "--", "nested/tracked.txt"]);
        run_git(&repo, ["commit", "--no-gpg-sign", "-m", "initial"]);
        (repo, nested)
    }

    fn create_state_root(root: &Path) -> PathBuf {
        let home = root.join("winds-home");
        fs::create_dir(&home).unwrap();
        home.canonicalize().unwrap()
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
        let canonical_home = create_state_root(&root);

        let first = open_existing_workspace(&nested, &canonical_home, 100).unwrap();
        assert_eq!(
            first.canonical_worktree_root,
            repo.canonicalize().unwrap().to_str().unwrap()
        );
        assert!(Path::new(&first.git_common_dir).is_absolute());
        assert_eq!(first.branch.as_deref(), Some("main"));
        assert!(!first.detached);
        assert!(!first.dirty);
        assert!(
            first
                .head_oid
                .as_deref()
                .is_some_and(|head| !head.is_empty())
        );

        let store = Store::open(&canonical_home).unwrap();
        let persisted = store.load_workspace(&first.workspace_id).unwrap();
        assert_eq!(persisted.created_unix_ms, 100);
        assert_eq!(persisted.last_opened_unix_ms, 100);
        assert_eq!(
            persisted.canonical_worktree_root,
            first.canonical_worktree_root
        );
        assert_eq!(persisted.git_common_dir, first.git_common_dir);
        drop(store);

        #[cfg(unix)]
        {
            let symlinked_repo = root.join("repo-link");
            std::os::unix::fs::symlink(&repo, &symlinked_repo).unwrap();
            let via_symlink =
                open_existing_workspace(&symlinked_repo, &canonical_home, 150).unwrap();
            assert_eq!(via_symlink.workspace_id, first.workspace_id);
            assert_eq!(
                via_symlink.canonical_worktree_root,
                first.canonical_worktree_root
            );
            assert_eq!(via_symlink.git_common_dir, first.git_common_dir);
            assert_eq!(via_symlink.head_oid, first.head_oid);
        }

        fs::write(repo.join("untracked.txt"), b"dirty\n").unwrap();
        let second = open_existing_workspace(&repo, &canonical_home, 200).unwrap();
        assert_eq!(second.workspace_id, first.workspace_id);
        assert_eq!(second.head_oid, first.head_oid);
        assert!(second.dirty);

        let store = Store::open(&canonical_home).unwrap();
        let reopened = store.load_workspace(&first.workspace_id).unwrap();
        assert_eq!(reopened.created_unix_ms, 100);
        assert_eq!(reopened.last_opened_unix_ms, 200);
        drop(store);

        cleanup_owned_root(&root);
    }

    #[test]
    fn unborn_head_is_observed_without_inventing_a_commit() {
        let root = test_root("unborn");
        let repo = initialize_unborn_repo(&root);
        let canonical_home = create_state_root(&root);

        let observed = open_existing_workspace(&repo, &canonical_home, 250).unwrap();
        assert_eq!(observed.head_oid, None);
        assert_eq!(observed.branch.as_deref(), Some("main"));
        assert!(!observed.detached);
        assert!(!observed.dirty);

        let store = Store::open(&canonical_home).unwrap();
        let persisted = store.load_workspace(&observed.workspace_id).unwrap();
        assert_eq!(persisted.created_unix_ms, 250);
        assert_eq!(persisted.last_opened_unix_ms, 250);
        drop(store);

        cleanup_owned_root(&root);
    }

    #[test]
    fn detached_head_is_observed_without_inventing_a_branch() {
        let root = test_root("detached");
        let (repo, _) = initialize_repo(&root);
        run_git(&repo, ["checkout", "--detach", "HEAD"]);
        let canonical_home = create_state_root(&root);

        let observed = open_existing_workspace(&repo, &canonical_home, 300).unwrap();
        assert!(observed.head_oid.is_some());
        assert_eq!(observed.branch, None);
        assert!(observed.detached);
        assert!(!observed.dirty);

        cleanup_owned_root(&root);
    }

    #[test]
    fn missing_file_non_git_and_bare_paths_fail_closed_before_registration() {
        let root = test_root("invalid");
        let canonical_home = create_state_root(&root);

        let missing = root.join("missing");
        let error = open_existing_workspace(&missing, &canonical_home, 400).unwrap_err();
        assert!(error.to_string().contains("does not exist"));

        let regular_file = root.join("regular-file");
        fs::write(&regular_file, b"not a directory\n").unwrap();
        let error = open_existing_workspace(&regular_file, &canonical_home, 401).unwrap_err();
        assert!(error.to_string().contains("not a directory"));

        let non_git = root.join("plain");
        fs::create_dir(&non_git).unwrap();
        let error = open_existing_workspace(&non_git, &canonical_home, 402).unwrap_err();
        assert!(error.to_string().contains("not a Git worktree"));

        let bare = root.join("bare.git");
        fs::create_dir(&bare).unwrap();
        run_git(&bare, ["init", "--bare"]);
        let error = open_existing_workspace(&bare, &canonical_home, 403).unwrap_err();
        assert!(error.to_string().contains("bare Git repositories"));
        assert!(!canonical_home.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn registration_rejects_actual_state_root_inside_the_checkout() {
        let root = test_root("state-boundary");
        let (repo, _) = initialize_repo(&root);
        let inside = repo.join("winds-state");
        fs::create_dir(&inside).unwrap();
        let canonical_inside = inside.canonicalize().unwrap();

        let error = open_existing_workspace(&repo, &canonical_inside, 500).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Winds state must live outside the source checkout")
        );
        assert!(!inside.join("winds.db").exists());

        cleanup_owned_root(&root);
    }
}
