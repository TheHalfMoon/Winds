use super::{
    FindProvenance, SemanticAvailability, SessionFindInput, SessionSelection,
    canonical_existing_path, rank_sessions, select_session, semantic_availability,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn session(
    session_id: &str,
    display_name: &str,
    changed: bool,
    recent: bool,
    test_derived: bool,
    symbol_derived: bool,
) -> SessionFindInput {
    SessionFindInput {
        session_id: session_id.to_owned(),
        display_name: display_name.to_owned(),
        changed,
        recent,
        test_derived,
        symbol_derived,
    }
}

#[test]
fn t084_session_ranking_is_deterministic_and_case_insensitive() {
    let sessions = vec![
        session("s-03", "Álpha Worker", false, true, false, false),
        session("s-01", "Alpha Planner", true, false, false, false),
        session("s-02", "alphabet", false, false, false, false),
    ];

    let first = rank_sessions("ALP", &sessions);
    let second = rank_sessions("alp", &sessions);
    assert_eq!(first, second);
    assert_eq!(
        first
            .iter()
            .map(|candidate| candidate.session_id.as_str())
            .collect::<Vec<_>>(),
        vec!["s-01", "s-02"]
    );
    assert!(first[0].provenance.contains(&FindProvenance::PrefixName));
    assert!(first[0].provenance.contains(&FindProvenance::Changed));
}

#[test]
fn t084_ambiguous_selection_is_explicit_and_stably_ordered() {
    let sessions = vec![
        session("session-b", "Build", false, false, false, false),
        session("session-a", "Builder", false, false, false, false),
    ];

    let SessionSelection::Ambiguous(candidates) = select_session("bui", &sessions) else {
        panic!("expected explicit ambiguity");
    };
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].session_id, "session-b");
    assert_eq!(candidates[1].session_id, "session-a");
}

#[test]
fn t084_changed_recent_test_and_symbol_candidates_retain_provenance() {
    let sessions = vec![session("s", "Needle", true, true, true, true)];
    let candidates = rank_sessions("needle", &sessions);
    assert_eq!(candidates.len(), 1);
    let provenance = &candidates[0].provenance;
    assert!(provenance.contains(&FindProvenance::ExactName));
    assert!(provenance.contains(&FindProvenance::Changed));
    assert!(provenance.contains(&FindProvenance::Recent));
    assert!(provenance.contains(&FindProvenance::TestDerived));
    assert!(provenance.contains(&FindProvenance::SymbolDerived));
}

#[test]
fn t084_unicode_names_match_without_destroying_original_identity() {
    let sessions = vec![session("جلسة-1", "مخطط", false, false, false, false)];
    let SessionSelection::Unique(candidate) = select_session("مخط", &sessions) else {
        panic!("expected one Unicode candidate");
    };
    assert_eq!(candidate.session_id, "جلسة-1");
    assert_eq!(candidate.display_name, "مخطط");
}

#[test]
fn t084_path_selection_returns_exact_canonical_identity_and_blocks_escape() {
    let root = temp_root("canonical-path");
    let child = root.join("folder");
    fs::create_dir_all(&child).unwrap();
    fs::write(child.join("file.txt"), b"winds").unwrap();

    let selected = canonical_existing_path(&root, &PathBuf::from("folder/file.txt")).unwrap();
    assert_eq!(selected, child.join("file.txt").canonicalize().unwrap());

    let outside = temp_root("outside");
    let outside_file = outside.join("outside.txt");
    fs::write(&outside_file, b"outside").unwrap();
    let error = canonical_existing_path(&root, &outside_file).unwrap_err();
    assert!(error.contains("escapes the canonical workspace root"));

    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn t084_semantic_intelligence_remains_explicitly_unavailable() {
    assert_eq!(semantic_availability(), SemanticAvailability::Unavailable);
}

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("winds-t084-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
