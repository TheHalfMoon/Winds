//! Deterministic, bounded, presentation-only terminal UX behavior.
//!
//! This module owns local presentation state only. It never launches a process,
//! opens a host target, performs a network fetch, mutates topology, or grants
//! process/controller authority. Consequential effects must pass through the
//! already accepted exact-target authority seams.

use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};
use std::collections::VecDeque;

/// Maximum number of retained presentation rows for one pane.
pub(crate) const MAX_SCROLLBACK_LINES: usize = 10_000;
/// AD-012-14 pane-local rendered/parsed working-buffer ceiling.
pub(crate) const MAX_PANE_WORKING_BUFFER_BYTES: usize = 2 * 1024 * 1024;
/// Maximum number of cells materialized by one selection.
pub(crate) const MAX_SELECTION_CELLS: usize = 256 * 1024;
/// Maximum number of reported literal-search matches.
pub(crate) const MAX_SEARCH_MATCHES: usize = 1_000;
/// Maximum length of one literal search needle.
pub(crate) const MAX_SEARCH_NEEDLE_BYTES: usize = 2_048;
/// Maximum length of one presentation-only pane title.
pub(crate) const MAX_CHROME_TITLE_BYTES: usize = 512;
/// Maximum length of one offered link target.
pub(crate) const MAX_LINK_TARGET_BYTES: usize = 2_048;
/// AD-012-14 graphics payload ceiling retained per pane.
pub(crate) const MAX_GRAPHICS_FRAME_BYTES: usize = 8 * 1024 * 1024;
/// Maximum payload of one local clipboard offer.
pub(crate) const MAX_CLIPBOARD_OFFER_BYTES: usize = 64 * 1024;
/// AD-012-14 maximum pending notifications.
pub(crate) const MAX_NOTIFICATIONS: usize = 128;
/// Deterministic notification burst window.
pub(crate) const NOTIFICATION_WINDOW_MS: i64 = 5_000;

pub(crate) const OFFERABLE_LINK_SCHEMES: [&str; 2] = ["https", "http"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GridPoint {
    pub(crate) row: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GridCell {
    pub(crate) point: GridPoint,
    pub(crate) byte: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectionError {
    TooLarge,
}

/// A deterministic selection over caller-supplied terminal bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalSelection {
    anchor: GridPoint,
    head: GridPoint,
}

impl TerminalSelection {
    pub(crate) const fn new(anchor: GridPoint, head: GridPoint) -> Self {
        Self { anchor, head }
    }

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

    pub(crate) fn ordered(&self) -> (GridPoint, GridPoint) {
        if (self.anchor.row, self.anchor.column) <= (self.head.row, self.head.column) {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

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
            if first_column >= width {
                continue;
            }
            let requested_last = if row == end.row { end.column } else { width };
            let last_column = requested_last.min(width - 1);
            count = count.saturating_add(last_column.saturating_sub(first_column) + 1);
            if count > MAX_SELECTION_CELLS {
                return count;
            }
        }
        count
    }

    pub(crate) fn cells(&self, lines: &[Vec<u8>]) -> Result<Vec<GridCell>, SelectionError> {
        if self.cell_count(lines) > MAX_SELECTION_CELLS {
            return Err(SelectionError::TooLarge);
        }
        let (start, end) = self.ordered();
        let mut cells = Vec::new();
        for row in start.row..=end.row {
            let line = lines.get(row).map(Vec::as_slice).unwrap_or(&[]);
            let first_column = if row == start.row { start.column } else { 0 };
            if first_column >= line.len() {
                continue;
            }
            let requested_last = if row == end.row {
                end.column
            } else {
                line.len()
            };
            let last_column = requested_last.min(line.len().saturating_sub(1));
            for column in first_column..=last_column {
                if let Some(byte) = line.get(column).copied() {
                    cells.push(GridCell {
                        point: GridPoint { row, column },
                        byte,
                    });
                }
            }
        }
        Ok(cells)
    }

    /// Copies the selected byte stream without widening each UTF-8 byte into an
    /// unrelated Unicode scalar. Bytes are preserved exactly, orphan partial
    /// sequences at either edge are dropped, and the remaining stream is decoded
    /// with `String::from_utf8_lossy` so complete multi-byte input stays intact
    /// and any still-invalid interior byte becomes a replacement character
    /// instead of a panic.
    pub(crate) fn copy_text(&self, lines: &[Vec<u8>]) -> Result<String, SelectionError> {
        let cells = self.cells(lines)?;
        let mut bytes = Vec::with_capacity(cells.len());
        let mut previous_row = None;
        for cell in cells {
            if previous_row.is_some_and(|row| row != cell.point.row) {
                bytes.push(b'\n');
            }
            bytes.push(cell.byte);
            previous_row = Some(cell.point.row);
        }
        Ok(decode_copied_bytes(&bytes))
    }
}

/// Drops a leading run of orphan UTF-8 continuation bytes, then decodes the rest.
/// A trailing incomplete sequence is dropped rather than rendered as U+FFFD so a
/// drag that stops inside a wide glyph copies the glyphs it fully covered.
fn decode_copied_bytes(bytes: &[u8]) -> String {
    let start = bytes
        .iter()
        .position(|byte| !is_utf8_continuation(*byte))
        .unwrap_or(bytes.len());
    let body = &bytes[start..];
    match std::str::from_utf8(body) {
        Ok(text) => text.to_owned(),
        Err(error) => {
            let valid_up_to = error.valid_up_to();
            if error.error_len().is_none() {
                String::from_utf8_lossy(&body[..valid_up_to]).into_owned()
            } else {
                String::from_utf8_lossy(body).into_owned()
            }
        }
    }
}

fn is_utf8_continuation(byte: u8) -> bool {
    byte & 0b1100_0000 == 0b1000_0000
}

fn head_start(line: &[u8], column: usize) -> usize {
    let column = column.min(line.len());
    if column >= line.len() || !is_word_byte(line[column]) {
        return column;
    }
    word_bounds(line, column).0
}

/// Word bytes are ASCII word characters plus every non-ASCII byte. Treating a
/// multi-byte UTF-8 run as one word keeps CJK/IME double-click selection aligned
/// to whole characters instead of slicing a single byte out of a glyph.
fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchMatch {
    pub(crate) row: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SearchResult {
    pub(crate) matches: Vec<SearchMatch>,
    pub(crate) truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalSearchQuery {
    pub(crate) needle: Vec<u8>,
    pub(crate) case_sensitive: bool,
    pub(crate) whole_word: bool,
}

impl TerminalSearchQuery {
    pub(crate) fn new(
        needle: &[u8],
        case_sensitive: bool,
        whole_word: bool,
    ) -> Result<Self, &'static str> {
        if needle.is_empty() {
            return Err("terminal search needle must not be empty");
        }
        if needle.len() > MAX_SEARCH_NEEDLE_BYTES {
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

pub(crate) fn search_lines(lines: &[Vec<u8>], query: &TerminalSearchQuery) -> SearchResult {
    let mut matches = Vec::new();
    let mut truncated = false;
    'rows: for (row, line) in lines.iter().enumerate() {
        for column in 0..line.len() {
            if !query.matches_at(line, column) {
                continue;
            }
            if matches.len() >= MAX_SEARCH_MATCHES {
                truncated = true;
                break 'rows;
            }
            matches.push(SearchMatch { row, column });
        }
    }
    SearchResult { matches, truncated }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LinkRefusal {
    TooLong,
    MissingScheme,
    UnsupportedScheme,
    ControlBytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkTarget {
    pub(crate) scheme: &'static str,
    pub(crate) target: String,
}

pub(crate) fn parse_link_target(raw: &[u8]) -> Result<LinkTarget, LinkRefusal> {
    if raw.len() > MAX_LINK_TARGET_BYTES {
        return Err(LinkRefusal::TooLong);
    }
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
        target: String::from_utf8_lossy(raw).into_owned(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LinkActivationGrant {
    pub(crate) target: LinkTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkOffer {
    Refused,
    Pending(LinkTarget),
    Granted(LinkActivationGrant),
}

impl LinkOffer {
    pub(crate) fn offer(raw: &[u8]) -> Self {
        match parse_link_target(raw) {
            Ok(target) => Self::Pending(target),
            Err(_) => Self::Refused,
        }
    }

    pub(crate) fn accept(self) -> Self {
        match self {
            Self::Pending(target) => Self::Granted(LinkActivationGrant { target }),
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClipboardOffer {
    pub(crate) source: ClipboardSource,
    pub(crate) payload: Vec<u8>,
}

/// The only clipboard source Spec 012 can represent. Remote clipboard bridging is
/// prohibited, so no remote variant exists to construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClipboardSource {
    LocalOnly,
}

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
        source: ClipboardSource::LocalOnly,
        payload: payload.to_vec(),
    })
}

/// A clipboard write is only representable after an explicit user/policy
/// acceptance bound to the exact pane, so an offer never becomes a host write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClipboardGrant {
    pub(crate) target: ExactPaneTarget,
    pub(crate) payload: Vec<u8>,
}

impl ClipboardOffer {
    pub(crate) fn accept(
        self,
        target: ExactPaneTarget,
        presented_generation: TopologyGeneration,
    ) -> Option<ClipboardGrant> {
        if target.topology_generation != presented_generation {
            return None;
        }
        Some(ClipboardGrant {
            target,
            payload: self.payload,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GraphicsRefusal {
    TooLarge,
    RemoteReference,
    StalePane,
}

/// A retained local graphics frame. FR-024 requires bounded payload semantics
/// and exact pane binding, so the frame carries the immutable pane identity that
/// produced it and can never be replayed onto another pane or generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GraphicsFrame {
    pub(crate) target: ExactPaneTarget,
    pub(crate) bytes: Vec<u8>,
}

fn contains_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
}

pub(crate) fn accept_graphics_frame(
    target: ExactPaneTarget,
    presented_generation: TopologyGeneration,
    payload: &[u8],
) -> Result<GraphicsFrame, GraphicsRefusal> {
    if target.topology_generation != presented_generation {
        return Err(GraphicsRefusal::StalePane);
    }
    if payload.len() > MAX_GRAPHICS_FRAME_BYTES {
        return Err(GraphicsRefusal::TooLarge);
    }
    if contains_ascii_case_insensitive(payload, b"http://")
        || contains_ascii_case_insensitive(payload, b"https://")
    {
        return Err(GraphicsRefusal::RemoteReference);
    }
    Ok(GraphicsFrame {
        target,
        bytes: payload.to_vec(),
    })
}

/// Presentation-only pane chrome. Titles, accents, borders, gaps, and scrollbars
/// are user-controlled display text; none of them can carry authority, so the
/// type exposes no trusted-state field at all (FR-025).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneAccent {
    Default,
    Running,
    Attention,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneChrome {
    pub(crate) title: String,
    pub(crate) accent: PaneAccent,
}

impl PaneChrome {
    pub(crate) fn new(title: &[u8]) -> Self {
        let mut bounded = title.to_vec();
        bounded.truncate(MAX_CHROME_TITLE_BYTES);
        Self {
            title: String::from_utf8_lossy(&bounded).into_owned(),
            accent: PaneAccent::Default,
        }
    }

    pub(crate) const fn with_accent(mut self, accent: PaneAccent) -> Self {
        self.accent = accent;
        self
    }

    /// Chrome can only ever be display state.
    pub(crate) const fn encodes_trusted_authority(&self) -> bool {
        false
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalNotification {
    pub(crate) now_unix_ms: i64,
}

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

    pub(crate) fn suppress(&mut self) {
        self.suppressed = true;
    }

    pub(crate) const fn is_suppressed(&self) -> bool {
        self.suppressed
    }

    pub(crate) const fn total(&self) -> u64 {
        self.total
    }

    pub(crate) fn retained(&self) -> usize {
        self.retained.len()
    }

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

/// FR-023 requires CJK/IME/input-source behavior to be directly exercised on
/// every claimed platform domain. T171 runs no native Windows, macOS, Linux, or
/// WSL input-method evidence, so the recorded claim state is `UNPROVEN` and the
/// production claim type below has no constructor that can produce anything but
/// `Unclaimed`.
pub(crate) const IME_SUPPORT_CLAIM_STATE: &str = "UNPROVEN";

/// T171 cannot manufacture a CJK/IME/input-source claim. Direct native-platform
/// claim admission belongs to T183; the truthful T171 production state is only
/// `Unclaimed`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum InputMethodClaim {
    #[default]
    Unclaimed,
}

impl InputMethodClaim {
    pub(crate) const fn unclaimed() -> Self {
        Self::Unclaimed
    }

    pub(crate) const fn is_claimed(&self) -> bool {
        false
    }

    pub(crate) const fn claim_state(&self) -> &'static str {
        IME_SUPPORT_CLAIM_STATE
    }
}

/// Pane-local bounded presentation scrollback. Both line count and retained bytes
/// are bounded; a single oversized row retains only its newest bounded bytes.
#[derive(Debug, Clone)]
pub(crate) struct BoundedScrollback {
    rows: VecDeque<Vec<u8>>,
    retained_bytes: usize,
    dropped_rows: u64,
    dropped_bytes: u64,
    truncated: bool,
}

impl BoundedScrollback {
    pub(crate) fn new() -> Self {
        Self {
            rows: VecDeque::new(),
            retained_bytes: 0,
            dropped_rows: 0,
            dropped_bytes: 0,
            truncated: false,
        }
    }

    pub(crate) fn push(&mut self, mut row: Vec<u8>) {
        if row.len() > MAX_PANE_WORKING_BUFFER_BYTES {
            let excess = row.len() - MAX_PANE_WORKING_BUFFER_BYTES;
            row.drain(..excess);
            self.dropped_bytes = self
                .dropped_bytes
                .saturating_add(u64::try_from(excess).unwrap_or(u64::MAX));
            self.truncated = true;
        }
        self.retained_bytes = self.retained_bytes.saturating_add(row.len());
        self.rows.push_back(row);
        while self.rows.len() > MAX_SCROLLBACK_LINES
            || self.retained_bytes > MAX_PANE_WORKING_BUFFER_BYTES
        {
            let Some(dropped) = self.rows.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(dropped.len());
            self.dropped_rows = self.dropped_rows.saturating_add(1);
            self.dropped_bytes = self
                .dropped_bytes
                .saturating_add(u64::try_from(dropped.len()).unwrap_or(u64::MAX));
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

    pub(crate) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(crate) const fn dropped_rows(&self) -> u64 {
        self.dropped_rows
    }

    pub(crate) const fn dropped_bytes(&self) -> u64 {
        self.dropped_bytes
    }

    pub(crate) const fn truncated(&self) -> bool {
        self.truncated
    }
}

impl Default for BoundedScrollback {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable pane identity captured at the presentation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExactPaneTarget {
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) tab_id: TabId,
    pub(crate) pane_id: PaneId,
    pub(crate) topology_generation: TopologyGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerPhase {
    Down,
    Drag,
    Up,
}

/// Local gesture semantics only; none of these variants grants runtime authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneInteractionKind {
    MouseCapture {
        button: PointerButton,
        phase: PointerPhase,
    },
    ContextMenu,
    CopyOnSelect,
    Scroll {
        vertical_lines: i16,
        horizontal_columns: i16,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BoundPaneInteraction {
    target: ExactPaneTarget,
    kind: PaneInteractionKind,
    column: u16,
    row: u16,
}

impl BoundPaneInteraction {
    pub(crate) const fn target(self) -> ExactPaneTarget {
        self.target
    }

    pub(crate) const fn kind(self) -> PaneInteractionKind {
        self.kind
    }

    pub(crate) const fn column(self) -> u16 {
        self.column
    }

    pub(crate) const fn row(self) -> u16 {
        self.row
    }

    pub(crate) const fn is_current_generation(self, generation: TopologyGeneration) -> bool {
        self.target.topology_generation.get() == generation.get()
    }
}

/// Binds a terminal gesture to the pane identity/generation visible when the
/// gesture was captured. A stale generation fails closed rather than retargeting
/// to whatever pane later receives focus.
pub(crate) fn bind_exact_pane_interaction(
    target: ExactPaneTarget,
    presented_generation: TopologyGeneration,
    kind: PaneInteractionKind,
    column: u16,
    row: u16,
) -> Option<BoundPaneInteraction> {
    if target.topology_generation != presented_generation {
        return None;
    }
    Some(BoundPaneInteraction {
        target,
        kind,
        column,
        row,
    })
}

#[cfg(test)]
#[path = "t171_terminal_ux_tests.rs"]
mod t171_terminal_ux_tests;
