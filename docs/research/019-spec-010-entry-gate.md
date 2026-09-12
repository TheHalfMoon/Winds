# Spec 010 Formal Entry Gate — Winds Desktop Agentic Workspace

**Status:** Governance entry candidate. Specification-only authority if canonically accepted.

**Canonical base at creation:** `10b7c7be292549e3a8f5315e7d513020be85b4a3`

**Date:** 2026-09-12

## 1. Purpose

Spec 009's first implementation program is canonically closed through T126 on `10b7c7be292549e3a8f5315e7d513020be85b4a3`, with post-merge repository `quality` successful on attempt 1.

The Founder now directs Winds toward a new product surface: the most beautiful, professional, and future-facing local interface for agentic software work. The goal is explicitly not another terminal emulator, editor skin, or chat sidebar. The next product layer should make terminal execution, agents, workflows, exact-candidate evidence, review, continuity, and human decisions legible as one coherent workspace.

This entry gate records that product-priority decision and may authorize only a separate Spec 010 specification candidate after this exact entry gate is independently qualified and canonically landed.

It does not authorize implementation, dependencies, a desktop framework, a daemon, IPC, background ownership, browser execution, remote execution, automatic routing, automatic Git landing, or any visual technology choice by itself.

## 2. Canonical inputs

The decision is bounded by:

- Winds Constitution 1.1.0;
- canonical Spec 007 Native Agentic Terminal UX closeout through T100;
- canonical Spec 008 Resumable Workflow & Decision Ledger closeout through T113;
- canonical Spec 009 Model Mesh & Explicit Multi-Provider Continuity closeout through T126;
- `docs/research/011-herdr-parity-and-beyond-roadmap.md`;
- `docs/research/012-agentic-era-terminal-north-star.md`;
- `docs/research/012-agentic-era-terminal-source-register.md`;
- `docs/research/014-loopforge-skillhone-roadmap-reconciliation.md`;
- `docs/research/015-selective-code-adoption-master-plan.md`;
- `docs/research/018-spec-009-entry-gate.md`;
- current accepted `winds workbench` TUI behavior and all canonical terminal/workflow/model-mesh truth boundaries;
- current public product-interface research used as non-canonical design input, including Linear's 2026 calmer UI refresh, Cursor 3's agent-first workspace, Raycast's command-first interaction model, Warp's agentic terminal/workspace direction, and current Tauri/React/Vite platform capabilities.

The inherited truth is:

```text
T114..T126=CLOSED_CANONICAL
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=CLOSED_CANONICAL
SPEC_009_TASKS=CLOSED_CANONICAL
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
CANONICAL_MAIN=10b7c7be292549e3a8f5315e7d513020be85b4a3
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

The desktop program must preserve those nonclaims and may not visually promote unproven agent/runtime state into trusted product truth.

## 3. Founder decision and roadmap supersession

Earlier research documents provisionally named `SPEC_010` as `Winds Continuum + durable local runtime owner`. Those documents explicitly described their sequence as future research ordering rather than implementation authority.

The Founder now supersedes that provisional ordering for the next formal specification stage:

```text
SPEC_010_NAME=WINDS_DESKTOP_AGENTIC_WORKSPACE
SPEC_010_FORMAL_SPEC_AUTHORIZED=YES_AFTER_THIS_ENTRY_LANDS
SPEC_010_PLAN_AUTHORIZED=NO
SPEC_010_TASKS_AUTHORIZED=NO
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
DURABLE_LOCAL_RUNTIME_OWNER_AUTHORIZED=NO
DAEMON_IPC_AUTHORIZED=NO
```

The durable-owner program is deferred, not rejected. If the later Spec 010 Plan proves that a particular P1 desktop requirement cannot be satisfied safely within the accepted one-process local architecture, that requirement must receive its own explicit architecture/governance decision before persistent ownership or IPC is introduced.

The default first-slice rule is therefore:

```text
PREMIUM_DESKTOP_UI=TARGET
ONE_PROCESS_LOCAL_ARCHITECTURE=PREFERRED_FIRST_SLICE
DESKTOP_PRESENTATION!=CANONICAL_AUTHORITY
VISUAL_STATUS!=VERIFICATION_EVIDENCE
AGENT_ACTIVITY!=HUMAN_ACCEPTANCE
```

## 4. Product thesis

Winds Desktop should be an **agentic workspace**, not a terminal application with extra chrome.

The interface should organize the user's work around five semantic planes:

1. **Mission** — what the user is trying to accomplish, current workflow/stage, and explicit next decision.
2. **Actors** — local agents/runtimes participating in the work, their exact source-labelled state, and their bounded authority.
3. **Work** — terminals, files, diffs, commands, artifacts, and browser/evidence views when separately authorized.
4. **Truth** — exact candidate, verification, provenance, blockers, stale state, source labels, and historical evidence.
5. **Command** — one universal keyboard-first surface for navigation, actions, search, and context-aware commands.

Terminal panes remain first-class when the work needs them, but they are not the application's visual or conceptual center.

## 5. Interface north star

The formal specification may define implementation-agnostic requirements for a premium desktop experience with the following qualities:

- calm, information-dense, low-noise visual hierarchy;
- cinematic polish without decorative motion that obscures state;
- instant keyboard-first navigation with pointer parity;
- a universal command palette/action surface;
- explicit context rather than hidden modes;
- flexible work surfaces that can host terminal, agent activity, workflow, diff/evidence, artifact, and future separately-authorized browser views;
- agent activity represented as structured work with provenance, not endless chat bubbles;
- exact distinction between `RUNNING`, `DONE`, `VERIFIED`, `ACCEPTED`, `BLOCKED`, `STALE`, `UNAVAILABLE`, and `OWNERSHIP_LOST` where applicable;
- source and authority visible on demand without permanently cluttering the canvas;
- beautiful dark and light appearance with platform-native window treatment where practical;
- typography, spacing, density, depth, iconography, animation, and focus behavior defined as a coherent Winds design language rather than framework defaults;
- accessibility, reduced motion, high contrast, keyboard-only operation, scalable text, and non-color-only state as product requirements;
- a responsive desktop layout that degrades gracefully across compact laptop and large-monitor workspaces;
- user-perceived latency targets that make common navigation and local UI actions feel immediate.

## 6. Proposed primary product surfaces

The separate Spec 010 specification may evaluate and define user stories for:

### A. Mission Control

A home/workspace surface showing active workstreams, stage truth, agent activity, blockers, exact candidate/evidence state, and items requiring human attention.

### B. Agent Canvas

A spatial/structured view of active and historical agent work across local runtimes, worktrees, stages, and artifacts. It must not imply authority or success from visual proximity, animation, or agent prose.

### C. Work Surface

A composable central canvas for terminal, diff, files, evidence, workflow, and future separately-authorized browser/reality surfaces. Pane layout is presentation state, not canonical identity.

### D. Truth Inspector

A persistent-on-demand evidence drawer that explains exact candidate/tree, verification runs, provenance, stale reasons, authority ceiling, continuity class, and why a claim is or is not trusted.

### E. Command Surface

A universal command/search palette that exposes navigation and only the actions already authorized by canonical product semantics. Search result ranking must not create execution, verification, acceptance, routing, or Git authority.

### F. Attention System

A bounded, quiet way to surface blocked work, approval requests, verification failures, stale evidence, ownership loss, and human decisions without converting every event into a notification stream.

## 7. Design language principles

The specification should require a Winds-owned design language with at least:

- **Quiet frame, vivid truth:** navigation and chrome recede; selected work and important truth receive contrast.
- **Depth from material hierarchy, not glass everywhere:** translucency may be used selectively, but legibility and platform performance win.
- **Motion explains state:** transitions communicate spatial continuity, ownership change, focus, or lifecycle movement; decorative perpetual animation is prohibited by default.
- **Density is adjustable:** compact professional density is the default; spacious modes may exist without changing semantics.
- **One accent, many semantic states:** brand color does not replace status semantics; state colors always have icon/text equivalents.
- **Terminal data stays terminal data:** forged ANSI/text cannot become trusted Winds labels, badges, notifications, hyperlinks, or host actions.
- **Agent output stays agent output:** model prose never becomes a trusted verification/acceptance visual merely because it is rendered in a premium surface.
- **The interface is reversible:** presentation choices must not lock canonical state into a UI-only schema.

## 8. Architecture candidates for Plan-stage evaluation

This entry gate does not select implementation technology. The later Plan may compare at least:

- Tauri 2.x desktop shell with the existing Rust core;
- React 19.x + TypeScript as a rich presentation layer;
- Vite 8.x or an equivalent bounded build system;
- a terminal rendering library only if exact dependency/provenance/security/platform review proves it preferable to reusing the accepted TUI/PTY seams through a bounded bridge;
- a small Winds-owned design system rather than adopting a full generic component framework by default.

The Plan must justify any dependency by current P1 requirements, exact version/license/MSRV/platform support, bundle/runtime cost, accessibility, security, removal path, and why platform/native/existing code is insufficient.

The Plan must explicitly evaluate whether Tauri/WebView IPC is sufficiently narrow for the first slice and must treat every UI-to-Rust command as an authority boundary. No remote origin may receive local Winds command authority.

## 9. Explicit non-goals for the first formal specification

Unless a later accepted amendment changes scope, Spec 010 does not automatically authorize:

- a persistent daemon or `windsd`;
- public IPC, HTTP, WebSocket, local-network, or remote-control protocol;
- cross-restart ownership of live terminal or agent processes;
- remote/mobile clients;
- browser automation/runtime or Browser Twin;
- SQL Studio;
- generic plugin/extension marketplace;
- generic provider SDK/gateway fleet;
- learned/automatic model routing or winner selection;
- training, fine-tuning, RL, autonomous skill mutation, or semantic memory;
- automatic merge/rebase/cherry-pick/push/PR creation/landing;
- hidden telemetry or mandatory cloud account/control plane;
- copying closed-source/proprietary UI assets or AGPL application code into the Winds codebase;
- weakening any existing verification, Git, authority, secret, terminal, workflow, model-mesh, platform, provenance, recovery, or human-decision boundary.

## 10. Quality bar

The formal specification must define measurable outcomes for at least:

- cold launch and first-interactive readiness;
- local navigation/action latency;
- animation/frame responsiveness under representative workload;
- memory footprint and idle CPU bounds;
- terminal/input correctness where terminal views are present;
- keyboard and pointer reachability;
- reduced-motion/high-contrast/scalable-text behavior;
- correct rendering of lifecycle/evidence/authority states;
- no authority escalation through renderer content, forged labels, URLs, clipboard requests, drag/drop, or remote WebView content;
- exact cross-platform claims for macOS, Windows, and Linux only where directly exercised;
- deterministic visual/state fixtures where pixel-level exactness is appropriate and semantic accessibility assertions where pixels are not authoritative.

Aesthetic quality must be reviewed explicitly. “Functional” is not sufficient acceptance for the desktop product.

## 11. Entry acceptance gate

This entry gate may land only if the exact final candidate proves:

- changed scope is exactly this governance/research document;
- canonical base descends from the complete Spec 009 T126 closeout;
- the old provisional `Spec 010 = durable owner` ordering is explicitly superseded rather than silently ignored;
- durable owner/daemon/IPC remains unauthorized;
- no production source, dependency, lockfile, migration, workflow semantic, runtime/provider/model, browser, remote, learning, credential, or automatic-landing mutation;
- repository `quality` succeeds on the exact final head;
- correctness/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review challenges whether the first slice can remain one-process and whether any visual/platform abstraction is premature;
- fresh independent substantive review challenges the product thesis, roadmap supersession, authority boundaries, UI trust model, architecture non-authorization, and quality bar;
- zero unresolved material findings/threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal landing;
- canonical merge identity/tree/ordered parents/signature verification;
- every actually-triggered applicable post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_010_FORMAL_SPEC_AUTHORIZED=YES
SPEC_010_PLAN_AUTHORIZED=NO
SPEC_010_TASKS_AUTHORIZED=NO
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
SPEC_010_DESKTOP_PRODUCT_DIRECTION=WINDS_DESKTOP_AGENTIC_WORKSPACE
DURABLE_LOCAL_RUNTIME_OWNER_AUTHORIZED=NO
DAEMON_IPC_AUTHORIZED=NO
BROWSER_RUNTIME_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
AUTOMATIC_ROUTING_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

## 12. Entry effect

If this gate closes canonically, the next authorized repository unit is a separate Spec 010 `spec.md` candidate only.

That specification must start from then-current canonical `main`, remain implementation-agnostic, define independently testable desktop user stories and measurable design/product outcomes, preserve all existing trust/nonclaim boundaries, and pass exact-candidate acceptance gates before any Plan or implementation begins.
