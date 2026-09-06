use super::PaneSize;
use std::collections::VecDeque;

pub(crate) const MAX_TRANSCRIPT_LINES: usize = 100_000;
pub(crate) const MAX_TRANSCRIPT_BYTES: usize = 32 * 1024 * 1024;
const VT100_SCROLLBACK_LINES: usize = 0;
const TERMINAL_DATA_AUTHORITY: &str = "TERMINAL_DATA_ONLY";

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
}

impl FailClosedCallbacks {
    fn record(counter: &mut u64, total: &mut u64) {
        *counter = counter.saturating_add(1);
        *total = total.saturating_add(1);
    }
}

impl vt100::Callbacks for FailClosedCallbacks {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        Self::record(
            &mut self.summary.audible_bell_requests,
            &mut self.summary.total_requests,
        );
    }

    fn visual_bell(&mut self, _: &mut vt100::Screen) {
        Self::record(
            &mut self.summary.visual_bell_requests,
            &mut self.summary.total_requests,
        );
    }

    fn resize(&mut self, _: &mut vt100::Screen, _: (u16, u16)) {
        Self::record(
            &mut self.summary.resize_requests,
            &mut self.summary.total_requests,
        );
    }

    fn set_window_icon_name(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        Self::record(
            &mut self.summary.icon_name_requests,
            &mut self.summary.total_requests,
        );
    }

    fn set_window_title(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        Self::record(
            &mut self.summary.title_requests,
            &mut self.summary.total_requests,
        );
    }

    fn copy_to_clipboard(&mut self, _: &mut vt100::Screen, _: &[u8], _: &[u8]) {
        Self::record(
            &mut self.summary.clipboard_copy_requests,
            &mut self.summary.total_requests,
        );
    }

    fn paste_from_clipboard(&mut self, _: &mut vt100::Screen, _: &[u8]) {
        Self::record(
            &mut self.summary.clipboard_paste_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_char(&mut self, _: &mut vt100::Screen, _: char) {
        Self::record(
            &mut self.summary.unhandled_char_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_control(&mut self, _: &mut vt100::Screen, _: u8) {
        Self::record(
            &mut self.summary.unhandled_control_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_escape(
        &mut self,
        _: &mut vt100::Screen,
        _: Option<u8>,
        _: Option<u8>,
        _: u8,
    ) {
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
        Self::record(
            &mut self.summary.unhandled_csi_requests,
            &mut self.summary.total_requests,
        );
    }

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, _: &[&[u8]]) {
        Self::record(
            &mut self.summary.unhandled_osc_requests,
            &mut self.summary.total_requests,
        );
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
    completed_lines: VecDeque<Vec<u8>>,
    partial_line: Vec<u8>,
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
            partial_line: Vec::new(),
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
            self.partial_line.push(byte);
            self.retained_bytes = self.retained_bytes.saturating_add(1);
            if byte == b'\n' {
                self.completed_lines
                    .push_back(std::mem::take(&mut self.partial_line));
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
        let mut lines: Vec<Vec<u8>> = self.completed_lines.iter().cloned().collect();
        if !self.partial_line.is_empty() {
            lines.push(self.partial_line.clone());
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
        self.parser.process(bytes);
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

    pub(crate) fn transcript_snapshot(&self) -> TranscriptSnapshot {
        self.transcript.snapshot()
    }

    pub(crate) const fn presentation_authority(&self) -> &'static str {
        TERMINAL_DATA_AUTHORITY
    }
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
