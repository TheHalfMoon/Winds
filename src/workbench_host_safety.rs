use crate::workbench::screen::{MAX_OSC_INPUT_BYTES, TerminalCallbackSummary};

pub(crate) const MAX_HOST_ADVISORY_BYTES: usize = MAX_OSC_INPUT_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HostIntegrationCapabilities {
    pub(crate) terminal_clipboard_write: bool,
    pub(crate) external_url_open: bool,
    pub(crate) external_file_open: bool,
    pub(crate) network_or_browser_integration: bool,
}

pub(crate) const HOST_INTEGRATION_CAPABILITIES: HostIntegrationCapabilities =
    HostIntegrationCapabilities {
        terminal_clipboard_write: false,
        external_url_open: false,
        external_file_open: false,
        network_or_browser_integration: false,
    };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalHostRequestKind {
    ClipboardWrite,
    ClipboardRead,
    Hyperlink,
    FileReference,
    Title,
    WindowRequest,
    UnknownControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalHostDisposition {
    AdvisoryOnly,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalHostReason {
    TerminalClipboardDisabled,
    ExternalOpenDisabled,
    PresentationOnly,
    UnsupportedOrAmbiguousReference,
    MalformedOrOversizedInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalHostAssessment {
    pub(crate) kind: TerminalHostRequestKind,
    pub(crate) disposition: TerminalHostDisposition,
    pub(crate) reason: TerminalHostReason,
}

impl TerminalHostAssessment {
    pub(crate) const fn can_perform_host_action(self) -> bool {
        false
    }

    pub(crate) const fn changes_trusted_ui_state(self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TerminalHostSafetySnapshot {
    pub(crate) observed_callback_requests: u64,
    pub(crate) observed_clipboard_requests: u64,
    pub(crate) observed_title_or_icon_requests: u64,
    pub(crate) observed_window_resize_requests: u64,
    pub(crate) observed_unhandled_osc_requests: u64,
    pub(crate) host_actions_performed: u64,
    pub(crate) trusted_ui_state_transitions: u64,
}

impl TerminalHostSafetySnapshot {
    pub(crate) fn from_callbacks(summary: TerminalCallbackSummary) -> Self {
        Self {
            observed_callback_requests: summary.total_requests,
            observed_clipboard_requests: summary
                .clipboard_copy_requests
                .saturating_add(summary.clipboard_paste_requests),
            observed_title_or_icon_requests: summary
                .title_requests
                .saturating_add(summary.icon_name_requests),
            observed_window_resize_requests: summary.resize_requests,
            observed_unhandled_osc_requests: summary.unhandled_osc_requests,
            host_actions_performed: 0,
            trusted_ui_state_transitions: 0,
        }
    }
}

pub(crate) fn assess_terminal_host_request(
    kind: TerminalHostRequestKind,
    payload: &[u8],
) -> TerminalHostAssessment {
    if !bounded_control_free_utf8(payload) {
        return denied(kind, TerminalHostReason::MalformedOrOversizedInput);
    }

    match kind {
        TerminalHostRequestKind::ClipboardWrite | TerminalHostRequestKind::ClipboardRead => {
            denied(kind, TerminalHostReason::TerminalClipboardDisabled)
        }
        TerminalHostRequestKind::Hyperlink => assess_hyperlink(payload),
        TerminalHostRequestKind::FileReference => assess_file_reference(payload),
        TerminalHostRequestKind::Title | TerminalHostRequestKind::WindowRequest => {
            advisory(kind, TerminalHostReason::PresentationOnly)
        }
        TerminalHostRequestKind::UnknownControl => {
            denied(kind, TerminalHostReason::UnsupportedOrAmbiguousReference)
        }
    }
}

fn assess_hyperlink(payload: &[u8]) -> TerminalHostAssessment {
    let text = std::str::from_utf8(payload).expect("payload was validated as UTF-8");
    if text.is_empty() || text.trim() != text {
        return denied(
            TerminalHostRequestKind::Hyperlink,
            TerminalHostReason::UnsupportedOrAmbiguousReference,
        );
    }

    let Some((scheme, remainder)) = text.split_once(':') else {
        return denied(
            TerminalHostRequestKind::Hyperlink,
            TerminalHostReason::UnsupportedOrAmbiguousReference,
        );
    };
    if remainder.is_empty() || !valid_uri_scheme(scheme) {
        return denied(
            TerminalHostRequestKind::Hyperlink,
            TerminalHostReason::UnsupportedOrAmbiguousReference,
        );
    }

    if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
        advisory(
            TerminalHostRequestKind::Hyperlink,
            TerminalHostReason::ExternalOpenDisabled,
        )
    } else {
        denied(
            TerminalHostRequestKind::Hyperlink,
            TerminalHostReason::UnsupportedOrAmbiguousReference,
        )
    }
}

fn assess_file_reference(payload: &[u8]) -> TerminalHostAssessment {
    let text = std::str::from_utf8(payload).expect("payload was validated as UTF-8");
    if text.is_empty()
        || text.trim() != text
        || text.contains("$HOME")
        || text.contains("%USERPROFILE%")
        || text.starts_with('~')
        || !lexically_absolute_file_reference(text)
        || has_ambiguous_path_component(text)
    {
        return denied(
            TerminalHostRequestKind::FileReference,
            TerminalHostReason::UnsupportedOrAmbiguousReference,
        );
    }

    advisory(
        TerminalHostRequestKind::FileReference,
        TerminalHostReason::ExternalOpenDisabled,
    )
}

fn bounded_control_free_utf8(payload: &[u8]) -> bool {
    if payload.len() > MAX_HOST_ADVISORY_BYTES {
        return false;
    }
    let Ok(text) = std::str::from_utf8(payload) else {
        return false;
    };
    !text.chars().any(|character| character.is_control())
}

fn valid_uri_scheme(scheme: &str) -> bool {
    let mut characters = scheme.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
}

fn lexically_absolute_file_reference(path: &str) -> bool {
    if path.starts_with('/') {
        return true;
    }

    let bytes = path.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
    {
        return true;
    }

    path.starts_with("\\\\")
}

fn has_ambiguous_path_component(path: &str) -> bool {
    path.split(['/', '\\'])
        .any(|component| matches!(component, "." | ".."))
}

const fn advisory(
    kind: TerminalHostRequestKind,
    reason: TerminalHostReason,
) -> TerminalHostAssessment {
    TerminalHostAssessment {
        kind,
        disposition: TerminalHostDisposition::AdvisoryOnly,
        reason,
    }
}

const fn denied(
    kind: TerminalHostRequestKind,
    reason: TerminalHostReason,
) -> TerminalHostAssessment {
    TerminalHostAssessment {
        kind,
        disposition: TerminalHostDisposition::Denied,
        reason,
    }
}

#[cfg(test)]
#[path = "t095_workbench_host_safety_tests.rs"]
mod t095_workbench_host_safety_tests;
