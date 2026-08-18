use crate::git::workspace::{WorkspaceInspection, open_existing_workspace};
use crate::git::workspace_clone::clone_and_register_workspace;
use crate::git::workspace_inventory::inventory_workspace_environment;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use crate::command::history::{SessionHistoryPolicy, SessionHistoryRecorder};
#[cfg(unix)]
use crate::domain::{ExecutionKind, FactSource};
#[cfg(unix)]
use crate::git::shell_profiles::{
    ShellProfile, discover_native_shell_profiles, validate_shell_profile_for_launch,
};
#[cfg(unix)]
use crate::git::terminal::{TerminalSession, TerminalSize};
#[cfg(unix)]
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
#[cfg(unix)]
use crate::store::{NewExecution, NewTerminalSession, NewWorkspace, Store};
#[cfg(unix)]
use rusqlite::{Connection, params};
#[cfg(unix)]
use std::io::{self, Cursor};
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(name: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t059-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let Ok(canonical_temp) = std::env::temp_dir().canonicalize() else {
            return;
        };
        let Ok(canonical_root) = self.0.canonicalize() else {
            return;
        };
        let owned_name = canonical_root
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("winds-t059-"));
        if canonical_root.starts_with(canonical_temp) && owned_name {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
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
        .env_remove("GIT_INDEX_FILE")
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
        "git failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[cfg(unix)]
fn initialize_repo(root: &Path, name: &str) -> PathBuf {
    let repo = root.join(name);
    fs::create_dir(&repo).unwrap();
    run_git(&repo, ["init", "--initial-branch=main"]);
    run_git(&repo, ["config", "user.name", "Winds T059"]);
    run_git(
        &repo,
        ["config", "user.email", "winds-t059@example.invalid"],
    );
    fs::write(repo.join("tracked.txt"), b"tracked\n").unwrap();
    run_git(&repo, ["add", "--", "tracked.txt"]);
    run_git(&repo, ["commit", "--no-gpg-sign", "-m", "fixture"]);
    repo
}

fn create_state_root(root: &Path, name: &str) -> PathBuf {
    let state = root.join(name);
    fs::create_dir(&state).unwrap();
    state.canonicalize().unwrap()
}

#[cfg(unix)]
fn profile_inventory(candidate: &Path) -> WorkspaceEnvironmentInventory {
    WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: "/unused/worktree".to_owned(),
        git_common_dir: "/unused/git-common".to_owned(),
        shell_candidates: vec![candidate.to_str().unwrap().to_owned()],
        detected_manifests: Vec::new(),
    }
}

#[cfg(unix)]
fn profile_for_candidate(candidate: &Path) -> ShellProfile {
    let candidate = candidate.to_str().unwrap();
    discover_native_shell_profiles(&profile_inventory(Path::new(candidate)))
        .unwrap()
        .into_iter()
        .find(|profile| profile.executable == candidate)
        .expect("T059 fixture candidate must be discoverable")
}

#[cfg(unix)]
fn create_executable(path: &Path, body: &[u8]) {
    fs::write(path, body).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[test]
fn t059_invalid_and_bare_workspaces_fail_before_registration() {
    let root = TestRoot::new("workspace-invalid");
    let state_root = create_state_root(root.path(), "state");

    let missing = root.path().join("missing");
    let error = open_existing_workspace(&missing, &state_root, 1).unwrap_err();
    assert!(error.to_string().contains("does not exist"));

    let plain = root.path().join("plain");
    fs::create_dir(&plain).unwrap();
    let error = open_existing_workspace(&plain, &state_root, 2).unwrap_err();
    assert!(error.to_string().contains("not a Git worktree"));

    let bare = root.path().join("bare.git");
    fs::create_dir(&bare).unwrap();
    run_git(&bare, ["init", "--bare"]);
    let error = open_existing_workspace(&bare, &state_root, 3).unwrap_err();
    assert!(error.to_string().contains("bare Git repositories"));

    assert!(!state_root.join("winds.db").exists());
}

#[cfg(unix)]
#[test]
fn t059_symlinked_workspace_fails_closed_when_target_is_invalid_or_missing() {
    let root = TestRoot::new("workspace-symlink");
    let state_root = create_state_root(root.path(), "state");
    let target = root.path().join("plain-target");
    fs::create_dir(&target).unwrap();
    let link = root.path().join("workspace-link");
    symlink(&target, &link).unwrap();

    let error = open_existing_workspace(&link, &state_root, 10).unwrap_err();
    assert!(error.to_string().contains("not a Git worktree"));
    assert!(!state_root.join("winds.db").exists());

    fs::remove_dir(&target).unwrap();
    let error = open_existing_workspace(&link, &state_root, 11).unwrap_err();
    assert!(error.to_string().contains("does not exist"));
    assert!(!state_root.join("winds.db").exists());
}

#[cfg(unix)]
#[test]
fn t059_credential_bearing_clone_url_persists_only_sanitized_identity() {
    let root = TestRoot::new("clone-credentials");
    let source = initialize_repo(root.path(), "source");
    let remote = root.path().join("remote.git");
    run_git(
        root.path(),
        [
            "clone",
            "--bare",
            "--",
            source.to_str().unwrap(),
            remote.to_str().unwrap(),
        ],
    );
    let remote = remote.canonicalize().unwrap();
    let remote_path = remote.to_str().unwrap();
    let secret_url = format!("file://alice:super-secret@localhost{remote_path}");
    let sanitized = format!("file://localhost{remote_path}");
    let state_root = create_state_root(root.path(), "state");
    let destination = root.path().join("clone");

    let cloned = clone_and_register_workspace(&secret_url, &destination, &state_root, 20).unwrap();
    assert_eq!(cloned.remote_identity, sanitized);
    assert!(!cloned.remote_identity.contains("alice"));
    assert!(!cloned.remote_identity.contains("super-secret"));

    let connection = Connection::open(state_root.join("winds.db")).unwrap();
    let persisted: String = connection
        .query_row(
            "SELECT remote_identity FROM workspace_clone_origins WHERE workspace_id = ?1",
            params![cloned.workspace.workspace_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted, sanitized);
    assert!(!persisted.contains("alice"));
    assert!(!persisted.contains("super-secret"));
}

#[test]
fn t059_clone_failure_never_registers_a_workspace() {
    let root = TestRoot::new("clone-failure");
    let state_root = create_state_root(root.path(), "state");
    let not_a_repo = root.path().join("not-a-repo");
    fs::write(&not_a_repo, b"not git\n").unwrap();
    let destination = root.path().join("failed-clone");

    let error =
        clone_and_register_workspace(not_a_repo.to_str().unwrap(), &destination, &state_root, 30)
            .unwrap_err();
    assert!(error.to_string().contains("system Git clone failed"));
    assert!(destination.is_dir());
    assert!(!state_root.join("winds.db").exists());
}

#[cfg(unix)]
#[test]
fn t059_disappearing_shell_executable_is_rejected_at_launch_validation() {
    let root = TestRoot::new("shell-disappears");
    let shell = root.path().join("fixture-shell");
    create_executable(&shell, b"#!/bin/sh\nexit 0\n");
    let profile = profile_for_candidate(&shell);
    validate_shell_profile_for_launch(&profile).unwrap();

    fs::remove_file(&shell).unwrap();
    let error = validate_shell_profile_for_launch(&profile).unwrap_err();
    assert!(error.to_string().contains("no longer usable"));
}

#[cfg(unix)]
#[test]
fn t059_unavailable_interpreter_start_failure_is_observed_as_final_exit() {
    let root = TestRoot::new("pty-start-failure");
    let marker = root.path().join("should-not-run");
    let shell = root.path().join("broken-shell");
    create_executable(
        &shell,
        format!(
            "#!/winds-t059-definitely-missing-interpreter\ntouch '{}'\n",
            marker.display()
        )
        .as_bytes(),
    );
    let profile = profile_for_candidate(&shell);
    validate_shell_profile_for_launch(&profile).unwrap();

    let mut session =
        TerminalSession::start(&profile, root.path(), TerminalSize { rows: 24, cols: 80 }).unwrap();
    let exit = session.wait().unwrap();
    assert_ne!(exit.exit_code, 0);
    assert_eq!(session.try_wait().unwrap(), Some(exit));
    assert!(!marker.exists());
}

#[cfg(unix)]
#[test]
fn t059_immediate_terminal_exit_is_observed_once_and_stays_final() {
    let root = TestRoot::new("immediate-exit");
    let shell = root.path().join("exit-shell");
    create_executable(&shell, b"#!/bin/sh\nexit 23\n");
    let profile = profile_for_candidate(&shell);

    let mut session =
        TerminalSession::start(&profile, root.path(), TerminalSize { rows: 24, cols: 80 }).unwrap();
    let exit = session.wait().unwrap();
    assert_eq!(exit.exit_code, 23);
    assert_eq!(session.try_wait().unwrap(), Some(exit));
}

#[cfg(unix)]
fn state_with_terminal_execution(root: &TestRoot, execution_id: &str) -> PathBuf {
    let state_root = root.path().join("history-state");
    let workspace_root = root.path().join("history-workspace");
    fs::create_dir(&workspace_root).unwrap();
    let workspace_root = workspace_root.canonicalize().unwrap();
    let mut store = Store::open(&state_root).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-t059-history",
                canonical_worktree_root: workspace_root.to_str().unwrap(),
                git_common_dir: workspace_root.join(".git").to_str().unwrap(),
            },
            1,
        )
        .unwrap();
    let arguments: Vec<String> = Vec::new();
    store
        .create_terminal_execution(
            NewExecution {
                execution_id,
                workspace_id: "workspace-t059-history",
                kind: ExecutionKind::Terminal,
                request_source: FactSource::CallerRequested,
                execution_domain: "native-t059",
            },
            NewTerminalSession {
                execution_id,
                profile_id: "t059-history-profile",
                shell_executable: "/t059-history-shell",
                shell_arguments: &arguments,
                requested_cwd: workspace_root.to_str().unwrap(),
                initial_cols: Some(80),
                initial_rows: Some(24),
            },
            2,
        )
        .unwrap();
    drop(store);
    state_root.canonicalize().unwrap()
}

#[cfg(unix)]
#[test]
fn t059_huge_terminal_output_is_fully_observed_but_retention_stays_bounded() {
    const OUTPUT_BYTES: usize = 1024 * 1024;
    const TRANSCRIPT_QUOTA: usize = 4 * 1024;
    const TOTAL_QUOTA: u64 = 64 * 1024;

    let root = TestRoot::new("huge-output");
    let execution_id = "t059-huge-output";
    let state_root = state_with_terminal_execution(&root, execution_id);
    let policy = SessionHistoryPolicy::local_bounded(false, TRANSCRIPT_QUOTA, TOTAL_QUOTA).unwrap();
    let recorder = SessionHistoryRecorder::new_local(execution_id, policy, &state_root).unwrap();
    let mut reader = recorder
        .wrap_output_reader(Box::new(Cursor::new(vec![b'x'; OUTPUT_BYTES])))
        .unwrap();
    let copied = io::copy(&mut reader, &mut io::sink()).unwrap();
    assert_eq!(copied, u64::try_from(OUTPUT_BYTES).unwrap());
    drop(reader);

    let persisted = recorder.persist().unwrap().unwrap();
    assert_eq!(
        persisted.manifest.transcript_observed_bytes,
        u64::try_from(OUTPUT_BYTES).unwrap()
    );
    assert_eq!(
        persisted.manifest.transcript_retained_bytes,
        TRANSCRIPT_QUOTA
    );
    assert!(persisted.manifest.transcript_capture_complete);
    assert!(persisted.manifest.transcript_truncated);

    let transcript =
        fs::read(state_root.join(&persisted.manifest.transcript.relative_path)).unwrap();
    assert_eq!(transcript.len(), TRANSCRIPT_QUOTA);
    assert!(transcript.iter().all(|byte| *byte == b'x'));
    let manifest_bytes = fs::metadata(state_root.join(&persisted.manifest_blob.relative_path))
        .unwrap()
        .len();
    let transcript_bytes = u64::try_from(transcript.len()).unwrap();
    assert!(transcript_bytes + manifest_bytes <= TOTAL_QUOTA);
}

#[test]
fn t059_environment_manifests_are_inventory_only_and_never_auto_executed() {
    let root = TestRoot::new("manifest-nonexecution");
    let worktree = root.path().join("workspace");
    let common_dir = root.path().join("git-common");
    fs::create_dir(&worktree).unwrap();
    fs::create_dir(&common_dir).unwrap();
    let worktree = worktree.canonicalize().unwrap();
    let common_dir = common_dir.canonicalize().unwrap();
    let marker = root.path().join("manifest-ran");
    let secret = "T059_MANIFEST_SECRET_MUST_NOT_PERSIST";

    fs::write(
        worktree.join(".envrc"),
        format!("export TOKEN={secret}\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    fs::write(worktree.join(".mise.toml"), format!("# {secret}\n")).unwrap();
    fs::create_dir(worktree.join(".devcontainer")).unwrap();
    fs::write(
        worktree.join(".devcontainer/devcontainer.json"),
        format!("{{\"secret\":\"{secret}\"}}\n"),
    )
    .unwrap();

    let workspace = WorkspaceInspection {
        workspace_id: "workspace-t059-inventory".to_owned(),
        canonical_worktree_root: worktree.to_str().unwrap().to_owned(),
        git_common_dir: common_dir.to_str().unwrap().to_owned(),
        head_oid: None,
        branch: Some("main".to_owned()),
        detached: false,
        dirty: false,
    };
    let inventory = inventory_workspace_environment(&workspace).unwrap();

    assert_eq!(
        inventory.detected_manifests,
        vec![
            ".devcontainer/devcontainer.json".to_owned(),
            ".envrc".to_owned(),
            ".mise.toml".to_owned(),
        ]
    );
    assert!(!marker.exists());
    let json = serde_json::to_string(&inventory).unwrap();
    assert!(!json.contains(secret));
    assert!(!json.contains("TOKEN="));
}
