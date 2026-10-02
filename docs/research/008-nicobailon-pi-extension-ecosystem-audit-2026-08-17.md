# Winds Research Archive — Nico Bailon Pi Extension Ecosystem Audit

**Status:** Research archive only. Non-normative. This document does **not** amend the Winds Constitution, Spec 003, plan, or tasks and does not authorize Agent Fleet, MCP, daemon, plugin-runtime, memory, or remote-ingress product code.

**Audit date:** 2026-08-17

**Subject:** `https://github.com/nicobailon`

**Purpose:** Preserve a broad audit of the Pi-oriented repositories and extensions published or mirrored under Nico Bailon's GitHub account and classify their future relevance to Winds.

---

## 1. Executive decision

The Nico Bailon Pi ecosystem is not a small collection of cosmetic extensions. Taken together, it covers most of the product primitives relevant to a future Winds control plane:

- heterogeneous agent/subagent execution;
- observable interactive CLI control;
- multi-agent messaging, coordination, roles, reservations, and review loops;
- model and skill routing;
- durable session-safe memory;
- compaction and context lifecycle;
- human clarification and approval surfaces;
- rewind/session-lineage ideas;
- usage/cost observability;
- MCP/tool discovery and adaptation;
- remote ingress experiments;
- terminal/TUI ergonomics.

The strongest conclusion is **not** “copy all Pi extensions.” The correct Winds use is selective donor/reference extraction behind Winds' stricter evidence and ownership boundaries.

The top future research targets are:

1. `pi-subagents` — fleet execution/delegation architecture.
2. `pi-interactive-shell` — observable external-CLI execution UX and lifecycle reference.
3. `pi-memory-workbench` — session-safe durable memory and append-only source lineage.
4. `pi-messenger` — multi-agent coordination, roles, file reservations, approval flows.
5. `pi-prompt-template-model` — declarative model/skill/subagent/worktree workflow policy.
6. `pi-mcp-adapter` — future token-efficient MCP/tool interoperability.

These are future Agent Fleet / interoperability / memory research inputs. They are not authorization to contaminate the active Spec 003 execution-spine slice.

---

## 2. Hard Winds scope firewall

This archive must not be interpreted as permission to:

- add a daemon to current Spec 003;
- add a public IPC/runtime protocol;
- add MCP/A2A/ACP integration to current Spec 003;
- add a generic extension/plugin runtime;
- infer exact command telemetry from PTY keystrokes or terminal text;
- modify shell dotfiles/profiles for telemetry;
- replace Winds' exact execution evidence with agent/self-reported status;
- import destructive Git rewind behavior into the primary checkout;
- add persistent fleet memory before a dedicated spec defines authority and retention;
- bypass donor license/provenance checks.

Current Winds evidence/ownership semantics remain stronger than several donor implementations and must win whenever they differ.

---

## 3. Tier S — deepest future donor/reference audit

### 3.1 `nicobailon/pi-subagents`

Classification: **TIER S — highest-priority Agent Fleet donor/reference**.

Observed capabilities include focused child Pi sessions, foreground/background execution, parallel and chained workflows, role-oriented agents, model overrides/fallbacks, worktree isolation, parent/child coordination, fleet inspection, steering, watchdog behavior, artifacts, events, logs, acceptance gates, and bounded spawn/concurrency behavior.

Its documentation surface is unusually mature and includes dedicated material for agents, configuration, extension API, missions, models, observability, tool reference, watchdog, and workflows.

High-value Winds seams to audit next:

- task/run state model;
- fleet lifecycle and concurrency bounds;
- machine-readable run artifacts/events;
- external CLI runner adapters;
- worktree isolation;
- parent/child escalation and steering;
- watchdog/adversarial review;
- model/provider selection and fallback;
- fleet TUI observability.

Provenance snapshot: GitHub reports `fork: false` and MIT license metadata. Direct reuse is therefore plausible subject to exact-path attribution and dependency audit.

Winds divergence: subagent completion/reviewer status is not authoritative by itself. Winds must bind it to independently observed execution/check/Git evidence before promotion.

### 3.2 `nicobailon/pi-interactive-shell`

Classification: **TIER S — external-agent execution UX/reference**.

Observed value:

- controls interactive CLIs in an observable terminal overlay;
- supports user takeover;
- foreground, hands-free, dispatch, and monitor-style workflows;
- structured launching for multiple agent CLIs and arbitrary commands;
- background dispatch and attach-later behavior;
- worktree-oriented flows;
- terminal monitoring/event concepts.

This is highly aligned with the long-term Winds goal of controlling Codex, Claude Code, Pi, Gemini CLI, Kilo, OpenCode, and other tools from one control plane.

Important legal finding: GitHub currently reports `fork: false` but **no detected repository license**, and a root `LICENSE` file was not found in this audit. Treat as **STUDY ONLY** until explicit reuse rights are established. Do not copy implementation source into Winds merely because the repository is public.

Winds divergence: do not adopt quiet-output heuristics or terminal text as authoritative command/process truth. Winds already has stricter PTY ownership and telemetry requirements.

### 3.3 `nicobailon/pi-memory-workbench`

Classification: **TIER S — Verified Experience Memory reference**.

Key ideas:

- plain-Markdown durable memory without a server;
- owner-session-specific `progress.md` and `todo.md` files;
- shared project dashboard explicitly marked non-authoritative;
- immutable append-only event files;
- subagents/unidentified sessions are read-only for durable state;
- source-cited long-term `memory.md` curation;
- additive event digests that never replace raw authoritative events;
- deterministic fallback behavior that avoids inventing verification outcomes;
- explicit supersession rather than destructive rewriting;
- multi-session conflict avoidance;
- read-only recall/lint surfaces.

This is an excellent reference for a future Winds narrative-memory layer, but Winds should keep its SQLite/execution/evidence ledger authoritative and treat Markdown memory as derived, cited, and revocable.

Provenance snapshot: GitHub reports `fork: false` and MIT license metadata.

### 3.4 `nicobailon/pi-messenger`

Classification: **TIER S — fleet coordination/reference, legal hold for copying**.

Observed capabilities include multi-agent shared communication, presence/activity, messaging, file reservations, stuck detection, dependency-oriented Crew workflows, reviewer/fix iterations, role/team charters, durable team memory, approval gates, concurrency/model configuration, and human participation.

High-value Winds ideas:

- file/work ownership reservations;
- dependency-wave scheduling;
- explicit participant roles;
- human approval gates;
- durable team state;
- stuck detection and handoff;
- reviewer/fix loops.

Important legal finding: GitHub reports `fork: false` but **no detected repository license**, and a root `LICENSE` file was not found during this audit. Treat source as **STUDY ONLY** unless licensing is clarified.

Winds divergence: statuses such as `SHIP`, `NEEDS_WORK`, or task-complete remain reports, not authoritative verification. Winds should map them through its evidence plane.

### 3.5 `nicobailon/pi-prompt-template-model`

Classification: **TIER S/A — declarative fleet policy/workflow DSL reference**.

Observed frontmatter/control concepts include:

- model/provider selection and fallback;
- skill injection;
- thinking level;
- restore previous model;
- chains and loops;
- model rotation;
- fresh-context iteration;
- convergence;
- boomerang context collapse;
- worktree isolation;
- subagent delegation;
- inherited context;
- parallel fan-out;
- best-of-N worker/reviewer/final-applier flows;
- delegated cwd.

This is close to a future declarative **Winds Fleet Plan** concept. Do not copy the Pi-specific prompt-template syntax prematurely; extract the control semantics into a dedicated future spec.

Provenance snapshot: GitHub reports `fork: false` and MIT license metadata.

---

## 4. Tier A — high-value future components

### `nicobailon/pi-intercom`

Future agent mailbox/escalation reference: local session-to-session messaging, ask/reply/pending state, cancellation/supersession, message identity/timestamps, supervisor escalation reasons, and bounded disconnected-session behavior. Future Fleet spec only; current Spec 003 forbids generic IPC/public runtime expansion.

### `nicobailon/pi-mcp-adapter`

Future interoperability reference. It exposes a token-efficient MCP surface with lazy tool discovery/description, lazy server startup, configuration provenance/conflict reporting, include/exclude controls, OAuth/security handling, and machine-readable status behavior.

Provenance snapshot: GitHub reports `fork: false` and MIT license metadata.

### `nicobailon/pi-web-access`

Future research/evidence-tool reference: provider routing/fallback, web/content/PDF/video access, bounded retrieval/cache, GitHub source handling, and machine-readable source checking with passage provenance. Useful for future research agents, not core execution authority.

### `nicobailon/pi-coordination`

Architecture reference for planning interviews, targeted scouting, validated task graphs, dependency dispatch, file reservations, discovered tasks, agent-to-agent messaging, supervisor restart/nudge, cost controls, async operation, causality/spans, and dashboarding. Likely overlaps with newer `pi-subagents` + `pi-messenger`; perform recency/source-overlap audit before choosing donor code.

### `nicobailon/pi-boomerang`

Context-lifecycle reference: execute a task, then collapse raw turn history into a handoff while preserving file changes. Useful for context isolation and fleet handoff. Its summaries are heuristic narrative state, not Winds evidence.

### `nicobailon/pi-custom-compaction`

Context-policy reference: configurable summary model fallback chain, token thresholds, retention budgets, model-specific profiles, templates, and fallback to native compaction when invalid. Useful for long-running multi-agent sessions.

### `nicobailon/pi-autoresearch`

Experiment/self-improvement loop reference: edit -> benchmark -> log -> keep/revert with append-only experiment records and optional correctness backpressure checks.

Provenance caution: the README installation points at `davebcn87/pi-autoresearch`, so treat Nico's repository as possible mirror/fork/derivative until exact lineage is established. Do not attribute original code solely to Nico without a commit-level provenance check.

### `nicobailon/pi-rewind-hook`

Conceptually valuable for exact session-to-file-state binding, snapshot deduplication, lineage, and retention. High risk for direct Winds reuse because it performs Git-backed rewind/restoration. Winds' non-destructive primary-checkout safety rules take precedence. Study the metadata/snapshot model, not its restore behavior, until a dedicated recovery spec exists.

### `nicobailon/pi-interview-tool`

Strong human-in-the-loop UX reference: structured single/multi/text/image/info questions, rich media, recommendation conviction/weight, autosave/recovery, and multi-agent queueing that avoids focus stealing. Valuable for future approval/clarification surfaces.

### `nicobailon/mcp-to-pi-tools`

Generates Pi-native TypeScript tools from arbitrary MCP servers, including AI-based tool grouping, TypeBox schemas, action discriminators, npm/python/custom-command transports, and optional HTTP endpoints.

The README claims MIT, but GitHub currently reports no detected license and a root `LICENSE` file was not found in this audit. Treat source as **STUDY ONLY** until licensing is clarified. Conceptually, it is useful for future tool-schema adaptation and capability compression; `pi-mcp-adapter` is the stronger current interoperability reference.

---

## 5. Tier B — useful UX / specialized references

### `nicobailon/pi-review-loop`

Iterative review-loop UX and fresh-context review patterns. Useful control pattern, but self-review must never substitute for independent Winds verification.

### `nicobailon/pi-model-switch`

Simple model catalog/search/switch tool with aliases and fallback chains. Useful reference for a small model-selection seam. Its README says foreground orchestration moved to `pi-orchestrate`; no repository named `pi-orchestrate` was found under Nico's GitHub during this audit, so treat that reference as unresolved rather than inventing a location.

### `nicobailon/pi-skill-palette`

Explicit skill queue/pin UX. Useful for human-controlled capability selection and trust-aware skill discovery. Runtime remains Pi-specific despite Agent Plugins package metadata.

### `nicobailon/pi-side-chat`

Conversation fork/side-agent UX. Read-only by default, optional edit mode, file-overlap warnings, and `peek_main`. Useful future analyst/reviewer side-channel pattern. Heuristic overlap detection is not sufficient for Winds write authority.

### `nicobailon/pi-design-deck`

Visual architecture/code/UI decision-deck tool with selectable options, persistent browser state, SSE option generation, snapshots, and export. Future product/approval UX reference.

### `nicobailon/pi-annotate`

Browser visual feedback pipeline: element picker, box-model/a11y/styles, screenshots, edit capture, native host, remote/WSL bridging. Useful future UI-feedback integration; not current execution-spine scope.

### `nicobailon/pi-powerline-footer`

Rich terminal status/TUI reference: model/context/token/git status, extension statuses, queue, editor stash, managed bash mode, persistent transcript/history, and custom status items. Good future fleet status-bar reference.

Winds divergence: do not import shell-history/ghost-suggestion semantics into authoritative command telemetry.

### `nicobailon/pi-tool-display`

Compact tool rendering, adaptive diffs, MCP-aware presentation, thinking labels, and tool-ownership switches. README installation references `MasuRii/pi-tool-display`, so verify fork/original lineage before any copy decision. Mostly UI/reference value.

### `nicobailon/pi-discord`

Remote ingress/headless session experiment with durable route queues, journals, sessions, queue leases, restart recovery, allowlists, and a detached daemon. Architecturally interesting for a future remote-control spec; explicitly **out of current Spec 003** because it introduces a daemon and external ingress.

### `nicobailon/pi-foreground-chains`

A skill rather than a runtime extension. Implements simple visible scout -> planner -> worker -> reviewer chains using file handoffs and `pi-interactive-shell`. Useful as a minimal workflow reference; modern `pi-subagents` is a much richer target.

---

## 6. Historical / superseded / adjacent

### `nicobailon/pi-subagent-enhanced`

Explicitly described as a fork of Pi's async-subagent example with truncation, artifacts, session-scoped notifications, and single/chain/parallel modes. Modern `pi-subagents` supersedes it for our purposes. Keep only as historical implementation archaeology.

### `nicobailon/pi-mono`

Not a Nico-original extension. GitHub reports it as a fork whose parent/source is `earendil-works/pi`, under MIT. Use the canonical upstream `earendil-works/pi` for Pi architecture/donor work unless a Nico-specific divergence is intentionally under study.

### `nicobailon/picpaster`

Adjacent macOS utility, not a Pi extension. Converts clipboard images/screenshots to temporary file paths for terminal/TUI paste. Small future UX idea for multimodal terminal workflows; not a Winds runtime priority.

---

## 7. `nicobailon/pi-extensions` monorepo inventory

The personal `pi-extensions` repository contains eight additional extension families:

### `tab-status`

Terminal-tab indicators for many concurrent Pi sessions: done/committed, done-without-commit, blocked/timeout, running. This is directly relevant to the long-term 20+ agent Fleet UX, although Winds should derive state from its own ledger rather than title heuristics.

### `ralph-wiggum`

Long-running flat iterative loops for verifiable tasks, multiple parallel loops in one repo, optional self-reflection, max-iteration and checklist controls. Useful future long-horizon workflow reference. Completion promises remain agent reports until Winds verifies outcomes.

### `agent-guidance`

Provider/model-specific instruction loading (`CLAUDE.md`, `CODEX.md`, `GEMINI.md`) in addition to Pi's normal `AGENTS.md`. Useful future adapter-specific guidance concept, but Winds should avoid multiplying hidden instruction sources without explicit provenance and precedence.

### `usage-extension`

Aggregated session/model/provider cost, message, token, and cache statistics. Useful future fleet observability/cost accounting UX. Provider-reported cost remains provider-reported, not independently observed truth.

### `raw-paste`

Editable raw paste UX instead of opaque collapsed paste markers. Low architectural value; useful terminal ergonomics reference.

### `code-actions`

Select assistant code blocks/snippets to copy, insert, or run. Useful convenience UX; not a control-plane primitive.

### `relaunch`

WIP session exit/relaunch/resume behavior. Study only if still active; current README marks it under development.

### `arcade`

Terminal minigames while waiting. No core Winds value.

The monorepo has a repository-level LICENSE file, but exact license attribution for any copied subpath should still be recorded at the donor commit/path level.

---

## 8. Winds synthesis — what to build from the ideas, not from blind copying

The combined Pi ecosystem suggests the following future Winds capability families:

```text
Winds Fleet Control
├── Fleet Runtime
│   ├── external CLI adapters
│   ├── subagent lifecycle
│   ├── bounded concurrency
│   ├── worktree isolation
│   └── attach / steer / stop
├── Coordination
│   ├── task dependency graph
│   ├── roles
│   ├── reservations / ownership
│   ├── mailbox / escalation
│   └── human approval gates
├── Context + Memory
│   ├── per-session working memory
│   ├── append-only events
│   ├── cited derived memory
│   ├── compaction policy
│   └── handoff / resume
├── Policy
│   ├── model/provider routing
│   ├── skill selection
│   ├── fallback chains
│   ├── best-of-N / reviewer topology
│   └── explicit cwd / worktree policy
├── Observability
│   ├── run status
│   ├── event/transcript views
│   ├── time/token/cost
│   ├── stuck detection
│   └── multi-agent navigation
└── Evidence + Verification (Winds-owned authority)
    ├── exact Git/workspace identity
    ├── observed process/command facts
    ├── deterministic checks
    ├── independent review
    └── safe human-selected promotion
```

The bottom layer is what distinguishes Winds. Pi ecosystem primitives can make the fleet easier to run, but Winds must remain the authority for what actually happened and what is safe to promote.

---

## 9. Recommended future donor deep-audit order

When an Agent Fleet research/spec phase is authorized, audit in this order:

1. `pi-subagents`: runtime state, API, artifacts/events, concurrency, external runners, worktrees, watchdog, observability.
2. `pi-interactive-shell`: process/PTY architecture, CLI adapters, attach/takeover, dispatch UX; legal license clarification first.
3. `pi-memory-workbench`: event schema, ownership, deterministic digest/recall, curation/supersession.
4. `pi-prompt-template-model`: declarative policy semantics and best-of-N/worktree topology.
5. `pi-messenger`: reservations, role/team model, approval gates, dependency-wave coordination; legal license clarification first.
6. `pi-intercom`: mailbox/escalation semantics.
7. `pi-mcp-adapter`: future interoperability/tool-surface compression.
8. `pi-custom-compaction` + `pi-boomerang`: context lifecycle.
9. `pi-autoresearch`: verified experiment loops and backpressure.
10. human-facing UX: `pi-interview-tool`, `pi-design-deck`, `pi-side-chat`, `tab-status`, `usage-extension`.

---

## 10. Provenance and licensing rules

For Winds donor work, public visibility is not a license.

Required before direct code reuse:

- pin donor repository + commit;
- record exact source paths copied/adapted;
- verify SPDX/license file at that exact revision;
- identify whether the repo is a fork/derivative and attribute the actual upstream author when necessary;
- preserve required notices/attribution;
- audit dependency licenses separately;
- record semantic changes made by Winds;
- reject source with unresolved reuse rights.

Specific audit cautions discovered here:

- `pi-subagents`: GitHub reports MIT and not a fork — permissive donor candidate.
- `pi-memory-workbench`: GitHub reports MIT and not a fork — permissive donor candidate.
- `pi-prompt-template-model`: GitHub reports MIT and not a fork — permissive donor candidate.
- `pi-mcp-adapter`: GitHub reports MIT and not a fork — permissive donor candidate.
- `pi-interactive-shell`: no detected license/root LICENSE in this audit — study only until clarified.
- `pi-messenger`: no detected license/root LICENSE in this audit — study only until clarified.
- `mcp-to-pi-tools`: README says MIT, but GitHub did not detect a license and root LICENSE was not found — study only until clarified.
- `pi-mono`: fork of `earendil-works/pi`; use canonical upstream for base Pi donor work.
- `pi-autoresearch`: README points to another repository for installation; verify lineage before attribution.
- `pi-tool-display`: README points to another repository for installation; verify lineage before attribution.

---

## 11. Sources inspected

Primary GitHub materials inspected during this audit include the account repository inventory, repository metadata, README files, the `pi-extensions` monorepo, internal extension READMEs, and the `pi-subagents/docs` index.

Key repository URLs:

- `https://github.com/nicobailon/pi-subagents`
- `https://github.com/nicobailon/pi-interactive-shell`
- `https://github.com/nicobailon/pi-memory-workbench`
- `https://github.com/nicobailon/pi-messenger`
- `https://github.com/nicobailon/pi-prompt-template-model`
- `https://github.com/nicobailon/pi-intercom`
- `https://github.com/nicobailon/pi-mcp-adapter`
- `https://github.com/nicobailon/pi-web-access`
- `https://github.com/nicobailon/pi-coordination`
- `https://github.com/nicobailon/pi-boomerang`
- `https://github.com/nicobailon/pi-custom-compaction`
- `https://github.com/nicobailon/pi-autoresearch`
- `https://github.com/nicobailon/pi-rewind-hook`
- `https://github.com/nicobailon/pi-interview-tool`
- `https://github.com/nicobailon/pi-design-deck`
- `https://github.com/nicobailon/pi-annotate`
- `https://github.com/nicobailon/pi-powerline-footer`
- `https://github.com/nicobailon/pi-review-loop`
- `https://github.com/nicobailon/pi-model-switch`
- `https://github.com/nicobailon/pi-skill-palette`
- `https://github.com/nicobailon/pi-side-chat`
- `https://github.com/nicobailon/pi-discord`
- `https://github.com/nicobailon/pi-foreground-chains`
- `https://github.com/nicobailon/pi-subagent-enhanced`
- `https://github.com/nicobailon/mcp-to-pi-tools`
- `https://github.com/nicobailon/pi-extensions`
- `https://github.com/nicobailon/pi-mono`
- `https://github.com/nicobailon/picpaster`

Unresolved reference observed in `pi-model-switch`: `pi-orchestrate`. No repository of that exact name was found under Nico's GitHub account during this audit.

---

## 12. Final research verdict

The Nico Bailon Pi ecosystem is one of the strongest concentrated OSS/reference sets found so far for the **future Winds Agent Fleet**, particularly for delegation, terminal control, coordination, memory, context management, model/skill policy, and human-in-the-loop UX.

The most important Winds architectural move is to combine those ideas with the thing Pi extensions generally do not provide as a single authority: **Winds' evidence-first execution and verification ledger**.

Future thesis:

> **Use Pi-style composable fleet primitives to make many agents easy to operate, but let Winds decide truth only from attributable evidence and independently verified outcomes.**
