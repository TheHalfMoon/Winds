use super::{PaneId, PaneLifecycleView, PaneSize, WorkbenchState};
use crate::git::Result;
use crate::git::shell_profiles::ShellProfile;
use crate::git::terminal::{
    TerminalDropCleanupOutcome, TerminalExit, TerminalSession, TerminalSize,
};
#[cfg(windows)]
use crate::git::wsl_launch::{WslTerminalLaunchPlan, launch_wsl_terminal};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

const TERMINAL_CLEANUP_TIMEOUT: Duration = Duration::from_millis(500);

struct PaneTerminal {
    session: TerminalSession,
    output_reader: Box<dyn Read + Send>,
}

#[derive(Default)]
pub(crate) struct WorkbenchTerminals {
    terminals: HashMap<PaneId, PaneTerminal>,
}

impl WorkbenchTerminals {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn has_owned_terminal(&self, pane_id: PaneId) -> bool {
        self.terminals.contains_key(&pane_id)
    }

    pub(crate) fn start_native(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        profile: &ShellProfile,
        cwd: &Path,
    ) -> Result<()> {
        let size = require_startable_pane(state, pane_id)?;
        if self.terminals.contains_key(&pane_id) {
            return Err("workbench pane already has an owned terminal session".into());
        }

        let session = match TerminalSession::start(profile, cwd, terminal_size(size)) {
            Ok(session) => session,
            Err(error) => {
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                return Err(error);
            }
        };
        self.adopt_started_session(state, pane_id, session)
    }

    #[cfg(windows)]
    pub(crate) fn start_wsl(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        plan: &WslTerminalLaunchPlan,
    ) -> Result<()> {
        let size = require_startable_pane(state, pane_id)?;
        if self.terminals.contains_key(&pane_id) {
            return Err("workbench pane already has an owned terminal session".into());
        }

        let launched = match launch_wsl_terminal(plan, terminal_size(size)) {
            Ok(launched) => launched,
            Err(error) => {
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                return Err(error);
            }
        };
        self.adopt_started_session(state, pane_id, launched.session)
    }

    fn adopt_started_session(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        mut session: TerminalSession,
    ) -> Result<()> {
        let output_reader = match session.take_output_reader() {
            Ok(reader) => reader,
            Err(reader_error) => {
                let cleanup = session.cleanup_for_drop(TERMINAL_CLEANUP_TIMEOUT);
                let lifecycle = match cleanup {
                    Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(_))
                    | Ok(TerminalDropCleanupOutcome::Terminated(_)) => PaneLifecycleView::Error,
                    Ok(TerminalDropCleanupOutcome::Unproven) | Err(_) => {
                        session.suppress_drop_cleanup_after_ownership_loss();
                        PaneLifecycleView::OwnershipLost
                    }
                };
                state.set_pane_lifecycle(pane_id, lifecycle);
                return Err(format!(
                    "terminal child started but workbench output ownership could not be established: {reader_error}"
                )
                .into());
            }
        };

        self.terminals.insert(
            pane_id,
            PaneTerminal {
                session,
                output_reader,
            },
        );
        state.set_pane_lifecycle(pane_id, PaneLifecycleView::Live);
        Ok(())
    }

    pub(crate) fn poll_exit(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
    ) -> Result<Option<TerminalExit>> {
        let wait = self
            .terminals
            .get_mut(&pane_id)
            .ok_or("workbench pane has no owned terminal session")?
            .session
            .try_wait();

        match wait {
            Ok(Some(exit)) => {
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Exited);
                Ok(Some(exit))
            }
            Ok(None) => {
                if state
                    .pane(pane_id)
                    .is_some_and(|pane| pane.lifecycle != PaneLifecycleView::Error)
                {
                    state.set_pane_lifecycle(pane_id, PaneLifecycleView::Live);
                }
                Ok(None)
            }
            Err(error) => {
                self.revoke_unproven_ownership(state, pane_id);
                Err(format!(
                    "terminal exit observation failed; workbench ownership is lost: {error}"
                )
                .into())
            }
        }
    }

    pub(crate) fn read_output_once(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        buffer: &mut [u8],
    ) -> Result<usize> {
        if buffer.is_empty() {
            return Err("workbench output buffer must not be empty".into());
        }
        require_output_readable_pane(state, pane_id)?;

        let read = self
            .terminals
            .get_mut(&pane_id)
            .ok_or("workbench pane has no owned terminal session")?
            .output_reader
            .read(buffer);

        match read {
            Ok(0) => match self.poll_exit(state, pane_id) {
                Ok(Some(_)) => Ok(0),
                Ok(None) => {
                    state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                    Err("terminal output reader closed while the owned child was still live".into())
                }
                Err(error) => Err(error),
            },
            Ok(count) => Ok(count),
            Err(read_error) => match self.poll_exit(state, pane_id) {
                Ok(Some(_)) => Err(format!(
                    "terminal output reader failed before the owned child exit was observed: {read_error}"
                )
                .into()),
                Ok(None) => {
                    state.set_pane_lifecycle(pane_id, PaneLifecycleView::Error);
                    Err(format!(
                        "terminal output reader failed while the owned child remained live: {read_error}"
                    )
                    .into())
                }
                Err(ownership_error) => Err(format!(
                    "terminal output reader failed: {read_error}; {ownership_error}"
                )
                .into()),
            },
        }
    }

    pub(crate) fn resize(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        size: PaneSize,
    ) -> Result<()> {
        require_nonzero_size(size)?;
        self.require_live_owned(state, pane_id)?;
        let resize = self
            .terminals
            .get_mut(&pane_id)
            .expect("live owned pane was just proven")
            .session
            .resize(terminal_size(size));
        match resize {
            Ok(()) => {
                state.resize_pane(pane_id, size);
                Ok(())
            }
            Err(error) => {
                self.refresh_after_operation_error(state, pane_id);
                Err(error)
            }
        }
    }

    pub(crate) fn interrupt(&mut self, state: &mut WorkbenchState, pane_id: PaneId) -> Result<()> {
        self.require_live_owned(state, pane_id)?;
        let interrupt = self
            .terminals
            .get_mut(&pane_id)
            .expect("live owned pane was just proven")
            .session
            .interrupt();
        if interrupt.is_err() {
            self.refresh_after_operation_error(state, pane_id);
        }
        interrupt
    }

    pub(crate) fn terminate(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
    ) -> Result<TerminalExit> {
        self.finish_owned_terminal(state, pane_id, "terminate")
    }

    pub(crate) fn close(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
    ) -> Result<TerminalExit> {
        self.finish_owned_terminal(state, pane_id, "close")
    }

    /// Close the presentation pane only after any retained terminal authority has
    /// been resolved. Unproven cleanup keeps the pane visible as OWNERSHIP_LOST.
    pub(crate) fn close_pane(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
    ) -> Result<Option<TerminalExit>> {
        let lifecycle = state
            .pane(pane_id)
            .ok_or("unknown workbench pane")?
            .lifecycle;

        let exit = if self.terminals.contains_key(&pane_id) {
            Some(self.finish_owned_terminal(state, pane_id, "pane close")?)
        } else if lifecycle == PaneLifecycleView::Live {
            state.set_pane_lifecycle(pane_id, PaneLifecycleView::OwnershipLost);
            return Err(
                "live workbench pane has no owned terminal session; refusing visual close".into(),
            );
        } else {
            None
        };

        if !state.close_pane(pane_id) {
            return Err("workbench pane disappeared during terminal-aware close".into());
        }
        Ok(exit)
    }

    fn require_live_owned(&mut self, state: &mut WorkbenchState, pane_id: PaneId) -> Result<()> {
        require_live_pane(state, pane_id)?;
        if !self.terminals.contains_key(&pane_id) {
            state.set_pane_lifecycle(pane_id, PaneLifecycleView::OwnershipLost);
            return Err("workbench pane is marked live without an owned terminal session".into());
        }
        if self.poll_exit(state, pane_id)?.is_some() {
            return Err("workbench terminal session has already exited".into());
        }
        Ok(())
    }

    fn refresh_after_operation_error(&mut self, state: &mut WorkbenchState, pane_id: PaneId) {
        let _ = self.poll_exit(state, pane_id);
    }

    fn finish_owned_terminal(
        &mut self,
        state: &mut WorkbenchState,
        pane_id: PaneId,
        operation: &str,
    ) -> Result<TerminalExit> {
        let mut terminal = self
            .terminals
            .remove(&pane_id)
            .ok_or("workbench pane has no owned terminal session")?;
        let cleanup = terminal.session.cleanup_for_drop(TERMINAL_CLEANUP_TIMEOUT);
        match cleanup {
            Ok(TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit))
            | Ok(TerminalDropCleanupOutcome::Terminated(exit)) => {
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::Exited);
                Ok(exit)
            }
            Ok(TerminalDropCleanupOutcome::Unproven) => {
                terminal
                    .session
                    .suppress_drop_cleanup_after_ownership_loss();
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::OwnershipLost);
                Err(format!(
                    "terminal {operation} could not prove owned child exit inside bounded cleanup window"
                )
                .into())
            }
            Err(error) => {
                terminal
                    .session
                    .suppress_drop_cleanup_after_ownership_loss();
                state.set_pane_lifecycle(pane_id, PaneLifecycleView::OwnershipLost);
                Err(format!(
                    "terminal {operation} cleanup failed and workbench ownership is lost: {error}"
                )
                .into())
            }
        }
    }

    fn revoke_unproven_ownership(&mut self, state: &mut WorkbenchState, pane_id: PaneId) {
        if let Some(mut terminal) = self.terminals.remove(&pane_id) {
            terminal
                .session
                .suppress_drop_cleanup_after_ownership_loss();
        }
        state.set_pane_lifecycle(pane_id, PaneLifecycleView::OwnershipLost);
    }

    #[cfg(test)]
    pub(super) fn replace_output_reader_for_test(
        &mut self,
        pane_id: PaneId,
        reader: Box<dyn Read + Send>,
    ) -> bool {
        let Some(terminal) = self.terminals.get_mut(&pane_id) else {
            return false;
        };
        terminal.output_reader = reader;
        true
    }
}

fn require_startable_pane(state: &WorkbenchState, pane_id: PaneId) -> Result<PaneSize> {
    let pane = state.pane(pane_id).ok_or("unknown workbench pane")?;
    if pane.lifecycle == PaneLifecycleView::Live {
        return Err("workbench pane is already marked live".into());
    }
    require_nonzero_size(pane.size)?;
    Ok(pane.size)
}

fn require_live_pane(state: &WorkbenchState, pane_id: PaneId) -> Result<()> {
    let pane = state.pane(pane_id).ok_or("unknown workbench pane")?;
    if pane.lifecycle != PaneLifecycleView::Live {
        return Err("workbench pane is not live".into());
    }
    Ok(())
}

fn require_output_readable_pane(state: &WorkbenchState, pane_id: PaneId) -> Result<()> {
    let pane = state.pane(pane_id).ok_or("unknown workbench pane")?;
    match pane.lifecycle {
        PaneLifecycleView::Live | PaneLifecycleView::Exited => Ok(()),
        PaneLifecycleView::Stopped
        | PaneLifecycleView::OwnershipLost
        | PaneLifecycleView::Error => {
            Err("workbench pane output is unavailable without retained terminal ownership".into())
        }
    }
}

fn require_nonzero_size(size: PaneSize) -> Result<()> {
    if size.rows == 0 || size.columns == 0 {
        return Err("workbench terminal size rows and columns must both be non-zero".into());
    }
    Ok(())
}

fn terminal_size(size: PaneSize) -> TerminalSize {
    TerminalSize {
        rows: size.rows,
        cols: size.columns,
    }
}
