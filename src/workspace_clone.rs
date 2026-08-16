use super::workspace::{WorkspaceInspection, open_existing_workspace};
use super::{Result, git_command};
use rusqlite::{Connection, params};
use serde::Serialize;
use std::ffi::OsStr;
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

    let output = git_command(parent)
        .arg("clone")
        .arg("--")
        .arg(OsStr::new(remote))
        .arg(&reserved_destination)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let status = output
            .status
            .code()
            .map_or_else(|| "signal".to_owned(), |code| code.to_string());
        return Err(format!(
            "system Git clone failed with status {status}; destination was not registered"
        )
        .into());
    }

    let workspace = open_existing_workspace(&reserved_destination, canonical_state_root, now_ms)?;
    persist_clone_origin(
        canonical_state_root,
        &workspace.workspace_id,
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
    if authority.is_empty() && !scheme.eq_ignore_ascii_case("file") {
        return Err("clone remote URL has no host after credential removal".into());
    }

    let path = &without_query_or_fragment[authority_end..];
    Ok(format!("{scheme}://{authority}{path}"))
}

fn sanitize_scp_like_remote(remote: &str) -> Option<String> {
    let colon = remote.find(':')?;
    let authority = &remote[..colon];
    let path = &remote[colon + 1..];
    if authority.is_empty()
        || path.is_empty()
        || authority.contains('/')
        || authority.contains('\\')
    {
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

fn persist_clone_origin(
    canonical_state_root: &Path,
    workspace_id: &str,
    remote_identity: &str,
    now_ms: i64,
) -> Result<()> {
    let connection = Connection::open(canonical_state_root.join("winds.db"))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(include_str!(
        "../migrations/0003_workspace_clone_origins.sql"
    ))?;
    connection.execute(
        "INSERT INTO workspace_clone_origins(workspace_id, remote_identity, recorded_unix_ms)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(workspace_id) DO UPDATE SET
             remote_identity = excluded.remote_identity,
             recorded_unix_ms = excluded.recorded_unix_ms",
        params![workspace_id, remote_identity, now_ms],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{clone_and_register_workspace, sanitize_remote_identity};
    use rusqlite::{Connection, params};
    use std::ffi::OsStr;
    use std::fs;
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

        cleanup_owned_root(&root);
    }

    #[test]
    fn clone_failure_happens_before_workspace_registration() {
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
        assert!(destination.is_dir());
        assert!(!state_root.join("winds.db").exists());

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
        assert!(sanitize_remote_identity("ext::sh -c 'echo unsafe'").is_err());
        assert!(sanitize_remote_identity("../relative/repo.git").is_err());
    }
}
