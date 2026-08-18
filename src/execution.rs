use crate::command::history::{
    PersistedSessionHistory, SessionHistoryPolicy, SessionHistoryRecorder, persisted_arguments,
};
use crate::domain::{ExecutionKind, FactSource, TerminalCloseReason};
use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::{
    TerminalDropCleanupOutcome, TerminalExit, TerminalSession, TerminalSessionId, TerminalSize,
};
#[cfg(windows)]
use crate::git::wsl_launch::{WslCwdResolution, WslTerminalLaunchPlan, launch_wsl_terminal};
use crate::store::{NewExecution, NewTerminalSession, Result, Store, TerminalFinalization};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LocalTerminalHistory<'a> {
    policy: SessionHistoryPolicy,
    state_root: &'a Path,
}

impl<'a> LocalTerminalHistory<'a> {
    pub(crate) const fn new(policy: SessionHistoryPolicy, state_root: &'a Path) -> Self {
        Self { policy, state_root }
    }
}

pub struct TerminalExecution<'store> {
    execution_id: String,
    store: &'store mut Store,
    session: TerminalSession,
    history: SessionHistoryRecorder,
    pending_final: Option<TerminalFinalization>,
    final_recorded: bool,
}

impl<'store> TerminalExecution<'store> {
    pub fn start_native(
        store: &'store mut Store,
        execution_id: &str,
        workspace_id: &str,
        profile: &ShellProfile,
        cwd: &Path,
        size: TerminalSize,
    ) -> Result<Self> {
        store.retry_deferred_terminal_finalizations_resilient()?;
        let history = SessionHistoryRecorder::new_disabled(execution_id)?;
        start_native_with_recorder(
            store,
            execution_id,
            workspace_id,
            profile,
            cwd,
            size,
            history,
        )
    }

    pub(crate) fn start_native_with_local_history(
        store: &'store mut Store,
        execution_id: &str,
        workspace_id: &str,
        profile: &ShellProfile,
        cwd: &Path,
        size: TerminalSize,
        history: LocalTerminalHistory<'_>,
    ) -> Result<Self> {
        store.retry_deferred_terminal_finalizations_resilient()?;
        let history =
            SessionHistoryRecorder::new_local(execution_id, history.policy, history.state_root)?;
        start_native_with_recorder(
            store,
            execution_id,
            workspace_id,
            profile,
            cwd,
            size,
            history,
        )
    }

    #[cfg(windows)]
    pub fn start_wsl(
        store: &'store mut Store,
        execution_id: &str,
        workspace_id: &str,
        plan: &WslTerminalLaunchPlan,
        size: TerminalSize,
    ) -> Result<Self> {
        store.retry_deferred_terminal_finalizations_resilient()?;
        let history = SessionHistoryRecorder::new_disabled(execution_id)?;
        start_wsl_with_recorder(store, execution_id, workspace_id, plan, size, history)
    }

    #[cfg(windows)]
    pub(crate) fn start_wsl_with_local_history(
        store: &'store mut Store,
        execution_id: &str,
        workspace_id: &str,
        plan: &WslTerminalLaunchPlan,
        size: TerminalSize,
        history: LocalTerminalHistory<'_>,
    ) -> Result<Self> {
        store.retry_deferred_terminal_finalizations_resilient()?;
        let history =
            SessionHistoryRecorder::new_local(execution_id, history.policy, history.state_root)?;
        start_wsl_with_recorder(store, execution_id, workspace_id, plan, size, history)
    }

    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    pub fn session_id(&self) -> TerminalSessionId {
        self.session.session_id()
    }

    pub fn profile_id(&self) -> &str {
        self.session.profile_id()
    }

    pub fn start_cwd(&self) -> &Path {
        self.session.start_cwd()
    }

    pub fn history_policy(&self) -> SessionHistoryPolicy {
        self.history.policy()
    }

    pub fn take_output_reader(&mut self) -> Result<Box<dyn Read + Send>> {
        let reader = self.session.take_output_reader()?;
        self.history.wrap_output_reader(reader)
    }

    pub fn persist_history(&mut self) -> Result<Option<PersistedSessionHistory>> {
        self.history.persist()
    }

    pub fn send_input(&mut self, bytes: &[u8]) -> Result<()> {
        if self.try_wait()?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.send_input(bytes)
    }

    pub fn resize(&mut self, size: TerminalSize) -> Result<()> {
        if self.try_wait()?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.resize(size)
    }

    pub fn current_size(&self) -> Result<TerminalSize> {
        self.session.current_size()
    }

    pub fn interrupt(&mut self) -> Result<()> {
        if self.try_wait()?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.interrupt()
    }

    pub fn try_wait(&mut self) -> Result<Option<TerminalExit>> {
        if self.pending_final.is_some() {
            self.persist_pending_final()?;
        }
        if self.final_recorded {
            return self.session.try_wait();
        }

        let exit = self.session.try_wait()?;
        if exit.is_some() {
            self.pending_final = Some(TerminalFinalization::Exited {
                ended_unix_ms: self.finalization_unix_ms()?,
            });
            self.persist_pending_final()?;
        }
        Ok(exit)
    }

    pub fn wait(&mut self) -> Result<TerminalExit> {
        if self.pending_final.is_some() {
            self.persist_pending_final()?;
            return self.session.wait();
        }
        if self.final_recorded {
            return self.session.wait();
        }

        let exit = self.session.wait()?;
        self.pending_final = Some(TerminalFinalization::Exited {
            ended_unix_ms: self.finalization_unix_ms()?,
        });
        self.persist_pending_final()?;
        Ok(exit)
    }

    pub fn terminate(&mut self) -> Result<TerminalExit> {
        self.controlled_cleanup(TerminalCloseReason::TerminatedByWinds, "terminate")
    }

    pub fn close(&mut self) -> Result<TerminalExit> {
        self.controlled_cleanup(TerminalCloseReason::ClosedByWinds, "close")
    }

    fn controlled_cleanup(
        &mut self,
        controlled_reason: TerminalCloseReason,
        operation: &str,
    ) -> Result<TerminalExit> {
        if self.pending_final.is_some() {
            self.persist_pending_final()?;
            return self.session.wait();
        }
        if self.final_recorded {
            return self.session.wait();
        }

        let observed_unix_ms = self.finalization_unix_ms()?;
        let outcome = self.session.cleanup_for_drop(Duration::from_millis(500))?;
        let (exit, finalization) = match outcome {
            TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit) => (
                exit,
                TerminalFinalization::Exited {
                    ended_unix_ms: observed_unix_ms,
                },
            ),
            TerminalDropCleanupOutcome::Terminated(exit) => (
                exit,
                TerminalFinalization::Interrupted {
                    ended_unix_ms: observed_unix_ms,
                    reason: controlled_reason,
                },
            ),
            TerminalDropCleanupOutcome::Unproven => {
                self.pending_final = Some(TerminalFinalization::OwnershipLost { observed_unix_ms });
                self.persist_pending_final()?;
                return Err(format!(
                    "terminal {operation} could not prove owned child exit inside bounded cleanup window"
                )
                .into());
            }
        };
        self.pending_final = Some(finalization);
        self.persist_pending_final()?;
        Ok(exit)
    }

    fn finalization_unix_ms(&self) -> Result<i64> {
        let execution = self.store.load_execution(&self.execution_id)?;
        let lower_bound = execution
            .started_unix_ms
            .unwrap_or(execution.requested_unix_ms);
        Ok(unix_ms()?.max(lower_bound))
    }

    fn persist_pending_final(&mut self) -> Result<()> {
        let Some(pending) = self.pending_final else {
            return Ok(());
        };
        self.store
            .apply_terminal_finalization(&self.execution_id, pending)?;
        self.pending_final = None;
        self.final_recorded = true;
        Ok(())
    }

    fn persist_or_defer_on_drop(&mut self, finalization: TerminalFinalization) {
        match self
            .store
            .apply_terminal_finalization(&self.execution_id, finalization)
        {
            Ok(()) => {
                self.pending_final = None;
                self.final_recorded = true;
            }
            Err(_) => {
                self.store
                    .defer_terminal_finalization(&self.execution_id, finalization);
            }
        }
    }
}

impl Drop for TerminalExecution<'_> {
    fn drop(&mut self) {
        if self.final_recorded {
            return;
        }
        if let Some(pending) = self.pending_final {
            self.persist_or_defer_on_drop(pending);
            return;
        }

        let observed_unix_ms = match self.finalization_unix_ms() {
            Ok(value) => value,
            Err(_) => return,
        };
        let finalization = match self.session.cleanup_for_drop(Duration::from_millis(500)) {
            Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(_)) => {
                TerminalFinalization::Exited {
                    ended_unix_ms: observed_unix_ms,
                }
            }
            Ok(TerminalDropCleanupOutcome::Terminated(_)) => TerminalFinalization::Interrupted {
                ended_unix_ms: observed_unix_ms,
                reason: TerminalCloseReason::ClosedByWinds,
            },
            Ok(TerminalDropCleanupOutcome::Unproven) | Err(_) => {
                TerminalFinalization::OwnershipLost { observed_unix_ms }
            }
        };
        self.persist_or_defer_on_drop(finalization);
    }
}

pub fn reconcile_terminal_executions_after_restart(store: &mut Store) -> Result<usize> {
    store.retry_deferred_terminal_finalizations_resilient()?;
    store.reconcile_unowned_terminal_sessions_after_restart(unix_ms()?)
}

fn start_native_with_recorder<'store>(
    store: &'store mut Store,
    execution_id: &str,
    workspace_id: &str,
    profile: &ShellProfile,
    cwd: &Path,
    size: TerminalSize,
    history: SessionHistoryRecorder,
) -> Result<TerminalExecution<'store>> {
    let requested_cwd = utf8_path(cwd, "terminal requested cwd")?;
    let execution_domain = serde_json::to_string(&profile.execution_domain)?;
    let persisted_shell_arguments = persisted_arguments(&profile.arguments, history.policy());
    let requested_unix_ms = unix_ms()?;
    store.create_terminal_execution(
        NewExecution {
            execution_id,
            workspace_id,
            kind: ExecutionKind::Terminal,
            request_source: FactSource::CallerRequested,
            execution_domain: &execution_domain,
        },
        NewTerminalSession {
            execution_id,
            profile_id: &profile.profile_id,
            shell_executable: &profile.executable,
            shell_arguments: &persisted_shell_arguments,
            requested_cwd,
            initial_cols: Some(size.cols),
            initial_rows: Some(size.rows),
        },
        requested_unix_ms,
    )?;

    match TerminalSession::start(profile, cwd, size) {
        Ok(session) => finish_started_session(store, execution_id, session, history),
        Err(error) => fail_launch(store, execution_id, error),
    }
}

#[cfg(windows)]
fn start_wsl_with_recorder<'store>(
    store: &'store mut Store,
    execution_id: &str,
    workspace_id: &str,
    plan: &WslTerminalLaunchPlan,
    size: TerminalSize,
    history: SessionHistoryRecorder,
) -> Result<TerminalExecution<'store>> {
    let requested_cwd = match &plan.cwd_resolution {
        WslCwdResolution::MappedWorkspace {
            linux_workspace_root,
            ..
        } => linux_workspace_root.as_str(),
        WslCwdResolution::FallbackHome { linux_home, .. } => linux_home.as_str(),
    };
    let execution_domain = serde_json::to_string(&plan.profile.execution_domain)?;
    let persisted_shell_arguments =
        persisted_arguments(&plan.profile.shell_arguments, history.policy());
    let requested_unix_ms = unix_ms()?;
    store.create_terminal_execution(
        NewExecution {
            execution_id,
            workspace_id,
            kind: ExecutionKind::Terminal,
            request_source: FactSource::CallerRequested,
            execution_domain: &execution_domain,
        },
        NewTerminalSession {
            execution_id,
            profile_id: &plan.profile.profile_id,
            shell_executable: &plan.profile.shell_executable,
            shell_arguments: &persisted_shell_arguments,
            requested_cwd,
            initial_cols: Some(size.cols),
            initial_rows: Some(size.rows),
        },
        requested_unix_ms,
    )?;

    match launch_wsl_terminal(plan, size) {
        Ok(launched) => finish_started_session(store, execution_id, launched.session, history),
        Err(error) => fail_launch(store, execution_id, error),
    }
}

fn finish_started_session<'store>(
    store: &'store mut Store,
    execution_id: &str,
    mut session: TerminalSession,
    history: SessionHistoryRecorder,
) -> Result<TerminalExecution<'store>> {
    let started_unix_ms = match unix_ms() {
        Ok(value) => value,
        Err(error) => {
            let cleanup = session.terminate();
            return Err(format!(
                "terminal child started but start time could not be recorded: {error}; owned child cleanup {}",
                if cleanup.is_ok() { "succeeded" } else { "failed" }
            )
            .into());
        }
    };

    if let Err(persist_error) = store.mark_terminal_running(execution_id, started_unix_ms) {
        let cleanup = session.terminate();
        let cleanup_proven = cleanup.is_ok();
        let repair = if cleanup_proven {
            let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms).max(started_unix_ms);
            store.mark_terminal_start_persistence_failed(
                execution_id,
                started_unix_ms,
                ended_unix_ms,
            )
        } else {
            Ok(())
        };
        let repair_note = match repair {
            Ok(()) if cleanup_proven => "interrupted cleanup state persisted".to_owned(),
            Ok(()) => {
                "cleanup was not proven; request remains non-final for restart reconciliation"
                    .to_owned()
            }
            Err(error) => format!("cleanup state persistence also failed: {error}"),
        };
        return Err(format!(
            "terminal child started but RUNNING persistence failed: {persist_error}; owned child cleanup {}; {repair_note}",
            if cleanup_proven { "succeeded" } else { "failed" }
        )
        .into());
    }

    Ok(TerminalExecution {
        execution_id: execution_id.to_owned(),
        store,
        session,
        history,
        pending_final: None,
        final_recorded: false,
    })
}

fn fail_launch<'store>(
    store: &'store mut Store,
    execution_id: &str,
    launch_error: Box<dyn std::error::Error + Send + Sync>,
) -> Result<TerminalExecution<'store>> {
    let execution = store.load_execution(execution_id)?;
    let failed_unix_ms = unix_ms()?.max(execution.requested_unix_ms);
    match store.mark_terminal_failed_to_start(execution_id, failed_unix_ms) {
        Ok(()) => Err(format!("terminal launch failed: {launch_error}").into()),
        Err(persist_error) => Err(format!(
            "terminal launch failed: {launch_error}; FAILED_TO_START persistence also failed: {persist_error}"
        )
        .into()),
    }
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str> {
    path.to_str()
        .ok_or_else(|| format!("{label} is not valid UTF-8").into())
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(i64::try_from(millis)?)
}

#[cfg(all(test, unix))]
mod tests {
    use super::{LocalTerminalHistory, TerminalExecution};
    use crate::command::history::SessionHistoryPolicy;
    use crate::domain::{ExecutionStatus, FactSource, TerminalCloseReason};
    use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
    use crate::git::terminal::TerminalSize;
    use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
    use crate::store::{NewWorkspace, Store};
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

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
            Self(fs::canonicalize(path).unwrap())
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

    fn native_sh_profile() -> ShellProfile {
        let inventory = WorkspaceEnvironmentInventory {
            host_os: std::env::consts::OS.to_owned(),
            host_arch: std::env::consts::ARCH.to_owned(),
            canonical_worktree_root: "/unused/worktree".to_owned(),
            git_common_dir: "/unused/git-common".to_owned(),
            shell_candidates: vec!["/bin/sh".to_owned()],
            detected_manifests: Vec::new(),
        };
        discover_native_shell_profiles(&inventory)
            .unwrap()
            .into_iter()
            .find(|profile| profile.executable == "/bin/sh")
            .expect("/bin/sh must be available on supported Unix CI hosts")
    }

    fn drain_output(mut reader: Box<dyn Read + Send>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => return,
                    Ok(_) => {}
                }
            }
        })
    }

    fn collect_output(mut reader: Box<dyn Read + Send>) -> thread::JoinHandle<Vec<u8>> {
        thread::spawn(move || {
            let mut output = Vec::new();
            reader.read_to_end(&mut output).unwrap();
            output
        })
    }

    fn store_with_workspace(root: &TestRoot) -> Store {
        let home = root.path().join("state");
        let store = Store::open(&home).unwrap();
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id: "workspace-1",
                    canonical_worktree_root: root.path().to_str().unwrap(),
                    git_common_dir: root.path().join(".git").to_str().unwrap(),
                },
                1,
            )
            .unwrap();
        store
    }

    #[test]
    fn native_terminal_records_running_then_natural_exit() {
        let root = TestRoot::new("natural-exit");
        let mut store = store_with_workspace(&root);
        let profile = native_sh_profile();
        let mut execution = TerminalExecution::start_native(
            &mut store,
            "execution-natural",
            "workspace-1",
            &profile,
            root.path(),
            TerminalSize { rows: 24, cols: 80 },
        )
        .unwrap();

        assert_eq!(execution.history_policy(), SessionHistoryPolicy::disabled());
        let output = drain_output(execution.take_output_reader().unwrap());
        execution.send_input(b"exit 0\n").unwrap();
        let exit = execution.wait().unwrap();
        assert_eq!(exit.exit_code, 0);
        assert!(execution.persist_history().unwrap().is_none());
        drop(execution);
        output.join().unwrap();

        let final_record = store.load_execution("execution-natural").unwrap();
        assert_eq!(final_record.status, ExecutionStatus::Exited);
        assert_eq!(final_record.status_source, FactSource::WindsObserved);
        assert!(final_record.started_unix_ms.is_some());
        assert!(final_record.ended_unix_ms.is_some());
        assert!(final_record.duration_ms.is_some());
        let terminal = store.load_terminal_session("execution-natural").unwrap();
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::ProcessExited)
        );
        assert_eq!(
            terminal.shell_arguments,
            vec!["<winds:history-disabled>".to_owned()]
        );
    }

    #[test]
    fn bounded_transcript_history_is_explicit_and_records_final_truncation_metadata() {
        let root = TestRoot::new("bounded-history");
        let state_home = root.path().join("state");
        let mut store = store_with_workspace(&root);
        let profile = native_sh_profile();
        let policy = SessionHistoryPolicy::local_bounded(false, 5, 16_384).unwrap();
        let mut execution = TerminalExecution::start_native_with_local_history(
            &mut store,
            "execution-bounded-history",
            "workspace-1",
            &profile,
            root.path(),
            TerminalSize { rows: 24, cols: 80 },
            LocalTerminalHistory::new(policy, &state_home),
        )
        .unwrap();

        let output = collect_output(execution.take_output_reader().unwrap());
        execution
            .send_input(b"printf 'abcdefgh'; exit 0\n")
            .unwrap();
        let exit = execution.wait().unwrap();
        assert_eq!(exit.exit_code, 0);
        let live_output = output.join().unwrap();
        assert!(live_output.len() > 5);
        let persisted = execution.persist_history().unwrap().unwrap();
        assert_eq!(persisted.manifest.policy, policy);
        assert_eq!(persisted.manifest.transcript_retained_bytes, 5);
        assert!(persisted.manifest.transcript_observed_bytes > 5);
        assert!(persisted.manifest.transcript_capture_complete);
        assert!(persisted.manifest.transcript_truncated);
        assert_eq!(
            fs::read(state_home.join(&persisted.manifest.transcript.relative_path))
                .unwrap()
                .len(),
            5
        );
        drop(execution);
    }

    #[test]
    fn controlled_terminate_records_interrupted() {
        let root = TestRoot::new("terminate");
        let mut store = store_with_workspace(&root);
        let profile = native_sh_profile();
        let mut execution = TerminalExecution::start_native(
            &mut store,
            "execution-terminate",
            "workspace-1",
            &profile,
            root.path(),
            TerminalSize { rows: 24, cols: 80 },
        )
        .unwrap();

        let output = drain_output(execution.take_output_reader().unwrap());
        let ready = root.path().join("terminate-ready");
        execution
            .send_input(b"printf ready > terminate-ready; while :; do sleep 1; done\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.is_file() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(ready.is_file(), "terminate fixture shell never became live");
        execution.terminate().unwrap();
        drop(execution);
        output.join().unwrap();
        let final_record = store.load_execution("execution-terminate").unwrap();
        assert_eq!(final_record.status, ExecutionStatus::Interrupted);
        let terminal = store.load_terminal_session("execution-terminate").unwrap();
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::TerminatedByWinds)
        );
    }

    #[test]
    fn dropping_live_terminal_records_only_proven_cleanup_truth() {
        let root = TestRoot::new("drop-live");
        let mut store = store_with_workspace(&root);
        let profile = native_sh_profile();
        let execution = TerminalExecution::start_native(
            &mut store,
            "execution-drop",
            "workspace-1",
            &profile,
            root.path(),
            TerminalSize { rows: 24, cols: 80 },
        )
        .unwrap();

        drop(execution);
        let final_record = store.load_execution("execution-drop").unwrap();
        assert_eq!(final_record.status_source, FactSource::WindsObserved);
        assert!(final_record.started_unix_ms.is_some());
        let terminal = store.load_terminal_session("execution-drop").unwrap();
        match final_record.status {
            ExecutionStatus::Interrupted => {
                assert_eq!(
                    terminal.close_reason,
                    Some(TerminalCloseReason::ClosedByWinds)
                );
                assert!(final_record.ended_unix_ms.is_some());
                assert!(final_record.duration_ms.is_some());
            }
            ExecutionStatus::OwnershipLost => {
                assert_eq!(
                    terminal.close_reason,
                    Some(TerminalCloseReason::OwnershipLostProcessStateUnknown)
                );
                assert_eq!(final_record.ended_unix_ms, None);
                assert_eq!(final_record.duration_ms, None);
            }
            other => {
                panic!("live terminal Drop must record only proven cleanup truth, got {other:?}")
            }
        }
        assert_eq!(store.pending_terminal_finalization_count(), 0);
    }

    #[test]
    fn launch_failure_records_failed_to_start_without_duration() {
        let root = TestRoot::new("failed-start");
        let mut store = store_with_workspace(&root);
        let profile = native_sh_profile();
        let missing = root.path().join("missing-cwd");
        let result = TerminalExecution::start_native(
            &mut store,
            "execution-failed",
            "workspace-1",
            &profile,
            &missing,
            TerminalSize { rows: 24, cols: 80 },
        );
        assert!(result.is_err());
        drop(result);

        let final_record = store.load_execution("execution-failed").unwrap();
        assert_eq!(final_record.status, ExecutionStatus::FailedToStart);
        assert_eq!(final_record.started_unix_ms, None);
        assert_eq!(final_record.duration_ms, None);
        let terminal = store.load_terminal_session("execution-failed").unwrap();
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::FailedToStart)
        );
    }
}
