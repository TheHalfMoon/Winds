use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
use crate::store::{NewExecution, NewShellCommand, NewTerminalSession, NewWorkspace, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

struct TestHome(PathBuf);

impl TestHome {
    fn new(name: &str) -> Self {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t068-store-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn store_with_workspace(home: &TestHome) -> Store {
    let store = Store::open(home.path()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t068-workspace",
                git_common_dir: "/tmp/t068-workspace/.git",
            },
            90,
        )
        .unwrap();
    store
}

fn create_shell_command(store: &mut Store, execution_id: &str, requested_unix_ms: i64) {
    let arguments = Vec::new();
    store
        .create_shell_command_execution(
            NewExecution {
                execution_id,
                workspace_id: "workspace-1",
                kind: ExecutionKind::ShellCommand,
                request_source: FactSource::CallerRequested,
                execution_domain: "host-test",
            },
            NewShellCommand {
                execution_id,
                executable: "test-shell",
                arguments: &arguments,
                command_source: FactSource::CallerRequested,
                requested_cwd: "/tmp/t068-workspace",
                cwd_source: FactSource::CallerRequested,
            },
            requested_unix_ms,
        )
        .unwrap();
}

#[test]
fn shell_command_exit_requires_a_durable_observed_fact() {
    let home = TestHome::new("exit-fact");
    let mut store = store_with_workspace(&home);
    create_shell_command(&mut store, "command-1", 100);
    store
        .mark_shell_command_running("command-1", Some(110))
        .unwrap();

    let error = store
        .record_shell_command_exit_observation("command-1", None, None)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires an exit code or observed end time")
    );
    assert_eq!(
        store.load_execution("command-1").unwrap().status,
        ExecutionStatus::Running
    );
    assert_eq!(
        store.load_shell_command("command-1").unwrap().exit_source,
        None
    );
    assert!(
        store
            .finalize_shell_command_from_observation("command-1")
            .is_err()
    );

    store
        .record_shell_command_exit_observation("command-1", Some(0), None)
        .unwrap();
    store
        .finalize_shell_command_from_observation("command-1")
        .unwrap();
    let execution = store.load_execution("command-1").unwrap();
    assert_eq!(execution.status, ExecutionStatus::Exited);
    assert_eq!(execution.ended_unix_ms, None);
    assert_eq!(execution.duration_ms, None);
}

#[test]
fn restart_reconciliation_never_records_events_before_request_time() {
    let home = TestHome::new("restart-clock");
    let mut store = store_with_workspace(&home);
    create_shell_command(&mut store, "command-1", 100);

    let shell_arguments = Vec::new();
    store
        .create_terminal_execution(
            NewExecution {
                execution_id: "terminal-1",
                workspace_id: "workspace-1",
                kind: ExecutionKind::Terminal,
                request_source: FactSource::CallerRequested,
                execution_domain: "host-test",
            },
            NewTerminalSession {
                execution_id: "terminal-1",
                profile_id: "profile-1",
                shell_executable: "/bin/sh",
                shell_arguments: &shell_arguments,
                requested_cwd: "/tmp/t068-workspace",
                initial_cols: Some(80),
                initial_rows: Some(24),
            },
            110,
        )
        .unwrap();

    assert_eq!(
        store
            .reconcile_unowned_shell_commands_after_restart(50)
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .reconcile_unowned_terminal_sessions_after_restart(50)
            .unwrap(),
        1
    );

    let shell_event = store
        .execution_events("command-1")
        .unwrap()
        .into_iter()
        .find(|event| event.kind == "ShellCommandOwnershipLostAfterRestart")
        .unwrap();
    assert_eq!(shell_event.created_unix_ms, 100);

    let terminal_event = store
        .execution_events("terminal-1")
        .unwrap()
        .into_iter()
        .find(|event| event.kind == "TerminalOwnershipLostAfterRestart")
        .unwrap();
    assert_eq!(terminal_event.created_unix_ms, 110);
}

#[test]
fn terminal_session_child_requires_terminal_execution_kind() {
    let home = TestHome::new("terminal-kind");
    let mut store = store_with_workspace(&home);
    create_shell_command(&mut store, "command-1", 100);

    let shell_arguments = Vec::new();
    let error = store
        .create_terminal_session(NewTerminalSession {
            execution_id: "command-1",
            profile_id: "profile-1",
            shell_executable: "/bin/sh",
            shell_arguments: &shell_arguments,
            requested_cwd: "/tmp/t068-workspace",
            initial_cols: Some(80),
            initial_rows: Some(24),
        })
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires TERMINAL execution kind")
    );
    assert!(store.load_terminal_session("command-1").is_err());
}
