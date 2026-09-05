# Winds Agentic-Era Terminal North Star — Source Register

**Status:** Research-only provenance register. Non-authorizing.

**Research freeze:** 2026-09-05

**Canonical Winds base:** `dfa2c524df7ce8a6d4aa481a61d2bbf0fbe87c3e`

This register records the primary sources used to inform `012-agentic-era-terminal-north-star.md`, the bounded claims drawn from them, and implementation-time cautions. External products evolve rapidly; every capability, API, protocol, license, and behavior MUST be revalidated before implementation.

---

## 1. Winds canonical sources

### W001 — Winds README

- Repository: `TheHalfMoon/Winds`
- Canonical base: `dfa2c524df7ce8a6d4aa481a61d2bbf0fbe87c3e`
- Path: `README.md`
- Purpose: current released and development-surface truth, authority taxonomy, terminal execution boundary, platform claims, and intentionally deferred features.
- Claim boundary: the README establishes that current Winds is a verification runtime with an accepted workspace-execution spine, not yet a GUI terminal, provider router, daemon, persistent detached-session service, browser runtime, or generic agent/provider platform.

### W002 — Winds Agentic Development Master Plan

- Path: `docs/research/008-agentic-development-master-plan.md`
- Purpose: existing pre-spec North Star for workspaces, connected sessions, memory classes, universal agent runtime direction, provider/runtime separation, authority truth, and exact-candidate verification.
- Claim boundary: research-only; does not authorize implementation.

### W003 — Spec 006 Agentic Terminal & Local Delegation Control Plane

- Path: `specs/006-agentic-terminal-local-delegation-control-plane/spec.md`
- Purpose: canonical invariants for session continuity, runtime/model separation, context transfer, authority, independent review, and exact-candidate evidence.
- Important inherited invariant:

```text
MODEL_CONTEXT_MAY_COMPACT; CANONICAL_WORK_EVIDENCE_TRUTH_MUST_NOT
```

- Claim boundary: exact current specification status and task authority must be reread from live repository truth before any implementation.

### W004 — Verified Learning Loop and Herdr roadmaps

- Paths:
  - `docs/research/010-verified-learning-loop-roadmap.md`
  - `docs/research/011-herdr-parity-and-beyond-roadmap.md`
- Purpose: future learning/verification direction and terminal/product parity research.
- Claim boundary: research-only and non-authorizing.

---

## 2. Warp sources

### R001 — Warp repository

- Repository: `warpdotdev/warp`
- Observed branch: `master`
- Observed commit on 2026-09-05: `a48ff8014824d3d04568f15b7da666ea6562c36b`
- Repository URL: `https://github.com/warpdotdev/warp`
- README: `https://github.com/warpdotdev/warp/blob/master/README.md`
- Bounded claims:
  - Warp describes itself as an agentic development environment born out of the terminal.
  - It supports its built-in coding agent and third-party CLI agents including Claude Code, Codex, Gemini CLI, and others.
  - The open repository contains a Rust-based UI/application architecture and agent-oriented components.
- Implementation caution:
  - Warp's repository is predominantly AGPL-3.0.
  - The repository README states that `warpui_core` and `warpui` are MIT while the rest is AGPL-3.0.
  - Winds MUST NOT copy/adapt AGPL implementation code into its MIT/Apache-2.0 codebase without an explicit compatible licensing decision.
  - Product behavior may be studied independently; implementation must be clean-room unless a separately compatible source is explicitly selected.

### R002 — Warp current Getting Started documentation

- URL: `https://docs.warp.dev/`
- Observed update: 2026-09-03 during this research pass.
- Bounded claims:
  - Warp combines Terminal and Agent modes.
  - It exposes modern terminal UX including block-based navigation, multi-line editing, syntax highlighting, and completions.
  - It includes code/file-tree/LSP/review experiences.
  - It enhances third-party CLI agents with richer terminal tooling.
- Use in Winds plan: interaction-quality benchmark only, not an implementation recipe.

### R003 — Warp Agents product page

- URL: `https://www.warp.dev/agents`
- Bounded claims observed on 2026-09-05:
  - vertical tabs for agent sessions;
  - attention/notification workflows;
  - interactive code review;
  - multiple coding-agent harnesses in one terminal experience;
  - orchestration/control-plane and shared-context product direction.
- Use in Winds plan: motivates first-class agent-session management and Agent Inbox concepts.

### R004 — Warp Drive product page

- URL: `https://www.warp.dev/drive`
- Bounded claims:
  - reusable rules, prompts, notebooks, workflows, and MCP-related shared context;
  - team/agent context is a first-class product surface.
- Use in Winds plan: context must be a product object, but Winds adds stricter provenance, canonical task identity, and evidence authority.

### R005 — Warp Terminal product page

- URL: `https://www.warp.dev/terminal`
- Bounded claim: Warp positions the terminal as a workbench for built-in and third-party agents.
- Use in Winds plan: daily-terminal replacement is a product benchmark.

### R006 — Warp block-model engineering article

- URL: `https://www.warp.dev/blog/block-model-behind-warps-agentic-development-environment`
- Published: 2026-04-29
- Bounded claims:
  - Warp describes a typed block model, GPU-backed renderer, and custom Rust UI foundation as architectural roots that support terminal and agent interactions in one scroll surface.
- Use in Winds plan:
  - independently adopt the **typed interaction object principle**;
  - do not copy AGPL implementation code.

### R007 — Warp Universal Agent Support article

- URL: `https://www.warp.dev/blog/universal-agent-support-level-up-coding-agent-warp`
- Published: 2026-04-14
- Bounded claims:
  - Warp adds management UX around third-party CLI agents including Codex and Claude Code.
- Use in Winds plan: reinforces that universal harness support is becoming table stakes, so Winds must differentiate through durable canonical continuity, authority, browser reality, and verification.

### R008 — Warp 2.0 / ADE article

- URL: `https://www.warp.dev/blog/reimagining-coding-agentic-development-environment`
- Published: 2025-06-24
- Bounded claims:
  - universal input combines command and agent workflows;
  - multi-agent management is a core ADE product primitive;
  - Drive provides shared knowledge/rules/context.
- Use in Winds plan: universal input and agent-management UX benchmark.

---

## 3. Pi sources

### P001 — Pi repository snapshot

- Repository: `earendil-works/pi`
- Observed branch: `main`
- Observed commit on 2026-09-05: `da840b6216578c2a571d0374ac6a2091a83f9d91`
- Repository URL: `https://github.com/earendil-works/pi`
- Purpose: current reference point for provider/session/compaction research.
- Implementation caution: revalidate licensing and exact source lineage before any reuse. Product concepts alone do not authorize code copying.

### P002 — Pi Providers documentation

- Path: `packages/coding-agent/docs/providers.md`
- Observed content source: current Pi repository during the 2026-09-05 research pass.
- Bounded claims:
  - Pi supports multiple subscription/OAuth and API-key providers;
  - provider credentials can come from auth storage or environment configuration;
  - it supports custom/local providers and multiple API compatibility families;
  - provider/model configuration is reloadable and provider-specific.
- Use in Winds plan:
  - a canonical Winds session should be provider-independent;
  - many APIs/models should be selectable inside one task/session;
  - Winds must add stronger provenance, cost, authority, and continuity semantics.

### P003 — Pi Sessions documentation

- Path: `packages/coding-agent/docs/sessions.md`
- Observed blob SHA: `1a50ee02d7cf53652fb3569bd66354779fea9dfd`
- Bounded claims:
  - Pi persists sessions as JSONL;
  - sessions support resume, naming, tree navigation, fork, clone, and export;
  - conversation history is a tree rather than only a linear transcript.
- Use in Winds plan:
  - session-tree UX is a useful reference;
  - Winds must link session lineage to canonical task/evidence/reality identity.

### P004 — Pi Compaction & Branch Summarization documentation

- Path: `packages/coding-agent/docs/compaction.md`
- Bounded claims:
  - Pi uses context-window-aware compaction;
  - recent messages are retained while older context is summarized;
  - branch changes can produce summaries;
  - file operations are carried through compaction metadata.
- Use in Winds plan:
  - long-lived model interaction requires compaction;
  - Winds must protect canonical objective/constraints/decisions/evidence independently from generated summaries.

### P005 — Pi Custom Models documentation

- Path: `packages/coding-agent/docs/custom-models.md`
- Bounded claims:
  - custom OpenAI-compatible, Anthropic-compatible, Google, local, and proxy-backed model definitions are supported;
  - per-provider/per-model compatibility and context/cost metadata can be represented.
- Use in Winds plan: informs Provider Mesh capability modeling, not an API/schema to copy blindly.

---

## 4. OpenAI Codex / browser / long-running sources

### O001 — Introducing the Codex app

- URL: `https://openai.com/index/introducing-the-codex-app/`
- Published: 2026-02-02
- Bounded claims:
  - Codex app is designed as a command center for multiple agents;
  - it supports parallel agent work and long-running tasks.
- Use in Winds plan: multi-agent command-center behavior is not unique by itself.

### O002 — Codex for (almost) everything

- URL: `https://openai.com/index/codex-for-almost-everything/`
- Published: 2026-04-16
- Bounded claims observed in the publication:
  - Codex app includes an in-app browser;
  - users can annotate pages to give precise instructions;
  - Codex can perform computer-use tasks and work across more tools/apps;
  - ongoing/repeatable and longer-running work is an explicit product direction.
- Use in Winds plan:
  - simple browser control is becoming a baseline feature;
  - Winds should differentiate with Browser Twin + exact Reality/evidence binding.

### O003 — Work with Codex from anywhere

- URL: `https://openai.com/index/work-with-codex-from-anywhere/`
- Published: 2026-05-14
- Bounded claims:
  - active Codex work can continue on laptops/devboxes/remote environments;
  - mobile can inspect live state, approvals, screenshots, terminal output, diffs, and tests;
  - a relay layer keeps active environments reachable without directly exposing machines to the public internet.
- Use in Winds plan:
  - future remote/mobile continuity is valuable;
  - local canonical truth and explicit security boundaries should be solved before remote expansion.

### O004 — Built-in browser help

- URL: `https://help.openai.com/en/articles/20001277-using-the-built-in-browser-in-the-chatgpt-desktop-app`
- Observed current help content during 2026-09-05 research.
- Bounded claims:
  - built-in browser on desktop can use multiple tabs;
  - it has its own browser state;
  - it supports page annotation and controlled website access;
  - a separate Chrome path can use an existing signed-in profile.
- Use in Winds plan:
  - browser profile/account context and explicit website access are required design concerns.

### O005 — Running Codex safely at OpenAI

- URL: `https://openai.com/index/running-codex-safely/`
- Published: 2026-05-08
- Bounded claims:
  - agent deployment requires explicit access controls, approvals, network policy, managed configuration, and telemetry.
- Use in Winds plan: browser/provider/durable agents need policy and telemetry as core architecture, not later polish.

### O006 — Codex Windows sandbox engineering article

- URL: `https://openai.com/index/building-codex-windows-sandbox/`
- Published: 2026-05-13
- Bounded claims:
  - effective agent sandboxing requires OS-enforced process/filesystem/network boundaries;
  - platform-specific enforcement matters.
- Use in Winds plan:
  - Winds must never equate a worktree, browser context, terminal session, or policy label with a security sandbox unless enforcement is proven.

---

## 5. Research conclusions supported by the source set

The following conclusions are product synthesis, not direct quotations from any single source.

### C001 — Beautiful terminal UX is necessary but insufficient

Warp demonstrates a high bar for terminal interaction quality and agent management. Winds must meet that usability bar while preserving its stronger verification and authority semantics.

### C002 — Universal agent/provider support is becoming table stakes

Warp supports multiple CLI agents; Pi supports many providers/models; Codex supports multi-agent work. Therefore “supports several agents/models” alone is not a sufficient enduring differentiator.

### C003 — Browser is becoming table stakes

Codex has an in-app browser and page annotation. Browser automation ecosystems are broad. Winds should not market “has a browser” as the unique invention.

The proposed differentiation is:

```text
BROWSER + EXACT CANDIDATE + SESSION + SERVICE + MEMORY + AUTHORITY + EVIDENCE
= VERIFIED REALITY
```

### C004 — Long-running continuity belongs outside model context

Pi's compaction behavior and Codex's long-running product direction both reinforce that task continuity cannot depend on an indefinitely growing active prompt. Winds should preserve canonical structured work/evidence state outside the model and reconstruct provider-compatible context views when needed.

### C005 — The strongest Winds moat is verified continuity across heterogeneous execution

The proposed defensible combination is:

```text
beautiful terminal UX
+ provider-independent canonical sessions
+ durable checkpoints
+ bounded agent delegation
+ isolated browser realities
+ exact Git candidate identity
+ deterministic evidence
+ independent review
+ explicit human landing
```

No source is cited as proving that no competing product can ever implement this combination. “Unique” remains a product hypothesis that must be continuously revalidated against the market.

---

## 6. Implementation-time revalidation checklist

Before a future formal spec or implementation uses any external-product assumption:

1. re-fetch the current upstream source/product documentation;
2. record exact commit/blob/version where source code matters;
3. re-check license and notice requirements;
4. separate product behavior from implementation details;
5. verify protocol/API stability and versioning;
6. re-check authentication/credential terms;
7. re-check local/cloud/privacy behavior;
8. re-check platform support;
9. run an updated competitor gap review;
10. preserve Winds' canonical authority/evidence model unless explicitly amended through governance.
