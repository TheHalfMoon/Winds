use super::workspace::{WorkspaceInspection, inspect_existing_workspace};
use super::{Result, git_command};
use crate::store::{NewWorkspace, Store};
use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

#[allow(
    dead_code,
    reason = "Spec 003 T046 backend API; the user-facing CLI caller lands in T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClonedWorkspace {
    pub workspace: WorkspaceInspection,
    pub remote_identity: String,
}

#[allow(
    dead_code,
    reason = "Spec 003 T046 backend API; the user-facing CLI caller lands in T057"
)]
pub fn clone_and_register_workspace(
    remote: &str,
    destination: &Path,
    canonical_state_root: &Path,
    now_ms: i64,
) -> Result<ClonedWorkspace> {
    let remote_identity = sanitize_remote_identity(remote)?;
    let reserved_destination = reserve_clone_destination(destination, canonical_state_root)?;
    let parent = reserved_destination
        .parent()
        .ok_or("clone destination has no parent directory")?;
    let git_remote = git_remote_argument(remote, &remote_identity)?;
    let git_destination = git_cli_local_path(&reserved_destination)?;

    require_reserved_clone_destination(&reserved_destination)?;
    let status = git_command(parent)
        .arg("-c")
        .arg("core.askPass=")
        .arg("clone")
        .arg("--")
        .arg(&git_remote)
        .arg(&git_destination)
        .env_remove("GIT_ASKPASS")
        .env_remove("SSH_ASKPASS")
        .env_remove("GIT_SSH")
        .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes")
        .env("GIT_SSH_VARIANT", "ssh")
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        let status = status
            .code()
            .map_or_else(|| "signal".to_owned(), |code| code.to_string());
        return match cleanup_failed_clone_destination(&reserved_destination) {
            Ok(()) => Err(format!(
                "system Git clone failed with status {status}; reserved destination was removed and not registered"
            )
            .into()),
            Err(cleanup_error) => Err(format!(
                "system Git clone failed with status {status}; destination was not registered and could not be safely removed: {cleanup_error}"
            )
            .into()),
        };
    }

    require_reserved_clone_destination(&reserved_destination)?;
    let workspace = inspect_existing_workspace(&reserved_destination, canonical_state_root)?;
    if Path::new(&workspace.canonical_worktree_root) != reserved_destination {
        return Err(
            "cloned workspace canonical root does not match the reserved clone destination".into(),
        );
    }
    let mut store = Store::open(canonical_state_root)?;
    store.register_cloned_workspace(
        NewWorkspace {
            workspace_id: &workspace.workspace_id,
            canonical_worktree_root: &workspace.canonical_worktree_root,
            git_common_dir: &workspace.git_common_dir,
        },
        &remote_identity,
        now_ms,
    )?;

    Ok(ClonedWorkspace {
        workspace,
        remote_identity,
    })
}

fn reserve_clone_destination(destination: &Path, canonical_state_root: &Path) -> Result<PathBuf> {
    if !destination.is_absolute() {
        return Err("clone destination must be an absolute path".into());
    }
    if destination.exists() {
        return Err(format!(
            "clone destination already exists: {}",
            destination.display()
        )
        .into());
    }

    let state_root = canonical_state_root
        .canonicalize()
        .map_err(|error| format!("Winds state root cannot be canonicalized: {error}"))?;
    if state_root != canonical_state_root {
        return Err("Winds state root must be supplied in canonical form".into());
    }
    if !state_root.is_dir() {
        return Err("Winds state root is not a directory".into());
    }

    let parent = destination
        .parent()
        .ok_or("clone destination has no parent directory")?;
    let file_name = destination
        .file_name()
        .ok_or("clone destination must name a directory")?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|error| format!("clone destination parent cannot be canonicalized: {error}"))?;
    if !canonical_parent.is_dir() {
        return Err("clone destination parent is not a directory".into());
    }

    let planned = canonical_parent.join(file_name);
    if planned.starts_with(&state_root) || state_root.starts_with(&planned) {
        return Err("clone destination and Winds state root must not overlap".into());
    }

    fs::create_dir(&planned).map_err(|error| {
        format!(
            "failed to reserve clone destination {}: {error}",
            planned.display()
        )
    })?;
    let canonical_reserved = planned
        .canonicalize()
        .map_err(|error| format!("reserved clone destination cannot be canonicalized: {error}"))?;
    if canonical_reserved != planned {
        return Err("reserved clone destination changed identity during validation".into());
    }

    Ok(planned)
}

fn require_reserved_clone_destination(destination: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(destination)
        .map_err(|error| format!("reserved clone destination cannot be inspected: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("reserved clone destination is no longer a real directory".into());
    }
    let canonical = destination
        .canonicalize()
        .map_err(|error| format!("reserved clone destination cannot be canonicalized: {error}"))?;
    if canonical != destination {
        return Err("reserved clone destination changed identity after reservation".into());
    }
    Ok(())
}

fn cleanup_failed_clone_destination(destination: &Path) -> Result<()> {
    require_reserved_clone_destination(destination)?;
    fs::remove_dir_all(destination).map_err(|error| {
        format!(
            "failed to remove reserved clone destination {}: {error}",
            destination.display()
        )
        .into()
    })
}

fn git_remote_argument(remote: &str, remote_identity: &str) -> Result<OsString> {
    if Path::new(remote).is_absolute() {
        return Ok(git_cli_local_path(Path::new(remote_identity))?.into_os_string());
    }
    Ok(OsString::from(remote))
}

#[cfg(not(windows))]
fn git_cli_local_path(path: &Path) -> Result<PathBuf> {
    Ok(path.to_path_buf())
}

#[cfg(windows)]
fn git_cli_local_path(path: &Path) -> Result<PathBuf> {
    let value = path.to_str().ok_or("local Git path is not valid UTF-8")?;
    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        let mut components = rest.split('\\');
        let server = components.next().unwrap_or_default();
        let share = components.next().unwrap_or_default();
        if server.is_empty() || share.is_empty() {
            return Err(
                "Windows verbatim UNC path must include non-empty server and share components"
                    .into(),
            );
        }
        return Ok(PathBuf::from(format!(r"\\{rest}")));
    }
    if let Some(rest) = value.strip_prefix(r"\\?\") {
        let bytes = rest.as_bytes();
        let ordinary_drive_path = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/');
        if !ordinary_drive_path {
            return Err(
                "Windows verbatim local Git path cannot be represented safely for Git CLI".into(),
            );
        }
        return Ok(PathBuf::from(rest));
    }
    Ok(path.to_path_buf())
}

fn sanitize_remote_identity(remote: &str) -> Result<String> {
    if remote.is_empty() {
        return Err("clone remote must not be empty".into());
    }
    if remote.chars().any(char::is_control) {
        return Err("clone remote contains control characters".into());
    }

    let local_path = Path::new(remote);
    if local_path.is_absolute() {
        let canonical = local_path
            .canonicalize()
            .map_err(|error| format!("local clone remote cannot be canonicalized: {error}"))?;
        return canonical
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| "local clone remote is not valid UTF-8".into());
    }

    if let Some((scheme, rest)) = remote.split_once("://") {
        return sanitize_url_remote(scheme, rest);
    }

    if remote.contains("::") {
        return Err("Git remote-helper transport syntax is not supported by Spec 003 T046".into());
    }

    if let Some(sanitized) = sanitize_scp_like_remote(remote) {
        return Ok(sanitized);
    }

    Err(
        "relative local clone remotes are ambiguous; use an absolute path or explicit Git URL"
            .into(),
    )
}

fn sanitize_url_remote(scheme: &str, rest: &str) -> Result<String> {
    let valid_scheme = !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, ch)| {
            if index == 0 {
                ch.is_ascii_alphabetic()
            } else {
                ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')
            }
        });
    if !valid_scheme {
        return Err("clone remote has an invalid URL scheme".into());
    }

    let scheme = scheme.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https" | "ssh" | "git" | "file") {
        return Err("clone remote URL scheme is not supported by Spec 003 T046".into());
    }

    let tail_index = rest
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(rest.len());
    let without_query_or_fragment = &rest[..tail_index];
    let authority_end = without_query_or_fragment
        .find('/')
        .unwrap_or(without_query_or_fragment.len());
    let raw_authority = &without_query_or_fragment[..authority_end];
    let authority = raw_authority
        .rsplit_once('@')
        .map_or(raw_authority, |(_, host)| host);
    if authority.is_empty() && scheme != "file" {
        return Err("clone remote URL has no host after credential removal".into());
    }

    let path = &without_query_or_fragment[authority_end..];
    Ok(format!("{scheme}://{authority}{path}"))
}

fn sanitize_scp_like_remote(remote: &str) -> Option<String> {
    let colon = remote.find(':')?;
    let authority = &remote[..colon];
    let raw_path = &remote[colon + 1..];
    if authority.is_empty()
        || raw_path.is_empty()
        || authority.contains('/')
        || authority.contains('\\')
    {
        return None;
    }
    let tail_index = raw_path
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
        .unwrap_or(raw_path.len());
    let path = &raw_path[..tail_index];
    if path.is_empty() {
        return None;
    }
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    if host.is_empty() {
        return None;
    }
    Some(format!("{host}:{path}"))
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::git_cli_local_path;
    use super::{clone_and_register_workspace, sanitize_remote_identity};
    #[cfg(unix)]
    use super::{git_remote_argument, reserve_clone_destination};
    use crate::store::Store;
    use rusqlite::{Connection, params};
    use std::ffi::OsStr;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn test_root(name: &str) -> PathBuf {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winds-t046-{name}-{}-{sequence}",
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

    fn initialize_remote(root: &Path, marker: &Path) -> (PathBuf, String) {
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        run_git(&source, ["init", "--initial-branch=main"]);
        run_git(&source, ["config", "user.name", "Winds Test"]);
        run_git(&source, ["config", "user.email", "winds@example.invalid"]);
        fs::write(source.join("tracked.txt"), b"tracked\n").unwrap();
        fs::write(
            source.join(".envrc"),
            format!("touch '{}'\n", marker.display()),
        )
        .unwrap();
        fs::write(source.join(".mise.toml"), b"[tools]\nnode = '22'\n").unwrap();
        run_git(
            &source,
            ["add", "--", "tracked.txt", ".envrc", ".mise.toml"],
        );
        run_git(&source, ["commit", "--no-gpg-sign", "-m", "fixture"]);
        let head = run_git(&source, ["rev-parse", "HEAD"]);

        let remote = root.join("remote.git");
        run_git(
            root,
            [
                "clone",
                "--bare",
                "--",
                source.to_str().unwrap(),
                remote.to_str().unwrap(),
            ],
        );
        (remote, head)
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
            .is_some_and(|name| name.starts_with("winds-t046-"));
        assert!(canonical_root.starts_with(&canonical_temp));
        assert!(owned_name);
        fs::remove_dir_all(&canonical_root).unwrap();
    }

    #[test]
    fn clone_registers_workspace_and_persists_only_sanitized_remote_identity() {
        let root = test_root("clone");
        let marker = root.join("bootstrap-ran");
        let (remote, source_head) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("cloned workspace");

        let cloned =
            clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 100)
                .unwrap();

        assert_eq!(
            cloned.workspace.head_oid.as_deref(),
            Some(source_head.as_str())
        );
        assert_eq!(cloned.workspace.branch.as_deref(), Some("main"));
        assert!(!cloned.workspace.detached);
        assert!(!cloned.workspace.dirty);
        assert_eq!(
            cloned.workspace.canonical_worktree_root,
            destination.canonicalize().unwrap().to_str().unwrap()
        );
        assert_eq!(
            cloned.remote_identity,
            remote.canonicalize().unwrap().to_str().unwrap()
        );
        assert!(destination.join(".envrc").is_file());
        assert!(destination.join(".mise.toml").is_file());
        assert!(!marker.exists());

        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        let (remote_identity, recorded_unix_ms): (String, i64) = connection
            .query_row(
                "SELECT remote_identity, recorded_unix_ms
                 FROM workspace_clone_origins WHERE workspace_id = ?1",
                params![cloned.workspace.workspace_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(remote_identity, cloned.remote_identity);
        assert_eq!(recorded_unix_ms, 100);
        drop(connection);

        cleanup_owned_root(&root);
    }

    #[test]
    fn clone_failure_happens_before_workspace_registration_and_allows_retry() {
        let root = test_root("failure");
        let state_root = create_state_root(&root);
        let not_a_repo = root.join("not-a-repo");
        fs::write(&not_a_repo, b"not git\n").unwrap();
        let destination = root.join("failed-clone");

        let error = clone_and_register_workspace(
            not_a_repo.to_str().unwrap(),
            &destination,
            &state_root,
            200,
        )
        .unwrap_err();
        assert!(error.to_string().contains("system Git clone failed"));
        assert!(!destination.exists());
        assert!(!state_root.join("winds.db").exists());

        let marker = root.join("retry-bootstrap-ran");
        let retry_root = root.join("retry-source");
        fs::create_dir(&retry_root).unwrap();
        let (remote, _) = initialize_remote(&retry_root, &marker);
        clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 201)
            .unwrap();
        assert!(destination.is_dir());
        assert!(!marker.exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn destination_validation_is_fail_closed_before_clone() {
        let root = test_root("destination");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);

        let existing = root.join("existing");
        fs::create_dir(&existing).unwrap();
        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &existing, &state_root, 300)
                .unwrap_err();
        assert!(error.to_string().contains("already exists"));

        let inside_state = state_root.join("source-inside-state");
        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &inside_state, &state_root, 301)
                .unwrap_err();
        assert!(error.to_string().contains("must not overlap"));
        assert!(!inside_state.exists());
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn origin_persistence_failure_rolls_back_workspace_registration() {
        let root = test_root("atomic-origin");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("atomic-clone");

        let store = Store::open(&state_root).unwrap();
        drop(store);
        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_clone_origin
                 BEFORE INSERT ON workspace_clone_origins
                 BEGIN
                     SELECT RAISE(ABORT, 'forced clone-origin failure');
                 END;",
            )
            .unwrap();
        drop(connection);

        let error =
            clone_and_register_workspace(remote.to_str().unwrap(), &destination, &state_root, 350)
                .unwrap_err();
        assert!(error.to_string().contains("forced clone-origin failure"));
        assert!(destination.is_dir());

        let connection = Connection::open(state_root.join("winds.db")).unwrap();
        let workspace_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
            .unwrap();
        let origin_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM workspace_clone_origins", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(workspace_count, 0);
        assert_eq!(origin_count, 0);
        drop(connection);

        cleanup_owned_root(&root);
    }

    #[test]
    fn remote_sanitization_removes_credentials_and_url_secret_components() {
        let sanitized = sanitize_remote_identity(
            "https://alice:super-secret@example.test/org/repo.git?token=also-secret#private",
        )
        .unwrap();
        assert_eq!(sanitized, "https://example.test/org/repo.git");
        assert!(!sanitized.contains("alice"));
        assert!(!sanitized.contains("super-secret"));
        assert!(!sanitized.contains("also-secret"));
        assert!(!sanitized.contains("private"));

        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git").unwrap(),
            "example.test:org/repo.git"
        );
        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git?token=secret").unwrap(),
            "example.test:org/repo.git"
        );
        assert_eq!(
            sanitize_remote_identity("git@example.test:org/repo.git#private").unwrap(),
            "example.test:org/repo.git"
        );
        assert!(sanitize_remote_identity("ext::sh -c 'echo unsafe'").is_err());
        assert!(sanitize_remote_identity("custom://example.test/org/repo.git").is_err());
        assert!(sanitize_remote_identity("../relative/repo.git").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn absolute_local_symlink_remote_uses_one_canonical_identity_for_git_and_persistence() {
        let root = test_root("remote-symlink");
        let first_root = root.join("first");
        let second_root = root.join("second");
        fs::create_dir(&first_root).unwrap();
        fs::create_dir(&second_root).unwrap();
        let (first_remote, _) = initialize_remote(&first_root, &root.join("first-marker"));
        let (second_remote, _) = initialize_remote(&second_root, &root.join("second-marker"));
        let link = root.join("remote-link");
        symlink(&first_remote, &link).unwrap();

        let identity = sanitize_remote_identity(link.to_str().unwrap()).unwrap();
        assert_eq!(
            identity,
            first_remote.canonicalize().unwrap().to_str().unwrap()
        );

        fs::remove_file(&link).unwrap();
        symlink(&second_remote, &link).unwrap();
        let git_argument = git_remote_argument(link.to_str().unwrap(), &identity).unwrap();
        assert_eq!(PathBuf::from(git_argument), PathBuf::from(&identity));
        assert_ne!(
            identity,
            second_remote.canonicalize().unwrap().to_str().unwrap()
        );

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn reserved_destination_revalidation_rejects_symlink_replacement() {
        let root = test_root("destination-replacement");
        let state_root = create_state_root(&root);
        let destination = root.join("clone");
        let replacement = root.join("replacement");
        fs::create_dir(&replacement).unwrap();
        let reserved = reserve_clone_destination(&destination, &state_root).unwrap();
        fs::remove_dir(&reserved).unwrap();
        symlink(&replacement, &reserved).unwrap();

        assert!(super::require_reserved_clone_destination(&reserved).is_err());

        fs::remove_file(&reserved).unwrap();
        cleanup_owned_root(&root);
    }

    #[cfg(windows)]
    #[test]
    fn windows_git_cli_local_path_removes_only_supported_verbatim_prefixes() {
        assert_eq!(
            git_cli_local_path(Path::new(r"\\?\C:\Temp\Winds Clone")).unwrap(),
            PathBuf::from(r"C:\Temp\Winds Clone")
        );
        assert_eq!(
            git_cli_local_path(Path::new(r"\\?\UNC\server\share\Winds Clone")).unwrap(),
            PathBuf::from(r"\\server\share\Winds Clone")
        );
        assert!(git_cli_local_path(Path::new(r"\\?\UNC\server")).is_err());
        assert!(git_cli_local_path(Path::new(r"\\?\UNC\")).is_err());
        assert!(git_cli_local_path(Path::new(r"\\?\Volume{abc}\repo")).is_err());
    }
}
