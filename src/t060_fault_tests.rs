use crate::command::history::{SessionHistoryPolicy, SessionHistoryRecorder};
use crate::command::{ExplicitCommandRequest, run_explicit_command};
use crate::domain::{ExecutionKind, ExecutionStatus, FactSource, TerminalCloseReason};
use crate::execution::TerminalExecution;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::store::{NewExecution, NewTerminalSession, NewWorkspace, Store};
use rusqlite::Connection;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const DEFAULT_SIZE: TerminalSize = TerminalSize { rows: 24, cols: 80 };

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(name: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let temp_root = Path::new("/tmp")
            .canonicalize()
            .expect("supported Unix test hosts must provide canonical /tmp");
        let path = temp_root.join(format!(
            "winds-t060-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(fs::canonicalize(path).unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn workspace(&self) -> PathBuf {
        self.0.join("workspace")
    }

    fn state(&self) -> PathBuf {
        self.0.join("state")
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let canonical_temp = match Path::new("/tmp").canonicalize() {
            Ok(path) => path,
            Err(_) => return,
        };
        let canonical_root = match self.0.canonicalize() {
            Ok(path) => path,
            Err(_) => return,
        };
        let owned_name = canonical_root
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("winds-t060-"));
        if owned_name && canonical_root.parent() == Some(canonical_temp.as_path()) {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
}

fn fixture_utf8_path(path: &Path, label: &str) -> String {
    match path.to_str() {
        Some(value) => value.to_owned(),
        None => panic!(
            "{label} must be UTF-8 under the canonical T060 /tmp fixture root: {}",
            path.display()
        ),
    }
}

fn store_with_workspace(root: &TestRoot) -> Store {
    fs::create_dir(root.workspace()).unwrap();
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let git_common_dir = workspace.join(".git");
    let workspace_text = fixture_utf8_path(&workspace, "workspace path");
    let git_common_text = fixture_utf8_path(&git_common_dir, "git common path");
    let store = Store::open(&root.state()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: &workspace_text,
                git_common_dir: &git_common_text,
            },
            1,
        )
        .unwrap();
    store
}

fn fault_connection(root: &TestRoot) -> Connection {
    Connection::open(root.state().join("winds.db")).unwrap()
}

fn native_sh_profile() -> ShellProfile {
    profile_for_candidate(Path::new("/bin/sh"))
}

fn profile_for_candidate(executable: &Path) -> ShellProfile {
    let executable_text = fixture_utf8_path(executable, "shell candidate");
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: "/unused/worktree".to_owned(),
        git_common_dir: "/unused/git-common".to_owned(),
        shell_candidates: vec![executable_text.clone()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| profile.executable == executable_text)
        .unwrap_or_else(|| panic!("candidate was not discoverable: {}", executable.display()))
}

fn create_typed_terminal_request(store: &mut Store, execution_id: &str, requested_unix_ms: i64) {
    let arguments = Vec::new();
    store
        .create_terminal_execution(
            NewExecution {
                execution_id,
                workspace_id: "workspace-1",
                kind: ExecutionKind::Terminal,
                request_source: FactSource::CallerRequested,
                execution_domain: "t060-test-domain",
            },
            NewTerminalSession {
                execution_id,
                profile_id: "t060-profile",
                shell_executable: "/bin/sh",
                shell_arguments: &arguments,
                requested_cwd: "/tmp/t060",
                initial_cols: Some(80),
                initial_rows: Some(24),
            },
            requested_unix_ms,
        )
        .unwrap();
}

fn wait_for_file(path: &Path, description: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {description}: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(5));
    }
}

fn write_executable(path: &Path, bytes: &[u8]) {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o700);
    let mut file = options.open(path).unwrap();
    use std::io::Write as _;
    file.write_all(bytes).unwrap();
    let mut permissions = file.metadata().unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child }
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Ok(Some(_)) = self.child.try_wait() {
            return;
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn input_and_resize_racing_with_exit_never_reopen_final_session() {
    let root = TestRoot::new("exit-race");
    let mut store = store_with_workspace(&root);
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let profile = native_sh_profile();
    let mut execution = TerminalExecution::start_native(
        &mut store,
        "t060-exit-race",
        "workspace-1",
        &profile,
        &workspace,
        DEFAULT_SIZE,
    )
    .unwrap();

    assert_eq!(execution.try_wait().unwrap(), None);
    execution
        .resize(TerminalSize { rows: 25, cols: 81 })
        .unwrap();
    execution.send_input(b"exit 0\n").unwrap();
    let _ = execution.resize(TerminalSize { rows: 26, cols: 82 });

    let deadline = Instant::now() + Duration::from_secs(5);
    let final_exit = loop {
        if let Some(exit) = execution.try_wait().unwrap() {
            break exit;
        }
        assert!(
            Instant::now() < deadline,
            "terminal did not exit inside race fixture deadline"
        );
        thread::sleep(Duration::from_millis(5));
    };

    assert_eq!(final_exit.exit_code, 0);
    assert!(execution.send_input(b"echo impossible\n").is_err());
    assert!(
        execution
            .resize(TerminalSize { rows: 30, cols: 90 })
            .is_err()
    );
    assert_eq!(execution.try_wait().unwrap(), Some(final_exit));
    drop(execution);

    let record = store.load_execution("t060-exit-race").unwrap();
    assert_eq!(record.status, ExecutionStatus::Exited);
    assert_eq!(record.status_source, FactSource::WindsObserved);
    assert!(record.ended_unix_ms.is_some());
}

#[test]
fn interrupt_then_close_escalates_only_while_session_is_still_owned() {
    let root = TestRoot::new("interrupt-close");
    let mut store = store_with_workspace(&root);
    let profile = native_sh_profile();
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let ready_marker = workspace.join(".t060-interrupt-ready");
    let reset_marker = workspace.join(".t060-interrupt-reset");
    let interrupt_child = workspace.join("t060-interrupt-child.sh");
    write_executable(
        &interrupt_child,
        b"#!/bin/sh\ntrap 'exit 130' INT\n: > .t060-interrupt-ready\nwhile :; do sleep 1; done\n",
    );
    let mut execution = TerminalExecution::start_native(
        &mut store,
        "t060-interrupt-close",
        "workspace-1",
        &profile,
        &workspace,
        DEFAULT_SIZE,
    )
    .unwrap();

    execution
        .send_input(b"sh ./t060-interrupt-child.sh\n")
        .unwrap();
    wait_for_file(&ready_marker, "interrupt readiness marker");

    execution.interrupt().unwrap();
    execution
        .send_input(b": > .t060-interrupt-reset\n")
        .unwrap();
    wait_for_file(&reset_marker, "post-interrupt shell-resume marker");
    assert_eq!(execution.try_wait().unwrap(), None);

    let close_started = Instant::now();
    let close_result = execution.close();
    assert!(
        close_started.elapsed() < Duration::from_secs(2),
        "explicit terminal close must remain bounded"
    );
    let close_proven = match close_result {
        Ok(_) => true,
        Err(error) => {
            assert!(
                error
                    .to_string()
                    .contains("could not prove owned child exit inside bounded cleanup window"),
                "unexpected explicit close error: {error}"
            );
            false
        }
    };
    drop(execution);

    let record = store.load_execution("t060-interrupt-close").unwrap();
    assert_eq!(record.status_source, FactSource::WindsObserved);
    let terminal = store.load_terminal_session("t060-interrupt-close").unwrap();
    if close_proven {
        assert_eq!(record.status, ExecutionStatus::Interrupted);
        assert!(record.ended_unix_ms.is_some());
        assert!(record.duration_ms.is_some());
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::ClosedByWinds)
        );
    } else {
        assert_eq!(record.status, ExecutionStatus::OwnershipLost);
        assert_eq!(record.ended_unix_ms, None);
        assert_eq!(record.duration_ms, None);
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::OwnershipLostProcessStateUnknown)
        );
        assert!(
            store
                .execution_events("t060-interrupt-close")
                .unwrap()
                .iter()
                .any(|event| {
                    event.kind == "TerminalOwnershipLostAfterCleanupFailure"
                        && event.source == FactSource::WindsObserved
                })
        );
    }
}

struct ReadThenFail {
    delivered: bool,
}

impl Read for ReadThenFail {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if !self.delivered {
            self.delivered = true;
            let bytes = b"abc";
            let count = bytes.len().min(buffer.len());
            buffer[..count].copy_from_slice(&bytes[..count]);
            return Ok(count);
        }
        Err(io::Error::other("t060 injected reader failure"))
    }
}

#[test]
fn reader_error_persists_incomplete_not_false_complete_transcript() {
    let root = TestRoot::new("reader-error");
    let mut store = store_with_workspace(&root);
    create_typed_terminal_request(&mut store, "t060-reader-error", 10);
    let policy = SessionHistoryPolicy::local_bounded(false, 64, 8 * 1024).unwrap();
    let recorder =
        SessionHistoryRecorder::new_local("t060-reader-error", policy, &root.state()).unwrap();
    let mut reader = recorder
        .wrap_output_reader(Box::new(ReadThenFail { delivered: false }))
        .unwrap();
    let mut buffer = [0_u8; 8];
    assert_eq!(reader.read(&mut buffer).unwrap(), 3);
    assert_eq!(&buffer[..3], b"abc");
    assert!(reader.read(&mut buffer).is_err());
    drop(reader);

    let persisted = recorder.persist().unwrap().unwrap();
    assert_eq!(persisted.manifest.transcript_observed_bytes, 3);
    assert_eq!(persisted.manifest.transcript_retained_bytes, 3);
    assert!(!persisted.manifest.transcript_capture_complete);
    assert!(persisted.manifest.transcript_truncated);
}

#[test]
fn sqlite_failure_before_terminal_spawn_rolls_back_request_and_never_runs_shell() {
    let root = TestRoot::new("sqlite-before-spawn");
    let mut store = store_with_workspace(&root);
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let marker = root.path().join("spawned.marker");
    let wrapper = workspace.join("t060-shell-wrapper");
    let script = format!(
        "#!/bin/sh\nprintf spawned > '{}'\nexec /bin/sh \"$@\"\n",
        marker.display()
    );
    write_executable(&wrapper, script.as_bytes());
    let profile = profile_for_candidate(&wrapper);

    let injector = fault_connection(&root);
    injector
        .execute_batch(
            "CREATE TRIGGER t060_fail_terminal_insert
             BEFORE INSERT ON executions
             WHEN NEW.kind = 'TERMINAL' AND NEW.execution_id = 't060-before-spawn'
             BEGIN
                 SELECT RAISE(ABORT, 't060 forced request persistence failure');
             END;",
        )
        .unwrap();

    let result = TerminalExecution::start_native(
        &mut store,
        "t060-before-spawn",
        "workspace-1",
        &profile,
        &workspace,
        DEFAULT_SIZE,
    );
    assert!(result.is_err());
    drop(result);
    assert!(
        !marker.exists(),
        "child wrapper ran despite pre-spawn SQLite failure"
    );
    assert!(store.load_execution("t060-before-spawn").is_err());
}

#[test]
fn sqlite_running_failure_after_child_spawn_is_repaired_to_interrupted() {
    let root = TestRoot::new("sqlite-after-spawn");
    let mut store = store_with_workspace(&root);
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let profile = native_sh_profile();
    let injector = fault_connection(&root);
    injector
        .execute_batch(
            "CREATE TRIGGER t060_fail_terminal_running
             BEFORE UPDATE OF status ON executions
             WHEN NEW.execution_id = 't060-after-spawn' AND NEW.status = 'RUNNING'
             BEGIN
                 SELECT RAISE(ABORT, 't060 forced RUNNING persistence failure');
             END;",
        )
        .unwrap();

    let result = TerminalExecution::start_native(
        &mut store,
        "t060-after-spawn",
        "workspace-1",
        &profile,
        &workspace,
        DEFAULT_SIZE,
    );
    let error = result
        .err()
        .expect("RUNNING persistence failure must surface");
    assert!(
        error
            .to_string()
            .contains("child started but RUNNING persistence failed")
    );

    let record = store.load_execution("t060-after-spawn").unwrap();
    assert_eq!(record.status, ExecutionStatus::Interrupted);
    assert_eq!(record.status_source, FactSource::WindsObserved);
    assert!(record.started_unix_ms.is_some());
    assert!(record.ended_unix_ms.is_some());
    assert_eq!(
        store
            .load_terminal_session("t060-after-spawn")
            .unwrap()
            .close_reason,
        Some(TerminalCloseReason::StartPersistenceFailed)
    );
}

#[test]
fn sqlite_exit_finalization_failure_is_deferred_then_retried_without_false_success() {
    let root = TestRoot::new("sqlite-finalize");
    let mut store = store_with_workspace(&root);
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let wrapper = workspace.join("t060-finalize-child");
    write_executable(&wrapper, b"#!/bin/sh\nsleep 1\nexit 0\n");
    let profile = profile_for_candidate(&wrapper);
    let injector = fault_connection(&root);
    injector
        .execute_batch(
            "CREATE TRIGGER t060_fail_terminal_exit
             BEFORE UPDATE OF status ON executions
             WHEN NEW.execution_id = 't060-finalize' AND NEW.status = 'EXITED'
             BEGIN
                 SELECT RAISE(ABORT, 't060 forced EXITED persistence failure');
             END;",
        )
        .unwrap();

    let mut execution = TerminalExecution::start_native(
        &mut store,
        "t060-finalize",
        "workspace-1",
        &profile,
        &workspace,
        DEFAULT_SIZE,
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match execution.try_wait() {
            Ok(None) => {}
            Ok(Some(exit)) => panic!(
                "durability failure falsely reported terminal success with exit {:?}",
                exit
            ),
            Err(error) => {
                assert!(
                    error
                        .to_string()
                        .contains("t060 forced EXITED persistence failure"),
                    "unexpected try_wait error instead of the injected finalization failure: {error}"
                );
                break;
            }
        }
        assert!(
            Instant::now() < deadline,
            "terminal did not reach the injected finalization failure inside fixture deadline"
        );
        thread::sleep(Duration::from_millis(5));
    }
    drop(execution);

    assert_eq!(store.pending_terminal_finalization_count(), 1);
    assert_eq!(
        store.load_execution("t060-finalize").unwrap().status,
        ExecutionStatus::Running
    );
    injector
        .execute_batch("DROP TRIGGER t060_fail_terminal_exit;")
        .unwrap();
    assert_eq!(store.retry_deferred_terminal_finalizations().unwrap(), 1);
    assert_eq!(store.pending_terminal_finalization_count(), 0);
    let record = store.load_execution("t060-finalize").unwrap();
    assert_eq!(record.status, ExecutionStatus::Exited);
    assert_eq!(record.status_source, FactSource::WindsObserved);
    assert_eq!(
        store
            .load_terminal_session("t060-finalize")
            .unwrap()
            .close_reason,
        Some(TerminalCloseReason::ProcessExited)
    );
}

#[test]
fn restart_reconciles_partial_terminal_rows_without_fabricating_typed_session() {
    let root = TestRoot::new("partial-persistence");
    let mut store = store_with_workspace(&root);
    for execution_id in ["t060-partial-requested", "t060-partial-running"] {
        store
            .create_execution(
                NewExecution {
                    execution_id,
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "t060-partial-domain",
                },
                100,
            )
            .unwrap();
    }
    fault_connection(&root)
        .execute(
            "UPDATE executions
             SET status = 'RUNNING', status_source = 'WINDS_OBSERVED', started_unix_ms = 110
             WHERE execution_id = 't060-partial-running'",
            [],
        )
        .unwrap();

    assert_eq!(
        store
            .reconcile_unowned_terminal_sessions_after_restart(200)
            .unwrap(),
        2
    );
    for execution_id in ["t060-partial-requested", "t060-partial-running"] {
        let record = store.load_execution(execution_id).unwrap();
        assert_eq!(record.status, ExecutionStatus::OwnershipLost);
        assert_eq!(record.status_source, FactSource::WindsObserved);
        assert_eq!(record.ended_unix_ms, None);
        assert_eq!(record.duration_ms, None);
        assert!(store.load_terminal_session(execution_id).is_err());
        assert!(
            store
                .execution_events(execution_id)
                .unwrap()
                .iter()
                .any(|event| {
                    event.kind == "TerminalOwnershipLostAfterRestart"
                        && event.source == FactSource::WindsObserved
                })
        );
    }
}

#[test]
fn stale_pid_reuse_fixture_never_signals_unrelated_live_process() {
    let root = TestRoot::new("pid-reuse");
    let mut unrelated = ChildGuard::new(
        Command::new("/bin/sh")
            .arg("-c")
            .arg("exec sleep 30")
            .spawn()
            .unwrap(),
    );
    let unrelated_pid = unrelated.id();

    let state = root.state();
    {
        let mut store = store_with_workspace(&root);
        let arguments = vec![format!("stale-pid-shaped-metadata={unrelated_pid}")];
        store
            .create_terminal_execution(
                NewExecution {
                    execution_id: "t060-stale-pid",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::Terminal,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "t060-stale-pid-domain",
                },
                NewTerminalSession {
                    execution_id: "t060-stale-pid",
                    profile_id: "t060-stale-pid-profile",
                    shell_executable: "/bin/sh",
                    shell_arguments: &arguments,
                    requested_cwd: "/tmp/t060",
                    initial_cols: Some(80),
                    initial_rows: Some(24),
                },
                100,
            )
            .unwrap();
        store.mark_terminal_running("t060-stale-pid", 110).unwrap();
    }

    let mut reopened = Store::open(&state).unwrap();
    assert_eq!(
        reopened
            .reconcile_unowned_terminal_sessions_after_restart(200)
            .unwrap(),
        1
    );
    assert_eq!(
        reopened.load_execution("t060-stale-pid").unwrap().status,
        ExecutionStatus::OwnershipLost
    );
    assert_eq!(
        unrelated.try_wait().unwrap(),
        None,
        "restart reconciliation signaled an unrelated live PID"
    );

    let column_names = {
        let connection = fault_connection(&root);
        let mut statement = connection
            .prepare("PRAGMA table_info(terminal_sessions)")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    assert!(
        !column_names
            .iter()
            .any(|name| name.to_ascii_lowercase().contains("pid"))
    );
}

#[test]
fn marker_like_child_output_never_becomes_shell_reported_or_extra_command_authority() {
    let root = TestRoot::new("marker-spoof");
    let mut store = store_with_workspace(&root);
    let workspace = fs::canonicalize(root.workspace()).unwrap();
    let executable = PathBuf::from("/bin/sh");
    let arguments = vec![
        "-c".to_owned(),
        "printf '__WINDS_COMMAND_END_spoof__\\n'; exit 0".to_owned(),
    ];

    let result = run_explicit_command(
        &mut store,
        ExplicitCommandRequest {
            execution_id: "t060-marker-spoof",
            workspace_id: "workspace-1",
            executable: &executable,
            arguments: &arguments,
            cwd: &workspace,
        },
    )
    .unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert_eq!(
        store
            .shell_command_count_for_workspace("workspace-1")
            .unwrap(),
        1
    );
    let command = store.load_shell_command("t060-marker-spoof").unwrap();
    assert_eq!(command.command_source, FactSource::CallerRequested);
    assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
    assert!(
        store
            .execution_events("t060-marker-spoof")
            .unwrap()
            .iter()
            .all(|event| event.source != FactSource::ShellReported)
    );
}
