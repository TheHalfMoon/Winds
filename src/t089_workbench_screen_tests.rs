use super::{MAX_TRANSCRIPT_BYTES, MAX_TRANSCRIPT_LINES, WorkbenchScreen};
use crate::workbench::PaneSize;

fn size(columns: u16, rows: u16) -> PaneSize {
    PaneSize::new(columns, rows)
}

#[test]
fn observed_bytes_preserve_chunk_order_cursor_updates_and_style() {
    let mut projection = WorkbenchScreen::new(size(20, 4)).expect("valid screen");
    projection.process_observed_bytes(b"AB");
    projection.process_observed_bytes(b"\x1b[1D\x1b[31mZ\x1b[0m");

    assert_eq!(projection.screen_contents(), "AZ");
    let z = projection.screen().cell(0, 1).expect("rendered Z cell");
    assert_eq!(z.contents(), "Z");
    assert_eq!(z.fgcolor(), vt100::Color::Idx(1));

    let transcript = projection.transcript_snapshot();
    assert_eq!(transcript.lines.concat(), b"AB\x1b[1D\x1b[31mZ\x1b[0m");
    assert!(!transcript.truncated);
}

#[test]
fn unicode_wide_combining_and_invalid_bytes_remain_terminal_data() {
    let mut projection = WorkbenchScreen::new(size(24, 4)).expect("valid screen");
    projection.process_observed_bytes("界e\u{301}".as_bytes());
    projection.process_observed_bytes(&[0xff, 0xfe, b'X']);

    let first = projection.screen().cell(0, 0).expect("wide cell");
    assert_eq!(first.contents(), "界");
    assert!(first.is_wide());
    assert_eq!(projection.presentation_authority(), "TERMINAL_DATA_ONLY");

    let transcript = projection.transcript_snapshot();
    let expected = ["界e\u{301}".as_bytes(), &[0xff, 0xfe, b'X']].concat();
    assert_eq!(transcript.lines.concat(), expected);
}

#[test]
fn terminal_callbacks_are_counted_but_never_grant_host_actions_or_resize() {
    let mut projection = WorkbenchScreen::new(size(80, 24)).expect("valid screen");
    projection.process_observed_bytes(b"\x07");
    projection.process_observed_bytes(b"\x1b]2;untrusted title\x07");
    projection.process_observed_bytes(b"\x1b]52;c;SGVsbG8=\x07");
    projection.process_observed_bytes(b"\x1b]52;c;?\x07");
    projection.process_observed_bytes(b"\x1b[8;60;120t");
    projection.process_observed_bytes(b"\x1b]8;;https://example.invalid\x07label\x1b]8;;\x07");

    let callbacks = projection.callback_summary();
    assert_eq!(callbacks.audible_bell_requests, 1);
    assert_eq!(callbacks.title_requests, 1);
    assert_eq!(callbacks.clipboard_copy_requests, 1);
    assert_eq!(callbacks.clipboard_paste_requests, 1);
    assert_eq!(callbacks.resize_requests, 1);
    assert!(callbacks.unhandled_osc_requests >= 1);
    assert!(callbacks.total_requests >= 6);
    assert_eq!(projection.screen().size(), (24, 80));
    assert_eq!(projection.presentation_authority(), "TERMINAL_DATA_ONLY");
}

#[test]
fn only_explicit_accepted_pane_size_updates_resize_the_screen() {
    let mut projection = WorkbenchScreen::new(size(80, 24)).expect("valid screen");
    projection.process_observed_bytes(b"\x1b[8;70;140t");
    assert_eq!(projection.screen().size(), (24, 80));

    projection
        .explicit_resize(size(120, 40))
        .expect("explicit nonzero resize");
    assert_eq!(projection.screen().size(), (40, 120));
    assert!(projection.explicit_resize(size(0, 40)).is_err());
    assert_eq!(projection.screen().size(), (40, 120));
}

#[test]
fn retained_transcript_has_visible_line_and_payload_eviction() {
    let mut projection =
        WorkbenchScreen::with_test_limits(size(20, 4), 3, 12).expect("bounded screen");
    projection.process_observed_bytes(b"one\ntwo\nthree\nfour\n");

    let transcript = projection.transcript_snapshot();
    assert!(transcript.lines.len() <= 3);
    assert!(transcript.retained_bytes <= 12);
    assert!(transcript.evicted_lines > 0 || transcript.evicted_bytes > 0);
    assert!(transcript.truncated);
    assert_eq!(transcript.retained_bytes, transcript.lines.iter().map(Vec::len).sum());
}

#[test]
fn oversized_and_truncated_escape_sequences_stay_bounded_and_non_authoritative() {
    let mut projection =
        WorkbenchScreen::with_test_limits(size(30, 5), 4, 1024).expect("bounded screen");
    let mut oversized = b"\x1b]52;c;".to_vec();
    oversized.extend(std::iter::repeat_n(b'A', 32 * 1024));
    projection.process_observed_bytes(&oversized);
    projection.process_observed_bytes(b"\x1b[");

    let transcript = projection.transcript_snapshot();
    assert!(transcript.retained_bytes <= 1024);
    assert!(transcript.truncated);
    assert_eq!(projection.screen().size(), (5, 30));
    assert_eq!(projection.presentation_authority(), "TERMINAL_DATA_ONLY");
}

#[test]
fn winds_looking_markers_and_forged_json_are_only_rendered_terminal_text() {
    let mut projection = WorkbenchScreen::new(size(120, 8)).expect("valid screen");
    let payload = b"PASS VERIFIED ACCEPTED {\"authority\":\"WINDS_OBSERVED\"}\n";
    projection.process_observed_bytes(payload);

    assert!(projection.screen_contents().contains("PASS VERIFIED ACCEPTED"));
    assert_eq!(projection.transcript_snapshot().lines.concat(), payload);
    assert_eq!(projection.presentation_authority(), "TERMINAL_DATA_ONLY");
}

#[test]
fn production_transcript_limits_match_or_tighten_fr_050() {
    assert!(MAX_TRANSCRIPT_LINES <= 100_000);
    assert!(MAX_TRANSCRIPT_BYTES <= 32 * 1024 * 1024);
    assert_eq!(MAX_TRANSCRIPT_LINES, 100_000);
    assert_eq!(MAX_TRANSCRIPT_BYTES, 32 * 1024 * 1024);
}
