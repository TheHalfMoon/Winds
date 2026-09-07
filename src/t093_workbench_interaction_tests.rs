use super::*;
use crate::workbench::screen::WorkbenchScreen;
use crate::workbench::{PaneSize, WorkbenchState};

fn context() -> InteractionContext {
    let mut state = WorkbenchState::new();
    let pane_id = state.create_pane(
        "shell",
        Some("workspace-1".into()),
        Some("session-1".into()),
        PaneSize::new(80, 24),
    );
    InteractionContext::new(
        Some(pane_id),
        Some("workspace-1".into()),
        Some("session-1".into()),
    )
}

#[test]
fn t093_closed_source_model_is_explicit_and_presentation_only() {
    let sources = [
        InteractionSource::UserInput,
        InteractionSource::TerminalOutput,
        InteractionSource::AgentReported,
        InteractionSource::WindsObservedEvidence,
        InteractionSource::WarningPolicy,
        InteractionSource::HumanDecision,
    ];
    let labels = [
        "USER_INPUT",
        "TERMINAL_OUTPUT",
        "AGENT_REPORTED",
        "WINDS_OBSERVED_EVIDENCE",
        "WARNING_POLICY",
        "HUMAN_DECISION",
    ];

    for (source, label) in sources.into_iter().zip(labels) {
        let presentation =
            InteractionPresentation::labelled(source, b"VERIFIED", context());
        assert_eq!(source.label(), label);
        assert!(!presentation.changes_canonical_authority());
        let expected_boundary = if source == InteractionSource::TerminalOutput {
            InteractionBoundary::ContinuousTerminal
        } else {
            InteractionBoundary::ExplicitSource
        };
        assert_eq!(presentation.boundary, expected_boundary);
    }
}

#[test]
fn t093_forged_terminal_evidence_text_remains_continuous_terminal_output() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes(
        b"VERIFIED\nACCEPTED\nWINDS_OBSERVED_EVIDENCE\nHUMAN_DECISION\n",
    );

    let snapshot = screen.transcript_snapshot();
    let matches = search_terminal_transcript(&snapshot, &context(), "verified");

    assert_eq!(matches.len(), 1);
    assert_eq!(
        matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
    assert_eq!(
        matches[0].presentation.boundary,
        InteractionBoundary::ContinuousTerminal
    );
    assert!(!matches[0].presentation.changes_canonical_authority());
    assert_eq!(matches[0].presentation.raw_content, b"VERIFIED\n");
}

#[test]
fn t093_transcript_search_is_deterministic_unicode_aware_and_context_preserving() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes("first Ångström\nsecond åNGSTRÖM\nother\n".as_bytes());
    let snapshot = screen.transcript_snapshot();
    let context = context();

    let first = search_terminal_transcript(&snapshot, &context, "ångström");
    let second = search_terminal_transcript(&snapshot, &context, "ÅNGSTRÖM");

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        first
            .iter()
            .map(|found| found.retained_line_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(first.iter().all(|found| {
        found.presentation.context.canonical_workspace_id.as_deref() == Some("workspace-1")
            && found
                .presentation
                .context
                .canonical_winds_session_id
                .as_deref()
                == Some("session-1")
            && found.presentation.context.pane_id == context.pane_id
    }));
}

#[test]
fn t093_retention_eviction_is_visible_without_rewriting_source_truth() {
    let mut screen =
        WorkbenchScreen::with_test_limits(PaneSize::new(80, 24), 2, 64).unwrap();
    screen.process_observed_bytes(b"first-line\nVERIFIED retained\nSECRET=fixture-token\n");
    let snapshot = screen.transcript_snapshot();

    assert!(snapshot.truncated);
    assert!(snapshot.evicted_lines > 0 || snapshot.evicted_bytes > 0);

    let matches = search_terminal_transcript(&snapshot, &context(), "secret=fixture-token");
    assert_eq!(matches.len(), 1);
    assert!(matches[0].retention.truncated);
    assert!(matches[0].retention.evicted_lines > 0 || matches[0].retention.evicted_bytes > 0);
    assert_eq!(
        matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
    assert!(!matches[0].presentation.changes_canonical_authority());
}

#[test]
fn t093_invalid_utf8_search_preserves_raw_terminal_bytes() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes(b"prefix\xffSECRET\n");
    let snapshot = screen.transcript_snapshot();

    let matches = search_terminal_transcript(&snapshot, &context(), "secret");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].presentation.raw_content, b"prefix\xffSECRET\n");
    assert_eq!(
        matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
}

#[test]
fn t093_empty_or_whitespace_query_returns_no_matches() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes(b"ordinary output\n");
    let snapshot = screen.transcript_snapshot();

    assert!(search_terminal_transcript(&snapshot, &context(), "").is_empty());
    assert!(search_terminal_transcript(&snapshot, &context(), "   ").is_empty());
}

#[test]
fn t093_retention_view_reports_existing_t089_bounds_without_new_persistence() {
    let mut screen =
        WorkbenchScreen::with_test_limits(PaneSize::new(80, 24), 3, 32).unwrap();
    screen.process_observed_bytes(b"one\ntwo\nthree\nfour\n");
    let snapshot = screen.transcript_snapshot();
    let retention = TranscriptRetentionView::from_snapshot(&snapshot);

    assert_eq!(retention.retained_lines, snapshot.lines.len());
    assert_eq!(retention.retained_bytes, snapshot.retained_bytes);
    assert_eq!(retention.evicted_lines, snapshot.evicted_lines);
    assert_eq!(retention.evicted_bytes, snapshot.evicted_bytes);
    assert_eq!(retention.truncated, snapshot.truncated);
    assert!(retention.retained_lines <= 3);
    assert!(retention.retained_bytes <= 32);
}