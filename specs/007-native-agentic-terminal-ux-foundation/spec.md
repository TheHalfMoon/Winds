# Feature Specification: Native Agentic Terminal UX Foundation

**Feature Branch**: `spec/007-native-agentic-terminal-ux-foundation`

**Created**: 2026-09-06

**Canonical Base**: `9090895f368f4c5f2dae99b56c15f9501dea4063`

**Status**: Authorized for specification only; planning, tasks, dependencies, migrations, UI-framework selection, runtime implementation, provider/browser execution, daemon/IPC work, and product-source changes are NOT authorized by this file alone

**Input**: Turn Winds' accepted local terminal/session/evidence foundations into a daily-driver interactive terminal workbench whose presentation is fast, keyboard-first, source-labelled, exact-candidate aware, and visibly verification-native, while preserving the current one-process architecture and refusing to convert terminal or Agent output into trusted evidence merely because it appears in the UI.

## Product North Star

Winds should feel like a modern terminal workbench without weakening the truth model that differentiates it from an ordinary terminal, chat shell, or agent console.

For every visible interaction the user should be able to answer:

1. **Which canonical workspace/session/work is this?** UI titles and pane geometry are presentation, not identity.
2. **Which live terminal does this pane actually own?** Child ownership remains tied to accepted terminal lifecycle semantics, not guessed from PID/native identifiers.
3. **What was typed, emitted, inferred, observed, or decided?** User input, terminal output, Agent-reported content, Winds-observed evidence, and human decisions remain distinguishable.
4. **Which exact candidate does this evidence apply to?** Candidate movement makes earlier evidence visibly stale/not applicable.
5. **Is the work merely finished, or actually verified?** `DONE`, `VERIFIED`, and `ACCEPTED` remain separate states.
6. **Can the user reach the needed shell, session, diff, and verification result quickly?** Navigation and findability are product requirements.
7. **Does the workbench remain truthful under stress?** Large output, resize storms, child exit, Unicode, malformed escape sequences, and platform differences must not create false state.

The differentiated loop is:

> **fast terminal work -> visible canonical context -> explicit evidence -> verification -> human decision**

not “decorate a shell transcript” and not “chat controls the machine.”

## Frozen Product Invariants

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

Spec 006 live-runtime nonclaims remain unchanged:

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

A developer opens Winds in an existing repository, starts multiple terminal panes, runs ordinary shell commands, resizes/focuses/closes panes, and continues normal shell work without losing the lifecycle and cross-platform correctness already established by Winds.

The workbench is a richer presentation over accepted terminal ownership semantics. It is not a new process-ownership authority.

**Independent Test**: On each claimed platform/domain, create multiple terminal panes from supported profiles, exercise input/output/resize/interrupt/terminate/close, and verify exact owned-child semantics remain intact while the workbench remains responsive and reports child exit/ownership loss truthfully.

**Acceptance Scenarios**:

1. **Given** an existing workspace and supported shell profile, **When** the user opens a pane, **Then** it starts through the accepted Winds terminal lifecycle and carries explicit canonical workspace/session context where available.
2. **Given** several panes, **When** focus/layout/display labels change, **Then** canonical workspace/workstream/session identity does not change.
3. **Given** an exact child exit, **When** the UI updates, **Then** the pane is shown as exited/stopped rather than visually live.
4. **Given** close/terminate where exact reaping cannot be proven, **When** the operation ends, **Then** truthful failure/ownership-loss is surfaced rather than converted to success.
5. **Given** the workbench process exits and restarts, **When** prior UI metadata is restored, **Then** prior children are not claimed as durably owned or reattachable.

### User Story 2 - Read Terminal Work as Typed, Source-Labelled Interaction Blocks (Priority: P1)

A developer can distinguish user-entered commands, terminal output, Agent-reported material already known to Winds, deterministic verification results, warnings/policy decisions, and explicit human decisions without treating visual grouping as evidence authority.

**Independent Test**: Feed fixtures containing commands, normal/multiline output, ANSI sequences, evidence-like strings, Agent-like “verified” claims, and actual canonical evidence. Verify only accepted evidence receives Winds-observed/evidence treatment.

**Acceptance Scenarios**:

1. Winds-owned command input may be grouped with subsequently observed output for readability, but output text cannot imply verification/acceptance.
2. Terminal strings such as `PASS`, `verified`, `approved`, evidence-shaped JSON, or forged UI labels remain terminal output.
3. Canonical evidence views expose exact source/candidate identity and applicability.
4. Agent-reported results remain `AGENT_REPORTED` unless separately observed or human-decided.
5. Ambiguous command/output attribution falls back to truthful continuous output rather than invented grouping.

### User Story 3 - Enter and Edit Shell Commands Without Fighting the Terminal (Priority: P1)

A developer has a keyboard-first shell input experience supporting normal commands, Unicode, long lines, multiline editing/paste where semantics permit, history recall, and explicit dispatch to one selected pane without silent model/provider routing.

**Independent Test**: Exercise empty input, Unicode, long commands, multiline/bracketed-paste-sensitive content, history navigation, rapid editing, pane switching before dispatch, and shell metacharacters. Verify bytes reach only the explicit terminal target.

**Acceptance Scenarios**:

1. Submitted shell input is sent only to the selected owned terminal pane.
2. Focus changes before submission change the explicit target; input is never broadcast by default.
3. Multiline paste preserves explicit shell/paste semantics and is never silently executed line-by-line when meaning would change.
4. Prompt-like text remains shell input while shell mode is active; Spec 007 adds no silent natural-language router.
5. Unsupported editor behavior degrades to accepted terminal semantics without silent byte loss/reordering.

### User Story 4 - Navigate Tabs, Panes, Workspaces, and Winds Sessions Quickly (Priority: P1)

A developer can create, focus, split, resize, close, and find terminal panes; move among workspace/session contexts; and understand which canonical Winds session a surface belongs to without memorizing runtime-native IDs.

Topology is local to the current workbench lifetime. It does not imply a daemon or durable multiplexer owner.

**Independent Test**: Build fixtures with multiple workspaces, similarly named sessions, Unicode/case-colliding labels, and many panes. Exercise keyboard/pointer navigation, layout changes, ambiguous search, pane exit, and UI restart.

**Acceptance Scenarios**:

1. Split/resize/focus/reorder never changes canonical terminal/session/evidence identity.
2. Similar names yield deterministic disambiguated candidates rather than silent recency selection.
3. A pane associated with a Winds session uses stable session identity rather than display text.
4. Exited/history panes are distinguishable from live owned panes.
5. Restored layout metadata does not imply restored live ownership.

### User Story 5 - See Exact Candidate, Diff, Evidence, and Verification State Where Work Happens (Priority: P1)

A developer can inspect current Git/candidate context, diff/evidence, and repository-native verification state from the workbench without creating a second verification authority or automatic landing path.

**Independent Test**: Show candidate A with accepted evidence, move to candidate B, and verify A evidence becomes historical/not-applicable. Verify Agent/terminal “done” text never substitutes for repository verification.

**Acceptance Scenarios**:

1. Evidence views expose exact candidate binding and applicability.
2. Candidate movement visibly stales earlier candidate-bound evidence/review while preserving history.
3. Agent/terminal success claims never cause `VERIFIED` or `ACCEPTED` without accepted evidence.
4. Explicit UI invocation of existing `winds verify` preserves the existing verification authority rather than inventing a UI-local verifier.
5. Even after all gates pass, landing remains an explicit human/repository-governed action.
6. Code/diff inspection is non-mutating under Spec 007 and grants no new Agent path authority.

### User Story 6 - Search and Revisit Useful Terminal History Without Turning Transcript Into Memory Authority (Priority: P2)

A developer can search retained commands/output and revisit prior terminal context while canonical decisions/evidence remain separate from transcript text.

**Independent Test**: Populate history with repeated commands, Unicode, large output, forged evidence strings, secret-like fixtures, and multiple sessions. Verify deterministic search/navigation, source context, bounded retention, and no authority promotion.

**Acceptance Scenarios**:

1. Search matches identify workspace/session/pane/source context where known.
2. Evidence-like transcript text remains transcript data.
3. Retention truncation is explicit; the UI never implies complete transcript retention after eviction.
4. Transcript retention/eviction never rewrites canonical evidence, approvals, or human decisions.

### User Story 7 - Keep the Workbench Responsive Under Real Terminal Stress (Priority: P1)

A developer can use Winds under large output, rapid resize/focus changes, many panes, Unicode/ANSI-heavy streams, and child churn without hangs, unbounded growth, dropped identity, or false lifecycle/evidence state.

**Independent Test**: Run deterministic stress and benchmark campaigns defined by the Frozen Performance Budgets below plus cross-platform correctness guards.

**Acceptance Scenarios**:

1. High-volume output applies explicit bounded retention/backpressure and remains interactive.
2. Resize/focus races with output/child exit preserve lifecycle truth and do not deadlock.
3. Unicode/wide/combining content preserves input/output ordering and does not corrupt canonical identity/search state.
4. Malformed/unsupported control sequences cannot authorize host actions or crash the workbench.
5. Idle panes do not scale through avoidable busy polling.

### User Story 8 - Preserve Cross-Platform Truth (Priority: P1)

A developer sees behavior that matches what Winds has directly exercised on native Windows, WSL2, Linux, and macOS. Platform-specific limitations are visible rather than disguised as universal support.

**Independent Test**: Exercise claimed workbench behaviors on each claimed platform/domain. Keep native Windows/ConPTY, WSL2 host/guest, and Unix PTY evidence distinct.

**Acceptance Scenarios**:

1. Native Windows operations preserve ConPTY-specific ownership/lifecycle truth.
2. WSL2 preserves explicit Windows-host versus Linux-guest path/domain truth.
3. Linux/macOS PTY signals/resize/close are independently exercised.
4. Untested platform behavior is unavailable/experimental/not-claimed rather than inferred from another platform.

## Terminal Safety Edge Cases

- Child exits while input is edited or dispatched.
- Pane closes while output-reader work is in flight.
- Resize/focus churn races multiple children producing output.
- Shell executable/profile disappears or changes before launch.
- Working directory is deleted, moved, inaccessible, or belongs to a different workspace.
- Labels collide, differ only by case, or contain Unicode controls/combining characters.
- Output contains ANSI/OSC, forged hyperlinks/title changes, OSC 52 clipboard requests, device-control-like bytes, or strings imitating Winds UI/evidence.
- Output contains invalid UTF-8 or binary-like bytes.
- A hyperlink uses an external, local-file, or command-like scheme.
- Bracketed paste behavior differs across shells/platforms.
- History contains secret-like content; retention/search must not become an implicit credential store.
- Retention truncates transcript while canonical evidence persists separately.
- Candidate identity changes while evidence/diff is visible.
- Terminal/Agent says “verified” after candidate movement.
- UI metadata survives restart while child ownership cannot be proven.
- WSL path translation is ambiguous/unsupported.
- A visible path/context lies outside an Agent's approved authority.

## Functional Requirements

### Terminal ownership and workbench model

- **FR-001**: The accepted terminal child/session handle MUST remain the authority for live lifecycle operations; UI state MUST NOT replace it.
- **FR-002**: Winds MUST support multiple concurrent terminal panes in one workbench process, subject to Tasks-defined implementation bounds.
- **FR-003**: Every pane MUST have a transient pane identity distinct from workspace/workstream/Winds-session/runtime-native/process identity.
- **FR-004**: Workspace/session labels and pane titles MUST remain presentation-only.
- **FR-005**: Focus/layout changes MUST NOT mutate canonical work/session identity or evidence applicability.
- **FR-006**: Close/interrupt/terminate/resize MUST use accepted platform-specific lifecycle paths and preserve failure/ownership-loss truth.
- **FR-007**: UI/process restart MUST NOT be represented as durable live-child reattachment.
- **FR-008**: Spec 007 MUST preserve one-process architecture and MUST NOT require a daemon, persistent owner, socket, local control server, or IPC protocol.

### Typed interaction and source truth

- **FR-009**: The workbench MUST distinguish at minimum Winds-owned command input, terminal output, canonical verification/evidence, warnings/policy state, and human decisions where those sources exist.
- **FR-010**: Existing Agent-reported content MUST remain explicitly source-labelled and MUST NOT be presented as Winds-observed evidence merely because it is rendered structurally.
- **FR-011**: Terminal text MUST NOT self-promote to evidence, verification, acceptance, authorization, or human decision.
- **FR-012**: Command/output grouping MUST use Winds-observed boundaries; ambiguous attribution MUST fall back to truthful continuous presentation.
- **FR-013**: Terminal byte/order semantics required by the terminal substrate MUST be preserved even when decoding/rendering needs replacement-character fallback.
- **FR-014**: Canonical evidence views MUST expose source identity plus exact candidate/applicability information.

### Input/editor behavior

- **FR-015**: Primary input MUST be explicit shell input to one selected pane; Spec 007 MUST NOT silently route text to a model/provider.
- **FR-016**: Dispatch MUST target exactly one explicit pane by default and MUST NOT broadcast silently.
- **FR-017**: Input MUST support Unicode and commands longer than one viewport line without silent truncation.
- **FR-018**: Multiline paste/input MUST preserve explicit shell/paste semantics and MUST NOT silently execute separate lines when meaning changes.
- **FR-019**: History recall/editing MUST preserve submitted command text or explicitly identify normalization; history MUST NOT become canonical task memory.
- **FR-020**: Unsupported editor behavior MUST degrade truthfully to accepted terminal semantics rather than silently dropping/reordering bytes.

### Tabs, panes, sessions, and navigation

- **FR-021**: The workbench MUST provide create/focus/split/resize/close operations for its accepted pane topology without implying persistent multiplexer ownership.
- **FR-022**: Navigation MUST expose enough canonical workspace/session context to distinguish similar labels.
- **FR-023**: Pane-to-Winds-session association MUST use stable canonical session identity.
- **FR-024**: Workspace/session/pane search MUST be deterministic for identical canonical inputs and return explicit candidates for material ambiguity.
- **FR-025**: Exited/stopped/history panes MUST be distinguishable from live owned panes.
- **FR-026**: Restored UI metadata MUST NOT imply restored live process ownership.
- **FR-027**: Visibility MUST NOT grant an Agent/delegate new file, tool, execution, or context-transfer authority.

### Verification-native workbench

- **FR-028**: The UI MUST visibly distinguish `IDLE`, Agent-reported `DONE` where applicable, `VERIFIED`, and `ACCEPTED`.
- **FR-029**: Displayed verification/evidence MUST remain bound to the exact candidate identity of the existing evidence model.
- **FR-030**: Candidate movement MUST visibly invalidate applicability of earlier candidate-bound evidence/review without deleting history.
- **FR-031**: Command exit status alone MUST NOT establish `VERIFIED` or `ACCEPTED`.
- **FR-032**: Terminal/Agent prose claiming success MUST NOT establish `VERIFIED` or `ACCEPTED`.
- **FR-033**: Any UI action invoking accepted `winds verify` behavior MUST remain explicit and preserve existing verification semantics.
- **FR-034**: Code/diff/evidence inspection MUST be non-mutating under Spec 007 unless a later accepted task explicitly authorizes one narrow mutation.
- **FR-035**: Spec 007 MUST NOT automatically select a winner, merge, rebase, cherry-pick, push, create a PR, or land changes.

### History, search, and retention

- **FR-036**: History/search MUST preserve source/workspace/session/pane context where known and MUST NOT imply completeness after bounded retention.
- **FR-037**: Search results containing evidence-like text MUST remain transcript data.
- **FR-038**: Transcript retention MUST be explicit and bounded and MUST NOT mutate canonical evidence, approvals, or human decisions.
- **FR-039**: Spec 007 MUST NOT introduce semantic-memory, vector/RAG, embedding, or learned-retrieval architecture.

### Terminal escape and host-integration safety

- **FR-040**: Terminal control sequences MUST NOT directly grant file, clipboard, command, network, browser, or other host authority beyond the accepted rendering contract.
- **FR-041**: Clipboard-writing escape sequences such as OSC 52 MUST be disabled, explicit-consent gated, or equivalently bounded; they MUST NOT silently overwrite the host clipboard by default.
- **FR-042**: Hyperlinks/file references emitted by terminal content MUST require explicit user interaction before external/open action and remain subject to scheme/path/policy validation.
- **FR-043**: Malformed, truncated, oversized, unknown, or unsupported control sequences MUST fail safely and MUST NOT crash the workbench or create privileged host actions.
- **FR-044**: Terminal output MUST NOT be able to forge trusted Winds UI/source labels in a way that changes evidence/authority state.

### Frozen performance and boundedness budgets

The future Plan MUST pin reproducible reference hardware/VM images, OS versions, build profile, fixture shell, measurement tooling, warmup rules, and raw-result retention. Those details may make measurements reproducible; they MUST NOT weaken the following Spec-level product thresholds without an accepted spec amendment.

- **FR-045**: **Cold/input-ready startup** with one deterministic fixture shell MUST be `<= 1500 ms p95` across at least 20 measured runs on the Plan-defined reference environment, from Winds process start to an input-ready workbench with the first pane established. Provider/model/network time is not part of this metric.
- **FR-046**: **Workbench-only input-to-dispatch overhead** MUST be `<= 16 ms p95` across at least 1000 deterministic dispatch iterations, measured from accepted UI submission to write-dispatch handoff, excluding child-shell execution time.
- **FR-047**: **Pane focus/split/navigation state-update latency** MUST be `<= 16 ms p95` across at least 1000 operations against a deterministic 50-pane topology model; this benchmark may use inert fixture panes and does not claim 50 simultaneously live shells.
- **FR-048**: **Retained-history search latency** MUST be `<= 100 ms p95` across at least 200 representative searches over a deterministic retained corpus of at least 100,000 logical lines.
- **FR-049**: **High-volume output** MUST process at least 100,000 logical lines and at least 10 MiB of deterministic terminal payload without deadlock, unbounded growth, lifecycle corruption, or evidence corruption; during the campaign, explicit focus/navigation actions MUST remain `<= 100 ms p95` on the reference environment.
- **FR-050**: **Default per-pane retained transcript** MUST be bounded to no more than 100,000 logical lines and no more than 32 MiB of retained payload, whichever bound is reached first, unless the user explicitly chooses a separately bounded larger policy. Eviction MUST be visible and MUST NOT affect canonical evidence.
- **FR-051**: **Idle scaling** for ten live fixture panes producing no output over a 60-second measurement window MUST average `<= 2%` of one logical CPU core for Winds workbench-owned processing and `<= 256 MiB` Winds workbench RSS overhead excluding child-process memory on the Plan-defined reference environment.
- **FR-052**: **Resize-storm correctness** MUST survive at least 1000 accepted resize requests across ten fixture panes within a ten-second campaign without deadlock/crash, and each live pane's final terminal size MUST equal the final accepted size for that pane.
- **FR-053**: Performance measurements MUST be stored with exact commit/tree, platform, build profile, benchmark fixture identity, reference-environment identity, and measurement method; older-candidate measurements MUST NOT qualify a moved candidate.

### Platform truth and accessibility

- **FR-054**: Platform support claims MUST be limited to directly exercised native Windows/ConPTY, WSL2, Linux, and macOS domains as applicable.
- **FR-055**: Native Windows and WSL2 MUST remain distinct execution/path domains in UI state/evidence.
- **FR-056**: Platform-specific unavailable behavior MUST be represented truthfully rather than inferred from another platform.
- **FR-057**: Core create/focus/navigate/search/verify-inspect operations MUST be reachable through documented keyboard interactions; pointer interaction may be additive.
- **FR-058**: Focus, selected pane, exited/live state, warnings, evidence applicability, and verification state MUST NOT rely on color alone.
- **FR-059**: Text selection/copy behavior and wide/combining-character behavior MUST be explicitly tested before accessibility parity is claimed.
- **FR-060**: Plan/Tasks MUST include accessibility checks appropriate to the selected UI stack and claimed platform surfaces.

### Scope and governance boundaries

- **FR-061**: Spec 007 MUST NOT add a persistent owner, daemon, public/private IPC/control protocol, remote execution route, or mobile/remote continuation surface.
- **FR-062**: Spec 007 MUST NOT add browser automation/profile/CDP runtime, provider mesh/provider SDK/authentication brokerage, automatic model routing, or new provider execution authority.
- **FR-063**: Spec 007 MUST NOT add ACP dependency landing, MCP runtime, A2A, generic runtime/plugin framework, integration SDK, plugin host, or marketplace.
- **FR-064**: Spec 007 MUST NOT add verified-learning activation, skill promotion, experiment plane, model training/fine-tuning/RL, vector/RAG memory, or executable self-modification.
- **FR-065**: Spec 007 MUST preserve all applicable Spec 003 and Spec 006 Git/evidence/authority/recovery/platform/privacy invariants.
- **FR-066**: UI framework, renderer, terminal parser/rendering library, clipboard integration, and every new dependency remain Plan-stage decisions and require exact-version/license/MSRV/platform/provenance/YAGNI review before Tasks may authorize them.

## Success Criteria

- **SC-001**: At least 10 concurrent fixture terminal panes can be started, navigated, resized, and closed in one workbench process while preserving exact owned-child lifecycle truth.
- **SC-002**: Repeated display renames, duplicate/case/Unicode names, and layout changes produce zero canonical identity collisions/evidence-link movement.
- **SC-003**: Forged terminal `PASS`/`VERIFIED`/`ACCEPTED` strings, evidence-like JSON, and trusted-looking labels produce zero promotion into canonical evidence state.
- **SC-004**: Candidate A evidence becomes visibly not-applicable after movement to candidate B while remaining historically inspectable.
- **SC-005**: Shell input proves exactly-one-pane dispatch by default, no provider/model invocation, and no silent broadcast.
- **SC-006**: Multiline/Unicode/long-input fixtures preserve explicit semantics without silent truncation or line-by-line execution drift.
- **SC-007**: Cold/input-ready startup meets FR-045 on the pinned reference environment.
- **SC-008**: Input dispatch and 50-pane topology-model interaction meet FR-046 and FR-047.
- **SC-009**: 100,000-line history search and high-volume output meet FR-048 and FR-049.
- **SC-010**: Retention remains within FR-050 and visibly reports eviction without changing canonical evidence.
- **SC-011**: Ten-pane idle measurement and resize-storm campaign meet FR-051 and FR-052.
- **SC-012**: Malformed/unknown/oversized escape-sequence fixtures cannot crash the workbench or trigger unauthorized clipboard/file/browser/command host actions.
- **SC-013**: Core workbench operations have keyboard-reachable paths and critical state remains distinguishable without color-only encoding.
- **SC-014**: Native Windows/ConPTY, WSL2, Linux, and macOS claims are backed by direct evidence for each behavior claimed; no platform inherits proof solely from another.
- **SC-015**: A workbench restart test proves UI metadata restoration, if implemented, never claims durable live-child ownership.
- **SC-016**: Existing repository `quality` plus applicable Spec 003/006 terminal/verification regression gates remain green on the final exact implementation candidate.
- **SC-017**: Correctness/safety, Ponytail/YAGNI, and fresh independent review reach the final exact implementation candidate with zero unresolved material findings.
- **SC-018**: Final reconciliation truthfully proves that no daemon/IPC, remote execution, browser runtime, provider mesh implementation, ACP/MCP runtime, generic plugin system, learning subsystem, automatic winner, or automatic landing path was introduced.

## Explicit Non-Goals

Spec 007 does not authorize:

- durable background ownership or detach/reattach across Winds process exit;
- daemon/server/socket/IPC/control API architecture;
- remote execution, SSH control, mobile continuation, or cloud sync;
- Browser Twin, browser automation, browser credential/profile management, screenshots as verification evidence, or CDP;
- provider mesh, provider SDK integration, provider credential brokerage, silent model routing, or new live Codex/Claude acceptance claims;
- MCP runtime, ACP dependency landing, ACP v2, A2A, generic runtime adapters, or generic tool gateway;
- recursive/multi-worker fleets beyond accepted Spec 006 boundaries;
- verified-learning activation, skill mutation/promotion, experiment plane, learned routing, training/fine-tuning/RL;
- vector/embedding/RAG memory or transcript-as-canonical-memory architecture;
- plugin host, integration SDK, marketplace, or executable third-party extension supply chain;
- full IDE/source editor replacement;
- automatic candidate winner selection or automatic Git landing operations.

## Assumptions

- Existing Spec 003 terminal lifecycle and Spec 006 canonical workspace/session/evidence structures remain starting truth rather than being replaced.
- A UI/rendering dependency may be introduced only after the Plan proves necessity and passes exact dependency review.
- The binary may evolve into a richer interactive surface while remaining one process for Spec 007; process separation is not required.
- Persistence solely for non-live presentation metadata/history may be considered by the Plan but cannot create durable process-ownership claims.
- The Plan may tighten performance budgets. Any relaxation of FR-045 through FR-052 requires an accepted Spec 007 amendment rather than an implementation-only exception.

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
