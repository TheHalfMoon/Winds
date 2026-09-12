# Feature Specification: Winds Desktop Agentic Workspace

**Feature Branch**: `spec/010-desktop-agentic-workspace`
**Created**: 2026-09-12
**Status**: Specification candidate only. Plan, Tasks, dependencies, desktop-framework selection, implementation, daemon/IPC, browser runtime, remote execution, and product-source changes are NOT authorized by this file alone.
**Input**: Founder directive to make Winds the most beautiful, professional, future-facing interface for the agentic era, with project-organized sessions in the left dock, two independent sessions side-by-side in one page, contextual files/evidence surfaces in the right dock, renameable sessions, and runtime/provider identity such as Claude or Codex visible in each session; the interaction should feel as direct and agent-native as modern Codex while remaining visibly and behaviorally Winds.

## Product Thesis

Winds Desktop is not a terminal emulator with extra chrome, not an editor clone, and not a chat application with terminals attached.

It is a **local agentic workspace** that makes four things simultaneously legible:

1. what the human is trying to accomplish;
2. which sessions and agents are doing work;
3. what files/artifacts/candidates that work affects;
4. what Winds actually knows, verifies, or still cannot prove.

The desktop should feel calm, exact, fast, and premium while preserving the evidence and authority model that differentiates Winds from ordinary agent consoles.

```text
DESKTOP_PRESENTATION != CANONICAL_AUTHORITY
SESSION_DISPLAY_NAME != SESSION_IDENTITY
PROJECT_DISPLAY_NAME != WORKSPACE_IDENTITY
REQUESTED_RUNTIME != OBSERVED_RUNTIME
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
DONE != VERIFIED != ACCEPTED != LANDED
PANE_LAYOUT != PROCESS_OWNERSHIP
DUAL_SESSION_VIEW != SHARED_INPUT
VISUAL_PROXIMITY != DELEGATION_AUTHORITY
```

## Canonical Baseline

Spec 010 begins only after the canonical entry gate landed as:

```text
ENTRY_MERGE=b653318236e885fcfd8de684afe5d733862fbc96
ENTRY_TREE=f639136dfd8bdb67ccff4bca8ee755a5199f7a09
POST_ENTRY_QUALITY=34713792141 SUCCESS ATTEMPT_1
SPEC_010_FORMAL_SPEC_AUTHORIZED=YES
SPEC_010_PLAN_AUTHORIZED=NO
SPEC_010_TASKS_AUTHORIZED=NO
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
DURABLE_LOCAL_RUNTIME_OWNER_AUTHORIZED=NO
DAEMON_IPC_AUTHORIZED=NO
```

Inherited live-runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

A richer interface MUST NOT visually promote those nonclaims into proof.

---

## Design and Interaction References — Non-Authoritative

Spec 010 uses external product/design research as a quality bar, never as copied product authority.

- OpenAI Codex desktop is a current interaction reference for project-organized agent threads, parallel agent work, direct prompting, long-running task supervision, and a focused agent command-center mental model. Winds MUST NOT copy proprietary visual assets, exact layouts, trade dress, or private implementation.
- `pbakaus/impeccable` at commit `cb56ed6c19a07329a9fa0cd4e657bee040156593` (Apache-2.0) is a design-process reference. Its `Operate` mode and craft-floor principles are appropriate to Winds Desktop: task-first scanability, restrained product color, complete interaction states, purposeful short motion, accessibility, and bounded visual QA. This Spec copies no Impeccable code or assets.
- Herdr remains a parity/research reference for terminal workspace organization, split panes, agent awareness, and attention rollups. Winds' target is broader: Project/Session organization, dual-session agent work, canonical truth/evidence, and contextual non-terminal work surfaces.

The intended feeling is:

```text
CODEX_DIRECTNESS + HERDR_PARALLELISM + WINDS_TRUTH + WINDS_VISUAL_IDENTITY
```

This is an interaction-quality target, not a cloning instruction.

## Product Model

### Project

A **Project** is the primary human organization unit in Winds Desktop.

For the first implementation program, a Project MUST be anchored to exactly one canonical Winds workspace/repository identity. A human-readable project name, icon, ordering, pinned state, and local UI preferences are presentation metadata only.

A Project may contain many Sessions.

A Project name MUST NOT replace, mutate, or become interchangeable with canonical `workspace_id`, repository root, Git identity, workstream identity, or verification identity.

### Session

A **Session** is the user-visible unit for one bounded stream of work. A session may expose a shell/terminal, an admitted agent runtime, a workflow/stage context, or a read-only historical state according to later Plan/Tasks authority.

Every Session MUST retain a stable canonical identity independent of its display name.

A user may rename a Session at any time. Renaming changes only the remembered display alias and MUST NOT alter runtime identity, native-session identity, Winds session identity, workflow/stage identity, candidate identity, evidence identity, ownership, or authority.

### Session Slot

A **Session Slot** is one visual position in the central desktop work surface. Slots are presentation only.

The initial product MUST support:

- single-session focus mode;
- dual-session mode with exactly two simultaneously visible session slots;
- independent focus and input ownership per slot;
- resize, swap, replace, and maximize-one-session interactions.

Placing two sessions next to each other MUST NOT create delegation, synchronization, shared prompt/input, shared terminal ownership, shared worktree identity, or shared authority.

### Runtime Identity Badge

Every admitted agent session MUST expose a visible runtime identity badge in the session list and session header.

The badge MUST include a non-color-only icon/glyph plus a text-accessible runtime label such as `Claude` or `Codex` when that runtime identity is legitimately known at the applicable proof level.

Requested runtime and locally observed runtime MUST remain distinguishable. A requested Claude/Codex launch may be represented as requested intent before observation, but the interface MUST NOT imply local observation until Winds has accepted evidence for it.

Unknown, unavailable, mismatched, or stale runtime identity MUST have an explicit visible state rather than borrowing the requested brand identity as proof.

---

## Canonical Desktop Layout

The default wide-screen layout SHOULD follow this semantic structure without requiring pixel-identical implementation:

```text
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│  Winds   Project / Workstream / Candidate                              Command / Status  │
├───────────────────────┬──────────────────────────────────────────────┬───────────────────┤
│ LEFT DOCK             │ CENTER WORK SURFACE                          │ RIGHT DOCK        │
│                       │                                              │                   │
│ Project A             │ ┌────────────────────┬────────────────────┐  │ Files             │
│   + New session       │ │ Session A          │ Session B          │  │ Changes           │
│   ◆ Claude · API      │ │ ◆ Claude           │ ◇ Codex            │  │ Evidence          │
│   ◇ Codex · Tests     │ │                    │                    │  │ Context           │
│   > Shell · Server    │ │ independent work   │ independent work   │  │ Artifacts         │
│                       │ │ surface            │ surface            │  │ Needs You         │
│ Project B             │ │                    │                    │  │                   │
│   ◆ Claude · Review   │ │                    │                    │  │ follows selected  │
│                       │ └────────────────────┴────────────────────┘  │ session/project   │
│ Needs You             │                                              │                   │
└───────────────────────┴──────────────────────────────────────────────┴───────────────────┘
```

The interface MAY collapse to one center session, compact docks, command-surface navigation, or smaller-device layouts as long as semantic identity and reachability remain intact.

---

# User Scenarios & Testing

## User Story 1 — Organize Work by Project and Remembered Sessions (Priority: P1)

A developer opens Winds and immediately sees projects in the left dock. Expanding a project shows its sessions with memorable names, runtime icons, current state, and enough workspace/worktree context to choose correctly.

The developer can create a new session inside a project, rename an existing session to something memorable such as `API auth repair`, pin or reorder relevant sessions where later Plan/Tasks authorize that presentation behavior, and return later without losing the display organization.

**Why this priority**: agentic work creates many concurrent contexts. If the human must remember raw terminal titles, native session IDs, paths, or pane numbers, the interface has failed.

**Independent Test**: Create at least three projects with multiple similarly named sessions, rename sessions repeatedly, restart the UI where persistence is implemented, and prove project/session organization remains deterministic while canonical identities remain unchanged.

**Acceptance Scenarios**:

1. **Given** a canonical workspace and no desktop project metadata, **When** it is opened in Winds Desktop, **Then** the UI can represent one Project anchored to that workspace without inventing a second canonical workspace identity.
2. **Given** a Project with several Sessions, **When** the user renames `Session 7` to `API auth repair`, **Then** every canonical session/runtime/workflow/candidate identity remains byte-for-byte unchanged.
3. **Given** two sessions with identical display aliases, **When** the user selects one, **Then** the UI exposes sufficient canonical context to avoid ambiguous action.
4. **Given** a session requiring human attention, **When** its Project is collapsed, **Then** the left dock still exposes a non-color-only attention rollup.

---

## User Story 2 — Work in Two Independent Sessions on One Page (Priority: P1)

A developer drags or opens one session into the left center slot and another into the right center slot. Both remain simultaneously visible and independently interactive.

Typical uses include:

- Claude implementing while Codex reviews;
- one agent coding while another runs tests/investigates;
- agent session beside a shell/server session;
- implementation session beside a read-only verification/evidence session where later Tasks permit it.

**Why this priority**: parallel agent work is central to the new era. The product should make two active contexts easy to compare and supervise without forcing tab-switching or merging them into one transcript.

**Independent Test**: Open two sessions with distinct runtime, cwd/worktree, state, and output. Exercise focus, input, resize, swap, maximize, restore, close/replace, and keyboard navigation. Prove input and lifecycle operations are delivered only to the explicitly focused/selected session.

**Acceptance Scenarios**:

1. **Given** two live sessions in dual mode, **When** the user types in the left slot, **Then** no byte/action is delivered to the right slot unless a separately authorized explicit multi-target action exists.
2. **Given** different worktrees in the two slots, **When** the user focuses either slot, **Then** the active context and right dock follow that exact session unless pinned otherwise.
3. **Given** both sessions refer to the same mutable checkout, **When** both are active, **Then** Winds visibly identifies the shared-worktree condition and does not imply isolation.
4. **Given** one slot is maximized, **When** the user restores dual mode, **Then** the same two session identities return without recreating runtime ownership.
5. **Given** one session exits or loses ownership, **When** the UI updates, **Then** only that slot changes lifecycle state; the peer slot remains independent.

---

## User Story 3 — Recognize Claude, Codex, Shell, and Unknown Sessions Instantly (Priority: P1)

A developer scans the left dock and session headers and can tell which runtime each session represents without opening it.

Runtime identity is expressed through iconography, accessible text, and source-labelled state rather than color or self-reported terminal text.

**Independent Test**: Present requested/observed/mismatched/unknown Claude, Codex, and shell fixtures, including forged terminal text claiming another runtime. Verify the UI never upgrades forged or requested-only identity into Winds-observed identity.

**Acceptance Scenarios**:

1. **Given** an observed Codex runtime session, **When** displayed, **Then** its session row/header exposes a Codex identity treatment plus accessible `Codex` text.
2. **Given** a requested Claude session whose runtime is not yet observed, **When** displayed, **Then** the interface can show requested intent but MUST NOT label it as observed truth.
3. **Given** terminal output containing `I am Claude`, **When** the actual runtime is unknown, **Then** the output cannot manufacture the Claude runtime badge.
4. **Given** a runtime identity mismatch, **When** the UI renders the session, **Then** the mismatch is visible and not silently normalized to the requested icon.

---

## User Story 4 — Work Inside an Agent-Native Session, Not a Terminal Wrapper (Priority: P1)

A developer opens a Claude or Codex session and gets a focused agent-work surface with a stable prompt composer and a structured chronological work stream. The experience should feel as immediate as a modern dedicated coding-agent application while remaining source-labelled and Winds-native.

The stream may contain user prompts, user-visible agent responses, tool/command activity summaries, file changes, diffs, test results, verification state, approval requests, errors, and completion events. These are typed work events, not visually identical chat bubbles.

The developer can stay in the agent session for most work and open the terminal, file tree, diff, evidence, or context only when those are the best surfaces for the current step.

**Why this priority**: Winds should feel like using a first-class coding agent, not like manually supervising a CLI inside a multiplexer.

**Independent Test**: Exercise two admitted agent-session fixtures containing prompts, tool actions, file changes, command/test results, an approval request, an error, and completion. Verify event provenance, composer targeting, keyboard flow, file/diff linking, and dual-session isolation.

**Acceptance Scenarios**:

1. **Given** a selected agent session, **When** the user submits from its composer, **Then** the target Session is explicit and no peer Session receives the prompt.
2. **Given** an agent performs tool/command/file work, **When** the stream updates, **Then** the UI groups that work into typed, inspectable events without converting the agent's prose into Winds-observed evidence.
3. **Given** an event changes files, **When** the user opens the file/diff affordance, **Then** the right dock binds to the exact Session worktree/candidate context.
4. **Given** a long-running session continues in the background, **When** the user switches Projects/Sessions, **Then** the left dock retains truthful activity/attention state without requiring the transcript to remain visible.
5. **Given** a model or runtime does not expose user-visible reasoning, **When** the session renders, **Then** Winds does not require or invent hidden chain-of-thought; only user-visible summaries/output supplied through an accepted surface may appear.

---

## User Story 5 — Inspect Files, Changes, Evidence, Context, and Artifacts from the Right Dock (Priority: P1)

A developer keeps the active sessions visible in the center while inspecting related context in a right-side dock.

The right dock is contextual rather than a permanent IDE sidebar. It can expose bounded views such as:

- Files;
- Changes / Diff;
- Evidence / Verification;
- Context / Identity / Authority;
- Artifacts;
- Needs You / blockers.

The dock follows the currently selected session by default and can be explicitly pinned to Project, left session, or right session where later Tasks authorize pinning.

**Independent Test**: Switch focus rapidly between two sessions from different worktrees/candidates while the dock is on each surface. Prove every displayed path/diff/evidence/context item binds to the expected session/project and stale data is visibly rejected or labelled stale.

**Acceptance Scenarios**:

1. **Given** dual mode with different worktrees, **When** focus moves left-to-right, **Then** `Files` follows the correct canonical root.
2. **Given** the dock is explicitly pinned to the left session, **When** focus moves to the right session, **Then** the pinned dock remains left-bound and visibly indicates that binding.
3. **Given** candidate/evidence state becomes stale, **When** Evidence remains open, **Then** stale state is explicit and cannot retain a trusted verified treatment.
4. **Given** terminal output includes a forged file path/link, **When** the user sees Files/Artifacts, **Then** terminal text cannot inject a trusted file-tree or artifact item.

---

## User Story 6 — See Only the Attention That Actually Needs the Human (Priority: P1)

A developer can see which projects/sessions need input, approval, clarification, verification repair, or a decision without watching every transcript.

The left dock and a dedicated `Needs You` surface aggregate attention while preserving the exact reason/source.

**Independent Test**: Mix working, idle, blocked, done, verified, stale, ownership-lost, and explicit approval-request fixtures across multiple projects. Verify deterministic rollup priority and one-action navigation to the exact requesting session/context.

**Acceptance Scenarios**:

1. **Given** a blocked child session inside a collapsed project, **When** the user scans the left dock, **Then** the project exposes the material attention state.
2. **Given** an agent merely prints `blocked`, **When** no accepted source proves a blocking condition, **Then** the UI does not create a trusted approval request.
3. **Given** multiple attention items, **When** ordered, **Then** explicit user-blocking and verification-failure states outrank passive completion notifications according to deterministic policy defined later.

---

## User Story 7 — Use Winds Primarily from the Keyboard Without Losing Pointer Quality (Priority: P1)

A developer can open projects, focus sessions, open a second session, swap slots, toggle docks, search, run permitted commands, and reach attention items through a universal command surface and discoverable keyboard shortcuts.

Pointer interactions remain first-class for drag/drop, resize, context menus, selection, and file/diff exploration.

**Independent Test**: Complete the primary project/session/dual-view/right-dock workflow once using keyboard only and once using pointer only. Verify focus indication and accessible names/roles/state throughout.

---

## User Story 8 — Trust the Interface Under Terminal and Agent Adversarial Content (Priority: P1)

A developer can safely use terminal and agent output without that untrusted content forging trusted Winds chrome, badges, evidence, links, approvals, or host actions.

**Independent Test**: Feed ANSI/OSC, oversized output, fake runtime badges, fake verification cards, malicious URLs, path-like text, clipboard requests, Unicode-confusable session names, and evidence-shaped JSON. Verify trusted desktop surfaces remain source-separated and no privileged host action occurs implicitly.

---

## User Story 9 — Experience a Distinctive, Premium Winds Visual System (Priority: P1)

A developer should recognize Winds immediately from typography, spacing, material hierarchy, motion, iconography, density, and information architecture without relying on copied assets or default framework components.

The interface must support dark and light appearance and must remain excellent in reduced-motion, high-contrast, scaled-text, and compact-density conditions.

Aesthetic acceptance requires explicit human visual review in addition to deterministic checks.

**Independent Test**: Evaluate canonical desktop fixtures across the required appearance/accessibility modes and at defined compact/standard/large viewport sizes. Verify semantic state remains readable and layout remains usable without color-only encoding or clipped primary controls.

---

## User Story 10 — Preserve Winds Truth While Showing Rich Workflow and Model Context (Priority: P2)

A developer can inspect current workflow/stage, runtime/provider/model target, continuity state, candidate/tree, evidence, and authority without the UI creating a competing source of truth.

The interface may summarize canonical state but must always be able to reveal source/provenance and why a claim is trusted, unknown, unavailable, stale, or blocked.

---

## User Story 11 — Scale from One Session to Many Without Becoming Noisy (Priority: P2)

A developer with many projects and sessions can search, filter, collapse, pin, reorder, and navigate without the left dock becoming a scrolling transcript index.

The Plan/Tasks may introduce virtualization or other presentation mechanisms only if measured need proves them necessary.

---

# Functional Requirements

## Project and Session Identity

- **FR-001**: Winds Desktop MUST provide a Project presentation unit anchored to exactly one canonical Winds workspace/repository identity in the first implementation program.
- **FR-002**: Project display name, icon, order, expanded/collapsed state, and similar UI preferences MUST NOT replace canonical workspace/repository identity.
- **FR-003**: A Project MUST support multiple user-visible Sessions.
- **FR-004**: Every Session MUST preserve a stable canonical Winds session identity or an explicit read-only/non-runtime identity class where applicable.
- **FR-005**: Session display alias MUST be independently renameable without mutating canonical session/runtime/native-session/workflow/stage/candidate/evidence identity.
- **FR-006**: Duplicate Session aliases MUST be permitted only if the UI exposes sufficient disambiguating canonical context before consequential action.
- **FR-007**: Project and Session deletion/closure semantics MUST distinguish presentation removal, runtime termination, and canonical-history retention; one MUST NOT silently imply another.
- **FR-008**: Restored presentation metadata MUST NOT imply restored live process ownership.
- **FR-009**: Session/project ordering or pinning MUST remain presentation state and MUST NOT affect execution, routing, verification, acceptance, or landing authority.
- **FR-010**: UI-only metadata MUST remain removable/rebuildable without destroying canonical Winds work/evidence truth.

## Left Dock — Projects and Sessions

- **FR-011**: The default desktop MUST expose a left dock whose primary hierarchy is Project -> Sessions.
- **FR-012**: Each Session row MUST expose at minimum display alias, runtime/session kind treatment, lifecycle/attention state, and enough workspace/worktree context to avoid material ambiguity.
- **FR-013**: Each Project row MUST expose material descendant attention state even while collapsed.
- **FR-014**: Users MUST be able to create a new Session from the target Project context through an explicit action.
- **FR-015**: Users MUST be able to rename a Session in-place or through an equally direct interaction.
- **FR-016**: Left-dock search/filter MUST deterministically identify materially ambiguous matches rather than selecting by recency alone for consequential actions.
- **FR-017**: The left dock MUST support keyboard and pointer focus/navigation.
- **FR-018**: Active, background, blocked/needs-attention, completed/unseen, idle/seen, exited, ownership-lost, stale, unavailable, and unknown states MUST remain visually distinguishable where those states exist canonically.
- **FR-019**: State MUST NOT be communicated by color alone.
- **FR-020**: A session row MUST NOT be allowed to render untrusted terminal/agent text as trusted Winds badges/chrome.
- **FR-021**: Project/session count scaling behavior MUST preserve stable focus and selection when rows virtualize, collapse, reorder, or update asynchronously.
- **FR-022**: The left dock MUST be collapsible without making Project/Session navigation inaccessible; the universal command surface MUST provide an alternate path.

## Dual Session Work Surface

- **FR-023**: Winds Desktop MUST provide a single-session focus mode and a first-class dual-session mode with exactly two concurrently visible Session Slots in the first implementation program.
- **FR-024**: The two Session Slots MUST retain independent canonical session binding.
- **FR-025**: Keyboard/pointer input MUST be delivered only to the explicitly focused/targeted Session Slot unless a separately specified multi-target action is explicitly invoked.
- **FR-026**: Dual-session mode MUST NOT implement implicit input broadcasting.
- **FR-027**: Users MUST be able to replace either slot without recreating or renaming the other session.
- **FR-028**: Users MUST be able to resize the split ratio within safe minimum bounds.
- **FR-029**: Users MUST be able to swap left/right session placement without changing canonical session identity.
- **FR-030**: Users MUST be able to maximize one Session Slot and restore the prior pair deterministically.
- **FR-031**: Focus state MUST be visually and accessibly obvious in dual mode.
- **FR-032**: Shared-worktree/shared-checkout conditions across the two sessions MUST be visibly identified and MUST NOT be described as isolated.
- **FR-033**: One session's exit/failure/ownership loss MUST NOT visually or behaviorally contaminate the peer session's lifecycle state.
- **FR-034**: Layout persistence, if selected by Plan/Tasks, MUST restore presentation bindings only and MUST revalidate live ownership independently.

## Runtime / Agent Identity Treatment

- **FR-035**: Admitted runtime sessions MUST expose an icon/glyph and text-accessible runtime label in the left dock and session header.
- **FR-036**: Initial admitted branded runtime treatments MUST include Codex and Claude where the underlying runtime identity is legitimately available.
- **FR-037**: Shell/non-agent sessions MUST use a distinct neutral session-kind treatment.
- **FR-038**: Requested runtime identity MUST remain distinguishable from Winds-observed runtime identity.
- **FR-039**: Agent/terminal self-report MUST NOT manufacture observed runtime/provider/model identity.
- **FR-040**: Unknown, unavailable, conflicting, or stale runtime identity MUST have an explicit non-brand-misleading treatment.
- **FR-041**: Icon-only runtime identification is insufficient; accessible text/name MUST be available.
- **FR-042**: Runtime icon assets or marks MUST receive license/trademark/provenance review before release use; otherwise Winds-owned neutral glyphs MUST be used.
- **FR-043**: Runtime/provider/model identity UI MUST preserve the canonical separation `RUNTIME != PROVIDER != MODEL`.

## Right Context Dock

- **FR-044**: Winds Desktop MUST expose a right-side contextual dock that can be collapsed without losing access to its functions.
- **FR-045**: The first specification MUST provide bounded product semantics for at least `Files`, `Changes`, `Evidence`, `Context`, `Artifacts`, and `Needs You` surfaces; Plan/Tasks may sequence their implementation.
- **FR-046**: The right dock MUST follow the currently selected Session by default.
- **FR-047**: If a pinning interaction is implemented, the binding target MUST be explicit as Project, left Session, or right Session.
- **FR-048**: Files MUST bind to the selected/pinned canonical workspace/worktree root and MUST NOT silently cross roots.
- **FR-049**: Changes/Diff MUST identify exact Git/worktree/candidate context and MUST NOT imply verification merely because a diff is rendered.
- **FR-050**: Evidence MUST reuse repository-native Winds verification/evidence semantics rather than create a UI-local verifier.
- **FR-051**: Context MUST distinguish presentation alias, canonical session/workflow/stage identity, runtime/provider/model identity, source, authority, and continuity proof level where available.
- **FR-052**: Artifacts MUST retain source/provenance and MUST NOT become trusted evidence merely because they appear in the right dock.
- **FR-053**: Needs You MUST bind every trusted attention request to exact source/reason/context rather than agent prose alone.
- **FR-054**: Stale right-dock data MUST become visibly stale when its bound identity moves.
- **FR-055**: Untrusted terminal/agent content MUST NOT inject trusted Files/Changes/Evidence/Context/Artifacts/Needs-You records.

## Session Header and Work Surface

- **FR-056**: Every visible Session Slot MUST expose a compact header with alias, runtime/session-kind treatment, lifecycle/attention state, and sufficient worktree/workflow context for safe operation.
- **FR-057**: Consequential session actions MUST expose their exact target before execution when dual-session ambiguity is possible.
- **FR-058**: Terminal rendering, if selected by Plan, MUST preserve the accepted PTY/ConPTY ownership and byte/order truth rather than invent a second terminal authority.
- **FR-059**: Terminal output MUST remain visually distinguishable from trusted Winds UI and evidence surfaces.
- **FR-060**: Agent conversation/activity presentation MUST retain source boundaries and MUST NOT convert model prose into verification or human decisions.
- **FR-061**: The center surface MUST support dragging/selecting a session from the left dock into a target slot or an equivalently direct keyboard action.
- **FR-062**: Closing a visual slot MUST NOT implicitly terminate a process unless the user selects a termination action whose semantics are explicit.
- **FR-063**: A slot bound to historical/non-live state MUST be visibly non-live.
- **FR-064**: Native runtime resume, reconstructed continuation, reassignment, handoff, ownership loss, unavailable, and unproven continuity MUST remain visually distinct where applicable.

## Command Surface and Navigation

- **FR-065**: Winds Desktop MUST provide a universal command/search surface reachable by keyboard.
- **FR-066**: Command search MUST span Projects, Sessions, permitted actions, files/context surfaces, and navigation without granting authority through ranking.
- **FR-067**: Destructive/consequential actions MUST require explicit selection/confirmation according to existing canonical authority rather than command-palette convenience.
- **FR-068**: Primary Project/Session/dual-view/right-dock actions MUST have keyboard-reachable paths.
- **FR-069**: Pointer parity MUST exist for primary navigation, session selection, split resizing, right-dock interaction, and context menus.
- **FR-070**: Keyboard shortcuts MUST be discoverable and conflict-aware; platform-specific variants MUST be explicit where needed.
- **FR-071**: Search/navigation result labels MUST expose enough identity to distinguish duplicate aliases or similarly named paths.

## Attention System

- **FR-072**: Winds Desktop MUST aggregate material human-attention states across Projects/Sessions without requiring transcript polling.
- **FR-073**: Attention rollup MUST be deterministic for identical canonical inputs.
- **FR-074**: A trusted attention item MUST identify why attention is needed and the exact Project/Session/workflow/candidate/authority context available.
- **FR-075**: Agent text such as `done`, `blocked`, or `approved` MUST NOT itself create trusted completion/approval state.
- **FR-076**: Completion notification MUST remain separate from verification, acceptance, and landing state.
- **FR-077**: Notification delivery MUST be bounded and suppressible; background activity MUST NOT create perpetual decorative motion or alert fatigue.

## Visual Language and Motion

- **FR-078**: Winds Desktop MUST define a Winds-owned visual language rather than shipping framework-default styling as the accepted product surface.
- **FR-079**: The visual language MUST define typography, spacing, radii, elevation/material hierarchy, icons, focus, semantic states, density, and motion rules.
- **FR-080**: Dark and light appearance MUST both be supported before final product acceptance unless Tasks explicitly stage one as a non-release prototype.
- **FR-081**: Brand accent color MUST NOT replace semantic lifecycle/evidence/authority state encoding.
- **FR-082**: Motion MUST explain focus, spatial continuity, state transition, or hierarchy and MUST avoid perpetual decorative animation by default.
- **FR-083**: Reduced-motion mode MUST remove or substitute non-essential transitions without hiding state changes.
- **FR-084**: Translucency/blur/glass treatments, if used, MUST remain selective and MUST NOT reduce text/state contrast or create platform-performance overclaims.
- **FR-085**: Compact professional density MUST be supported; density changes MUST NOT change semantics.
- **FR-086**: The application MUST remain recognizably Winds without requiring proprietary/copyrighted UI asset copying from another product.

## Accessibility

- **FR-087**: All primary controls MUST expose appropriate accessible names/roles/states in the selected desktop stack.
- **FR-088**: Critical status MUST have non-color-only text/icon semantics.
- **FR-089**: Keyboard focus MUST remain visible and deterministic across left dock, dual slots, command surface, and right dock.
- **FR-090**: Scalable text MUST not make primary actions unreachable at the Plan-defined acceptance scale.
- **FR-091**: High-contrast appearance MUST preserve Project/Session hierarchy and trusted/untrusted state distinctions.
- **FR-092**: Screen-reader/semantic accessibility requirements MUST be defined for the selected stack at Plan/Tasks stage and exercised before final closeout.

## Performance and Responsiveness

The Plan MUST pin reproducible measurement environments before implementation claims qualify.

- **FR-093**: Cold desktop launch to first interactive Project/Session shell MUST meet a Plan-defined p95 budget no looser than 2500 ms on the reference environment unless a later accepted Spec amendment changes the product requirement.
- **FR-094**: Project/session selection and dock switching that require no external process/network work MUST meet a Plan-defined local interaction p95 budget no looser than 50 ms from accepted input to committed UI state.
- **FR-095**: Dual-session focus switching and split-resize presentation MUST remain visually responsive under representative terminal output load; Plan MUST define a measurable frame/input budget.
- **FR-096**: Idle desktop overhead MUST receive explicit CPU/RSS budgets separate from child-process memory.
- **FR-097**: Hidden/background sessions MUST NOT trigger unnecessary full-frame visual work merely because they emit output.
- **FR-098**: The left dock MUST remain usable under the Plan-defined large Project/Session fixture, with measured search/scroll/focus performance and stable selection.
- **FR-099**: Every performance claim MUST retain exact candidate/tree, platform, build profile, fixture, renderer/window environment, measurement method, and raw/lossless-enough result provenance.

## Security, Trust, and Host Actions

- **FR-100**: Desktop renderer content MUST be treated as untrusted unless it originates from a canonical Winds-owned state path explicitly classified otherwise.
- **FR-101**: Terminal/agent content MUST NOT be able to forge trusted Winds chrome, runtime badges, verification state, attention requests, authority prompts, or human decisions.
- **FR-102**: URL/file/open actions originating from untrusted content MUST require explicit user action and scheme/path/policy validation.
- **FR-103**: Clipboard-writing requests from terminal/agent content MUST NOT silently mutate the host clipboard by default.
- **FR-104**: Drag/drop MUST NOT silently expand filesystem/workspace authority or execute imported content.
- **FR-105**: Any desktop-to-Rust bridge selected by Plan MUST expose the narrowest command surface, validate every argument in Rust, and prevent remote/untrusted origins from obtaining local Winds command authority.
- **FR-106**: The desktop MUST NOT require a hidden cloud account, telemetry control plane, or network service for local Project/Session operation.
- **FR-107**: Secrets MUST NOT be persisted into UI metadata, logs, crash diagnostics, screenshots/fixtures, or evidence merely to power presentation.
- **FR-108**: Browser runtime, remote control, public IPC, persistent daemon, generic plugin execution, and automatic Git landing remain unauthorized unless separately specified.

## Platform Truth

- **FR-109**: macOS, Windows, Linux, and WSL claims MUST remain bounded to directly exercised domains.
- **FR-110**: Native Windows and WSL2 MUST remain visibly distinct execution/path domains where both are exposed.
- **FR-111**: Platform-native window/chrome integration MAY differ by OS, but canonical Project/Session/evidence semantics MUST remain consistent.
- **FR-112**: A platform limitation MUST be shown truthfully rather than hidden behind universal-looking UI affordances.

## Agent-Native Session Interaction

- **FR-113**: Every admitted agent Session MUST provide a first-class session interaction surface with a stable prompt composer and a chronological work stream; a raw terminal MUST NOT be the only desktop interaction path for that Session once the applicable runtime adapter is authorized.
- **FR-114**: The prompt composer MUST expose its exact target Session and MUST remain keyboard reachable, multiline-capable, and compatible with explicit file/context attachment semantics selected later by Plan/Tasks.
- **FR-115**: The agent work stream MUST distinguish at least user prompt, user-visible agent response, tool/action summary, command execution/result, file change/diff, test/verification result, approval/attention request, error, and completion event classes where those events exist.
- **FR-116**: Every typed work event MUST retain source/provenance sufficient to preserve `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`; visual grouping MUST NOT upgrade trust.
- **FR-117**: Tool/command activity MAY collapse detail for scanability, but target, status, source, and material failure/attention state MUST remain visible without expanding every event.
- **FR-118**: File-change events MUST bind to exact worktree/path context and provide a direct route to the right-dock Files/Changes surface without inventing candidate verification.
- **FR-119**: Test/check events MUST distinguish agent-reported results from repository-native Winds verification evidence.
- **FR-120**: In dual-session mode, each composer and contextual action MUST target exactly one Session by default; no visual proximity, selection rectangle, or shared Project MAY create implicit multi-session dispatch.
- **FR-121**: Send, stop, interrupt, retry, approve, deny, or equivalent session actions MUST preserve the underlying runtime/authority semantics and identify the exact target before consequential execution.
- **FR-122**: Background agent Sessions MUST retain truthful lifecycle/attention rollups in the left dock so the user can supervise parallel work without keeping every stream visible.
- **FR-123**: Winds Desktop MUST NOT require, reconstruct, expose, or claim access to hidden model chain-of-thought. User-visible reasoning summaries or explanations may be rendered only when supplied through an accepted user-visible runtime surface and MUST remain source-labelled.
- **FR-124**: Session-stream content MUST support normal selection/copy/navigation while remaining unable to forge trusted Winds controls, badges, approvals, evidence, files, or host actions.
- **FR-125**: Switching away from and back to a Session MUST preserve available session history/context truthfully; missing, truncated, evicted, or unavailable history MUST be explicit rather than visually reconstructed as complete.
- **FR-126**: The session surface SHOULD prefer inline/progressive interactions over modal interruption for routine work; modals MAY be used only where protected focus or consequential confirmation is materially required.
- **FR-127**: The default agent-work presentation MUST optimize for scanability over chat-bubble theater: dense typed events, calm hierarchy, restrained color, and progressive detail rather than every event receiving equal card weight.
- **FR-128**: The first desktop design system MUST define complete default/hover/focus/active/disabled/loading/error/empty states for primary session controls before final acceptance.
- **FR-129**: Routine desktop transitions SHOULD complete within a Plan-defined range centered on approximately 150–250 ms unless direct manipulation or platform-native behavior requires another measured rule; decorative page-load choreography is prohibited by default.
- **FR-130**: Project-scoped Session organization MUST remain the primary desktop history/navigation model; a global flat Recents list MAY exist as a convenience surface but MUST NOT be the only way to recover long-running project work.
- **FR-131**: The design MUST use familiar product affordances for standard operations unless a materially better task-specific interaction is proven; visual novelty alone is insufficient justification for reinventing scroll, selection, forms, menus, dialogs, or focus behavior.
- **FR-132**: Winds' accepted desktop appearance MUST be recognizably its own and MUST NOT reproduce Codex, Claude, Herdr, Cursor, Linear, Warp, or Impeccable trade dress, proprietary assets, or exact visual composition.

---

# Success Criteria

- **SC-001**: A user can create/open three Projects, create at least four Sessions in one Project, rename every Session, and return to the same presentation organization without any canonical identity changing because of rename/reorder/pin actions.
- **SC-002**: Dual-session mode directly proves two independent live session targets can remain visible simultaneously while input sent to one never reaches the peer without an explicit separately authorized multi-target action.
- **SC-003**: Dual mode directly proves resize, swap, maximize, restore, replace-one-slot, and close-one-slot preserve the untouched peer session identity and lifecycle.
- **SC-004**: A shared-worktree fixture visibly distinguishes two sessions sharing one mutable checkout from sessions using distinct worktrees.
- **SC-005**: Claude, Codex, shell, requested-only, observed, mismatched, unknown, unavailable, and stale runtime identity fixtures render without accepting forged agent/terminal claims as observed identity.
- **SC-006**: Runtime identity is understandable with color disabled because icon/glyph plus accessible text/state remain available.
- **SC-007**: The left dock deterministically exposes a material blocked/needs-attention descendant even when its Project is collapsed.
- **SC-008**: A fixture with duplicate Project/Session aliases cannot cause a consequential action to target the wrong canonical session without explicit disambiguation.
- **SC-009**: Files and Changes surfaces follow the correct worktree as focus moves between two sessions from different roots.
- **SC-010**: Evidence/Context surfaces reject or visibly stale candidate-bound content after exact candidate/tree movement.
- **SC-011**: Terminal/agent fixtures containing forged `VERIFIED`, `ACCEPTED`, Claude/Codex labels, file links, attention requests, or evidence-shaped JSON cannot alter trusted desktop state.
- **SC-012**: Every primary Project/Session/dual-view/right-dock workflow can be completed keyboard-only with visible focus.
- **SC-013**: Every primary Project/Session/dual-view/right-dock workflow can be completed pointer-only where pointer hardware is present.
- **SC-014**: Reduced-motion, high-contrast, dark, light, compact-density, and scaled-text acceptance fixtures preserve every critical lifecycle/evidence/authority distinction.
- **SC-015**: The accepted desktop candidate has an explicit human aesthetic review confirming the surface meets the approved Winds visual-language rubric; deterministic CI alone cannot self-certify aesthetic quality.
- **SC-016**: Plan-defined launch and local-interaction performance campaigns meet FR-093 and FR-094 on the exact accepted candidate in the declared reference environments.
- **SC-017**: Plan-defined dual-session output/resize/focus stress meets FR-095 without lost input ownership or false lifecycle state.
- **SC-018**: Plan-defined idle CPU/RSS and large-session-list campaigns meet FR-096 through FR-098 with exact-candidate evidence.
- **SC-019**: Security fixtures prove untrusted content cannot trigger clipboard write, external open, trusted badge creation, authority prompt creation, or privileged desktop bridge calls without accepted user/policy action.
- **SC-020**: Every released platform claim has directly exercised exact-candidate evidence; no native-Windows/WSL/Linux/macOS parity claim is inferred from another domain.
- **SC-021**: The final first implementation program introduces no persistent daemon/public IPC/remote control/browser runtime/generic plugin system/automatic routing/automatic Git landing unless this Spec is separately amended first.
- **SC-022**: Project/session UI metadata can be removed/rebuilt without destroying canonical workspace/session/workflow/candidate/evidence history.
- **SC-023**: Presentation restart never claims restored live-child ownership solely from remembered Project/Session layout metadata.
- **SC-024**: The final exact candidate passes repository quality, applicable desktop/platform/security/performance workflows, author correctness/safety review, Ponytail/YAGNI review, fresh independent substantive review, and has zero unresolved material findings/threads.
- **SC-025**: Final closeout reconciles every FR/SC to exact evidence and preserves all inherited Spec 006/007/008/009 nonclaims and historical failures without relabelling them.
- **SC-026**: Two simultaneously visible Claude/Codex-style agent-session fixtures can each accept prompts and stream typed work events while exact input/action targeting proves zero unintended cross-session dispatch.
- **SC-027**: A canonical agent-session fixture containing command activity, file change, diff, test result, approval request, error, and completion renders each event with the required source/trust distinction and direct contextual navigation.
- **SC-028**: Switching among at least 100 project-scoped Session fixtures preserves deterministic selection/search and does not require a global flat Recents list to recover the target Session.
- **SC-029**: No acceptance test, UI contract, or product copy claims hidden model chain-of-thought access; user-visible summaries remain explicitly source-labelled.
- **SC-030**: Final human visual review confirms the accepted build satisfies the Winds Operate-mode craft rubric: task-first scanability, restrained semantic color, complete component states, purposeful short motion, non-default product detailing, and no copied competitor trade dress.

---

# Explicit Non-Goals

Spec 010 does NOT by itself authorize:

- a persistent local daemon or `windsd`;
- public/private socket, HTTP, WebSocket, or generic IPC control plane beyond any narrowly selected in-process desktop bridge that the later Plan explicitly qualifies;
- cross-restart ownership of live terminal/agent processes;
- remote/mobile clients;
- browser automation, Browser Twin, CDP, or web verification runtime;
- SQL Studio;
- generic plugin/extension marketplace or dynamic code loading;
- automatic model/provider routing, learned routing, winner scoring, or silent fallback;
- training, fine-tuning, RL, autonomous skill mutation, or semantic/vector memory;
- automatic candidate selection, merge, rebase, cherry-pick, push, PR creation, or landing;
- copying proprietary/closed-source product assets or AGPL application code into Winds under incompatible terms;
- a custom editor, custom browser engine, or custom terminal protocol merely to achieve visual differentiation;
- weakening any accepted Git/evidence/authority/secret/workflow/terminal/model-mesh/platform/recovery/human-decision boundary.

---

# Plan-Stage Decisions Required

The later Plan MUST decide with exact evidence, not preference:

1. desktop shell/window technology;
2. presentation framework/build tool;
3. terminal rendering strategy;
4. UI-to-Rust bridge shape and threat model;
5. UI metadata persistence location/schema and migration/reversibility strategy;
6. design-token/icon strategy and brand-asset provenance;
7. file-tree/diff rendering strategy;
8. accessibility test stack;
9. visual regression/screenshot strategy;
10. performance reference environments and thresholds tighter than or equal to the Spec ceilings;
11. packaging/bundle scope for the first implementation program;
12. whether any P1 requirement truly requires persistent ownership/daemon/IPC; default answer remains NO unless proven otherwise.

Candidate architecture references MAY include Tauri 2.x, React 19.x, Vite 8.x, and a bounded terminal renderer, but this Spec selects none of them.

The Plan SHOULD also define a bounded design workflow inspired by the pinned Impeccable Operate/craft-floor process: capture product truth, select one coherent Winds visual world, build the full primary surface, inspect in one batched visual/accessibility pass, repair findings in one batch, and confirm with at most one additional visual pass. This process reference MUST NOT override canonical Winds product/evidence truth or create an external runtime dependency.

---

# Specification Acceptance Gate

This Spec may land only if its exact final candidate proves:

- canonical base is the accepted Spec 010 entry merge or a governance-only forward descendant;
- changed scope is exactly `specs/010-winds-desktop-agentic-workspace/spec.md` unless a tightly coupled governance-only correction is separately justified;
- requirements remain implementation-agnostic and do not smuggle framework/dependency selection into specification authority;
- Project/Session/dual-session/right-dock/runtime-icon requirements preserve canonical identity and authority truth;
- the prior provisional `Spec 010 = durable owner` roadmap is not silently reintroduced;
- no production source, dependency, lockfile, migration, workflow, desktop runtime, daemon/IPC, browser, remote, provider/model execution, credential, learning, routing, or automatic-landing behavior changes;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review challenges unnecessary features, abstractions, dependencies, and any premature durable-owner assumption without simplifying away accessibility/security/truth requirements;
- fresh independent substantive review challenges user scenarios, testability, identity/authority boundaries, Herdr-parity assumptions, visual-quality measurability, and non-goals;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal landing;
- merge tree/ordered parents/signature verified;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN_AUTHORIZED=YES
SPEC_010_TASKS_AUTHORIZED=NO
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
DURABLE_LOCAL_RUNTIME_OWNER_AUTHORIZED=NO
DAEMON_IPC_AUTHORIZED=NO
```

No source implementation may begin until a separate Plan and Tasks sequence independently qualifies and lands.
