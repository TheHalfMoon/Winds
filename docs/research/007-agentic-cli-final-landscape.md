# Final Agentic CLI / Terminal Landscape

**Status:** Research dossier only. No product implementation is authorized by this document.

**Research freeze:** 2026-08-20

**Canonical Winds base inspected:** `048cf59e8bdbe3a757b4d3ead214099ce18369bd`

**Authority boundary:** This document does **not** amend the Constitution or Spec 003, does **not** close T068, does **not** start T069, and does **not** authorize a daemon/session owner, public/local IPC, ACP/MCP runtime implementation, generic agent adapters, Agent Fleet, remote execution, plugin runtime, or any other future product code. Future implementation remains gated by Constitution -> Spec -> Plan -> Tasks and exact-candidate verification.

**Research basis:** `006-agent-fleet-donor-audit.md`, current primary product/protocol sources, and current agent/security/evaluation research.

### Source reproducibility policy

All claim-bearing external sources in this dossier were retrieved/re-verified on **2026-08-20**. URLs in the body remain navigation links and may move after the freeze.

Where a GitHub-hosted source was materially used and an exact revision was captured, the immutable pin is recorded below and/or beside the claim:

- `earendil-works/pi` — `5cd93f688aaab89dbb6dfa4aca535f21796ae185`;
- `aaif-goose/goose` — `fc6311acb734923651713cf0e6a4539f7e3b3625`;
- `openai/codex` — `9bf673718a4605b49e47d00762121d372af95439`;
- `google-gemini/gemini-cli` — `e90c63fa158b8facd1872d32b34b07e516308f2b`;
- `cline/cline` — `16875140fbc7bae51aad79c203837b4f51e54aa5`;
- `mistralai/mistral-vibe` — `5e6aa0f6beb3454454f4c1de74a7652ba577ab05`;
- `manaflow-ai/cmux` — `10549b7c1289999f298f48af0a55754932569d0a` for the agent-hook/hibernation material inspected during this research cycle;
- `smtg-ai/claude-squad` — `ce1ffb4392b01f38e2c4599c7c84d2a93973b138`;
- `ctxrs/ctx` — `94c0d32e1f4c3f7f7c78febdb916066d8df67c6c`.

Vendor documentation sites and product pages that do not expose a public immutable revision are treated as **`MUTABLE_VENDOR_DOC`** sources retrieved on 2026-08-20, not as byte-for-byte frozen evidence. Dated protocol/release posts and research publication identifiers provide a stronger publication identity but must still be re-verified and version-pinned before formal specification or implementation. Absence of an immutable public archive is a source limitation, not permission to treat the current live page as historical truth.

---

## 1. Executive conclusion

The 2026 market has moved beyond “one AI agent in one terminal.” The following are now table stakes or rapidly becoming table stakes:

- named/resumable sessions;
- multiple concurrent agents;
- Git-worktree isolation;
- model/runtime selection;
- read-only/plan versus implementation modes;
- approvals and trust controls;
- background/headless execution;
- persistent or resumable terminal/session UX;
- notifications/attention routing;
- fuzzy file/folder selection and repository-aware context;
- skills, hooks, MCP, custom agents, and structured machine interfaces;
- remote monitoring/continuation;
- session-history retrieval and context compaction.

Winds therefore must **not** define its moat as “many agents,” “worktrees,” “persistent terminals,” “session resume,” “an inbox,” “history search,” or “support for every agent.” Competitors already ship substantial parts of that bundle.

The strongest defensible direction is the combination of:

> **runtime-neutral canonical task/work/evidence continuity**
> + **explicit transfer/loss provenance**
> + **externally enforced local authority**
> + **exact workspace/candidate identity**
> + **exact-candidate deterministic verification**
> + **independence-preserving review**
> + **explicit human landing authority**.

Winds should target this position:

> **Winds is the verified local runtime for agentic software development.**
>
> **Any agent. One runtime. Verified work.**

The stronger user promise remains:

> **Run any agent. Build any team. Keep it alive. Gate its authority. Verify its work.**

---

## 2. Competitive landscape

### 2.1 JetBrains Junie CLI

**Observed strengths**

- multiple live sessions in one CLI instance;
- searchable history and resume;
- human-readable titles;
- direct file/folder selection and IDE-assisted symbol intelligence;
- worktree workflows;
- follow-up prompts while work is active;
- imports of guidelines/skills/commands/MCP configuration;
- remote continuation.

**Winds lesson:** workspace/session naming, search, resume, `@`-style context selection, and optional IDE intelligence are baseline UX, not differentiation.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://junie.jetbrains.com/docs/junie-cli.html
- https://junie.jetbrains.com/docs/slash-commands.html
- https://junie.jetbrains.com/docs/junie-cli-worktrees.html
- https://junie.jetbrains.com/docs/junie-cli-jetbrains-ide-integration.html

### 2.2 Factory Droid

**Observed strengths**

- separates interaction mode from autonomy level;
- orchestrator/worker/validator structures;
- model selection per worker/validator;
- resumable background subagents;
- worktree execution;
- structured JSON/JSON-RPC modes;
- session search/resume/fork;
- organization-level autonomy ceilings.

**Winds lesson:** role, workflow, runtime/model, direct execution authority, and delegation authority must be separate concepts. Child execution authority must never silently exceed the human-approved delegation/team ceiling.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://docs.factory.ai/droid-cli/quickstart
- https://docs.factory.ai/droid-cli/cli-reference
- https://docs.factory.ai/autonomy-and-safety/auto-run
- https://docs.factory.ai/harness/subagents
- https://docs.factory.ai/droid-exec/overview

### 2.3 Pi

**Observed strengths**

- durable local session records;
- distinct continue/resume/tree/fork/clone/compact semantics;
- searchable session picker;
- branching session history;
- local full history while model context can compact;
- flexible models/extensions and community delegation patterns.

**Winds lesson:** `NEW_SESSION != NEW_TASK`. Canonical workstream identity must survive context compaction and runtime changes.

Primary source:

- immutable: https://github.com/earendil-works/pi/tree/5cd93f688aaab89dbb6dfa4aca535f21796ae185/packages/coding-agent/docs
- navigation: https://github.com/earendil-works/pi/tree/main/packages/coding-agent/docs

The earlier `badlogic/pi-mono` location now redirects to this repository; the canonical repository identity was re-verified on 2026-08-20.

### 2.4 Goose

**Observed strengths**

- named sessions, resume/fork/history;
- custom agents and recipes;
- delegated agents in separate sessions;
- model flexibility;
- MCP/extensions;
- explicit separation between reusable role and reusable workflow.

**Winds lesson:** keep `Role`, `Workflow`, `Runtime`, and `Model` separate. Delegation needs explicit context inheritance rather than raw parent-history dumping.

Primary sources:

- immutable repository docs: https://github.com/aaif-goose/goose/tree/fc6311acb734923651713cf0e6a4539f7e3b3625/documentation
- navigation: https://github.com/aaif-goose/goose/tree/main/documentation
- live docs (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20): https://goose.docs.block.xyz/

The earlier `block/goose` location now redirects to `aaif-goose/goose`; the canonical repository identity was re-verified on 2026-08-20.

### 2.5 Herdr

**Observed strengths**

- agent-aware terminal panes;
- broad coding-agent detection;
- persistent terminal/session ownership;
- detach/reattach workflows;
- automation primitives;
- remote attachment/control;
- distinction between stronger lifecycle hooks and weaker screen heuristics.

**Winds lesson:** persistent execution is strategically important but is not enough. Winds must add canonical task identity, authority provenance, exact candidate identity, evidence, and human decisions.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://herdr.dev/docs/agents/
- https://herdr.dev/docs/agent-automation/
- https://herdr.dev/docs/session-state/
- https://herdr.dev/docs/persistence-remote/

### 2.6 Warp

**Observed strengths**

- agents operate in a real interactive terminal;
- human Take Over / Hand Back in the same session;
- command approval and bounded auto-approval patterns.

**Winds lesson:** human and agent should eventually share one execution substrate with explicit control ownership and observed handoff events.

Primary source (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://docs.warp.dev/agent-platform/capabilities/full-terminal-use

### 2.7 Zed

**Observed strengths**

- native agent, ACP external agents, and terminal threads;
- native CLIs retain their own authentication/configuration;
- project-grouped parallel threads;
- worktree isolation;
- thread history and external-agent integration.

**Winds lesson:** a terminal-first heterogeneous runtime is valid. Winds should not force third-party agents through a proprietary model/auth gateway.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://zed.dev/docs/ai/agents
- https://zed.dev/docs/ai/terminal-threads
- https://zed.dev/docs/ai/external-agents

### 2.8 VS Code / GitHub Copilot CLI

**Observed strengths**

- shared agent-session abstractions across local/background/CLI/cloud surfaces;
- handoff between local/editor and CLI contexts;
- background sessions that can outlive a window;
- workspace-oriented session management;
- fleet/subagent patterns;
- remote control;
- searchable history;
- LSP/code intelligence;
- shell integration with command/cwd/exit telemetry.

**Winds lesson:** connected sessions and cross-surface handoff are becoming baseline expectations. Winds must make continuity runtime-neutral and evidence-aware.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://code.visualstudio.com/docs/agents/concepts/sessions
- https://code.visualstudio.com/docs/agents/agents-window
- https://code.visualstudio.com/docs/terminal/shell-integration
- https://docs.github.com/en/copilot/concepts/agents/copilot-cli

### 2.9 Claude Code

**Observed strengths**

- subagents, teams, worktrees, and background work;
- isolated context per subagent;
- model/tool/permission controls;
- optional persistent memory;
- lead/teammate coordination.

**Winds lesson:** lead/team UX is validated, but Winds must generalize it across heterogeneous runtimes while keeping authority and verification external to agent prose.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://code.claude.com/docs/en/sub-agents
- https://code.claude.com/docs/en/agent-teams
- https://code.claude.com/docs/en/worktrees

### 2.10 OpenAI Codex

**Observed strengths**

- structured app-server surface;
- threads/turns/items lifecycle;
- structured approvals with thread/turn identity;
- explicit approval-policy/sandbox-policy concepts.

**Winds lesson:** prefer structured native interfaces over terminal scraping whenever available.

Primary source:

- immutable: https://github.com/openai/codex/tree/9bf673718a4605b49e47d00762121d372af95439/codex-rs/app-server
- navigation: https://github.com/openai/codex/tree/main/codex-rs/app-server

### 2.11 Gemini CLI

**Observed strengths**

- folder trust;
- restricted behavior before trust;
- sandboxing and narrow sandbox expansion;
- confirmation for mutating tools;
- `@` file/directory context;
- checkpointing.

**Winds lesson:** discovery is not trust. Project files, hooks, MCP, skills, and configuration are inputs that need independent authority treatment.

Primary source:

- immutable: https://github.com/google-gemini/gemini-cli/tree/e90c63fa158b8facd1872d32b34b07e516308f2b/docs
- navigation: https://github.com/google-gemini/gemini-cli/tree/main/docs

### 2.12 OpenCode

**Observed strengths**

- granular allow/ask/deny rules;
- per-agent overrides;
- read-only/plan-style agents;
- resource-aware rules for shell, external directories, MCP, skills, subagents;
- explicit deny precedence.

**Winds lesson:** a single `sandboxed=true` flag is inadequate. Safety-relevant capabilities need explicit resource/action semantics and truthful enforcement provenance.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://opencode.ai/docs/agents
- https://opencode.ai/docs/permissions

### 2.13 Cursor Agent CLI

**Observed strengths**

- explicit workspace root and worktree support;
- resume/continue/history;
- human command approval;
- structured non-interactive output.

**Winds lesson:** workspace identity and continuation should be uniform across runtimes rather than relearned per agent.

Primary source (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://cursor.com/docs/cli/using

### 2.14 Kiro CLI

**Observed strengths**

- spec workflow;
- capability-based permissions;
- deny/ask/allow precedence;
- protected permission configuration;
- lifecycle/tool hooks;
- inspectable persistent/session/knowledge context.

**Winds lesson:** protect the policy plane from the agents it governs, and do not confuse hook output with independent Winds evidence.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://kiro.dev/docs/cli/v3/specs/
- https://kiro.dev/docs/cli/chat/permissions/
- https://kiro.dev/docs/cli/hooks/

### 2.15 Qoder, Kimi, Cline, Mistral Vibe, Aider, Kilo

These reinforce the same convergence:

- session persistence/search/resume;
- multi-root/context controls;
- isolated subagents;
- structured/headless modes;
- worktrees;
- approvals/trust;
- repository maps;
- local daemons/relays in some products.

**Winds lesson:** new agents should enter through capability-discovered adapters rather than architecture-wide agent-name conditionals.

Primary sources:

- Qoder (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20): https://docs.qoder.com/cli/cli-reference
- Kimi (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20): https://moonshotai.github.io/kimi-code/
- Cline immutable: https://github.com/cline/cline/tree/16875140fbc7bae51aad79c203837b4f51e54aa5
- Cline navigation: https://github.com/cline/cline
- Mistral Vibe immutable: https://github.com/mistralai/mistral-vibe/tree/5e6aa0f6beb3454454f4c1de74a7652ba577ab05
- Mistral Vibe navigation: https://github.com/mistralai/mistral-vibe
- Aider (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20): https://aider.chat/docs/repomap.html
- Kilo (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20): https://kilo.ai/docs/code-with-ai/platforms/cli

---

## 3. Direct orchestration/environment competitors that materially change the Winds strategy

### 3.1 Superset

Superset is a first-class direct competitor to the future Winds environment.

**Observed strengths**

- runs many heterogeneous coding agents in parallel;
- supports Claude Code, Codex, Gemini, OpenCode, Pi, Copilot, Mistral Vibe, Kimi, Cursor Agent, Droid, and others;
- each task gets an isolated Git worktree/branch workspace;
- built-in terminal, diff, commit, PR, port, and dev-server workflow;
- scheduled automations;
- local-first behavior with remote hosts/workspaces;
- CLI and TypeScript SDK;
- MCP server able to create/manage tasks, workspaces, agents, terminals, and automations;
- remote host server/relay model.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://superset.sh/
- https://docs.superset.sh/overview
- https://docs.superset.sh/agent-integration
- https://docs.superset.sh/remote-workspaces
- https://docs.superset.sh/mcp-server
- https://docs.superset.sh/cli/getting-started

**Strategic consequence:** parallel agents, worktrees, remote workspaces, automation, agent dashboards, and broad agent support cannot be claimed as a unique Winds moat. Winds must beat this category on continuity semantics, local authority truth, exact-candidate evidence, and independent review.

### 3.2 cmux

cmux is a strong adjacent/direct competitor for terminal organization, agent state, attention, and resumability.

**Observed strengths**

- terminal/workspace organization designed around coding agents;
- agent hooks and state/notification integration across multiple runtimes;
- workspace/tab auto-naming where safe agent conversation sources exist;
- saved session restoration for supported agent sessions;
- Agent Hibernation: idle, restorable background agents can be terminated and later resumed using the native saved session;
- process-generation and workspace/surface revalidation before hibernation actions;
- explicit limitations where arbitrary live process state cannot be restored.

Primary sources:

- immutable repository: https://github.com/manaflow-ai/cmux/tree/10549b7c1289999f298f48af0a55754932569d0a
- immutable agent hooks: https://github.com/manaflow-ai/cmux/blob/10549b7c1289999f298f48af0a55754932569d0a/docs/agent-hooks.md
- navigation: https://github.com/manaflow-ai/cmux

**Strategic consequence:** persistence/resume UX, attention indicators, notifications, and agent-oriented terminal organization are not sufficient differentiation. Winds must source-label state and bind it to task/evidence/authority truth.

### 3.3 Conductor

Conductor validates the “workspace around an agent task” model.

**Observed strengths**

- independent workspaces with separate branch/worktree, files, terminals, setup, diff, checks, and PR path;
- parallel Claude Code, Codex, and Cursor sessions;
- explicit guidance on when agents should share a workspace versus use separate workspaces;
- built-in review/check/merge flow.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://www.conductor.build/docs/concepts/parallel-agents
- https://www.conductor.build/docs/guides/parallel-agents/run-multiple-claude-code-sessions
- https://www.conductor.build/docs/guides/parallel-agents/run-multiple-codex-sessions

**Strategic consequence:** “one worktree workspace per independent task” is validated product substrate, not the moat. Winds should retain exact workspace/candidate identity and stronger independent verification semantics.

### 3.4 Claude Squad and similar terminal orchestrators

Claude Squad and related projects combine tmux/session management, Git worktrees, multiple local coding agents, and diff/review UX.

Primary sources:

- immutable: https://github.com/smtg-ai/claude-squad/tree/ce1ffb4392b01f38e2c4599c7c84d2a93973b138
- navigation: https://github.com/smtg-ai/claude-squad

**Strategic consequence:** pane orchestration plus worktrees is already commoditized. Winds should avoid becoming a nicer tmux wrapper.

### 3.5 ctx

`ctx` is a particularly important memory/history donor and adjacent competitor.

**Observed strengths**

- open-source Rust CLI;
- discovers/imports persisted coding-agent histories;
- local Tantivy-backed lexical index with optional local semantic/hybrid retrieval;
- normalized sessions, events, metadata, relationships, and repository-activity records with cited retrieval;
- searches prior work and returns cited session/event identities;
- supports histories from many coding-agent harnesses;
- local/private by default and does not require a model API for its local search path;
- automatic indexing is the default and uses a persistent background daemon; manual indexing mode is available when the user wants no persistent daemon.

Primary sources:

- immutable repository: https://github.com/ctxrs/ctx/tree/94c0d32e1f4c3f7f7c78febdb916066d8df67c6c
- immutable indexing spec: https://github.com/ctxrs/ctx/blob/94c0d32e1f4c3f7f7c78febdb916066d8df67c6c/docs/daemon-semantic-indexing-spec.md
- navigation: https://github.com/ctxrs/ctx

**Strategic consequence:** cross-agent transcript/history search is not itself a Winds moat and should not be rebuilt prematurely. A ctx-style retrieval layer can inform or complement Winds, but imported transcript/history remains **retrieved evidence about prior conversation**, not canonical Winds task/evidence truth.

---

## 4. ACP protocol state as of 2026-08-20

ACP is now substantially richer than the earlier research snapshot. Winds should prefer it where it supplies stable semantics, while pinning an exact revision at implementation time.

### Stable/completed ACP v1 surfaces relevant to Winds

As of the research freeze, ACP primary sources show these as stabilized/completed:

- Session Config Options — 2026-02-04;
- Session List — 2026-03-09;
- Session Info Update — 2026-03-09;
- Session Resume — 2026-04-22;
- Session Close — 2026-04-23;
- Additional Workspace Roots / `additionalDirectories` — 2026-06-01;
- Session Delete — 2026-06-05;
- Session Usage Updates / `usage_update` — 2026-06-05;
- Message IDs — 2026-06-05;
- Model Config Category — 2026-06-24;
- Rust and TypeScript ACP SDKs at 1.0 — 2026-06-25;
- Request Cancellation / `$/cancel_request` — 2026-06-29;
- Boolean Config Options — 2026-07-06;
- Elicitation — 2026-07-22.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20; dated update/RFD state recorded above):

- https://agentclientprotocol.com/updates
- https://agentclientprotocol.com/rfds/additional-directories
- https://agentclientprotocol.com/rfds/request-cancellation
- https://agentclientprotocol.com/rfds/elicitation

Important details for Winds:

- `additionalDirectories` expands declared workspace scope but is **not** a sandbox.
- lifecycle-time additional roots are explicit and capability-gated;
- Winds must still enforce local filesystem authority independently where it claims enforcement;
- `usage_update` is agent-reported protocol telemetry and must not automatically become independent Winds accounting truth;
- session resume does not eliminate the need for Winds canonical task continuity because native resume capability/state can still vary by runtime.

### ACP work that remains evolving

- ACP v2 documentation/schema was published as **Draft** on 2026-07-20.
- Streamable HTTP/WebSocket remote transport work remains Active/evolving.
- additional draft RFDs continue to move quickly.

Primary sources (`MUTABLE_VENDOR_DOC`, retrieved 2026-08-20):

- https://agentclientprotocol.com/updates
- https://agentclientprotocol.com/rfds

**Winds rule:** never persist draft ACP semantics as permanent Winds product truth merely because a current SDK exposes them. Formal implementation must pin and audit the exact ACP protocol/SDK revision.

---

## 5. MCP direction

MCP is an interoperability protocol spanning tools, data/resources, extensions, tasks, and authorization. Those protocol surfaces do **not** make MCP the source of Winds session truth, authority truth, canonical work truth, or candidate-verification truth.

The dated **2026-07-28** MCP generation materially changed prior assumptions around stateless operation, extensions, tasks, and authorization. Winds must pin the exact MCP specification/SDK revision used by a future implementation and must not mirror transient protocol structure into permanent internal persistence without a product reason.

Primary source (dated release post, retrieved 2026-08-20):

- https://blog.modelcontextprotocol.io/posts/2026-07-28/

---

## 6. Research findings that affect the architecture

### 6.1 Agent-computer interface quality matters

SWE-agent demonstrates that the tool/interface surface materially changes software-agent performance.

- https://arxiv.org/abs/2405.15793

**Implication:** precise context navigation, deterministic tools, bounded outputs, and good error surfaces are product capabilities, not decoration.

### 6.2 Multi-agent quantity is not quality

MetaGPT, ChatDev, and related work support specialized roles and structured communication but also motivate caution around naive chains and coordination overhead.

- https://arxiv.org/abs/2308.00352
- https://aclanthology.org/2024.acl-long.810/

**Implication:** team contracts, bounded delegation, deterministic gates, and reviewer independence matter more than agent count.

### 6.3 Memory should be structured and selective

Long-horizon agent memory research supports separating durable structured state from the bounded model view.

- https://arxiv.org/abs/2512.13564
- https://arxiv.org/abs/2602.06052
- https://arxiv.org/abs/2603.19935

**Implication:** model-context compaction may be lossy; canonical work/evidence truth must not be.

### 6.4 Models should not define their own least privilege

AuthBench reports persistent difficulty in deriving correct least-privilege authorization even for strong models.

- https://arxiv.org/abs/2605.14859

**Implication:** agents may request authority; deterministic policy/human grants decide it.

### 6.5 Tool data is a hostile boundary

AgentDojo and security-principles work reinforce prompt-injection and confused-deputy risks.

- https://proceedings.neurips.cc/paper_files/paper/2024/hash/97091a5177d8dc64b1da8bf3e1f6fb54-Abstract-Datasets_and_Benchmarks_Track.html
- https://arxiv.org/abs/2505.24019

**Implication:** files, web, MCP, terminal output, and another agent's prose are inputs, never authority.

### 6.6 Evaluation must be candidate-bound and benchmark-skeptical

Repository-native deterministic gates bound to the exact candidate remain stronger product evidence than an agent saying “tests passed” or a broad benchmark score.

- https://openai.com/index/introducing-swe-bench-verified/
- https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/

---

## 7. Market convergence: what Winds should assume competitors can copy

Winds should assume serious products can eventually provide:

1. named workspaces/sessions;
2. fuzzy history and resume;
3. continue/fork/new semantics;
4. `@` file/folder context;
5. worktree-isolated tasks;
6. plan/read-only and implementation modes;
7. parallel workers/subagents;
8. model/runtime selection;
9. approvals/trust controls;
10. headless structured output;
11. skills/hooks/MCP/custom agents;
12. context compaction/history search;
13. background execution;
14. notifications/attention routing;
15. remote monitoring/continuation;
16. IDE/LSP/symbol intelligence;
17. time/token/cost visibility;
18. persistent/resumable agent terminals.

These are substrate. The moat must live above them.

---

## 8. The gaps Winds should own

### Gap A — runtime-neutral canonical continuity

The same developer task should survive Claude -> Codex -> Pi -> Goose -> another runtime without pretending vendor-private state transferred when it did not.

Winds must be able to report:

- canonical task/workstream identity;
- exact state transferred;
- state unavailable/not transferred;
- native session identity and proof level;
- reconstructed context sources.

### Gap B — external local authority

Direct execution authority and delegation authority are separate axes. A read-only Planner may coordinate a write-enabled Builder only when the human-approved team contract explicitly grants a delegation ceiling that covers that Builder capability.

```text
CHILD_EXECUTION_AUTHORITY
  ⊆ PLANNER_DELEGATION_CEILING
  ⊆ APPROVED_TEAM_AUTHORITY
  ⊆ HUMAN_GRANTED_AUTHORITY

PLANNER_EXECUTION_AUTHORITY
  ⊆ HUMAN_GRANTED_AUTHORITY
```

The Planner cannot self-expand its delegation ceiling, cannot grant a child capabilities outside the approved team contract, and does not need direct possession of every capability it is authorized to delegate. Authority remains external to model reasoning and host/domain scoped.

### Gap C — evidence separated from claims

```text
AGENT_REPORTED
!=
WINDS_OBSERVED
!=
HUMAN_DECIDED
```

Imported history, vendor telemetry, shell hooks, and agent prose retain provenance and never silently become independent Winds evidence.

### Gap D — exact-candidate verification

Tests, static checks, security gates, and independent review must bind to the exact candidate identity. A changed candidate invalidates stale verification/review.

### Gap E — independence-preserving review

A fresh reviewer should receive requirements and exact candidate evidence without automatically receiving builder persuasion/confidence or planner preference.

### Gap F — attention routing with authority/evidence semantics

The inbox should prioritize authority requests, blockers, exact-candidate review readiness, stale evidence, and human decisions — not merely terminal notifications.

### Gap G — truthful state semantics

```text
IDLE != DONE != VERIFIED != ACCEPTED
```

Process state, agent-turn state, task state, evidence state, and decision state must remain separate.

---

## 9. Donor/strategy decisions

### Adopt/integrate where mature

- ACP stable lifecycle/config/session features, version-pinned;
- agent-native structured interfaces such as Codex app-server;
- MCP for external tools/data, extensions, tasks, and authorization interoperability, version-pinned;
- system Git/worktrees;
- OS/runtime controls for real enforcement;
- existing local agent installations/authentication.

### Deep-study/integrate selectively

- Superset: workspace/parallel/remote orchestration UX;
- cmux: attention, terminal/session restoration, hibernation semantics;
- Conductor: task-workspace/review model;
- ctx: local cross-agent history indexing and cited retrieval;
- Junie/Zed/VS Code: session and navigation UX;
- Herdr/Warp: persistent terminal/control ownership;
- Kiro/Gemini/OpenCode: trust and capability policy;
- Pi/Goose/Droid: continuation/delegation semantics.

### Do not copy as architecture

- pane multiplexing as the core product;
- agent-count/fleet breadth as a moat;
- worktree-as-sandbox claims;
- transcript-as-memory claims;
- agent-provided “done/tests passed/safe” as acceptance evidence;
- one giant arbitrary-code plugin runtime before product need;
- automatic winner scoring.

---

## 10. Final research verdict

The final landscape does **not** support building a generic AI terminal, a worktree dashboard, a multiplexer with agent badges, or a fleet launcher as the central thesis. Those categories already have serious competitors.

The strongest direction is:

> **Winds = named workspace/session environment + runtime-neutral canonical continuity + heterogeneous agent compatibility + persistent execution + external local authority + exact candidate identity + independently observed evidence + independence-preserving review + explicit human landing decision.**

The durable distinction is not the provider list.

The durable distinction is that Winds can truthfully answer:

- **What task is this?**
- **Which workspace/candidate is authoritative?**
- **What context actually transferred?**
- **What was lost or reconstructed?**
- **What authority did each actor really hold?**
- **What happened on the machine?**
- **Which evidence belongs to this exact candidate?**
- **Was the review genuinely independent and candidate-current?**
- **What did the human decide?**

That is the basis for the future master plan in `008-agentic-development-master-plan.md`.
