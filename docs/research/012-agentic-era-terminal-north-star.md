# Winds Agentic-Era Terminal North Star

**Status:** Research-only product and architecture plan. Non-authorizing.

**Research freeze:** 2026-09-05

**Canonical Winds base inspected:** `dfa2c524df7ce8a6d4aa481a61d2bbf0fbe87c3e`

**Relationship to existing roadmap:** This document extends `008-agentic-development-master-plan.md`, `010-verified-learning-loop-roadmap.md`, `011-herdr-parity-and-beyond-roadmap.md`, and Spec 006 research. It does not supersede the Constitution, an accepted specification, plan, task contract, or exact-head implementation evidence.

**Authority firewall:** This document does **not** authorize implementation, runtime installation, provider authentication, model calls, browser control, remote execution, daemon creation, UI framework adoption, dependency adoption, background agents, autonomous landing, or any widening of Spec 006. Future implementation must still follow the repository's canonical `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing` discipline.

**Licensing firewall:** Warp product behavior and public architecture are research references. The Warp repository is predominantly AGPL-3.0, while selected UI framework crates are MIT. Winds MUST NOT copy or adapt AGPL implementation code into its MIT/Apache-2.0 codebase without an explicit compatible licensing decision. Product ideas, interaction principles, and independently designed implementations may be studied clean-room style. Pi and every other donor/reference require implementation-time license and provenance revalidation.

---

## 1. Founder product directive

Winds should become the terminal and local development runtime developers choose for the agentic era, not merely a wrapper around existing agent CLIs.

The target experience is:

```text
BEAUTIFUL_ENOUGH_TO_REPLACE_THE_DAILY_TERMINAL
FAST_ENOUGH_TO_FEEL_NATIVE
SIMPLE_ENOUGH_FOR_A_NEW_DEVELOPER
POWERFUL_ENOUGH_FOR_AGENT_FLEETS
LOCAL_FIRST_AND_TRUTHFUL
ANY_AGENT_ANY_MODEL_ANY_PROVIDER
DAYS_LONG_WORK_WITHOUT_CANONICAL_CONTEXT_LOSS
BROWSER_AND_CODE_IN_ONE_VERIFIED_REALITY
VERIFY_BEFORE_ACCEPT
```

The ambition is not to make a slightly better Warp, Codex, Pi, terminal emulator, or multi-agent launcher. The ambition is to combine the best interaction lessons from modern terminals with Winds' unique verification and authority model, then add primitives that become more valuable as agents become more autonomous.

### Product definition

> **Winds is the verified agentic development runtime: a beautiful terminal, a universal agent workbench, a durable memory and execution substrate, a multi-provider control plane, and a browser-aware verification environment bound to exact software reality.**

### Short positioning candidates

```text
The terminal for the agentic era.
Any agent. Any model. One continuous workspace. Verified reality.
Build for days. Switch models. Keep the truth.
```

---

## 2. What to learn from Warp — and where Winds must go beyond it

Warp demonstrates that terminal adoption is driven by product feel as much as raw capability. The important lessons are not a theme or gradient. They are structural.

### 2.1 Typed interaction instead of an undifferentiated byte stream

Warp's block model treats commands, outputs, agent exchanges, plans, diffs, and other interactions as addressable UI objects. Winds should adopt the **principle**, not the implementation:

```text
Terminal event stream
      ↓
Typed Winds Interaction Objects
      ↓
render / search / collapse / copy / cite / attach / verify / revisit
```

A command should remain a real terminal command, but its lifecycle can also be represented as a typed record with:

- exact command identity;
- cwd/workspace/session identity;
- start/end/lifecycle state;
- bounded output references;
- exit status when observed;
- Git before/after observations where applicable;
- source and authority labels;
- links to agent task/evidence where applicable.

### 2.2 One universal input

The fastest daily UX should have one high-quality editor that can express:

- shell commands;
- natural-language agent prompts;
- `/` actions;
- `@` file/symbol/session/evidence/browser references;
- model/provider selection;
- explicit authority mode;
- image/paste/structured attachment input.

The user should not need to navigate a maze of modes just to ask an agent, run `git status`, attach a file, or switch model.

### 2.3 Agent sessions must be visually manageable

Borrow the interaction lesson behind vertical tabs, attention indicators, notifications, and code review, but bind them to Winds semantics:

```text
Session
├── role
├── canonical task
├── runtime
├── provider/model
├── candidate/worktree
├── authority
├── attention state
├── context health
├── cost/budget
└── evidence state
```

A developer supervising eight agents should be able to answer at a glance:

- Which session needs me?
- Which exact candidate is it modifying?
- Which model/provider is it using?
- Is it blocked, waiting, running, reviewing, or done?
- Is the result merely agent-reported, deterministically verified, independently reviewed, or human accepted?

### 2.4 Rich terminal ergonomics are a product requirement

Future Winds should plan for:

- excellent multi-line editing;
- syntax-aware command input;
- completions and history search;
- command blocks;
- collapsible output;
- copy/share/export at block granularity;
- smooth keyboard and mouse navigation;
- tabs/panes/workspaces;
- inline diffs;
- file tree and optional code preview/editor surface;
- LSP-backed navigation when available;
- notifications and attention routing;
- themes and typography worthy of daily use;
- fast startup and low-idle-resource usage;
- Linux/macOS/Windows first-class behavior where platform claims are genuinely proven.

Winds should not force users to choose between a beautiful terminal and a rigorous agent control plane.

---

## 3. The Winds differentiator: Verified Reality, not just terminal + agent

Browser support alone is not unique. Multi-agent support alone is not unique. Provider switching alone is not unique. Persistent chat alone is not unique.

The new differentiated abstraction should be a **Winds Reality**.

### 3.1 Winds Reality

A Winds Reality is an explicitly identified development state:

```text
WindsReality
- workspace identity
- canonical task/workstream identity
- exact Git base/candidate/tree identity where applicable
- worktree/capsule identity
- service/process topology references
- terminal/session identities
- agent runtime identities
- provider/model identities
- browser-profile/browser-context identity
- selected browser pages/tabs and local-app routes
- canonical memory checkpoint
- authority policy/ceiling
- deterministic evidence references
- review references
- human decisions
```

This is not a claim that Winds snapshots every byte of a machine. It is a canonical graph of identities, observations, references, and evidence sufficient to reason truthfully about one development state.

### 3.2 Reality Graph

Winds should eventually expose a **Reality Graph** connecting:

```text
TASK
  ↓
SESSION ──→ AGENT RUNTIME ──→ PROVIDER / MODEL
  ↓
WORKTREE ──→ EXACT CANDIDATE
  ↓              ↓
SERVICES       EVIDENCE
  ↓              ↓
BROWSER ─────→ USER-VISIBLE RESULT
  ↓
ANNOTATIONS / SCREENSHOTS / DOM-SELECTED REFERENCES
```

The graph enables capabilities that ordinary terminals cannot provide honestly.

### 3.3 Reality Branches — candidate unique feature

A **Reality Branch** is a fork of canonical development state for comparative agent work.

Example:

```text
Task: redesign checkout

Reality A
- candidate A
- Claude builder
- localhost:3001
- browser profile A
- screenshots A
- tests A

Reality B
- candidate B
- Codex builder
- localhost:3002
- browser profile B
- screenshots B
- tests B

Winds compare A B
→ code diff
→ test/evidence diff
→ browser visual/DOM diff
→ performance diff where measured
→ accessibility diff where measured
→ authority/provenance diff
→ human decision
```

This is substantially more useful than “run several agents.” Winds can make parallel alternatives **comparable as verified realities**.

### 3.4 Reality invariants

```text
BROWSER_SCREENSHOT != VERIFICATION_BY_ITSELF
MODEL_JUDGMENT != HUMAN_ACCEPTANCE
BROWSER_STATE != GIT_STATE
WORKTREE != SECURITY_SANDBOX
RUNTIME != MODEL
SESSION != TASK
CONVERSATION_MEMORY != CANONICAL_MEMORY
CHANGED_CANDIDATE_INVALIDATES_CANDIDATE_BOUND_EVIDENCE
REALITY_BRANCH_NEVER_IMPLIES_WINNER
```

---

## 4. Verified Browser: the browser should be native to Winds reality

The user requirement is not merely “add a browser like Codex.” Winds should eventually provide a browser workflow designed for software development and verification.

### 4.1 Product surface

Future browser capabilities should include, subject to formal specification and safety review:

- browser tabs inside the Winds workbench;
- localhost and remote web navigation;
- user-controlled signed-in profiles with explicit boundaries;
- element/region annotation for agent instructions;
- screenshots and visual checkpoints;
- DOM/semantic element references where safe and available;
- console/network observations where explicitly enabled;
- download/upload workflows with clear provenance;
- responsive viewport presets;
- accessibility inspection;
- frontend visual regression evidence;
- browser action timeline;
- browser state linked to exact Winds session/task/reality;
- agent browser control only under explicit site/authority policy.

### 4.2 Browser Twin

The more differentiated feature should be **Browser Twin**: each relevant Reality Branch can have its own isolated browser context tied to its candidate/service topology.

```text
Reality A -> candidate A -> service A -> browser context A
Reality B -> candidate B -> service B -> browser context B
```

The user can inspect both simultaneously without cookies, routes, ports, screenshots, or agent navigation silently crossing realities.

### 4.3 Verified Browser Review

A future Winds review flow should be able to produce a structured browser review report:

```text
Candidate identity: <exact tree>
Route: /checkout
Viewport: 1440x900
Observed browser context: <id>
Assertions:
- visual checkpoint present
- no console errors observed during bounded flow
- expected element text/role present
- optional accessibility checks
- optional performance checks
- optional interaction script result
Evidence source: WINDS_OBSERVED
Human annotations: HUMAN_DECIDED
Agent interpretation: AGENT_REPORTED
```

Winds must keep those authorities distinct.

### 4.4 Safety posture

Browser control can reach sensitive authenticated systems. Future design MUST include:

- explicit host allow/ask/deny policy;
- profile isolation;
- credential-entry boundary that keeps secrets out of prompt/transcript storage where possible;
- download/upload authority boundaries;
- irreversible-action confirmation policy;
- visible account/site context;
- safe handling of prompt injection from web content;
- agent/browser telemetry separated from verification claims.

---

## 5. Provider Mesh: many APIs and models inside one Winds session

Pi demonstrates a useful product property: a session can work across many providers and models rather than belonging permanently to one vendor.

Winds should go further by keeping the **canonical session independent from the provider/model**.

### 5.1 Provider Mesh principle

```text
WINDS_SESSION != PROVIDER_SESSION
WINDS_SESSION != MODEL
MODEL_SWITCH != NEW_TASK
PROVIDER_SWITCH != CONTEXT_RESET
```

A single Winds session may eventually use:

- OpenAI/Codex;
- Anthropic/Claude;
- Google/Gemini;
- xAI/Grok;
- OpenRouter;
- Azure OpenAI;
- Amazon Bedrock;
- Vertex AI;
- local Ollama/llama.cpp/vLLM/LM Studio compatible endpoints;
- organization gateways/proxies;
- future provider adapters.

Support is not automatic authority to store credentials or call every provider. Each provider path must have explicit authentication, protocol, cost, privacy, and capability semantics.

### 5.2 One session, multiple model roles

Example UX:

```text
winds session checkout-redesign

Planner:      openai/gpt-5.x
Builder A:    anthropic/claude-...
Builder B:    openai/codex-...
Reviewer:     google/gemini-...
Visual critic: provider/model-with-vision
Local helper: ollama/qwen-coder
```

The task and canonical memory remain Winds-owned. Model-specific transcripts and native session IDs remain attached provenance, not the task identity.

### 5.3 Explicit model switch

A developer should eventually be able to switch a live canonical session deliberately:

```text
/model openai/gpt-...
/model anthropic/claude-...
/model local/qwen-...
```

Before the next turn, Winds prepares a provider-compatible context view from canonical state plus selected conversational history. The UI should explain whether continuity is:

- native resumed;
- same provider/new model;
- cross-provider reconstructed;
- context-compacted;
- missing provider-private state.

### 5.4 Provider Router — later and policy-bound

After direct selection is proven, Winds may support explicit routing policies such as:

- fastest qualified model;
- cheapest within a quality floor;
- local-only;
- privacy-constrained;
- long-context preferred;
- vision-required;
- independent reviewer must use a different provider/runtime;
- fallback only after explicit failure class.

Automatic routing MUST remain observable, explainable, budget-bounded, and reversible. Provider selection must never silently weaken an authority/privacy policy.

### 5.5 Provider credential model

Future design should prefer:

1. provider-native OAuth/subscription paths where legitimately supported;
2. OS credential store or an explicit Winds credential broker with strict local protection;
3. environment/keychain references rather than copying secrets into project files;
4. explicit per-provider scoped configuration;
5. never writing raw secrets into canonical memory, evidence, prompts, logs, Git, or crash reports.

### 5.6 Usage and cost ledger

Every provider/model call should be attributable to:

- Winds workspace/session/task;
- provider/model;
- role;
- input/output/cache usage where reported;
- cost estimate or provider-reported cost where available;
- latency;
- retry/fallback reason;
- context checkpoint used;
- source quality (`PROVIDER_REPORTED`, `WINDS_CALCULATED`, etc.).

A user should be able to answer: “What did this feature cost across all agents and providers?”

---

## 6. Days-long Codex and agent work: Durable Agent Runtime

The requirement “run for days and never forget the context” cannot be solved by an infinitely growing prompt. Model context windows are finite and provider-native sessions may expire, compact, change model, lose process ownership, or become unavailable.

Winds should solve this by making **canonical task continuity durable outside the model**.

### 6.1 Core invariant

Spec 006 already points in the correct direction:

```text
MODEL_CONTEXT_MAY_COMPACT; CANONICAL_WORK_EVIDENCE_TRUTH_MUST_NOT
```

Extend the research direction with:

```text
PROCESS_MAY_DIE; CANONICAL_TASK_MUST_SURVIVE
PROVIDER_SESSION_MAY_EXPIRE; WINDS_SESSION_MUST_SURVIVE
MODEL_MAY_CHANGE; TASK_IDENTITY_MUST_SURVIVE
TRANSCRIPT_MAY_COMPACT; ACTIVE_CONSTRAINTS_MUST_SURVIVE
BACKGROUND_WORK_MUST_CHECKPOINT_BEFORE_EXTERNAL_WAIT
NO_RETRY_WITHOUT_IDEMPOTENCY_OR_EXPLICIT_POLICY
```

### 6.2 Future durable owner

A post-Spec-006 formalization should evaluate a local durable owner, likely a `windsd`-class process or equivalent architecture, responsible for lifecycle state rather than UI process lifetime.

Potential responsibilities:

- session ownership;
- task queue;
- event journal;
- agent process supervision;
- checkpoint scheduling;
- wake timers/events;
- human-attention requests;
- provider call lifecycle;
- bounded retries;
- cancellation;
- leases/heartbeats;
- crash recovery;
- reattachment from terminal/desktop/mobile clients;
- resource/budget enforcement;
- durable browser/service references where applicable.

This requires a formal spec. Existing Spec 003 intentionally does not provide cross-restart live-session ownership.

### 6.3 Agent states for long work

Do not model a multi-day agent as only `RUNNING` or `DONE`.

Candidate states:

```text
QUEUED
STARTING
ACTIVE
WAITING_TOOL
WAITING_PROVIDER
WAITING_HUMAN
WAITING_TIME
CHECKPOINTING
PAUSED_BUDGET
PAUSED_POLICY
RECOVERING
OWNERSHIP_LOST
FAILED
CANCELLED
COMPLETED_AGENT_REPORTED
VERIFICATION_PENDING
VERIFIED
ACCEPTED_HUMAN
```

### 6.4 Durable checkpoint

A checkpoint should reference structured state such as:

```yaml
checkpoint:
  workspace_id: ...
  task_id: ...
  session_id: ...
  role: builder
  objective: ...
  active_constraints: [...]
  accepted_decisions: [...]
  completed_steps: [...]
  current_step: ...
  next_actions: [...]
  open_questions: [...]
  relevant_files: [...]
  exact_git_state: ...
  runtime_identity: ...
  provider_model: ...
  native_session_ref: ...
  provider_continuity: resumed|reconstructed|unavailable
  authority: ...
  budget: ...
  evidence_refs: [...]
  browser_reality_refs: [...]
  pending_external_wait: ...
```

This is canonical structured state, not a prompt dump.

### 6.5 Wake and continuation

A days-long session should be able to pause and later resume because of:

- timer/schedule;
- user response;
- GitHub event;
- CI completion;
- file/repository change;
- service health change;
- provider availability;
- browser/manual approval;
- budget reset;
- explicit user resume.

Each wake-up reconstructs the model-facing context from canonical state and current reality. It must not pretend hidden provider memory was preserved if it was not.

### 6.6 Context health UI

The user should see context health rather than discover memory loss after failure:

```text
Context health
Canonical task state        HEALTHY
Evidence bindings           HEALTHY
Native Codex session        RESUMED
Conversation context        COMPACTED x3
Imported transcript         PARTIAL
Provider-private memory     UNAVAILABLE
Last checkpoint             2m ago
Unsaved canonical facts     0
```

---

## 7. Memory architecture: never forget the work, not every token

Winds should use multiple memory classes with explicit provenance.

### 7.1 Canonical Work Memory

Structured durable truth:

- objective;
- acceptance criteria;
- constraints;
- decisions;
- dependencies;
- current step;
- blockers;
- relevant artifacts;
- exact candidate identity;
- required verification/review gates;
- human approvals/decisions.

### 7.2 Evidence Memory

Immutable/content-addressed references to what Winds actually observed.

### 7.3 Conversation Memory

Provider/runtime transcripts, summaries, tool messages, branch history. Useful but not canonical by implication.

### 7.4 Semantic Retrieval Memory

Indexed repository/session knowledge used to retrieve relevant context. Retrieval relevance is not authority.

### 7.5 Episodic Agent Memory

Potential learned patterns such as “this repository's test command” or “this user prefers X,” but only after explicit provenance and conflict rules are defined. Learned memory must not silently override protected policy, repository governance, or evidence.

### 7.6 Memory reconciliation

Every compaction should preserve a machine-checkable protected set:

```text
PROTECTED_FROM_LOSS
- current objective
- active constraints
- human decisions
- authority ceilings
- exact candidate refs
- unresolved blockers
- verification requirements
- security/privacy boundaries
```

If a generated summary omits or conflicts with this protected set, canonical state wins and the summary is rejected or repaired.

### 7.7 Session tree

Pi's session-tree and branch-summary ideas are useful. Winds should support:

- continue;
- fork;
- clone;
- navigate prior checkpoints;
- branch summaries;
- model/provider changes in history;
- exact lineage between canonical checkpoints.

The key Winds addition is that the tree is linked to **task/evidence/reality identity**, not just conversation messages.

---

## 8. Agent teams without swarm chaos

Winds should support many agents, but team topology must remain explicit.

### 8.1 Roles are first-class

Candidate roles:

- Planner;
- Builder;
- Reviewer;
- Security Reviewer;
- Researcher;
- Test/QA Agent;
- Browser/Visual Reviewer;
- Operations/Deployment Agent;
- Specialist/Consultant.

Role does not imply model or runtime.

### 8.2 Delegation graph

```text
Human
  ↓ ceiling
Planner
  ├── Builder A
  ├── Builder B
  ├── Researcher
  └── Reviewer
```

Every edge has:

- bounded task;
- context capsule;
- authority ceiling;
- resource roots;
- model/provider budget;
- time budget;
- allowed tools;
- required return schema;
- evidence expectations.

### 8.3 Agent Inbox

A critical UX primitive should be an **Agent Inbox** aggregating only things that need human attention:

- permission request;
- clarification;
- plan review;
- budget increase;
- merge/landing decision;
- browser irreversible action;
- conflicting agent conclusions;
- failed verification;
- blocked durable task.

This prevents developers from watching eight live transcripts.

---

## 9. The future Winds workbench

Winds should remain usable as a CLI/TUI, but the long-term daily product may require a richer native workbench.

### 9.1 Surfaces

Candidate surfaces:

```text
Winds CLI        automation / scripting / power-user commands
Winds TUI        SSH/headless/full-keyboard agent management
Winds Desktop    rich terminal + browser + diff + reality graph
Winds Mobile     approvals/attention/review/status only, later
```

All surfaces should talk to the same canonical local task/session/evidence model when a durable owner is present.

### 9.2 Main desktop layout candidate

```text
┌──────────────────────────────────────────────────────────────┐
│ Workspace | Reality | Branch | Model | Context | Cost | Sync│
├──────────────┬───────────────────────────────────────────────┤
│ Sessions     │ Universal interaction stream                  │
│              │                                               │
│ ● Planner    │  command block                                │
│ ◐ Builder A  │  agent plan                                   │
│ ! Builder B  │  tool activity                                │
│ ✓ Reviewer   │  diff                                         │
│              │  browser annotation                           │
│ Agents Inbox │  verification card                            │
├──────────────┼───────────────────────────────────────────────┤
│ Files/Code   │ Terminal / Browser / Diff / Evidence / Graph │
└──────────────┴───────────────────────────────────────────────┘
```

The interface should be reducible to a minimal terminal when desired.

### 9.3 Universal block types

Possible independently designed typed blocks:

- ShellCommand;
- ShellOutput;
- AgentPrompt;
- AgentReasoningSummary;
- AgentToolAction;
- AgentQuestion;
- Plan;
- Todo;
- FileDiff;
- ReviewComment;
- BrowserPage;
- BrowserAnnotation;
- Screenshot;
- TestResult;
- EvidenceCard;
- HumanDecision;
- ProviderSwitch;
- Checkpoint;
- Delegation;
- CostUsage;
- Warning/PolicyDecision.

Blocks should remain searchable, linkable, collapsible, and source-labelled.

---

## 10. Developer-experience standard: zero friction

The product should be judged by whether an expert developer voluntarily keeps it open all day.

### 10.1 First-run target

```text
install Winds
open existing repository
shell works immediately
existing shell config mostly behaves predictably
Git identity is visible
Codex/Claude/Pi discovery is visible but not trusted automatically
agent session can be created intentionally
no mandatory cloud account for local terminal basics
```

### 10.2 Daily actions should be one gesture away

- create terminal;
- create agent session;
- continue prior task;
- switch model;
- attach file/symbol;
- open browser;
- inspect diff;
- compare reality branches;
- run verification;
- ask independent reviewer;
- approve/deny;
- stop all agents safely.

### 10.3 Performance budgets should become formal

Future specs should define measurable budgets for:

- cold start;
- new tab/session latency;
- keystroke-to-render latency;
- scroll performance;
- shell startup overhead;
- idle CPU/memory;
- large-output rendering;
- session search latency;
- context reconstruction latency;
- browser attachment latency.

“Feels fast” should become testable where possible.

---

## 11. Proposed post-Spec-006 architecture

This is a research decomposition, not implementation authorization.

```text
                         ┌─────────────────────┐
                         │ Winds Workbench UI  │
                         └──────────┬──────────┘
                                    │
                  ┌─────────────────▼──────────────────┐
                  │ Local Winds Control API / Protocol │
                  └─────────────────┬──────────────────┘
                                    │
       ┌────────────────────────────▼─────────────────────────────┐
       │                Durable Winds Runtime Owner               │
       │ sessions | tasks | events | leases | checkpoints | wait │
       └──────┬──────────┬────────────┬────────────┬─────────────┘
              │          │            │            │
       ┌──────▼───┐ ┌────▼─────┐ ┌────▼────┐ ┌────▼────────┐
       │ Terminal │ │ Agent Hub │ │ Browser │ │ Verification │
       │ Runtime  │ │/Providers │ │ Runtime │ │ + Evidence   │
       └──────┬───┘ └────┬─────┘ └────┬────┘ └────┬────────┘
              │          │            │            │
              └──────────┴────────────┴────────────┘
                                    │
                         ┌──────────▼──────────┐
                         │ Canonical State DB │
                         │ + content evidence │
                         └─────────────────────┘
```

### 11.1 Core modules to formalize separately

- **Terminal Runtime** — PTY/ConPTY/WSL lifecycle and rich terminal events.
- **Agent Hub** — runtime adapters, provider mesh, model identity, delegation.
- **Context Engine** — canonical memory, retrieval, compaction safety, capsules.
- **Durable Scheduler** — checkpoints, waits, retries, long-running lifecycle.
- **Browser Runtime** — profile/context isolation, browser actions, annotations.
- **Reality Graph** — exact links among candidate, services, browser, agents, evidence.
- **Verification Engine** — existing Winds authority preserved and expanded carefully.
- **Workbench UI** — beautiful interaction layer without owning canonical truth.

The UI must never become the only persistence authority.

---

## 12. Candidate formalization sequence after current authorized work

Do not start these merely because this document exists. The sequence is a research recommendation for future Spec Kit formalization after current canonical dependencies are satisfied.

### N0 — Product kernel and UI architecture decision

Decide:

- retained CLI/TUI role;
- desktop framework/rendering approach;
- clean-room Warp product-study boundary;
- local IPC design;
- daemon/durable-owner necessity;
- data model migrations;
- performance budgets.

### N1 — Beautiful terminal workbench foundation

Prove:

- rich terminal renderer;
- typed command blocks;
- universal input;
- tabs/panes;
- workspace/session browser;
- history/search;
- code/diff preview;
- current Winds verification remains intact.

### N2 — Provider Mesh

Prove:

- multiple provider configurations;
- explicit model switching within one canonical Winds session;
- auth/secret boundaries;
- token/cost ledger;
- provider capability truth;
- cross-provider context reconstruction with provenance.

### N3 — Durable Agent Runtime

Prove:

- durable task/session owner;
- process lifecycle and restart recovery;
- checkpoints;
- waits/wake events;
- cancellation;
- budgets/leases;
- days-long fixture/soak without canonical context loss.

### N4 — Canonical Memory and Session Tree

Prove:

- protected canonical facts survive repeated compaction;
- continue/fork/clone;
- branch summaries;
- semantic retrieval with provenance;
- model/provider replacement without task identity loss.

### N5 — Verified Browser

Prove:

- browser contexts linked to canonical Winds realities;
- annotations;
- screenshots/DOM references;
- localhost iteration;
- host/profile/credential safety;
- browser evidence authority taxonomy.

### N6 — Reality Branches and comparative verification

Prove:

- two isolated candidate realities;
- independent service/browser contexts;
- deterministic comparison report;
- code/test/browser evidence comparison;
- no automatic winner;
- explicit human selection.

### N7 — Agent team control plane

Scale from one bounded delegation to:

- multiple workers;
- specialized reviewers;
- Agent Inbox;
- explicit topology;
- per-child provider/model/budget/authority;
- parallel realities.

### N8 — Remote/mobile continuity, only after local truth is strong

Potential later scope:

- secure relay;
- remote status;
- approvals;
- review;
- start/pause/resume;
- no secret/project-state centralization by default.

---

## 13. “Better than Warp” must be measurable, not rhetorical

Winds should not claim superiority without evidence. Candidate product scorecard:

### Terminal quality

- startup latency;
- input latency;
- large-output smoothness;
- shell compatibility;
- completion quality;
- cross-platform stability.

### Agent quality

- successful task completion under defined evals;
- human intervention rate;
- context-loss incidents;
- resume/reconstruction correctness;
- average time to verified candidate;
- provider/model switching success.

### Verification quality

- stale evidence detection;
- exact-candidate binding correctness;
- false-positive acceptance rate target: zero for defined gates;
- independent review applicability;
- reality comparison reproducibility.

### Durable work

- multi-day soak duration;
- crash/restart recovery rate;
- checkpoint data loss target;
- duplicate external side effects under retry target: zero for protected fixture operations;
- canonical protected-fact retention through repeated compactions.

### Browser

- isolated-reality leakage target: zero in defined fixtures;
- annotation-to-code traceability;
- bounded browser regression reproducibility;
- credential leakage target: zero in defined storage/log/prompt fixtures.

### UX

- time from install to first useful shell;
- time to create/continue an agent session;
- number of gestures to switch model;
- number of gestures to inspect/approve agent work;
- usability testing versus baseline terminals/ADEs.

---

## 14. Decisions this plan intentionally does not make yet

The following require dedicated research/specification before adoption:

- whether the desktop renderer uses an existing UI toolkit or a Winds-specific stack;
- whether terminal rendering is fully GPU-native from the first rich UI release;
- exact browser engine/runtime choice;
- exact durable execution framework versus Winds-owned implementation;
- exact vector/semantic memory engine;
- exact credential-store abstraction;
- exact provider SDK dependencies;
- whether MCP, ACP, vendor-native protocols, or a combination becomes the primary universal tool/runtime bridge;
- cloud sync architecture;
- collaboration/team tenancy;
- pricing or hosted service strategy.

Do not lock these prematurely merely to imitate another product.

---

## 15. Product anti-goals

Winds should reject these failure modes:

```text
PRETTY_TERMINAL_WITH_WEAK_TRUTH
CHAT_APP_WITH_A_SHELL_PANEL
PROVIDER_LOCK_IN
ONE_GIANT_TRANSCRIPT_AS_MEMORY
UNBOUNDED_SWARM
SILENT_MODEL_ROUTING
SILENT_AUTHORITY_ESCALATION
BROWSER_AUTOMATION_WITHOUT_ACCOUNT_SITE_BOUNDARIES
BACKGROUND_AGENTS_WITHOUT_CHECKPOINTS_OR_STOP_CONTROL
CLOUD_REQUIRED_FOR_LOCAL_TERMINAL_BASICS
AGENT_SAYS_DONE_THEREFORE_DONE
AUTO_MERGE_AS_DEFAULT
COPYING_AGPL_IMPLEMENTATION_INTO_MIT_APACHE_CODE
```

---

## 16. North-star user story

A developer opens Winds in the morning.

They see their repository, running services, three agent sessions, yesterday's canonical task checkpoint, and one item in the Agent Inbox. They approve a bounded question, then continue the task without knowing or caring whether the original provider-native session still exists.

They ask Winds to try two implementations. One builder uses Codex, another Claude. A researcher uses a third API. All remain attached to the same canonical task but operate in separate Reality Branches. Each branch starts its own local service and Browser Twin. The agents work for hours, checkpointing structured state. The developer closes the UI and later reopens it; canonical work, evidence, budgets, and blockers are still there.

When model context gets large, conversational context compacts, but Winds retains protected objective/constraints/decisions/evidence exactly. A builder can switch provider without pretending hidden provider memory transferred.

Later the developer opens the comparison view. Winds shows exact code differences, deterministic test evidence, browser screenshots and annotations, accessibility/performance evidence where configured, provider/model provenance, and independent review. No agent chooses the winner. The human selects one reality and explicitly authorizes landing through the repository's accepted workflow.

That is the product direction:

> **A beautiful terminal where agents can work for days across models and browsers without losing canonical truth, and where every important result remains tied to exact, reviewable reality.**

---

## 17. Research references

Primary source bindings and claim limits are recorded separately in:

- `docs/research/012-agentic-era-terminal-source-register.md`

Key research families:

- Warp product UX, block model, universal agent support, shared context, and open-source architecture;
- Pi provider/model configuration, sessions, session trees, and compaction behavior;
- OpenAI Codex multi-agent, long-running, browser/computer-use, remote-continuity, and safety product behavior;
- current Winds README, Spec 006 invariants, and existing agentic-development research plans.

Every external product/protocol must be revalidated at implementation time because these systems evolve rapidly.
