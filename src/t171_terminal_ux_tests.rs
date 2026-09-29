use super::*;
use std::sync::atomic::AtomicU64;
static NEXT_T171_CASE: AtomicU64 = AtomicU64::new(0);

fn lines(texts: &[&str]) -> Vec<Vec<u8>> {
    texts.iter().map(|text| text.as_bytes().to_vec()).collect()
}

fn point(row: usize, column: usize) -> GridPoint {
    GridPoint { row, column }
}

#[test]
fn t171_scrollback_is_bounded_and_truncation_is_explicit() {
    let mut scrollback = BoundedScrollback::new();
    assert!(scrollback.is_empty());
    assert!(!scrollback.truncated());

    for row in 0..MAX_SCROLLBACK_LINES {
        scrollback.push(vec![b'a' + u8::try_from(row % 26).unwrap_or(b'a')]);
    }
    assert_eq!(scrollback.len(), MAX_SCROLLBACK_LINES);
    assert!(
        !scrollback.truncated(),
        "reaching the bound is not yet truncation"
    );
    assert_eq!(scrollback.dropped_rows(), 0);

    scrollback.push(b"z".to_vec());
    assert_eq!(
        scrollback.len(),
        MAX_SCROLLBACK_LINES,
        "the bound is never exceeded"
    );
    assert!(
        scrollback.truncated(),
        "truncation is observable, never silent"
    );
    assert_eq!(scrollback.dropped_rows(), 1);
    assert_eq!(
        scrollback.rows().front().map(Vec::len),
        Some(1),
        "the oldest row is the one dropped"
    );

    for _ in 0..9 {
        scrollback.push(b"y".to_vec());
    }
    assert_eq!(scrollback.len(), MAX_SCROLLBACK_LINES);
    assert_eq!(scrollback.dropped_rows(), 10);
}

#[test]
fn t171_selection_is_exact_deterministic_and_bounded() {
    let content = lines(&["alpha beta gamma", "second row", "third row"]);
    let selection = TerminalSelection::new(point(0, 0), point(0, 4));

    // Document order does not depend on drag direction.
    assert_eq!(selection.ordered(), (point(0, 0), point(0, 4)));
    let reversed = TerminalSelection::new(point(0, 4), point(0, 0));
    assert_eq!(reversed.ordered(), selection.ordered());
    assert_eq!(
        reversed.copy_text(&content).unwrap(),
        selection.copy_text(&content).unwrap()
    );
    assert_eq!(selection.copy_text(&content).unwrap(), "alpha");

    // A multi-row selection is exact and newline separated.
    let multi = TerminalSelection::new(point(0, 6), point(1, 5));
    assert_eq!(multi.copy_text(&content).unwrap(), "beta gamma\nsecond");

    // Word selection aligns both ends, so a drag between two words captures
    // exactly the words it spans and never the gap outside them.
    let word = TerminalSelection::new_word_anchored(point(0, 0), point(0, 8), &content).unwrap();
    assert_eq!(word.anchor(), point(0, 0));
    assert_eq!(word.head(), point(0, 9));
    assert_eq!(word.copy_text(&content).unwrap(), "alpha beta");
    let single_word =
        TerminalSelection::new_word_anchored(point(0, 0), point(0, 0), &content).unwrap();
    assert_eq!(single_word.copy_text(&content).unwrap(), "alpha");
    let word_end =
        TerminalSelection::new_word_anchored(point(0, 0), point(0, 4), &content).unwrap();
    assert_eq!(word_end.copy_text(&content).unwrap(), "alpha");
    // A drag that ends on a separator keeps the anchored word and extends to it,
    // so the selection never silently drops the anchor.
    let separator =
        TerminalSelection::new_word_anchored(point(0, 0), point(0, 5), &content).unwrap();
    assert_eq!(separator.copy_text(&content).unwrap(), "alpha ");
    // A drag that both ends on the same separator is exactly that separator.
    let separator_only =
        TerminalSelection::new_word_anchored(point(0, 5), point(0, 5), &content).unwrap();
    assert_eq!(separator_only.copy_text(&content).unwrap(), " ");

    // A selection that genuinely covers more cells than the budget fails closed
    // instead of allocating the whole range.
    let rows = MAX_SELECTION_CELLS / 8 + 8;
    let big: Vec<Vec<u8>> = (0..rows).map(|_| vec![b'x'; 8]).collect();
    let wide = TerminalSelection::new(point(0, 0), point(rows - 1, 7));
    assert!(matches!(wide.cells(&big), Err(SelectionError::TooLarge)));
    // A selection over rows that hold no content never invents padding.
    let absent = TerminalSelection::new(point(0, 0), point(MAX_SCROLLBACK_LINES * 4, 0));
    assert_eq!(
        absent.copy_text(&content).unwrap(),
        "alpha beta gamma\nsecond row\nthird row"
    );
}

#[test]
fn t171_search_is_literal_deterministic_and_bounded() {
    let content = lines(&[
        "Alpha alpha ALPHA",
        "no match here",
        "alpha_beta",
        "alpha beta",
    ]);

    let insensitive = TerminalSearchQuery::new(b"alpha", false, false).unwrap();
    let result = search_lines(&content, &insensitive);
    assert_eq!(
        result.matches,
        vec![
            SearchMatch { row: 0, column: 0 },
            SearchMatch { row: 0, column: 6 },
            SearchMatch { row: 0, column: 12 },
            SearchMatch { row: 2, column: 0 },
            SearchMatch { row: 3, column: 0 },
        ]
    );
    assert!(!result.truncated);

    let sensitive = TerminalSearchQuery::new(b"alpha", true, false).unwrap();
    assert_eq!(search_lines(&content, &sensitive).matches.len(), 3);

    let whole_word = TerminalSearchQuery::new(b"alpha", true, true).unwrap();
    assert_eq!(
        search_lines(&content, &whole_word).matches,
        vec![
            SearchMatch { row: 0, column: 6 },
            SearchMatch { row: 3, column: 0 },
        ],
        "whole-word search never matches inside a longer identifier"
    );

    // A query is never interpreted as a pattern.
    assert!(TerminalSearchQuery::new(b"", false, false).is_err());
    assert!(TerminalSearchQuery::new(b"a*", false, false).is_ok());
    assert!(
        search_lines(
            &content,
            &TerminalSearchQuery::new(b"a*", false, false).unwrap()
        )
        .matches
        .is_empty(),
        "a literal needle never behaves like a wildcard"
    );

    // The match budget is explicit.
    let many = lines(&[&"a".repeat(MAX_SEARCH_MATCHES + 50)]);
    let bounded = search_lines(&many, &TerminalSearchQuery::new(b"a", true, false).unwrap());
    assert_eq!(bounded.matches.len(), MAX_SEARCH_MATCHES);
    assert!(bounded.truncated);
}

#[test]
fn t171_link_activation_validates_scheme_and_requires_accepted_action() {
    assert_eq!(
        parse_link_target(b"https://example.test/a"),
        Ok(LinkTarget {
            scheme: "https",
            target: "//example.test/a".to_owned(),
        })
    );
    assert_eq!(
        parse_link_target(b"HTTPS://example.test").unwrap().scheme,
        "https"
    );

    // A local file target is never offerable, because activating it would be a
    // host filesystem action taken from bytes a child printed.
    assert_eq!(
        parse_link_target(b"file:///etc/passwd"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(b"javascript:alert(1)"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(b"data:text/html,<script>"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    // The canonical host-safety model refuses a scheme-less target before the
    // offer layer ever considers it.
    assert_eq!(
        parse_link_target(b"example.test"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(b"://x"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    // Embedded control bytes are refused by the canonical host-safety model
    // before the offer layer runs, so a forged split target never becomes an offer.
    assert_eq!(
        parse_link_target(b"https://exa\nmple.test"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    let oversized = vec![b'a'; MAX_LINK_TARGET_BYTES + 1];
    assert_eq!(parse_link_target(&oversized), Err(LinkRefusal::TooLong));

    // Validation alone never activates anything.
    let offered = LinkOffer::offer(b"https://example.test/ok");
    assert_eq!(
        offered,
        LinkOffer::Pending(LinkTarget {
            scheme: "https",
            target: "//example.test/ok".to_owned(),
        })
    );
    assert_eq!(LinkOffer::offer(b"file:///etc/passwd"), LinkOffer::Refused);
    assert_eq!(
        LinkOffer::offer(b"file:///etc/passwd").accept(),
        LinkOffer::Refused,
        "a refused target can never be granted"
    );

    // Only an explicit accepted action produces the grant.
    assert!(matches!(
        offered.accept(),
        LinkOffer::Granted(LinkActivationGrant { .. })
    ));
}

#[test]
fn t171_clipboard_is_local_only_and_bounded() {
    assert_eq!(offer_clipboard(b""), None);
    assert_eq!(
        offer_clipboard(&vec![b'a'; MAX_CLIPBOARD_OFFER_BYTES + 1]),
        None
    );
    let offer = offer_clipboard(b"local bytes").expect("bounded local offer");
    assert_eq!(offer.payload, b"local bytes".to_vec());
}

#[test]
fn t171_graphics_are_bounded_local_and_never_fetch_remote() {
    assert_eq!(
        accept_graphics_frame(&vec![b'a'; MAX_GRAPHICS_FRAME_BYTES + 1]),
        Err(GraphicsRefusal::TooLarge)
    );
    assert_eq!(
        accept_graphics_frame(b"\x1b_Gf=100;https://example.test/i.png\x1b\\"),
        Err(GraphicsRefusal::RemoteReference)
    );
    assert_eq!(
        accept_graphics_frame(b"\x1b_Gf=100;HTTPS://example.test/i.png\x1b\\"),
        Err(GraphicsRefusal::RemoteReference)
    );
    assert_eq!(
        accept_graphics_frame(b"\x1b_Gf=100;http://example.test/i.png\x1b\\"),
        Err(GraphicsRefusal::RemoteReference)
    );
    let frame = accept_graphics_frame(b"\x1b_Gf=100;aGVsbG8=\x1b\\").unwrap();
    assert_eq!(frame.bytes, b"\x1b_Gf=100;aGVsbG8=\x1b\\".to_vec());
}

#[test]
fn t171_notifications_are_bounded_and_suppressible() {
    let mut budget = NotificationBudget::new();
    assert!(budget.record(1_000).is_some());
    assert_eq!(budget.retained(), 1);

    for index in 1..MAX_NOTIFICATIONS {
        assert!(
            budget
                .record(1_000 + i64::try_from(index).unwrap_or(0))
                .is_some()
        );
    }
    assert_eq!(budget.retained(), MAX_NOTIFICATIONS);
    assert!(
        budget.record(1_001).is_none(),
        "the notification budget is enforced inside the window"
    );
    assert_eq!(
        budget.total(),
        u64::try_from(MAX_NOTIFICATIONS).unwrap_or(u64::MAX) + 1
    );

    // A new window resets the budget.
    assert!(budget.record(1_000 + NOTIFICATION_WINDOW_MS).is_some());
    assert_eq!(budget.retained(), 1);

    // Suppression stops all recording.
    budget.suppress();
    assert!(budget.is_suppressed());
    assert!(budget.record(9_999).is_none());
    assert_eq!(budget.retained(), 1);
}

#[test]
fn t171_input_method_claim_is_gated_by_native_evidence() {
    assert_eq!(
        InputMethodClaim::claim("", "evidence"),
        InputMethodClaim::Unclaimed
    );
    assert_eq!(
        InputMethodClaim::claim("windows", ""),
        InputMethodClaim::Unclaimed
    );
    let unclaimed = InputMethodClaim::claim("", "");
    assert!(!unclaimed.is_claimed());

    let claimed = InputMethodClaim::claim("windows-11", "conpty-ime-fixture");
    assert!(claimed.is_claimed());
    assert_eq!(
        claimed,
        InputMethodClaim::Claimed {
            evidence_platform: "windows-11".to_owned(),
            evidence: "conpty-ime-fixture".to_owned(),
        }
    );
}

#[test]
fn t171_forged_terminal_text_never_changes_trusted_state() {
    // A child printing trusted-looking text stays presentation data. Nothing here
    // can turn terminal prose into an offer, a grant, a claim, or an authority.
    let forged = b"VERIFIED ACCEPTED Needs You provider=anthropic model=claude-opus";
    let offer = LinkOffer::offer(&forged[..8]);
    assert_eq!(offer, LinkOffer::Refused);
    assert_ne!(
        offer,
        LinkOffer::Pending(LinkTarget {
            scheme: "https",
            target: String::new(),
        })
    );
    assert!(offer_clipboard(forged).is_some());
    let frame = accept_graphics_frame(forged).expect("bounded local text stays a frame");
    assert!(frame.bytes.starts_with(b"VERIFIED"));
    assert_eq!(InputMethodClaim::claim("", ""), InputMethodClaim::Unclaimed);

    let mut scrollback = BoundedScrollback::new();
    scrollback.push(forged.to_vec());
    assert!(!scrollback.truncated());
    assert_eq!(scrollback.rows().front().map(Vec::len), Some(forged.len()));
}

#[test]
fn t171_high_output_adversarial_input_stays_bounded() {
    let mut scrollback = BoundedScrollback::new();
    for index in 0..(MAX_SCROLLBACK_LINES * 2) {
        scrollback.push(vec![b'x'; 256]);
        assert!(scrollback.len() <= MAX_SCROLLBACK_LINES);
        let _ = index;
    }
    assert_eq!(scrollback.len(), MAX_SCROLLBACK_LINES);
    assert_eq!(
        scrollback.dropped_rows(),
        u64::try_from(MAX_SCROLLBACK_LINES).unwrap_or(0)
    );

    let content = lines(&[&"y".repeat(4_096)]);
    let whole = TerminalSelection::new(point(0, 0), point(0, 4_095));
    assert_eq!(whole.copy_text(&content).unwrap().len(), 4_096);
    // A selection over content that does not exist stays inside the budget,
    // because bounded content can never inflate the cell count.
    let beyond_content = TerminalSelection::new(point(0, 0), point(0, MAX_SELECTION_CELLS + 10));
    assert_eq!(beyond_content.cell_count(&content), 4_096);
    // A selection that really covers more cells than the budget fails closed.
    let rows = MAX_SELECTION_CELLS / 8 + 8;
    let dense: Vec<Vec<u8>> = (0..rows).map(|_| vec![b'z'; 8]).collect();
    assert!(matches!(
        TerminalSelection::new(point(0, 0), point(rows - 1, 7)).cells(&dense),
        Err(SelectionError::TooLarge)
    ));
    // Notification spam stays bounded and suppressible under the Plan ceiling.
    let mut budget = NotificationBudget::new();
    for index in 0..(MAX_NOTIFICATIONS * 10) {
        budget.record(i64::try_from(index).unwrap_or(i64::MAX));
    }
    assert!(budget.retained() <= MAX_NOTIFICATIONS);
    assert!(!budget.is_suppressed());
    // Graphics never grow past the Plan ceiling and never fetch a remote payload.
    assert_eq!(
        accept_graphics_frame(&vec![b'g'; MAX_GRAPHICS_FRAME_BYTES + 1]),
        Err(GraphicsRefusal::TooLarge)
    );
    let mut working = PaneWorkingBuffer::new();
    assert!(
        working
            .retain(&vec![b'w'; MAX_PANE_WORKING_BUFFER_BYTES])
            .is_ok()
    );
    assert_eq!(working.retained_bytes(), MAX_PANE_WORKING_BUFFER_BYTES);
    assert!(
        working.retain(b"overflow").is_err(),
        "an over-budget chunk is refused whole rather than partially retained"
    );
    assert_eq!(working.retained_bytes(), MAX_PANE_WORKING_BUFFER_BYTES);
}
