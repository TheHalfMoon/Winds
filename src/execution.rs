use crate::domain::{ExecutionKind, FactSource, TerminalCloseReason};
use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::{
    TerminalExit, TerminalSession, TerminalSessionId, TerminalSize,
};
#[cfg(windows)]
use crate::git::wsl_launch::{
    WslCwdResolution, WslTerminalLaunchPlan, launch_wsl_terminal,
};
use crate::store::{NewExecution, NewTerminalSession, Result, Store};
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
enum PendingFinal {
    Exited {
        ended_unix_ms: i64,
    },
    Interrupted {
        ended_unix_ms: i64,
        reason: TerminalCloseReason,
    },
}

pub struct TerminalExecution {
    execution_id: String,
    session: TerminalSession,
    pending_final: Option<PendingFinal>,
    final_recorded: bool,
}

impl TerminalExecution {
    pub fn start_native(
        store: &mut Store,
        execution_id: &str,
        workspace_id: &str,
        profile: &ShellProfile,
        cwd: &Path,
        size: TerminalSize,
    ) -> Result<Self> {
        let requested_cwd = utf8_path(cwd, "terminal requested cwd")?;
        let execution_domain = serde_json::to_string(&profile.execution_domain)?;
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
                shell_arguments: &profile.arguments,
                requested_cwd,
                initial_cols: Some(size.cols),
                initial_rows: Some(size.rows),
            },
            requested_unix_ms,
        )?;

        match TerminalSession::start(profile, cwd, size) {
            Ok(session) => finish_started_session(store, execution_id, session),
            Err(error) => fail_launch(store, execution_id, error),
        }
    }

    #[cfg(windows)]
    pub fn start_wsl(
        store: &mut Store,
        execution_id: &str,
        workspace_id: &str,
        plan: &WslTerminalLaunchPlan,
        size: TerminalSize,
    ) -> Result<Self> {
        let requested_cwd = match &plan.cwd_resolution {
            WslCwdResolution::MappedWorkspace {
                linux_workspace_root,
                ..
            } => linux_workspace_root.as_str(),
            WslCwdResolution::FallbackHome { linux_home, .. } => linux_home.as_str(),
        };
        let execution_domain = serde_json::to_string(&plan.profile.execution_domain)?;
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
                shell_arguments: &plan.profile.shell_arguments,
                requested_cwd,
                initial_cols: Some(size.cols),
                initial_rows: Some(size.rows),
            },
            requested_unix_ms,
        )?;

        match launch_wsl_terminal(plan, size) {
            Ok(launched) => finish_started_session(store, execution_id, launched.session),
            Err(error) => fail_launch(store, execution_id, error),
        }
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

    pub fn take_output_reader(&mut self) -> Result<Box<dyn Read + Send>> {
        self.session.take_output_reader()
    }

    pub fn send_input(&mut self, store: &mut Store, bytes: &[u8]) -> Result<()> {
        if self.try_wait(store)?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.send_input(bytes)
    }

    pub fn resize(&mut self, store: &mut Store, size: TerminalSize) -> Result<()> {
        if self.try_wait(store)?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.resize(size)
    }

    pub fn current_size(&self) -> Result<TerminalSize> {
        self.session.current_size()
    }

    pub fn interrupt(&mut self, store: &mut Store) -> Result<()> {
        if self.try_wait(store)?.is_some() {
            return Err("terminal execution has already exited".into());
        }
        self.session.interrupt()
    }

    pub fn try_wait(&mut self, store: &mut Store) -> Result<Option<TerminalExit>> {
        if self.pending_final.is_some() {
            self.persist_pending_final(store)?;
        }
        if self.final_recorded {
            return self.session.try_wait();
        }

        let exit = self.session.try_wait()?;
        if exit.is_some() {
            self.pending_final = Some(PendingFinal::Exited {
                ended_unix_ms: unix_ms()?,
            });
            self.persist_pending_final(store)?;
        }
        Ok(exit)
    }

    pub fn wait(&mut self, store: &mut Store) -> Result<TerminalExit> {
        if self.pending_final.is_some() {
            self.persist_pending_final(store)?;
            return self.session.wait();
        }
        if self.final_recorded {
            return self.session.wait();
        }

        let exit = self.session.wait()?;
        self.pending_final = Some(PendingFinal::Exited {
            ended_unix_ms: unix_ms()?,
        });
        self.persist_pending_final(store)?;
        Ok(exit)
    }

    pub fn terminate(&mut self, store: &mut Store) -> Result<TerminalExit> {
        if self.pending_final.is_some() {
            self.persist_pending_final(store)?;
            return self.session.wait();
        }
        if self.final_recorded {
            return self.session.wait();
        }
        if let Some(exit) = self.session.try_wait()? {
            self.pending_final = Some(PendingFinal::Exited {
                ended_unix_ms: unix_ms()?,
            });
            self.persist_pending_final(store)?;
            return Ok(exit);
        }

        let exit = self.session.terminate()?;
        self.pending_final = Some(PendingFinal::Interrupted {
            ended_unix_ms: unix_ms()?,
            reason: TerminalCloseReason::TerminatedByWinds,
        });
        self.persist_pending_final(store)?;
        Ok(exit)
    }

    pub fn close(&mut self, store: &mut Store) -> Result<TerminalExit> {
        if self.pending_final.is_some() {
            self.persist_pending_final(store)?;
            return self.session.wait();
        }
        if self.final_recorded {
            return self.session.wait();
        }
        if let Some(exit) = self.session.try_wait()? {
            self.pending_final = Some(PendingFinal::Exited {
                ended_unix_ms: unix_ms()?,
            });
            self.persist_pending_final(store)?;
            return Ok(exit);
        }

        let exit = self.session.close()?;
        self.pending_final = Some(PendingFinal::Interrupted {
            ended_unix_ms: unix_ms()?,
            reason: TerminalCloseReason::ClosedByWinds,
        });
        self.persist_pending_final(store)?;
        Ok(exit)
    }

    fn persist_pending_final(&mut self, store: &mut Store) -> Result<()> {
        let Some(pending) = self.pending_final else {
            return Ok(());
        };
        match pending {
            PendingFinal::Exited { ended_unix_ms } => {
                store.mark_terminal_exited(&self.execution_id, ended_unix_ms)?;
            }
            PendingFinal::Interrupted {
                ended_unix_ms,
                reason,
            } => {
                store.mark_terminal_interrupted(&self.execution_id, reason, ended_unix_ms)?;
            }
        }
        self.pending_final = None;
        self.final_recorded = true;
        Ok(())
    }
}

pub fn reconcile_terminal_executions_after_restart(store: &mut Store) -> Result<usize> {
    store.reconcile_unowned_terminal_sessions_after_restart(unix_ms()?)
}

fn finish_started_session(
    store: &mut Store,
    execution_id: &str,
    mut session: TerminalSession,
) -> Result<TerminalExecution> {
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
            let ended_unix_ms = unix_ms().unwrap_or(started_unix_ms);
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
            Ok(()) => "cleanup was not proven; request remains non-final for restart reconciliation"
                .to_owned(),
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
        session,
        pending_final: None,
        final_recorded: false,
    })
}

fn fail_launch(
    store: &mut Store,
    execution_id: &str,
    launch_error: Box<dyn std::error::Error + Send + Sync>,
) -> Result<TerminalExecution> {
    let failed_unix_ms = unix_ms()?;
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
    use super::TerminalExecution;
    use crate::domain::{ExecutionStatus, FactSource, TerminalCloseReason};
    use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
    use crate::git::terminal::TerminalSize;
    use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
    use crate::store::{NewWorkspace, Store};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(name: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "winds-t053-{name}-{}-{sequence}",
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

    fn store_with_workspace(root: &TestRoot) -> Store {
        let home = root.path().join("state");
        let mut store = Store::open(&home).unwrap();
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

        let running = store.load_execution("execution-natural").unwrap();
        assert_eq!(running.status, ExecutionStatus::Running);
        assert_eq!(running.status_source, FactSource::WindsObserved);
        assert!(running.started_unix_ms.is_some());

        execution.send_input(&mut store, b"exit 0\n").unwrap();
        let exit = execution.wait(&mut store).unwrap();
        assert_eq!(exit.exit_code, 0);

        let final_record = store.load_execution("execution-natural").unwrap();
        assert_eq!(final_record.status, ExecutionStatus::Exited);
        assert_eq!(final_record.status_source, FactSource::WindsObserved);
        assert!(final_record.ended_unix_ms.is_some());
        assert!(final_record.duration_ms.is_some());
        let terminal = store.load_terminal_session("execution-natural").unwrap();
        assert_eq!(terminal.close_reason, Some(TerminalCloseReason::ProcessExited));
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

        execution.terminate(&mut store).unwrap();
        let final_record = store.load_execution("execution-terminate").unwrap();
        assert_eq!(final_record.status, ExecutionStatus::Interrupted);
        let terminal = store.load_terminal_session("execution-terminate").unwrap();
        assert_eq!(
            terminal.close_reason,
            Some(TerminalCloseReason::TerminatedByWinds)
        );
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