use super::*;
use crate::domain::{
    BlobEvidence, CheckEvidence, CheckStatus, Eligibility, EvidenceReport, WorkspaceRecord,
};
use crate::git::Repo;
use crate::store::{NewRun, Store};
use crossterm::event::{KeyEvent, KeyEventState};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_EVIDENCE_FIXTURE: AtomicU64 = AtomicU64::new(1);

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn workspace(id: &str, root: &str) -> WorkspaceRecord {
    WorkspaceRecord {
        workspace_id: id.to_owned(),
        canonical_worktree_root: root.to_owned(),
        git_common_dir: format!("{root}/.git"),
        created_unix_ms: 1,
        last_opened_unix_ms: 1,
    }
}

struct EvidenceFixture {
    root: PathBuf,
    state: PathBuf,
}

impl EvidenceFixture {
    fn new(name: &str) -> Self {
        let sequence = NEXT_EVIDENCE_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "winds-t098-evidence-{name}-{}-{sequence}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("repo");
        let state = base.join("state");
        std::fs::create_dir_all(&root).expect("create T098 evidence repository root");
        let fixture = Self { root, state };
        fixture.git(&["init"]);
        fixture.git(&["config", "user.email", "t098@example.invalid"]);
        fixture.git(&["config", "user.name", "T098 Fixture"]);
        fixture
    }

    fn repo(&self) -> Repo {
        Repo::open(&self.root).expect("open T098 evidence repository")
    }

    fn store(&self) -> Store {
        Store::open(&self.state).expect("open T098 evidence store")
    }

    fn commit(&self, name: &str, content: &str) -> String {
        std::fs::write(self.root.join(name), content).expect("write T098 evidence fixture file");
        self.git(&["add", "--", name]);
        self.git(&["commit", "-m", &format!("T098 fixture {name}")]);
        self.git_text(&["rev-parse", "HEAD"])
    }

    fn tree(&self, oid: &str) -> String {
        self.git_text(&["rev-parse", &format!("{oid}^{{tree}}")])
    }

    fn persist_eligible(
        &self,
        store: &mut Store,
        run_id: &str,
        candidate_oid: &str,
        candidate_tree: &str,
    ) {
        let repo_path = self
            .root
            .to_str()
            .expect("T098 evidence repository path is UTF-8");
        store
            .create_run(
                NewRun {
                    run_id,
                    repo_path,
                    base_oid: candidate_oid,
                    candidate_ref: "HEAD",
                    candidate_oid,
                    candidate_tree,
                    worktree_path: repo_path,
                    check_command: "cargo test --locked",
                    timeout_secs: 60,
                },
                1,
            )
            .expect("create T098 verification run");
        store
            .mark_workspace_ready(run_id, 2)
            .expect("mark T098 verification run ready");
        let report = EvidenceReport {
            schema_version: 1,
            run_id: run_id.to_owned(),
            authority: "WINDS_OBSERVED",
            repo_path: repo_path.to_owned(),
            base_oid: candidate_oid.to_owned(),
            candidate_ref: "HEAD".to_owned(),
            candidate_oid: candidate_oid.to_owned(),
            candidate_tree: candidate_tree.to_owned(),
            worktree_path: repo_path.to_owned(),
            check: CheckEvidence {
                authority: "WINDS_OBSERVED",
                command: "cargo test --locked".to_owned(),
                status: CheckStatus::Pass,
                exit_code: Some(0),
                duration_ms: 1,
                stdout: BlobEvidence {
                    relative_path: "fixture/stdout".to_owned(),
                    sha256: "0".repeat(64),
                    captured_bytes: 0,
                    truncated: false,
                },
                stderr: BlobEvidence {
                    relative_path: "fixture/stderr".to_owned(),
                    sha256: "1".repeat(64),
                    captured_bytes: 0,
                    truncated: false,
                },
            },
            eligibility: Eligibility::Eligible,
            warnings: Vec::new(),
        };
        store
            .save_evidence_for_test(&report, 3)
            .expect("persist T098 eligible evidence");
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run T098 evidence git fixture");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_text(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run T098 evidence git text fixture");
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .expect("T098 evidence git output is UTF-8")
            .trim()
            .to_owned()
    }
}

impl Drop for EvidenceFixture {
    fn drop(&mut self) {
        if let Some(base) = self.root.parent() {
            let _ = std::fs::remove_dir_all(base);
        }
    }
}

#[test]
fn t098_keyboard_paths_cover_core_actions_search_and_verification_inspection_without_pointer() {
    let mut state = WorkbenchState::new();
    state.create_pane(
        "primary",
        Some("workspace-1".into()),
        Some("session-1".into()),
        PaneSize::new(80, 24),
    );
    let workspaces = vec![workspace("workspace-1", "/repos/winds")];
    let mut terminals = terminal::WorkbenchTerminals::new();
    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let mut navigation = ui::WorkbenchNavigation::new();
    let mut accessibility = WorkbenchAccessibilityState::default();

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "workspace-1".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &workspaces,
                &[],
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    let search = navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert!(matches!(
        search,
        ui::NavigationEffect::Find(ui::FindResolution::Unique(ui::FindMatch {
            target: ui::NavigationTarget::Workspace { ref workspace_id },
            ..
        })) if workspace_id == "workspace-1"
    ));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('n'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 2);

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('h'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 3);

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('v'), KeyModifiers::ALT),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 4);

    let before_focus = state.selected_pane();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Left, KeyModifiers::ALT),
        )
        .unwrap();
    assert_ne!(state.selected_pane(), before_focus);

    let selected = state.selected_pane().unwrap();
    let before_size = state.pane(selected).unwrap().size;
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT),
        )
        .unwrap();
    assert_eq!(
        state.pane(selected).unwrap().size.columns,
        before_size.columns.saturating_add(1)
    );

    let pane_count = state.panes().len();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('w'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), pane_count - 1);

    assert!(accessibility.handle_event(&key(KeyCode::Char('e'), KeyModifiers::CONTROL), false));
    assert!(accessibility.verification_inspection_open());

    assert!(accessibility.handle_event(&key(KeyCode::Enter, KeyModifiers::NONE), false));
    assert!(editor.lines().iter().all(String::is_empty));
    assert!(accessibility.verification_inspection_open());

    assert!(accessibility.handle_event(&key(KeyCode::Esc, KeyModifiers::NONE), false));
    assert!(!accessibility.verification_inspection_open());
}

#[test]
fn t098_selected_and_lifecycle_states_are_explicit_text_not_color_only() {
    let mut state = WorkbenchState::new();
    let live = state.create_pane("live", None, None, PaneSize::new(80, 24));
    let exited = state.create_pane("exited", None, None, PaneSize::new(80, 24));
    let ownership_lost = state.create_pane("ownership", None, None, PaneSize::new(80, 24));
    let error = state.create_pane("error", None, None, PaneSize::new(80, 24));
    state.set_pane_lifecycle(live, PaneLifecycleView::Live);
    state.set_pane_lifecycle(exited, PaneLifecycleView::Exited);
    state.set_pane_lifecycle(ownership_lost, PaneLifecycleView::OwnershipLost);
    state.set_pane_lifecycle(error, PaneLifecycleView::Error);
    state.focus_pane(ownership_lost);

    let text = pane_accessibility_text(&state, true);
    assert!(text.contains("selection=SELECTED lifecycle=OWNERSHIP_LOST"));
    assert!(text.contains("selection=NOT_SELECTED lifecycle=LIVE"));
    assert!(text.contains("lifecycle=EXITED"));
    assert!(text.contains("lifecycle=ERROR"));
}

#[test]
fn t098_verification_projection_wording_keeps_done_verified_and_accepted_distinct() {
    assert_eq!(
        agent_progress_accessibility_label(context::AgentProgressProjection::AgentReportedDone),
        "AGENT_REPORTED_DONE"
    );
    assert_eq!(
        verification_accessibility_label(context::VerificationProjectionState::NotRun),
        "VERIFICATION_NOT_RUN"
    );
    assert_eq!(
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        ),
        "VERIFIED_FOR_EXACT_CANDIDATE"
    );
    assert_eq!(
        verification_accessibility_label(context::VerificationProjectionState::Stale),
        "VERIFICATION_STALE_NOT_APPLICABLE"
    );
    assert_eq!(
        acceptance_accessibility_label(
            context::HumanAcceptanceProjectionState::AcceptedForExactCandidate
        ),
        "HUMAN_ACCEPTED_FOR_EXACT_CANDIDATE"
    );
    assert_ne!(
        agent_progress_accessibility_label(context::AgentProgressProjection::AgentReportedDone),
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        )
    );
    assert_ne!(
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        ),
        acceptance_accessibility_label(
            context::HumanAcceptanceProjectionState::AcceptedForExactCandidate
        )
    );

    let unloaded = verification_inspection_text("candidate-oid", "candidate-tree", None);
    assert!(unloaded.contains("verification_state=CANONICAL_EVIDENCE_NOT_LOADED"));
    assert!(unloaded.contains("INVARIANT=AGENT_REPORTED_DONE != VERIFIED != ACCEPTED"));
    assert!(unloaded.contains("MODE=READ_ONLY"));
}

#[test]
fn t098_production_context_loader_projects_current_and_stale_persisted_evidence() {
    let fixture = EvidenceFixture::new("projection");
    let candidate_a = fixture.commit("a.txt", "a\n");
    let candidate_a_tree = fixture.tree(&candidate_a);
    let repo = fixture.repo();
    let mut store = fixture.store();
    fixture.persist_eligible(&mut store, "verify-a", &candidate_a, &candidate_a_tree);

    let current = load_workbench_candidate_context(&repo, &store).unwrap();
    assert_eq!(current.candidate.oid, candidate_a);
    assert_eq!(
        current.verification.state,
        context::VerificationProjectionState::VerifiedForExactCandidate
    );
    assert_eq!(current.verification.applicable_evidence_count, 1);
    assert_eq!(current.verification.stale_evidence_count, 0);
    let current_text = verification_inspection_text(
        &current.candidate.oid,
        &current.candidate.tree,
        Some(&current),
    );
    assert!(current_text.contains("verification_state=VERIFIED_FOR_EXACT_CANDIDATE"));
    assert!(current_text.contains("evidence_applicability=applicable:1 stale:0"));
    assert!(current_text.contains("review_state=CANONICAL_REVIEW_NOT_LOADED"));
    assert!(current_text.contains("human_acceptance=CANONICAL_DECISION_NOT_LOADED"));

    let candidate_b = fixture.commit("b.txt", "b\n");
    let stale = load_workbench_candidate_context(&repo, &store).unwrap();
    assert_eq!(stale.candidate.oid, candidate_b);
    assert_eq!(
        stale.verification.state,
        context::VerificationProjectionState::Stale
    );
    assert_eq!(stale.verification.applicable_evidence_count, 0);
    assert_eq!(stale.verification.stale_evidence_count, 1);
    let stale_text = verification_inspection_text(
        &stale.candidate.oid,
        &stale.candidate.tree,
        Some(&stale),
    );
    assert!(stale_text.contains("verification_state=VERIFICATION_STALE_NOT_APPLICABLE"));
    assert!(stale_text.contains("evidence_applicability=applicable:0 stale:1"));
}

#[test]
fn t098_small_terminal_fallback_is_deterministic_and_uses_canonical_identity_not_title() {
    assert!(use_compact_workbench_layout(59, 10));
    assert!(use_compact_workbench_layout(60, 9));
    assert!(!use_compact_workbench_layout(60, 10));

    let mut state = WorkbenchState::new();
    state.create_pane(
        "transient 界e\u{301}",
        Some("workspace-canonical".into()),
        Some("session-canonical".into()),
        PaneSize::new(40, 8),
    );
    let output = output::WorkbenchOutput::new();
    let accessibility = WorkbenchAccessibilityState::default();

    let first = compact_workbench_text(
        &state,
        &output,
        accessibility,
        "0123456789abcdef",
        "fedcba9876543210",
    );
    let second = compact_workbench_text(
        &state,
        &output,
        accessibility,
        "0123456789abcdef",
        "fedcba9876543210",
    );
    assert_eq!(first, second);
    assert!(first.contains("workspace=workspace-canonical"));
    assert!(first.contains("session=session-canonical"));
    assert!(first.contains("candidate_oid=0123456789abcdef"));
    assert!(first.contains("candidate_tree=fedcba9876543210"));
    assert!(first.contains("selection=SELECTED"));
    assert!(!first.contains("transient 界e\u{301}"));
}

#[test]
fn t098_unicode_wide_combining_selection_copy_fixture_preserves_identity_and_exact_text() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "editor",
        Some("workspace-u".into()),
        Some("session-u".into()),
        PaneSize::new(80, 24),
    );
    let identity_before = (
        state.pane(pane).unwrap().canonical_workspace_id.clone(),
        state.pane(pane).unwrap().canonical_winds_session_id.clone(),
        state.selected_pane(),
    );

    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let fixture = "wide=界 combining=e\u{301} emoji=🙂";
    editor.insert_text(fixture).unwrap();
    editor.select_all();
    assert!(editor.is_selecting());

    let copied_fixture = editor.lines().join("\n");
    assert_eq!(copied_fixture.as_bytes(), fixture.as_bytes());
    editor.cancel_selection();

    let identity_after = (
        state.pane(pane).unwrap().canonical_workspace_id.clone(),
        state.pane(pane).unwrap().canonical_winds_session_id.clone(),
        state.selected_pane(),
    );
    assert_eq!(identity_after, identity_before);
}
