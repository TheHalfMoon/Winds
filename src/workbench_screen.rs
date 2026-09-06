use super::PaneSize;
use std::collections::VecDeque;

pub(crate) const MAX_TRANSCRIPT_LINES: usize = 100_000;
pub(crate) const MAX_TRANSCRIPT_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_OSC_INPUT_BYTES: usize = 8 * 1024;
const VT100_SCROLLBACK_LINES: usize = 0;
const TERMINAL_DATA_AUTHORITY: &str = "TERMINAL_DATA_ONLY";
const ESCAPE: u8 = 0x1b;
const BELL: u8 = 0x07;
const CANCEL: u8 = 0x18;
const SUBSTITUTE: u8 = 0x1a;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TerminalCallbackSummary {
    pub(crate) total_requests: u64,
    pub(crate) audible_bell_requests: u64,
    pub(crate) visual_bell_requests: u64,
    pub(crate) resize_requests: u64,
    pub(crate) icon_name_requests: u64,
    pub(crate) title_requests: u64,
    pub(crate) clipboard_copy_requests: u64,
    pub(crate) clipboard_paste_requests: u64,
    pub(crate) unhandled_char_requests: u64,
    pub(crate) unhandled_control_requests: u64,
    pub(crate) unhandled_escape_requests: u64,
    pub(crate) unhandled_csi_requests: u64,
    pub(crate) unhandled_osc_requests: u64,
}

#[derive(Debug, Default)]
struct FailClosedCallbacks {
    summary: TerminalCallbackSummary,
    suppressed: bool,
}

impl FailClosedCallbacks {
    fn record(counter: &mut u64, total: &mut u64) {
        *counter = counter.saturating_add(1);
        *total = total.saturating_add(1);
    }
}

impl vt100::Callbacks for FailClosedCallbacks {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.audible_bell_requests,
            &mut self.summary.total_requests,
        );
    }

    fn visual_bell(&mut self, _: &mut vt100::Screen) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.visual_bell_requests,
            &mut self.summary.total_requests,
        );
    }

    fn resize(&mut self, _: &mut vt100::Screen, _: (u16, u16)) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.resize_requests,
            &mut self.summary.total_requests,
        );
    }

    fn set_window_icon_name(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.icon_name_requests,
            &mut self.summary.total_requests,
        );
    }

    fn set_window_title(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.title_requests,
            &mut self.summary.total_requests,
        );
    }

    fn copy_to_clipboard(&mut self, _: &mut vt100::Screen, _: &[u8], _: &[u8]) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.clipboard_copy_requests,
            &mut self.summary.total_requests,
        );
    }

    fn paste_from_clipboard(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.clipboard_paste_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_char(&mut self, _: &mut vt100::Screen, _: char) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.unhandled_char_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_control(&mut self, _: &mut vt100::Screen, _: u8) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.unhandled_control_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_escape(&mut self, _: &mut vt100::Screen, _: Option<u8>, _: Option<u8>, _: u8) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.unhandled_escape_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_csi(
        &mut self,
        _: &mut vt100::Screen,
        _: Option<u8>,
        _: Option<u8>,
        _: &[&[u16]],
        _: char,
    ) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.unhandled_csi_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, _: &[&[u8]]) {
        if self.suppressed {
            return;
        }
        Self::record(
            &mut self.summary.unhandled_osc_requests,
            &mut self.summary.total_requests,
        );
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TerminalInputGuardSummary {
    pub(crate) dropped_oversized_osc_sequences: u64,
}

#[derive(Debug, Default)]
struct OscInputGuard {
    escape_pending: bool,
    in_osc: bool,
    dropping_oversized_osc: bool,
    osc_input_bytes: usize,
    summary: TerminalInputGuardSummary,
}

impl OscInputGuard {
    fn observe_non_osc_byte(&mut self, byte: u8) {
        if self.escape_pending && byte == b']' {
            self.escape_pending = false;
            self.in_osc = true;
            self.osc_input_bytes = 0;
            return;
        }

        if byte == ESCAPE {
            self.escape_pending = true;
        } else if self.escape_pending && is_escape_ignored_control(byte) {
            // VTE executes these C0 controls without leaving Escape state.
        } else {
            self.escape_pending = false;
        }
    }

    fn observe_osc_terminator(&mut self, byte: u8) {
        self.in_osc = false;
        self.dropping_oversized_osc = false;
        self.osc_input_bytes = 0;
        self.escape_pending = byte == ESCAPE;
    }

    fn record_oversized_osc(&mut self) {
        self.in_osc = false;
        self.dropping_oversized_osc = true;
        self.osc_input_bytes = 0;
        self.escape_pending = false;
        self.summary.dropped_oversized_osc_sequences = self
            .summary
            .dropped_oversized_osc_sequences
            .saturating_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TranscriptSnapshot {
    pub(crate) lines: Vec<Vec<u8>>,
    pub(crate) retained_bytes: usize,
    pub(crate) evicted_lines: u64,
    pub(crate) evicted_bytes: u64,
    pub(crate) truncated: bool,
}

#[derive(Debug)]
struct BoundedTranscript {
    completed_lines: VecDeque<VecDeque<u8>>,
    partial_line: VecDeque<u8>,
    retained_bytes: usize,
    evicted_lines: u64,
    evicted_bytes: u64,
    truncated: bool,
    max_lines: usize,
    max_bytes: usize,
}

impl BoundedTranscript {
    fn new(max_lines: usize, max_bytes: usize) -> Result<Self, &'static str> {
        if max_lines == 0 || max_bytes == 0 {
            return Err("transcript bounds must be positive");
        }
        Ok(Self {
            completed_lines: VecDeque::new(),
            partial_line: VecDeque::new(),
            retained_bytes: 0,
            evicted_lines: 0,
            evicted_bytes: 0,
            truncated: false,
            max_lines,
            max_bytes,
        })
    }

    fn push(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.partial_line.push_back(byte);
            self.retained_bytes = self.retained_bytes.saturating_add(1);
            if byte == b'\n' {
                self.completed_lines
                    .push_back(self.partial_line.drain(..).collect());
            }
            self.enforce_bounds();
        }
    }

    fn retained_line_count(&self) -> usize {
        self.completed_lines.len() + usize::from(!self.partial_line.is_empty())
    }

    fn enforce_bounds(&mut self) {
        while self.retained_line_count() > self.max_lines {
            let Some(line) = self.completed_lines.pop_front() else {
                break;
            };
            self.record_eviction(line.len(), true);
        }

        while self.retained_bytes > self.max_bytes {
            let excess = self.retained_bytes - self.max_bytes;
            if let Some(front) = self.completed_lines.front_mut() {
                if front.len() <= excess {
                    let line = self
                        .completed_lines
                        .pop_front()
                        .expect("front line was just observed");
                    self.record_eviction(line.len(), true);
                } else {
                    front.drain(..excess);
                    self.record_eviction(excess, false);
                }
            } else {
                let remove = excess.min(self.partial_line.len());
                self.partial_line.drain(..remove);
                self.record_eviction(remove, false);
            }
        }
    }

    fn record_eviction(&mut self, bytes: usize, whole_line: bool) {
        self.retained_bytes = self.retained_bytes.saturating_sub(bytes);
        self.evicted_bytes = self
            .evicted_bytes
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
        if whole_line {
            self.evicted_lines = self.evicted_lines.saturating_add(1);
        }
        if bytes > 0 || whole_line {
            self.truncated = true;
        }
    }

    fn snapshot(&self) -> TranscriptSnapshot {
        let mut lines: Vec<Vec<u8>> = self
            .completed_lines
            .iter()
            .map(|line| line.iter().copied().collect())
            .collect();
        if !self.partial_line.is_empty() {
            lines.push(self.partial_line.iter().copied().collect());
        }
        TranscriptSnapshot {
            lines,
            retained_bytes: self.retained_bytes,
            evicted_lines: self.evicted_lines,
            evicted_bytes: self.evicted_bytes,
            truncated: self.truncated,
        }
    }
}

pub(crate) struct WorkbenchScreen {
    parser: vt100::Parser<FailClosedCallbacks>,
    transcript: BoundedTranscript,
    osc_guard: OscInputGuard,
}

impl WorkbenchScreen {
    pub(crate) fn new(size: PaneSize) -> Result<Self, &'static str> {
        Self::with_limits(size, MAX_TRANSCRIPT_LINES, MAX_TRANSCRIPT_BYTES)
    }

    fn with_limits(
        size: PaneSize,
        max_lines: usize,
        max_bytes: usize,
    ) -> Result<Self, &'static str> {
        validate_size(size)?;
        Ok(Self {
            parser: vt100::Parser::new_with_callbacks(
                size.rows,
                size.columns,
                VT100_SCROLLBACK_LINES,
                FailClosedCallbacks::default(),
            ),
            transcript: BoundedTranscript::new(max_lines, max_bytes)?,
            osc_guard: OscInputGuard::default(),
        })
    }

    #[cfg(test)]
    pub(super) fn with_test_limits(
        size: PaneSize,
        max_lines: usize,
        max_bytes: usize,
    ) -> Result<Self, &'static str> {
        Self::with_limits(size, max_lines, max_bytes)
    }

    pub(crate) fn process_observed_bytes(&mut self, bytes: &[u8]) {
        self.transcript.push(bytes);
        for &byte in bytes {
            self.process_guarded_byte(byte);
        }
    }

    fn process_guarded_byte(&mut self, byte: u8) {
        if self.osc_guard.dropping_oversized_osc {
            if is_osc_terminator(byte) {
                self.process_parser_byte_suppressed(byte);
                self.osc_guard.observe_osc_terminator(byte);
            }
            return;
        }

        if self.osc_guard.in_osc {
            if is_osc_terminator(byte) {
                self.parser.process(&[byte]);
                self.osc_guard.observe_osc_terminator(byte);
                return;
            }

            if self.osc_guard.osc_input_bytes >= MAX_OSC_INPUT_BYTES {
                self.process_parser_byte_suppressed(BELL);
                self.osc_guard.record_oversized_osc();
                return;
            }

            self.parser.process(&[byte]);
            self.osc_guard.osc_input_bytes += 1;
            return;
        }

        self.parser.process(&[byte]);
        self.osc_guard.observe_non_osc_byte(byte);
    }

    fn process_parser_byte_suppressed(&mut self, byte: u8) {
        self.parser.callbacks_mut().suppressed = true;
        self.parser.process(&[byte]);
        self.parser.callbacks_mut().suppressed = false;
    }

    pub(crate) fn explicit_resize(&mut self, size: PaneSize) -> Result<(), &'static str> {
        validate_size(size)?;
        self.parser.screen_mut().set_size(size.rows, size.columns);
        Ok(())
    }

    pub(crate) fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    pub(crate) fn screen_contents(&self) -> String {
        self.parser.screen().contents()
    }

    pub(crate) fn callback_summary(&self) -> TerminalCallbackSummary {
        self.parser.callbacks().summary
    }

    pub(crate) fn input_guard_summary(&self) -> TerminalInputGuardSummary {
        self.osc_guard.summary
    }

    pub(crate) fn transcript_snapshot(&self) -> TranscriptSnapshot {
        self.transcript.snapshot()
    }

    pub(crate) const fn presentation_authority(&self) -> &'static str {
        TERMINAL_DATA_AUTHORITY
    }
}

fn is_osc_terminator(byte: u8) -> bool {
    matches!(byte, BELL | CANCEL | SUBSTITUTE | ESCAPE)
}

fn is_escape_ignored_control(byte: u8) -> bool {
    matches!(byte, 0x00..=0x17 | 0x19 | 0x1c..=0x1f)
}

fn validate_size(size: PaneSize) -> Result<(), &'static str> {
    if size.columns == 0 || size.rows == 0 {
        return Err("terminal screen dimensions must be positive");
    }
    Ok(())
}

#[cfg(test)]
#[path = "t089_workbench_screen_tests.rs"]
mod t089_workbench_screen_tests;
