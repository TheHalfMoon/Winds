# Implementation Plan: Winds Desktop Agentic Workspace

## Summary

Build the smallest premium desktop surface that satisfies canonical Spec 010 while preserving Winds' existing Rust authority, exact-candidate evidence model, terminal lifecycle truth, workflow/session identity, Model Mesh continuity semantics, and human landing boundary.

The first desktop implementation program will create a local-first, one-process desktop application where:

1. Projects organize canonical local work without replacing workspace/repository identity;
2. the left dock presents named, searchable sessions grouped under Projects;
3. each session visibly identifies its observed/requested runtime state, including Codex and Claude where qualified;
4. two sessions can be worked side-by-side as independent first-class surfaces with no implicit input or action broadcast;
5. the right dock provides Files, Changes, Evidence, Context, Artifacts, and Needs You views bound to the selected Project/session/candidate identity;
6. session work is presented as a calm typed work stream rather than an undifferentiated chat transcript;
7. the terminal remains a first-class surface but is not the product's organizing primitive;
8. Rust remains canonical authority and the renderer remains an untrusted presentation/client boundary;
9. stale asynchronous results cannot silently cross Project/session/worktree/candidate bindings;
10. the product is visually distinctive, keyboard-native, accessible, responsive, and measurably fast.

This Plan selects a Tauri 2 desktop shell with React 19.3 and Vite 8 for the renderer, xterm.js only for explicit terminal rendering, and a Winds-authored design system. It does not authorize dependencies or source changes until Tasks are canonically accepted.

## Constitution Check

The implementation MUST preserve:

```text
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
AGENT_COMPLETION_IS_NOT_VERIFICATION
VERIFICATION_IS_NOT_ACCEPTANCE
ACCEPTANCE_IS_NOT_LANDING
VERIFY_THE_EXACT_CANDIDATE
PROJECT_ALIAS != CANONICAL_WORKSPACE_IDENTITY
SESSION_ALIAS != CANONICAL_SESSION_IDENTITY
PRESENTATION_STATE != CANONICAL_AUTHORITY
RENDERER_CONTENT != TRUSTED_EVIDENCE
RUNTIME_ICON != RUNTIME_AUTHORITY
DUAL_SESSION_VISIBILITY != INPUT_BROADCAST_AUTHORITY
RIGHT_DOCK_BINDING != DATA_AUTHORITY
TERMINAL_OUTPUT != VERIFICATION_EVIDENCE
NO_MAGIC_WINNER
NO_SILENT_LANDING
```

The desktop shell MUST NOT create a second truth store for verification, runtime identity, workflow state, Model Mesh state, candidate acceptance, or Git landing authority.

## Canonical Baseline

Planning base:

```text
BASE=9190afdc9b1db93d69909c5d3acbce48690522f7
BASE_TREE=76a76b9fdc79ec70ab708e119eabe539b039da88
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=IN_QUALIFICATION
SPEC_010_TASKS_AUTHORIZED=NO
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
POST_SPEC_MERGE_QUALITY=34714992834 ATTEMPT_1 SUCCESS
```

Existing seams to reuse rather than duplicate include:

- `Store` / `winds.db` and accepted append-only durable records;
- canonical workspace and worktree identity;
- `WorkflowRunIdentity` and `StageRunIdentity`;
- Winds session/runtime binding semantics;
- `RuntimeKind::{Codex, Claude}` and runtime discovery/provenance;
- Model Mesh current target, continuity, blocker, reviewer, and provenance projections;
- accepted PTY/ConPTY terminal ownership and lifecycle paths;
- verification/candidate/evidence projections;
- Spec 007 workbench navigation, terminal safety, accessibility, and performance lessons;
- Spec 008 workflow status, why-blocked, decision, and reconstruction projections;
- Spec 009 explicit target and continuity truth.

## Product Architecture

### 1. One Rust authority, one desktop renderer

The first slice remains one local desktop process. Tauri provides the desktop host and a narrow application bridge. The existing Rust domain remains authoritative.

```text
┌────────────────────────────────────────────────────────────┐
│ Winds Desktop Renderer                                    │
│ React view state / focus / layout / ephemeral composition │
└───────────────────────┬────────────────────────────────────┘
                        │ narrow typed Tauri commands/events
                        ▼
┌────────────────────────────────────────────────────────────┐
│ Winds Rust Application Core                               │
│ workspace/session/runtime/workflow/model mesh/evidence    │
│ PTY ownership / Git authority / Store                     │
└────────────────────────────────────────────────────────────┘
```

Renderer state may cache presentation data, but canonical facts MUST be reconstructed from Rust-owned projections or bounded request snapshots.

No renderer string, DOM state, icon, localStorage value, route, query parameter, or JavaScript object may grant execution, verification, acceptance, landing, credential, or Git authority.

### 2. Desktop framework selection

Selected Plan candidate stack, subject to exact Tasks-stage dependency qualification:

```text
Rust desktop host: tauri 2.11.5
Tauri build helper: tauri-build 2.6.3
Tauri JS API: @tauri-apps/api 2.11.1
Tauri CLI: @tauri-apps/cli 2.11.4
Renderer: react 19.3.0 + react-dom 19.3.0
Bundler/dev server: vite 8.3.0
React integration: @vitejs/plugin-react 6.1.1
JavaScript toolchain: Node 22.22.3 + npm 10.9.8 + package-lock.json
Terminal renderer: @xterm/xterm 6.0.0
Terminal fit helper: @xterm/addon-fit 0.11.0
Icon library: lucide-react 1.45.0
```

Rationale:

- Tauri preserves Rust as the host authority and avoids shipping a full Chromium runtime with the application.
- React is selected for a highly interactive stateful desktop surface, current ecosystem maturity, accessibility tooling, and explicit View Transition support in 19.3.
- Vite keeps the renderer build small and fast and Vite 8 uses a unified Rolldown-based bundler.
- xterm.js is limited to the terminal view where a mature terminal renderer is justified; Winds does not implement another terminal emulator from scratch.
- Lucide provides one consistent icon grammar. Brand/runtime marks remain separately governed because generic UI icons and third-party trademarks are different concerns.

The Plan rejects Electron for the first implementation because the current requirements do not justify bundling Chromium/Node as an application runtime. It also rejects a custom GPU/native UI framework because Winds would need to rebuild text, accessibility, input, layout, IME, and component infrastructure unrelated to its differentiating product truth.

### 3. Dependency minimization

Do not add Redux, Zustand, MobX, TanStack Query, a router framework, a component mega-library, Tailwind, CSS-in-JS, animation framework, editor framework, or design-system package in the first slice unless a later concrete task proves necessity.

Use:

- npm and the committed `package-lock.json` for deterministic renderer dependency resolution; no second JavaScript package manager in the first program;
- React local state/context/reducer primitives for renderer-only state;
- CSS custom properties and authored CSS for design tokens;
- platform/browser APIs for focus, dialogs/popovers, selection, and reduced-motion behavior where suitable;
- Tauri invoke/event primitives only through a Winds-authored typed bridge facade;
- existing Rust/store projections rather than parallel renderer caches where canonical truth matters.

### 4. Project and Session model

The desktop Project is a presentation grouping over one explicit canonical workspace/repository binding.

```text
DesktopProjectView {
  project_view_id,
  display_name,
  canonical_workspace_id,
  canonical_repo_root,
  presentation_order,
  collapsed,
}
```

`display_name` is user-editable presentation state. It MUST NOT rewrite `canonical_workspace_id`, repository identity, worktree identity, Git identity, workflow identity, or evidence bindings.

A desktop Session is a presentation/work surface bound to a canonical Winds session/runtime/work scope.

```text
DesktopSessionView {
  session_view_id,
  display_name,
  canonical_session_id,
  project_view_id,
  runtime_identity_projection,
  workflow_binding,
  workspace_or_worktree_binding,
  activity_state,
  attention_state,
}
```

Renaming changes only `display_name`. Runtime badges are derived from source-labelled runtime identity projections. Unknown, unavailable, conflicting, or stale runtime identity gets an explicit neutral/error state instead of a guessed vendor icon.

### 5. Left Dock — Projects and Sessions

The left dock is the primary organizational navigation surface.

Required first-slice behavior:

- Projects group sessions;
- Project rows are collapsible;
- sessions are selectable and renameable;
- session rows show runtime mark, display name, concise activity state, and attention state;
- selected state is obvious without color alone;
- keyboard navigation can traverse Projects and Sessions;
- empty/new project and empty project session states teach the next action;
- list virtualization is not introduced until measured need exists;
- session count, attention count, and state rollups are derived rather than authoritative.

The left dock must remain visually quiet. It is navigation, not a dashboard.

### 6. Center Workspace — one or two first-class sessions

The center workspace supports exactly these first-program modes:

```text
SINGLE_SESSION
DUAL_SESSION
```

Dual Session is not generic recursive pane tiling in the first desktop program. It is a deliberate product primitive for comparing or working two sessions simultaneously.

Each side has its own canonical session binding, runtime identity header, work stream, composer/input target, terminal or focused work sub-surface, scroll position, selected work event, focus state, and right-dock targeting relationship.

Operations:

- choose left/right session;
- swap sides;
- resize divider;
- maximize one side temporarily;
- return to dual view;
- close one side back to single mode;
- focus and keyboard-cycle between sides.

There is no implicit broadcast command, prompt, key, terminal input, approval, file mutation, model selection, or Git action across both sides.

### 7. Codex-like session interaction, Winds-native semantics

The session surface aims for the directness and flow of a dedicated coding-agent application without copying another product's visual design.

A session presents a typed Work Stream:

```text
USER_PROMPT
AGENT_STATUS
AGENT_MESSAGE
TOOL_ACTION
FILE_READ
FILE_CHANGE
DIFF
COMMAND
COMMAND_OUTPUT
TEST_RESULT
VERIFICATION_RESULT
APPROVAL_REQUEST
HUMAN_DECISION
WARNING
ERROR
COMPLETION_REPORT
```

Every event retains source/provenance. Agent material never becomes Winds-observed evidence by presentation.

The composer is persistent at the bottom of the active session. It supports multiline input, explicit send, keyboard access, attachment/context affordances only when authorized, clear target identity, and fail-closed disabled state when the selected session cannot accept input.

No hidden chain-of-thought UI is required or claimed. Agent reasoning may be represented only through explicitly supplied summaries/events that the runtime exposes lawfully and that retain their source class.

### 8. Right Context Dock

The right dock is contextual, not permanently authoritative. First-program tabs:

```text
FILES
CHANGES
EVIDENCE
CONTEXT
ARTIFACTS
NEEDS_YOU
```

Dock target modes:

```text
PROJECT
ACTIVE_SESSION
LEFT_SESSION
RIGHT_SESSION
```

Every asynchronous right-dock request is created with an immutable `DesktopBindingSnapshotV1` containing all applicable material identity:

```text
DesktopBindingSnapshotV1 {
  project_view_id,
  canonical_workspace_id,
  canonical_session_id?,
  worktree_identity?,
  workflow_run_id?,
  stage_run_id?,
  candidate_oid?,
  candidate_tree?,
  runtime_binding_digest?,
}
```

Every response carries the snapshot that originated it. Before render/application, the renderer compares that snapshot with the current dock binding. A mismatch is discarded by default. A diagnostic/history surface may intentionally show the result only with explicit `STALE` state and its original binding visible.

This mechanism is mandatory for Files, Changes, Evidence, Context, Artifacts, and Needs You.

### 9. Files and Changes

The desktop does not become a full IDE in the first program.

Files provides bounded repository tree/navigation, open/reveal file identity, text preview for supported bounded files, explicit binary/large-file state, safe path presentation, and no silent file mutation from preview.

Changes provides repository-native Git diff/status projections bound to exact repository/candidate identity. Renderer syntax highlighting may improve presentation but never changes Git/evidence authority.

A full source editor, LSP stack, language server manager, extension marketplace, or IDE project model is explicitly deferred.

### 10. Evidence and Context

Evidence surfaces expose existing Winds-native verification and provenance state. They must visually differentiate:

```text
AGENT_REPORTED
WINDS_OBSERVED
HUMAN_DECIDED
UNKNOWN
STALE
UNAVAILABLE
CONFLICTING
```

Context shows structured session/workflow/model-mesh context and transfer/reconstruction truth without exposing raw secrets or inventing provider-private state.

### 11. Needs You

`Needs You` is an attention projection, not an autonomous approval system.

It aggregates only actionable human attention states already supported by canonical authority, such as explicit approval requests, clarification requests, blocked workflow decisions, verification failures needing human review, conflict/ambiguity requiring selection, and separately authorized human acceptance/landing decisions.

An item includes source, canonical scope, exact reason, and available actions. An action is enabled only when the Rust core proves the corresponding authority and content binding.

### 12. Terminal surface

The terminal remains first-class but subordinate to the session/work context.

Use the existing Rust PTY/ConPTY ownership path. xterm.js receives terminal bytes/events for rendering and sends explicit input/resize requests back through the bounded bridge.

The renderer MUST NOT infer process ownership from visible terminal output; treat `PASS`, `DONE`, `verified`, JSON, ANSI titles, OSC content, or agent text as evidence; permit OSC 52 or arbitrary host actions to bypass existing policy; claim durable cross-restart ownership; or create a second PTY implementation.

### 13. Typed desktop bridge

Create a narrow Winds-authored Rust/TypeScript contract rather than calling arbitrary Rust commands from components.

Conceptual namespaces:

```text
desktop.project.*
desktop.session.*
desktop.workspace.*
desktop.right_dock.*
desktop.terminal.*
desktop.workflow.*
desktop.model_mesh.*
desktop.verification.*
```

Each command has versioned request/response types where compatibility matters, explicit canonical binding inputs, bounded payload sizes, source/provenance fields where truth class matters, deterministic errors, and no raw SQL/filesystem/shell/Git passthrough from renderer to Rust.

Tauri's transport is an implementation mechanism, not public Winds IPC and not external API authority.

### 14. Desktop persistence

Persist only presentation state necessary for product continuity, such as Project display name/order/collapse state, Session alias/order/pin/archive state if Tasks admit those fields, last selected Project/session, single/dual layout and divider ratio, right-dock target/tab/width, and theme/accessibility preferences.

Canonical runtime/workflow/evidence identity remains in existing Winds stores.

The preferred first-program persistence location is Winds-owned SQLite using a dedicated migration only if Tasks prove persistence is necessary and define exact schema ownership. Do not use localStorage as canonical persistence. Ephemeral renderer preferences may use in-memory state until persistence is authorized.

### 15. Visual System — Winds, not a clone

Create `PRODUCT.md`, `DESIGN.md`, and surface briefs only after Tasks authorize design artifact paths. The visual system follows Impeccable's Operate-mode discipline as a design/process reference, pinned to repository commit `cb56ed6c19a07329a9fa0cd4e657bee040156593`, without importing its runtime code or visual assets.

The product character:

- calm, dark-first but fully light-capable;
- high-information but not cramped;
- restrained accent usage;
- strong typographic hierarchy without decorative display type in controls;
- one coherent icon family;
- quiet surfaces with depth used sparingly;
- clear focus and selected states;
- motion communicates state only;
- no gradient text, decorative glass, neon terminal clichés, fake AI glow, card-grid dashboard aesthetic, or oversized chat bubbles;
- terminal/code use monospace; navigation, labels, work events, and actions use the primary UI sans;
- status meaning never relies on color alone.

The visual world should feel like a precision instrument for parallel intelligence: composed, fast, legible, and confident.

### 16. Motion

Default transitions should normally fall within 150-250ms and use motion only for dock state, session selection/focus, single-to-dual transitions, work-event state changes, Needs You state, and contextual reveal where spatial continuity helps.

Respect `prefers-reduced-motion` and provide equivalent non-motion state cues.

React 19.3 View Transitions may be used only where platform/WebView support is directly qualified and fallback remains correct.

### 17. Runtime marks and icons

Generic interface icons use one consistent library. Runtime identity marks are separate presentation.

First program MUST support visually distinct states for:

```text
CODEX
CLAUDE
SHELL
UNKNOWN
UNAVAILABLE
CONFLICTING
STALE
```

Tasks must decide whether vendor trademark/logo assets are legally and technically appropriate. Until that is proven, Winds-authored runtime glyphs/monograms may identify observed runtime families without pretending to be official vendor branding.

A runtime mark is never sufficient evidence of runtime identity; the underlying source-labelled projection remains authoritative.

### 18. Accessibility

Required design/implementation gates include WCAG 2.2 AA contrast, keyboard access to all core actions, visible focus, semantic labels for icon-only actions, semantic selected/current states, no color-only status, reduced-motion support, zoom/scaling without clipping critical actions, screen-reader meaningful Project/Session/runtime/status structure, text selection and caret visibility, and platform-native IME behavior for composer and terminal where supported.

### 19. Performance budgets

Tasks must freeze reproducible environments before qualification. This Plan proposes first-program engineering budgets:

```text
cold app launch to useful shell: <= 1500 ms p95 on reference environment
project/session selection visual response: <= 50 ms p95
single ↔ dual layout interaction: <= 100 ms p95 excluding first data fetch
composer keystroke-to-paint overhead: <= 16 ms p95
right-dock cached tab switch: <= 50 ms p95
right-dock stale-binding rejection: deterministic before render
idle desktop CPU: <= 2% of one logical core on reference environment
renderer+host idle RSS target: <= 300 MiB excluding child agents/terminals
```

Do not relax correctness, stale-binding checks, provenance, accessibility, or security to meet latency budgets.

### 20. Responsive behavior

Desktop is the product target, not mobile web.

Layout classes:

- wide desktop: left dock + dual center + right dock;
- standard laptop: collapsible right dock and resizable left dock;
- narrow desktop: single-session center with temporary contextual dock and explicit dual-mode entry only when minimum widths allow.

Dual view must fail visibly or offer a sensible fallback when minimum usable width cannot be satisfied; it must not compress two sessions into unreadable columns.

### 21. Security and trust boundary

The WebView/renderer is not trusted with arbitrary host capability.

Tasks must explicitly qualify Tauri capabilities/permissions and allow only the minimum commands required by the active slice.

Prohibit arbitrary shell execution from renderer strings, arbitrary filesystem APIs, generic command dispatchers, arbitrary Git passthrough, raw database queries, secret enumeration, credential storage in renderer/localStorage, remote navigation inside privileged app origin, runtime-downloaded executable code/plugins, and public HTTP/WebSocket/IPC control servers.

### 22. Packaging and platforms

Initial implementation targets macOS, Linux, and native Windows. Windows/ConPTY behavior remains distinctly qualified. WSL2 remains a distinct execution domain rather than a separate desktop host target.

Desktop packaging/signing/notarization/release installers are later task slices. A development build does not imply signed distribution readiness.

## Design and Research Inputs

Non-authoritative references:

- canonical Winds Spec 007 Workbench implementation and evidence;
- `docs/research/011-herdr-parity-and-beyond-roadmap.md`;
- `docs/research/012-agentic-era-terminal-north-star.md`;
- Herdr current public workspace/pane/agent-attention behavior as parity research;
- current OpenAI Codex desktop/app interaction patterns as session-flow research;
- `pbakaus/impeccable` commit `cb56ed6c19a07329a9fa0cd4e657bee040156593` as design-process/craft reference.

These references set quality floors and research input. They do not override Winds requirements and do not authorize cloning visual identity, proprietary assets, private behavior, or authority semantics.

## Rejected First-Program Architecture

Explicitly reject unless a later accepted amendment proves necessity:

- Electron;
- a durable `windsd` owner;
- public HTTP/WebSocket/local-socket API;
- generic plugin/runtime marketplace;
- browser runtime or embedded arbitrary web browsing;
- remote/mobile clients;
- recursive terminal multiplexer as the primary desktop IA;
- full IDE/editor/LSP implementation;
- provider/model gateway;
- automatic routing or winner selection;
- automatic Git landing;
- server-side renderer;
- cloud control plane;
- generic event-sourcing rewrite of existing Winds stores;
- second verification/evidence store;
- copying Codex/Herdr/Impeccable UI implementation or brand assets.

## Proposed Implementation Sequence

Tasks should decompose this Plan into narrow, dependency-ordered slices approximately as follows:

1. dependency/provenance/lock qualification for exact desktop stack;
2. desktop Rust shell + inert React frame with zero new product authority;
3. Winds design tokens and app shell geometry;
4. read-only Project/Session navigation projection;
5. session rename/presentation persistence with canonical identity invariants;
6. runtime marks and source-labelled status;
7. single-session Work Stream + composer shell;
8. dual-session workspace and independent input/focus semantics;
9. Files right-dock projection with immutable binding snapshots;
10. Changes/Evidence/Context/Artifacts dock projections;
11. Needs You attention projection and bounded canonical actions;
12. terminal rendering over existing PTY ownership using xterm.js;
13. keyboard/command/navigation polish and responsive layouts;
14. accessibility, theming, motion, empty/error/loading states;
15. performance and stress qualification;
16. platform packaging/development-host qualification;
17. adversarial renderer/bridge/stale-binding campaign;
18. final Spec 010 reconciliation and closeout.

Tasks remain free to split these further when a smaller reviewable seam improves evidence quality.

## Plan Acceptance Gate

The exact final Plan candidate may land only if:

- it changes planning/design documentation only;
- canonical Spec 010 entry and formal specification are closed;
- every selected dependency is exact-versioned as a Plan candidate and Tasks are required to audit license/MSRV/platform/source/checksum/graph before adoption;
- the architecture preserves one-process local authority and does not smuggle in a daemon/public IPC surface;
- Rust authority and renderer trust boundaries are explicit;
- Project/session alias semantics cannot rewrite canonical identity;
- dual-session semantics contain no implicit broadcast;
- asynchronous dock results are identity-bound and stale-safe;
- terminal rendering reuses existing PTY authority;
- Codex/Herdr/Impeccable remain non-authoritative references;
- visual direction is distinct and anti-clone constraints are explicit;
- repository quality passes on the exact final head;
- author correctness/safety/architecture review passes;
- Ponytail/YAGNI review finds no unjustified framework, abstraction, daemon, protocol, store, editor, or dependency;
- fresh independent substantive review finds zero unresolved material defects;
- exact base/head/tree/scope/ruleset/mergeability is reconciled before guarded landing;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical Plan landing may repository truth state:

```text
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS_AUTHORIZED=YES
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
```
