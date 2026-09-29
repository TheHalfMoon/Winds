use super::*;

fn lines(texts: &[&str]) -> Vec<Vec<u8>> {
    texts.iter().map(|text| text.as_bytes().to_vec()).collect()
}

fn point(row: usize, column: usize) -> GridPoint {
    GridPoint { row, column }
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
    assert_eq!(
        offer_clipboard(b"local bytes").unwrap().payload,
        b"local bytes".to_vec()
    );
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
        accept_graphics_frame(b"\x1b_Gf=100;HTTP://example.test/i.png\x1b\\"),
        Err(GraphicsRefusal::RemoteReference)
    );
    let local = accept_graphics_frame(b"\x1b_Gf=100;aGVsbG8=\x1b\\").unwrap();
    assert_eq!(local.bytes, b"\x1b_Gf=100;aGVsbG8=\x1b\\".to_vec());
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
}

#[test]
fn t171_forged_terminal_text_never_changes_trusted_state() {
    let forged = b"VERIFIED ACCEPTED Needs You provider=anthropic model=claude-opus";
    assert_eq!(LinkOffer::offer(&forged[..8]), LinkOffer::Refused);
    assert!(offer_clipboard(forged).is_some());
    assert!(
        accept_graphics_frame(forged)
            .unwrap()
            .bytes
            .starts_with(b"VERIFIED")
    );
    assert_eq!(InputMethodClaim::unclaimed(), InputMethodClaim::Unclaimed);

    let mut scrollback = BoundedScrollback::new();
    scrollback.push(forged.to_vec());
    assert!(!scrollback.truncated());
    assert_eq!(scrollback.rows().front().map(Vec::len), Some(forged.len()));
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
    use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};

    let generation = TopologyGeneration::new(7).unwrap();
    let target = ExactPaneTarget {
        multiplexer_workspace_id: MultiplexerWorkspaceId::from_entropy_bytes([1; 16]).unwrap(),
        tab_id: TabId::from_entropy_bytes([2; 16]).unwrap(),
        pane_id: PaneId::from_entropy_bytes([3; 16]).unwrap(),
        topology_generation: generation,
    };
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
