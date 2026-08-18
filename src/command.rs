pub(crate) mod history;

use self::history::{SessionHistoryPolicy, persisted_arguments};
use crate::domain::{ExecutionKind, ExecutionStatus, FactSource, WorkspaceRecord};
use crate::git::{observe_worktree_state, shell_profiles::ShellExecutionDomain};
use crate::store::git_observation::{
    GitObservationAvailability, GitObservationBoundary, NewExecutionGitObservation,
};
use crate::store::{NewExecution, NewShellCommand, Result, Store};
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
    pub duration_ms: Option<u64>,
}

pub fn run_explicit_command(
    store: &mut Store,
    request: ExplicitCommandRequest<'_>,
) -> Result<ExplicitCommandResult> {
    run_explicit_command_with_history_policy(
        store,
        request,
        SessionHistoryPolicy::command_history_only(),
    )
}

pub fn run_explicit_command_with_history_policy(
    store: &mut Store,
    request: ExplicitCommandRequest<'_>,
    history_policy: SessionHistoryPolicy,
) -> Result<ExplicitCommandResult> {
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
    let workspace = store.load_workspace(request.workspace_id)?;
    let execution_domain = serde_json::to_string(&ShellExecutionDomain::NativeHost {
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    })?;
    let persisted_arguments = persisted_arguments(request.arguments, history_policy);
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
            arguments: &persisted_arguments,
            command_source: FactSource::CallerRequested,
            requested_cwd: &cwd,
            cwd_source: FactSource::CallerRequested,
        },
        requested_unix_ms,
    )?;

    if let Err(observation_error) = record_git_boundary_observation(
        store,
        request.execution_id,
        &workspace,
        GitObservationBoundary::Before,
    ) {
        let failed_unix_ms = trustworthy_wall_time_after(requested_unix_ms, None);
        let repair = store.mark_shell_command_failed_to_start(request.execution_id, failed_unix_ms);
        return match repair {
            Ok(()) => Err(format!(
                "explicit command was not started because BEFORE Git observation persistence failed: {observation_error}"
            )
            .into()),
            Err(repair_error) => Err(format!(
                "explicit command was not started because BEFORE Git observation persistence failed: {observation_error}; FAILED_TO_START persistence also failed: {repair_error}"
            )
            .into()),
        };
    }

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
            let failed_unix_ms = trustworthy_wall_time_after(requested_unix_ms, None);
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

    let started_unix_ms = trustworthy_wall_time_after(requested_unix_ms, None);
    if let Err(persist_error) =
        store.mark_shell_command_running(request.execution_id, started_unix_ms)
    {
        let cleanup_proven = cleanup_owned_child(&mut child);
        let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);
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
            let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);
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
    let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);
    let exit_code = status.code();
    store
        .record_shell_command_exit_observation(request.execution_id, exit_code, ended_unix_ms)
        .map_err(|error| {
            format!(
                "explicit command exited but its durable exit observation could not be persisted: {error}"
            )
        })?;
    if let Err(error) = store.finalize_shell_command_from_observation(request.execution_id) {
        return Err(format!(
            "explicit command exited and its durable exit observation is persisted, but final execution-state persistence remains pending: {error}"
        )
        .into());
    }
    record_git_boundary_observation(
        store,
        request.execution_id,
        &workspace,
        GitObservationBoundary::After,
    )
    .map_err(|error| {
        format!(
            "explicit command exited and its lifecycle finalization is persisted, but AFTER Git observation persistence failed: {error}"
        )
    })?;
    let execution = store.load_execution(request.execution_id)?;
    if execution.status != ExecutionStatus::Exited {
        return Err("explicit command finalization did not persist EXITED state".into());
    }
    Ok(ExplicitCommandResult {
        exit_code,
        duration_ms: execution.duration_ms,
    })
}

fn record_git_boundary_observation(
    store: &mut Store,
    execution_id: &str,
    workspace: &WorkspaceRecord,
    boundary: GitObservationBoundary,
) -> Result<()> {
    let root = Path::new(&workspace.canonical_worktree_root);
    let common_dir = Path::new(&workspace.git_common_dir);
    let observed_unix_ms = unix_ms().ok();
    match observe_worktree_state(root, common_dir) {
        Ok(observation) => store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id,
            boundary,
            availability: GitObservationAvailability::Observed,
            head_oid: observation.head_oid.as_deref(),
            branch: observation.branch.as_deref(),
            detached: Some(observation.detached),
            dirty: Some(observation.dirty),
            worktree_state_sha256: Some(&observation.worktree_state_sha256),
            observed_unix_ms,
        }),
        Err(_) => store.record_execution_git_observation(NewExecutionGitObservation {
            execution_id,
            boundary,
            availability: GitObservationAvailability::Unavailable,
            head_oid: None,
            branch: None,
            detached: None,
            dirty: None,
            worktree_state_sha256: None,
            observed_unix_ms,
        }),
    }
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

// This validates caller-requested cwd against the current filesystem view. It is not an
// OS sandbox or a hostile concurrent-rename containment primitive.
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

fn trustworthy_wall_time_after(
    requested_unix_ms: i64,
    started_unix_ms: Option<i64>,
) -> Option<i64> {
    non_regressing_wall_time(unix_ms().ok(), requested_unix_ms, started_unix_ms)
}

fn non_regressing_wall_time(
    candidate_unix_ms: Option<i64>,
    requested_unix_ms: i64,
    started_unix_ms: Option<i64>,
) -> Option<i64> {
    candidate_unix_ms.filter(|candidate| {
        *candidate >= requested_unix_ms
            && started_unix_ms.is_none_or(|started| *candidate >= started)
    })
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(i64::try_from(millis)?)
}

#[cfg(test)]
mod tests {
    use super::history::SessionHistoryPolicy;
    use super::{
        ExplicitCommandRequest, non_regressing_wall_time, run_explicit_command,
        run_explicit_command_with_history_policy,
    };
    use crate::domain::{ExecutionKind, ExecutionStatus, FactSource};
    use crate::store::git_observation::{GitObservationAvailability, GitObservationBoundary};
    use crate::store::{NewExecution, NewShellCommand, NewWorkspace, Store};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t056-{name}-{}-{sequence}",
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

    fn store_with_git_workspace(root: &TestRoot) -> (Store, PathBuf) {
        let home = root.path().join("state");
        let workspace_root = root.path().join("workspace-git");
        fs::create_dir(&workspace_root).unwrap();
        run_git(&workspace_root, &["init", "--initial-branch=main"]);
        run_git(&workspace_root, &["config", "user.name", "Winds Test"]);
        run_git(
            &workspace_root,
            &["config", "user.email", "winds@example.invalid"],
        );
        fs::write(workspace_root.join("tracked.txt"), b"initial\n").unwrap();
        run_git(&workspace_root, &["add", "--", "tracked.txt"]);
        run_git(
            &workspace_root,
            &["commit", "--no-gpg-sign", "-m", "initial"],
        );

        let canonical_workspace = fs::canonicalize(&workspace_root).unwrap();
        let common_dir = PathBuf::from(run_git(
            &canonical_workspace,
            &["rev-parse", "--path-format=absolute", "--git-common-dir"],
        ))
        .canonicalize()
        .unwrap();
        let store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-git",
                    canonical_worktree_root: canonical_workspace.to_str().unwrap(),
                    git_common_dir: common_dir.to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        (store, canonical_workspace)
    }

    fn run_git(cwd: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .arg("-c")
            .arg("core.fsmonitor=false")
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

    fn workspace_path(root: &TestRoot) -> PathBuf {
        fs::canonicalize(root.path().join("workspace")).unwrap()
    }

    #[cfg(unix)]
    fn shell_script(script: &str) -> (PathBuf, Vec<String>) {
        (
            PathBuf::from("/bin/sh"),
            vec!["-c".to_owned(), script.to_owned()],
        )
    }

    #[cfg(windows)]
    fn shell_script(script: &str) -> (PathBuf, Vec<String>) {
        let executable = PathBuf::from(std::env::var_os("COMSPEC").expect("COMSPEC on Windows CI"));
        (
            executable,
            vec!["/D".to_owned(), "/C".to_owned(), script.to_owned()],
        )
    }

    fn command_parts(exit_code: i32, marker: bool) -> (PathBuf, Vec<String>) {
        #[cfg(unix)]
        let script = if marker {
            format!("printf '__WINDS_COMMAND_END_spoof__\\n'; exit {exit_code}")
        } else {
            format!("exit {exit_code}")
        };
        #[cfg(windows)]
        let script = if marker {
            format!("echo __WINDS_COMMAND_END_spoof__ & exit /B {exit_code}")
        } else {
            format!("exit /B {exit_code}")
        };
        shell_script(&script)
    }

    #[cfg(unix)]
    fn tracked_mutation_script() -> &'static str {
        "printf 'changed\\n' > tracked.txt"
    }

    #[cfg(windows)]
    fn tracked_mutation_script() -> &'static str {
        "echo changed>tracked.txt"
    }

    #[cfg(unix)]
    fn commit_script() -> &'static str {
        "printf 'committed\\n' >> tracked.txt; git add -- tracked.txt; git commit --no-gpg-sign -m t055 >/dev/null 2>&1"
    }

    #[cfg(windows)]
    fn commit_script() -> &'static str {
        "echo committed>>tracked.txt && git add -- tracked.txt && git commit --no-gpg-sign -m t055 >NUL 2>&1"
    }

    #[cfg(unix)]
    fn branch_switch_script() -> &'static str {
        "git switch -c t055-observed >/dev/null 2>&1"
    }

    #[cfg(windows)]
    fn branch_switch_script() -> &'static str {
        "git switch -c t055-observed >NUL 2>&1"
    }

    #[cfg(unix)]
    fn add_dirty_file_script() -> &'static str {
        "printf 'second\\n' > dirty-b.txt"
    }

    #[cfg(windows)]
    fn add_dirty_file_script() -> &'static str {
        "echo second>dirty-b.txt"
    }

    #[cfg(unix)]
    fn hide_git_metadata_script() -> &'static str {
        "mv .git .git-hidden"
    }

    #[cfg(windows)]
    fn hide_git_metadata_script() -> &'static str {
        "rmdir /s /q .git"
    }

    #[cfg(unix)]
    fn secret_assignment_script() -> &'static str {
        "API_KEY=super-secret; exit 0"
    }

    #[cfg(windows)]
    fn secret_assignment_script() -> &'static str {
        "set API_KEY=super-secret & exit /B 0"
    }

    #[test]
    fn regressed_wall_clock_is_discarded_instead_of_corrupting_lifecycle_truth() {
        assert_eq!(non_regressing_wall_time(Some(9), 10, None), None);
        assert_eq!(non_regressing_wall_time(Some(10), 10, Some(11)), None);
        assert_eq!(non_regressing_wall_time(Some(12), 10, Some(11)), Some(12));
        assert_eq!(non_regressing_wall_time(None, 10, Some(11)), None);
    }

    #[test]
    fn starting_command_does_not_finalize_unrelated_observed_exit() {
        let root = TestRoot::new("no-global-finalize");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (pending_executable, pending_arguments) = command_parts(0, false);
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-pending-observed-exit",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-pending-observed-exit",
                    executable: pending_executable.to_str().unwrap(),
                    arguments: &pending_arguments,
                    command_source: FactSource::CallerRequested,
                    requested_cwd: cwd.to_str().unwrap(),
                    cwd_source: FactSource::CallerRequested,
                },
                10,
            )
            .unwrap();
        store
            .mark_shell_command_running("command-pending-observed-exit", Some(11))
            .unwrap();
        store
            .record_shell_command_exit_observation(
                "command-pending-observed-exit",
                Some(0),
                Some(12),
            )
            .unwrap();

        let (executable, arguments) = command_parts(0, false);
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-independent",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();

        let pending = store
            .load_execution("command-pending-observed-exit")
            .unwrap();
        assert_eq!(pending.status, ExecutionStatus::Running);
        let independent = store.load_execution("command-independent").unwrap();
        assert_eq!(independent.status, ExecutionStatus::Exited);
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
        assert!(result.duration_ms.is_some());
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
        assert!(command.observed_end_unix_ms.is_some());
        let git_observations = store.load_execution_git_observations("command-1").unwrap();
        assert_eq!(git_observations.len(), 2);
        assert!(git_observations.iter().all(|observation| {
            observation.availability == GitObservationAvailability::Unavailable
        }));
    }

    #[test]
    fn explicit_command_redacts_obvious_secret_metadata_without_changing_runtime_arguments() {
        let root = TestRoot::new("secret-metadata");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = shell_script(secret_assignment_script());
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-secret",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(0));
        let command = store.load_shell_command("command-secret").unwrap();
        assert_eq!(command.arguments.len(), arguments.len());
        assert!(
            command
                .arguments
                .iter()
                .any(|argument| argument == "<winds:redacted>")
        );
        assert!(!command.arguments.join(" ").contains("super-secret"));
    }

    #[test]
    fn explicit_command_policy_can_disable_command_argument_history() {
        let root = TestRoot::new("history-disabled");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, false);
        run_explicit_command_with_history_policy(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "command-history-disabled",
                workspace_id: "workspace-1",
                executable: &executable,
                arguments: &arguments,
                cwd: &cwd,
            },
            SessionHistoryPolicy::disabled(),
        )
        .unwrap();
        let command = store
            .load_shell_command("command-history-disabled")
            .unwrap();
        assert_eq!(
            command.arguments,
            vec!["<winds:history-disabled>".to_owned()]
        );
    }

    #[test]
    fn parser_free_explicit_run_does_not_upgrade_marker_like_output_authority() {
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
        assert_eq!(command.arguments, arguments);
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
    fn spawn_failure_is_explicit_and_records_only_the_before_boundary() {
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
        assert_eq!(command.observed_end_unix_ms, None);
        let observations = store
            .load_execution_git_observations("command-failed")
            .unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].boundary, GitObservationBoundary::Before);
        assert_eq!(
            observations[0].availability,
            GitObservationAvailability::Unavailable
        );
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
    fn command_git_observations_are_stable_for_no_change_and_anchor_to_workspace_root() {
        let root = TestRoot::new("git-no-change");
        let (mut store, workspace) = store_with_git_workspace(&root);
        let nested = workspace.join("nested");
        fs::create_dir(&nested).unwrap();
        let (executable, arguments) = command_parts(0, false);
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-no-change",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &nested,
            },
        )
        .unwrap();

        let observations = store
            .load_execution_git_observations("git-no-change")
            .unwrap();
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].boundary, GitObservationBoundary::Before);
        assert_eq!(observations[1].boundary, GitObservationBoundary::After);
        assert!(observations.iter().all(|observation| {
            observation.availability == GitObservationAvailability::Observed
                && observation.source == FactSource::WindsObserved
                && observation.dirty == Some(false)
        }));
        assert_eq!(observations[0].head_oid, observations[1].head_oid);
        assert_eq!(observations[0].branch, observations[1].branch);
        assert_eq!(
            observations[0].worktree_state_sha256,
            observations[1].worktree_state_sha256
        );
    }

    #[test]
    fn tracked_mutation_changes_worktree_digest_without_changing_head() {
        let root = TestRoot::new("git-mutation");
        let (mut store, workspace) = store_with_git_workspace(&root);
        let (executable, arguments) = shell_script(tracked_mutation_script());
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-mutation",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &workspace,
            },
        )
        .unwrap();
        let observations = store
            .load_execution_git_observations("git-mutation")
            .unwrap();
        assert_eq!(observations[0].dirty, Some(false));
        assert_eq!(observations[1].dirty, Some(true));
        assert_eq!(observations[0].head_oid, observations[1].head_oid);
        assert_ne!(
            observations[0].worktree_state_sha256,
            observations[1].worktree_state_sha256
        );
    }

    #[test]
    fn commit_creation_changes_head_and_returns_to_clean_state() {
        let root = TestRoot::new("git-commit");
        let (mut store, workspace) = store_with_git_workspace(&root);
        let (executable, arguments) = shell_script(commit_script());
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-commit",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &workspace,
            },
        )
        .unwrap();
        let observations = store.load_execution_git_observations("git-commit").unwrap();
        assert_eq!(observations[0].dirty, Some(false));
        assert_eq!(observations[1].dirty, Some(false));
        assert_ne!(observations[0].head_oid, observations[1].head_oid);
        assert_eq!(observations[0].branch, observations[1].branch);
    }

    #[test]
    fn branch_switch_changes_branch_without_fabricating_head_change() {
        let root = TestRoot::new("git-branch");
        let (mut store, workspace) = store_with_git_workspace(&root);
        let (executable, arguments) = shell_script(branch_switch_script());
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-branch",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &workspace,
            },
        )
        .unwrap();
        let observations = store.load_execution_git_observations("git-branch").unwrap();
        assert_eq!(observations[0].head_oid, observations[1].head_oid);
        assert_eq!(observations[0].branch.as_deref(), Some("main"));
        assert_eq!(observations[1].branch.as_deref(), Some("t055-observed"));
        assert_eq!(observations[0].detached, Some(false));
        assert_eq!(observations[1].detached, Some(false));
    }

    #[test]
    fn dirty_to_dirty_mutation_remains_distinguishable_by_digest() {
        let root = TestRoot::new("git-dirty-digest");
        let (mut store, workspace) = store_with_git_workspace(&root);
        fs::write(workspace.join("dirty-a.txt"), b"first\n").unwrap();
        let (executable, arguments) = shell_script(add_dirty_file_script());
        run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-dirty-digest",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &workspace,
            },
        )
        .unwrap();
        let observations = store
            .load_execution_git_observations("git-dirty-digest")
            .unwrap();
        assert_eq!(observations[0].dirty, Some(true));
        assert_eq!(observations[1].dirty, Some(true));
        assert_ne!(
            observations[0].worktree_state_sha256,
            observations[1].worktree_state_sha256
        );
    }

    #[test]
    fn repository_becoming_unavailable_persists_unknown_after_state_without_losing_exit() {
        let root = TestRoot::new("git-after-unavailable");
        let (mut store, workspace) = store_with_git_workspace(&root);
        let (executable, arguments) = shell_script(hide_git_metadata_script());
        let result = run_explicit_command(
            &mut store,
            ExplicitCommandRequest {
                execution_id: "git-after-unavailable",
                workspace_id: "workspace-git",
                executable: &executable,
                arguments: &arguments,
                cwd: &workspace,
            },
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(
            store
                .load_execution("git-after-unavailable")
                .unwrap()
                .status,
            ExecutionStatus::Exited
        );
        let observations = store
            .load_execution_git_observations("git-after-unavailable")
            .unwrap();
        assert_eq!(
            observations[0].availability,
            GitObservationAvailability::Observed
        );
        assert_eq!(
            observations[1].availability,
            GitObservationAvailability::Unavailable
        );
        assert_eq!(observations[1].head_oid, None);
        assert_eq!(observations[1].branch, None);
        assert_eq!(observations[1].detached, None);
        assert_eq!(observations[1].dirty, None);
        assert_eq!(observations[1].worktree_state_sha256, None);
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
            .mark_shell_command_running("command-restart", Some(11))
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
        assert_eq!(command.observed_end_unix_ms, None);
    }

    #[test]
    fn lifecycle_repairs_remain_final_when_wall_clock_is_unavailable() {
        let root = TestRoot::new("clock-repairs");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, false);

        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-clock-failed",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-clock-failed",
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
            .mark_shell_command_failed_to_start("command-clock-failed", None)
            .unwrap();
        let failed = store.load_execution("command-clock-failed").unwrap();
        assert_eq!(failed.status, ExecutionStatus::FailedToStart);
        assert_eq!(failed.started_unix_ms, None);
        assert_eq!(failed.ended_unix_ms, None);
        assert_eq!(failed.duration_ms, None);

        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-clock-interrupted",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-clock-interrupted",
                    executable: executable.to_str().unwrap(),
                    arguments: &arguments,
                    command_source: FactSource::CallerRequested,
                    requested_cwd: cwd.to_str().unwrap(),
                    cwd_source: FactSource::CallerRequested,
                },
                20,
            )
            .unwrap();
        store
            .mark_shell_command_running("command-clock-interrupted", None)
            .unwrap();
        store
            .mark_shell_command_interrupted("command-clock-interrupted", None)
            .unwrap();
        let interrupted = store.load_execution("command-clock-interrupted").unwrap();
        assert_eq!(interrupted.status, ExecutionStatus::Interrupted);
        assert_eq!(interrupted.started_unix_ms, None);
        assert_eq!(interrupted.ended_unix_ms, None);
        assert_eq!(interrupted.duration_ms, None);
    }

    #[test]
    fn durable_exit_observation_survives_store_restart_before_finalization() {
        let root = TestRoot::new("durable-exit");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(9, false);
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-durable",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-durable",
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
            .mark_shell_command_running("command-durable", Some(11))
            .unwrap();
        store
            .record_shell_command_exit_observation("command-durable", Some(9), Some(20))
            .unwrap();
        drop(store);

        let mut reopened = Store::open(&root.path().join("state")).unwrap();
        let ownership_lost = reopened
            .reconcile_unowned_shell_commands_after_restart(30)
            .unwrap();
        assert_eq!(ownership_lost, 0);
        let execution = reopened.load_execution("command-durable").unwrap();
        assert_eq!(execution.status, ExecutionStatus::Exited);
        assert_eq!(execution.ended_unix_ms, Some(20));
        assert_eq!(execution.duration_ms, Some(9));
        let command = reopened.load_shell_command("command-durable").unwrap();
        assert_eq!(command.exit_code, Some(9));
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
        assert_eq!(command.observed_end_unix_ms, Some(20));
    }

    #[test]
    fn observed_exit_without_wall_clock_finalizes_with_unknown_timing() {
        let root = TestRoot::new("clock-unknown");
        let mut store = store_with_workspace(&root);
        let cwd = workspace_path(&root);
        let (executable, arguments) = command_parts(0, false);
        store
            .create_shell_command_execution(
                NewExecution {
                    execution_id: "command-clock-unknown",
                    workspace_id: "workspace-1",
                    kind: ExecutionKind::ShellCommand,
                    request_source: FactSource::CallerRequested,
                    execution_domain: "native-test",
                },
                NewShellCommand {
                    execution_id: "command-clock-unknown",
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
            .mark_shell_command_running("command-clock-unknown", None)
            .unwrap();
        store
            .record_shell_command_exit_observation("command-clock-unknown", Some(0), None)
            .unwrap();
        store
            .finalize_shell_command_from_observation("command-clock-unknown")
            .unwrap();
        let execution = store.load_execution("command-clock-unknown").unwrap();
        assert_eq!(execution.status, ExecutionStatus::Exited);
        assert_eq!(execution.started_unix_ms, None);
        assert_eq!(execution.ended_unix_ms, None);
        assert_eq!(execution.duration_ms, None);
        let command = store.load_shell_command("command-clock-unknown").unwrap();
        assert_eq!(command.exit_code, Some(0));
        assert_eq!(command.exit_source, Some(FactSource::WindsObserved));
        assert_eq!(command.observed_end_unix_ms, None);
    }
}
