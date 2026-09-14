use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::{
    TerminalDropCleanupOutcome, TerminalExit, TerminalSession, TerminalSize,
};
use crate::git::workspace::open_existing_workspace;
use crate::git::workspace_inventory::inventory_workspace_environment;
use crate::store::{Result, Store};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_INPUT_BYTES: usize = 16 * 1024;
const MAX_TERMINAL_DIMENSION: u16 = 1000;
const TERMINAL_CLEANUP_TIMEOUT: Duration = Duration::from_millis(500);
static NEXT_TERMINAL_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopTerminalLifecycle {
    Live,
    Exited,
    Interrupted,
    OwnershipLost,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTerminalStatus {
    pub terminal_id: String,
    pub canonical_session_id: String,
    pub canonical_workspace_id: String,
    pub profile_id: String,
    pub profile_display_name: String,
    pub lifecycle: DesktopTerminalLifecycle,
    pub rows: u16,
    pub cols: u16,
    pub exit_code: Option<u32>,
    pub signal: Option<String>,
    pub close_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTerminalStartRequest {
    pub canonical_session_id: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTerminalTargetRequest {
    pub canonical_session_id: String,
    pub terminal_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTerminalInputRequest {
    pub canonical_session_id: String,
    pub terminal_id: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopTerminalResizeRequest {
    pub canonical_session_id: String,
    pub terminal_id: String,
    pub rows: u16,
    pub cols: u16,
}

pub struct DesktopTerminalStart {
    pub status: DesktopTerminalStatus,
    pub output_reader: Box<dyn Read + Send>,
}

struct DesktopOwnedTerminal {
    status: DesktopTerminalStatus,
    session: TerminalSession,
}

#[derive(Default)]
pub struct DesktopTerminalRegistry {
    terminals: HashMap<String, DesktopOwnedTerminal>,
    terminal_by_session: HashMap<String, String>,
    final_statuses: HashMap<String, DesktopTerminalStatus>,
    latest_final_by_session: HashMap<String, DesktopTerminalStatus>,
}

impl DesktopTerminalRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status_for_session(&self, canonical_session_id: &str) -> Option<DesktopTerminalStatus> {
        self.terminal_by_session
            .get(canonical_session_id)
            .and_then(|terminal_id| self.terminals.get(terminal_id))
            .map(|terminal| terminal.status.clone())
            .or_else(|| {
                self.latest_final_by_session
                    .get(canonical_session_id)
                    .cloned()
            })
    }

    pub fn status_for_target(
        &self,
        target: &DesktopTerminalTargetRequest,
    ) -> Result<DesktopTerminalStatus> {
        if let Some(terminal) = self.terminals.get(&target.terminal_id) {
            require_target_matches(&terminal.status, target)?;
            return Ok(terminal.status.clone());
        }
        let status = self
            .final_statuses
            .get(&target.terminal_id)
            .ok_or("unknown desktop terminal identity")?;
        require_target_matches(status, target)?;
        Ok(status.clone())
    }

    pub fn start(
        &mut self,
        home: &Path,
        request: DesktopTerminalStartRequest,
    ) -> Result<DesktopTerminalStart> {
        validate_size(request.rows, request.cols)?;
        if self
            .terminal_by_session
            .contains_key(&request.canonical_session_id)
        {
            return Err("canonical Session already has an active desktop terminal".into());
        }

        let store = Store::open(home)?;
        let session = store.load_winds_session(&request.canonical_session_id)?;
        if store
            .load_desktop_session_presentation(&request.canonical_session_id)?
            .is_some_and(|presentation| presentation.archived)
        {
            return Err("archived Session cannot start an active desktop terminal".into());
        }
        let workstream = store.load_workstream(&session.workstream_id)?;
        let workspace = store.load_workspace(&workstream.workspace_id)?;
        drop(store);

        let canonical_home = home
            .canonicalize()
            .map_err(|error| format!("desktop Winds home cannot be canonicalized: {error}"))?;
        let observation = open_existing_workspace(
            Path::new(&workspace.canonical_worktree_root),
            &canonical_home,
            unix_ms()?,
        )?;
        if observation.workspace_id != workspace.workspace_id
            || observation.canonical_worktree_root != workspace.canonical_worktree_root
            || observation.git_common_dir != workspace.git_common_dir
        {
            return Err(
                "canonical Session workspace identity no longer matches observed Git truth".into(),
            );
        }

        let inventory = inventory_workspace_environment(&observation)?;
        let profiles = discover_native_shell_profiles(&inventory)?;
        let profile = select_profile(&profiles)?;
        self.start_with_profile(
            request.canonical_session_id,
            workspace.workspace_id,
            profile,
            Path::new(&workspace.canonical_worktree_root),
            request.rows,
            request.cols,
        )
    }

    fn start_with_profile(
        &mut self,
        canonical_session_id: String,
        canonical_workspace_id: String,
        profile: &ShellProfile,
        cwd: &Path,
        rows: u16,
        cols: u16,
    ) -> Result<DesktopTerminalStart> {
        validate_size(rows, cols)?;
        if self.terminal_by_session.contains_key(&canonical_session_id) {
            return Err("canonical Session already has an active desktop terminal".into());
        }

        let mut session = TerminalSession::start(profile, cwd, TerminalSize { rows, cols })?;
        let output_reader = match session.take_output_reader() {
            Ok(reader) => reader,
            Err(error) => {
                let cleanup = session.cleanup_for_drop(TERMINAL_CLEANUP_TIMEOUT);
                if matches!(cleanup, Ok(TerminalDropCleanupOutcome::Unproven) | Err(_)) {
                    session.suppress_drop_cleanup_after_ownership_loss();
                }
                return Err(format!(
                    "terminal child started but desktop output ownership could not be established: {error}"
                )
                .into());
            }
        };
        let terminal_id = next_terminal_id()?;
        let status = DesktopTerminalStatus {
            terminal_id: terminal_id.clone(),
            canonical_session_id: canonical_session_id.clone(),
            canonical_workspace_id,
            profile_id: profile.profile_id.clone(),
            profile_display_name: profile.display_name.clone(),
            lifecycle: DesktopTerminalLifecycle::Live,
            rows,
            cols,
            exit_code: None,
            signal: None,
            close_reason: None,
        };
        self.terminal_by_session
            .insert(canonical_session_id, terminal_id.clone());
        self.terminals.insert(
            terminal_id,
            DesktopOwnedTerminal {
                status: status.clone(),
                session,
            },
        );
        Ok(DesktopTerminalStart {
            status,
            output_reader,
        })
    }

    pub fn send_input(
        &mut self,
        request: DesktopTerminalInputRequest,
    ) -> Result<DesktopTerminalStatus> {
        if request.bytes.is_empty() {
            return Err("desktop terminal input must not be empty".into());
        }
        if request.bytes.len() > MAX_INPUT_BYTES {
            return Err(
                format!("desktop terminal input exceeds {MAX_INPUT_BYTES} byte limit").into(),
            );
        }
        let target = DesktopTerminalTargetRequest {
            canonical_session_id: request.canonical_session_id,
            terminal_id: request.terminal_id,
        };
        let result = self
            .active_terminal_mut(&target)?
            .session
            .send_input(&request.bytes);
        self.finish_operation(target, result)
    }

    pub fn resize(
        &mut self,
        request: DesktopTerminalResizeRequest,
    ) -> Result<DesktopTerminalStatus> {
        validate_size(request.rows, request.cols)?;
        let target = DesktopTerminalTargetRequest {
            canonical_session_id: request.canonical_session_id,
            terminal_id: request.terminal_id,
        };
        let result = self
            .active_terminal_mut(&target)?
            .session
            .resize(TerminalSize {
                rows: request.rows,
                cols: request.cols,
            });
        if result.is_ok() {
            let terminal = self.active_terminal_mut(&target)?;
            terminal.status.rows = request.rows;
            terminal.status.cols = request.cols;
        }
        self.finish_operation(target, result)
    }

    pub fn interrupt(
        &mut self,
        target: DesktopTerminalTargetRequest,
    ) -> Result<DesktopTerminalStatus> {
        let result = self.active_terminal_mut(&target)?.session.interrupt();
        self.finish_operation(target, result)
    }

    pub fn terminate(
        &mut self,
        target: DesktopTerminalTargetRequest,
    ) -> Result<DesktopTerminalStatus> {
        self.finish_terminal(target, "TERMINATED_BY_WINDS", |session| session.terminate())
    }

    pub fn close(&mut self, target: DesktopTerminalTargetRequest) -> Result<DesktopTerminalStatus> {
        self.finish_terminal(target, "CLOSED_BY_WINDS", |session| session.close())
    }

    pub fn observe_output_end(
        &mut self,
        target: DesktopTerminalTargetRequest,
        read_error: Option<String>,
    ) -> Result<DesktopTerminalStatus> {
        if !self.terminals.contains_key(&target.terminal_id) {
            return self.status_for_target(&target);
        }
        if read_error.is_none() {
            let started = Instant::now();
            loop {
                match self.active_terminal_mut(&target)?.session.try_wait() {
                    Ok(Some(exit)) => {
                        return self.record_final(
                            &target,
                            DesktopTerminalLifecycle::Exited,
                            Some(exit),
                            Some("PROCESS_EXITED".to_owned()),
                        );
                    }
                    Ok(None) if started.elapsed() < Duration::from_millis(100) => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Ok(None) => break,
                    Err(error) => {
                        return self.lose_ownership(
                            &target,
                            format!("terminal exit observation failed after output end: {error}"),
                        );
                    }
                }
            }
        }
        let reason = read_error.unwrap_or_else(|| {
            "terminal output stream closed while child remained live".to_owned()
        });
        self.cleanup_after_stream_failure(&target, reason)
    }

    fn active_terminal_mut(
        &mut self,
        target: &DesktopTerminalTargetRequest,
    ) -> Result<&mut DesktopOwnedTerminal> {
        let terminal = self
            .terminals
            .get_mut(&target.terminal_id)
            .ok_or("desktop terminal is not live-owned")?;
        require_target_matches(&terminal.status, target)?;
        Ok(terminal)
    }

    fn finish_operation(
        &mut self,
        target: DesktopTerminalTargetRequest,
        result: Result<()>,
    ) -> Result<DesktopTerminalStatus> {
        match result {
            Ok(()) => Ok(self.active_terminal_mut(&target)?.status.clone()),
            Err(operation_error) => {
                let wait = self.active_terminal_mut(&target)?.session.try_wait();
                match wait {
                    Ok(Some(exit)) => {
                        self.record_final(
                            &target,
                            DesktopTerminalLifecycle::Exited,
                            Some(exit),
                            Some("PROCESS_EXITED".to_owned()),
                        )?;
                    }
                    Ok(None) => {}
                    Err(ownership_error) => {
                        self.lose_ownership(
                            &target,
                            format!("terminal operation failed and ownership could not be revalidated: {ownership_error}"),
                        )?;
                    }
                }
                Err(operation_error)
            }
        }
    }

    fn finish_terminal<F>(
        &mut self,
        target: DesktopTerminalTargetRequest,
        close_reason: &str,
        operation: F,
    ) -> Result<DesktopTerminalStatus>
    where
        F: FnOnce(&mut TerminalSession) -> Result<TerminalExit>,
    {
        let exit = {
            let terminal = self.active_terminal_mut(&target)?;
            operation(&mut terminal.session)
        };
        match exit {
            Ok(exit) => self.record_final(
                &target,
                DesktopTerminalLifecycle::Interrupted,
                Some(exit),
                Some(close_reason.to_owned()),
            ),
            Err(error) => {
                self.lose_ownership(&target, format!("terminal cleanup failed: {error}"))?;
                Err(error)
            }
        }
    }

    fn cleanup_after_stream_failure(
        &mut self,
        target: &DesktopTerminalTargetRequest,
        reason: String,
    ) -> Result<DesktopTerminalStatus> {
        let cleanup = self
            .active_terminal_mut(target)?
            .session
            .cleanup_for_drop(TERMINAL_CLEANUP_TIMEOUT);
        match cleanup {
            Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)) => self.record_final(
                target,
                DesktopTerminalLifecycle::Exited,
                Some(exit),
                Some("PROCESS_EXITED".to_owned()),
            ),
            Ok(TerminalDropCleanupOutcome::Terminated(exit)) => self.record_final(
                target,
                DesktopTerminalLifecycle::Interrupted,
                Some(exit),
                Some("OUTPUT_STREAM_FAILURE_CLEANUP".to_owned()),
            ),
            Ok(TerminalDropCleanupOutcome::Unproven) | Err(_) => {
                self.lose_ownership(target, reason)
            }
        }
    }

    fn lose_ownership(
        &mut self,
        target: &DesktopTerminalTargetRequest,
        reason: String,
    ) -> Result<DesktopTerminalStatus> {
        let mut terminal = self
            .terminals
            .remove(&target.terminal_id)
            .ok_or("desktop terminal is not live-owned")?;
        require_target_matches(&terminal.status, target)?;
        terminal
            .session
            .suppress_drop_cleanup_after_ownership_loss();
        let _ = reason;
        terminal.status.lifecycle = DesktopTerminalLifecycle::OwnershipLost;
        terminal.status.close_reason = Some("OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN".to_owned());
        terminal.status.exit_code = None;
        terminal.status.signal = None;
        self.terminal_by_session
            .remove(&target.canonical_session_id);
        let status = terminal.status.clone();
        self.final_statuses
            .insert(target.terminal_id.clone(), status.clone());
        self.latest_final_by_session
            .insert(target.canonical_session_id.clone(), status.clone());
        Ok(status)
    }

    fn record_final(
        &mut self,
        target: &DesktopTerminalTargetRequest,
        lifecycle: DesktopTerminalLifecycle,
        exit: Option<TerminalExit>,
        close_reason: Option<String>,
    ) -> Result<DesktopTerminalStatus> {
        let mut terminal = self
            .terminals
            .remove(&target.terminal_id)
            .ok_or("desktop terminal is not live-owned")?;
        require_target_matches(&terminal.status, target)?;
        terminal.status.lifecycle = lifecycle;
        terminal.status.exit_code = exit.as_ref().map(|value| value.exit_code);
        terminal.status.signal = exit.and_then(|value| value.signal);
        terminal.status.close_reason = close_reason;
        self.terminal_by_session
            .remove(&target.canonical_session_id);
        let status = terminal.status.clone();
        self.final_statuses
            .insert(target.terminal_id.clone(), status.clone());
        self.latest_final_by_session
            .insert(target.canonical_session_id.clone(), status.clone());
        Ok(status)
    }

    #[cfg(test)]
    pub(crate) fn start_with_test_profile(
        &mut self,
        canonical_session_id: &str,
        canonical_workspace_id: &str,
        profile: &ShellProfile,
        cwd: &Path,
        rows: u16,
        cols: u16,
    ) -> Result<DesktopTerminalStart> {
        self.start_with_profile(
            canonical_session_id.to_owned(),
            canonical_workspace_id.to_owned(),
            profile,
            cwd,
            rows,
            cols,
        )
    }
}

fn require_target_matches(
    status: &DesktopTerminalStatus,
    target: &DesktopTerminalTargetRequest,
) -> Result<()> {
    if status.canonical_session_id != target.canonical_session_id {
        return Err("desktop terminal target does not match canonical Session identity".into());
    }
    Ok(())
}

fn validate_size(rows: u16, cols: u16) -> Result<()> {
    if rows == 0 || cols == 0 {
        return Err("desktop terminal rows and columns must both be non-zero".into());
    }
    if rows > MAX_TERMINAL_DIMENSION || cols > MAX_TERMINAL_DIMENSION {
        return Err(format!(
            "desktop terminal dimensions exceed {MAX_TERMINAL_DIMENSION} cell limit"
        )
        .into());
    }
    Ok(())
}

fn select_profile(profiles: &[ShellProfile]) -> Result<&ShellProfile> {
    if profiles.is_empty() {
        return Err("no qualified native shell profile is available for this workspace".into());
    }
    for preferred in [std::env::var_os("SHELL"), std::env::var_os("COMSPEC")]
        .into_iter()
        .flatten()
    {
        if let Some(preferred) = preferred.to_str()
            && let Some(profile) = profiles
                .iter()
                .find(|profile| profile.executable == preferred)
        {
            return Ok(profile);
        }
    }
    profiles
        .first()
        .ok_or_else(|| "no qualified native shell profile is available for this workspace".into())
}

fn next_terminal_id() -> Result<String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let sequence = NEXT_TERMINAL_ID.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        "desktop-terminal-{}-{nanos}-{sequence}",
        std::process::id()
    ))
}

fn unix_ms() -> Result<i64> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    i64::try_from(millis)
        .map_err(|_| "system time exceeds supported desktop timestamp range".into())
}
