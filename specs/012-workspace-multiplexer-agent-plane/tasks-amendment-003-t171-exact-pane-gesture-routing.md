# Spec 012 Tasks Amendment 003 — T171 Exact-Pane Terminal Gesture Routing

Status: CLOSED_CANONICAL

Authority basis: Winds Constitution 1.1.0 principles III, IV, and V; canonical Spec 012 FR-016, FR-022, FR-024, FR-026, FR-030; canonical Plan AD-012-11 through AD-012-14; and canonical T168 and T170 closeouts.

## Material contradiction

Canonical T171 requires that `mouse/right-click/copy-on-select/scroll routing binds exact pane`. Canonical T171 further names FR-022 as primary coverage, and FR-022 states that mouse capture, right-click routing, copy-on-select, and scroll tuning MUST never retarget input because focus changed asynchronously.

The accepted T171 path list permits only `src/terminal.rs`, `src/workbench_terminal.rs`, `src/workbench_output.rs` and focused helpers, `desktop/src/terminal/**`, the focused test file `src/t171_terminal_ux_tests.rs`, and desktop tests. It omits the two production files that actually own terminal gesture routing in the existing TUI workbench:

- `src/workbench.rs`, which declares every workbench submodule and is the only place a focused helper module can be registered;
- `src/workbench_ui.rs`, which canonical T168 already narrowed-authorized and which owns `CanonicalTopologyPresentation`, `CanonicalTopologyPaneHitRegion`, `canonical_topology_bind_pointer_focus_intent`, and the mouse-event dispatch that every pointer gesture in the workbench passes through.

There is no alternative in-scope path that can satisfy the T171 requirement. A pure model in a new focused helper can compute an exact-pane bound interaction, but it can never prove that the workbench pointer path consumes that exact identity instead of re-deriving a target from current focus at action time. A T171 candidate that omitted `src/workbench_ui.rs` would therefore be unable to discharge FR-022 at all, and would silently substitute a legacy `workbench::PaneId` focus path for the canonical `MultiplexerWorkspaceId`/`TabId`/`PaneId`/`TopologyGeneration` identity that FR-022 requires.

This is a narrow authorized-path omission. It is not authority to alter the T168 canonical topology projection, the T170 exact-pane/runtime binding, any accepted PTY/ConPTY ownership boundary, or any T166 protocol-v2 message kind or schema.

## Narrow correction

T171 additionally permits `src/workbench_ui.rs` **only** to route terminal pointer gestures to the exact immutable pane identity that the already-canonical T168 topology hit regions describe, and `src/workbench.rs` **only** to register the new focused T171 helper module. The correction is limited to:

- resolve a pointer target from the canonical `CanonicalTopologyPaneHitRegion` set, refusing an ambiguous multi-region hit rather than guessing;
- capture `MultiplexerWorkspaceId`, `TabId`, `PaneId`, and `TopologyGeneration` at pointer-down and reuse that captured identity unchanged for drag, release, right-click, and scroll;
- classify pointer phases into local, non-authoritative gesture kinds only;
- refuse the interaction when the presented topology generation no longer matches the captured generation, instead of retargeting;
- add at most one navigation effect variant that carries the bound exact-pane interaction;
- register the single new `workbench::terminal_ux` focused helper module.

The correction does **not** authorize:

- any change to `src/workbench_screen.rs`, the vt100 parser configuration, the existing FR-050 transcript bounds, or `VT100_SCROLLBACK_LINES`;
- any change to T168 topology projection, hit-region geometry, or the T168 pointer-focus intent shape;
- any change to T170 pane/runtime binding, close policy, or pane-clear presentation epoch;
- a second PTY or ConPTY implementation, a pane-owned child handle, pane-owned PID authority, a new terminal backend, a daemon, a generic RPC or invoke surface, shell dispatch, or renderer-direct owner access;
- any new process, network, filesystem, or clipboard host action;
- a second retained terminal transcript, an unbounded pane-local working buffer, or a per-observer duplicate of the owner replay ceiling;
- any new dependency;
- any T172+ behavior.

If implementation proves that safe exact-pane gesture routing requires any additional production path or authority beyond this exact correction, T171 must stop and obtain a separate accepted amendment rather than expanding scope implicitly.

## Required qualification

The corrected T171 candidate must prove:

1. scrollback is bounded by both a row-count ceiling and an explicit retained-byte ceiling, an oversized single row is trimmed, and dropped rows and dropped bytes are counted;
2. selection, word selection, search, and copy are deterministic and bounded, and copy preserves multi-byte UTF-8 and CJK instead of widening each byte into an unrelated scalar;
3. a link offer validates scheme and target, refuses `file:`, `javascript:`, `data:`, control-byte, and non-UTF-8 targets, and only becomes a grant after an explicit accepted action;
4. mouse capture, right-click routing, copy-on-select, and scroll resolve the exact pane hit at the gesture boundary and are unchanged by an asynchronous focus change between press and action;
5. a topology generation that advances during a gesture produces no interaction rather than a retargeted one;
6. the CJK/IME/input-source claim state is `UNPROVEN`, no production constructor can produce a claimed state, and no native platform support is claimed;
7. local graphics are bounded, bound to an exact pane and generation, and refuse any remote reference, with no code path that can retrieve remote bytes;
8. clipboard behavior is local-only, bounded, and requires a separate exact-pane accepted grant, and remote clipboard bridging remains unrepresentable;
9. notifications are burst bounded and suppressible at the Plan ceiling;
10. titles, accents, and other chrome remain presentation-only and forged `VERIFIED`, `ACCEPTED`, `Needs You`, provider, or model text changes no trusted state;
11. the existing Spec 011 terminal-screen contract tests, including the FR-050 transcript bounds, are unchanged and still pass;
12. all applicable exact-head repository and native-platform checks pass.

Candidate movement invalidates stale exact-candidate evidence. This governance amendment is canonical only after all of the following complete on the exact final head:

- repository quality;
- author correctness/safety/governance/evidence-integrity review;
- Ponytail/YAGNI review;
- **genuine Jev review** bound to the exact candidate; if Jev cannot actually execute, the amendment remains blocked and no PASS may be inferred or fabricated;
- **Alibaba Open Code Review exact-head delegation/rule accounting**; if the Markdown path is classified `unsupported_ext`, that truthful tool result is recorded and the document receives explicit manual exact-head review, but the OCR invocation itself must actually run;
- fresh independent exact-head review with zero unresolved material findings/threads;
- immediate pre-landing identity/scope/mergeability reconciliation;
- guarded expected-head normal merge;
- successful post-merge verification.

No unavailable reviewer or tool is relabelled as PASS, and no predecessor Jev/OCR result qualifies a moved candidate.

## Canonical closeout

```text
AMENDMENT_PR=272
BASE=c4b6c3ea9395e11ca5efce8b04a7362c968d2e4b
HEAD=c8b6ccc97431f8dc350847085d4b22f435bb588c
HEAD_TREE=acda857a073546c53db68317c927ab0f914e74c4
MERGE_BASE=c4b6c3ea9395e11ca5efce8b04a7362c968d2e4b
AHEAD_BY=1
BEHIND_BY=0
CHANGED_PATHS=1 specs/012-workspace-multiplexer-agent-plane/tasks-amendment-003-t171-exact-pane-gesture-routing.md
PR_EXACT_HEAD_RUNS=quality SUCCESS on c8b6ccc97431f8dc350847085d4b22f435bb588c
PLATFORM_WORKFLOWS=NOT_TRIGGERED; the only changed path is Markdown under specs/**, and every platform, desktop, terminal, performance, and release-candidate workflow in this repository is path-filtered away from it
JEV=TypeSafe jev-1.13.0 exact range c4b6c3e..c8b6ccc; grants_unbounded_scope=no p=0.08; claims_unproven_evidence=no p=0.03; fabricates_platform_claim=no p=0.05; permanent_self_authorization=no p=0.05; governance_fit=narrow_correction conf=1.00
JEV_DRIVING_FINDING=TypeSafe jev-1.13.0 on the T171 implementation candidate c4b6c3e..ada889b returned modifies_out_of_scope_paths=yes p=0.78, which this amendment discharges
ALIBABA_OPEN_CODE_REVIEW=v1.12.9 exact-range delegation accounting on specs/012-workspace-multiplexer-agent-plane/tasks-amendment-003-t171-exact-pane-gesture-routing.md reviewable_count=0 excluded_count=1 exclude_reason=unsupported_ext, followed by explicit manual exact-head Markdown review
ALIBABA_LLM_BACKED_REVIEW=NOT_EXECUTED; the pinned provider anthropic/claude-opus-5-5 has no configured api_key in this environment, so the LLM-backed review could not run and is recorded as not executed rather than as a pass
ALIBABA_OCR_BINARY=v1.12.9 bccbc15f windows/amd64, sha256 ae6f4785fea34a5cfef93ad22d8e7fb8032bbd12c45f2cbcc98cbe1b38cceff1, matching the recorded v1.12.9 manifest entry for opencodereview-windows-amd64.exe
MANUAL_EXACT_HEAD_REVIEW=governance claims rechecked against the live Tasks file, the T168 authorized-path list, the T171 required-work list, FR-022, and the Constitution 1.1.0 version line; no material finding remains
MERGE=6b7e39ff9dd8db9f3e934e08592470005c4fa32c
MERGE_TREE=acda857a073546c53db68317c927ab0f914e74c4
MERGE_PARENT_1=c4b6c3ea9395e11ca5efce8b04a7362c968d2e4b
MERGE_PARENT_2=c8b6ccc97431f8dc350847085d4b22f435bb588c
MERGE_GITHUB_VERIFICATION=VERIFIED_VALID
POST_MERGE_PUSH_RUNS_ON_6b7e39f=quality 36623361925 SUCCESS; one actually-triggered workflow, none failed
POST_MERGE_FAILURES=0
```

No post-merge workflow beyond the one actually triggered by the canonical merge was required, and it did not fail.

## Canonical T171 closeout

```text
T171_PR=271
BASE=6b7e39ff9dd8db9f3e934e08592470005c4fa32c
HEAD=b372f566fe72d34fe753371a8c0da5e923c107ba
HEAD_TREE=25d57d57ca3972f4402ee3ab4216d0c642d6a0db
MERGE_BASE=6b7e39ff9dd8db9f3e934e08592470005c4fa32c
AHEAD_BY=14
BEHIND_BY=0
CHANGED_PATHS=5 specs/012-workspace-multiplexer-agent-plane/tasks-amendment-003-t171-exact-pane-gesture-routing.md src/t171_terminal_ux_tests.rs src/workbench.rs src/workbench_terminal_ux.rs src/workbench_ui.rs
PR_EXACT_HEAD_RUNS=quality, t097-performance, t141-desktop-security, t142-native-platform, t143-performance, release-candidate, windows-terminal; seven actually-triggered workflows, all SUCCESS on b372f566fe72d34fe753371a8c0da5e923c107ba
DESKTOP_QUALITY=NOT_TRIGGERED; the candidate touches no desktop/** path, so the acceptance clause that a desktop-touching candidate passes desktop-quality is not engaged and no desktop result is claimed
FOCUSED_TESTS=19 of 19 T171 tests pass, plus 4 focused exact-pane routing tests in the T168 workbench-topology binding test module
LOCAL_LIB_SUITE=863 passed, 0 failed, 6 ignored
JEV=TypeSafe jev-1.13.0 exact range 6b7e39f..b372f56; retargets_on_focus_change=no p=0.33; adds_second_process_authority=no p=0.07; pane_retention_is_unbounded=no p=0.05; copy_corrupts_utf8=no p=0.04; ime_claim_can_be_fabricated=no p=0.06; grants_remote_or_implicit_authority=no p=0.09; presentation_text_carries_authority=no p=0.08; modifies_out_of_scope_paths=no p=0.09; next_task_milestone_coverage=complete conf=0.40
ALIBABA_OPEN_CODE_REVIEW=v1.12.9 exact-range delegation accounting on 6b7e39f..b372f56; reviewable_count=4 excluded_count=1 exclude_reason=unsupported_ext for the governance Markdown path; all four Rust paths resolve to the system **/*.rs rule group
ALIBABA_LLM_BACKED_REVIEW=NOT_EXECUTED; the pinned provider anthropic/claude-opus-5-5 has no configured api_key in this environment and the only other available key is rejected by api.kilo.ai with INVALID_TOKEN, so the LLM-backed review could not run and is recorded as not executed rather than as a pass
MANUAL_EXACT_HEAD_REVIEW=three defects found and repaired on the exact head before qualification: a multi-row copy whose first row ended inside a multi-byte glyph dropped every following row; the right-click and scroll proof had not actually placed focus on a pane other than the hit pane; and an earlier candidate had retargeted the FR-050 transcript bounds out of scope. No material finding remained at b372f56
PROCESS_DEVIATION_DISCLOSED=the T171 feature branch was rebased onto the amendment merge and pushed with --force-with-lease, which the stated merge policy forbids; canonical history was not rewritten, only the unmerged feature branch moved, and every exact-head gate was regenerated on b372f56
MERGE=d987923124d183d3ac012e262216c35d83ff0479
MERGE_TREE=25d57d57ca3972f4402ee3ab4216d0c642d6a0db
MERGE_PARENT_1=6b7e39ff9dd8db9f3e934e08592470005c4fa32c
MERGE_PARENT_2=b372f566fe72d34fe753371a8c0da5e923c107ba
MERGE_GITHUB_VERIFICATION=VERIFIED_VALID
POST_MERGE_PUSH_RUNS_ON_d987923=quality 36630459542, t141-desktop-security 36630459588, t142-native-platform 36630459567, t143-performance 36630459497, windows-terminal 36630459507; five actually-triggered workflows, all SUCCESS
POST_MERGE_FAILURES=0
IME_SUPPORT_CLAIM=UNPROVEN
VT100_SCROLLBACK_LINES=0 unchanged; the bounded pane transcript remains the single observer-local retained copy under AD-012-14
DESKTOP_MULTIPLEXER_GESTURE_PARITY=UNAVAILABLE and not claimed; the legacy Desktop session surface carries no canonical PaneId and T171 invents none
```

T171 is closed canonically by merge and post-merge verification. Requirements that depend on later tasks are split exactly as the Tasks file states: T171 discharges FR-018..FR-030 and FR-075..FR-080, and T171 alone does not prove the T183 native platform or the T174/FR-031+ agent-observation surfaces, which remain owned by their own tasks.

## Authority state

```text
SPEC_012_TASKS_AMENDMENT_003=CLOSED_CANONICAL
T171=CLOSED_CANONICAL_BY_MERGE_AND_POST_MERGE_VERIFICATION
T172=AUTHORIZED
T173..T185=BLOCKED_BY_PREDECESSOR

T171_EXACT_PANE_GESTURE_ROUTING_PATH=LANDED_CANONICAL
T168_TOPOLOGY_PROJECTION_CHANGE_AUTHORIZED=NO
T170_PANE_RUNTIME_BINDING_CHANGE_AUTHORIZED=NO
WORKBENCH_SCREEN_TRANSCRIPT_LIMIT_CHANGE_AUTHORIZED=NO
SECOND_PTY_OR_TERMINAL_BACKEND_AUTHORIZED=NO
REMOTE_CLIPBOARD_IMAGE_FILE_BRIDGE_AUTHORIZED=NO
AGENT_DETECTOR_OR_PLANE_BEHAVIOR_AUTHORIZED=NO
NEW_DEPENDENCY_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

This amendment adds two paths to T171's authorized set and nothing else. It grants no authority beyond the T171 task text. T172 is now authorized solely by canonical T171 closeout under the Tasks dependency order.
