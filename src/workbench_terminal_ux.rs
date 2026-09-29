//! Deterministic, bounded, presentation-only terminal UX behavior.
//!
//! Everything in this module produces presentation state: bounded scrollback,
//! selection, search results, link offers, clipboard offers, graphics frames,
//! notifications, and the input-method claim. Nothing here launches a process,
//! performs a network fetch, opens a path, mutates a workspace, or grants
//! authority. A caller must still route any real effect through its own exact
//! authority check.

use std::collections::VecDeque;

/// Bounded scrollback retained per pane. The bound is explicit: once the line
/// budget is reached, older rows are dropped and truncation becomes observable.
pub(crate) const MAX_SCROLLBACK_LINES: usize = 10_000;
/// Hard ceiling on one selection, so a runaway drag cannot grow unbounded.
pub(crate) const MAX_SELECTION_CELLS: usize = 256 * 1024;
/// Hard ceiling on reported search matches.
pub(crate) const MAX_SEARCH_MATCHES: usize = 1_000;
/// Hard ceiling on one link target, in bytes.
pub(crate) const MAX_LINK_TARGET_BYTES: usize = 2_048;
/// Plan ceiling AD-012-14: the pane-local retained graphics payload.
pub(crate) const MAX_GRAPHICS_FRAME_BYTES: usize = 8 * 1024 * 1024;
/// Plan ceiling AD-012-14: the pane-local rendered and parsed working buffer.
pub(crate) const MAX_PANE_WORKING_BUFFER_BYTES: usize = 2 * 1024 * 1024;
/// Hard ceiling on one clipboard offer, in bytes.
pub(crate) const MAX_CLIPBOARD_OFFER_BYTES: usize = 64 * 1024;
/// Plan ceiling AD-012-14: the notification pending queue.
pub(crate) const MAX_NOTIFICATIONS: usize = 128;
/// The notification window, in milliseconds.
pub(crate) const NOTIFICATION_WINDOW_MS: i64 = 5_000;

/// The link schemes a terminal is permitted to offer.
///
/// A local `file` scheme is deliberately absent. Opening a local path would be a
/// host filesystem action taken from bytes a child process printed, so it is
/// never offered regardless of user intent.
pub(crate) const OFFERABLE_LINK_SCHEMES: [&str; 2] = ["https", "http"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GridPoint {
    pub(crate) row: usize,
    pub(crate) column: usize,
}

/// One exact cell in a terminal grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GridCell {
    pub(crate) point: GridPoint,
    pub(crate) byte: u8,
}

/// Why a selection could not be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionError {
    /// The selection would exceed the bounded cell budget.
    TooLarge,
}

/// A bounded, deterministic selection over one exact pane's grid.
///
/// The selection is a pure range over supplied cell content. It carries no
/// authority and cannot read anything the caller did not hand it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalSelection {
    anchor: GridPoint,
    head: GridPoint,
}

impl TerminalSelection {
    /// Starts a selection at an exact cell.
    pub(crate) const fn new(anchor: GridPoint, head: GridPoint) -> Self {
        Self { anchor, head }
    }

    /// Starts a selection at a point and extends it to the word around `head`.
    ///
    /// Both ends are word-aligned, so a word-anchored drag never captures the
    /// gap between two words.
    pub(crate) fn new_word_anchored(
        anchor: GridPoint,
        head: GridPoint,
        lines: &[Vec<u8>],
    ) -> Result<Self, SelectionError> {
        let anchor_line = lines.get(anchor.row).map(Vec::as_slice).unwrap_or(&[]);
        let head_line = lines.get(head.row).map(Vec::as_slice).unwrap_or(&[]);
        let (anchor_start, _) = word_bounds(anchor_line, anchor.column);
        let (_, head_end) = word_bounds(head_line, head.column);
        Ok(Self {
            anchor: GridPoint {
                row: anchor.row,
                column: anchor_start,
            },
            head: GridPoint {
                row: head.row,
                column: head_end
                    .saturating_sub(1)
                    .max(head_start(head_line, head.column)),
            },
        })
    }

    pub(crate) const fn anchor(&self) -> GridPoint {
        self.anchor
    }

    pub(crate) const fn head(&self) -> GridPoint {
        self.head
    }

    /// The selection bounds in document order, independent of drag direction.
    pub(crate) fn ordered(&self) -> (GridPoint, GridPoint) {
        if (self.anchor.row, self.anchor.column) <= (self.head.row, self.head.column) {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// The exact inclusive cell count the selection covers over `lines`.
    ///
    /// The count is computed from the supplied content, so it stays exact for any
    /// real selection and saturates instead of overflowing for a fabricated one.
    pub(crate) fn cell_count(&self, lines: &[Vec<u8>]) -> usize {
        let (start, end) = self.ordered();
        if end.row == start.row {
            let width = lines.get(start.row).map_or(0, Vec::len);
            if start.column >= width {
                return 0;
            }
            return end
                .column
                .min(width - 1)
                .saturating_sub(start.column)
                .saturating_add(1);
        }
        let mut count = 0_usize;
        for row in start.row..=end.row {
            let width = lines.get(row).map_or(0, Vec::len);
            let first_column = if row == start.row { start.column } else { 0 };
            let last_column = if row == end.row { end.column } else { width };
            if first_column >= width {
                continue;
            }
            // Clamp to the same bound `cells` uses, so the budget check and the
            // materialized result can never disagree about a selection's size.
            let last_column = last_column.min(width - 1);
            count = count.saturating_add(last_column.saturating_sub(first_column) + 1);
            if count > MAX_SELECTION_CELLS {
                return count;
            }
        }
        count
    }

    /// The exact cells the selection covers, in document order.
    ///
    /// The result is bounded. A selection larger than the cell budget fails
    /// closed rather than allocating an unbounded copy.
    pub(crate) fn cells(&self, lines: &[Vec<u8>]) -> Result<Vec<GridCell>, SelectionError> {
        let (start, end) = self.ordered();
        if self.cell_count(lines) > MAX_SELECTION_CELLS {
            return Err(SelectionError::TooLarge);
        }
        let mut cells = Vec::new();
        for row in start.row..=end.row {
            let line = lines.get(row).map(Vec::as_slice).unwrap_or(&[]);
            let first_column = if row == start.row { start.column } else { 0 };
            let last_column = if row == end.row {
                end.column
            } else {
                line.len()
            };
            if first_column >= line.len() {
                continue;
            }
            for column in first_column..=last_column.min(line.len().saturating_sub(1)) {
                cells.push(GridCell {
                    point: GridPoint { row, column },
                    byte: line.get(column).copied().unwrap_or(b' '),
                });
            }
        }
        Ok(cells)
    }

    /// The deterministic plain text the selection copies.
    ///
    /// Newlines separate rows, trailing padding is never invented, and the copy
    /// never contains an escape byte that the child did not print.
    pub(crate) fn copy_text(&self, lines: &[Vec<u8>]) -> Result<String, SelectionError> {
        let cells = self.cells(lines)?;
        let mut text = String::with_capacity(cells.len());
        let mut previous_row = None;
        for cell in cells {
            if previous_row.is_some_and(|row| row != cell.point.row) {
                text.push('\n');
            }
            text.push(cell.byte as char);
            previous_row = Some(cell.point.row);
        }
        Ok(text)
    }
}

/// The start column of the word around `column`, used to keep a word-anchored
/// head inside a non-word run.
fn head_start(line: &[u8], column: usize) -> usize {
    let column = column.min(line.len());
    if column >= line.len() || !is_word_byte(line[column]) {
        return column;
    }
    let (start, _) = word_bounds(line, column);
    start
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The deterministic word bounds around `column`.
///
/// Words are maximal runs of ASCII alphanumerics and underscore. Everything else
/// is a separator, so word selection is stable for any given line and column.
fn word_bounds(line: &[u8], column: usize) -> (usize, usize) {
    let column = column.min(line.len());
    if column >= line.len() || !is_word_byte(line[column]) {
        return (column, column);
    }
    let mut start = column;
    while start > 0 && is_word_byte(line[start - 1]) {
        start -= 1;
    }
    let mut end = column + 1;
    while end < line.len() && is_word_byte(line[end]) {
        end += 1;
    }
    (start, end)
}

/// One exact search hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchMatch {
    pub(crate) row: usize,
    pub(crate) column: usize,
}

/// The bounded, deterministic result of one search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchResult {
    pub(crate) matches: Vec<SearchMatch>,
    /// True when matches were dropped because the bound was reached.
    pub(crate) truncated: bool,
}

/// A bounded, literal, case-sensitive-or-insensitive line search.
///
/// The query is matched literally. It is never interpreted as a regular
/// expression, so a pathological pattern cannot cost unbounded work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalSearchQuery {
    pub(crate) needle: Vec<u8>,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_word: bool,
}

impl TerminalSearchQuery {
    /// Rejects an empty or overlong needle rather than matching everything.
    pub(crate) fn new(
        needle: &[u8],
        case_sensitive: bool,
        whole_word: bool,
    ) -> Result<Self, &'static str> {
        if needle.is_empty() {
            return Err("terminal search needle must not be empty");
        }
        if needle.len() > MAX_LINK_TARGET_BYTES {
            return Err("terminal search needle exceeds the bounded length");
        }
        Ok(Self {
            needle: needle.to_vec(),
            case_sensitive,
            whole_word,
        })
    }

    fn matches_at(&self, line: &[u8], column: usize) -> bool {
        let end = column.saturating_add(self.needle.len());
        if end > line.len() {
            return false;
        }
        let candidate = &line[column..end];
        let equal = if self.case_sensitive {
            candidate == self.needle.as_slice()
        } else {
            candidate.eq_ignore_ascii_case(self.needle.as_slice())
        };
        if !equal {
            return false;
        }
        if !self.whole_word {
            return true;
        }
        let before_is_word = column > 0 && is_word_byte(line[column - 1]);
        let after_is_word = end < line.len() && is_word_byte(line[end]);
        !before_is_word && !after_is_word
    }
}

/// Runs the search over bounded line content in deterministic order.
pub(crate) fn search_lines(lines: &[Vec<u8>], query: &TerminalSearchQuery) -> SearchResult {
    let mut matches = Vec::new();
    let mut truncated = false;
    'outer: for (row, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        for column in 0..line.len() {
            if !query.matches_at(line, column) {
                continue;
            }
            if matches.len() >= MAX_SEARCH_MATCHES {
                truncated = true;
                break 'outer;
            }
            matches.push(SearchMatch { row, column });
        }
    }
    SearchResult { matches, truncated }
}

/// Why a link target was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkRefusal {
    /// The target exceeded the bounded length.
    TooLong,
    /// No scheme was present.
    MissingScheme,
    /// The scheme is not offerable, including any local-file scheme.
    UnsupportedScheme,
    /// The target carried embedded control bytes.
    ControlBytes,
}

/// One validated, offerable link target.
///
/// Validation is about what may be *offered*. It never opens anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkTarget {
    pub(crate) scheme: &'static str,
    pub(crate) target: String,
}
/// Validates one OSC 8 target against the offerable scheme set.
///
/// The canonical fail-closed host model decides first. This layer only decides
/// what may be *offered* on top of that verdict, and it never opens anything.
pub(crate) fn parse_link_target(raw: &[u8]) -> Result<LinkTarget, LinkRefusal> {
    if raw.len() > MAX_LINK_TARGET_BYTES {
        return Err(LinkRefusal::TooLong);
    }
    // The accepted host-safety model is the authority on whether a terminal
    // request may ever reach a host action. Only an advisory verdict that still
    // performs no host action can become an offer.
    let assessment = super::screen::host_safety::assess_terminal_host_request(
        super::screen::host_safety::TerminalHostRequestKind::Hyperlink,
        raw,
    );
    if !matches!(
        assessment.disposition,
        super::screen::host_safety::TerminalHostDisposition::AdvisoryOnly
    ) || assessment.can_perform_host_action()
        || assessment.changes_trusted_ui_state()
    {
        return Err(LinkRefusal::UnsupportedScheme);
    }
    if raw.iter().any(|byte| *byte < 0x20 || *byte == 0x7f) {
        return Err(LinkRefusal::ControlBytes);
    }
    let separator = raw
        .iter()
        .position(|byte| *byte == b':')
        .ok_or(LinkRefusal::MissingScheme)?;
    let scheme_bytes = &raw[..separator];
    if scheme_bytes.is_empty() || !scheme_bytes.iter().all(u8::is_ascii_alphabetic) {
        return Err(LinkRefusal::MissingScheme);
    }
    let scheme_text = std::str::from_utf8(scheme_bytes).map_err(|_| LinkRefusal::MissingScheme)?;
    let scheme = OFFERABLE_LINK_SCHEMES
        .iter()
        .copied()
        .find(|candidate| candidate.eq_ignore_ascii_case(scheme_text))
        .ok_or(LinkRefusal::UnsupportedScheme)?;
    Ok(LinkTarget {
        scheme,
        target: String::from_utf8_lossy(&raw[separator + 1..]).into_owned(),
    })
}

/// An explicit grant to activate one exact link target.
///
/// A grant is never implied by the target being valid. It exists only because an
/// accepted user or policy action produced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkActivationGrant {
    pub(crate) target: LinkTarget,
}

/// One offered link, before and after an accepted activation decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkOffer {
    /// The target failed validation and is never offered.
    Refused,
    /// Offered for user review. Nothing has been opened.
    Pending(LinkTarget),
    /// An accepted user or policy action produced this exact grant.
    Granted(LinkActivationGrant),
}

impl LinkOffer {
    /// Validation alone never produces a grant.
    pub(crate) fn offer(raw: &[u8]) -> Self {
        match parse_link_target(raw) {
            Ok(target) => Self::Pending(target),
            Err(_) => Self::Refused,
        }
    }

    /// Records an explicit accepted activation for this exact target.
    pub(crate) fn accept(self) -> Self {
        match self {
            Self::Pending(target) => Self::Granted(LinkActivationGrant { target }),
            other => other,
        }
    }
}

/// One offered clipboard write. The child never writes the clipboard itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClipboardOffer {
    pub(crate) payload: Vec<u8>,
}

/// Accepts one bounded local clipboard offer and refuses anything else.
///
/// The offer is local only: it holds bytes the local child printed. It is never
/// applied on the child's behalf, and the canonical host-safety model still owns
/// the decision to act.
pub(crate) fn offer_clipboard(payload: &[u8]) -> Option<ClipboardOffer> {
    if payload.is_empty() || payload.len() > MAX_CLIPBOARD_OFFER_BYTES {
        return None;
    }
    let assessment = super::screen::host_safety::assess_terminal_host_request(
        super::screen::host_safety::TerminalHostRequestKind::ClipboardWrite,
        payload,
    );
    if assessment.can_perform_host_action() || assessment.changes_trusted_ui_state() {
        return None;
    }
    Some(ClipboardOffer {
        payload: payload.to_vec(),
    })
}

/// Why a graphics payload was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphicsRefusal {
    /// The frame exceeded the bounded size.
    TooLarge,
    /// The frame referenced a remote location, which is never fetched.
    RemoteReference,
}

/// One bounded local graphics frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphicsFrame {
    pub(crate) bytes: Vec<u8>,
}

/// Accepts one bounded local graphics frame and refuses anything remote.
///
/// A remote reference is refused rather than fetched. There is deliberately no
/// code path in this module that can retrieve bytes from a network location.
pub(crate) fn accept_graphics_frame(payload: &[u8]) -> Result<GraphicsFrame, GraphicsRefusal> {
    if payload.len() > MAX_GRAPHICS_FRAME_BYTES {
        return Err(GraphicsRefusal::TooLarge);
    }
    if payload
        .windows(8)
        .any(|window| window[..7].eq_ignore_ascii_case(b"http://"))
        || payload
            .windows(9)
            .any(|window| window[..8].eq_ignore_ascii_case(b"https://"))
    {
        return Err(GraphicsRefusal::RemoteReference);
    }
    Ok(GraphicsFrame {
        bytes: payload.to_vec(),
    })
}

/// Bounded pane-local rendered and parsed working bytes.
///
/// This mirrors the Plan ceiling for a pane-local working buffer. A pane view
/// references bounded owner replay and keeps only this much of its own
/// working state, so the owner replay budget is never multiplied by observers.
#[derive(Debug, Clone, Default)]
pub(crate) struct PaneWorkingBuffer {
    retained_bytes: usize,
    high_water_bytes: usize,
}

impl PaneWorkingBuffer {
    pub(crate) const fn new() -> Self {
        Self {
            retained_bytes: 0,
            high_water_bytes: 0,
        }
    }

    /// Records one bounded chunk. An over-budget chunk is refused whole rather
    /// than partially retained, so a pane can never hold a torn buffer.
    pub(crate) fn retain(&mut self, chunk: &[u8]) -> Result<(), ()> {
        let next = self.retained_bytes.saturating_add(chunk.len());
        if next > MAX_PANE_WORKING_BUFFER_BYTES {
            return Err(());
        }
        self.retained_bytes = next;
        self.high_water_bytes = self.high_water_bytes.max(next);
        Ok(())
    }

    pub(crate) fn release(&mut self, chunk: &[u8]) {
        self.retained_bytes = self.retained_bytes.saturating_sub(chunk.len());
    }

    pub(crate) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(crate) const fn high_water_bytes(&self) -> usize {
        self.high_water_bytes
    }
}

/// One bounded, suppressible terminal notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalNotification {
    pub(crate) now_unix_ms: i64,
}

/// A bounded, suppressible notification budget.
#[derive(Debug, Clone, Default)]
pub(crate) struct NotificationBudget {
    suppressed: bool,
    window_started_unix_ms: Option<i64>,
    window_count: u64,
    total: u64,
    retained: VecDeque<TerminalNotification>,
}

impl NotificationBudget {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Notifications are suppressible, and suppression stops all recording.
    pub(crate) fn suppress(&mut self) {
        self.suppressed = true;
    }

    pub(crate) fn is_suppressed(&self) -> bool {
        self.suppressed
    }

    pub(crate) const fn total(&self) -> u64 {
        self.total
    }

    pub(crate) fn retained(&self) -> usize {
        self.retained.len()
    }

    /// Records one notification inside the bounded window.
    ///
    /// Returns the recorded notification, or `None` when suppressed, out of
    /// window, or over budget. A bell can never grow an unbounded queue.
    pub(crate) fn record(&mut self, now_unix_ms: i64) -> Option<TerminalNotification> {
        if self.suppressed {
            return None;
        }
        match self.window_started_unix_ms {
            Some(started) if now_unix_ms.saturating_sub(started) < NOTIFICATION_WINDOW_MS => {}
            _ => {
                self.window_started_unix_ms = Some(now_unix_ms);
                self.window_count = 0;
                self.retained.clear();
            }
        }
        if self.window_count >= u64::try_from(MAX_NOTIFICATIONS).unwrap_or(u64::MAX) {
            self.total = self.total.saturating_add(1);
            return None;
        }
        self.window_count = self.window_count.saturating_add(1);
        self.total = self.total.saturating_add(1);
        let notification = TerminalNotification { now_unix_ms };
        self.retained.push_back(notification.clone());
        Some(notification)
    }
}

/// The explicit claim state for CJK, IME, and input-source support.
///
/// The claim is gated by native platform evidence. Without that evidence the
/// state stays unclaimed, which is a truthful statement rather than a silent
/// assumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputMethodClaim {
    /// No native platform evidence was produced, so no support is claimed.
    Unclaimed,
    /// Native platform evidence qualified this claim.
    Claimed {
        /// The platform that produced the evidence.
        evidence_platform: String,
        /// The bounded evidence that qualified it.
        evidence: String,
    },
}

impl InputMethodClaim {
    /// Records a claim only when bounded native evidence accompanies it.
    pub(crate) fn claim(evidence_platform: &str, evidence: &str) -> Self {
        if evidence_platform.is_empty() || evidence.is_empty() {
            return Self::Unclaimed;
        }
        Self::Claimed {
            evidence_platform: evidence_platform.to_owned(),
            evidence: evidence.to_owned(),
        }
    }

    pub(crate) const fn is_claimed(&self) -> bool {
        matches!(self, Self::Claimed { .. })
    }
}

/// Bounded scrollback for one exact pane.
///
/// Rows are appended in order and dropped from the front once the line budget is
/// reached. Truncation is explicit and observable, never silent.
#[derive(Debug, Clone)]
pub(crate) struct BoundedScrollback {
    rows: VecDeque<Vec<u8>>,
    dropped_rows: u64,
    truncated: bool,
}

impl BoundedScrollback {
    pub(crate) fn new() -> Self {
        Self {
            rows: VecDeque::new(),
            dropped_rows: 0,
            truncated: false,
        }
    }

    /// Appends one row, dropping the oldest row when the budget is reached.
    pub(crate) fn push(&mut self, row: Vec<u8>) {
        self.rows.push_back(row);
        while self.rows.len() > MAX_SCROLLBACK_LINES {
            self.rows.pop_front();
            self.dropped_rows = self.dropped_rows.saturating_add(1);
            self.truncated = true;
        }
    }

    pub(crate) fn rows(&self) -> &VecDeque<Vec<u8>> {
        &self.rows
    }

    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(crate) const fn truncated(&self) -> bool {
        self.truncated
    }

    pub(crate) const fn dropped_rows(&self) -> u64 {
        self.dropped_rows
    }
}

impl Default for BoundedScrollback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "t171_terminal_ux_tests.rs"]
mod t171_terminal_ux_tests;
