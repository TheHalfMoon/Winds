# Feature Specification: Native Agentic Terminal UX Foundation

**Feature Branch**: `spec/007-native-agentic-terminal-ux-foundation`

**Created**: 2026-09-06

**Canonical Base**: `9090895f368f4c5f2dae99b56c15f9501dea4063`

**Status**: Authorized for specification only; planning, tasks, dependencies, migrations, UI-framework selection, runtime implementation, provider/browser execution, daemon/IPC work, and product-source changes are NOT authorized by this file alone

**Input**: Turn Winds' already accepted local terminal/session/evidence foundations into a daily-driver interactive terminal workbench whose presentation is fast, keyboard-first, source-labelled, exact-candidate aware, and visibly verification-native, while preserving the current one-process architecture and refusing to convert terminal or Agent output into trusted evidence merely because it appears in the UI.

## Product North Star

Winds should feel like a modern terminal workbench without weakening the truth model that differentiates it from an ordinary terminal, chat shell, or agent console.

For every visible interaction the user should be able to answer:

1. **Which canonical workspace/session/work is this?** UI titles and pane geometry are presentation, not identity.
2. **Which live terminal does this pane actually own?** Child ownership remains tied to the accepted terminal lifecycle semantics, not guessed from PID/native identifiers.
3. **What was typed, what was emitted, and what was inferred by the UI?** User input, terminal output, Agent-reported content, Winds-observed evidence, and human decisions remain distinguishable.
4. **Which exact candidate does this evidence apply to?** Candidate movement makes earlier evidence visibly stale/not applicable.
5. **Is the work merely finished, or actually verified?** `DONE`, `VERIFIED`, and `ACCEPTED` remain separate states.
6. **Can the user get to the needed shell, session, file/diff, and verification result quickly?** Navigation and findability are first-class product requirements.
7. **Does the workbench remain truthful under stress?** Large output, resize storms, child exit, Unicode, malformed escape sequences, and platform differences must not create false state.

The differentiated loop is:

> **fast terminal work -> visible canonical context -> explicit evidence -> verification -> human decision**

not “decorate a shell transcript” and not “chat controls the machine.”

## Frozen Product Invariants

The following are requirements:

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

Spec 006 live-runtime nonclaims also remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Nothing in this specification upgrades those separate live-evidence lanes.

## User Scenarios & Testing

### User Story 1 - Use Winds as a Fast Daily-Driver Terminal Workbench (Priority: P1)

A developer opens Winds in an existing repository, starts one or more terminal panes, runs ordinary shell commands, resizes/focuses/closes panes, and continues normal shell work without losing the lifecycle and cross-platform correctness already established by Winds.

The workbench is a richer presentation over accepted terminal ownership semantics. It is not a new process-ownership authority.

**Why this priority**: If ordinary terminal use is slower, less compatible, or less truthful than the existing Winds terminal substrate, every higher-level agentic feature becomes a regression rather than a product improvement.

**Independent Test**: On each claimed platform/domain, create multiple terminal panes from supported profiles, exercise input/output/resize/interrupt/terminate/close, and verify exact owned-child semantics remain intact while the workbench remains responsive and reports child exit/ownership loss truthfully.

**Acceptance Scenarios**:

1. **Given** an existing workspace and supported shell profile, **When** the user opens a terminal pane, **Then** the pane starts through the accepted Winds terminal lifecycle and is associated with explicit canonical workspace/session context where available.
2. **Given** several active panes, **When** the user changes focus, layout, or display labels, **Then** those UI operations do not change canonical workspace/workstream/session identity.
3. **Given** a pane whose exact child exits, **When** the UI updates, **Then** Winds shows the terminal as exited/stopped and does not keep presenting it as live because the pane still exists visually.
4. **Given** a pane close/terminate operation, **When** exact owned-child reaping cannot be proven, **Then** the underlying truthful ownership-loss/failure state is surfaced rather than converted into success.
5. **Given** the workbench process exits, **When** it is started again, **Then** Spec 007 does not claim that prior child processes remained durably owned or reattachable; persistent owner/detach semantics are outside this specification.

### User Story 2 - Read Terminal Work as Typed, Source-Labelled Interaction Blocks (Priority: P1)

A developer can visually distinguish user-entered commands, shell/terminal output, relevant Agent-reported material already known to Winds, deterministic verification results, warnings/policy decisions, and explicit human decisions without treating a visual block boundary as evidence authority.

**Why this priority**: A richer terminal is useful only if it reduces transcript ambiguity without silently manufacturing semantic truth.

**Independent Test**: Feed deterministic fixtures containing commands, normal output, multiline output, stderr-like text, ANSI sequences, strings that imitate Winds evidence, Agent-like “verified” claims, and actual repository-native evidence records. Verify block/source labels are deterministic and only accepted evidence receives Winds-observed/evidence treatment.

**Acceptance Scenarios**:

1. **Given** a command entered through Winds-owned input, **When** its output is displayed, **Then** the UI may group input and subsequently observed terminal bytes for readability but MUST NOT infer verification/acceptance from the output text.
2. **Given** terminal output containing strings such as “PASS”, “verified”, “approved”, JSON resembling evidence, or forged UI markers, **When** rendered, **Then** it remains terminal output and cannot self-promote to `WINDS_OBSERVED`, `VERIFIED`, or `ACCEPTED`.
3. **Given** an accepted verification/evidence record from Winds' canonical evidence path, **When** displayed in the workbench, **Then** its exact source/candidate identity and applicability are visible and distinguish it from ordinary output.
4. **Given** an Agent-reported result already represented by accepted Spec 006 structures, **When** surfaced, **Then** it remains explicitly `AGENT_REPORTED` unless separately observed or human-decided.
5. **Given** output whose boundaries cannot be safely attributed to one command, **When** rendered, **Then** Winds preserves truthful continuous/raw terminal presentation rather than inventing a command-result relationship.

### User Story 3 - Enter and Edit Shell Commands Without Fighting the Terminal (Priority: P1)

A developer has a keyboard-first universal shell input experience that supports ordinary commands, multiline editing/paste where shell semantics permit, history recall, cancellation/clearing, and explicit dispatch to the selected pane without silently routing text to a model/provider.

**Why this priority**: The main input path must be faster and safer than a conventional terminal before richer agent input modes are justified.

**Independent Test**: Exercise empty input, Unicode, long commands, multiline paste, bracketed-paste-sensitive content, rapid edits, history navigation, pane switching before dispatch, and shell metacharacters. Verify bytes reach only the explicitly targeted owned terminal and no provider/model path is invoked.

**Acceptance Scenarios**:

1. **Given** one selected terminal pane, **When** the user submits shell input, **Then** input is sent only to that explicit pane's accepted terminal input path.
2. **Given** several panes, **When** focus changes before submission, **Then** the UI clearly identifies the target and never broadcasts input by default.
3. **Given** pasted multiline text, **When** the input surface cannot preserve safe shell/paste semantics, **Then** Winds requires explicit dispatch behavior or falls back to the terminal's accepted paste semantics rather than silently executing line by line.
4. **Given** shell-looking text that resembles an Agent prompt, **When** the workbench is in shell mode, **Then** it remains shell input; Spec 007 adds no silent natural-language/model router.
5. **Given** an unsupported input/editor capability on a platform, **When** used, **Then** Winds reports or falls back truthfully rather than dropping/reordering bytes.

### User Story 4 - Navigate Tabs, Panes, Workspaces, and Winds Sessions Quickly (Priority: P1)

A developer can create, focus, split, resize, close, and find terminal panes; move between workspace/session contexts; and understand which canonical Winds session a surface belongs to without memorizing native runtime IDs or full paths.

All topology is local to the current workbench lifetime under this specification. It does not imply a daemon or durable multiplexer owner.

**Independent Test**: Build a deterministic fixture with multiple workspaces, similarly named Winds sessions, Unicode/case-colliding display labels, and many panes. Exercise keyboard and pointer navigation, layout changes, ambiguous search, pane exit, and UI restart. Verify stable canonical references remain separate from transient pane/layout identity.

**Acceptance Scenarios**:

1. **Given** multiple panes, **When** the user splits/resizes/focuses/reorders presentation, **Then** terminal/session identity and evidence links remain unchanged.
2. **Given** similar workspace/session names, **When** the user searches, **Then** deterministic candidates include enough canonical context to disambiguate rather than silently selecting by recency.
3. **Given** a Winds session selected from existing canonical records, **When** a terminal pane is associated with it, **Then** the association uses stable session identity rather than display text.
4. **Given** a pane that has exited, **When** navigation lists it, **Then** the UI distinguishes exited/history state from a live owned terminal.
5. **Given** application restart, **When** prior layout/session presentation is restored where supported, **Then** restoration of UI metadata MUST NOT claim restoration of live child ownership.

### User Story 5 - See Exact Candidate, Diff, Evidence, and Verification State Where Work Happens (Priority: P1)

A developer can inspect the current Git/candidate context, relevant diff/evidence, and repository-native verification state from the workbench without turning the UI into a second verification authority or automatically landing changes.

**Why this priority**: Verification visibility is the primary product distinction Winds must preserve while becoming a better terminal.

**Independent Test**: Attach candidate A and its accepted evidence to a fixture workspace, surface it in the workbench, then move to candidate B. Verify A evidence remains historical and visibly not applicable to B until B receives fresh accepted evidence. Verify an Agent/terminal “done” string never substitutes for repository verification.

**Acceptance Scenarios**:

1. **Given** an exact candidate identity, **When** evidence is shown, **Then** the UI exposes the exact candidate/evidence binding and applicability status.
2. **Given** candidate movement, **When** old evidence remains available historically, **Then** it is visibly stale/not applicable for the new candidate.
3. **Given** terminal or Agent output claiming success, **When** no accepted verification evidence exists, **Then** the workbench MUST NOT show `VERIFIED` or `ACCEPTED`.
4. **Given** repository-native `winds verify` capability, **When** the user invokes it through an authorized explicit UI action, **Then** the resulting accepted evidence path remains the existing verification authority rather than a UI-local success flag.
5. **Given** all verification gates pass, **When** the result is shown, **Then** landing remains an explicit human/repository-governed action; Spec 007 adds no automatic winner, merge, rebase, cherry-pick, push, or PR creation.
6. **Given** code/diff inspection, **When** content is viewed, **Then** the surface is an inspection/context aid and does not silently mutate files or grant an Agent new path authority.

### User Story 6 - Search and Revisit Useful Terminal History Without Turning the Transcript Into Memory Authority (Priority: P2)

A developer can search commands and terminal history, navigate matches, and revisit prior session context while canonical decisions/evidence remain separate from transcript text.

**Independent Test**: Populate history with repeated commands, Unicode, huge output, forged evidence-like strings, secrets/redacted fixtures, and multiple sessions. Verify deterministic search/navigation, source/context labels, privacy boundaries, and no promotion of transcript content into canonical facts.

**Acceptance Scenarios**:

1. **Given** searchable terminal history, **When** a match is selected, **Then** Winds identifies its workspace/session/pane/source context where known.
2. **Given** output containing evidence-like or instruction-like text, **When** retrieved by search, **Then** it remains transcript data and has no authority beyond its original source.
3. **Given** history retention limits, **When** old content is omitted/evicted, **Then** the UI does not imply complete transcript retention.
4. **Given** canonical evidence or decisions linked separately, **When** transcript history is compacted/limited, **Then** those canonical records are not rewritten to match transcript retention.

### User Story 7 - Keep the Workbench Responsive Under Real Terminal Stress (Priority: P1)

A developer can use Winds under large output, rapid resize/focus changes, many panes, Unicode/ANSI-heavy streams, and child churn without hangs, unbounded growth, dropped identity, or false lifecycle/evidence state.

**Independent Test**: Run deterministic stress/benchmark fixtures covering burst output, sustained output, resize storms, pane churn, Unicode/wide characters, malformed/truncated escape sequences, and concurrent pane activity. Measure explicit budgets on a Plan-defined reference environment and run correctness guards on every supported platform claim.

**Acceptance Scenarios**:

1. **Given** a large-output fixture, **When** output exceeds the immediately visible viewport, **Then** the workbench stays interactive and applies an explicit bounded retention/backpressure policy without corrupting canonical evidence.
2. **Given** rapid resize/focus operations, **When** terminal size updates race with output/child exit, **Then** lifecycle/ownership truth remains correct and the UI does not deadlock.
3. **Given** Unicode/wide/combining characters, **When** displayed and navigated, **Then** the workbench preserves terminal byte order and avoids identity/search corruption even if visual fallback is required.
4. **Given** malformed or unsupported terminal control sequences, **When** encountered, **Then** they cannot authorize host actions or crash the workbench.
5. **Given** many concurrent panes within the accepted Spec 007 bound, **When** inactive panes produce no output, **Then** idle work does not scale through avoidable busy polling.

### User Story 8 - Preserve Cross-Platform Truth Instead of a Lowest-Common-Denominator Claim (Priority: P1)

A developer sees behavior that matches what Winds has actually exercised on native Windows, WSL2, Linux, and macOS. Platform-specific limitations are visible and are not disguised as universal support.

**Independent Test**: Exercise the specification's claimed terminal/workbench behaviors through platform-specific CI or directly documented evidence. Verify native Windows ConPTY and WSL2 remain distinct execution domains, Unix PTY behavior is independently exercised, and unsupported behavior is labelled unavailable rather than inferred from another platform.

**Acceptance Scenarios**:

1. **Given** a native Windows pane, **When** terminal lifecycle operations occur, **Then** ConPTY-specific accepted ownership/lifecycle semantics remain authoritative.
2. **Given** a WSL2 profile, **When** launched from Windows, **Then** Winds preserves explicit Windows-host versus Linux-guest path/domain truth.
3. **Given** Linux/macOS PTY operation, **When** signals/resize/close occur, **Then** Unix-specific accepted semantics are exercised rather than inferred from Windows tests.
4. **Given** a workbench feature not directly tested on one platform/domain, **When** support is reported, **Then** it is marked unavailable/experimental/not-claimed as appropriate instead of being represented as proven parity.

## Terminal Safety Edge Cases

- Child exits while a command is being edited or dispatched.
- Pane closes while output-reader work is in flight.
- Rapid split/resize/focus churn while multiple children emit output.
- Shell profile/executable disappears or changes before launch.
- Working directory is deleted, moved, inaccessible, or belongs to another workspace.
- Display labels collide, differ only by case, or contain Unicode control/combining characters.
- Terminal output includes ANSI/OSC sequences, forged hyperlinks, title changes, OSC 52 clipboard requests, device-control-like bytes, or strings that imitate Winds UI/evidence records.
- Output contains invalid UTF-8/binary-like bytes.
- A hyperlink points to an external scheme, local file, or command-like URL.
- Bracketed paste state and multiline paste differ across shells/platforms.
- Command history contains secret-like material; search/retention must not create a new implicit credential store.
- Output retention truncates a transcript while canonical evidence still exists separately.
- Candidate identity changes while an evidence/diff view is open.
- A terminal/Agent says “verified” after the exact candidate has moved.
- UI restoration metadata exists after process restart but prior child ownership cannot be proved.
- WSL path translation is ambiguous or unsupported.
- A pane is visible to the user but its associated context/path is outside an Agent's approved authority.

## Functional Requirements

### Terminal ownership and workbench model

- **FR-001**: Winds MUST retain the already accepted terminal child/session handle as the authority for live terminal lifecycle operations; Spec 007 UI state MUST NOT replace that authority.
- **FR-002**: Winds MUST support multiple concurrently visible/available terminal panes within one workbench process, subject to explicit implementation bounds defined by Tasks.
- **FR-003**: Every live pane MUST have a transient pane identity separate from canonical workspace, workstream, Winds session, runtime-native, and process identities.
- **FR-004**: Workspace/session display labels and pane titles MUST remain presentation-only and MUST NOT be used as canonical identity.
- **FR-005**: Pane focus/layout changes MUST NOT mutate canonical workspace/workstream/session identity or evidence applicability.
- **FR-006**: Pane close/interrupt/terminate/resize MUST use the accepted platform-specific terminal lifecycle path and MUST preserve truthful failure/ownership-loss outcomes.
- **FR-007**: Application/UI restart MUST NOT be represented as durable live-child reattachment under Spec 007.
- **FR-008**: Spec 007 MUST preserve the current one-process architecture and MUST NOT require a daemon, persistent owner, socket, local control server, or IPC protocol.

### Typed interaction and source truth

- **FR-009**: The workbench MUST distinguish at minimum user/Winds-owned command input, terminal output, canonical verification/evidence, warnings/policy state, and explicit human decisions where those sources are available.
- **FR-010**: Agent-reported content surfaced from existing canonical structures MUST remain explicitly source-labelled and MUST NOT be presented as Winds-observed evidence merely because it is rendered in a structured block.
- **FR-011**: Terminal output text MUST NOT self-promote to evidence, verification, acceptance, authorization, or human decision regardless of its contents or formatting.
- **FR-012**: UI grouping of command and output MUST be based only on Winds-observed interaction boundaries; ambiguous attribution MUST fall back to truthful ungrouped/continuous presentation.
- **FR-013**: The workbench MUST preserve output byte/order semantics required by the terminal substrate even when text decoding/rendering requires lossy or replacement-character presentation.
- **FR-014**: Canonical evidence cards/views MUST expose source identity and exact candidate/applicability information sufficient to distinguish them from transcript output.

### Input/editor behavior

- **FR-015**: The primary input mode MUST be explicit shell input to one selected terminal pane; Spec 007 MUST NOT silently route text to a model/provider.
- **FR-016**: Input dispatch MUST target exactly one explicit pane by default and MUST NOT broadcast without a separately explicit future action.
- **FR-017**: The input/editor MUST support Unicode and commands materially longer than one viewport line without silently truncating content.
- **FR-018**: Multiline paste/input MUST preserve explicit shell/paste semantics and MUST NOT silently execute each line as separate commands when that changes meaning.
- **FR-019**: History recall/editing MUST preserve the exact submitted command text or explicitly identify any normalization; history display MUST NOT become canonical task memory.
- **FR-020**: Unsupported input/editor behavior MUST degrade truthfully to accepted terminal semantics rather than silently dropping/reordering input.

### Tabs, panes, sessions, and navigation

- **FR-021**: The workbench MUST provide explicit create/focus/split/resize/close operations for its accepted pane topology without implying persistent multiplexer ownership.
- **FR-022**: Navigation MUST expose enough workspace/session context to distinguish similar or duplicate display labels.
- **FR-023**: Associating a pane with an existing Winds session MUST use the stable canonical session identity, not display text or provider-native identity.
- **FR-024**: Search over workspaces/sessions/panes MUST be deterministic for identical canonical inputs and MUST return explicit candidates when ambiguity is material.
- **FR-025**: Exited/stopped/history panes MUST be distinguishable from live owned panes.
- **FR-026**: Restored presentation/layout metadata after process restart MUST NOT imply restored live process ownership.
- **FR-027**: Pane/workspace/session visibility MUST NOT grant an Agent or delegated actor new file, tool, execution, or context-transfer authority.

### Verification-native workbench

- **FR-028**: The workbench MUST visibly distinguish `IDLE`, `DONE`/Agent-reported completion where applicable, `VERIFIED`, and `ACCEPTED` rather than collapsing them.
- **FR-029**: Accepted verification/evidence displayed by Spec 007 MUST remain bound to the exact candidate identity used by the existing verification/evidence model.
- **FR-030**: Candidate movement MUST make earlier candidate-bound evidence/review visibly stale or not applicable without deleting its historical record.
- **FR-031**: Terminal command exit status alone MUST NOT establish `VERIFIED` or `ACCEPTED`.
- **FR-032**: Terminal or Agent prose claiming success MUST NOT establish `VERIFIED` or `ACCEPTED`.
- **FR-033**: Any workbench action that invokes existing `winds verify` behavior MUST remain an explicit user action and MUST preserve the accepted verification authority/semantics rather than implementing a second UI-local verifier.
- **FR-034**: Code/diff/evidence inspection surfaces MUST be non-mutating under Spec 007 unless a later accepted task explicitly authorizes a specific mutation path.
- **FR-035**: Spec 007 MUST NOT automatically select a winning candidate, merge, rebase, cherry-pick, push, create a PR, or land changes.

### History, search, and retention

- **FR-036**: Terminal history/search MUST preserve source/workspace/session/pane context where known and MUST NOT represent transcript completeness when retention is bounded.
- **FR-037**: Search results containing forged evidence-like text MUST remain transcript data and MUST NOT inherit canonical authority from matching trusted keywords.
- **FR-038**: Any history-retention policy MUST be explicit and bounded; transcript eviction/truncation MUST NOT mutate canonical evidence, approvals, or human decisions.
- **FR-039**: Spec 007 history/search MUST NOT introduce a general semantic-memory, vector/RAG, embedding, or learned-retrieval subsystem.

### Terminal escape and host-integration safety

- **FR-040**: Terminal control sequences MUST NOT directly grant file, clipboard, command, network, browser, or other host-side authority beyond the accepted terminal-rendering contract.
- **FR-041**: Clipboard-writing escape sequences such as OSC 52 MUST be disabled, consent-gated, or otherwise explicitly bounded by the future Plan/Tasks; they MUST NOT silently overwrite the host clipboard by default.
- **FR-042**: Hyperlinks/file references emitted by terminal content MUST require explicit user interaction before any external/open action and MUST remain subject to scheme/path/policy validation.
- **FR-043**: Malformed, truncated, oversized, unknown, or unsupported escape/control sequences MUST fail safely and MUST NOT crash the workbench or create privileged host actions.
- **FR-044**: Terminal output MUST NOT be able to forge trusted Winds UI chrome/source labels in a way that changes the underlying evidence/authority state.

### Performance, boundedness, and platform truth

- **FR-045**: The future Plan MUST define one or more reproducible reference benchmark environments before implementation Tasks claim latency/resource targets.
- **FR-046**: The implementation program MUST include deterministic or reproducible performance fixtures for cold/workbench startup overhead, input-to-dispatch overhead, pane/navigation interaction, large-output handling, and idle-resource scaling.
- **FR-047**: On the Plan-defined reference environment, workbench-only input-to-dispatch overhead SHOULD remain <= 16 ms p95 under the baseline one-pane fixture and MUST have an explicit hard regression threshold before implementation closes.
- **FR-048**: On the Plan-defined reference environment, pane focus/split/navigation model updates SHOULD remain <= 16 ms p95 for the accepted baseline topology and MUST have an explicit hard regression threshold before implementation closes.
- **FR-049**: The implementation MUST exercise at least a 100,000-line or equivalently bounded high-volume deterministic output fixture without deadlock, unbounded memory growth, lifecycle corruption, or evidence-state corruption.
- **FR-050**: Output retention/backpressure MUST be explicitly bounded and testable; Winds MUST NOT rely on unbounded transcript growth for correctness.
- **FR-051**: Inactive panes with no incoming output MUST NOT require avoidable continuous busy polling; idle scaling MUST be measured in the implementation qualification program.
- **FR-052**: Platform support claims MUST be limited to directly exercised native Windows/ConPTY, WSL2, Linux, and macOS domains as applicable.
- **FR-053**: Native Windows and WSL2 MUST remain distinct execution/path domains in UI state and evidence.
- **FR-054**: Platform-specific unavailable/unsupported behavior MUST be represented truthfully rather than inferred from another platform's success.

### Accessibility and interaction quality

- **FR-055**: All core create/focus/navigate/search/verify-inspect operations MUST be reachable through documented keyboard interactions; pointer interaction may be additive.
- **FR-056**: Focus state, selected pane, exited/live state, warnings, evidence applicability, and verification status MUST not rely on color alone.
- **FR-057**: The workbench MUST preserve readable text/selection/copy semantics and must define behavior for wide/combining characters and high-DPI/scale changes before claiming accessibility parity.
- **FR-058**: The future Plan/Tasks MUST include accessibility checks appropriate to the selected UI stack and directly claimed platform surfaces.

### Scope and governance boundaries

- **FR-059**: Spec 007 MUST NOT add a persistent owner, daemon, public/private IPC/control protocol, remote execution route, or mobile/remote continuation surface.
- **FR-060**: Spec 007 MUST NOT add browser automation/profile/CDP runtime, provider mesh/provider SDK/authentication brokerage, automatic model routing, or new provider execution authority.
- **FR-061**: Spec 007 MUST NOT add ACP dependency landing, MCP runtime, A2A, generic runtime/plugin framework, integration SDK, plugin host, or marketplace.
- **FR-062**: Spec 007 MUST NOT add verified-learning activation, skill promotion, experiment plane, model training/fine-tuning/RL, vector/RAG memory, or executable self-modification.
- **FR-063**: Spec 007 MUST preserve all applicable Spec 003 and Spec 006 Git/evidence/authority/recovery/platform/privacy invariants.
- **FR-064**: UI framework, renderer, terminal-rendering library, parser, clipboard integration, and any other new dependency are Plan-stage decisions and require exact-version/license/MSRV/platform/provenance/YAGNI review before Tasks may authorize them.

## Success Criteria

- **SC-001**: A user can start at least 10 concurrent fixture terminal panes in one workbench process, navigate/focus/resize/close them deterministically, and preserve exact owned-child lifecycle truth for every pane.
- **SC-002**: Repeated workspace/session display renames, duplicate/case/Unicode names, and pane layout changes produce zero canonical identity collisions or evidence-link movement in deterministic tests.
- **SC-003**: Forged terminal strings containing `PASS`, `VERIFIED`, `ACCEPTED`, evidence-like JSON, or trusted-looking UI labels produce zero promotion into canonical evidence/verification state.
- **SC-004**: Exact candidate A evidence becomes visibly not-applicable/stale after movement to candidate B while remaining historically inspectable.
- **SC-005**: Shell input fixtures prove exactly-one-pane dispatch by default, no provider/model invocation, and no silent broadcast.
- **SC-006**: Multiline/Unicode/long-input fixtures preserve explicit byte/text semantics without silent truncation or line-by-line execution drift.
- **SC-007**: A 100,000-line or equivalently bounded high-volume terminal-output campaign completes without deadlock, unbounded retention, child-ownership corruption, or evidence-state corruption.
- **SC-008**: Malformed/unknown/oversized escape-sequence fixtures cannot crash the workbench or trigger unauthorized clipboard/file/browser/command host actions.
- **SC-009**: Core workbench operations have keyboard-reachable paths and critical state is distinguishable without color-only encoding.
- **SC-010**: Plan-defined benchmark evidence demonstrates the accepted hard budgets for input-to-dispatch and pane/navigation interactions; aspirational `<=16 ms p95` targets are either met or transparently amended before implementation closeout.
- **SC-011**: Idle-resource and large-output measurements are reproducible on the Plan-defined reference environment and show bounded behavior for the accepted pane topology.
- **SC-012**: Native Windows/ConPTY, WSL2, Linux, and macOS claims are each backed by direct evidence for the exact behaviors claimed; no platform inherits proof solely from another platform.
- **SC-013**: A workbench restart test proves presentation/session metadata restoration, if implemented, never claims durable live-child ownership without a persistent owner.
- **SC-014**: Existing repository `quality` and applicable Spec 003/006 terminal/verification regression gates remain green on the final exact implementation candidate.
- **SC-015**: Independent correctness/safety and Ponytail/YAGNI reviews find zero unresolved material defects or unjustified architecture/dependency expansion on the final implementation candidate.
- **SC-016**: The final Spec 007 reconciliation can truthfully state that no daemon/IPC, remote execution, browser runtime, provider mesh implementation, ACP/MCP runtime, generic plugin system, learning subsystem, automatic winner, or automatic landing behavior was introduced.

## Explicit Non-Goals

Spec 007 does not authorize:

- durable background ownership or detach/reattach across Winds process exit;
- daemon/server/socket/IPC/control API architecture;
- remote execution, SSH control, mobile continuation, or cloud sync;
- Browser Twin, browser automation, browser credential/profile management, screenshots as verification evidence, or CDP;
- provider mesh, provider SDK integration, provider credential brokerage, silent model routing, or new live Codex/Claude acceptance claims;
- MCP runtime, ACP dependency landing, ACP v2, A2A, generic runtime adapters, or generic tool gateway;
- recursive/multi-worker fleets beyond already accepted Spec 006 boundaries;
- verified-learning activation, skill mutation/promotion, experiment plane, learned routing, training/fine-tuning/RL;
- vector/embedding/RAG memory or transcript-as-canonical-memory architecture;
- plugin host, integration SDK, marketplace, or third-party executable extension supply chain;
- full IDE/source editor replacement;
- automatic candidate winner selection or automatic Git landing operations.

## Assumptions

- Existing Spec 003 terminal lifecycle and Spec 006 canonical workspace/session/evidence structures remain the starting truth rather than being replaced.
- The first implementation program may introduce a UI/rendering dependency only after the Plan proves it is necessary and passes exact dependency review.
- The current binary may evolve into a richer interactive surface while remaining one process for Spec 007; process separation is not required to satisfy this specification.
- Persistence needed solely for non-live presentation metadata/history may be considered by the future Plan, but it must not create durable process-ownership claims.
- Performance thresholds require a reproducible reference environment; the Plan must pin that environment and convert aspirational targets into explicit hard acceptance thresholds before implementation Tasks close.

## Governance / Downstream Authorization

Canonical acceptance of this specification authorizes **Spec 007 Plan creation only**.

It does not authorize Tasks, implementation, dependencies, migrations, UI framework selection, source changes, provider/browser execution, daemon/IPC, remote execution, learning, or later specifications.

Only after this specification lands and is post-merge verified may repository truth state:

```text
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN_AUTHORIZED=YES
SPEC_007_TASKS_AUTHORIZED=NO
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
```
