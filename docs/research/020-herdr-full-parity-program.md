# Herdr Full-Parity Program — Founder-Directed Successor Roadmap

**Status:** Research/governance preparation only. Spec 010 is now closed canonically; this document remains non-authorizing until a separate successor entry gate lands.

**Prepared/refreshed:** 2026-09-18

**Winds canonical base inspected:** `7ed50fb6b173c4853394a93b51691e54fa463349`

**Herdr exact source pin:** `herdrdev/herdr@da6bcd5969779bfe0396bcf89a8025d4375d611e`

**Herdr tree:** `aece03633c003ba0fce01bc2564ad14e0fe9cac9`

**Herdr repository license:** Apache-2.0.

## 1. Founder direction

On 2026-09-16 the Founder directed Winds to reach full Herdr feature parity, not merely use Herdr as a research reference. The Founder also stated that they have permission to copy and use Herdr source code.

This direction establishes the product target, but it does not bypass Winds Constitution 1.1.0 or the required `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing` sequence.

The permission statement is recorded as **FOUNDER_PERMISSION_ASSERTION**. Public Apache-2.0 licensing is independently observable at the pinned repository. Every copied/adapted slice must still retain required notices/provenance and separately audit vendored or third-party code before reuse.

## 2. Program outcome

The terminal state of this program is:

```text
HERDR_CAPABILITY_INVENTORY=COVERED_OR_EXPLICITLY_SUPERSEDED_BY_STRONGER_WINDS_CAPABILITY
PERSISTENT_AGENT_RUNTIME=PROVEN
DETACH_REATTACH=PROVEN
MULTI_CLIENT=PROVEN
WORKSPACE_TAB_PANE_MULTIPLEXER=PROVEN
AGENT_DETECTION_AND_ATTENTION=PROVEN
LOCAL_AUTOMATION_CONTROL=PROVEN
REMOTE_SSH_MULTI_MACHINE=PROVEN
WORKTREE_WORKFLOWS=PROVEN
INTEGRATION_LIFECYCLE=PROVEN
PLUGIN_RUNTIME_AND_MARKETPLACE=PROVEN
LIVE_HANDOFF_AND_UPDATE=PROVEN
WINDOWS_MACOS_LINUX_CLAIMS=EXACTLY_EVIDENCED
WINDS_AUTHORITY_AND_VERIFICATION_INVARIANTS=PRESERVED
```

Parity means capability semantics and user outcomes, not visual cloning. Winds must retain its own brand, information architecture, canonical Project/Session identity, authority model, evidence model, and human-decision boundaries.

## 2A. Zero-omission capability ledger

The required parity scope is enumerated in `docs/research/021-herdr-exhaustive-capability-ledger.md`. That ledger is normative for omission prevention inside this research program: every recorded Herdr capability must reach `PROVEN_WINDS_PARITY`, `PROVEN_WINDS_SUPERSET`, or an explicit `FOUNDER_ACCEPTED_NOT_APPLICABLE` disposition before full-program closeout. A successor Spec may redesign a mechanism but may not silently remove a ledger row.

## 3. Current Herdr capability floor

The pinned source and tests establish the following parity floor for later formal specifications:

### Persistent runtime and session ownership

- background server owns PTYs/process state independently from clients;
- detach/reattach without killing pane processes;
- named runtime sessions and persistent state;
- multiple clients attached to one runtime;
- snapshot/session restore and native agent-session resume where supported;
- headless server mode;
- live server handoff, including current Windows transfer/handoff foundations.

### Workspace, tabs, panes, and terminal UX

- workspaces and tabs;
- recursive pane splits;
- focus, move, swap, resize, zoom, close, and split-ratio control;
- mouse and keyboard navigation;
- saved/applied layouts;
- copy/selection/search/link handling;
- terminal graphics/input compatibility, including Kitty-oriented paths and Windows input handling;
- popup/scratch-style pane surfaces and terminal metadata/title handling.

### Agent plane

The current detection enum contains 24 agent families:

`Pi`, `Claude`, `Codex`, `Gemini`, `Cursor`, `Devin`, `Antigravity`, `Cline`, `Omp`, `Mastracode`, `OpenCode`, `GithubCopilot`, `Kimi`, `Kiro`, `Droid`, `Amp`, `Grok`, `Hermes`, `Kilo`, `Qodercli`, `Qwen`, `Letta`, `Maki`, and `Muse`.

The current frozen serialized `IntegrationTarget::ALL` registry contains 17 targets:

`Pi`, `Omp`, `Claude`, `Codex`, `Copilot`, `Devin`, `Droid`, `Kimi`, `Opencode`, `Kilo`, `Hermes`, `Qodercli`, `Qwen`, `Cursor`, `Mastracode`, `AntigravityCli`, and `Grok`.

Herdr also exposes `Letta` as one experimental CLI-only installable integration deliberately kept outside the frozen generation-1 enum. The parity ledger therefore tracks the 17 frozen targets plus the experimental Letta install/status/uninstall lifecycle separately rather than hiding it in the 17-target count.

Parity must include:

- process/screen/OSC/hook-driven agent recognition where applicable;
- lifecycle states and attention aggregation;
- agent focus/navigation;
- agent prompt/start/explain semantics where supported;
- native-session identity/resume integrations where supported;
- source/confidence truth rather than pretending all detection is equally authoritative.

### Automation and local API

The pinned source exposes structured operations for at least:

- workspace and tab operations;
- pane read/focus/input/text/process/wait operations;
- layout export/apply/split ratios;
- agent explain/prompt/start and agent-view operations;
- events subscribe/wait;
- worktree operations;
- integration operations;
- plugin actions and managed plugin panes;
- server/config/manifest operations.

Winds parity requires CLI and private local-control semantics over the same canonical object model, with request identity, explicit errors, versioning, ownership/authentication, and least authority.

### Remote and multi-machine

- SSH-based remote attach;
- saved machines;
- multi-machine navigation from one client;
- remote host installation/setup;
- remote session continuity;
- clipboard/image/terminal bridging where supported;
- reconnect/error/backoff semantics;
- incremental surface transport and compatibility negotiation.

### Worktrees

- list/create/open/remove flows;
- workspace grouping by worktree;
- worktree-aware navigation and actions;
- race/error handling around concurrent create/remove and dirty state.

### Integrations and plugins; parity-plus marketplace target

- integration install/uninstall/status lifecycle;
- plugin manifests;
- link/install/list/unlink/enable/disable/update/uninstall semantics;
- platform/minimum-version constraints;
- startup hooks and event hooks;
- shareable plugin actions;
- managed plugin panes/popups/splits/tabs;
- link handlers;
- per-plugin runtime/config/path handling;
- GitHub-backed plugin installation/discovery using explicit source revision/provenance, plus local linking/registry semantics. The pinned Herdr source does **not** establish a distinct curated marketplace or catalog service; a broader Winds marketplace remains a Spec 014 parity-plus target rather than Herdr capability-floor evidence.

### Installation, update, handoff, and performance

- install/package paths across supported platforms;
- update/self-update behavior;
- live server handoff preserving compatible running panes;
- remote host package/update paths;
- render/surface reuse and incremental delta transport;
- hidden/background render discipline and explicit scaling benchmarks.

## 4. Winds parity-plus invariants

Winds must not copy weaker semantics together with source code.

1. Canonical `Project`, `Session`, workstream, candidate, and evidence identities remain separate from pane/process/vendor session IDs.
2. `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED` remains mandatory.
3. Agent `done` does not mean verified, accepted, or safe to land.
4. Raw terminal text never grants host authority, verification authority, or approval authority.
5. Worktrees provide Git isolation, not an OS sandbox claim.
6. Private IPC must be authenticated/owned, versioned, least-authority, bounded, and threat-modeled before implementation.
7. Remote transport authority may never exceed the explicit local human-approved ceiling.
8. Plugin discovery is not plugin trust. Capability declaration, provenance, integrity, update rollback, and revocation are required.
9. Native agent resume is labelled separately from live continuation and snapshot reconstruction.
10. Human landing/acceptance remains explicit unless a later separately accepted governance change states otherwise.

## 5. Founder desktop product direction carried into parity

The parity program must preserve the Founder-selected Winds desktop direction rather than importing Herdr's TUI appearance:

- monochrome black/charcoal/gray visual system; no default blue glow;
- black terminal output canvas and graphite user-input surfaces;
- left Workspace panel organized as `Project -> Sessions`;
- narrow far-left agent dock with compact logo + small label treatment;
- initial visible targets include `Auto`, `Codex`, `Claude`, and `OpenCode`, followed by `+ Add Agent`;
- Add Agent may later expose Herdr-supported agents plus Winds additions such as Mistral, Junie, Gemini CLI, Aider, Goose, and custom CLI targets only after source/install provenance is qualified;
- clicking an installed agent opens/targets its CLI immediately under the exact Winds Session;
- a missing agent produces an explicit install sheet showing official source, exact command/version, affected paths/config, and requiring user confirmation;
- terminal creation exposes profiles such as Ubuntu/WSL, system shell, zsh, bash, fish, and PowerShell where actually available;
- Project Files / folders are a collapsible right-side work surface;
- agent/runtime rails remain small and on-demand rather than turning Winds into a permanent chatbot dashboard;
- Winds `Auto` is a product/orchestration mode, but automatic target routing is not authorized merely by this research document;
- typography targets the clean modern feel identified by the Founder; any Avantt or other commercial font asset requires independent license/admission proof and must not be scraped from another company's site.

## 6. Proposed formal successor sequence

This sequence remains a research proposal until a post-T145 successor entry gate canonically accepts it. It intentionally decomposes the full parity goal so each high-authority boundary has its own threat model and evidence.

### Spec 011 candidate — Persistent Agent Runtime & Private Local Control

Scope target:

- persistent background owner for PTYs/processes;
- named runtime namespaces;
- detach/reattach;
- multi-client observer/controller ownership;
- durable topology/session metadata;
- bounded history replay;
- native agent-session resume distinctions;
- private authenticated/versioned local control plane and events;
- crash/restart/ownership-loss truth;
- no remote transport or generic plugin marketplace yet.

### Spec 012 candidate — Workspace Multiplexer & Agent Plane

Scope target:

- full workspace/tab/recursive-pane topology;
- split/move/swap/resize/zoom/focus/close;
- saved layouts and drag/mouse/keyboard operation;
- terminal profiles and advanced terminal UX parity;
- broad agent detection and lifecycle state;
- `Needs You` aggregation;
- structured pane/agent automation;
- worktree workflows;
- Founder compact agent dock and install/launch UX for explicitly qualified integrations.

### Spec 013 candidate — Remote Machines & Thin Clients

Scope target:

- SSH transport and saved machines;
- local thin client -> remote Winds owner;
- multi-machine navigation;
- remote owner setup/update;
- clipboard/image/file bridging where safe;
- reconnect/backoff/offline truth;
- remote session persistence;
- cross-platform transport/security qualification.

### Spec 014 candidate — Integration, Plugin & Marketplace Platform

Scope target:

- integration install/uninstall/status/update lifecycle;
- agent catalog expansion;
- plugin manifest/capabilities;
- install/link/list/enable/disable/update/uninstall;
- hooks/actions/managed panes/link handlers;
- marketplace/discovery;
- signatures/hashes/source revision/provenance;
- capability prompts, revocation, rollback, and supply-chain threat model.

### Spec 015 candidate — Live Handoff, Update & Full-Parity Closeout

Scope target:

- compatible live owner replacement without killing panes;
- Unix and Windows handoff where directly proven;
- atomic/rollback-safe update and remote-host update;
- incremental surface transport/render discipline;
- high-pane/high-client/huge-output performance campaigns;
- final Herdr capability matrix reconciliation;
- no parity claim until every required row is `PROVEN`, `SUPERSEDED_BY_STRONGER_WINDS_CAPABILITY`, or explicitly rejected by a separately accepted Founder/product decision.

## 7. Direct-code reuse policy

The Founder permits source copying, but Winds should copy only coherent slices that survive an admission review. For every copied/adapted Herdr slice, record:

- exact Herdr commit and source path(s);
- original license and required copyright/NOTICE text;
- any vendored/third-party provenance underneath that slice;
- reuse mode: `DIRECT_COPY`, `ADAPTED_COPY`, `TEST_PORT`, `DESIGN_REFERENCE`, or `WINDS_NATIVE_REIMPLEMENTATION`;
- why direct reuse is smaller/safer than reimplementation;
- Winds destination paths;
- security/authority delta;
- platform implications;
- deterministic tests and fault tests;
- removal/update path;
- correctness/safety, Ponytail/YAGNI, and independent-review disposition.

Do not bulk-copy the Herdr repository into Winds. Full feature parity does not require indiscriminate code transplantation.

## 8. Fresh-source reconciliation requirement

The prior roadmap used Herdr pin `ef2674bab8a3b38984578473c1a80589ebcbb333`. The superseded 2026-09-16 audit used `18061191fdc019498610aee81f0df93f6c2ebd31`. The current 2026-09-18 audit uses `da6bcd5969779bfe0396bcf89a8025d4375d611e`, tree `aece03633c003ba0fce01bc2564ad14e0fe9cac9`, which is another 21 commits ahead of the September 16 pin.

The 21-commit delta materially touches remote handoff/local fallback behavior, session deletion identity, worktree removal, release promotion and v0.9.1 metadata, OpenCode integration registration, saved-machine connection metadata, native-Windows terminal/input recovery, default shell selection, and Windows input qualification. The core inventory remains 24 agent families, 104 serialized public API methods, and 17 frozen integration targets, with one additional experimental CLI-only Letta integration surface. A formal successor entry must use this audit or a newer exact pin; it may not cite either older pin as current.


## 8A. Superseded research preservation

Draft PR #212 is preserved closed/unmerged as historical research. It was based on Winds `8d75cab...`, became 67 commits behind the Spec 010 closeout mainline, and pinned Herdr `18061191...`. It is not rewritten as current evidence. This refreshed program starts from canonical Winds `7ed50fb...` and current audited Herdr `da6bcd59...`.

The refresh also corrects an inventory ambiguity without rewriting history: the frozen serialized integration enum has 17 targets, while Letta is an additional experimental CLI-only installable target outside that enum. Later parity closeout must account for both surfaces.

## 9. Current authority firewall

As of this document's refreshed base:

```text
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL:YES
T145=CLOSED_CANONICAL:YES
SPEC_010_MERGE=7ed50fb6b173c4853394a93b51691e54fa463349
SPEC_010_POST_MERGE_QUALITY=35330722928 SUCCESS ATTEMPT_1
SPEC_011_ENTRY_AUTHORIZED:NO
HERDR_RUNTIME_CODE_ADMISSION_AUTHORIZED:NO
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED:NO
PRIVATE_IPC_IMPLEMENTATION_AUTHORIZED:NO
REMOTE_EXECUTION_IMPLEMENTATION_AUTHORIZED:NO
PLUGIN_MARKETPLACE_IMPLEMENTATION_AUTHORIZED:NO
```

The Founder direction establishes the product target but does not mutate the remaining authority facts. The next legal transition is a separate documentation-only successor entry-gate candidate. Canonical acceptance of that gate may authorize **Spec 011 specification creation only**; it must not authorize Plan, Tasks, source implementation, persistent runtime ownership, private IPC, remote execution, plugin runtime, or direct donor-code admission.

## 10. Definition of full program completion

The program is not complete because a GUI resembles Herdr or because copied modules compile. Completion requires:

- a current pinned Herdr feature inventory;
- every parity row reconciled to exact Winds evidence;
- persistent/local/remote/plugin threat models accepted;
- platform claims directly exercised;
- install/update/uninstall/recovery fault campaigns;
- multi-client and ownership races tested;
- no authority/evidence regression;
- exact dependency/license/provenance ledger;
- human UX acceptance for the final integrated desktop experience;
- canonical closeout declaring the exact parity matrix and remaining truthful nonclaims.
