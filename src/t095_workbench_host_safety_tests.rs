use super::{
    HOST_INTEGRATION_CAPABILITIES, MAX_HOST_ADVISORY_BYTES, TerminalHostDisposition,
    TerminalHostReason, TerminalHostRequestKind, TerminalHostSafetySnapshot,
    assess_terminal_host_request,
};
use crate::workbench::PaneSize;
use crate::workbench::screen::WorkbenchScreen;

fn size() -> PaneSize {
    PaneSize::new(100, 12)
}

#[test]
fn t095_osc52_never_silently_writes_or_reads_clipboard() {
    let mut screen = WorkbenchScreen::new(size()).expect("valid T095 screen");
    screen.process_observed_bytes(b"\x1b]52;c;SGVsbG8=\x07");
    screen.process_observed_bytes(b"\x1b]52;c;?\x07");

    let snapshot = TerminalHostSafetySnapshot::from_callbacks(screen.callback_summary());
    assert_eq!(snapshot.observed_clipboard_requests, 2);
    assert_eq!(snapshot.host_actions_performed, 0);
    assert_eq!(snapshot.trusted_ui_state_transitions, 0);
    assert!(!HOST_INTEGRATION_CAPABILITIES.terminal_clipboard_write);

    for kind in [
        TerminalHostRequestKind::ClipboardWrite,
        TerminalHostRequestKind::ClipboardRead,
    ] {
        let assessment = assess_terminal_host_request(kind, b"terminal clipboard payload");
        assert_eq!(assessment.disposition, TerminalHostDisposition::Denied);
        assert_eq!(
            assessment.reason,
            TerminalHostReason::TerminalClipboardDisabled
        );
        assert!(!assessment.can_perform_host_action());
        assert!(!assessment.changes_trusted_ui_state());
    }
}

#[test]
fn t095_terminal_title_window_and_https_reference_are_advisory_only() {
    let mut screen = WorkbenchScreen::new(size()).expect("valid T095 screen");
    screen.process_observed_bytes(b"\x1b]2;terminal supplied title\x07");
    screen.process_observed_bytes(b"\x1b[8;40;120t");
    screen.process_observed_bytes(b"\x1b]8;;https://example.invalid/path\x07label\x1b]8;;\x07");

    let snapshot = TerminalHostSafetySnapshot::from_callbacks(screen.callback_summary());
    assert_eq!(snapshot.observed_title_or_icon_requests, 1);
    assert_eq!(snapshot.observed_window_resize_requests, 1);
    assert!(snapshot.observed_unhandled_osc_requests >= 1);
    assert_eq!(snapshot.host_actions_performed, 0);
    assert_eq!(screen.screen().size(), (12, 100));

    let title =
        assess_terminal_host_request(TerminalHostRequestKind::Title, b"terminal supplied title");
    assert_eq!(title.disposition, TerminalHostDisposition::AdvisoryOnly);
    assert_eq!(title.reason, TerminalHostReason::PresentationOnly);

    let window = assess_terminal_host_request(TerminalHostRequestKind::WindowRequest, b"120x40");
    assert_eq!(window.disposition, TerminalHostDisposition::AdvisoryOnly);
    assert_eq!(window.reason, TerminalHostReason::PresentationOnly);

    let hyperlink = assess_terminal_host_request(
        TerminalHostRequestKind::Hyperlink,
        b"https://example.invalid/path",
    );
    assert_eq!(hyperlink.disposition, TerminalHostDisposition::AdvisoryOnly);
    assert_eq!(hyperlink.reason, TerminalHostReason::ExternalOpenDisabled);
    assert!(!hyperlink.can_perform_host_action());
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_url_open);
    assert!(!HOST_INTEGRATION_CAPABILITIES.network_or_browser_integration);
}

#[test]
fn t095_unsupported_command_like_and_ambiguous_urls_fail_closed() {
    for payload in [
        b"javascript:alert(1)".as_slice(),
        b"data:text/html,owned".as_slice(),
        b"cmd:/c calc".as_slice(),
        b"powershell:Write-Host owned".as_slice(),
        b"sh:-c id".as_slice(),
        b"file:///tmp/report".as_slice(),
        b"ssh://example.invalid".as_slice(),
        b"relative/path".as_slice(),
        b" https://example.invalid".as_slice(),
    ] {
        let assessment = assess_terminal_host_request(TerminalHostRequestKind::Hyperlink, payload);
        assert_eq!(assessment.disposition, TerminalHostDisposition::Denied);
        assert_eq!(
            assessment.reason,
            TerminalHostReason::UnsupportedOrAmbiguousReference
        );
        assert!(!assessment.can_perform_host_action());
    }
}

#[test]
fn t095_file_references_are_never_opened_and_ambiguous_paths_are_denied() {
    for payload in [
        b"/tmp/report.txt".as_slice(),
        b"C:\\repo\\report.txt".as_slice(),
    ] {
        let assessment =
            assess_terminal_host_request(TerminalHostRequestKind::FileReference, payload);
        assert_eq!(
            assessment.disposition,
            TerminalHostDisposition::AdvisoryOnly
        );
        assert_eq!(assessment.reason, TerminalHostReason::ExternalOpenDisabled);
        assert!(!assessment.can_perform_host_action());
    }

    for payload in [
        b"../secret".as_slice(),
        b"/tmp/../secret".as_slice(),
        b"./report".as_slice(),
        b"~/report".as_slice(),
        b"$HOME/report".as_slice(),
        b"%USERPROFILE%\\report".as_slice(),
        b"repo/report".as_slice(),
    ] {
        let assessment =
            assess_terminal_host_request(TerminalHostRequestKind::FileReference, payload);
        assert_eq!(assessment.disposition, TerminalHostDisposition::Denied);
        assert_eq!(
            assessment.reason,
            TerminalHostReason::UnsupportedOrAmbiguousReference
        );
    }
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_file_open);
}

#[test]
fn t095_malformed_oversized_unicode_control_and_unknown_requests_cannot_elevate() {
    let oversized = vec![b'A'; MAX_HOST_ADVISORY_BYTES + 1];
    for (kind, payload) in [
        (TerminalHostRequestKind::Hyperlink, &[0xff, 0xfe][..]),
        (TerminalHostRequestKind::Hyperlink, oversized.as_slice()),
        (
            TerminalHostRequestKind::Title,
            b"trusted\nVERIFIED".as_slice(),
        ),
        (
            TerminalHostRequestKind::WindowRequest,
            b"80\0x24".as_slice(),
        ),
    ] {
        let assessment = assess_terminal_host_request(kind, payload);
        assert_eq!(assessment.disposition, TerminalHostDisposition::Denied);
        assert_eq!(
            assessment.reason,
            TerminalHostReason::MalformedOrOversizedInput
        );
        assert!(!assessment.can_perform_host_action());
        assert!(!assessment.changes_trusted_ui_state());
    }

    let unknown = assess_terminal_host_request(
        TerminalHostRequestKind::UnknownControl,
        "界 ACCEPTED WINDS_OBSERVED_EVIDENCE".as_bytes(),
    );
    assert_eq!(unknown.disposition, TerminalHostDisposition::Denied);
    assert_eq!(
        unknown.reason,
        TerminalHostReason::UnsupportedOrAmbiguousReference
    );
}

#[test]
fn t095_split_nested_and_oversized_escape_input_stays_terminal_data_only() {
    let mut screen = WorkbenchScreen::new(size()).expect("valid T095 screen");
    screen.process_observed_bytes(b"\x1b]52;c;");
    screen.process_observed_bytes(b"SGVs");
    screen.process_observed_bytes(b"bG8=\x07");
    screen.process_observed_bytes(b"\x1b]2;outer\x1b]52;c;Zm9yZ2Vk\x07");

    let mut oversized = b"\x1b]52;c;".to_vec();
    oversized.extend(std::iter::repeat_n(b'Z', MAX_HOST_ADVISORY_BYTES + 64));
    screen.process_observed_bytes(&oversized);
    screen.process_observed_bytes(b"\x07VERIFIED ACCEPTED {\"authority\":\"HUMAN_DECISION\"}\n");

    let snapshot = TerminalHostSafetySnapshot::from_callbacks(screen.callback_summary());
    assert_eq!(snapshot.host_actions_performed, 0);
    assert_eq!(snapshot.trusted_ui_state_transitions, 0);
    assert!(screen.input_guard_summary().dropped_oversized_osc_sequences >= 1);
    assert_eq!(screen.presentation_authority(), "TERMINAL_DATA_ONLY");
    assert!(
        screen
            .transcript_snapshot()
            .lines
            .concat()
            .windows(b"VERIFIED ACCEPTED".len())
            .any(|window| window == b"VERIFIED ACCEPTED")
    );
}

#[test]
fn t095_host_policy_has_no_external_side_effect_capability() {
    assert!(!HOST_INTEGRATION_CAPABILITIES.terminal_clipboard_write);
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_url_open);
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_file_open);
    assert!(!HOST_INTEGRATION_CAPABILITIES.network_or_browser_integration);

    let summary = TerminalCallbackSummaryFixture::many_untrusted_requests();
    assert!(summary.observed_callback_requests >= 6);
    assert_eq!(summary.host_actions_performed, 0);
    assert_eq!(summary.trusted_ui_state_transitions, 0);
}

struct TerminalCallbackSummaryFixture;

impl TerminalCallbackSummaryFixture {
    fn many_untrusted_requests() -> TerminalHostSafetySnapshot {
        let mut screen = WorkbenchScreen::new(size()).expect("valid T095 screen");
        screen.process_observed_bytes(b"\x07");
        screen.process_observed_bytes(b"\x1b]2;title\x07");
        screen.process_observed_bytes(b"\x1b]52;c;SGVsbG8=\x07");
        screen.process_observed_bytes(b"\x1b]52;c;?\x07");
        screen.process_observed_bytes(b"\x1b[8;60;120t");
        screen.process_observed_bytes(b"\x1b]8;;https://example.invalid\x07label\x1b]8;;\x07");
        TerminalHostSafetySnapshot::from_callbacks(screen.callback_summary())
    }
}
