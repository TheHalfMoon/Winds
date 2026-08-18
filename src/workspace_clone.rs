use super::workspace::{WorkspaceInspection, inspect_existing_workspace};
use super::{Result, git_command};
use crate::store::{NewWorkspace, Store};
use serde::Serialize;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
use std::ffi::OsString;
use std::fs;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CLONE_STAGING_ID: AtomicU64 = AtomicU64::new(0);
const MAX_STAGING_ATTEMPTS: usize = 128;

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
    clone_and_register_workspace_impl(remote, destination, canonical_state_root, now_ms, |_, _| {
        Ok(())
    })
}

fn clone_and_register_workspace_impl<F>(
    remote: &str,
    destination: &Path,
    canonical_state_root: &Path,
    now_ms: i64,
    after_staging_created: F,
) -> Result<ClonedWorkspace>
where
    F: FnOnce(&Path, &Path) -> Result<()>,
{
    let remote_identity = sanitize_remote_identity(remote)?;
    let planned_destination = plan_clone_destination(destination, canonical_state_root)?;
    let parent = planned_destination
        .parent()
        .ok_or("clone destination has no parent directory")?;
    let staging_root = create_private_clone_staging(parent)?;
    let staged_checkout = staging_root.join("checkout");
    let git_remote = git_remote_argument(remote, &remote_identity)?;
    let git_destination = git_cli_local_path(&staged_checkout)?;

    after_staging_created(&staged_checkout, &planned_destination)?;

    let status = git_command(&staging_root)
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
        return Err(format!(
            "system Git clone failed with status {status}; requested destination was not published or registered; partial private staging was retained at {}",
            staging_root.display()
        )
        .into());
    }

    require_owned_staged_checkout(&staging_root, &staged_checkout)?;
    atomic_publish_no_replace(&staged_checkout, &planned_destination).map_err(|error| {
        format!(
            "cloned checkout could not be atomically published without replacing the requested destination; requested destination was not registered and private staging was retained at {}: {error}",
            staging_root.display()
        )
    })?;

    // Only the now-empty private staging parent is removed. The public requested
    // destination is never recursively cleaned by Winds.
    let _ = fs::remove_dir(&staging_root);

    let workspace = inspect_existing_workspace(&planned_destination, canonical_state_root)?;
    if Path::new(&workspace.canonical_worktree_root) != planned_destination {
        return Err(
            "cloned workspace canonical root does not match the atomically published clone destination"
                .into(),
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

fn plan_clone_destination(destination: &Path, canonical_state_root: &Path) -> Result<PathBuf> {
    if !destination.is_absolute() {
        return Err("clone destination must be an absolute path".into());
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
    match fs::symlink_metadata(&planned) {
        Ok(_) => {
            return Err(format!("clone destination already exists: {}", planned.display()).into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "clone destination cannot be inspected before clone: {}: {error}",
                planned.display()
            )
            .into());
        }
    }

    if planned.starts_with(&state_root) || state_root.starts_with(&planned) {
        return Err("clone destination and Winds state root must not overlap".into());
    }

    Ok(planned)
}

fn create_private_clone_staging(parent: &Path) -> Result<PathBuf> {
    for _ in 0..MAX_STAGING_ATTEMPTS {
        let sequence = NEXT_CLONE_STAGING_ID.fetch_add(1, Ordering::Relaxed);
        let staging = parent.join(format!(
            ".winds-clone-stage-{}-{sequence}",
            std::process::id()
        ));
        let mut builder = fs::DirBuilder::new();
        builder.recursive(false);
        #[cfg(unix)]
        builder.mode(0o700);
        match builder.create(&staging) {
            Ok(()) => {
                let canonical = staging.canonicalize().map_err(|error| {
                    format!("private clone staging cannot be canonicalized: {error}")
                })?;
                if canonical != staging {
                    return Err("private clone staging changed identity during creation".into());
                }
                return Ok(staging);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create private clone staging under {}: {error}",
                    parent.display()
                )
                .into());
            }
        }
    }
    Err("could not allocate a unique private clone staging directory".into())
}

fn require_owned_staged_checkout(staging_root: &Path, staged_checkout: &Path) -> Result<()> {
    let staging_metadata = fs::symlink_metadata(staging_root)
        .map_err(|error| format!("private clone staging cannot be inspected: {error}"))?;
    if staging_metadata.file_type().is_symlink() || !staging_metadata.is_dir() {
        return Err("private clone staging is no longer a real directory".into());
    }
    let canonical_staging = staging_root
        .canonicalize()
        .map_err(|error| format!("private clone staging cannot be canonicalized: {error}"))?;
    if canonical_staging != staging_root {
        return Err("private clone staging changed identity after creation".into());
    }

    let checkout_metadata = fs::symlink_metadata(staged_checkout)
        .map_err(|error| format!("staged clone checkout cannot be inspected: {error}"))?;
    if checkout_metadata.file_type().is_symlink() || !checkout_metadata.is_dir() {
        return Err("staged clone checkout is not a real directory".into());
    }
    let canonical_checkout = staged_checkout
        .canonicalize()
        .map_err(|error| format!("staged clone checkout cannot be canonicalized: {error}"))?;
    if canonical_checkout != staged_checkout
        || canonical_checkout.parent() != Some(canonical_staging.as_path())
    {
        return Err("staged clone checkout escaped its private staging parent".into());
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = unix_path_cstring(source, "staged clone source")?;
    let destination = unix_path_cstring(destination, "clone destination")?;
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(target_os = "macos")]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = unix_path_cstring(source, "staged clone source")?;
    let destination = unix_path_cstring(destination, "clone destination")?;
    let result =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn unix_path_cstring(path: &Path, label: &str) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| format!("{label} contains an embedded NUL byte").into())
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
}

#[cfg(windows)]
fn atomic_publish_no_replace(source: &Path, destination: &Path) -> Result<()> {
    let source = windows_path_wide(source, "staged clone source")?;
    let destination = windows_path_wide(destination, "clone destination")?;
    let result = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) };
    if result != 0 {
        Ok(())
    } else {
        Err(format!(
            "atomic no-replace clone publish failed: {}",
            std::io::Error::last_os_error()
        )
        .into())
    }
}

#[cfg(windows)]
fn windows_path_wide(path: &Path, label: &str) -> Result<Vec<u16>> {
    let mut encoded = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if encoded.contains(&0) {
        return Err(format!("{label} contains an embedded NUL code unit").into());
    }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn atomic_publish_no_replace(_source: &Path, _destination: &Path) -> Result<()> {
    Err("atomic no-replace clone publish is unsupported on this platform".into())
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
    #[cfg(unix)]
    use super::git_remote_argument;
    use super::{
        clone_and_register_workspace, clone_and_register_workspace_impl, sanitize_remote_identity,
    };
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
    fn concurrent_destination_creation_blocks_atomic_publish_without_replacement() {
        let root = test_root("publish-race");
        let marker = root.join("bootstrap-ran");
        let (remote, _) = initialize_remote(&root, &marker);
        let state_root = create_state_root(&root);
        let destination = root.join("raced-destination");
        let replacement_marker = destination.join("replacement-marker");
        let mut staged_checkout = None;

        let error = clone_and_register_workspace_impl(
            remote.to_str().unwrap(),
            &destination,
            &state_root,
            360,
            |staged, requested| {
                staged_checkout = Some(staged.to_path_buf());
                let expected_requested = destination
                    .parent()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .join(destination.file_name().unwrap());
                assert_eq!(requested, expected_requested);
                fs::create_dir(requested)?;
                fs::write(requested.join("replacement-marker"), b"replacement\n")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("atomically published"));
        assert_eq!(fs::read(&replacement_marker).unwrap(), b"replacement\n");
        assert!(staged_checkout.unwrap().is_dir());
        assert!(!state_root.join("winds.db").exists());

        cleanup_owned_root(&root);
    }

    #[test]
    fn failed_clone_never_recursively_cleans_a_concurrent_destination() {
        let root = test_root("failure-race");
        let state_root = create_state_root(&root);
        let not_a_repo = root.join("not-a-repo");
        fs::write(&not_a_repo, b"not git\n").unwrap();
        let destination = root.join("raced-destination");
        let replacement_marker = destination.join("replacement-marker");

        let error = clone_and_register_workspace_impl(
            not_a_repo.to_str().unwrap(),
            &destination,
            &state_root,
            361,
            |_, requested| {
                fs::create_dir(requested)?;
                fs::write(requested.join("replacement-marker"), b"replacement\n")?;
                Ok(())
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("system Git clone failed"));
        assert_eq!(fs::read(&replacement_marker).unwrap(), b"replacement\n");
        assert!(!state_root.join("winds.db").exists());

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
    fn destination_validation_rejects_broken_symlink_before_staging() {
        let root = test_root("broken-destination");
        let state_root = create_state_root(&root);
        let destination = root.join("broken-destination");
        symlink(root.join("missing-target"), &destination).unwrap();

        let error = super::plan_clone_destination(&destination, &state_root).unwrap_err();
        assert!(error.to_string().contains("already exists"));

        fs::remove_file(&destination).unwrap();
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
