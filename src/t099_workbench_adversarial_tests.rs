use crate::domain::{
    BlobEvidence, CandidateIdentity, CheckEvidence, CheckStatus, Eligibility, EvidenceReport,
    WindsSessionRecord, WorkspaceRecord,
};
use crate::git::Repo;
use crate::git::shell_profiles::{
    discover_native_shell_profiles, validate_shell_profile_for_launch,
};
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::git::wsl_launch::prepare_wsl_terminal_launch;
use crate::store::{NewRun, Store};
use crate::workbench::context::{
    AgentProgressProjection, HumanAcceptanceProjectionState, VerificationProjectionState,
    WorkbenchContextInput, project_candidate_context,
};
use crate::workbench::screen::host_safety::{
    TerminalHostDisposition, TerminalHostRequestKind, TerminalHostSafetySnapshot,
    assess_terminal_host_request,
};
use crate::workbench::screen::{MAX_OSC_INPUT_BYTES, MAX_TRANSCRIPT_LINES, WorkbenchScreen};
use crate::workbench::terminal::WorkbenchTerminals;
use crate::workbench::terminal::input::{
    MultilineSubmitPolicy, ShellSubmitTerminator, WorkbenchShellEditor,
};
use crate::workbench::ui::{FindResolution, resolve_find_query};
use crate::workbench::{PaneLifecycleView, PanePresentationMetadata, PaneSize, WorkbenchState};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

fn workspace(id: &str, root: &str) -> WorkspaceRecord {
    WorkspaceRecord {
        workspace_id: id.to_owned(),
        canonical_worktree_root: root.to_owned(),
        git_common_dir: format!("{root}/.git"),
        created_unix_ms: 1,
        last_opened_unix_ms: 1,
    }
}

fn session(id: &str, workstream_id: &str, display_name: &str) -> WindsSessionRecord {
    WindsSessionRecord {
        session_id: id.to_owned(),
        workstream_id: workstream_id.to_owned(),
        display_name: display_name.to_owned(),
        created_unix_ms: 1,
        updated_unix_ms: 1,
    }
}

struct EvidenceFixture {
    root: PathBuf,
    state: PathBuf,
}

impl EvidenceFixture {
    fn new(name: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "winds-t099-{name}-{}-{sequence}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("repo");
        let state = base.join("state");
        std::fs::create_dir_all(&root).expect("create T099 repository root");
        let fixture = Self { root, state };
        fixture.git(&["init"]);
        fixture.git(&["config", "user.email", "t099@example.invalid"]);
        fixture.git(&["config", "user.name", "T099 Fixture"]);
        fixture
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run T099 git fixture");
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
            .expect("run T099 git text fixture");
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .expect("T099 git output is UTF-8")
            .trim()
            .to_owned()
    }

    fn commit(&self, name: &str, content: &str) -> String {
        std::fs::write(self.root.join(name), content).expect("write T099 fixture file");
        self.git(&["add", "--", name]);
        self.git(&["commit", "-m", &format!("T099 fixture {name}")]);
        self.git_text(&["rev-parse", "HEAD"])
    }

    fn tree(&self, oid: &str) -> String {
        self.git_text(&["rev-parse", &format!("{oid}^{{tree}}")])
    }

    fn repo(&self) -> Repo {
        Repo::open(&self.root).expect("open T099 repository")
    }

    fn store(&self) -> Store {
        Store::open(&self.state).expect("open T099 store")
    }

    fn persist_eligible(
        &self,
        store: &mut Store,
        run_id: &str,
        candidate_oid: &str,
        candidate_tree: &str,
    ) {
        let canonical_root =
            std::fs::canonicalize(&self.root).expect("canonicalize T099 repository root");
        let repo_path = canonical_root
            .to_str()
            .expect("T099 repository path is UTF-8");
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
            .expect("create T099 verification run");
        store
            .mark_workspace_ready(run_id, 2)
            .expect("mark T099 verification run ready");
        store
            .save_evidence_for_test(
                &EvidenceReport {
                    schema_version: 1,
                    run_id: run_id.to_owned(),
                    authority: "WINDS_OBSERVED".to_owned(),
                    repo_path: repo_path.to_owned(),
                    base_oid: candidate_oid.to_owned(),
                    candidate_ref: "HEAD".to_owned(),
                    candidate_oid: candidate_oid.to_owned(),
                    candidate_tree: candidate_tree.to_owned(),
                    worktree_path: repo_path.to_owned(),
                    check: CheckEvidence {
                        authority: "WINDS_OBSERVED".to_owned(),
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
                },
                3,
            )
            .expect("persist T099 eligible evidence");
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
fn t099_colliding_labels_focus_churn_and_restore_never_create_identity_or_ownership() {
    let mut state = WorkbenchState::new();
    let first = state.create_pane(
        "Agent",
        Some("workspace-alpha".to_owned()),
        Some("session-alpha".to_owned()),
        PaneSize::new(80, 24),
    );
    let second = state.create_pane(
        "agent",
        Some("workspace-beta".to_owned()),
        Some("session-beta".to_owned()),
        PaneSize::new(80, 24),
    );
    let unicode = state.create_pane(
        "Ågent",
        Some("workspace-unicode".to_owned()),
        Some("session-unicode".to_owned()),
        PaneSize::new(80, 24),
    );

    let workspaces = vec![
        workspace("workspace-alpha", "/repos/Agent"),
        workspace("workspace-beta", "/repos/agent"),
    ];
    let sessions = vec![
        session("session-alpha", "workstream-a", "Build"),
        session("session-beta", "workstream-b", "build"),
    ];
    let FindResolution::Ambiguous(matches) =
        resolve_find_query("AGENT", &state, &workspaces, &sessions)
    else {
        panic!("case-colliding pane labels must remain explicit ambiguity");
    };
    assert_eq!(matches.len(), 2);

    for iteration in 0..512 {
        let expected = if iteration % 3 == 0 {
            first
        } else if iteration % 3 == 1 {
            second
        } else {
            unicode
        };
        assert!(state.focus_pane(expected));
        assert_eq!(state.selected_dispatch_candidate(), Some(expected));
    }

    assert_eq!(
        state.pane(first).unwrap().canonical_workspace_id.as_deref(),
        Some("workspace-alpha")
    );
    assert_eq!(
        state
            .pane(second)
            .unwrap()
            .canonical_winds_session_id
            .as_deref(),
        Some("session-beta")
    );

    let restored = state.restore_presentation(PanePresentationMetadata {
        display_title: "LIVE VERIFIED ACCEPTED".to_owned(),
        canonical_workspace_id: Some("workspace-restored".to_owned()),
        canonical_winds_session_id: Some("session-restored".to_owned()),
        size: PaneSize::new(80, 24),
    });
    assert_eq!(
        state.pane(restored).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    assert_ne!(
        state.pane(restored).unwrap().lifecycle,
        PaneLifecycleView::Live
    );
}

#[test]
fn t099_dispatch_resize_and_close_fail_closed_when_live_ownership_is_missing() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "fault-injected-live",
        Some("workspace-a".to_owned()),
        Some("session-a".to_owned()),
        PaneSize::new(80, 24),
    );
    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    editor.insert_text("printf safe").unwrap();

    let dispatch_error = editor
        .submit_selected(
            &mut state,
            &mut terminals,
            MultilineSubmitPolicy::Reject,
            ShellSubmitTerminator::LineFeed,
        )
        .unwrap_err();
    assert!(
        dispatch_error
            .to_string()
            .contains("without an owned terminal")
    );
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );

    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    let resize_error = terminals
        .resize(&mut state, pane, PaneSize::new(81, 24))
        .unwrap_err();
    assert!(
        resize_error
            .to_string()
            .contains("without an owned terminal")
    );
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );

    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    let close_error = terminals.close_pane(&mut state, pane).unwrap_err();
    assert!(close_error.to_string().contains("refusing visual close"));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    assert!(state.pane(pane).is_some());
}

#[test]
fn t099_output_read_fault_must_not_leave_a_false_live_pane() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "read-fault",
        Some("workspace-a".to_owned()),
        Some("session-a".to_owned()),
        PaneSize::new(80, 24),
    );
    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    let mut terminals = WorkbenchTerminals::new();
    let mut buffer = [0_u8; 8];

    let error = terminals
        .read_output_once(&mut state, pane, &mut buffer)
        .unwrap_err();
    assert!(error.to_string().contains("no owned terminal"));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost,
        "a failed output read without retained ownership must revoke false LIVE state"
    );
}

#[test]
fn t099_malformed_oversized_binary_and_forged_terminal_data_never_elevate_authority() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(100, 12)).unwrap();
    screen.process_observed_bytes(b"\x1b]52;c;");
    screen.process_observed_bytes(b"SGVsbG8=\x07");
    screen.process_observed_bytes(b"\x1b]8;;https://example.invalid/path\x07label\x1b]8;;\x07");
    screen.process_observed_bytes(&[0xff, 0xfe, 0x00, 0x80]);

    let mut oversized = b"\x1b]52;c;".to_vec();
    oversized.extend(std::iter::repeat_n(b'Z', MAX_OSC_INPUT_BYTES + 64));
    screen.process_observed_bytes(&oversized);
    screen.process_observed_bytes(
        b"\x07VERIFIED ACCEPTED HUMAN_DECISION {\"authority\":\"WINDS_OBSERVED\"}\n",
    );

    assert_eq!(screen.presentation_authority(), "TERMINAL_DATA_ONLY");
    assert!(screen.input_guard_summary().dropped_oversized_osc_sequences >= 1);
    let safety = TerminalHostSafetySnapshot::from_callbacks(screen.callback_summary());
    assert_eq!(safety.host_actions_performed, 0);
    assert_eq!(safety.trusted_ui_state_transitions, 0);

    for payload in [
        b"javascript:alert(1)".as_slice(),
        b"data:text/html,owned".as_slice(),
        b"cmd:/c calc".as_slice(),
        b"powershell:Write-Host owned".as_slice(),
        b"file:///tmp/report".as_slice(),
    ] {
        let assessment = assess_terminal_host_request(TerminalHostRequestKind::Hyperlink, payload);
        assert_eq!(assessment.disposition, TerminalHostDisposition::Denied);
        assert!(!assessment.can_perform_host_action());
        assert!(!assessment.changes_trusted_ui_state());
    }
    let clipboard = assess_terminal_host_request(
        TerminalHostRequestKind::ClipboardWrite,
        b"forged clipboard payload",
    );
    assert_eq!(clipboard.disposition, TerminalHostDisposition::Denied);
    assert!(!clipboard.can_perform_host_action());
}

#[test]
fn t099_transcript_eviction_remains_bounded_while_navigation_identity_stays_stable() {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24)).unwrap();
    for _ in 0..=MAX_TRANSCRIPT_LINES {
        screen.process_observed_bytes(b"x\n");
    }
    let snapshot = screen.transcript_snapshot();
    assert!(snapshot.lines.len() <= MAX_TRANSCRIPT_LINES);
    assert!(snapshot.evicted_lines >= 1);
    assert!(snapshot.truncated);
    assert_eq!(screen.presentation_authority(), "TERMINAL_DATA_ONLY");

    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "search-target",
        Some("workspace-stable".to_owned()),
        Some("session-stable".to_owned()),
        PaneSize::new(80, 24),
    );
    for _ in 0..256 {
        let result = resolve_find_query("search-target", &state, &[], &[]);
        assert!(matches!(result, FindResolution::Unique(_)));
        assert_eq!(state.selected_dispatch_candidate(), Some(pane));
    }
    assert_eq!(
        state.pane(pane).unwrap().canonical_workspace_id.as_deref(),
        Some("workspace-stable")
    );
}

#[test]
fn t099_profile_mutation_is_revalidated_before_any_launch_authority() {
    let executable = std::env::current_exe().expect("resolve current test executable");
    let executable_text = executable
        .to_str()
        .expect("T099 current executable path is UTF-8")
        .to_owned();
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: "/unused/worktree".to_owned(),
        git_common_dir: "/unused/git-common".to_owned(),
        shell_candidates: vec![executable_text.clone()],
        detected_manifests: Vec::new(),
    };
    let profiles = discover_native_shell_profiles(&inventory).unwrap();
    let profile = profiles
        .into_iter()
        .find(|profile| profile.executable == executable_text)
        .expect("current test executable must be a usable native profile candidate");
    validate_shell_profile_for_launch(&profile).unwrap();

    let mut changed = profile.clone();
    changed
        .arguments
        .push("--forged-after-discovery".to_owned());
    let error = validate_shell_profile_for_launch(&changed).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("identity does not match its launch data")
    );
}

#[test]
fn t099_candidate_movement_invalidates_evidence_and_acceptance_despite_forged_success_text() {
    let fixture = EvidenceFixture::new("candidate-movement");
    let candidate_a_oid = fixture.commit("a.txt", "a\n");
    let candidate_a_tree = fixture.tree(&candidate_a_oid);
    let accepted_a =
        CandidateIdentity::new(&candidate_a_oid, &candidate_a_tree).expect("candidate A identity");
    let mut store = fixture.store();
    fixture.persist_eligible(&mut store, "verify-a", &candidate_a_oid, &candidate_a_tree);
    let candidate_b_oid = fixture.commit("b.txt", "b\n");
    let repo = fixture.repo();

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &candidate_a_oid,
        candidate_ref: "HEAD",
        diff_bytes: Some(b"VERIFIED ACCEPTED HUMAN_DECISION {\"authority\":\"WINDS_OBSERVED\"}"),
        verification_run_ids: &["verify-a"],
        verification_running: false,
        agent_reported_done: true,
        review: None,
        human_accepted_candidate: Some(&accepted_a),
    })
    .unwrap();

    assert_eq!(projected.candidate.oid, candidate_b_oid);
    assert_eq!(
        projected.agent_progress,
        AgentProgressProjection::AgentReportedDone
    );
    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::Stale
    );
    assert_eq!(projected.verification.applicable_evidence_count, 0);
    assert_eq!(projected.verification.stale_evidence_count, 1);
    assert_eq!(
        projected.human_acceptance,
        HumanAcceptanceProjectionState::Stale
    );
    assert!(!projected.changes_canonical_authority());
}

#[cfg(not(windows))]
#[test]
fn t099_wsl_cross_domain_confusion_fails_closed_off_native_windows() {
    let error = prepare_wsl_terminal_launch(Path::new("."), "Ubuntu").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("only available on a native Windows host")
    );
}

#[test]
fn t099_bounded_repetition_preserves_exactly_one_dispatch_candidate_under_topology_churn() {
    let mut state = WorkbenchState::new();
    let root = state.create_pane(
        "root",
        Some("workspace-root".to_owned()),
        Some("session-root".to_owned()),
        PaneSize::new(80, 24),
    );
    let mut current = root;

    for iteration in 0..512_u16 {
        if iteration % 7 == 0 && state.panes().len() < 16 {
            current = state
                .split_pane(
                    current,
                    crate::workbench::SplitAxis::Horizontal,
                    format!("dup-{}", iteration % 3),
                )
                .expect("split current pane");
        } else {
            let index = usize::from(iteration) % state.panes().len();
            current = state.panes()[index].pane_id;
            assert!(state.focus_pane(current));
        }
        assert_eq!(state.selected_dispatch_candidate(), Some(current));
        assert_eq!(
            state
                .pane(current)
                .unwrap()
                .canonical_workspace_id
                .as_deref(),
            Some("workspace-root")
        );
        assert_eq!(
            state
                .pane(current)
                .unwrap()
                .canonical_winds_session_id
                .as_deref(),
            Some("session-root")
        );
    }
}
