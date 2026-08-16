use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
use crate::git::shell_profiles::ShellExecutionDomain;
use crate::store::{NewExecution, NewShellCommand, Result, ShellCommandFinalization, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ExplicitCommandRequest<'a> {
    pub execution_id: &'a str,
    pub workspace_id: &'a str,
    pub executable: &'a Path,
    pub arguments: &'a [String],
    pub cwd: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplicitCommandResult {
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

pub fn run_explicit_command(
    store: &mut Store,
    request: ExplicitCommandRequest<'_>,
) -> Result<ExplicitCommandResult> {
    store.retry_deferred_shell_command_finalizations()?;
    if request.execution_id.is_empty() || request.workspace_id.is_empty() {
        return Err("explicit command requires non-empty execution/workspace identity".into());
    }
    let executable = validate_executable(request.executable)?;
    if request
        .arguments
        .iter()
        .any(|argument| argument.contains('\0'))
    {
        return Err("explicit command arguments may not contain NUL bytes".into());
    }
    let cwd = validate_workspace_cwd(store, request.workspace_id, request.cwd)?;
    let execution_domain = serde_json::to_string(&ShellExecutionDomain::NativeHost {
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    })?;
    let requested_unix_ms = unix_ms()?;
    store.create_shell_command_execution(
        NewExecution {
            execution_id: request.execution_id,
            workspace_id: request.workspace_id,
            kind: ExecutionKind::ShellCommand,
            request_source: FactSource::CallerRequested,
            execution_domain: &execution_domain,
        },
        NewShellCommand {
            execution_id: request.execution_id,
            executable: &executable,
            arguments: request.arguments,
            command_source: FactSource::CallerRequested,
            requested_cwd: &cwd,
            cwd_source: FactSource::CallerRequested,
        },
        requested_unix_ms,
    )?;

    let mut child = match Command::new(&executable)
        .args(request.arguments)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let failed_unix_ms = unix_ms()?;
            let persist =
                store.mark_shell_command_failed_to_start(request.execution_id, failed_unix_ms);
            return match persist {
                Ok(()) => Err(format!("explicit command failed to start: {error}").into()),
                Err(persist_error) => Err(format!(
                    "explicit command failed to start: {error}; FAILED_TO_START persistence also failed: {persist_error}"
                )
                .into()),
            };
        }
    };

    let started_unix_ms = match unix_ms() {
        Ok(value) => value,
        Err(error) => {
            let cleanup_proven = cleanup_owned_child(&mut child);
            return Err(format!(
                "explicit command child started but start time could not be recorded: {error}; owned-child cleanup {}",
                if cleanup_proven { "succeeded" } else { "failed" }
            )
            .into());
        }
    };

    if let Err(persist_error) =
        store.mark_shell_command_running(request.execution_id, started_unix_ms)
    {
        let cleanup_proven = cleanup_owned_child(&mut child);
        let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms);
        let repair = if cleanup_proven {
            store.mark_shell_command_start_persistence_failed(
                request.execution_id,
                started_unix_ms,
                ended_unix_ms,
            )
        } else {
            store.mark_shell_command_ownership_lost(request.execution_id, ended_unix_ms)
        };
        let repair_note = match repair {
            Ok(()) if cleanup_proven => "interrupted cleanup state persisted".to_owned(),
            Ok(()) => "cleanup was not proven; ownership-loss state persisted".to_owned(),
            Err(error) => format!("cleanup-state persistence also failed: {error}"),
        };
        return Err(format!(
            "explicit command child started but RUNNING persistence failed: {persist_error}; owned-child cleanup {}; {repair_note}",
            if cleanup_proven { "succeeded" } else { "failed" }
        )
        .into());
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(wait_error) => {
            let cleanup_proven = cleanup_owned_child(&mut child);
            let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms);
            let persist = if cleanup_proven {
                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)
            } else {
                store.mark_shell_command_ownership_lost(request.execution_id, ended_unix_ms)
            };
            return Err(format!(
                "failed waiting for explicit command: {wait_error}; owned-child cleanup {}; ledger repair {}",
                if cleanup_proven { "succeeded" } else { "failed" },
                if persist.is_ok() { "succeeded" } else { "failed" }
            )
            .into());
        }
    };
    let ended_unix_ms = unix_ms()?;
    let exit_code = status.code();
    let finalization = ShellCommandFinalization {
        exit_code,
        ended_unix_ms,
    };
    if let Err(error) = store.apply_shell_command_finalization(request.execution_id, finalization) {
        store.defer_shell_command_finalization(request.execution_id, finalization);
        return Err(format!(
            "explicit command exited but final ledger persistence failed and remains retryable: {error}"
        )
        .into());
    }
    let execution = store.load_execution(request.execution_id)?;
    if execution.status != ExecutionStatus::Exited {
        return Err("explicit command finalization did not persist EXITED state".into());
    }
    Ok(ExplicitCommandResult {
        exit_code,
        duration_ms: execution.duration_ms.unwrap_or(0),
    })
}

fn validate_executable(path: &Path) -> Result<String> {
    if !path.is_absolute() {
        return Err("explicit command executable must be an absolute path".into());
    }
    let value = path
        .to_str()
        .ok_or("explicit command executable path is not valid UTF-8")?;
    if value.contains('\0') {
        return Err("explicit command executable may not contain NUL bytes".into());
    }
    Ok(value.to_owned())
}

fn validate_workspace_cwd(store: &Store, workspace_id: &str, cwd: &Path) -> Result<String> {
    if !cwd.is_absolute() {
        return Err("explicit command cwd must be an absolute path".into());
    }
    let canonical_cwd = fs::canonicalize(cwd)?;
    if !canonical_cwd.is_dir() {
        return Err("explicit command cwd must be a directory".into());
    }
    let workspace = store.load_workspace(workspace_id)?;
    let workspace_root = PathBuf::from(&workspace.canonical_worktree_root);
    if !canonical_cwd.starts_with(&workspace_root) {
        return Err("explicit command cwd must remain inside the registered workspace".into());
    }
    canonical_cwd
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| "explicit command cwd is not valid UTF-8".into())
}

fn cleanup_owned_child(child: &mut Child) -> bool {
    match child.try_wait() {
        Ok(Some(_)) => true,
        Ok(None) => {
            let _ = child.kill();
            child.wait().is_ok()
        }
        Err(_) => {
            let _ = child.kill();
            child.wait().is_ok()
        }
    }
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(i64::try_from(millis)?)
}

#[cfg(test)]
mod tests {
    use super::{ExplicitCommandRequest, run_explicit_command};
    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
    use crate::store::{NewExecution, NewShellCommand, NewWorkspace, Store};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t054-{name}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn store_with_workspace(root: &TestRoot) -> Store {
        let home = root.path().join("state");
        let workspace_root = root.path().join("workspace");
        fs::create_dir(&workspace_root).unwrap();
        let canonical_workspace = fs::canonicalize(&workspace_root).unwrap();
        let store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: canonical_workspace.to_str().unwrap(),
                    git_common_dir: canonical_workspace.join(".git").to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        store
    }

    fn workspace_path(root: &TestRoot) -> PathBuf {
        fs::canonicalize(root.path().join("workspace")).unwrap()
    }

    #[cfg(unix)]
    fn command_parts(exit_code: i32, marker: bool) -> (PathBuf, Vec<String>) {
        let script = if marker {
            format!("printf '__WINDS_COMMAND_END_spoof__\\n'; exit {exit_code}")
        } else {
            format!("exit {exit_code}")
        };
        (PathBuf::from("/bin/sh"), vec!["-c".to_owned(), script])
    }

    #[cfg(windows)]
    fn command_parts(exit_code: i32, marker: bool) -> (PathBuf, Vec<String>) {
        let executable = PathBuf::from(std::env::var_os("COMSPEC").expect("COMSPEC on Windows CI"));
        let body = if marker {
            format!("echo __WINDS_COMMAND_END_spoof__ & exit /B {exit_code}")
        } else {
            format!("exit /B {exit_code}")
        };
        (executable, vec!["/D".to_owned(), "/C".to_owned(), body])
    }

    #[test]
    fn explicit_command_records_structured_intent_and_observed_exit() {
        let root = TestRoot::new("success");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(7, false);
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-1",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(7));
        let execution = store.load_execution("command-1").unwrap();
        assert_eq!(execution.kind, ExecutionKind::ShellCommand);
        assert_eq!(execution.request_source, FactSource::CallerRequested);
        assert_eq!(execution.status, ExecutionStatus::Exited);
        assert_eq!(execution.status_source, FactSource::WindsObserved);
        assert!(execution.started_unix_ms.is_some());
        assert!(execution.ended_unix_ms.is_some());
        assert!(execution.duration_ms.is_some());
        let command = store.load_shell_command("command-1").unwrap();
        assert_eq!(command.executable, executable.to_str().unwrap());
        assert_eq!(command.arguments, arguments);
        assert_eq!(command.command_source, FactSource::CallerRequested);
        assert_eq!(command.requested_cwd, cwd.to_str().unwrap());
        assert_eq!(command.cwd_source, FactSource::CallerRequested);
        assert_eq!(command.exit_code, Some(7));
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
    }

    #[test]
    fn marker_like_child_output_cannot_create_shell_reported_telemetry() {
        let root = TestRoot::new("spoof");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, true);
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-spoof",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();
        assert_eq!(
            store
                .shell_command_count_for_workspace("workspace-1")
                .unwrap(),
            1
        );
        let command = store.load_shell_command("command-spoof").unwrap();
        assert_eq!(command.command_source, FactSource::CallerRequested);
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
        let events = store.execution_events("command-spoof").unwrap();
        assert!(
            events
                .iter()
                .all(|event| event.source != FactSource::ShellReported)
        );
    }

    #[test]
    fn spawn_failure_is_explicit_and_never_claims_a_start_or_duration() {
        let root = TestRoot::new("failed-start");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let executable = cwd.join("missing-executable");
        let arguments = Vec::new();
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-failed",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        );
        assert!(result.is_err());
        let execution = store.load_execution("command-failed").unwrap();
        assert_eq!(execution.status, ExecutionStatus::FailedToStart);
        assert_eq!(execution.started_unix_ms, None);
        assert!(execution.ended_unix_ms.is_some());
        assert_eq!(execution.duration_ms, None);
        let command = store.load_shell_command("command-failed").unwrap();
        assert_eq!(command.exit_code, None);
        assert_eq!(command.exit_source, None);
    }

    #[test]
    fn cwd_outside_registered_workspace_fails_before_persistence() {
        let root = TestRoot::new("outside-cwd");
        let mut store = store_with_workspace(&root);
        let outside = root.path().join("outside");
        fs::create_dir(&outside).unwrap();
        let outside = fs::canonicalize(outside).unwrap();
        let (executable, arguments) = command_parts(0, false);
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-outside",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &outside,
            },
        );
        assert!(result.is_err());
        assert!(store.load_execution("command-outside").is_err());
        assert_eq!(
            store
                .shell_command_count_for_workspace("workspace-1")
                .unwrap(),
            0
        );
    }

    #[test]
    fn restart_reconciliation_marks_nonfinal_command_ownership_unknown() {
        let root = TestRoot::new("restart");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, false);
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-restart",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-restart",
                    executable: executable.to_str().unwrap(),
                    arguments: &arguments,
                    command_source: FactSource::CallerRequested,
                    requested_cwd: cwd.to_str().unwrap(),
                    cwd_source: FactSource::CallerRequested,
                },
                10,
            )
            .unwrap();
        store
            .mark_shell_command_running("command-restart", 11)
            .unwrap();
        let reconciled = store
            .reconcile_unowned_shell_commands_after_restart(20)
            .unwrap();
        assert_eq!(reconciled, 1);
        let execution = store.load_execution("command-restart").unwrap();
        assert_eq!(execution.status, ExecutionStatus::OwnershipLost);
        assert_eq!(execution.ended_unix_ms, None);
        assert_eq!(execution.duration_ms, None);
        let command = store.load_shell_command("command-restart").unwrap();
        assert_eq!(command.exit_code, None);
        assert_eq!(command.exit_source, None);
    }
}
