use super::super::{PaneId, WorkbenchState};
use super::WorkbenchTerminals;
use crate::git::Result;
use ratatui_textarea::{CursorMove, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkbenchInputMode {
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellCursorMove {
    Forward,
    Back,
    Up,
    Down,
    Head,
    End,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MultilineSubmitPolicy {
    Reject,
    Literal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellSubmitTerminator {
    LineFeed,
    CarriageReturnLineFeed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ShellDispatchReceipt {
    pub(crate) pane_id: PaneId,
    pub(crate) submitted_bytes: Vec<u8>,
}

pub(crate) struct WorkbenchShellEditor {
    textarea: TextArea<'static>,
}

impl Default for WorkbenchShellEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkbenchShellEditor {
    pub(crate) fn new() -> Self {
        Self {
            textarea: TextArea::default(),
        }
    }

    pub(crate) const fn mode(&self) -> WorkbenchInputMode {
        WorkbenchInputMode::Shell
    }

    pub(crate) fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    pub(crate) fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    pub(crate) fn is_selecting(&self) -> bool {
        self.textarea.is_selecting()
    }

    pub(crate) fn insert_char(&mut self, character: char) -> Result<()> {
        if character == '\r' || character == '\n' {
            return Err(
                "shell editor newline insertion must use the explicit newline operation".into(),
            );
        }
        self.textarea.insert_char(character);
        Ok(())
    }

    pub(crate) fn insert_text(&mut self, text: &str) -> Result<bool> {
        if text.contains('\r') || text.contains('\n') {
            return Err(
                "shell editor single-line insertion rejects newline-bearing text; use explicit multiline paste"
                    .into(),
            );
        }
        Ok(self.textarea.insert_str(text))
    }

    pub(crate) fn insert_newline(&mut self) {
        self.textarea.insert_newline();
    }

    pub(crate) fn paste_single_line(&mut self, text: &str) -> Result<bool> {
        self.insert_text(text)
    }

    pub(crate) fn paste_multiline_literal(&mut self, text: &str) -> Result<bool> {
        if text.contains('\r') {
            return Err(
                "carriage-return paste is unsupported in T091; no silent CR/CRLF normalization is permitted"
                    .into(),
            );
        }
        Ok(self.textarea.insert_str(text))
    }

    pub(crate) fn backspace(&mut self) -> bool {
        self.textarea.delete_char()
    }

    pub(crate) fn delete_next(&mut self) -> bool {
        self.textarea.delete_next_char()
    }

    pub(crate) fn move_cursor(&mut self, movement: ShellCursorMove) {
        let movement = match movement {
            ShellCursorMove::Forward => CursorMove::Forward,
            ShellCursorMove::Back => CursorMove::Back,
            ShellCursorMove::Up => CursorMove::Up,
            ShellCursorMove::Down => CursorMove::Down,
            ShellCursorMove::Head => CursorMove::Head,
            ShellCursorMove::End => CursorMove::End,
            ShellCursorMove::Top => CursorMove::Top,
            ShellCursorMove::Bottom => CursorMove::Bottom,
        };
        self.textarea.move_cursor(movement);
    }

    pub(crate) fn start_selection(&mut self) {
        self.textarea.start_selection();
    }

    pub(crate) fn cancel_selection(&mut self) {
        self.textarea.cancel_selection();
    }

    pub(crate) fn select_all(&mut self) {
        self.textarea.select_all();
    }

    pub(crate) fn undo(&mut self) -> bool {
        self.textarea.undo()
    }

    pub(crate) fn redo(&mut self) -> bool {
        self.textarea.redo()
    }

    pub(crate) fn submitted_bytes(
        &self,
        multiline: MultilineSubmitPolicy,
        terminator: ShellSubmitTerminator,
    ) -> Result<Vec<u8>> {
        if self.textarea.lines().len() > 1 && multiline == MultilineSubmitPolicy::Reject {
            return Err(
                "multiline shell submission requires explicit literal-multiline policy; refusing silent line-by-line execution"
                    .into(),
            );
        }

        let mut bytes = self.textarea.lines().join("\n").into_bytes();
        match terminator {
            ShellSubmitTerminator::LineFeed => bytes.push(b'\n'),
            ShellSubmitTerminator::CarriageReturnLineFeed => bytes.extend_from_slice(b"\r\n"),
        }
        Ok(bytes)
    }

    pub(crate) fn submit_selected(
        &mut self,
        state: &mut WorkbenchState,
        terminals: &mut WorkbenchTerminals,
        multiline: MultilineSubmitPolicy,
        terminator: ShellSubmitTerminator,
    ) -> Result<ShellDispatchReceipt> {
        let bytes = self.submitted_bytes(multiline, terminator)?;
        let pane_id = terminals.dispatch_selected_input(state, &bytes)?;
        self.textarea.clear();
        Ok(ShellDispatchReceipt {
            pane_id,
            submitted_bytes: bytes,
        })
    }
}

impl WorkbenchTerminals {
    pub(crate) fn dispatch_selected_input(
        &mut self,
        state: &mut WorkbenchState,
        bytes: &[u8],
    ) -> Result<PaneId> {
        let pane_id = state
            .selected_dispatch_candidate()
            .ok_or("workbench shell submit requires one explicitly selected pane")?;
        self.require_live_owned(state, pane_id)?;

        let send = self
            .terminals
            .get_mut(&pane_id)
            .expect("selected live-owned pane was just proven")
            .session
            .send_input(bytes);
        if send.is_err() {
            self.refresh_after_operation_error(state, pane_id);
        }
        send?;
        Ok(pane_id)
    }
}

#[cfg(test)]
#[path = "t091_workbench_input_tests.rs"]
mod t091_workbench_input_tests;
