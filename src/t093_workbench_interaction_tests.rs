use super::*;
use crate::workbench::screen::WorkbenchScreen;
use crate::workbench::{PaneSize, WorkbenchState};

fn context() -> InteractionContext {
    context_for("workspace-1", "session-1")
}

fn context_for(workspace_id: &str, session_id: &str) -> InteractionContext {
    let mut state = WorkbenchState::new();
    let pane_id = state.create_pane(
        "shell",
        Some(workspace_id.into()),
        Some(session_id.into()),
        PaneSize::new(80, 24),
    );
    InteractionContext::new(
        Some(pane_id),
        Some(workspace_id.into()),
        Some(session_id.into()),
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
        let presentation = InteractionPresentation::labelled(source, b"VERIFIED", context());
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
    screen.process_observed_bytes(b"VERIFIED\nACCEPTED\nWINDS_OBSERVED_EVIDENCE\nHUMAN_DECISION\n");

    let snapshot = screen.transcript_snapshot();
    let result = search_terminal_transcript(&snapshot, &context(), "verified");

    assert_eq!(result.matches.len(), 1);
    assert_eq!(
        result.matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
    assert_eq!(
        result.matches[0].presentation.boundary,
        InteractionBoundary::ContinuousTerminal
    );
    assert!(!result.matches[0].presentation.changes_canonical_authority());
    assert_eq!(result.matches[0].presentation.raw_content, b"VERIFIED\n");
    assert_eq!(result.scope, TranscriptSearchScope::RetainedWindowOnly);
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
    assert_eq!(first.matches.len(), 2);
    assert_eq!(
        first
            .matches
            .iter()
            .map(|found| found.retained_line_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(first.matches.iter().all(|found| {
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
    let mut screen = WorkbenchScreen::with_test_limits(PaneSize::new(80, 24), 2, 64).unwrap();
    screen.process_observed_bytes(b"first-line\nVERIFIED retained\nSECRET=fixture-token\n");
    let snapshot = screen.transcript_snapshot();

    assert!(snapshot.truncated);
    assert!(snapshot.evicted_lines > 0 || snapshot.evicted_bytes > 0);

    let result = search_terminal_transcript(&snapshot, &context(), "secret=fixture-token");
    assert_eq!(result.matches.len(), 1);
    assert!(result.retention.truncated);
    assert!(result.retention.evicted_lines > 0 || result.retention.evicted_bytes > 0);
    assert!(result.matches[0].retention.truncated);
    assert_eq!(
        result.matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
    assert!(!result.matches[0].presentation.changes_canonical_authority());
}

#[test]
fn t093_zero_match_search_still_exposes_retained_window_truncation() {
    let mut screen = WorkbenchScreen::with_test_limits(PaneSize::new(80, 24), 2, 32).unwrap();
    screen.process_observed_bytes(b"one\ntwo\nthree\nfour\n");
    let snapshot = screen.transcript_snapshot();

    let result = search_terminal_transcript(&snapshot, &context(), "definitely-absent");

    assert!(result.matches.is_empty());
    assert_eq!(result.scope, TranscriptSearchScope::RetainedWindowOnly);
    assert!(result.retention.truncated);
    assert!(result.retention.evicted_lines > 0 || result.retention.evicted_bytes > 0);
}

#[test]
fn t093_multi_session_search_preserves_each_context_without_recency_guessing() {
    let mut first_screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    first_screen.process_observed_bytes(b"deploy alpha\n");
    let first_snapshot = first_screen.transcript_snapshot();
    let first_context = context_for("workspace-a", "session-a");

    let mut second_screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    second_screen.process_observed_bytes(b"deploy beta\n");
    let second_snapshot = second_screen.transcript_snapshot();
    let second_context = context_for("workspace-b", "session-b");

    let views = [
        TerminalTranscriptView::new(&second_snapshot, &second_context),
        TerminalTranscriptView::new(&first_snapshot, &first_context),
    ];
    let result = search_terminal_transcripts(&views, "DEPLOY");

    assert_eq!(result.matches.len(), 2);
    assert_eq!(result.retention.transcript_count, 2);
    assert_eq!(
        result
            .matches
            .iter()
            .map(|found| found.transcript_index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        result.matches[0]
            .presentation
            .context
            .canonical_winds_session_id
            .as_deref(),
        Some("session-b")
    );
    assert_eq!(
        result.matches[1]
            .presentation
            .context
            .canonical_winds_session_id
            .as_deref(),
        Some("session-a")
    );
    assert!(
        result
            .matches
            .iter()
            .all(|found| found.presentation.source == InteractionSource::TerminalOutput)
    );
}

#[test]
fn t093_invalid_utf8_search_preserves_raw_terminal_bytes() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes(b"prefix\xffSECRET\n");
    let snapshot = screen.transcript_snapshot();

    let result = search_terminal_transcript(&snapshot, &context(), "secret");
    assert_eq!(result.matches.len(), 1);
    assert_eq!(
        result.matches[0].presentation.raw_content,
        b"prefix\xffSECRET\n"
    );
    assert_eq!(
        result.matches[0].presentation.source,
        InteractionSource::TerminalOutput
    );
}

#[test]
fn t093_empty_or_whitespace_query_returns_no_matches_but_keeps_scope_truth() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    screen.process_observed_bytes(b"ordinary output\n");
    let snapshot = screen.transcript_snapshot();

    for query in ["", "   "] {
        let result = search_terminal_transcript(&snapshot, &context(), query);
        assert!(result.matches.is_empty());
        assert_eq!(result.scope, TranscriptSearchScope::RetainedWindowOnly);
        assert_eq!(result.retention.transcript_count, 1);
    }
}

#[test]
fn t093_retention_view_reports_existing_t089_bounds_without_new_persistence() {
    let mut screen = WorkbenchScreen::with_test_limits(PaneSize::new(80, 24), 3, 32).unwrap();
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
