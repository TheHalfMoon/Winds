# Herdr Parity and Beyond Roadmap

**Status:** Research-only, non-authorizing product roadmap.

**Research date:** 2026-09-01.

**Canonical Winds base inspected:** `06e515471cf91a0f1d5b257d6e9820096d9a0197`.

**Herdr source pin inspected:** `herdrdev/herdr@ef2674bab8a3b38984578473c1a80589ebcbb333` plus live public documentation retrieved on 2026-09-01.

## Scope firewall

This document does **not** amend the Winds Constitution, Spec 006, plan, or tasks. It does **not** authorize a daemon, public/local IPC, remote execution, plugin runtime, marketplace, terminal renderer, additional agent integration, migration, dependency, runtime/model call, or implementation work. The current Spec 006 dependency ladder remains authoritative.

Any implementation derived from this roadmap MUST enter the normal `Constitution -> Spec -> Plan -> Tasks -> Implement` sequence after its prerequisites are canonically complete. In particular, current T079/T080 evidence/review gates are not widened or bypassed.

## 1. Executive conclusion

Herdr validates a major product thesis that Winds already identified: coding agents are becoming long-lived terminal runtimes rather than isolated one-shot commands. The important Herdr innovation is not its TUI appearance. It is the runtime model beneath the TUI:

- a persistent server owns PTYs and process/session state;
- clients become detachable views rather than process owners;
- workspaces, tabs, panes, and agents are explicit runtime objects;
- agents are classified into attention-relevant lifecycle states;
- automation uses first-class workspace/pane/agent primitives rather than blind keystroke scripting;
- local, SSH, thin-client, and programmatic clients can drive the same runtime;
- agent-native session references can restore supported agents after server restart;
- a plugin and integration ecosystem keeps the core small while extending workflows.

Winds should reach feature parity with this runtime class, then exceed it by preserving what Herdr does not make its primary moat: canonical workstream identity, authority provenance, evidence classes, exact-candidate verification, reviewer independence, explicit human decisions, and safe cross-runtime continuity.

Recommended product position:

> **Winds is the verified execution environment for developers and coding agents.**
>
> **Any agent. Persistent work. Explicit authority. Verified results.**

The target is not to clone Herdr's visual identity or blindly transplant its implementation. Feature semantics should be independently implemented inside Winds' safety and evidence model. Herdr is Apache-2.0, so any direct source reuse that is ever chosen must retain required license/provenance notices and must still pass Winds' independent review and dependency/scope gates.

## 2. Why Herdr has strong product pull

The public YC description and Herdr's own launch/funding writing show a coherent wedge:

1. Developers increasingly run fleets of coding agents for hours or days.
2. The bottleneck moves from model capability to runtime management: which agent is working, blocked, done, or lost in a terminal.
3. Agent work should survive client disconnects, closed laptops, and dropped SSH sessions.
4. Users should keep their existing agent CLIs rather than adopt yet another proprietary agent.
5. A background runtime can serve many clients and automation surfaces.
6. A small open core plus integrations/plugins can create distribution and ecosystem effects.
7. The product already demonstrated strong public pull before YC: substantial stars, downloads, integrations, and community plugins.

The exact internal YC investment memo is not public, so any statement about YC's private reasoning would be speculation. The defensible inference is that Herdr sits at the intersection of a fast-growing behavior change (parallel long-lived coding agents), a painful operational problem (terminal/session sprawl), an agent-agnostic infrastructure wedge, open-source distribution, and visible early traction.

## 3. Herdr feature inventory Winds should match or surpass

### 3.1 Persistent runtime and session ownership

Target parity:

- background runtime owns PTYs, panes, process lifecycle, and live session state;
- attach/detach without killing pane processes;
- named sessions with separate runtime namespaces;
- multiple clients attached to one runtime;
- snapshot restore of workspace/tab/pane topology, cwd, focus, and labels;
- optional bounded pane-history replay with explicit privacy warnings;
- native agent-session restore where a supported integration reports a trustworthy native session reference;
- live server handoff/update path where processes survive compatible updates;
- explicit distinction between live continuation, snapshot reconstruction, and native agent resume.

Winds-beyond requirement:

- preserve canonical Winds `workspace_id`, `workstream_id`, and `session_id` independently from native runtime/process/session IDs;
- label every restore result truthfully (`LIVE_CONTINUED`, `NATIVE_RESUMED`, `RECONSTRUCTED`, `UNAVAILABLE`, etc.);
- never convert pane history or agent prose into authoritative evidence.

### 3.2 Workspace / tab / pane multiplexer

Target parity:

- project workspaces;
- tabs per workspace;
- recursive split panes;
- split right/down and directional focus;
- resize, swap, move, zoom/focus, close, rename;
- layout export/apply and split-ratio control;
- mouse-native pane focus, border drag, selection, and context menus;
- keyboard-first navigation and tmux-style prefix compatibility;
- pane-local cwd/process/agent metadata;
- direct terminal attach by pane/terminal target;
- read-only observers and a single writable controller ownership model.

Winds-beyond requirement:

- premium native GUI plus TUI/CLI surfaces over the same semantic layout model;
- Smart Split that chooses placement from available geometry and minimum usable pane sizes;
- drag-and-drop pane restructuring with deterministic recursive topology;
- saved named layouts and workspace-specific layouts;
- explicit controller ownership and authority provenance for every interactive attach.

### 3.3 Agent detection and lifecycle state

Target parity:

- broad coding-agent recognition;
- screen/terminal-manifest detection when no authoritative lifecycle hook exists;
- direct lifecycle integrations when vendors expose adequate hooks;
- separate native-session identity integrations for restore;
- custom agent labels/categories;
- explicit agent states such as `working`, `blocked`, `done`, `idle`, and `unknown`;
- state rollups from pane -> tab -> workspace;
- focus directly to the agent/pane requiring attention;
- sandbox/wrapper hints where the real process is hidden from host detection.

Winds-beyond requirement:

- every state carries source/provenance and confidence/enforcement class;
- distinguish `AGENT_REPORTED`, `WINDS_OBSERVED`, and `HUMAN_DECIDED` state;
- `blocked` / `needs_you` should identify the exact blocking request rather than only color a pane;
- agent state cannot authorize execution or acceptance.

### 3.4 Attention queue

Target parity:

- aggregated view of working/blocked/done agents;
- filters and quick navigation;
- no need to poll every pane manually;
- notifications only when attention is meaningful.

Winds-beyond requirement:

- a first-class `Needs You` queue with exact reason, requested capability, task/workstream identity, candidate identity where applicable, and one-click focus;
- approval/deny controls only when the request is content-bound to a reviewed approval digest;
- recently completed results remain non-authoritative until verification gates succeed.

### 3.5 Agent automation

Target parity should expose three distinct primitives:

- layout: workspaces/tabs/pane topology;
- pane: raw terminal input/output/process control;
- agent: recognized coding-agent lifecycle operations.

Required operations should cover, where safe and formally authorized:

- create/list/get/focus/rename/move/close workspace and tabs;
- split/swap/move/zoom/resize/read/focus panes;
- send text, keys, or bounded input;
- wait for output;
- inspect process and pane metadata;
- list/get/read/explain/focus agents;
- prompt/wait/start/rename supported agents;
- event subscribe/wait;
- layout export/apply;
- machine-readable JSON output.

Winds-beyond requirement:

- structured vendor-native protocols are preferred over scraping where available;
- automation must pass the Winds authority evaluator;
- one agent prompting another is delegation and must respect explicit delegation ceilings;
- raw `send_keys` is a weaker control surface and must never be treated as equivalent to a structured accepted turn.

### 3.6 Local control API

Target parity:

- local socket/control surface with request IDs and machine-readable responses;
- CLI and local API cover the same semantic objects;
- events subscription/waiting;
- terminal read/observe/control;
- plugin and integration operations;
- API context derives from explicit active workspace/tab/pane when appropriate.

Winds-beyond requirement:

- do not introduce this during current Spec 006;
- future private IPC must have an explicit threat model, versioning, authentication/ownership semantics, lifecycle design, rate/boundary limits, and least-authority defaults;
- public protocol comes only after the private surface proves a concrete need.

### 3.7 Remote execution and thin clients

Target parity:

- normal SSH + remote Herdr/Winds process workflow;
- local thin client attaching over SSH;
- remote server starts/attaches automatically when appropriate;
- local desktop bridges for clipboard/image/file transfer where safe;
- named session targeting on remote hosts;
- remote sessions survive the local client disappearing;
- Windows/macOS/Linux client interoperability where proven.

Winds-beyond requirement:

- remote execution remains a separate future specification;
- transport authentication, host identity, capability delegation, secret boundaries, file-transfer rules, controller ownership, and audit evidence must be explicit;
- remote runtime never silently gains more authority than the local human-approved ceiling.

### 3.8 Git worktrees

Target parity:

- list/create/open/remove worktrees;
- group worktree-derived workspaces coherently;
- report Git/worktree provenance into automation context;
- shortcuts/context actions to create isolated worktree workflows.

Winds-beyond requirement:

- retain Winds' existing rule: worktrees are Git/workspace isolation, not OS sandboxes;
- bind worktree, exact Git base, candidate, and verification evidence explicitly;
- never force-clean or silently delete ambiguous worktree state.

### 3.9 Terminal UX

Target parity:

- mouse-first and keyboard-first workflows;
- drag split borders;
- pane selection and right-click menus;
- copy-on-select and copy mode;
- literal forward/backward scrollback search;
- hyperlinks;
- configurable pane scrollbars;
- status/tab bars;
- configurable themes and custom color variables;
- terminal query compatibility and correct host appearance propagation;
- CJK/IME and enhanced keyboard handling;
- Kitty graphics where supported;
- terminal/window title synchronization;
- configurable pane borders and sidebar colors;
- popup/scratch terminals;
- static state symbols rather than decorative constant animation.

Winds-beyond requirement:

- native GUI should remain terminal-first and visually quieter than an IDE;
- Winds Explorer and Chat Dock are context surfaces, not permanent editors;
- terminal output hover can expose `Copy`, `Ask`, `Pin`, and metadata without turning commands into large cards;
- Chat -> `Run` / `Run in...` and terminal error -> `Ask` should be first-class contextual bridges;
- accessibility, reduced motion, high contrast, and keyboard-only operation are mandatory product quality gates.

### 3.10 Performance and rendering discipline

Herdr's published optimization work demonstrates an important principle: background output should not force unnecessary rendering for every attached client.

Target requirements:

- server parses necessary PTY output but only renders/transmits what a client can actually see or explicitly observes;
- hidden panes do not trigger full visible-frame work;
- passive mouse movement does not cause continuous repaint;
- static agent-state symbols do not animate when nothing meaningful changed;
- many clients should not multiply unnecessary render cost;
- output-heavy/huge-scrollback cases get explicit performance budgets and soak tests.

Winds-beyond requirement:

- measure server, renderer, serialization, transport, and client costs separately;
- publish repeatable performance evidence rather than vague "fast" claims;
- define idle CPU/RAM, 10-agent, 50-pane, huge-output, resize, and multi-client budgets before claiming production readiness.

### 3.11 Integrations

Target parity:

- install/uninstall/status lifecycle and native-session integrations;
- support both lifecycle-authoritative hooks and session-identity-only hooks;
- do not overclaim hook coverage when terminal/screen state remains the actual lifecycle authority;
- vendor configuration modifications must be narrow, reversible, and ownership-tagged;
- unsupported agents still run normally as terminal processes.

Winds-beyond requirement:

- first-class integrations remain capability-driven, not a generic adapter abstraction before concrete need;
- each integration records runtime identity, version, provenance, lifecycle authority, native-session semantics, and uninstall/recovery behavior;
- formal targets after Codex/Claude should be selected from real usage and protocol quality, not logo count.

### 3.12 Plugins and marketplace

Target parity eventually:

- plugin manifest;
- install/link/list/unlink/enable/disable/update/uninstall;
- GitHub repository/subdirectory installs;
- platform/minimum-version declarations;
- startup hooks;
- event hooks;
- shareable actions;
- managed plugin terminal panes;
- link handlers;
- per-plugin config/state directories;
- logs;
- marketplace discovery/index;
- exact source revision recorded;
- architecture-aware assets and integrity hashes where binaries are downloaded.

Winds-beyond requirement:

- this is explicitly **not authorized by current Spec 006**;
- plugin capabilities must be declared and bounded;
- plugin execution authority must flow through the same Winds policy system as agents;
- marketplace listings are discovery, not trust;
- signatures/hashes/provenance, update rollback, dangerous capability disclosure, and revocation need a formal supply-chain model;
- extensions must not be able to rewrite protected Winds evidence/authority state.

### 3.13 Installation, updates, and handoff

Target parity:

- single native binary experience where possible;
- macOS/Linux/Windows installers and package-manager paths;
- checksum-verified release assets;
- atomic/rollback-safe activation;
- live handoff path where compatible server upgrades can preserve running processes;
- remote helper installation/update integrity;
- configuration validation before activation.

Winds-beyond requirement:

- release provenance and exact binary identity should be observable and reportable;
- failed update cannot strand or silently kill active sessions without truthful recovery state;
- migration/update behavior needs deterministic fault tests.

## 4. What Winds should deliberately not copy

Winds should not copy Herdr's branding, visual identity, textual copy, or incidental implementation choices. More importantly, Winds should not inherit weaker semantic assumptions when Winds already has stronger invariants.

Do not collapse:

- native agent session ID into canonical Winds session/workstream identity;
- terminal detection into verification evidence;
- `done` into accepted/correct;
- direct terminal access into safe delegated authority;
- Git worktree into sandbox;
- plugin installation into plugin trust;
- remote connectivity into remote authorization;
- agent prompt completion into candidate acceptance.

## 5. Winds parity-plus architecture

Recommended conceptual layers after the required formal specifications exist:

```text
Winds Clients
  Native GUI | TUI | CLI | Local API | Remote Thin Client
            |
            v
Private Control Plane
  workspace | layout | pane | agent | attention | event | session
            |
            v
Persistent Execution Host
  PTY/ConPTY | process ownership | terminal state | cwd | transport
            |
            +----------------------+
            |                      |
            v                      v
Agent Integration Plane        Git/Workspace Plane
Codex | Claude | follow-ons     repo | worktree | candidate identity
            |                      |
            +----------+-----------+
                       v
Canonical Winds Plane
workstream/session continuity | authority | evidence | human decisions
                       |
                       v
Verification Plane
exact candidate -> deterministic checks -> independent review -> human landing
```

The architectural moat is the bottom half. Herdr proves the upper runtime half matters; Winds should connect that runtime experience to a stronger evidence and authority model.

## 6. Ordered post-Spec-006 program

The following is a proposed sequencing concept, **not Tasks authorization**.

### H0 — Reconciliation and threat-model entry

- reconcile this research against then-canonical Winds and current Herdr;
- decide Constitution amendments, if any;
- define client/server trust, ownership, local-control authentication, persistence, update, and recovery boundaries;
- pin dependencies/protocols only when concrete.

### H1 — Persistent owner walking skeleton

- one local persistent host;
- one workspace;
- one pane;
- detach/reattach;
- truthful process ownership and recovery;
- no plugin/remote/generic runtime surface.

### H2 — Recursive terminal multiplexer

- workspace/tab/pane topology;
- split/resize/swap/move/focus;
- saved layouts;
- direct pane attach/observer/controller ownership;
- GUI and TUI share model semantics.

### H3 — Agent attention plane

- lifecycle/status source model;
- Codex/Claude exact integrations first;
- `Needs You` queue;
- pane/tab/workspace rollups;
- explicit source/confidence labels.

### H4 — Automation surface

- structured local CLI/control operations for layout, pane, agent, wait/events;
- authority evaluation on every consequential operation;
- private local API only after threat-model acceptance.

### H5 — Git worktree execution ergonomics

- worktree create/open/remove with Winds Git safety;
- workspace grouping;
- exact candidate/base linkage;
- no sandbox overclaim.

### H6 — Remote thin client

- SSH transport first;
- host identity/authorization model;
- remote attach/detach;
- safe clipboard/file bridge;
- cross-platform evidence.

### H7 — Update/handoff/recovery hardening

- server handoff;
- atomic updates;
- state migration/fault injection;
- pane history privacy controls;
- native agent restore/reconstruction truth.

### H8 — Integration SDK

- versioned integration manifest;
- lifecycle versus session-authority distinction;
- reversible vendor hook installation;
- narrow capability set.

### H9 — Plugin host

- out-of-process plugin actions/hooks/panes;
- capability permissions;
- per-plugin config/state/logs;
- supply-chain provenance and revocation;
- no protected-state writes.

### H10 — Marketplace

- discovery index, exact commit pin, compatibility/platform metadata;
- explicit "listed != trusted" UX;
- integrity and rollback UX.

### H11 — Native premium client

- Winds visual system;
- Explorer context dock;
- lifted Chat Dock;
- recursive split manipulation;
- command palette/quick open;
- session/attention control surfaces;
- terminal <-> chat contextual bridge;
- accessibility and native interaction polish.

### H12 — Production qualification

- Linux/macOS/Windows and WSL evidence;
- 10-agent / 50-pane / multi-client soak and performance budgets;
- disconnect/reconnect/update/fault tests;
- remote security tests;
- plugin supply-chain tests;
- privacy/redaction tests;
- exact documentation/claim reconciliation.

## 7. Product features that should make Winds better than Herdr

Herdr parity is the floor, not the strategy. Winds should add:

- **Verified completion:** an agent can be `done` while the work remains `UNVERIFIED`; verification is a separate visible state.
- **Authority-visible execution:** every agent/pane action can reveal who requested it, under which grant, and what enforcement level actually exists.
- **Canonical cross-agent continuity:** Codex -> Claude -> another runtime can continue one Winds workstream without pretending native session continuity.
- **Exact candidate awareness:** terminal/workspace context knows the Git base/candidate and can attach deterministic evidence.
- **Independent review surface:** review results are evidence tied to an exact candidate, not chat decoration.
- **Needs You with content-bound approval:** the queue can safely approve a specific action without creating a reusable blanket permission.
- **Terminal ↔ Chat bridge:** select error/output -> `Ask`; agent suggests command -> `Run in...` a chosen terminal; all without turning Winds into a chatbot.
- **Smart Split and task-aware layouts:** geometry is optimized for active work rather than just manual pane creation.
- **Context Explorer, not IDE replacement:** files/folders become paths, Git context, agent context, and terminal launch points without a full editor dominating the product.
- **Verified learning later:** only after the separate research roadmap's protected evaluation gates exist, successful verified experience may propose improvements without self-authorizing them.

## 8. Competitive bar

A future Winds release should not claim Herdr-class parity until it can demonstrate, with reproducible evidence:

- persistent detach/reattach of real terminal work;
- recovery semantics across server restart;
- recursive layout manipulation;
- many simultaneous panes/agents without uncontrolled CPU/render amplification;
- accurate blocked/working/done/idle/unknown state with source labels;
- machine automation that is safer than blind keystrokes;
- local and remote ownership rules;
- at least Codex and Claude real-runtime integration;
- exact Git/workstream/session continuity;
- explicit authority/approval behavior;
- deterministic verification separated from agent completion;
- cross-platform claims that match actual tested platforms.

## 9. Primary sources

Herdr repository/source:

- `https://github.com/herdrdev/herdr` — inspected at `ef2674bab8a3b38984578473c1a80589ebcbb333`
- `README.md`
- `CHANGELOG.md`
- `Cargo.toml`

Herdr documentation, retrieved 2026-09-01:

- `https://herdr.dev/docs/`
- `https://herdr.dev/docs/concepts/`
- `https://herdr.dev/docs/agents/`
- `https://herdr.dev/docs/agent-automation/`
- `https://herdr.dev/docs/session-state/`
- `https://herdr.dev/docs/persistence-remote/`
- `https://herdr.dev/docs/socket-api/`
- `https://herdr.dev/docs/cli-reference/`
- `https://herdr.dev/docs/integrations/`
- `https://herdr.dev/docs/marketplace/`
- `https://herdr.dev/docs/configuration/`
- `https://herdr.dev/docs/config-reference/`
- `https://herdr.dev/docs/keyboard/`
- `https://herdr.dev/docs/windows-beta/`

Product/YC sources, retrieved 2026-09-01:

- `https://www.ycombinator.com/companies/herdr`
- `https://herdr.dev/`
- `https://herdr.dev/blog/herdr-is-joining-y-combinator/`
- `https://herdr.dev/blog/ten-agents-three-clients-95-percent-less-cpu/`

## 10. Final recommendation

Do not compete with Herdr by merely recreating its TUI. Absorb the runtime lessons, reach semantic feature parity in formally authorized slices, then combine them with Winds' existing evidence/authority architecture and the premium native terminal client direction.

The durable target is:

> **Herdr-class persistence and orchestration + best-in-class terminal UX + Winds-class authority and verification.**

That combination is materially stronger than a terminal multiplexer, an agent manager, or a verification CLI by itself.
