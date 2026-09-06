# Tasks: Native Agentic Terminal UX Foundation

## Canonical Inputs

- Constitution 1.1.0: canonical.
- Spec 007 specification: canonical via PR #102.
- Spec 007 Plan: canonical via PR #103.
- Tasks base: `f6e1d1dbd437e704a72ffdff158c48b266cf0f58`.
- Canonical Plan tree: `4005e9b66f964b0c4befcdbee73e385d1daabd87`.

This file decomposes Spec 007 into independently reviewable, dependency-ordered implementation slices. It does not itself add a dependency, modify runtime behavior, start a terminal workbench, mutate Git state, open a browser/provider route, create a daemon/IPC path, activate learning, or authorize automatic landing.

At this Tasks base:

```text
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS_AUTHORIZED=YES
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
```

Canonical acceptance and post-merge verification of this file authorize **T087 only**. Every later task remains unauthorized until its exact predecessor closes canonically.

Spec 006 live-runtime nonclaims remain unchanged:

```text
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Nothing in Spec 007 upgrades those separate lanes.

## Global Rules

1. Execute tasks strictly in dependency order. Only the next dependency-satisfied task is authorized.
2. Every implementation task starts from exact then-canonical `main`; live repository/GitHub truth overrides handoffs, stale branches, stale CI, stale reviews, stale hashes, and cached authority.
3. Preserve all accepted Spec 003 terminal ownership/lifecycle semantics and Spec 006 identity/evidence/authority semantics. In particular:

```text
WORKBENCH_PRESENTATION != CANONICAL_AUTHORITY
PANE_ID != WORKSPACE_ID != WORKSTREAM_ID != WINDS_SESSION_ID != NATIVE_RUNTIME_ID
DISPLAY_NAME != IDENTITY
TERMINAL_OUTPUT != VERIFICATION_EVIDENCE
COMMAND_EXIT_STATUS != ACCEPTANCE
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
IDLE != DONE != VERIFIED != ACCEPTED
PANE_VISIBILITY != FILE_OR_EXECUTION_AUTHORITY
HYPERLINK_OR_ESCAPE_SEQUENCE != HOST_ACTION_AUTHORIZATION
UI_RESTART != PROOF_OF_LIVE_CHILD_OWNERSHIP
WORKTREE != SANDBOX
CHANGED_CANDIDATE_INVALIDATES_STALE_EVIDENCE_AND_REVIEW
NO_AUTOMATIC_WINNER
NO_SILENT_LANDING
VERIFY_THE_EXACT_CANDIDATE
CURRENT_ONE_PROCESS_ARCHITECTURE_PRESERVED
NO_DAEMON_OR_IPC_IN_SPEC_007
SHELL_COMPATIBILITY_BEFORE_DECORATION
```

4. No automatic winner selection, merge, rebase, cherry-pick, push, PR creation, force-clean, primary-checkout mutation, or autonomous landing behavior may be introduced.
5. No persistent service/daemon, socket/RPC/IPC layer, remote execution, browser runtime, provider/model gateway, generic Agent runtime, ACP/MCP runtime, plugin host/marketplace, verified-learning activation, vector/RAG memory, LSP/IDE replacement, custom PTY, custom full text editor, generalized workflow engine, or executable self-modification.
6. No new dependency unless the exact task explicitly authorizes that exact direct dependency and requires exact-version/resolved-graph/checksum/license/MSRV/platform/feature/YAGNI qualification. Dependency authority does not carry forward to unrelated crates.
7. No task may relax Spec 007 FR-045..FR-053 performance/boundedness thresholds. Relaxation requires a canonical Spec amendment before implementation continues.
8. Platform claims are limited to directly exercised domains. Native Windows != WSL2; Linux != macOS; compile success alone != runtime/platform proof.
9. Terminal bytes, escape sequences, parser callbacks, hyperlinks, clipboard requests, title/window requests, shell output, and Agent prose are untrusted presentation inputs unless separately elevated by existing canonical evidence paths. Recognition != authority.
10. **Focused-test registration rule**: whenever a task adds a new `src/tNNN_*.rs` test module or a new workbench module that must be reachable from the binary crate, that task also authorizes the smallest necessary `src/main.rs` edit solely for module declaration, focused-test registration, or the exact CLI entry explicitly authorized by that task. The acceptance gate MUST prove each task test module was compiled and executed; file existence alone is insufficient.
11. Prefer pure deterministic state tests before PTY/ConPTY integration. Do not start real terminal children where a pure fixture proves the requirement.
12. Workbench UI state and optional presentation persistence are non-authoritative. No new database table is authorized unless an exact later task names it and proves in-memory state insufficient. T087-T100 authorize no schema migration by default.
13. HEAD/TREE movement after CI, benchmark evidence, or review invalidates merge-ready state. Re-run every candidate-bound gate required by the task on the new exact candidate.
14. Historical evidence remains historical. A prior task's platform/benchmark/review result is not current proof for a later task unless the later task explicitly allows inherited evidence and the relevant candidate-bound claim has not changed.
15. Product dependency selection remains exactly bounded to the canonical Plan unless a canonical Plan/Spec amendment changes it. In particular, `tui-term`, Alacritty terminal runtime, Tokio/async, webview/desktop shells, automatic clipboard crates, fuzzy-search libraries, syntax/LSP/editor frameworks, provider/browser stacks, daemon/IPC, learning/vector memory, and generic plugin systems remain unselected.

## Standard Acceptance Gate

Every implementation task requires on its exact final candidate:

- repository `quality` = SUCCESS;
- focused deterministic tests for the changed surface, with evidence that every task-authorized `src/tNNN_*.rs` module was registered and actually executed;
- applicable existing Spec 003 and Spec 006 regression tests remain green;
- platform evidence only for platforms/domains claimed by that task;
- author correctness/safety/evidence-integrity review;
- Ponytail/YAGNI review;
- fresh independent substantive review on the exact candidate, or a review stack whose final delta reaches it;
- zero unresolved material findings and zero unresolved material review threads;
- exact changed-file/scope reconciliation;
- no unauthorized dependency/runtime/protocol/authority expansion;
- expected-head guarded merge;
- post-merge canonical main/tree verification before the next task starts;
- applicable push/post-merge CI checked before successor authority is asserted.

Documentation-only closeout tasks must satisfy the same review/race/landing discipline, while focused implementation tests are N/A only when the task changes no implementation surface and explicitly proves that fact.

## Authorization Ladder

Canonical acceptance of this file authorizes T087 only.

```text
T087 -> T088 -> T089 -> T090 -> T091 -> T092 -> T093
     -> T094 -> T095 -> T096 -> T097 -> T098 -> T099 -> T100
```

A task closes only after guarded landing and post-merge verification. Closing T100 authorizes no later Spec 007 implementation phase.

---

## Phase 1 — Minimal UI Dependency and Pure State Foundation

### [ ] T087 — Ratatui/Crossterm dependency qualification and inert workbench shell

**Purpose**: land only the host-TUI dependencies needed for the first inert workbench shell and prove their exact dependency/security/platform shape before any terminal parser/editor dependency is added.

**Authorized paths**:
- `Cargo.toml`
- `Cargo.lock`
- `src/workbench.rs`
- `src/t087_workbench_dependency_tests.rs`
- `src/main.rs` only under Global Rule 10, including the smallest explicit `workbench` CLI proof entry if needed
- a narrowly scoped dependency-evidence artifact under `specs/007-native-agentic-terminal-ux-foundation/evidence/` if required to record resolved graph/checksum/license/MSRV/features

**Exact direct dependency authority**:

```text
ratatui = "=0.30.2"
crossterm = "=0.29.0"
```

No other new direct dependency is authorized by T087. `vt100` and `ratatui-textarea` remain blocked for later tasks.

**Required dependency gate**:
- exact versions pinned;
- exact Cargo resolved graph captured and inspected;
- lockfile source identities/checksums recorded for new graph entries;
- direct/transitive licenses reviewed for compatibility with Winds `MIT OR Apache-2.0` distribution;
- crate MSRVs remain <= Winds Rust `1.97.1`;
- enabled feature graph is explicit and minimal;
- Crossterm `osc52` is NOT enabled;
- Crossterm async `event-stream` is NOT enabled;
- no Tokio/async executor, network/provider/browser client, clipboard automation path, daemon/service framework, or unrelated UI framework enters the graph;
- if Ratatui default features pull unjustified functionality, use a smaller explicit feature set instead;
- refuse landing if existing dependencies can satisfy the exact T087 shell without Ratatui/Crossterm.

**Implementation acceptance**:
- `src/workbench.rs` introduces only the minimal non-authoritative workbench shell/render seam required to prove the dependency path;
- deterministic rendering can be exercised without spawning a terminal child;
- no persistent event loop, PTY ownership, parser, editor, history persistence, Git mutation, or provider/model behavior;
- any CLI proof exits cleanly and restores host terminal state on normal/error paths it actually enters;
- focused T087 tests are registered and executed;
- Ubuntu + macOS compile/test evidence plus native-Windows dependency/build qualification before any universal platform claim.

**Depends on**: canonical Tasks. **Closes to authorize**: T088.

### [ ] T088 — Pure workbench topology, selection, and presentation identity

**Purpose**: implement deterministic in-memory pane topology without terminal process ownership.

**Authorized paths**:
- `src/workbench.rs`
- optional `src/workbench_state.rs` only if the pure state seam is clearer as a separate module
- `src/t088_workbench_topology_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- opaque transient `PaneId` distinct from every canonical/native ID;
- deterministic create/split/focus/resize/reorder/close state transitions;
- selected pane is explicit and at most one dispatch target can resolve;
- pane display title/order/layout changes never change canonical workspace/session association;
- stable optional canonical IDs remain stable through layout mutations;
- exited/stopped/ownership-lost/error presentation states are distinct from `LIVE`;
- restored/persisted-looking fixture metadata never establishes live ownership;
- deterministic fixtures include >=50 inert panes for topology operations without authorizing 50 live shells;
- Unicode/case/colliding display labels do not alter identity;
- no terminal child, parser, editor dependency, database schema, or provider/model work;
- focused T088 tests registered/executed.

**Depends on**: T087 `CLOSED_CANONICAL`. **Closes to authorize**: T089.

---

## Phase 2 — Terminal Screen Projection and Existing Lifecycle Integration

### [ ] T089 — vt100 screen projection, bounded transcript, and fail-closed callbacks

**Purpose**: parse observed terminal bytes into non-authoritative screen/transcript state without owning a terminal child or authorizing host actions.

**Authorized paths**:
- `Cargo.toml`
- `Cargo.lock`
- `src/workbench_screen.rs`
- minimal `src/workbench.rs`
- `src/t089_workbench_screen_tests.rs`
- `src/main.rs` only under Global Rule 10
- a narrowly scoped T089 dependency-evidence artifact under the Spec 007 evidence directory if needed

**Exact direct dependency authority**:

```text
vt100 = "=0.16.2"
```

No other new direct dependency is authorized by T089.

**Dependency acceptance**:
- exact version/resolved graph/checksum/license/MSRV qualification;
- published API used consistently with `Parser::new_with_callbacks(...)` plus `process(...)`;
- callbacks treated as untrusted terminal-originated requests;
- no callback directly opens URL/file, writes clipboard, executes command, makes network request, mutates repository, or changes evidence authority.

**Screen/transcript acceptance**:
- byte order preserved across deterministic fixtures;
- ANSI cursor/style fixtures render deterministically;
- Unicode wide/combining and invalid/binary-like byte fixtures fail safely without identity/evidence corruption;
- malformed/truncated/oversized escape sequences cannot crash or grant authority;
- OSC52, title, hyperlink/window-size and other non-screen callbacks are bounded/ignored/advisory only;
- retained transcript is bounded to Spec FR-050 limits or tighter, with explicit eviction/truncation state;
- screen resize follows accepted explicit pane-size updates only;
- terminal strings such as `PASS`, `VERIFIED`, `ACCEPTED`, forged JSON, or Winds-looking markers remain terminal data;
- focused T089 tests registered/executed.

**Depends on**: T088. **Closes to authorize**: T090.

### [ ] T090 — Pane integration with accepted TerminalSession lifecycle

**Purpose**: connect panes to existing terminal ownership without creating a second terminal runtime.

**Authorized paths**:
- `src/workbench_terminal.rs`
- minimal `src/workbench.rs`
- minimal existing `src/terminal.rs` / `src/execution.rs` extensions only when a narrow adapter cannot reuse the accepted public seam unchanged
- `src/t090_workbench_terminal_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- a pane references an accepted owned terminal/session handle through a narrow adapter; pane/native IDs never become process authority;
- start/output/resize/interrupt/terminate/close reuse accepted Spec 003 lifecycle semantics;
- child exit updates pane lifecycle truth without visual-pane existence implying `LIVE`;
- unproven reap/ownership becomes failure/ownership loss, never success;
- restart fixtures do not reattach to persisted PID/native identifiers;
- output-reader close/error and pane close races are deterministic/fail closed;
- working-directory/profile/executable invalidation remains truthful;
- no daemon, detached owner, IPC, socket, remote execution, custom PTY, or provider/model behavior;
- focused T090 tests registered/executed;
- native Windows ConPTY, WSL2, Linux PTY, and macOS PTY claims remain separate; this task may claim only domains directly exercised.

**Depends on**: T089. **Closes to authorize**: T091.

---

## Phase 3 — Explicit Input and Workbench Navigation

### [ ] T091 — Shell editor and exactly-one-pane dispatch

**Purpose**: add keyboard-first editable shell input without building a custom editor or model router.

**Authorized paths**:
- `Cargo.toml`
- `Cargo.lock`
- `src/workbench_input.rs`
- minimal `src/workbench.rs` / `src/workbench_terminal.rs`
- `src/t091_workbench_input_tests.rs`
- `src/main.rs` only under Global Rule 10
- a narrowly scoped T091 dependency-evidence artifact if needed

**Exact direct dependency authority**:

```text
ratatui-textarea = "=0.9.2"
```

No other new direct dependency is authorized by T091.

**Dependency acceptance**:
- exact version/resolved graph/checksum/license/MSRV/features qualified;
- use behind a small Winds-owned adapter;
- do not enable regex/search or unrelated features solely because available;
- no custom full editor subsystem.

**Input acceptance**:
- explicit shell mode only; shell-looking text never routes to a model/provider;
- input dispatch resolves exactly one selected live/owned pane at submit time;
- no implicit broadcast;
- focus changes before submit are visible and change the eventual target deterministically;
- child exit/ownership loss before submit rejects dispatch;
- Unicode, long input, edits, selection, undo/redo, multiline paste, bracketed-paste-sensitive fixtures and shell metacharacters preserve exact intended bytes/semantics;
- unsupported multiline/paste behavior requires explicit safe fallback, never silent line-by-line execution;
- no silent normalization of submitted shell content;
- focused T091 tests registered/executed.

**Depends on**: T090. **Closes to authorize**: T092.

### [ ] T092 — Host event loop, keyboard/pointer pane navigation, and canonical findability

**Purpose**: make the workbench usable while keeping host events and search non-authoritative.

**Authorized paths**:
- `src/workbench_ui.rs`
- minimal `src/workbench.rs` / `src/workbench_input.rs`
- existing read-only workspace/session selection seams only where required
- `src/t092_workbench_navigation_tests.rs`
- `src/main.rs` only under Global Rule 10, including the exact production `workbench` command surface if not already landed

**Acceptance**:
- synchronous/bounded host event loop; no Tokio/async runtime;
- event wait/poll blocks or sleeps when idle rather than busy-polling;
- keyboard-first create/focus/split/resize/close/navigation paths;
- pointer support, if included, never becomes the only route to required actions;
- deterministic search across current panes, canonical workspaces, and existing Winds sessions;
- exact normalized matches outrank weaker matches; ambiguity returns explicit candidates rather than recency guessing;
- Unicode/case/similar-name fixtures;
- display-name selection resolves to canonical IDs before context/execution use;
- semantic/embedding search remains unavailable rather than simulated;
- terminal resize/input events cannot alter canonical identity or evidence authority;
- focused T092 tests registered/executed.

**Depends on**: T091. **Closes to authorize**: T093.

---

## Phase 4 — Typed Interaction and Verification-Native Context

### [ ] T093 — Typed source labels, bounded terminal history, and transcript search

**Purpose**: distinguish user input, terminal output, Agent-reported material, accepted Winds evidence, warnings, and human decisions without manufacturing authority from presentation.

**Authorized paths**:
- `src/workbench_interaction.rs`
- minimal `src/workbench_screen.rs` / `src/workbench.rs`
- existing history/store read paths only if required without schema change
- `src/t093_workbench_interaction_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- closed typed source model separates USER_INPUT, TERMINAL_OUTPUT, AGENT_REPORTED, WINDS_OBSERVED/EVIDENCE, WARNING/POLICY, and HUMAN_DECISION equivalents;
- terminal bytes can never self-promote via text content or escape sequence;
- uncertain command/output boundaries fall back to truthful continuous/raw terminal presentation;
- bounded transcript search returns source/workspace/session/pane context where known;
- transcript retention/eviction never rewrites canonical evidence or implies complete history;
- secret-like fixtures do not create a new canonical credential/memory store;
- no new persistence schema; transcript persistence remains blocked unless a later accepted amendment/task explicitly authorizes it;
- focused T093 tests registered/executed.

**Depends on**: T092. **Closes to authorize**: T094.

### [ ] T094 — Read-only exact candidate, diff, evidence, and verification projection

**Purpose**: surface canonical context where work happens without creating a second verifier or Git authority.

**Authorized paths**:
- `src/workbench_context.rs`
- minimal existing read-only `src/git.rs`, `src/store.rs`, `src/domain.rs`, or check/evidence adapters only if required
- minimal `src/workbench.rs` / `src/workbench_ui.rs`
- `src/t094_workbench_context_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- exact candidate OID/tree and accepted evidence applicability are read from existing canonical observation/evidence paths;
- candidate movement makes earlier evidence/review visibly stale/not applicable without deleting historical records;
- terminal/Agent success strings never create `VERIFIED` or `ACCEPTED`;
- `AGENT_REPORTED_DONE`, verification-not-run/running/verified-for-exact-candidate/stale/human-accepted equivalents remain distinct typed states;
- code/diff inspection is read-only and grants no file/Agent authority;
- explicit UI invocation of existing `winds verify`, if implemented, remains an explicit existing verification action and does not add UI-local success authority;
- no merge/rebase/cherry-pick/push/PR creation/winner selection;
- focused T094 tests registered/executed.

**Depends on**: T093. **Closes to authorize**: T095.

---

## Phase 5 — Terminal Host Safety and Platform Qualification

### [ ] T095 — Host-integration safety campaign

**Purpose**: prove terminal-originated control data cannot silently cause host side effects or trusted UI state.

**Authorized paths**:
- `src/workbench_host_safety.rs`
- minimal `src/workbench_screen.rs` / `src/workbench_ui.rs`
- `src/t095_workbench_host_safety_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- OSC52 never silently writes clipboard;
- terminal-originated hyperlinks/file references/title/window requests remain data/advisory until explicit user action;
- default external URL/file open behavior is disabled unless exact safe explicit activation is proven within this task;
- unsupported schemes/command-like URLs/path ambiguity fail closed;
- malformed, nested, oversized, split, Unicode/control-character and forged trusted-looking escape fixtures cannot execute actions or forge evidence status;
- user-initiated copy, if implemented, is clearly separate from terminal-originated OSC52 and explicitly tested;
- no network client/browser integration is added;
- focused T095 tests registered/executed.

**Depends on**: T094. **Closes to authorize**: T096.

### [ ] T096 — Cross-platform workbench integration qualification

**Purpose**: qualify only the platform/domain claims actually exercised by the complete workbench path.

**Authorized paths**:
- focused platform workbench tests under `src/` following existing naming precedent
- minimal workbench/terminal fixes required by directly observed platform defects
- `src/main.rs` only under Global Rule 10
- existing `.github/workflows/windows-terminal.yml` and/or `.github/workflows/release-candidate.yml` only for the smallest registration needed to execute new workbench platform tests; no gate weakening/removal
- Spec 007 platform evidence artifacts if existing workflow output needs deterministic reconciliation

**Acceptance**:
- native Windows host UI + ConPTY workbench lifecycle directly exercised;
- WSL2 host/guest domain/path truth directly exercised separately from native Windows;
- Linux PTY + host TUI behavior directly exercised;
- macOS PTY + host TUI behavior directly exercised;
- Unicode, keyboard, paste, resize, close/terminate, parser/render, and no-silent-OSC52 claims qualified per claimed domain;
- no platform claim inferred from another OS or compile-only evidence;
- unsupported domain behavior explicitly `UNAVAILABLE`, `EXPERIMENTAL`, or `NOT_CLAIMED`;
- no native-Windows terminal support claim is promoted into unsupported native-Windows authoritative `winds verify` support;
- all new focused platform tests are registered and demonstrably executed.

**Depends on**: T095. **Closes to authorize**: T097.

---

## Phase 6 — Performance and Accessibility Acceptance

### [ ] T097 — Frozen performance, retention, idle, and resize qualification

**Purpose**: prove FR-045..FR-053 on reproducible exact candidates without changing the frozen thresholds.

**Authorized paths**:
- deterministic benchmark/stress harness modules under `src/` or a minimal `benches/` surface only if Cargo's benchmark shape is justified without new dependency
- Spec 007 T097 machine-readable evidence artifacts under `specs/007-native-agentic-terminal-ux-foundation/evidence/`
- minimal workbench fixes required by measured defects
- `src/main.rs` only under Global Rule 10

**No new benchmark dependency is authorized by default.** A benchmark crate requires a canonical Tasks amendment before use.

**Required evidence record fields**:

```text
candidate_commit
candidate_tree
os
os_version_or_image
arch
rust_version
build_profile
cpu_description_or_runner_class
logical_cpu_count
memory
fixture_shell
fixture_id
measurement_command
warmup_policy
sample_count
raw_samples_or_machine_readable_summary
p50
p95
max
```

**Acceptance**:
- release-like build profile;
- cold/input-ready startup <=1500 ms p95 over >=20 runs;
- workbench-only input-to-dispatch overhead <=16 ms p95 over >=1000 iterations;
- 50 inert-pane topology operation p95 <=16 ms over >=1000 operations;
- >=100,000-line retained-history search p95 <=100 ms over >=200 representative searches;
- >=100,000 lines and >=10 MiB deterministic output campaign with navigation p95 <=100 ms for the specified navigation measurement;
- per-pane retained transcript <=100,000 logical lines and <=32 MiB payload or tighter, with visible eviction state;
- ten idle live fixture panes over 60 seconds <=2% of one logical CPU core and <=256 MiB workbench RSS overhead excluding child memory;
- 1000 resize requests across ten fixture panes in ten seconds preserve final-size correctness;
- exact evidence identity/staleness remains correct after candidate movement;
- no provider/network/model call in benchmark fixtures;
- failures trigger repair/requalification, not threshold relaxation.

**Depends on**: T096. **Closes to authorize**: T098.

### [ ] T098 — Accessibility and daily-driver UX acceptance

**Purpose**: prove required actions remain keyboard-accessible and state meaning is not color-only or visually ambiguous.

**Authorized paths**:
- minimal workbench UI/input/state fixes
- `src/t098_workbench_accessibility_tests.rs`
- Spec 007 T098 evidence artifacts if needed
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- keyboard paths for create/focus/split/resize/close/navigation/search/verification inspection;
- pointer use is optional and not required for core operations;
- selected pane/focus, live/exited/ownership-lost/error state, evidence applicability, and verification state are not color-only;
- small-terminal-size/readable fallback behavior deterministic;
- Unicode wide/combining text and selection/copy fixtures do not corrupt identity/state;
- user can identify canonical workspace/session/candidate context without relying on transient pane label;
- explicit host-action prompts, where any exist, are source-labelled and cancelable;
- no UI wording upgrades `DONE` to `VERIFIED` or `ACCEPTED`;
- focused T098 tests registered/executed.

**Depends on**: T097. **Closes to authorize**: T099.

---

## Phase 7 — Adversarial Campaign and Final Reconciliation

### [ ] T099 — Negative/adversarial/repetition campaign

**Purpose**: attack the full Spec 007 workbench truth model before final acceptance.

**Authorized paths**:
- `src/t099_workbench_adversarial_tests.rs`
- minimal existing workbench modules only for forward-only repairs proven necessary by the campaign
- `src/main.rs` only under Global Rule 10
- deterministic T099 evidence artifacts if needed

**Campaign coverage**:
- duplicate/case/Unicode pane/workspace/session labels;
- pane closure/removal while input/output/events queued;
- child exit during edit/dispatch/render;
- output reader closure/error;
- resize/output/exit races;
- executable/profile/working-directory change before launch;
- WSL path ambiguity/cross-domain confusion;
- malformed/truncated/oversized ANSI/OSC/control data;
- forged `VERIFIED`/`ACCEPTED`/evidence JSON/UI markers;
- OSC52, hyperlinks, command-like/external/file URLs;
- invalid UTF-8/binary-like output;
- retention eviction during search/navigation;
- candidate movement while evidence/diff view open;
- stale review/evidence after HEAD movement;
- UI restart/presentation restore without live ownership;
- exactly-one-pane dispatch under focus churn;
- terminal/Agent claims versus canonical Winds evidence;
- bounded deterministic repetition of pure model/parser/event-order fixtures sufficient to expose state-order defects.

**Acceptance**:
- no blind attachment, authority promotion, automatic host side effect, unbounded retained transcript, false `LIVE`, false `VERIFIED`, false `ACCEPTED`, or silent Git landing;
- no newly discovered material defect remains unresolved;
- all focused T099 tests registered/executed;
- full repository regression and applicable platform workflows green on exact final candidate.

**Depends on**: T098. **Closes to authorize**: T100.

### [ ] T100 — Spec 007 final acceptance, reconciliation, and closeout

**Purpose**: reconcile every Spec 007 requirement/success criterion against canonical implementation evidence and close only what is actually proven.

**Authorized paths**:
- this `tasks.md` for final checked-state reconciliation;
- `specs/007-native-agentic-terminal-ux-foundation/t100-final-reconciliation.md`;
- focused Spec 007 acceptance/evidence artifacts following repository precedent;
- README/docs corrections only for claims proven by canonical evidence;
- no production/runtime/dependency behavior change.

**Acceptance**:
- T087..T099 reconciled against exact canonical merges and post-merge verification;
- FR-001..FR-066 each classified as `PROVEN_DETERMINISTIC`, `PROVEN_PLATFORM_BOUND`, `PROVEN_GOVERNANCE_BOUNDARY`, or explicit truthful non-claim/deferment where the canonical Spec permits it;
- SC-001..SC-018 each reconciled to exact evidence;
- FR-045..FR-053 frozen thresholds proven without relaxation or the program remains open;
- all dependency versions/features/resolved-graph evidence reconciled; no unauthorized dependency remains;
- platform claims limited to directly exercised native Windows, WSL2, Linux, and macOS domains;
- existing Spec 006 live-runtime nonclaims remain separate and unchanged unless independently governed elsewhere;
- no older-head CI/review/benchmark evidence represented as current;
- final exact implementation state has correctness/safety review, Ponytail/YAGNI review, and fresh independent substantive review with zero unresolved material findings;
- final deterministic gates and applicable platform/security workflows green;
- no daemon/IPC, remote execution, browser/provider runtime, ACP/MCP runtime, generic plugin system, learning subsystem, vector/RAG memory, custom PTY, automatic winner, or automatic landing path introduced;
- exact main/base/head/tree/scope/ruleset reconciliation before guarded landing;
- guarded explicit final landing;
- post-merge canonical main/tree and push checks verified.

**Completion state, only if proven**:

```text
T087..T100=CLOSED_CANONICAL
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS=CLOSED_CANONICAL
SPEC_007_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

Closing T100 authorizes no later Spec 007 phase and does not automatically authorize Spec 008, verified learning, durable runtime/daemon/IPC, remote execution, browser/provider orchestration, ACP/MCP, plugins, or any later research roadmap item.

**Depends on**: T099 `CLOSED_CANONICAL`. **Closes to authorize**: no successor implementation task.

## Tasks Acceptance Gate

This Tasks file may land only if all are true on its exact final candidate:

- canonical base is post-Plan `main` `f6e1d1dbd437e704a72ffdff158c48b266cf0f58` unless live truth moves before candidate creation, in which case the file/base metadata must be reconciled forward-only;
- changed scope is exactly this Tasks document unless an explicitly justified governance-only correction is needed;
- dependency authority is task-local and minimized (`ratatui`/`crossterm` T087, `vt100` T089, `ratatui-textarea` T091);
- no implementation, Cargo, source, workflow, migration, runtime, provider/browser, daemon/IPC, remote, learning, plugin, ACP/MCP, or automatic landing change occurs in the Tasks PR;
- every FR-001..FR-066 and SC-001..SC-018 has a plausible dependency-ordered implementation/evidence path;
- frozen FR-045..FR-053 thresholds are unchanged;
- canonical acceptance authorizes T087 only;
- exact-head repository `quality` succeeds;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- final exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- expected-head guarded merge;
- post-merge canonical main/tree and push checks verified.

Only after this Tasks file lands canonically and is post-merge verified may repository truth state:

```text
SPEC_007_TASKS=CLOSED_CANONICAL
SPEC_007_IMPLEMENTATION_AUTHORIZED=T087_ONLY
T087=AUTHORIZED
T088..T100=BLOCKED_BY_DEPENDENCY
```
