use super::PaneId;
use super::screen::TranscriptSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InteractionSource {
    UserInput,
    TerminalOutput,
    AgentReported,
    WindsObservedEvidence,
    WarningPolicy,
    HumanDecision,
}

impl InteractionSource {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::UserInput => "USER_INPUT",
            Self::TerminalOutput => "TERMINAL_OUTPUT",
            Self::AgentReported => "AGENT_REPORTED",
            Self::WindsObservedEvidence => "WINDS_OBSERVED_EVIDENCE",
            Self::WarningPolicy => "WARNING_POLICY",
            Self::HumanDecision => "HUMAN_DECISION",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InteractionBoundary {
    ExplicitSource,
    ContinuousTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InteractionContext {
    pub(crate) pane_id: Option<PaneId>,
    pub(crate) canonical_workspace_id: Option<String>,
    pub(crate) canonical_winds_session_id: Option<String>,
}

impl InteractionContext {
    pub(crate) fn new(
        pane_id: Option<PaneId>,
        canonical_workspace_id: Option<String>,
        canonical_winds_session_id: Option<String>,
    ) -> Self {
        Self {
            pane_id,
            canonical_workspace_id,
            canonical_winds_session_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InteractionPresentation {
    pub(crate) source: InteractionSource,
    pub(crate) boundary: InteractionBoundary,
    pub(crate) raw_content: Vec<u8>,
    pub(crate) context: InteractionContext,
}

impl InteractionPresentation {
    pub(crate) fn labelled(
        source: InteractionSource,
        content: impl AsRef<[u8]>,
        context: InteractionContext,
    ) -> Self {
        let boundary = if source == InteractionSource::TerminalOutput {
            InteractionBoundary::ContinuousTerminal
        } else {
            InteractionBoundary::ExplicitSource
        };
        Self {
            source,
            boundary,
            raw_content: content.as_ref().to_vec(),
            context,
        }
    }

    /// Source labels describe presentation provenance only. Canonical evidence
    /// and human-decision authority remain owned by their existing repository paths.
    pub(crate) const fn changes_canonical_authority(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TranscriptRetentionView {
    pub(crate) retained_lines: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) evicted_lines: u64,
    pub(crate) evicted_bytes: u64,
    pub(crate) truncated: bool,
}

impl TranscriptRetentionView {
    pub(crate) fn from_snapshot(snapshot: &TranscriptSnapshot) -> Self {
        Self {
            retained_lines: snapshot.lines.len(),
            retained_bytes: snapshot.retained_bytes,
            evicted_lines: snapshot.evicted_lines,
            evicted_bytes: snapshot.evicted_bytes,
            truncated: snapshot.truncated,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TranscriptSearchMatch {
    pub(crate) retained_line_index: usize,
    pub(crate) presentation: InteractionPresentation,
    pub(crate) retention: TranscriptRetentionView,
}

pub(crate) fn search_terminal_transcript(
    snapshot: &TranscriptSnapshot,
    context: &InteractionContext,
    query: &str,
) -> Vec<TranscriptSearchMatch> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let retention = TranscriptRetentionView::from_snapshot(snapshot);
    snapshot
        .lines
        .iter()
        .enumerate()
        .filter_map(|(retained_line_index, line)| {
            let searchable = normalize(&String::from_utf8_lossy(line));
            searchable
                .contains(&normalized_query)
                .then(|| TranscriptSearchMatch {
                    retained_line_index,
                    presentation: InteractionPresentation::labelled(
                        InteractionSource::TerminalOutput,
                        line,
                        context.clone(),
                    ),
                    retention,
                })
        })
        .collect()
}

fn normalize(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

#[cfg(test)]
#[path = "t093_workbench_interaction_tests.rs"]
mod t093_workbench_interaction_tests;
