use super::{
    ContextCandidateProvenance, ContinuationResolution, PathCandidateSeed, SymbolFindability,
    SymbolPathCandidateSeed, resolve_path_candidates, resolve_symbol_path_candidates,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

struct TestHome {
    path: PathBuf,
    owned_base: PathBuf,
}

impl TestHome {
    fn new(name: &str) -> Self {
        assert!(
            Path::new(name)
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
            "test-home name must contain only normal path components"
        );
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let owned_base = std::env::temp_dir().join(format!(
            "winds-t084-agentic-findability-owned-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&owned_base).unwrap();
        let path = owned_base.join(name);
        fs::create_dir(&path).unwrap();
        Self { path, owned_base }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let Ok(canonical_base) = fs::canonicalize(&self.owned_base) else {
            return;
        };
        let Ok(canonical_path) = fs::canonicalize(&self.path) else {
            return;
        };
        if canonical_path.parent() != Some(canonical_base.as_path()) {
            return;
        }
        let _ = fs::remove_dir_all(&canonical_path);
        let _ = fs::remove_dir(&canonical_base);
    }
}

fn store_with_findability_fixture(home: &TestHome) -> Store {
    let workspace_root = home.path().join("workspace");
    let git_common_dir = workspace_root.join(".git");
    fs::create_dir_all(&git_common_dir).unwrap();
    let workspace_root = workspace_root.canonicalize().unwrap();
    let git_common_dir = git_common_dir.canonicalize().unwrap();
    let workspace_root = workspace_root.to_str().unwrap();
    let git_common_dir = git_common_dir.to_str().unwrap();

    let store = Store::open(home.path()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-main",
                canonical_worktree_root: workspace_root,
                git_common_dir,
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-main",
                workspace_id: "workspace-main",
                display_name: "Findability task",
            },
            20,
        )
        .unwrap();

    for (index, (session_id, display_name)) in [
        ("session-alpha-a", "Alpha Planner"),
        ("session-alpha-b", "alpha plan review"),
        ("session-beta", "Beta"),
        ("session-reviewer", "Release Reviewer"),
        ("session-unicode", "Résumé Planner"),
    ]
    .into_iter()
    .enumerate()
    {
        store
            .create_winds_session(
                NewWindsSession {
                    session_id,
                    workstream_id: "workstream-main",
                    display_name,
                },
                30 + index as i64,
            )
            .unwrap();
    }

    store
}

fn selected_session_id(resolution: ContinuationResolution) -> String {
    match resolution {
        ContinuationResolution::Selected(session) => session.session_id,
        ContinuationResolution::Candidates(candidates) => panic!(
            "expected one selected session, got candidates: {:?}",
            candidates
                .into_iter()
                .map(|candidate| candidate.session_id)
                .collect::<Vec<_>>()
        ),
    }
}

fn candidate_session_ids(resolution: ContinuationResolution) -> Vec<String> {
    match resolution {
        ContinuationResolution::Selected(session) => {
            panic!("expected explicit ambiguity, got selected session: {}", session.session_id)
        }
        ContinuationResolution::Candidates(candidates) => candidates
            .into_iter()
            .map(|candidate| candidate.session_id)
            .collect(),
    }
}

#[test]
fn session_findability_is_case_unicode_and_fuzzy_aware_without_silent_ambiguity() {
    let home = TestHome::new("sessions");
    let store = store_with_findability_fixture(&home);

    let exact = store
        .resolve_winds_continuation("workspace-main", Some("session-beta"))
        .unwrap();
    assert_eq!(selected_session_id(exact), "session-beta");

    let unicode_case = store
        .resolve_winds_continuation("workspace-main", Some("RÉSUMÉ"))
        .unwrap();
    assert_eq!(selected_session_id(unicode_case), "session-unicode");

    let fuzzy = store
        .resolve_winds_continuation("workspace-main", Some("rlsrvwr"))
        .unwrap();
    assert_eq!(selected_session_id(fuzzy), "session-reviewer");

    let ambiguous = store
        .resolve_winds_continuation("workspace-main", Some("ALPHA"))
        .unwrap();
    assert_eq!(
        candidate_session_ids(ambiguous),
        vec!["session-alpha-a".to_owned(), "session-alpha-b".to_owned()]
    );

    let repeated = store
        .resolve_winds_continuation("workspace-main", Some("ALPHA"))
        .unwrap();
    assert_eq!(
        candidate_session_ids(repeated),
        vec!["session-alpha-a".to_owned(), "session-alpha-b".to_owned()]
    );
}

#[test]
fn path_findability_canonicalizes_files_and_directories_and_merges_provenance() {
    let home = TestHome::new("paths");
    let root = home.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("src/Résumé.rs"), "pub fn résumé() {}\n").unwrap();
    fs::write(root.join("tests/findability.rs"), "#[test] fn fixture() {}\n").unwrap();

    let seeds = [
        PathCandidateSeed {
            path: Path::new("src/Résumé.rs"),
            provenance: ContextCandidateProvenance::Changed,
        },
        PathCandidateSeed {
            path: Path::new("src/Résumé.rs"),
            provenance: ContextCandidateProvenance::Recent,
        },
        PathCandidateSeed {
            path: Path::new("tests/findability.rs"),
            provenance: ContextCandidateProvenance::Test,
        },
        PathCandidateSeed {
            path: Path::new("src"),
            provenance: ContextCandidateProvenance::Recent,
        },
    ];

    let candidates = resolve_path_candidates(&root, &seeds).unwrap();
    assert_eq!(candidates.len(), 3);

    let résumé_path = root.join("src/Résumé.rs").canonicalize().unwrap();
    let résumé = candidates
        .iter()
        .find(|candidate| candidate.canonical_path == résumé_path)
        .unwrap();
    assert_eq!(
        résumé.provenance,
        vec![
            ContextCandidateProvenance::Changed,
            ContextCandidateProvenance::Recent,
        ]
    );

    let test_path = root.join("tests/findability.rs").canonicalize().unwrap();
    let test_candidate = candidates
        .iter()
        .find(|candidate| candidate.canonical_path == test_path)
        .unwrap();
    assert_eq!(
        test_candidate.provenance,
        vec![ContextCandidateProvenance::Test]
    );

    let src_path = root.join("src").canonicalize().unwrap();
    assert!(candidates.iter().any(|candidate| {
        candidate.canonical_path == src_path
            && candidate.provenance == vec![ContextCandidateProvenance::Recent]
    }));
}

#[test]
fn path_findability_fails_closed_outside_workspace_and_grants_no_root_expansion() {
    let home = TestHome::new("boundary");
    let root = home.path().join("repo");
    let outside = home.path().join("outside");
    fs::create_dir_all(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    let outside_file = outside.join("visible-but-not-authorized.txt");
    fs::write(&outside_file, "visibility is not authority\n").unwrap();

    let seeds = [PathCandidateSeed {
        path: &outside_file,
        provenance: ContextCandidateProvenance::Recent,
    }];
    let error = resolve_path_candidates(&root, &seeds).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("outside the canonical workspace root")
    );
}

#[test]
fn symbol_findability_preserves_provenance_and_reports_unavailable_semantics_explicitly() {
    let home = TestHome::new("symbols");
    let root = home.path().join("repo");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn verify_candidate() {}\n").unwrap();

    assert_eq!(
        resolve_symbol_path_candidates(&root, None).unwrap(),
        SymbolFindability::Unavailable
    );

    let empty: [SymbolPathCandidateSeed<'_>; 0] = [];
    assert_eq!(
        resolve_symbol_path_candidates(&root, Some(&empty)).unwrap(),
        SymbolFindability::Candidates(Vec::new())
    );

    let symbols = [SymbolPathCandidateSeed {
        path: Path::new("src/lib.rs"),
        symbol: "verify_candidate",
    }];
    let resolved = resolve_symbol_path_candidates(&root, Some(&symbols)).unwrap();
    let SymbolFindability::Candidates(candidates) = resolved else {
        panic!("available symbol intelligence must not be reported unavailable");
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        candidates[0].canonical_path,
        root.join("src/lib.rs").canonicalize().unwrap()
    );
    assert_eq!(
        candidates[0].provenance,
        vec![ContextCandidateProvenance::Symbol(
            "verify_candidate".to_owned()
        )]
    );

    let invalid = [SymbolPathCandidateSeed {
        path: Path::new("src/lib.rs"),
        symbol: "   ",
    }];
    assert!(resolve_symbol_path_candidates(&root, Some(&invalid)).is_err());
}