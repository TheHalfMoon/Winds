# Final Agentic CLI / Terminal Landscape

**Status:** Research dossier only. No product implementation is authorized by this document.

**Research date:** 2026-08-20

**Canonical Winds base inspected:** `048cf59e8bdbe3a757b4d3ead214099ce18369bd`

**Relationship to active work:** This document does **not** amend Spec 003, does **not** authorize T069, and does **not** authorize a daemon, public runtime protocol, Agent Fleet, MCP runtime, plugin runtime, remote execution, or any other future product code. The active repository constitution and Spec 003 remain authoritative until explicitly amended through Constitution -> Spec -> Plan -> Tasks.

**Relationship to prior research:** This is the final broad market/research sweep after `006-agent-fleet-donor-audit.md`. It expands that audit with current JetBrains Junie, Factory Droid, Pi, Goose, Herdr, Warp, Zed, VS Code/Copilot, Claude Code, Codex, Gemini CLI, OpenCode, Cursor, Kiro, Qoder, Kimi Code, Cline, Mistral Vibe, Aider, Kilo, and the current ACP/MCP direction, plus recent agent-memory, multi-agent, ACI, evaluation, and agent-security research.

---

## 1. Executive conclusion

The market is converging quickly on the same primitives:

- persistent or resumable **sessions**;
- human-readable session titles and searchable history;
- parallel agents and subagents;
- Git worktrees for concurrent edits;
- plan/read-only versus implementation modes;
- per-tool or per-command approvals;
- background/headless execution;
- skills, hooks, MCP, and custom agents;
- native or structured machine interfaces for automation;
- remote monitoring/control;
- context compaction and repository-aware navigation.

Those features are becoming table stakes. Winds should not try to win by having one more model selector, one more `/fleet` command, or one more terminal sidebar.

The durable opportunity is to combine four properties that remain fragmented across the market:

> **Connected sessions across heterogeneous agent runtimes + inherited local authority + persistent terminal ownership + independently verified exact-candidate work.**

The product should therefore target this position:

> **Winds is the verified local runtime for agentic software development.**
>
> **Any agent. One runtime. Verified work.**

A stronger user promise is:

> **Run any agent. Build any team. Keep it alive. Gate its authority. Verify its work.**

---

## 2. Competitive landscape

### 2.1 JetBrains Junie CLI

**Strongest observed ideas**

- Multiple live sessions in one CLI instance; `/new` can start another task while existing sessions keep running in the background.
- `/history` provides task history across directories, live/saved status, search, and resume.
- Human-readable persistent titles through `/title` / `/rename`.
- `@` gives direct file/folder selection; IDE integration extends this to classes/symbols.
- Real-time follow-up prompts can be added while the agent is already working.
- Worktree menu for parallel file-changing sessions.
- Passive JetBrains IDE bridge: if a matching IDE is running, Junie can use indexing, semantic analysis, inspections, test configurations, refactorings, open-file context, and symbol-aware completion.
- `/import` can import guidelines, skills, commands, and MCP configuration from other coding agents.
- Remote continuation to a web/device surface.

**Winds lesson**

Winds must make workspace/session naming, history, fuzzy file/directory selection, and optional IDE intelligence first-class. It should not require an IDE, but when JetBrains/VS Code/LSP intelligence is available it should opportunistically improve symbol navigation and context selection.

Primary sources:

- https://junie.jetbrains.com/docs/junie-cli.html
- https://junie.jetbrains.com/docs/slash-commands.html
- https://junie.jetbrains.com/docs/junie-cli-worktrees.html
- https://junie.jetbrains.com/docs/junie-cli-jetbrains-ide-integration.html

### 2.2 Factory Droid

**Strongest observed ideas**

- Distinguishes **interaction mode** from **autonomy level**: Normal, Spec, and Mission are workflow shapes, while Off/Low/Medium/High control what can happen without approval.
- Mission mode has an orchestrator plus worker and validator agents, with independent model selection for workers/validators.
- Custom Droids are subagents with their own model/tool policy and isolated context.
- Background subagent execution returns a task ID; `TaskOutput` and `TaskStop` provide lifecycle control.
- A subagent can be **resumed by task ID with its full previous context preserved**; its autonomy is re-aligned to the current parent level on the next turn.
- Worktree execution, structured JSON/JSON-RPC modes, session search/resume/fork, and separate spec model selection.
- Organization maximum autonomy clamps local/child choices.

**Winds lesson**

Separate role/mode from authority. A planner should be able to choose a worker model without acquiring extra machine authority. Child authority must never exceed parent/team/human ceilings. Resumable workers should be a primitive, not an accidental vendor feature.

Primary sources:

- https://docs.factory.ai/droid-cli/quickstart
- https://docs.factory.ai/droid-cli/cli-reference
- https://docs.factory.ai/autonomy-and-safety/specification-mode
- https://docs.factory.ai/autonomy-and-safety/auto-run
- https://docs.factory.ai/harness/subagents
- https://docs.factory.ai/droid-exec/overview

### 2.3 Pi

**Strongest observed ideas**

- Sessions are durable JSONL records grouped by working directory.
- `--continue`, interactive resume, explicit names, `/tree`, `/fork`, `/clone`, and `/compact` have distinct meanings.
- Session picker supports search/rename/delete and project-oriented continuation.
- Session history is a **tree**, not merely a flat transcript; branch summaries can preserve useful information when moving between branches.
- Full session storage can retain history while compaction supplies a bounded model context.
- Pi's extension ecosystem and community subagent/intercom tools demonstrate parent/child delegation and model-per-role patterns without requiring one provider.

**Winds lesson**

`NEW_SESSION != NEW_TASK`. Winds should preserve a canonical workstream/session graph even when the model context is compacted, and should distinguish continue, fork, clone, and new. Human names should hide native IDs in ordinary workflows.

Primary source:

- https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/sessions.md

### 2.4 Goose (AAIF / Block lineage)

**Strongest observed ideas**

- Named sessions, resume, fork, history, editable conversation history, and extension/container options.
- Custom agents define reusable roles and optional model preferences.
- Delegated agents run in separate sessions and can use a different model.
- Delegated subagents do not automatically inherit the full parent conversation.
- One custom agent can delegate to another; repeatable chains are better represented as recipes rather than role definitions.
- Recipes separate reusable workflow from reusable agent persona/role.
- MCP/extension support remains central.

**Winds lesson**

Keep `Role`, `Workflow`, `Runtime`, and `Model` separate. Cross-agent delegation should have an explicit context-inheritance policy instead of dumping parent history into every child.

Primary sources:

- https://github.com/aaif-goose/goose/blob/main/documentation/docs/guides/goose-cli-commands.md
- https://github.com/aaif-goose/goose/blob/main/documentation/docs/guides/context-engineering/custom-agents.md
- https://goose-docs.ai/docs/guides/sessions/session-management/
- https://goose-docs.ai/docs/guides/recipes/recipe-reference/

### 2.5 Herdr

**Strongest observed ideas**

- Agent-aware real terminal panes with `idle`, `working`, and `blocked` state.
- Broad coding-agent detection including Pi, Copilot CLI, Devin, Kimi, Hermes, Qoder, Droid, OpenCode, Kilo, Claude Code, Codex, Cursor Agent, Amp, Grok CLI, Kiro, and others.
- Persistent terminal/session ownership and detach/reattach-oriented workflows.
- Agent automation primitives such as start/prompt/wait/read.
- Remote attachment and a control API.
- Clear distinction between stronger lifecycle-hook state signals and weaker screen-manifest heuristics.

**Winds lesson**

A future persistent session owner is strategically important; teams of agents should not disappear when the client UI closes. But Winds must go beyond multiplexer state by tracking delegation, authority, exact workspace/candidate identity, evidence, and human decisions. Heuristic state must be source-labelled rather than promoted to truth.

Primary sources:

- https://herdr.dev/docs/agents/
- https://herdr.dev/docs/agent-automation/
- https://herdr.dev/docs/session-state/
- https://herdr.dev/docs/persistence-remote/
- https://herdr.dev/docs/socket-api/

### 2.6 Warp

**Strongest observed ideas**

- An agent can operate inside the same live PTY the human is viewing, including interactive programs such as `psql`, `vim`, `python`, `gdb`, and long-running services.
- The human can **Take Over** the same session and later hand control back.
- Proposed commands can be approved once or auto-approved for a bounded session pattern.

**Winds lesson**

Human terminal and agent terminal should share the same execution substrate. `Take Over` / `Hand Back` should eventually work for any supported agent runtime, not just a Winds-native model.

Primary source:

- https://docs.warp.dev/agent-platform/capabilities/full-terminal-use

### 2.7 Zed

**Strongest observed ideas**

- Three agent paths: native agent, ACP External Agents, and Terminal Threads.
- Terminal Threads let native CLIs keep their own authentication, model/provider, skills, MCP, and configuration while Zed supplies organization/history UX.
- Parallel threads are grouped by project and can use different agents.
- Worktree isolation for parallel edits.
- Thread history and some external-session import.

**Winds lesson**

A terminal-first heterogeneous runtime is correct. Winds should provide one workspace/session surface over ACP agents and native CLI/TUI sessions without forcing agent credentials or model configuration into a proprietary Winds provider.

Primary sources:

- https://zed.dev/docs/ai/agents
- https://zed.dev/docs/ai/terminal-threads
- https://zed.dev/docs/ai/parallel-agents
- https://zed.dev/docs/ai/external-agents

### 2.8 VS Code / Microsoft / GitHub Copilot CLI

**Strongest observed ideas**

- Agent sessions are becoming a shared abstraction across local, background, CLI, cloud, and remote surfaces.
- VS Code can hand off a local session to Copilot CLI with conversation/context carried forward.
- Background Copilot CLI sessions run outside the VS Code window and can continue after the window closes.
- Sessions can contain multiple chats sharing workspace/isolation.
- Agent-host tooling can list sessions, create sessions/chats, read another session's recent context, and send messages across sessions.
- Agents window is workspace-first and supports worktree isolation and session configuration.
- Copilot CLI `/fleet` decomposes work into dependency-aware parallel subagents; subagents can use different specified models/custom agents.
- Searchable session history, remote control, research agent, a different-model rubber-duck critic, LSP code intelligence, and on-demand tool loading.
- VS Code terminal shell integration provides command/cwd/exit telemetry plus recent-command and recent-directory pickers.

**Winds lesson**

Connected sessions and cross-harness handoff are now strategic table stakes. Winds should exceed them by making continuity runtime-neutral and evidence-aware, not tied to one vendor account or harness.

Primary sources:

- https://code.visualstudio.com/docs/agents/concepts/sessions
- https://code.visualstudio.com/docs/agents/agent-types/copilot-cli
- https://code.visualstudio.com/docs/agents/run/sessions/manage-sessions
- https://code.visualstudio.com/docs/agents/agents-window
- https://code.visualstudio.com/docs/terminal/shell-integration
- https://docs.github.com/en/copilot/concepts/agents/copilot-cli
- https://docs.github.com/en/copilot/concepts/agents/copilot-cli/fleet
- https://docs.github.com/en/copilot/concepts/agents/copilot-cli/about-remote-control

### 2.9 Claude Code

**Strongest observed ideas**

- Separate primitives for subagents, agent view, agent teams, worktrees, and batch work.
- Subagents get isolated context, custom prompts, model/tool/permission controls, optional persistent memory, background execution, and optional worktree isolation.
- Agent Teams provide lead + teammates, shared tasks, and inter-agent messaging.
- Worktree isolation is well integrated into parallel workflows.
- Agent Teams explicitly document coordination/token overhead and current resumption limitations.

**Winds lesson**

The lead/team UX is validated, but Winds should generalize it across heterogeneous runtimes/models and make authority inheritance and session continuity stronger than vendor-local teams.

Primary sources:

- https://code.claude.com/docs/en/agents
- https://code.claude.com/docs/en/sub-agents
- https://code.claude.com/docs/en/agent-teams
- https://code.claude.com/docs/en/worktrees

### 2.10 OpenAI Codex

**Strongest observed ideas**

- Structured app-server surface rather than TUI scraping.
- Threads/turns/items expose lifecycle in machine-readable form.
- Approval requests carry thread/turn identity and support session-scoped acceptance and policy/network amendments.
- Codex remains a strong reference for separating approval policy from sandbox policy and for structured control of coding-agent work.

**Winds lesson**

Use native structured surfaces whenever available. Do not parse terminal rendering when an app server / ACP adapter can give exact lifecycle and approval events.

Primary source:

- https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md

### 2.11 Gemini CLI

**Strongest observed ideas**

- Folder trust discovers project-local commands, MCP, hooks, skills, and setting overrides before trust is granted.
- Untrusted workspaces run in restricted mode rather than silently loading project automation.
- Sandboxing supports multiple backends and tool-level isolation.
- **Sandbox expansion** asks for a narrow additional permission when a command needs filesystem/network access beyond the current sandbox.
- Mutating tools require confirmation by default.
- `@` references files/directories.
- Checkpointing preserves pre-edit project snapshots.

**Winds lesson**

Workspace discovery is not trust. Project automation, MCP, hooks, skills, and environment inputs should be inventoried before execution. Authority escalation should be explicit and narrow.

Primary sources:

- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/trusted-folders.md
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/sandbox.md
- https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/tools.md
- https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/checkpointing.md

### 2.12 OpenCode

**Strongest observed ideas**

- Fine-grained `allow` / `ask` / `deny` rules over actions/resources.
- Per-agent permission overrides.
- Separate read-only/explore/plan-style agents.
- Rules can cover shell patterns, external directories, MCP tools, skills, and subagents.
- Durable project approvals cannot override explicit deny rules.

**Winds lesson**

Use a capability/resource policy rather than a single `sandboxed=true` flag. Explicit denies must dominate convenience approvals.

Primary sources:

- https://opencode.ai/docs/agents
- https://opencode.ai/docs/permissions

### 2.13 Cursor Agent CLI

**Strongest observed ideas**

- Explicit workspace root and agent worktree support.
- Resume/continue/history surfaces.
- Human command approval in interactive mode.
- Structured output in non-interactive mode.

**Winds lesson**

Workspace identity, continuation, and worktree selection should be uniform across runtimes rather than relearned per CLI.

Primary source:

- https://cursor.com/docs/cli/using

### 2.14 AWS Kiro CLI

**Strongest observed ideas**

- Spec workflow combines requirements/design/planning with task execution and verification.
- Capability-based permissions include filesystem, shell, web, MCP, subagent, skill, context, diagnostics, and sandbox network.
- `deny > ask > allow` across user/workspace/agent/session scopes.
- Permission configuration files are protected by an unoverrideable deny so the agent cannot rewrite its own policy.
- Hooks can observe/block lifecycle/tool events.
- Context is explicitly separated into persistent agent resources, temporary session context, and knowledge bases; context usage is inspectable.

**Winds lesson**

Protect the policy plane from the agents it governs. Make context composition visible and measured. Keep deterministic lifecycle hooks as a potential gate input, but never confuse hook output with independent Winds evidence unless Winds owns the observation.

Primary sources:

- https://kiro.dev/docs/cli/v3/specs/
- https://kiro.dev/docs/cli/chat/permissions/
- https://kiro.dev/docs/cli/hooks/
- https://kiro.dev/docs/cli/chat/context/

### 2.15 Qoder CLI

**Strongest observed ideas**

- `--cwd`, repeated `--add-dir`, remote modes, ACP server mode.
- Agents, skills, hooks, plugins, and MCP as explicit CLI subcommands.
- Structured/non-interactive automation surfaces.

**Winds lesson**

Multi-root and ACP capability negotiation are important, but Winds should replace path-heavy UX with a fuzzy human picker and explicit per-root authority.

Primary source:

- https://docs.qoder.com/cli/cli-reference

### 2.16 Kimi Code CLI

**Strongest observed ideas**

- Durable sessions grouped by working directory with session metadata and per-agent event streams.
- Resume/continue/session picker.
- Main agent + isolated subagents; subagents return conclusions without polluting the main agent's history.
- Skills, hooks, MCP, and `--add-dir` support.
- Session storage includes request traces useful for debugging.

**Winds lesson**

Subagent context isolation and persistent event records are good defaults. Winds should add exact evidence provenance and cross-runtime continuity above them.

Primary sources:

- https://moonshotai.github.io/kimi-code/en/guides/sessions.html
- https://moonshotai.github.io/kimi-code/en/customization/agents
- https://moonshotai.github.io/kimi-code/en/reference/kimi-command

### 2.17 Cline

**Strongest observed ideas**

- Shared engine across CLI, IDE, JetBrains, SDK, and multi-agent Kanban.
- Interactive and headless/JSON automation.
- Plan/act, MCP, checkpoints, rules, skills, provider configuration.
- Kanban gives parallel agents their own worktrees and dependency chains.

**Winds lesson**

A shared execution engine with multiple clients is a proven direction. Winds should keep the local truth/authority engine independent from the TUI rather than bake all semantics into one UI process forever.

Primary sources:

- https://github.com/cline/cline
- https://github.com/cline/cline/blob/main/apps/cli/README.md

### 2.18 Mistral Vibe

**Strongest observed ideas**

- Modern CLI/TUI, `@` path completion, stateful terminal, project-aware context, multiple agent profiles, subagent task delegation, approvals, trust folders, and programmatic JSON/streaming modes.
- Explicit max turns, max price, and token budgets in programmatic mode.
- ACP adapter exists in the codebase.

**Winds lesson**

Budget ceilings should be first-class team/session policy. Modern file-selection UX is table stakes.

Primary source:

- https://github.com/mistralai/mistral-vibe

### 2.19 Aider

**Strongest observed idea**

- Repository map uses a concise symbol-oriented representation of the codebase instead of sending every file.

**Winds lesson**

Use an inspectable bounded context map, ideally enriched by LSP/IDE intelligence, instead of raw repository dumping.

Primary source:

- https://aider.chat/docs/repomap.html

### 2.20 Kilo and other long-tail CLIs

Kilo provides workspace-scoped session history/search, session rename/export/delete, past-chat context attachment, resume, remote relay, and a local daemon. Herdr's current support table also demonstrates a fast-moving long tail: Devin CLI, Hermes, Amp, Grok CLI, Kiro, Qoder, Kimi, Cursor, OpenCode, Droid, Pi, Cline, Gemini, and others.

**Winds lesson**

Do not hard-code the product around today's top five agent names. Build capability discovery and compatibility tiers so new CLIs can enter without rewriting the user model.

Primary sources:

- https://kilo.ai/docs/code-with-ai/platforms/cli
- https://kilo.ai/docs/code-with-ai/agents/session-history
- https://herdr.dev/docs/agents/

---

## 3. Protocol direction

### 3.1 ACP is the preferred coding-agent interoperability layer

The 2026 ACP direction strengthens the existing recommendation from research 006:

- standardized session discovery/listing;
- session metadata/title updates;
- session close;
- session configuration options for model/mode/reasoning selectors;
- active work on remote transports;
- evolving multi-root lifecycle semantics.

Winds should not invent a public agent protocol while ACP can carry the needed lifecycle/control semantics. Winds may still need private internal types for authority, evidence, and orchestration that ACP does not define.

Primary sources:

- https://agentclientprotocol.com/updates
- https://agentclientprotocol.com/announcements/session-list-stabilized
- https://agentclientprotocol.com/announcements/session-info-update-stabilized
- https://agentclientprotocol.com/announcements/session-close-stabilized
- https://agentclientprotocol.com/announcements/session-config-options-stabilized

### 3.2 MCP is the tool/data interoperability layer, not Winds session truth

The final MCP `2026-07-28` specification moved to a stateless core, formal extensions, Tasks, authorization hardening, and deprecations for roots/sampling/logging. This is a reminder that Winds must pin a concrete MCP revision when implementation is authorized rather than mirror old MCP assumptions into its persistent schema.

MCP should expose tools/data; it should not become the source of truth for Winds sessions, authority, or candidate verification.

Primary source:

- https://blog.modelcontextprotocol.io/posts/2026-07-28/

---

## 4. Research findings that materially affect Winds

### 4.1 Agent-computer interface quality changes agent performance

**SWE-agent: Agent-Computer Interfaces Enable Automated Software Engineering** shows that the interface/tool surface supplied to an agent materially affects software-engineering performance. This supports investing in agent-friendly navigation, precise tools, good error surfaces, and bounded context selection rather than treating Winds as a passive terminal wrapper.

- https://arxiv.org/abs/2405.15793

### 4.2 Generalist coding agents need controlled environments and evaluation

**OpenHands** frames software agents as generalist actors using code, CLI, and browser capabilities and emphasizes sandboxed execution, multi-agent coordination, and benchmark integration.

- https://arxiv.org/abs/2407.16741

### 4.3 Naive multi-agent chaining can cascade hallucinations

**MetaGPT** explicitly identifies cascading hallucinations in naively chained multi-agent systems and uses standardized operating procedures/roles to reduce inconsistency.

- https://arxiv.org/abs/2308.00352

**ChatDev** similarly shows value in specialized roles and structured communication across design, coding, and testing.

- https://aclanthology.org/2024.acl-long.810/

**Winds implication:** multi-agent quantity is not a quality metric. Team contracts, role boundaries, structured handoffs, deterministic gates, and independent review matter more than spawning more workers.

### 4.4 Parallel candidate exploration can improve quality, but must not auto-select

**Cross-Team Collaboration** explores multiple decision paths with multiple teams rather than a single development chain.

- https://arxiv.org/abs/2406.08979

**Winds implication:** support alternate candidates and independent exploration, but retain the constitutional rule that Winds never invents a magic winner score and the human selects the candidate.

### 4.5 Agent runtimes increasingly resemble operating systems

**AIOS** studies scheduling, context switching, memory, storage, tool management, and access control as kernel-like services for agents.

- https://arxiv.org/abs/2403.16971

**Winds implication:** the layered runtime idea is valid, but Winds should not import a heavy generic agent OS prematurely. Build only concrete local developer primitives justified by the product.

### 4.6 Memory must be structured and selective, not raw transcript stuffing

Recent memory research treats memory as a first-class primitive for long-horizon agents:

- **Memory in the Age of AI Agents** distinguishes factual, experiential, and working memory and analyzes formation/evolution/retrieval.
- **Rethinking Memory Mechanisms of Foundation Agents in the Second Half** emphasizes long-horizon context explosion and selective reuse.
- **Memori** reports that structured summaries/semantic representation can outperform raw full-context approaches with substantially fewer tokens on LoCoMo.

Sources:

- https://arxiv.org/abs/2512.13564
- https://arxiv.org/abs/2602.06052
- https://arxiv.org/abs/2603.19935

**Winds implication:** preserve full local transcripts/artifacts where policy allows, but construct model context from a structured canonical work memory and evidence memory. Compaction may change the model view; it must never rewrite canonical task/evidence truth.

### 4.7 Models should not be trusted to infer their own least privilege

**Do Coding Agents Understand Least-Privilege Authorization? (AuthBench, 2026)** finds that frontier models can simultaneously omit needed permissions and grant unnecessary/sensitive access, and more reasoning does not solve the basic mismatch.

- https://arxiv.org/abs/2605.14859

**Winds implication:** authority must be an external deterministic policy system. A planner may request authority; it must not authoritatively decide its own permissions or a child's permissions.

### 4.8 Prompt injection makes tool data a hostile boundary

**AgentDojo (NeurIPS 2024)** demonstrates attacks in which untrusted tool-returned data hijacks tool-using agents.

- https://proceedings.neurips.cc/paper_files/paper/2024/hash/97091a5177d8dc64b1da8bf3e1f6fb54-Abstract-Datasets_and_Benchmarks_Track.html

**LLM Agents Should Employ Security Principles** argues for defense in depth, least privilege, complete mediation, and psychologically usable controls.

- https://arxiv.org/abs/2505.24019

**Winds implication:** project files, MCP output, web results, terminal output, and another agent's prose are inputs, not authority. Every consequential local action still passes Winds policy/approval boundaries.

### 4.9 Evaluation itself must be treated skeptically

SWE-bench Verified improved reproducibility through human validation and containerized evaluation, but OpenAI later documented material benchmark limitations for frontier capability measurement.

Sources:

- https://openai.com/index/introducing-swe-bench-verified/
- https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/

**Winds implication:** do not build the product around one leaderboard. The core verification primitive remains repo-native deterministic gates bound to the exact candidate, with benchmark suites used only as additional product evaluation.

---

## 5. Market convergence: features that are now table stakes

A credible future Winds agentic environment should assume the following eventually become baseline expectations:

1. Named workspaces and named sessions.
2. Searchable/fuzzy session history and resume.
3. `continue`, `fork`, and `new` as distinct operations.
4. Human-friendly file/folder context pickers and `@` references.
5. Worktree-isolated parallel edit sessions.
6. Plan/read-only versus implementation modes.
7. Subagents/parallel workers.
8. Model selection per session/role.
9. Tool/command approval and trust controls.
10. Headless structured output for automation.
11. Skills/hooks/MCP/custom-agent extension points.
12. Session compaction and context inspection.
13. Background work and an attention/status surface.
14. Remote monitoring/continuation.
15. IDE/LSP/symbol intelligence where available.
16. Cost/token/time budget visibility.

Winds should implement these only when their prerequisite specs authorize them, but none is a durable moat by itself.

---

## 6. The gaps Winds should own

### Gap A — connected continuity across different agent runtimes

Most tools resume their own sessions well. Fewer can continue the **same developer task** across Claude -> Pi -> Goose -> Codex -> another runtime while retaining workspace/task/evidence truth and explicitly stating what vendor-private context did not transfer.

### Gap B — a universal authority hierarchy

Vendor tools have good permission systems, but policy semantics differ and child-agent inheritance is inconsistent. Winds can enforce:

`CHILD_AUTHORITY ⊆ PARENT_AUTHORITY ⊆ TEAM_AUTHORITY ⊆ HUMAN_GRANTED_AUTHORITY`

independently of model prose.

### Gap C — evidence separated from agent claims

Many systems show that an agent says tests passed. Winds can independently bind exact process/Git/test/review evidence to the exact candidate tree.

### Gap D — persistent agent terminal runtime plus verification

Herdr and VS Code demonstrate persistence/background ownership. Winds can combine persistent sessions with exact identity, authority provenance, and verification rather than treating persistence as only a multiplexer concern.

### Gap E — frictionless directory/context navigation across every CLI

Different CLIs require different path flags and session IDs. Winds can make the user remember only workspace/session names and goals, while fuzzy selectors resolve directories, files, symbols, changes, tests, prior sessions, and artifacts.

### Gap F — independent-review context isolation

A reviewer should inherit requirements and exact candidate evidence without automatically inheriting builder persuasion, confidence, or hidden conversational bias.

### Gap G — attention routing across many agents

The human should manage decisions and authority requests, not panes. A single inbox can rank authority requests, blocked work, review readiness, and informational completions.

---

## 7. Final donor decisions

### Adopt / integrate where mature

- **ACP** for coding-agent lifecycle/interoperability.
- **Agent-native structured interfaces** such as Codex app-server where stronger than terminal scraping.
- **MCP** for external tools/data, version-pinned at implementation time.
- Existing local CLIs and their existing authentication rather than a mandatory Winds model gateway.
- System Git/worktrees and OS/runtime controls for real authority, not prompt-only safety.

### Deep-study UX/behavior

- Junie: named/history sessions, `@` file/folder/symbol UX, optional IDE bridge.
- Droid: mode/autonomy separation, resumable subagents, worker/validator models.
- Pi: session tree/fork/clone/compact semantics.
- Goose: agent vs recipe vs skill separation and isolated delegation.
- Herdr: persistent agent terminals and state provenance.
- Warp: shared PTY takeover.
- Zed/VS Code: project-grouped sessions, terminal/external agent coexistence, handoff.
- Claude/Copilot: lead/team/fleet coordination patterns.
- Gemini/Kiro/OpenCode: trust, sandbox expansion, capability policies.
- Aider: bounded repository map.

### Compatibility targets, not core architecture

Cline, Kimi, Qoder, Cursor, Kilo, Mistral Vibe, Aider, Devin, Amp, Grok, Hermes, Kiro, Copilot CLI, and future CLIs should enter through capability-discovered adapters. They must not force agent-specific product branches throughout Winds core.

---

## 8. Research verdict

The final landscape does **not** support building a generic "AI terminal" or a clone of Herdr/Zed/Claude Teams. Those categories are already competitive and converging.

The strongest defensible direction is:

> **Winds = workspace/session operating environment + universal agent runtime + persistent execution + local authority broker + connected cross-agent memory + exact-candidate evidence.**

The next document, `008-agentic-development-master-plan.md`, translates this research into the future product/architecture plan while preserving the current Spec 003 boundary.
