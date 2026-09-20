# Feature Specification: Workspace Multiplexer & Agent Plane

**Feature Branch**: spec/012-workspace-multiplexer-agent-plane  
**Created**: 2026-09-20  
**Status**: Specification candidate only. Plan, Tasks, implementation, dependencies, migrations, donor-code admission, remote transport, plugin runtime, marketplace, updater, release promotion, and automatic Git landing are NOT authorized by this file alone.  
**Input**: Canonical Spec 012 Entry Gate after canonical Spec 011 closeout.

## Product Thesis

Winds needs a compact, exact-target local workspace multiplexer and agent plane, not a generic orchestration platform.

Spec 012 defines user-visible and authority-safe behavior for workspaces, tabs, panes, terminal interaction, agent detection/presentation, bounded local automation, and worktree-aware workflows. It consumes the accepted desktop and persistent-runtime foundations without broadening them into remote execution, public RPC, arbitrary shell dispatch, plugin execution, automatic Git landing, or unproven provider/model claims.

Core invariants:

~~~text
WORKSPACE_ID != WORKSPACE_ALIAS
TAB_ID != TAB_ALIAS
PANE_ID != PANE_LABEL
PANE_ID != OS_PID
PANE_ID != RUNTIME_NAMESPACE_ID
PANE_ID != CANONICAL_WINDS_SESSION
PANE_ID != PROVIDER_NATIVE_SESSION_ID
PANE_ID != GIT_CANDIDATE_OR_EVIDENCE_ID

AGENT_DETECTED != AGENT_EXECUTION_PROVEN
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
NEEDS_YOU != VERIFIED != ACCEPTED
REPLAYED_OUTPUT != CANONICAL_EVIDENCE
PROCESS_EXIT != VERIFIED
DIFF_RENDERED != VERIFIED
DONE != VERIFIED
~~~

This specification defines product requirements, trust boundaries, failure semantics, and measurable outcomes. It does not select a UI framework, terminal library, manifest engine, command bus, Git library, persistence schema, donor-code slice, or implementation architecture.

## Canonical Baseline

Spec 012 begins only after the canonical Entry Gate landed and completed post-merge qualification:

~~~text
SPEC_011_T161_MERGE=bdc30eb34dea2d89213aebabb1bccdf2d5eb5edd
SPEC_011_T161_TREE=76e059cb15638f2013580a04e2a00282a4e857e2
SPEC_011_POST_MERGE_QUALITY=35510161683 SUCCESS

SPEC_012_ENTRY_MERGE=ec214544d94d90b23fe4a0156d0040a2faf0622c
SPEC_012_ENTRY_TREE=91fb27328373288dbc0e2be4904bae2e5c58aca5
SPEC_012_ENTRY_POST_MERGE_QUALITY=35513773204 SUCCESS

SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_FORMAL_SPEC_AUTHORIZED=YES
SPEC_012_PLAN_AUTHORIZED=NO
SPEC_012_TASKS_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO
~~~

The accepted Spec 011 persistent owner/private local-control plane remains the upstream authority. Spec 012 may consume it only through explicit exact-target operations and MUST NOT create a second process-ownership authority, public network surface, implicit controller-promotion path, or PID-based recovery path.

## Identity and Authority Model

The following domains remain distinct:

- Project ID: existing canonical Winds project identity.
- Session ID: existing canonical Winds session/workstream identity.
- Runtime Namespace ID: accepted Spec 011 local runtime identity.
- Workspace ID: immutable multiplexer workspace identity.
- Tab ID: immutable tab identity scoped to a workspace.
- Pane ID: immutable pane identity. Pane bindings MUST be validated against the current topology generation, and Pane IDs MUST NOT be reused for replacement panes.
- Agent Observation ID: immutable detection observation bound to exact pane/runtime/source evidence.
- Provider-Native Session ID: provider-owned identity when independently available.
- Repository/Worktree Identity: canonical repository root plus exact worktree identity/path.
- Candidate/Evidence Identity: existing Git/evidence authority, never inferred from pane/session state.

Aliases, titles, labels, sort positions, UI focus, PID values, shell titles, agent text, and provider-reported names are presentation metadata only.

Every consequential request MUST bind immutable target identity plus the current topology/runtime generation. A stale-generation request MUST fail closed instead of retargeting by alias, index, nearest pane, reused PID, or current focus.

## Entry-Time Herdr Research Pin and Parity Envelope

The Entry Gate binds:

~~~text
HERDR_REPOSITORY=herdrdev/herdr
HERDR_DEFAULT_BRANCH=master
HERDR_PIN=29f9f4056f344af60f411004fc89c7eb5f357c48
HERDR_TREE=8e11bddc337094b8aece2a7abea4ee6ee6340f2a
HERDR_PACKAGE_VERSION=0.9.1
HERDR_ROOT_LICENSE=Apache-2.0
AGENT_FAMILIES=24
FROZEN_SERIALIZED_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INSTALLABLE_TARGETS=1
PUBLIC_SERIALIZED_API_METHODS=105
HERDR_BASELINE_CAPABILITY_ROWS=219
~~~

Spec 012 owns the exact 142 ledger rows already marked Spec 012 in docs/research/021-herdr-exhaustive-capability-ledger.md:

- A01-A24: all 24 agent-detection families.
- M005, M006, M007, M010-M013, M015-M090, M105: 84 public-method parity rows assigned to this domain.
- R008-R030, R039, R040, R058, R060, R062-R067, R071: 34 additional runtime/UI/config/worktree parity rows.

Rows assigned to Specs 013-015 remain outside this Spec even when adjacent upstream code shares modules.

All 24 families are detection/presentation scope at specification time:

~~~text
Pi
Claude
Codex
Gemini
Cursor
Devin
Antigravity
Cline
Omp
Mastracode
OpenCode
GithubCopilot
Kimi
Kiro
Droid
Amp
Grok
Hermes
Kilo
Qodercli
Qwen
Letta
Maki
Muse
~~~

No family receives real-execution authority merely because it can be detected. Desired agent.start, agent.prompt, agent.wait, agent.send_keys, launch, or install behavior remains dependent on later Plan/Tasks proof. This Spec does not authorize any implementation seam.

## Spec 012 Ledger Traceability

The 142-row assignment is omission-prevention evidence, not a mandate to copy upstream architecture. The specification maps the assigned rows as follows:

| Ledger rows | Specification disposition |
| --- | --- |
| A01-A24 | FR-031 through FR-047; SC-007 through SC-011. Detection/presentation only at specification time. |
| M005-M006 | FR-035 through FR-037. Manifest inspection/refresh may be qualified later without generic plugin execution. |
| M007 | FR-028 and SC-019. Local bounded notification behavior only. |
| M010 | FR-059 through FR-064; SC-016. Typed allowlisted command invocation only. |
| M011-M013 | FR-025 and FR-075 through FR-084. Window/surface presentation only; no remote authority. |
| M015-M023 | FR-001 through FR-015. Workspace identity, metadata, ordering, movement, focus, and close remain exact-target. |
| M024-M027 | FR-065 through FR-074; SC-017 through SC-018. Worktree discovery/create/open/remove only under explicit trust and path safety. |
| M028-M034 | FR-001 through FR-015. Tab identity/navigation/mutation remain exact-target. |
| M035-M046 | FR-031 through FR-047. Agent read/presentation operations are separated from later execution-seam qualification. |
| M047-M090 and M105 | FR-003 through FR-030 plus FR-048 through FR-064. Pane topology, read/input/copy/link/graphics/agent association/events/wait/clear behavior stays exact-target and bounded. |
| R008-R017 | FR-001 through FR-030. Multiplexer, layout, scrollback, selection, links, local graphics/clipboard, terminal presentation, and local notifications. |
| R018-R020 | FR-031 through FR-047. Agent detection, state, and compact agent-dock presentation. |
| R021-R030 | FR-021 through FR-030, FR-055 through FR-064, and FR-075 through FR-086. Keybindings, navigation, palette, menus/overlays/settings/theme/sidebar/tab presentation, mouse, and CJK/IME remain local presentation/control concerns. |
| R039-R040 | FR-065 through FR-074. Worktree workflows and repository trust. |
| R058 | First-run onboarding is presentation-only Plan-stage qualification. It MUST NOT create account, network, plugin, execution, or repository authority and MUST truthfully expose unsupported/unproven capabilities. |
| R060 | FR-017. Terminal default-shell/profile behavior remains inside accepted ownership. |
| R062 | FR-018 and SC-006. Scrollback memory is bounded. |
| R063 | FR-006 through FR-015 and FR-055 through FR-064. Naming and close-confirmation UX cannot substitute aliases/focus for immutable target identity. |
| R064-R066 | FR-025, FR-028, and FR-075 through FR-084. Pane chrome, accent/status presentation, and toasts remain accessible, bounded, and non-authoritative. |
| R067 | FR-072. Worktree root-directory configuration is explicit and path-safe. |
| R071 | FR-029 plus FR-085-FR-086. Host redraw/cursor behavior requires direct platform evidence. |

For rows whose upstream implementation mixes local and remote behavior, only the local Spec 012 portion is covered here. Remote clipboard/file bridging, remote title/control authority, remote machines, SSH, and thin-client behavior remain assigned to Spec 013 and are not imported through shared upstream modules.

## Qualification-Time Herdr Drift Reconciliation

A fresh qualification check observed one upstream commit after the Entry pin:

~~~text
HERDR_QUALIFICATION_HEAD=c00a62dda169beb472fc2f386d0f673ca100dfa4
HERDR_QUALIFICATION_TREE=ff0822ed7bb848e2e340dc382920da7c519f2458
HERDR_DELTA_FROM_ENTRY_PIN=1_COMMIT_AHEAD_0_BEHIND
HERDR_QUALIFICATION_GITHUB_VERIFICATION=VERIFIED_VALID
HERDR_QUALIFICATION_GITHUB_VERIFICATION_REASON=valid
HERDR_QUALIFICATION_GITHUB_VERIFIED_AT=2026-09-20T14:10:17Z
HERDR_QUALIFICATION_GITHUB_COMMITTER=GitHub <noreply@github.com>
HERDR_QUALIFICATION_GITHUB_COMMIT_URL=https://github.com/herdrdev/herdr/commit/c00a62dda169beb472fc2f386d0f673ca100dfa4
HERDR_QUALIFICATION_GITHUB_VERIFIED_PARENT=29f9f4056f344af60f411004fc89c7eb5f357c48
~~~

The exact upstream commit is "fix: preserve delayed mouse reports with confirmed keyboard input (#4247)". It changes eight files under rendered-client terminal input/setup, raw-input framing, Unix fd helpers, and client-mode tests. The change preserves delayed SGR mouse reports when host keyboard escape disambiguation is confirmed, buffers host input observed during capability probing, and captures geometry before replaying buffered mouse input.

This movement is material to Spec 012 terminal input qualification, especially FR-022, FR-023, FR-029, SC-024, and the Plan-stage keyboard/mouse/IME/input-source decision. It does not modify agent detection manifests, integration enums, or the serialized API schema; therefore the Entry-pinned 24-agent, 17-plus-1-integration, 105-public-method, 219-baseline-row, and 142-Spec-012-row counts remain unchanged by this one-commit delta.

The newer upstream head is research evidence only. It does not admit source reuse or expand implementation authority.

Herdr remains research/design evidence. No direct copy, adapted copy, or test port is admitted by this file.

---

# User Scenarios & Testing

## User Story 1 — Exact Workspace, Tab, and Pane Topology (Priority: P1)

A developer can create and manage local workspaces, tabs, and recursive panes without aliases, visual order, or stale indices becoming authority.

**Independent Test**: Create multiple workspaces with duplicate names, tabs with duplicate labels, and nested panes; reorder and rename them; prove every read/mutation still targets immutable identities.

**Acceptance Scenarios**:

1. Duplicate aliases never change immutable targeting.
2. Split/swap/move/resize/zoom operations produce deterministic topology and preserve surviving pane identities.
3. A stale pane generation fails explicitly after topology changes.
4. A closed pane cannot be targeted through a later replacement occupying the same visual position.

## User Story 2 — Deterministic Layout and Navigation (Priority: P1)

A user can navigate by direction, neighbor, edge, indexed search, mouse, and keyboard while the multiplexer remains exact-target and deterministic.

**Independent Test**: Exercise asymmetric layouts, zoomed panes, moved tabs, duplicate labels, rapid focus changes, and layout export/apply; identical initial topology plus operation sequence must produce the same accepted result.

**Acceptance Scenarios**:

1. Directional focus uses a documented deterministic rule.
2. Layout export preserves topology semantics without embedding transient authority.
3. Layout apply cannot silently restore stale runtime/process authority.
4. Saved layouts are presentation/topology templates, not proof of live process continuity.

## User Story 3 — Terminal Interaction Stays Inside Accepted Ownership (Priority: P1)

Users receive professional terminal behavior—scrollback, selection, search, copy, link handling, clear, profiles, IME/input behavior, borders, labels, and local presentation settings—without introducing a second terminal backend or weakening PTY/ConPTY ownership.

**Independent Test**: Exercise terminal interaction under high output, large scrollback, CJK/IME input, copy/search, pane clear, local links, and platform-specific keyboard/mouse behavior while confirming exact target and bounded memory.

**Acceptance Scenarios**:

1. Pane clear has explicit screen/scrollback semantics and cannot clear a peer pane.
2. Scrollback remains bounded under sustained output.
3. Selection/copy/search cannot mutate process authority.
4. Link activation requires accepted user/policy action and cannot turn terminal text into privileged control.

## User Story 4 — Detect All 24 Agent Families Truthfully (Priority: P1)

Winds can classify source-observed agent sessions across all 24 required families while exposing source, confidence, freshness, and ambiguity.

**Independent Test**: Run positive, negative, stale, ambiguous, customized-control, and unknown-agent fixtures for every family and prove the resulting classification is attributable and cannot be upgraded from terminal text alone.

**Acceptance Scenarios**:

1. Positive classifications identify the accepted evidence class.
2. Ambiguous evidence yields ambiguous/unknown rather than false certainty.
3. Stale observations cannot remain presented as current.
4. Detection never proves real provider execution, provider identity, model identity, or provider-native resume by itself.

## User Story 5 — Agent Plane Gives Compact, Exact Human Control (Priority: P1)

A user can list, inspect, focus, rename, view, and understand detected agents from a compact Winds-owned surface while retaining exact pane/session/runtime bindings.

**Independent Test**: Populate many detected agents across projects/workspaces/panes, including duplicate labels and stale observations; prove selection and actions remain exact and source-labelled.

**Acceptance Scenarios**:

1. Agent list/get/read/explain expose immutable bindings and provenance.
2. Renaming presentation labels never changes provider/session/runtime identity.
3. Focus/view operations are presentation behavior and do not confer controller authority.
4. A stale agent-to-pane binding cannot silently retarget.

## User Story 6 — Needs You Is Trusted Attention, Not Agent Theater (Priority: P1)

Winds aggregates human-attention needs from accepted structured states without treating arbitrary agent text such as "approve this" or "done" as trusted authority.

**Independent Test**: Inject forged attention-looking terminal text, real structured attention events, stale events, duplicates, and multiple projects; only accepted structured sources may create trusted Needs You items.

**Acceptance Scenarios**:

1. Trusted Needs You items identify exact available context.
2. Duplicate events do not duplicate consequential authority.
3. Stale items retain original binding visibly or are deterministically removed.
4. Agent output remains source-labelled untrusted content even when it resembles Winds chrome.

## User Story 7 — Exact-Target Local Automation Never Broadcasts Implicitly (Priority: P1)

Users may invoke bounded local operations such as pane navigation, text/key input, or agent actions only against explicit targets and accepted controller authority.

**Independent Test**: Create multiple writable panes/agents and race focus, controller changes, delayed commands, duplicate request IDs, and stale topology. No operation may cross-target or broadcast unless a separately specified multi-target operation exists.

**Acceptance Scenarios**:

1. Focus alone never authorizes a consequential operation.
2. Input/send-keys/send-text/prompt requests carry exact target identity and generation.
3. Duplicate/replayed consequential requests follow explicit idempotency/rejection semantics.
4. Command-palette convenience cannot become arbitrary shell dispatch.

## User Story 8 — Worktree Operations Are Safe and Do Not Acquire Git Landing Authority (Priority: P1)

Users can discover, create, open, and remove worktrees through explicit repository trust and exact identity without gaining merge/rebase/push/PR/landing behavior.

**Independent Test**: Exercise trusted/untrusted repositories, nested roots, ambiguous paths, dirty worktrees, external worktrees, missing metadata, symlink/reparse ambiguity, and removal attempts.

**Acceptance Scenarios**:

1. Worktree operations require exact repository/worktree binding.
2. Repository trust is explicit and cannot be inferred from focus or agent output.
3. Removal fails closed when ownership/path safety is ambiguous.
4. No worktree action performs automatic merge, rebase, cherry-pick, push, PR creation, or landing.

## User Story 9 — Command Palette and Custom Commands Stay Typed and Allowlisted (Priority: P1)

A user can discover and invoke permitted Winds actions without the palette becoming a generic RPC or shell dispatcher.

**Independent Test**: Enumerate palette actions, malformed parameters, stale target context, forged action names, and unavailable capabilities; only typed admitted operations may execute.

**Acceptance Scenarios**:

1. Every command resolves to a known operation schema.
2. Unknown or arbitrary method names fail closed.
3. Consequential commands expose exact target/context when required.
4. Search ranking never grants authority or silently changes target.

## User Story 10 — Local Clipboard, Graphics, Notifications, and Presentation Remain Non-Authoritative (Priority: P2)

Local terminal presentation may support graphics, clipboard interaction, notifications, themes, titles, borders, toasts, and sound where later implementation authority permits, without remote bridging or trusted-state forgery.

**Independent Test**: Exercise hostile terminal content, oversized graphics/clipboard payloads, notification spam, title spoofing, and local clipboard requests.

**Acceptance Scenarios**:

1. Terminal titles/graphics cannot forge trusted Winds status.
2. Clipboard mutation requires accepted user/policy action.
3. Notification/toast delivery is bounded and suppressible.
4. Remote clipboard/image/file bridging remains outside Spec 012.

## User Story 11 — Scale Remains Usable and Bounded (Priority: P2)

A developer can supervise many workspaces, tabs, panes, and detected agents without unbounded memory, event queues, or attention noise.

**Independent Test**: Exercise Plan-defined large-topology fixtures, high terminal output, many agent observations, rapid navigation, and slow consumers.

**Acceptance Scenarios**:

1. Scrollback, event queues, and retained detection history remain bounded.
2. Search/navigation remains deterministic with duplicate names.
3. Slow consumers cannot force unbounded owner/client memory growth.
4. Attention aggregation remains stable and deduplicated.

## User Story 12 — Platform Claims Are Directly Proven (Priority: P1)

Winds presents only behavior directly exercised on the platform domain being claimed.

**Independent Test**: Run applicable keyboard, mouse, IME, clipboard, terminal, cursor, worktree/path, and persistent-owner integration campaigns on every platform claimed by a later implementation.

**Acceptance Scenarios**:

1. Native Windows claims are not inferred from WSL.
2. WSL claims are not inferred from native Windows or Linux.
3. macOS/Linux behavior is not generalized to Windows path/input/security semantics.
4. Unsupported or unproven behavior is explicit.

---

# Threat Model

Spec 012 protects exact target identity, local controller authority, topology integrity, detection truth, repository/worktree safety, bounded resources, trusted Winds chrome, and canonical evidence/Git boundaries.

Later Plan/Tasks MUST qualify at least:

- stale workspace/tab/pane IDs and topology generations;
- duplicate aliases, reordered tabs, reused visual positions, and delayed requests;
- forged terminal/agent output shaped like commands, statuses, approvals, evidence, links, or notifications;
- false-positive, false-negative, ambiguous, and stale agent detection;
- malformed detection-manifest content if manifests become reloadable;
- duplicate/replayed consequential requests and request-ID collisions;
- simultaneous focus/controller/input/resize/close races;
- slow subscribers and event/backpressure exhaustion;
- oversized scrollback, graphics, clipboard, title, notification, and metadata payloads;
- unsafe link/file/clipboard actions from untrusted content;
- repository-root confusion, nested repositories, symlink/reparse substitution, path traversal, dirty worktrees, and ambiguous removal;
- command-palette method forgery or arbitrary shell/RPC escalation;
- renderer/WebView attempts to invoke privileged local operations outside a typed bridge;
- platform-specific keyboard/mouse/IME/cursor differences;
- the inherited same-effective-user non-isolation boundary.

Explicit nonclaim:

> Spec 012 does not promise isolation from arbitrary malicious code already executing as the same effective OS user/principal unless a later separately governed boundary is selected and directly proven.

---

# Functional Requirements

## Identity, Topology, and Layout

- **FR-001**: Every workspace MUST have an immutable Winds-generated identity independent of alias, order, Project, Session, runtime namespace, OS PID, or worktree path.
- **FR-002**: Every tab MUST have an immutable identity independent of alias, ordinal position, and focus.
- **FR-003**: Every pane MUST have an immutable identity independent of label, coordinates, shell title, PID, agent label, and focus.
- **FR-004**: Topology mutations MUST carry the target topology generation or an equivalently strong stale-target proof selected later by Plan.
- **FR-005**: Stale topology mutations MUST fail closed and MUST NOT retarget by alias, ordinal, nearest geometry, focus, or replacement pane.
- **FR-006**: Workspace create/list/get/focus/rename/move/close MUST preserve immutable identity.
- **FR-007**: Tab create/list/get/focus/rename/move/close MUST preserve immutable identity.
- **FR-008**: Pane split/swap/move/resize/zoom/focus/close MUST be deterministic for the same accepted initial state and operation sequence.
- **FR-009**: Neighbor/edge/directional-focus behavior MUST define deterministic tie-breaking independent of timing.
- **FR-010**: Layout export MUST serialize topology/presentation only and MUST NOT serialize live process authority, credentials, controller leases, or canonical evidence as reusable authority.
- **FR-011**: Layout apply MUST create/bind topology under current authority rather than pretending prior live ownership still exists.
- **FR-012**: Saved layouts MUST be distinguishable from live runtime state.
- **FR-013**: Pane metadata/reporting MUST preserve source attribution and MUST NOT convert labels into trusted identity.
- **FR-014**: Pane clear behavior MUST define screen, scrollback, selection, and retained-history effects precisely before implementation.
- **FR-015**: Multi-pane operations MUST default to one explicit target; implicit all-pane/all-agent broadcast is prohibited.

## Terminal UX Inside the Accepted Local Authority Plane

- **FR-016**: Spec 012 MUST reuse the accepted PTY/ConPTY ownership boundary rather than define a second terminal process owner.
- **FR-017**: Terminal profile/default-shell behavior MUST preserve platform truth and exact runtime target.
- **FR-018**: Scrollback storage MUST be bounded with explicit truncation semantics.
- **FR-019**: Scrollback read/edit/search/copy MUST remain presentation/data operations and MUST NOT grant process authority.
- **FR-020**: Selection and word-selection behavior MUST be deterministic enough for repeatable qualification.
- **FR-021**: Link resolution/activation MUST validate target and require accepted user/policy action before host effects.
- **FR-022**: Mouse capture, right-click routing, copy-on-select, and scroll tuning MUST never retarget input because focus changed asynchronously.
- **FR-023**: CJK/IME/input-source behavior MUST be directly exercised on every claimed platform domain.
- **FR-024**: Local graphics MAY be supported only with bounded payload/resource semantics and exact pane binding.
- **FR-025**: Terminal titles/themes/effects/borders/gaps/scrollbars/labels are presentation and MUST NOT encode trusted authority solely through user-controlled text or color.
- **FR-026**: Local clipboard behavior, if implemented, MUST be explicitly user/policy mediated, bounded, and local-only in Spec 012.
- **FR-027**: Remote clipboard/image/file bridging is prohibited.
- **FR-028**: Notifications, toasts, and sound MUST be rate/burst bounded and suppressible.
- **FR-029**: Host redraw/cursor policy MUST not claim cross-platform parity without native evidence.
- **FR-030**: Terminal content resembling Winds control/status/evidence MUST remain untrusted data.

## Agent Detection and Agent Plane

- **FR-031**: Winds MUST provide a specification path for all 24 Entry-pinned agent-detection families A01-A24.
- **FR-032**: Every agent observation MUST retain source, exact pane/runtime binding, observation freshness/generation, and classification state.
- **FR-033**: Detection MUST support observed, unknown, ambiguous, stale, and unavailable states; false certainty is prohibited.
- **FR-034**: Agent classification MUST NOT be based solely on arbitrary free-form terminal text.
- **FR-035**: Manifest-driven detection, if selected later, MUST validate schema/version/content and fail safely on malformed/unsupported manifests.
- **FR-036**: Manifest refresh, if selected, MUST be atomic to observers and preserve the previous accepted set on invalid refresh.
- **FR-037**: Detection refresh MUST NOT create generic plugin execution or arbitrary code loading.
- **FR-038**: Agent list/get/read/explain MUST expose immutable bindings and provenance.
- **FR-039**: Agent rename/view/focus are presentation/navigation operations and MUST NOT alter provider-native identity or controller authority.
- **FR-040**: Agent-to-pane association MUST reject stale pane/runtime generations.
- **FR-041**: Agent start/prompt/wait/send-keys are desired exact-target local behaviors only; implementation remains blocked until later Tasks prove an admitted execution seam.
- **FR-042**: No detected family receives launch/install/execution authority merely by appearing in the detector.
- **FR-043**: Provider identity, model identity, provider-native session identity, process ownership, and Winds Session identity MUST remain separate.
- **FR-044**: Detection MUST NOT convert provider-native or terminal-reported status directly into Winds verification/acceptance truth.
- **FR-045**: The Founder agent dock MUST remain compact, scanable, and identity-safe under duplicate labels and concurrent observations.
- **FR-046**: Sorting/filtering/grouping MUST not change authority or silently drop material Needs You state.
- **FR-047**: Detection-only support is acceptable where execution is unproven; the UI MUST state that limitation truthfully.

## Needs You, Events, and Human Attention

- **FR-048**: Trusted Needs You items MUST derive from accepted structured Winds/runtime state, not arbitrary rendered agent text.
- **FR-049**: Every trusted attention item MUST retain exact available Project/Session/runtime/workspace/tab/pane/agent binding and source class.
- **FR-050**: Attention aggregation MUST be deterministic and deduplicate equivalent events.
- **FR-051**: A stale attention item MUST be discarded or rendered explicitly stale with its original binding; it MUST NOT be rebound silently.
- **FR-052**: Event subscribe/wait surfaces MUST use bounded queues/backpressure and explicit loss semantics.
- **FR-053**: Delayed events MUST be checked against current binding before rendering as current truth.
- **FR-054**: Completion, process exit, agent-reported success, verification, acceptance, and landing MUST remain distinct.

## Bounded Local Automation and Command Surface

- **FR-055**: Every consequential local action MUST bind one explicit immutable target and accepted controller authority.
- **FR-056**: Focus, hover, selection, or search ranking alone MUST NOT authorize a consequential action.
- **FR-057**: Send-text, send-keys, raw-input, prompt, start, wait, clear, close, resize, and equivalent operations MUST preserve exact request/result correlation.
- **FR-058**: Duplicate/replayed consequential request IDs MUST follow documented idempotency or explicit rejection semantics.
- **FR-059**: Command-palette/custom-command entries MUST resolve to typed allowlisted Winds operations.
- **FR-060**: Spec 012 MUST NOT define a generic arbitrary shell dispatcher, dynamic method bus, public RPC server, plugin command bus, or remote command plane.
- **FR-061**: Any renderer/WebView bridge MUST expose the narrowest typed surface and validate privileged arguments outside untrusted renderer content.
- **FR-062**: No operation may implicitly fan out to multiple runtime/controller targets unless a future separately specified multi-target operation defines targets, authority, confirmation, and failure semantics.
- **FR-063**: Automation results MUST retain the immutable target snapshot used at request time and cannot be silently attributed to newly focused state.
- **FR-064**: Terminal/agent output MUST never become privileged command input merely because it matches action-like syntax.

## Worktree-Aware Workflows

- **FR-065**: Worktree discovery MUST bind an exact repository identity/root.
- **FR-066**: Repository trust MUST be explicit before create/open/remove can become consequential.
- **FR-067**: Worktree create/open/remove MUST validate path ownership, repository identity, and ambiguity under later Plan-defined safety rules.
- **FR-068**: Worktree removal MUST fail closed for ambiguous ownership, unsafe traversal, unexpected symlink/reparse behavior, or unproven cleanup.
- **FR-069**: Workspace/tab/pane/agent state MAY bind worktree identity but MUST NOT convert that binding into Git candidate/evidence authority.
- **FR-070**: Dirty/uncommitted state MUST be surfaced before any destructive worktree action authorized later.
- **FR-071**: Spec 012 worktree behavior MUST NOT merge, rebase, cherry-pick, push, create PRs, approve, or land code automatically.
- **FR-072**: Worktree root-directory configuration MUST be explicit, reversible, and unable to escape accepted path policy.
- **FR-073**: Removing UI metadata or saved layouts MUST NOT delete repository/worktree content.
- **FR-074**: Agent output requesting Git actions MUST remain untrusted and cannot bypass Git governance.

## Presentation, Accessibility, and Scale

- **FR-075**: Workspace/agent surfaces MUST use Winds-owned visual language and MUST NOT reproduce Herdr or another product's trade dress.
- **FR-076**: Critical runtime, detection, attention, stale, controller, and error states MUST be understandable without color alone.
- **FR-077**: Primary workspace/tab/pane/agent operations MUST have keyboard-reachable paths.
- **FR-078**: Pointer parity MUST exist for primary navigation/pane manipulation where pointer hardware is present.
- **FR-079**: Focus indicators MUST remain visible and deterministic across workspace, tab, pane, agent dock, palette, and overlays.
- **FR-080**: Reduced-motion/high-contrast/scaled-text modes MUST preserve critical authority/lifecycle distinctions.
- **FR-081**: Large-topology search/navigation MUST remain deterministic under duplicate names.
- **FR-082**: Hidden/background panes and agents MUST NOT trigger unbounded presentation work merely because output/events continue.
- **FR-083**: Scrollback, event queues, retained detection history, graphics buffers, and notification queues MUST have explicit bounded-resource behavior.
- **FR-084**: Performance claims MUST retain exact candidate, platform, build profile, fixture, measurement method, and result provenance.

## Platform and Security Truth

- **FR-085**: macOS, Linux, native Windows, and WSL claims MUST be bounded to directly exercised domains.
- **FR-086**: Native Windows and WSL MUST remain distinct for paths, input, cursor, process ownership, and security assertions.
- **FR-087**: Same-effective-user execution MUST NOT be described as a sandbox boundary.
- **FR-088**: Remote machines, SSH control, thin clients, remote owner setup/update, and cross-machine session control remain prohibited.
- **FR-089**: Generic integration install/uninstall/status lifecycle, plugin registry/runtime, marketplace/catalog, hooks/actions, and third-party dynamic execution remain prohibited.
- **FR-090**: Updater channels, live binary handoff, package/release promotion, and generic version orchestration remain prohibited.
- **FR-091**: Secrets/full environments MUST NOT be persisted into layout, pane, agent, or attention metadata merely for continuity or presentation.
- **FR-092**: Every later donor/dependency proposal MUST pass separate exact provenance/license/authority qualification before admission.

---

# Entry Question Resolutions

1. Workspace/tab/pane identities are immutable separate domains; none is an alias, Project/Session/runtime/provider/worktree/candidate identity.
2. Topology operations bind exact target plus generation and fail closed on staleness.
3. Read/list/explain/navigation/presentation are observer-safe; input/prompt/start/close/resize/clear and other consequential mutations require accepted controller/authority semantics.
4. Consequential operations default to one explicit target; no arbitrary shell/RPC/method bus is specified.
5. Detection states are observed, unknown, ambiguous, stale, and unavailable with provenance; terminal text alone is insufficient.
6. Trusted Needs You state comes from accepted structured state/events, never solely from agent prose.
7. All 24 families are required for detection/presentation; none receives execution authority at specification time.
8. Accepted PTY/ConPTY ownership is reused; terminal resources are bounded; clear/copy/selection/link/IME remain exact-target and presentation-safe.
9. Worktree operations require explicit repository trust and exact path identity; Git landing remains excluded.
10. The exact 142 ledger rows already marked Spec 012 are in scope; rows assigned to Specs 013-015 remain there.
11. Every claimed macOS/Linux/native-Windows/WSL behavior requires direct evidence in its own domain.
12. No dependency or migration is authorized here; every later proposal requires Plan/Tasks qualification.
13. YAGNI boundary: no remote transport, generic RPC, plugin marketplace/runtime, arbitrary shell bus, updater, orchestration framework, or automatic Git landing.
14. Later qualification must prove stale-target rejection, zero unintended cross-target dispatch, deterministic topology, bounded resources, and exact agent/worktree binding at scale.

---

# Success Criteria

- **SC-001**: Duplicate workspace/tab/pane labels plus reorder/rename operations produce zero identity confusion.
- **SC-002**: A stale pane request after close/replacement is rejected and never mutates the replacement pane.
- **SC-003**: Split/swap/move/resize/zoom/focus/close are deterministic under the Plan-defined topology fixture.
- **SC-004**: Layout export/apply/save proves topology restoration does not claim restored live process/controller authority.
- **SC-005**: Pane clear affects only its exact target with accepted screen/scrollback semantics.
- **SC-006**: Sustained terminal output stays within accepted per-pane resource ceilings.
- **SC-007**: All 24 agent families have positive/negative/ambiguous/stale qualification evidence before final support claims.
- **SC-008**: Forged agent/terminal labels such as VERIFIED, ACCEPTED, Needs You, provider, or model names cannot mutate trusted state.
- **SC-009**: Detector refresh failure preserves the last accepted set or exposes explicit unavailable state; malformed rules never partially activate.
- **SC-010**: Agent list/get/read/explain/focus/view/rename preserve exact pane/runtime/source binding under duplicate labels.
- **SC-011**: Detection support alone never produces a claim of real provider execution.
- **SC-012**: Needs You aggregation is deterministic and duplicate-free for identical canonical structured inputs.
- **SC-013**: A late attention/event result for A cannot be rendered as current truth for newly focused B.
- **SC-014**: Simultaneous multi-pane input tests prove zero unintended cross-pane dispatch.
- **SC-015**: Delayed/replayed request IDs cannot repeat consequential operations contrary to defined semantics.
- **SC-016**: Command-palette fuzzing cannot invoke unknown methods or arbitrary shell/RPC behavior.
- **SC-017**: Worktree create/open/remove rejects untrusted repositories and ambiguous/unsafe paths.
- **SC-018**: No Spec 012 worktree test performs merge, rebase, cherry-pick, push, PR creation, approval, or landing.
- **SC-019**: Local clipboard/link/graphics/notification fixtures cannot create remote-control authority or trusted-status forgery.
- **SC-020**: High-output plus slow-consumer stress keeps scrollback/event/detection/notification memory within Plan-defined bounds.
- **SC-021**: Large-topology fixtures remain searchable/navigable with duplicate names and exact selection.
- **SC-022**: Keyboard-only primary workflows complete with visible deterministic focus.
- **SC-023**: Pointer-primary topology manipulation completes without changing authority semantics.
- **SC-024**: CJK/IME and input-origin claims are directly exercised on every claimed platform.
- **SC-025**: Native-Windows results are not used as WSL evidence and WSL results are not used as native-Windows evidence.
- **SC-026**: Security fixtures prove untrusted terminal/agent/renderer content cannot forge trusted controls or invoke privileged generic dispatch.
- **SC-027**: Performance/resource campaigns retain exact environment and raw/lossless-enough evidence.
- **SC-028**: Final closeout accounts for all 142 Spec 012 ledger rows as implemented, intentionally non-applicable, explicitly deferred within-domain, or blocked by recorded authority—never silently omitted.
- **SC-029**: Final closeout preserves Spec 011 identity/authority/nonclaims including no PID-as-authority, no public network plane, no same-user sandbox claim, and no automatic landing.
- **SC-030**: Final implementation passes repository quality, applicable native-platform/security/stress workflows, author correctness/safety review, Ponytail/YAGNI review, fresh independent substantive review, and has zero unresolved material findings/threads.

---

# Explicit Non-Goals

Spec 012 does NOT by itself authorize or require:

- SSH transport, remote machines, remote execution, thin/mobile clients, cloud relay, remote owner management, or cross-machine control;
- public TCP/HTTP/WebSocket/RPC control planes;
- generic integration installer lifecycle for the 17 frozen targets or experimental Letta installer;
- plugin registry/runtime, hooks/actions/panes/link-handler marketplace, catalog, or arbitrary third-party dynamic code loading;
- updater channels, live binary handoff, package/release promotion, or generic update orchestration;
- automatic merge, rebase, cherry-pick, push, PR creation, approval, winner selection, or landing;
- a second terminal backend or process-ownership authority;
- automatic provider/model routing, learned routing, or provider/model truth inferred from detection;
- browser automation/runtime, Browser Twin, CDP, SQL Studio, MCP/A2A expansion, or generic service orchestration;
- a claim that worktrees, PTYs, local endpoints, or same-user boundaries are OS sandboxes;
- copying Herdr or any donor source merely because Founder permission exists;
- any dependency, migration, persistence schema, UI framework, terminal renderer, Git library, manifest engine, or hot-reload system before later Plan/Tasks qualification.

---

# Plan-Stage Decisions Required

The later Plan MUST decide with exact evidence:

1. the smallest Winds-native workspace/tab/pane state model and topology representation;
2. the stale-generation/request-binding mechanism;
3. layout serialization format, compatibility/versioning, and authority stripping;
4. terminal renderer integration with the accepted PTY/ConPTY owner;
5. bounded scrollback/graphics/event/detection-memory ceilings and backpressure;
6. precise pane-clear semantics;
7. link/clipboard/notification/sound host-action policy;
8. keyboard/mouse/IME/input-source qualification matrix;
9. agent detector representation, schema/versioning, validation, and refresh model;
10. how all 24 agent families are tested without turning provider strings into authority;
11. agent observation/confidence/stale-state model;
12. exact Needs You structured sources and deduplication rules;
13. agent dock/list/filter/sort behavior under duplicate labels and high scale;
14. whether any execution seam for agent start/prompt/wait/send-keys can be safely admitted per runtime without generic dispatch;
15. typed command-palette/custom-command registry and parameter validation;
16. event subscribe/wait queue bounds and request/result correlation;
17. repository trust model and worktree create/open/remove path-safety policy;
18. saved layout/workspace persistence location and whether a migration is necessary;
19. exact accessibility and performance acceptance environments;
20. exact platform qualification matrix for macOS/Linux/native Windows/WSL;
21. whether any Herdr slice is materially smaller/safer to reuse than Winds-native implementation; default is no code admission until an exact Task proves it;
22. removal/update/recovery strategy for every newly admitted dependency or persisted format.

Every dependency or donor-code proposal MUST record exact version/revision, license/notices, vendor/third-party provenance, security/authority impact, supported platform domain, update/removal path, and why accepted code is insufficient.

---

# Specification Acceptance Gate

This Spec may land only if its exact final candidate proves:

- canonical base is the post-merge-qualified Spec 012 Entry closeout or a governance-only forward descendant;
- changed scope is exactly specs/012-workspace-multiplexer-agent-plane/spec.md unless a tightly coupled governance-only correction is separately justified;
- the specification remains implementation-independent and does not smuggle in a framework, daemon, RPC server, plugin system, new persistence engine, or donor-code decision;
- all 14 Entry questions are answered explicitly;
- the exact 142 Spec 012 ledger rows are accounted for by scope, requirements, later qualification, or explicit non-goal/defer boundaries;
- all 24 agent families remain detection/presentation-only until later exact execution Tasks separately prove authority;
- immutable workspace/tab/pane identity and stale-target rejection are explicit;
- observer-safe/read-only behavior remains distinct from consequential controller actions;
- zero implicit multi-pane/multi-agent broadcast authority is introduced;
- terminal UX remains inside the accepted PTY/ConPTY and persistent-owner authority plane;
- worktree behavior does not acquire Git landing authority;
- remote transport, plugin/marketplace, updater/distribution, and automatic Git landing remain excluded;
- Herdr freshness is rechecked and material movement is reconciled instead of silently inherited;
- repository quality succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review challenges generic multiplexer-framework creep, generic RPC/command-bus creep, unnecessary persistence, scope leakage, and premature donor-code adoption;
- fresh independent substantive review challenges stale-target semantics, identity separation, detection truth, Needs You derivation, cross-target dispatch, worktree safety, resource bounds, platform truth, and non-goals;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation occurs immediately before landing;
- guarded normal merge uses the exact expected head;
- merge tree/ordered parents/GitHub signature are reconciled;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

~~~text
SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN_AUTHORIZED=YES
SPEC_012_TASKS_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO

HERDR_DIRECT_COPY_AUTHORIZED=NO
HERDR_ADAPTED_COPY_AUTHORIZED=NO
HERDR_TEST_PORT_AUTHORIZED=NO

REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_BINARY_HANDOFF_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
~~~

No source implementation may begin until a separate Plan and Tasks sequence independently qualifies and lands.
