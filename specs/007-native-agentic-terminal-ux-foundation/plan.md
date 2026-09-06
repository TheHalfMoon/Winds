# Implementation Plan: Native Agentic Terminal UX Foundation

## Summary

Build the smallest one-process terminal workbench that satisfies Spec 007 without replacing the accepted Spec 003 terminal lifecycle, weakening Spec 006 identity/evidence authority, or prematurely introducing durable runtime, browser, provider, plugin, or learning architecture.

The first implementation program will:

1. reuse the existing `TerminalSession` / `TerminalExecution` lifecycle and existing native Windows, WSL2, Linux, and macOS terminal paths;
2. add a pure workbench topology/state model whose pane identity is presentation-only;
3. add a bounded terminal-screen projection and transcript-retention layer over observed terminal bytes;
4. add a keyboard-first shell editor and explicit exactly-one-pane dispatch path;
5. add deterministic navigation/search over panes, canonical workspaces, and existing Winds sessions;
6. surface exact-candidate Git/evidence/verification state read-only, preserving existing verification authority;
7. add fail-closed terminal host-integration handling for clipboard, hyperlinks, unsupported escape sequences, and forged trusted-looking output;
8. add deterministic benchmark/stress harnesses for every frozen Spec 007 performance budget;
9. prove platform and accessibility claims only where directly exercised;
10. preserve one process and explicit human landing throughout.

This Plan selects a small Rust terminal-UI dependency set for later Task-stage addition, but this Plan PR itself adds no dependency, lockfile, migration, production source, workflow semantic, or runtime behavior.

## Constitution Check

The implementation program MUST preserve:

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

Additional constitutional constraints:

- terminal/process ownership remains the accepted retained-child/session authority, never a pane/widget/PID guess;
- repository-native Git/evidence observations remain the verification authority;
- terminal and Agent prose remain source-labelled observations, never self-certified evidence;
- no automatic merge, rebase, cherry-pick, push, PR creation, winner selection, or primary-checkout mutation;
- no persistent service, daemon, socket, local control server, IPC protocol, remote execution route, mobile continuation surface, browser runtime, provider mesh, generic plugin host, ACP/MCP runtime, verified-learning activation, vector/RAG memory, or executable self-modification;
- each implementation slice must start from exact then-current canonical `main`, remain dependency-ordered, pass deterministic gates, receive author correctness/safety review, Ponytail/YAGNI review, and fresh independent substantive review before guarded landing;
- HEAD/TREE movement invalidates prior exact-candidate qualification.

## Canonical Baseline

Planning base:

```text
785a368d07ba0b31a6f06847865cd3c69094777f
```

At this base:

```text
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN_AUTHORIZED=YES
SPEC_007_TASKS_AUTHORIZED=NO
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
```

Spec 006 live-runtime nonclaims remain unchanged:

```text
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Current repository facts to reuse rather than duplicate:

- one Rust 2024 package and one `winds` binary;
- Rust version floor `1.97.1`;
- existing dependencies include `libc`, `portable-pty`, bundled `rusqlite`, `serde`, `serde_json`, and `sha2`;
- `src/terminal.rs` owns accepted live terminal/session lifecycle semantics;
- `src/execution.rs` binds terminal activity to existing execution/history semantics;
- native Windows terminal tests exercise ConPTY behavior;
- WSL launch/path-domain code already distinguishes Windows-host and Linux-guest truth;
- existing Store, workspace, session, Git observation, candidate verification, evidence, and history surfaces are canonical starting points;
- Spec 003 and Spec 006 regression suites remain applicable and must stay green.

## Architecture Decision: One Process, Four New Seams

Do not create a second terminal runtime. The workbench sits above existing lifecycle code through four narrow seams.

### 1. Pure workbench topology/state

Add a deterministic in-memory workbench model responsible only for presentation topology and selection.

Candidate concepts:

```text
WorkbenchState
PaneId
PaneState
PaneLifecycleView
SplitAxis
WorkbenchLayout
WorkbenchSelection
PaneTarget
```

Rules:

- `PaneId` is opaque and transient for the current workbench lifetime;
- pane order/layout/focus/title are never canonical workspace/session identity;
- pane association stores stable canonical IDs only when an existing canonical relation is known;
- a pane may remain visible after child exit, but its lifecycle view must be `EXITED`/`STOPPED`/`OWNERSHIP_LOST` as observed rather than `LIVE`;
- restored presentation metadata never recreates live process ownership;
- topology operations are pure enough to benchmark and fuzz without spawning shells.

### 2. Bounded terminal-screen projection

Existing `TerminalSession` continues to own PTY/ConPTY I/O and lifecycle. A per-pane screen projection consumes the observed byte stream and produces renderable terminal cells plus bounded transcript data.

Selected parser candidate for Task-stage dependency addition: `vt100 = "=0.16.2"`.

Why direct `vt100` rather than a full terminal-widget wrapper:

- it parses terminal bytes into an in-memory screen without owning process lifecycle;
- it exposes a callback seam for non-screen escape sequences;
- Winds can decide host-integration policy explicitly instead of inheriting a widget's implicit behavior;
- it keeps accepted `portable-pty` lifecycle untouched;
- it avoids adopting a separate terminal-controller abstraction whose ownership semantics could conflict with Winds.

The screen projection MUST:

- preserve byte processing order;
- resize only from accepted pane-size changes and retain final accepted dimensions;
- bound scrollback/transcript retention according to FR-050;
- expose explicit eviction/truncation state;
- never convert parsed text into canonical evidence;
- treat parser callbacks as untrusted terminal-originated requests, not commands.

### 3. Workbench UI/event loop

Selected Task-stage UI candidates:

```text
ratatui = "=0.30.2"
crossterm = "=0.29.0"
ratatui-textarea = "=0.9.2"
```

The workbench event loop should remain synchronous/small unless profiling proves a more complex runtime is necessary. No Tokio/async runtime is justified by Spec 007.

Responsibilities:

- enter/restore terminal UI mode safely;
- read keyboard/pointer/resize/paste events;
- drain bounded terminal-output channels;
- update pure workbench state;
- render only from current state;
- dispatch shell input only to one explicit selected pane;
- expose keyboard-first navigation/search/verification-inspection actions;
- sleep/block when idle rather than busy-polling.

The event loop MUST NOT become process authority. It requests lifecycle actions through existing accepted terminal/session APIs.

### 4. Read-only verification/context adapter

Add a read-only adapter over existing workspace/session/Git/evidence/store surfaces for workbench presentation.

The adapter may expose:

- canonical workspace ID/path/display context;
- stable Winds session/workstream identity where already present;
- exact current candidate OID/tree observations;
- accepted evidence/report/run references;
- stale/not-applicable state after candidate movement;
- explicit `winds verify` invocation as an existing authorized verification action when a later Task permits that UI command.

The adapter MUST NOT:

- fabricate evidence locally;
- promote terminal/Agent output;
- mutate files through code/diff inspection;
- merge/push/land candidates;
- infer current evidence applicability from display text.

## Dependency Decision Record

No dependency changes occur in this Plan PR. Tasks may add the selected crates only after this Plan lands canonically and Tasks specifically authorize the dependency slice.

All selected crates must be pinned exactly in `Cargo.toml`/lockfile by the implementation task that introduces them, followed by exact resolved-graph/checksum/license/MSRV/platform inspection.

### Ratatui 0.30.2

Purpose: layout, widgets, rendering abstraction, terminal UI composition.

Evidence revalidated 2026-09-06:

- version: `0.30.2`;
- license: MIT;
- Rust MSRV: `1.88.0`, below Winds `1.97.1`;
- default backend path includes Crossterm support;
- no daemon/network/runtime dependency is required.

Primary evidence:

- https://docs.rs/crate/ratatui/0.30.2
- https://docs.rs/crate/ratatui/0.30.2/source/Cargo.toml

Plan decision:

- select Ratatui for layout/rendering;
- do not enable experimental/unstable features without a later exact Task amendment;
- prefer the minimum feature set that supports the chosen Crossterm backend and required widgets; dependency Task must compare default features versus explicit minimal features before landing.

### Crossterm 0.29.0

Purpose: cross-platform terminal host mode, input/events, resize, paste, keyboard and pointer event acquisition.

Evidence revalidated 2026-09-06:

- version: `0.29.0`;
- license: MIT;
- Rust MSRV: `1.63.0`, below Winds `1.97.1`;
- default features include bracketed-paste, events, windows, and derive-more;
- OSC52 clipboard support is a separate optional `osc52` feature.

Primary evidence:

- https://docs.rs/crate/crossterm/0.29.0
- https://docs.rs/crate/crossterm/0.29.0/source/Cargo.toml.orig

Plan decision:

- select Crossterm `0.29.0` as the host-terminal backend;
- DO NOT enable the `osc52` feature in Spec 007;
- clipboard writes from terminal output remain disabled or separately explicit-consent gated through Winds logic;
- do not add the async `event-stream` feature unless measured evidence proves the blocking/event-poll path cannot satisfy the frozen budgets.

### ratatui-textarea 0.9.2

Purpose: multiline keyboard-first input editing without creating a custom editor subsystem.

Evidence revalidated 2026-09-06:

- version: `0.9.2`;
- license: MIT;
- maintained under the Ratatui project;
- supports multiline editing, Unicode-aware wrapping behavior, selection, undo/redo, and Crossterm-backed input conversion;
- the crate is pre-1.0, so exact pinning and focused adapter tests are required.

Primary evidence:

- https://docs.rs/crate/ratatui-textarea/0.9.2
- https://github.com/ratatui/ratatui-textarea

Plan decision:

- use it behind a small Winds-owned shell-editor adapter;
- do not enable regex search solely for editor use unless a Task proves the need;
- keep dispatch semantics in Winds, not inside the editor widget;
- normalize no submitted shell content silently.

### vt100 0.16.2

Purpose: deterministic terminal-byte parsing into a screen representation.

Evidence revalidated 2026-09-06:

- version: `0.16.2`;
- license: MIT;
- Rust MSRV: `1.70`;
- dependencies are small (`itoa`, `unicode-width`, `vte`);
- `Parser::process_cb` exposes callbacks for non-screen escape sequences including OSC52/window-size related requests.

Primary evidence:

- https://docs.rs/vt100/0.16.2/vt100/
- https://docs.rs/crate/vt100/0.16.2/source/Cargo.toml.orig
- https://github.com/doy/vt100-rust/blob/main/CHANGELOG.md

Plan decision:

- use the parser directly rather than `tui-term` or another terminal-controller wrapper;
- implement a Winds-owned callback policy that records/ignores/gates non-screen host requests according to Spec 007;
- terminal-originated callbacks never acquire authority merely because the parser recognized them.

### Explicit dependency non-selections

Do not select in Spec 007 unless a future accepted amendment proves necessity:

- `tui-term`: useful reference, but its controller seam is unnecessary when Winds already owns terminal lifecycle; direct parser + Winds renderer is smaller and safer;
- Tokio/async executor: no demonstrated need for the first one-process workbench;
- webview/desktop-shell frameworks: outside the first terminal-native foundation;
- clipboard crates for automatic terminal-driven writes: not needed for the fail-closed first slice;
- fuzzy-search libraries: deterministic matching can begin with existing/simple bounded algorithms; add only if Tasks demonstrate measured need;
- syntax/LSP/editor frameworks: outside this foundation;
- tracing/telemetry frameworks solely for benchmarks: use deterministic local measurement artifacts first.

## Host-Integration Safety Architecture

### Escape/control sequences

Terminal output is untrusted presentation input.

For every parsed non-screen action:

```text
TERMINAL_BYTES
  -> PARSER
  -> WINDS_CALLBACK_POLICY
  -> IGNORE | RENDER_ONLY | EXPLICIT_USER_ACTION_REQUIRED
```

No callback may directly execute a command, open a browser/file, write the clipboard, make a network request, mutate repository state, or change evidence authority.

### OSC 52 / clipboard

Initial Spec 007 implementation posture:

- Crossterm OSC52 feature disabled;
- terminal-originated OSC52 never silently writes host clipboard;
- parser callback may record a bounded advisory event for UI indication if required by a later Task;
- user-initiated copy from visibly selected workbench text is a separate UI action and must be tested independently.

### Hyperlinks and file references

Terminal-originated links are data until explicit user activation.

Activation path must validate:

- scheme;
- local versus external target;
- canonical/path context when applicable;
- existence/availability where relevant;
- policy/authority boundary;
- whether the action is inspection-only or has side effects.

The initial implementation SHOULD support inspection/copy before adding external-open behavior. External/open integration may be deferred if it cannot be proven safely inside Spec 007.

### Forged Winds-looking output

Trusted status/evidence elements must render from typed internal state, not by parsing text labels from terminal output. Terminal output can display any characters, including `VERIFIED`, but cannot occupy the trusted source/status channel.

## Workbench State Model

The Plan expects a closed, explicit state model rather than a generic widget graph.

Illustrative shape:

```text
WorkbenchState
- panes: ordered bounded collection<PaneId, PaneState>
- layout: WorkbenchLayout
- selected_pane: Option<PaneId>
- input: ShellEditorState
- navigation: NavigationState
- evidence_view: EvidenceViewState
- notice: bounded status/warning queue

PaneState
- pane_id: PaneId
- terminal_handle_key: internal reference to accepted owned session object
- canonical_workspace_id: optional stable ID
- canonical_winds_session_id: optional stable ID
- display_title: presentation-only
- lifecycle_view: LIVE | EXITED | STOPPED | OWNERSHIP_LOST | ERROR
- parser/screen state
- bounded transcript/search state
- eviction marker
```

The exact Rust shape belongs to Tasks. The invariants above do not.

## Output and Backpressure Model

Do not let renderer speed define terminal-process truth.

Use bounded per-pane ingestion and transcript state with explicit overload behavior. Tasks must choose concrete capacities and prove them against FR-049/FR-050.

Required behavior:

- terminal-reader work may continue independently of frame rendering;
- parsed screen state and bounded retained transcript are updated in byte order;
- if a bounded internal channel reaches capacity, the design must apply a proven backpressure/coalescing strategy that does not silently reorder bytes or claim complete retained history;
- viewport redraws may be coalesced; terminal byte semantics may not;
- rendering can skip intermediate frames while lifecycle/output observations remain ordered;
- eviction is visible and cannot alter canonical evidence.

## Input and Dispatch Model

Shell editing and shell dispatch are distinct.

The editor adapter owns only editable user text and cursor/selection state. On explicit submit:

1. capture the exact selected `PaneId` and current editable content;
2. resolve the selected pane to its accepted live terminal handle;
3. reject if the pane is no longer live/owned;
4. apply explicit shell/paste dispatch semantics;
5. send bytes only to that one terminal path;
6. record history through existing accepted mechanisms where applicable;
7. never route to a provider/model in Spec 007.

Pane focus changes before submission must affect the target visibly; no implicit broadcast exists.

## Navigation and Search Model

Use deterministic, bounded search with canonical disambiguation.

Search domains in Spec 007:

- current workbench panes;
- canonical workspaces already known to Winds;
- existing Winds sessions/workstreams;
- bounded retained terminal transcript;
- exact candidate/evidence references where already indexed by existing Store surfaces.

Rules:

- display label is not identity;
- exact normalized match outranks prefix/containment when such ranking is used;
- material ambiguity returns explicit candidates;
- Unicode/case/similar-name fixtures are mandatory;
- semantic/embedding search remains unavailable, not faked;
- transcript search never becomes canonical memory.

## Verification-Native UI Model

Define distinct typed presentation states:

```text
AGENT_REPORTED_DONE
VERIFICATION_NOT_RUN
VERIFICATION_RUNNING
VERIFIED_FOR_EXACT_CANDIDATE
EVIDENCE_STALE_FOR_CURRENT_CANDIDATE
HUMAN_ACCEPTED
```

Exact naming belongs to Tasks, but the authority separation does not.

Candidate movement must cause a recomputation of applicability from canonical Git/evidence observations. The UI cannot carry a green status forward merely because the same pane/session remains visible.

## Presentation Persistence

Spec 007 does not require persistent workbench layout. If Tasks later authorize presentation persistence because it materially improves usability, it must be strictly non-authoritative.

Persistable examples may include:

- split ratios;
- selected workspace/session reference;
- pane display labels;
- bounded history preference;
- user UI preferences.

Never persist a claim that a prior terminal child is currently live/owned. On process restart, every live ownership relation begins from actual newly observed/created runtime truth.

Do not add a new database table until an exact Task proves a persisted field is necessary. Prefer in-memory first implementation.

## Performance Qualification Plan

All frozen Spec budgets are acceptance gates, not aspirations.

### Reference environment

Tasks must pin reproducible reference environments before a performance claim can close. The first reference set should use GitHub-hosted or explicitly documented reproducible machines where the metric is meaningful, plus native-platform evidence for platform-specific behavior.

Record at minimum:

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
raw_samples_or_machine-readable_summary
p50
p95
max
```

### Build profile

Performance acceptance must use an explicit release-like build profile. Debug-build timings cannot qualify frozen latency budgets.

### Determinism

Benchmark fixtures must avoid provider/network/model calls. Shell fixture behavior should be local and deterministic. Raw results belong in repository evidence artifacts only when an authorized Task defines the exact path/format.

### Budget mapping

- FR-045: cold/input-ready startup campaign, >=20 runs;
- FR-046: workbench-only dispatch overhead, >=1000 iterations;
- FR-047: topology model, 50 inert panes, >=1000 operations;
- FR-048: >=100,000 logical line retained-history corpus, >=200 representative searches;
- FR-049: >=100,000 lines and >=10 MiB deterministic terminal payload plus navigation latency;
- FR-050: enforced line/payload retention bounds plus explicit eviction state;
- FR-051: ten idle live fixture panes, 60 seconds, CPU/RSS measurement excluding child memory;
- FR-052: 1000 resize requests across ten fixture panes in ten seconds with final-size correctness;
- FR-053: exact evidence identity and no reuse after candidate movement.

A Task may tighten thresholds but may not relax them without a canonical Spec amendment.

## Cross-Platform Qualification

### Native Windows

Directly exercise:

- Crossterm host input/render lifecycle;
- existing ConPTY child ownership;
- pane resize/interrupt/terminate/close integration;
- Unicode/paste/keyboard behavior;
- terminal parser/render path;
- no OSC52 silent clipboard write;
- performance/correctness claims made for native Windows.

Native Windows evidence does not prove WSL2.

### WSL2

Keep Windows host UI and Linux guest terminal/path truth explicit. Reuse existing WSL discovery/launch/path-domain logic. No path conversion may be invented in workbench presentation.

### Linux

Exercise Unix PTY lifecycle plus host TUI behavior directly.

### macOS

Exercise Unix PTY lifecycle plus host TUI behavior directly; do not infer from Linux.

### Claim discipline

A feature not directly exercised on a domain is `UNAVAILABLE`, `EXPERIMENTAL`, or `NOT_CLAIMED` as appropriate. Platform parity is never inferred from compiling alone.

## Accessibility Plan

Before Spec 007 closeout, Tasks must prove:

- keyboard paths for create/focus/split/resize/close/navigation/search/verification inspection;
- focus and selected pane not color-only;
- live/exited/ownership-lost state not color-only;
- evidence applicability and verification state not color-only;
- text selection/copy fixtures;
- Unicode wide/combining-character behavior;
- terminal-size constraints and readable fallback behavior.

The chosen TUI stack does not itself prove accessibility; Winds behavior must be tested.

## Testing Strategy

### Pure model tests first

Test without PTY/processes wherever possible:

- pane identity/topology mutations;
- selection and exactly-one-pane dispatch resolution;
- lifecycle-view projection;
- canonical association stability through layout changes;
- deterministic search/disambiguation;
- evidence applicability/staleness projection;
- retention/eviction;
- parser callback policy;
- keyboard command mapping;
- performance topology/search microbenchmarks.

### Parser/renderer fixtures

Use deterministic byte fixtures for:

- ANSI styles/cursor movement;
- Unicode/wide/combining text;
- malformed/truncated/oversized escape sequences;
- OSC52;
- hyperlink/title/window-size requests;
- forged Winds labels/evidence-like JSON;
- invalid UTF-8/binary-like streams;
- very large output.

### Real terminal integration

Reuse existing terminal profiles and lifecycle test helpers. Add only the minimum new integration layer proving panes do not break accepted lifecycle semantics.

### Regression gates

Every implementation candidate must run:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

and all applicable existing Spec 003/006 workflow gates. Any new focused test module must be registered and proven executed according to canonical repository rules.

## Implementation Slice Strategy

Tasks should authorize small dependency-ordered slices. This Plan does not itself authorize any slice.

Recommended sequence:

1. dependency/license/MSRV/resolved-graph qualification and minimal UI skeleton;
2. pure workbench topology/state model;
3. vt100 screen projection + fail-closed callback policy;
4. pane-to-existing-terminal lifecycle integration;
5. shell editor + explicit one-pane dispatch;
6. keyboard/pointer pane navigation and deterministic canonical search;
7. typed interaction/source-labelling and bounded transcript retention;
8. exact candidate/evidence/verification read-only surfaces;
9. terminal host-integration safety campaign;
10. cross-platform integration qualification;
11. performance/retention/idle/resize campaigns;
12. accessibility and UX acceptance campaign;
13. full negative/adversarial/regression campaign;
14. final Spec 007 reconciliation and exact-candidate acceptance.

Tasks may split a recommended slice further when needed for reviewability. They may not combine later authority or skip dependencies.

## Dependency Landing Gate

The Task that first edits `Cargo.toml` / `Cargo.lock` must, on its exact candidate:

- pin exact direct versions;
- capture the resolved Cargo graph;
- inspect direct/transitive licenses for compatibility with `MIT OR Apache-2.0` distribution;
- inspect crate checksums/source identities from lockfile;
- verify MSRV against Winds `1.97.1`;
- compile/test on Ubuntu, macOS, and native Windows where the dependency participates in claimed behavior;
- verify Crossterm `osc52` is not enabled;
- verify no unwanted async/network/provider/clipboard dependency is introduced;
- run Ponytail/YAGNI review specifically against each new crate and enabled feature;
- refuse landing if a smaller existing-dependency implementation proves sufficient.

## Failure and Recovery Semantics

The workbench must remain fail-closed under:

- terminal child exit during input/edit/render;
- output reader closure/error;
- parser failure or unsupported sequence;
- UI rendering error;
- resize race;
- pane removal while events are queued;
- candidate movement while evidence is visible;
- deleted/moved/unavailable working directory;
- executable/profile replacement before new launch;
- WSL path ambiguity;
- transcript eviction;
- user cancellation/terminal restore failure.

A workbench failure must not rewrite terminal lifecycle truth or canonical evidence. Best-effort host terminal restoration should be explicit and separately tested.

## Security / Privacy Posture

- no raw environment capture for UI state;
- no provider credentials or model invocation;
- no new network listener/client path;
- no transcript-to-canonical-memory promotion;
- bounded transcript storage only when explicitly Task-authorized;
- terminal output is untrusted and may contain secrets or prompt injection text;
- copy/export/open operations require explicit user intent;
- path visibility does not grant Agent authority;
- code/diff inspection is read-only under Spec 007;
- no clipboard writes from terminal-originated OSC52 by default;
- no automatic external URL/file opening;
- no automatic Git landing operations.

## Evidence Integrity

Every acceptance artifact must bind to exact candidate state. At minimum record candidate commit/tree and the command/runner/platform that produced the result.

No evidence may be current if:

- HEAD changed after it was produced;
- the tested tree differs;
- the platform/domain differs from the claim;
- a benchmark used a different fixture/reference environment without explicit comparability proof;
- the result is Agent/terminal prose rather than Winds/CI observation;
- the result came from a prior implementation slice and was not rerun when the current task requires exact-head proof.

## Ponytail / YAGNI Boundaries

Explicitly reject during Spec 007 unless an accepted amendment changes scope:

- daemon/process supervisor architecture;
- IPC/socket/RPC layer;
- desktop GUI/webview application shell;
- custom terminal protocol;
- custom full text editor;
- custom PTY implementation;
- provider/model gateway;
- generic Agent runtime interface;
- browser/CDP integration;
- plugin marketplace/SDK;
- semantic/vector memory;
- LSP/IDE replacement;
- generalized workflow engine;
- automatic Git landing;
- speculative persistence schema for future phases.

Prefer closed enums, concrete modules, pure state transitions, and direct adapters until two proven use cases justify an abstraction.

## External Research Provenance

Research inputs are evidence for planning decisions, not copied implementation code.

Primary dependency sources revalidated on 2026-09-06:

- Ratatui: https://github.com/ratatui/ratatui and https://docs.rs/crate/ratatui/0.30.2
- Crossterm: https://github.com/crossterm-rs/crossterm and https://docs.rs/crate/crossterm/0.29.0
- ratatui-textarea: https://github.com/ratatui/ratatui-textarea and https://docs.rs/crate/ratatui-textarea/0.9.2
- vt100: https://github.com/doy/vt100-rust and https://docs.rs/crate/vt100/0.16.2

Research-only product references such as Warp may inform interaction principles but MUST NOT be copied or adapted from incompatible AGPL implementation code into Winds. Implementation must remain independently designed against the accepted Spec/Plan and compatible dependency sources.

## Plan Acceptance Gate

This Plan can land only if all are true on its exact final candidate:

- canonical base is the post-Spec-007-spec merge main;
- changed scope is planning/documentation only;
- no `Cargo.toml`, lockfile, migration, workflow semantic, source, runtime, provider, browser, daemon, IPC, learning, or plugin change occurs;
- dependency decisions have exact version/license/MSRV/provenance rationale and remain Task-stage additions rather than Plan-stage code changes;
- architecture reuses accepted terminal lifecycle and verification authority rather than duplicating them;
- all FR-001..FR-066 and SC-001..SC-018 have a plausible implementation/evidence path;
- frozen FR-045..FR-053 budgets are not weakened;
- exact-head repository `quality` succeeds;
- correctness/safety/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads remain;
- exact main/base/head/tree/scope/ruleset/mergeability are reconciled immediately before guarded merge;
- guarded expected-head merge succeeds;
- post-merge canonical main/tree and push checks are verified.

## Downstream Authorization

Canonical acceptance and post-merge verification of this Plan authorize **Spec 007 Tasks creation only**.

They do not authorize implementation, dependency landing, migrations, source changes, runtime changes, provider/browser execution, daemon/IPC, remote execution, learning, plugins, ACP/MCP, automatic landing, or later specifications.

Only after this Plan lands and is post-merge verified may repository truth state:

```text
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS_AUTHORIZED=YES
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
```
