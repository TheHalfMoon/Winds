# Spec 012 Tasks Amendment 003 — T171 Exact-Pane Terminal Gesture Routing

Status: PROPOSED

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

## Authority state

```text
SPEC_012_TASKS_AMENDMENT_003=PROPOSED
T171=BLOCKED_PENDING_THIS_AMENDMENT
T172..T185=BLOCKED_BY_PREDECESSOR

T171_EXACT_PANE_GESTURE_ROUTING_PATH=ADDED_TO_T171_AUTHORIZED_PATHS
T168_TOPOLOGY_PROJECTION_CHANGE_AUTHORIZED=NO
T170_PANE_RUNTIME_BINDING_CHANGE_AUTHORIZED=NO
WORKBENCH_SCREEN_TRANSCRIPT_LIMIT_CHANGE_AUTHORIZED=NO
SECOND_PTY_OR_TERMINAL_BACKEND_AUTHORIZED=NO
REMOTE_CLIPBOARD_IMAGE_FILE_BRIDGE_AUTHORIZED=NO
AGENT_DETECTOR_OR_PLANE_BEHAVIOR_AUTHORIZED=NO
NEW_DEPENDENCY_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

This amendment adds two paths to T171's authorized set and nothing else. It grants no authority beyond the T171 task text, and T172 remains blocked until T171 is closed canonically.
