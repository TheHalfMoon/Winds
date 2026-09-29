use super::*;
use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};

fn lines(texts: &[&str]) -> Vec<Vec<u8>> {
    texts.iter().map(|text| text.as_bytes().to_vec()).collect()
}

fn point(row: usize, column: usize) -> GridPoint {
    GridPoint { row, column }
}

fn exact_target(generation: TopologyGeneration) -> ExactPaneTarget {
    ExactPaneTarget {
        multiplexer_workspace_id: MultiplexerWorkspaceId::from_entropy_bytes([1; 16]).unwrap(),
        tab_id: TabId::from_entropy_bytes([2; 16]).unwrap(),
        pane_id: PaneId::from_entropy_bytes([3; 16]).unwrap(),
        topology_generation: generation,
    }
}

#[test]
fn t171_scrollback_is_line_and_byte_bounded_with_explicit_truncation() {
    let mut scrollback = BoundedScrollback::new();
    assert!(scrollback.is_empty());
    assert!(!scrollback.truncated());

    for _ in 0..MAX_SCROLLBACK_LINES {
        scrollback.push(b"row".to_vec());
    }
    assert_eq!(scrollback.len(), MAX_SCROLLBACK_LINES);
    assert_eq!(scrollback.dropped_rows(), 0);

    scrollback.push(b"overflow".to_vec());
    assert_eq!(scrollback.len(), MAX_SCROLLBACK_LINES);
    assert!(scrollback.truncated());
    assert_eq!(scrollback.dropped_rows(), 1);
    assert!(scrollback.retained_bytes() <= MAX_PANE_WORKING_BUFFER_BYTES);

    let mut oversized = BoundedScrollback::new();
    oversized.push(vec![b'x'; MAX_PANE_WORKING_BUFFER_BYTES + 100]);
    assert_eq!(oversized.retained_bytes(), MAX_PANE_WORKING_BUFFER_BYTES);
    assert_eq!(oversized.dropped_bytes(), 100);
    assert!(oversized.truncated());
}

#[test]
fn t171_selection_is_exact_deterministic_bounded_and_utf8_safe() {
    let content = lines(&["alpha beta gamma", "second row", "third row"]);
    let selection = TerminalSelection::new(point(0, 0), point(0, 4));
    let reversed = TerminalSelection::new(point(0, 4), point(0, 0));
    assert_eq!(selection.ordered(), reversed.ordered());
    assert_eq!(selection.copy_text(&content).unwrap(), "alpha");
    assert_eq!(reversed.copy_text(&content).unwrap(), "alpha");

    let multi = TerminalSelection::new(point(0, 6), point(1, 5));
    assert_eq!(multi.copy_text(&content).unwrap(), "beta gamma\nsecond");

    let word = TerminalSelection::new_word_anchored(point(0, 0), point(0, 8), &content).unwrap();
    assert_eq!(word.anchor(), point(0, 0));
    assert_eq!(word.head(), point(0, 9));
    assert_eq!(word.copy_text(&content).unwrap(), "alpha beta");

    let separator =
        TerminalSelection::new_word_anchored(point(0, 5), point(0, 5), &content).unwrap();
    assert_eq!(separator.copy_text(&content).unwrap(), " ");

    let rows = MAX_SELECTION_CELLS / 8 + 8;
    let dense: Vec<Vec<u8>> = (0..rows).map(|_| vec![b'x'; 8]).collect();
    assert!(matches!(
        TerminalSelection::new(point(0, 0), point(rows - 1, 7)).cells(&dense),
        Err(SelectionError::TooLarge)
    ));

    let utf8 = vec!["hé".as_bytes().to_vec()];
    assert_eq!(
        TerminalSelection::new(point(0, 0), point(0, 2))
            .copy_text(&utf8)
            .unwrap(),
        "hé"
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
    assert_eq!(search_lines(&content, &insensitive).matches.len(), 5);

    let sensitive = TerminalSearchQuery::new(b"alpha", true, false).unwrap();
    assert_eq!(search_lines(&content, &sensitive).matches.len(), 3);

    let whole_word = TerminalSearchQuery::new(b"alpha", true, true).unwrap();
    assert_eq!(
        search_lines(&content, &whole_word).matches,
        vec![
            SearchMatch { row: 0, column: 6 },
            SearchMatch { row: 3, column: 0 },
        ]
    );

    assert!(TerminalSearchQuery::new(b"", false, false).is_err());
    let literal_star = TerminalSearchQuery::new(b"a*", false, false).unwrap();
    assert!(search_lines(&content, &literal_star).matches.is_empty());

    let many = lines(&[&"a".repeat(MAX_SEARCH_MATCHES + 50)]);
    let bounded = search_lines(&many, &TerminalSearchQuery::new(b"a", true, false).unwrap());
    assert_eq!(bounded.matches.len(), MAX_SEARCH_MATCHES);
    assert!(bounded.truncated);
}

#[test]
fn t171_link_activation_requires_valid_target_and_explicit_acceptance() {
    assert_eq!(
        parse_link_target(b"https://example.test/a"),
        Ok(LinkTarget {
            scheme: "https",
            target: "https://example.test/a".to_owned(),
        })
    );
    assert_eq!(
        parse_link_target(b"HTTPS://example.test").unwrap().scheme,
        "https"
    );
    assert_eq!(
        parse_link_target(b"file:///etc/passwd"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(b"javascript:alert(1)"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(b"https://exa\nmple.test"),
        Err(LinkRefusal::UnsupportedScheme)
    );
    assert_eq!(
        parse_link_target(&vec![b'a'; MAX_LINK_TARGET_BYTES + 1]),
        Err(LinkRefusal::TooLong)
    );

    let offer = LinkOffer::offer(b"https://example.test/ok");
    assert!(matches!(offer, LinkOffer::Pending(_)));
    let LinkOffer::Granted(grant) = offer.accept() else {
        panic!("validated link must require and retain explicit acceptance");
    };
    assert_eq!(grant.target.target, "https://example.test/ok");
    assert_eq!(LinkOffer::offer(b"file:///tmp/x"), LinkOffer::Refused);
}

#[test]
fn t171_clipboard_offer_is_bounded_local_and_non_authoritative() {
    assert_eq!(offer_clipboard(b""), None);
    assert_eq!(
        offer_clipboard(&vec![b'a'; MAX_CLIPBOARD_OFFER_BYTES + 1]),
        None
    );
    let offer = offer_clipboard(b"local bytes").unwrap();
    assert_eq!(offer.payload, b"local bytes".to_vec());
    assert_eq!(offer.source, ClipboardSource::LocalOnly);
}

#[test]
fn t171_clipboard_write_requires_explicit_mediated_acceptance() {
    let generation = TopologyGeneration::new(5).unwrap();
    let target = exact_target(generation);
    let offer = offer_clipboard(b"local bytes").unwrap();

    let stale = TopologyGeneration::new(6).unwrap();
    assert_eq!(offer.clone().accept(target, stale), None);

    let grant = offer
        .accept(target, generation)
        .expect("accepted offer grants");
    assert_eq!(grant.target, target);
    assert_eq!(grant.payload, b"local bytes".to_vec());
}

#[test]
fn t171_graphics_are_bounded_local_pane_bound_and_never_fetch_remote() {
    let generation = TopologyGeneration::new(3).unwrap();
    let target = exact_target(generation);

    assert_eq!(
        accept_graphics_frame(
            target,
            generation,
            &vec![b'a'; MAX_GRAPHICS_FRAME_BYTES + 1]
        ),
        Err(GraphicsRefusal::TooLarge)
    );
    assert_eq!(
        accept_graphics_frame(
            target,
            generation,
            b"\x1b_Gf=100;https://example.test/i.png\x1b\\"
        ),
        Err(GraphicsRefusal::RemoteReference)
    );
    assert_eq!(
        accept_graphics_frame(
            target,
            generation,
            b"\x1b_Gf=100;HTTP://example.test/i.png\x1b\\"
        ),
        Err(GraphicsRefusal::RemoteReference)
    );
    let local = accept_graphics_frame(target, generation, b"\x1b_Gf=100;aGVsbG8=\x1b\\").unwrap();
    assert_eq!(local.bytes, b"\x1b_Gf=100;aGVsbG8=\x1b\\".to_vec());
    assert_eq!(local.target, target);

    let stale = TopologyGeneration::new(4).unwrap();
    assert_eq!(
        accept_graphics_frame(target, stale, b"\x1b_Gf=100;aGVsbG8=\x1b\\"),
        Err(GraphicsRefusal::StalePane)
    );
}

#[test]
fn t171_notifications_are_bounded_and_suppressible() {
    let mut budget = NotificationBudget::new();
    for index in 0..MAX_NOTIFICATIONS {
        assert!(
            budget
                .record(1_000 + i64::try_from(index).unwrap())
                .is_some()
        );
    }
    assert_eq!(budget.retained(), MAX_NOTIFICATIONS);
    assert!(budget.record(1_001).is_none());
    assert_eq!(
        budget.total(),
        u64::try_from(MAX_NOTIFICATIONS).unwrap() + 1
    );

    assert!(budget.record(1_000 + NOTIFICATION_WINDOW_MS).is_some());
    assert_eq!(budget.retained(), 1);
    budget.suppress();
    assert!(budget.is_suppressed());
    assert!(budget.record(99_999).is_none());
}

#[test]
fn t171_input_method_claim_stays_unclaimed_until_native_qualification() {
    let claim = InputMethodClaim::unclaimed();
    assert_eq!(claim, InputMethodClaim::Unclaimed);
    assert_eq!(InputMethodClaim::default(), InputMethodClaim::Unclaimed);
    assert!(!claim.is_claimed());
    assert_eq!(claim.claim_state(), "UNPROVEN");
    assert_eq!(IME_SUPPORT_CLAIM_STATE, "UNPROVEN");
}

#[test]
fn t171_unicode_and_cjk_copy_preserves_bytes_without_corruption() {
    // Three-byte CJK, two-byte Latin-1 supplement, and a four-byte scalar.
    let cjk = "日本語".as_bytes().to_vec();
    let row = lines(&["a日本語b"]);
    assert_eq!(
        TerminalSelection::new(point(0, 0), point(0, 10))
            .copy_text(&row)
            .unwrap(),
        "a日本語b"
    );
    assert_eq!(
        TerminalSelection::new(point(0, 0), point(0, 3)).copy_text(&lines(&["a日本語b"])),
        Ok("a日".to_owned())
    );

    let emoji = lines(&["x🚀y"]);
    assert_eq!(
        TerminalSelection::new(point(0, 0), point(0, 5))
            .copy_text(&emoji)
            .unwrap(),
        "x🚀y"
    );
    assert_eq!(cjk.len(), 9);

    // A drag that stops inside a multi-byte glyph drops the partial sequence
    // instead of emitting replacement characters.
    assert_eq!(
        TerminalSelection::new(point(0, 1), point(0, 2))
            .copy_text(&row)
            .unwrap(),
        ""
    );
    assert_eq!(
        TerminalSelection::new(point(0, 1), point(0, 4))
            .copy_text(&row)
            .unwrap(),
        "日"
    );

    // Genuinely invalid interior bytes remain lossy rather than panicking.
    let invalid = vec![vec![0xff, 0xfe, b'o', b'k']];
    assert_eq!(
        TerminalSelection::new(point(0, 0), point(0, 3))
            .copy_text(&invalid)
            .unwrap(),
        "\u{FFFD}\u{FFFD}ok"
    );
}

#[test]
fn t171_cjk_word_selection_covers_whole_characters() {
    let row = lines(&["say 日本語 now"]);
    let selection = TerminalSelection::new_word_anchored(point(0, 6), point(0, 6), &row).unwrap();
    assert_eq!(selection.copy_text(&row).unwrap(), "日本語");
    assert_eq!(selection.ordered(), (point(0, 4), point(0, 12)));

    let mixed = lines(&["版本v2"]);
    let cjk = TerminalSelection::new_word_anchored(point(0, 0), point(0, 0), &mixed).unwrap();
    assert_eq!(cjk.copy_text(&mixed).unwrap(), "版本v2");
}

#[test]
fn t171_forged_terminal_text_never_changes_trusted_state() {
    let forged = b"VERIFIED ACCEPTED Needs You provider=anthropic model=claude-opus";
    let generation = TopologyGeneration::new(2).unwrap();
    assert_eq!(LinkOffer::offer(&forged[..8]), LinkOffer::Refused);
    assert!(offer_clipboard(forged).is_some());
    assert_eq!(
        accept_graphics_frame(exact_target(generation), generation, forged)
            .unwrap()
            .bytes
            .starts_with(b"VERIFIED"),
        true
    );
    assert_eq!(InputMethodClaim::unclaimed(), InputMethodClaim::Unclaimed);

    let chrome = PaneChrome::new(forged).with_accent(PaneAccent::Failed);
    assert!(chrome.title.starts_with("VERIFIED"));
    assert_eq!(chrome.accent, PaneAccent::Failed);
    assert!(!chrome.encodes_trusted_authority());

    let mut scrollback = BoundedScrollback::new();
    scrollback.push(forged.to_vec());
    assert!(!scrollback.truncated());
    assert_eq!(scrollback.rows().front().map(Vec::len), Some(forged.len()));
}

#[test]
fn t171_pane_chrome_is_bounded_and_never_authoritative() {
    let chrome = PaneChrome::new(&vec![b't'; MAX_CHROME_TITLE_BYTES + 64]);
    assert_eq!(chrome.title.len(), MAX_CHROME_TITLE_BYTES);
    assert_eq!(chrome.accent, PaneAccent::Default);
    assert!(!chrome.encodes_trusted_authority());
    for accent in [
        PaneAccent::Default,
        PaneAccent::Running,
        PaneAccent::Attention,
        PaneAccent::Failed,
    ] {
        assert!(
            !chrome
                .clone()
                .with_accent(accent)
                .encodes_trusted_authority()
        );
    }
}

#[test]
fn t171_high_output_and_working_state_stay_inside_plan_ceiling() {
    let mut scrollback = BoundedScrollback::new();
    for _ in 0..(MAX_SCROLLBACK_LINES * 2) {
        scrollback.push(vec![b'x'; 256]);
        assert!(scrollback.len() <= MAX_SCROLLBACK_LINES);
        assert!(scrollback.retained_bytes() <= MAX_PANE_WORKING_BUFFER_BYTES);
    }
    assert!(scrollback.truncated());
    assert!(scrollback.dropped_rows() > 0);
    assert!(scrollback.dropped_bytes() > 0);

    let mut working = PaneWorkingBuffer::new();
    assert!(
        working
            .retain(&vec![b'w'; MAX_PANE_WORKING_BUFFER_BYTES])
            .is_ok()
    );
    assert_eq!(working.retained_bytes(), MAX_PANE_WORKING_BUFFER_BYTES);
    assert!(working.retain(b"overflow").is_err());
    working.release(&vec![b'w'; 1024]);
    assert_eq!(
        working.retained_bytes(),
        MAX_PANE_WORKING_BUFFER_BYTES - 1024
    );
    assert_eq!(working.high_water_bytes(), MAX_PANE_WORKING_BUFFER_BYTES);
}

#[test]
fn t171_exact_pane_interaction_is_generation_bound_and_authority_free() {
    let generation = TopologyGeneration::new(7).unwrap();
    let target = exact_target(generation);
    let bound =
        bind_exact_pane_interaction(target, generation, PaneInteractionKind::ContextMenu, 11, 4)
            .expect("current exact pane target should bind");
    assert_eq!(bound.target(), target);
    assert_eq!(bound.kind(), PaneInteractionKind::ContextMenu);
    assert_eq!(bound.column(), 11);
    assert_eq!(bound.row(), 4);
    assert!(bound.is_current_generation(generation));
    assert!(!bound.is_current_generation(TopologyGeneration::new(8).unwrap()));

    assert!(
        bind_exact_pane_interaction(
            target,
            TopologyGeneration::new(8).unwrap(),
            PaneInteractionKind::CopyOnSelect,
            0,
            0,
        )
        .is_none()
    );

    let mouse = bind_exact_pane_interaction(
        target,
        generation,
        PaneInteractionKind::MouseCapture {
            button: PointerButton::Left,
            phase: PointerPhase::Drag,
        },
        3,
        2,
    )
    .unwrap();
    assert_eq!(mouse.target(), target);

    let scroll = bind_exact_pane_interaction(
        target,
        generation,
        PaneInteractionKind::Scroll {
            vertical_lines: 3,
            horizontal_columns: 0,
        },
        3,
        2,
    )
    .unwrap();
    assert_eq!(scroll.target(), target);
}
