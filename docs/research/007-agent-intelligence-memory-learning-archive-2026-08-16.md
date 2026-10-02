# Winds Research Archive — Agent Fleet, Verified Memory, Learning, and Predictive Execution

**Status:** Research archive only. Non-normative. This document does **not** amend the Winds Constitution, Spec 003, or authorize Agent Fleet / memory / learning / public-protocol product code.

**Archive date:** 2026-08-16

**Canonical Winds base at archive creation:** `a0582d5b48358bf44ff9dba5248ba1aaf0813416`

**Concurrent active implementation:** Spec 003 / T049, PR #20, expected exact head `7f98048b985daf2bddbe95d2d0e5d387af1c1565` at archive creation.

**Existing donor dossier:** `docs/research/006-agent-fleet-donor-audit.md`

---

## 1. Why this archive exists

This file preserves the strategic research and architectural synthesis developed while Winds is still implementing the Workspace Execution Spine. It exists so that high-value future ideas are not lost, while keeping them out of the active implementation scope.

The founder direction is that Winds should ultimately become the developer control plane above many interchangeable coding agents and AI CLIs, not merely another coding agent. A user should be able to operate many agents — potentially 20+ concurrent workers — while Winds remains the authority for execution evidence, verification, recovery, and human-controlled promotion.

The long-term distinction is:

> **Run every coding agent. Trust only the evidence. Learn only from verified experience.**

Winds must never become the bottleneck merely because the user wants to run many agents. Orchestration should scale through local-first execution, self-hostability, heterogeneous external agents/providers, explicit backpressure, and evidence-aware scheduling rather than arbitrary product-imposed quotas.

This archive adds a second long-term thesis to the existing verification thesis:

> Winds should not merely run agents. It should observe verified execution, remember what actually worked, predict the consequences of future actions, and improve how it deploys an interchangeable agent fleet.

That future thesis is **not** authorization to implement it during Spec 003.

---

## 2. Current scope firewall — do not contaminate Spec 003

The research in this document is deliberately parked behind a hard scope boundary.

At archive creation:

- canonical `main` is `a0582d5b48358bf44ff9dba5248ba1aaf0813416`;
- T048 is closed/canonical;
- T049 is the only active implementation slice;
- PR #20 implements only native-Windows WSL distribution discovery;
- T050+ PTY/runtime work is not yet authorized by this research archive;
- Agent Fleet is not part of T049;
- ACP/MCP/A2A/public runtime protocols are not part of T049;
- persistent agent memory is not part of T049;
- learned routing is not part of T049;
- self-improvement is not part of T049;
- a world model / predictive execution model is not part of T049;
- model training is not part of T049;
- no new daemon, plugin system, service mesh, or generic runtime abstraction is authorized here.

The Winds Constitution and active Spec Kit documents remain authoritative over this archive.

---

## 3. Evidence ontology remains the foundation

Future intelligence must preserve the existing Winds truth boundary instead of weakening it.

At minimum, future data should distinguish:

- `AGENT_REPORTED` — what an agent/model claims happened;
- `SHELL_REPORTED` — shell/terminal telemetry that is useful but may be spoofable or incomplete;
- `WINDS_OBSERVED` — facts independently observed by Winds from trusted execution/Git/check surfaces;
- `HUMAN_DECIDED` — explicit user/founder decisions such as which candidate is selected or promoted.

A future memory or learning subsystem must never silently upgrade an agent claim into verification truth.

### 3.1 Verified Experience invariant

Proposed future invariant:

> **An agent experience is not promotable to reusable memory, skill, routing evidence, or training data until its claimed outcome is independently evidenced.**

A possible future record shape:

```text
VerifiedExperience
├── task identity
├── repository/base/candidate identity
├── agent + model + toolchain identity
├── action / trajectory summary
├── observed commands/processes
├── observed file/Git changes
├── deterministic check results
├── independent review evidence
├── outcome classification
├── cost/time/resource telemetry
├── failure/repair chain
├── provenance
└── confidence / completeness flags
```

This is a research direction, not a committed schema.

---

## 4. Long-term Winds architecture thesis

The strongest synthesis from the current donor/research sweep is an eight-plane architecture.

```text
Human / Project Intent
        │
        ▼
1. Fleet Intelligence / Planning
   decomposition • routing • delegation • concurrency
        │
        ▼
2. Agent Fleet
   Codex • Claude Code • Goose • Gemini CLI • Kilo • OpenCode • Pi • others
        │
        ▼
3. Execution Plane
   workspace • worktree • shell • PTY • process • WSL/native domains
        │
        ▼
4. Evidence Plane
   exact Git identity • command/process facts • artifacts • tests • telemetry
        │
        ▼
5. Verification Plane
   deterministic gates • independent review • safety/recovery checks
        │
        ▼
6. Verified Experience Memory
   durable successful/failed trajectories with provenance
        │
        ▼
7. Predictive Execution / World Model
   state + candidate action -> expected next state / cost / risk / surprise
        │
        ▼
8. Learning / Improvement
   routing • skills • policies • delegation topology • verifier selection
        └──────────────────────────────────────────────────────────────↺

Cross-cutting: Governance / Security / Human Authority
```

The system should improve the **fleet policy**, not grant uncontrolled self-modification to an individual agent.

---

## 5. Agent Fleet direction

The desired future Winds experience is one control plane for heterogeneous AI CLIs and coding agents.

A user should eventually be able to:

- discover locally available agents;
- launch multiple agents in isolated workspaces/worktrees;
- delegate self-contained tasks;
- resume sessions where the underlying agent supports it;
- run agents sequentially or in parallel;
- restrict tools/permissions per worker;
- inspect terminal/process/worktree state;
- compare candidate outputs;
- run independent verification/review;
- preserve failures for recovery rather than silently deleting them;
- account for time/tokens/cost/resources with source provenance;
- switch models/providers without rewriting the orchestration system;
- route work based on evidence rather than vendor lock-in;
- avoid an artificial Winds-imposed daily/PR/file/review quota.

### 5.1 Transport priority

Preserve the transport priority already recorded in the donor dossier:

1. standardized structured protocol such as ACP when stable and appropriate;
2. vendor-native machine API / app server;
3. machine-readable CLI mode;
4. compatibility relay around a CLI;
5. PTY/TUI interpretation only as the last compatibility path.

Do not invent a new public Winds protocol when an existing standard is sufficient.

---

## 6. Newly emphasized OSS donor/reference set

The full earlier donor audit remains in `006-agent-fleet-donor-audit.md`. This section preserves the strategic additions and emphasis from subsequent research.

### 6.1 Goose — Tier-1 donor/reference candidate

Source:

- https://goose-docs.ai
- https://github.com/aaif-goose/goose

Observed strategic value:

- open-source agent with Desktop, CLI, and API surfaces;
- Rust implementation;
- multiple model/provider support;
- MCP extensions;
- ACP interoperability surfaces;
- recipes and subrecipes;
- internal and external subagents;
- sequential and parallel delegation;
- multi-model planning/execution;
- permissions and security-oriented modes;
- session/context handling;
- terminal integration patterns.

High-value future audit categories:

1. subagent/delegation engine;
2. ACP client/server/provider behavior;
3. provider routing and multi-model configuration;
4. recipes/subrecipes as reusable procedural knowledge;
5. extension/tool permission model;
6. security/adversarial review patterns;
7. session/context lifecycle;
8. terminal UX;
9. custom distribution patterns.

Important Winds divergence:

- Goose terminal integration may modify/evaluate shell initialization snippets; Spec 003 requires Winds not to modify persistent shell dotfiles/profiles for instrumentation.
- shell telemetry must retain source attribution and not be promoted to authoritative process evidence merely because a shell reports it.
- MCP/ACP/public interoperability belongs in a future explicit spec, not the current execution-spine slice.

License snapshot observed during research: Apache-2.0. Re-verify exact commit/path/license before copying any source.

### 6.2 Kilo Code — strong future provider/agent UX donor

Source:

- https://github.com/Kilo-Org/kilocode

Strategic value:

- broad model/provider selection;
- custom modes/agents;
- model switching during work;
- coding-agent CLI workflows;
- tool control and provider abstraction.

Decision: deep-study and possible permissive donor after exact provenance audit.

License snapshot observed during research: MIT. Re-verify before copy.

### 6.3 Zed — architecture/UX/performance reference

Source:

- https://github.com/zed-industries/zed

Strategic value:

- high-performance editor/workspace architecture;
- integrated terminal/editor UX patterns;
- collaboration architecture;
- Rust performance patterns;
- agent interaction surfaces.

Decision: primarily **STUDY**. The repository has GPL-licensed source with separately marked Apache-licensed components; no GPL source should enter Winds merely because it is technically attractive. Exact component-level license/provenance analysis is mandatory before reuse.

### 6.4 Kiro — product/spec UX reference, not presumed code donor

Source:

- https://github.com/kirodotdev/Kiro

Strategic value:

- Specs;
- Hooks;
- Steering;
- agentic chat;
- MCP-oriented product UX;
- spec-driven development concepts.

Decision: product/UX reference until a specific reusable source path and license are established. Do not infer broad code-reuse rights from repository visibility.

### 6.5 delegate-skills — heterogeneous CLI compatibility reference

Source:

- https://github.com/amElnagdy/delegate-skills

Already recorded in the donor dossier. It remains highly relevant for:

- delegation briefs;
- relay result normalization;
- touched-file discovery;
- exit/signal/OOM-aware reporting;
- session/resume compatibility;
- Windows process-tree behavior;
- keeping implementers separate from landing/promotion authority.

Treat it as compatibility/fallback donor material, not justification to prefer bespoke relays over structured native transports.

### 6.6 Pi and other agent CLIs

Source previously identified:

- https://github.com/earendil-works/pi

Preserve for future comparative audit of agent-loop minimalism, CLI ergonomics, session behavior, and tool interfaces. No copy decision is made by this archive.

### 6.7 Existing high-value interoperability donors

Do not lose the earlier Tier-0/Tier-1 donor set in `006-agent-fleet-donor-audit.md`, including:

- `agentclientprotocol/rust-sdk`;
- `agentclientprotocol/registry`;
- `agentclientprotocol/codex-acp`;
- `agentclientprotocol/claude-agent-acp`;
- `openclaw/acpx`;
- `coder/agentapi`;
- Worktrunk;
- agent-worktree;
- Gas Town;
- Beads;
- ccusage;
- CodexBar;
- models.dev;
- Atuin;
- VS Code shell integration;
- Microsoft Terminal / ConPTY;
- WezTerm / portable-pty;
- deterministic verifier/reviewer references documented there.

The existing dossier remains the canonical donor inventory until deliberately superseded.

---

## 7. Research paper register

### 7.1 Agent Memory Survey — arXiv:2602.06052

Source:

- https://arxiv.org/abs/2602.06052

Classification for Winds: **Tier S research foundation** for future memory architecture.

Key ideas preserved from the review:

- agent memory is broader than a vector database;
- useful distinctions include working, episodic, semantic, procedural, and sensory memory;
- memory may be user-centric or agent-centric, internal or external;
- procedural memory is especially relevant to reusable skills/routines/workflows;
- multi-agent systems need private and shared memory with explicit read/write policies;
- deduplication, contradiction handling, consistency, access control, and leakage boundaries matter;
- coding agents benefit from remembering failure/repair trajectories, not merely code snippets;
- memory quality requires evaluation, including false-memory and integrity concerns;
- memory poisoning and privacy leakage are first-class security risks.

Winds implication:

Do not build "chat history with embeddings" and call it memory. Future memory should be evidence-aware and provenance-preserving.

Potential future decomposition:

```text
Working memory     -> active task/session state
Episodic memory    -> past execution trajectories
Semantic memory    -> stable project/tool/environment facts
Procedural memory  -> verified reusable skills/workflows
Shared fleet memory-> selectively visible verified experience
```

No implementation is authorized here.

### 7.2 Foundation Agents — arXiv:2504.01990

Source:

- https://arxiv.org/pdf/2504.01990

Classification for Winds: **Tier S architecture survey/reference**.

Preserved implications:

- multi-agent systems benefit from specialization and division of labor;
- communication topology matters and need not be all-to-all;
- static and dynamic communication patterns have different cost/coordination tradeoffs;
- partial knowledge sharing and asynchronous communication can reduce overhead;
- agent ecosystems remain fragmented, increasing the value of interoperability and explicit interfaces.

Winds implication:

A future 20+ agent fleet must not devolve into every agent talking to every other agent. Winds should explicitly schedule role assignment, visibility, handoff, and verification topology.

Example future roles, not committed product types:

```text
planner -> implementer(s) -> verifier(s) -> reviewer -> human decision
```

The exact topology should remain task-dependent rather than hard-coded mythology.

### 7.3 Inherent — Training AI Scientists to Replicate Research / Faraday — arXiv:2608.13331v1

Sources:

- https://arxiv.org/pdf/2608.13331
- https://www.alphaxiv.org/pdf/2608.13331v1
- https://inherentlabs.ai/research/training-to-replicate

Paper title: **Training AI Scientists to Replicate Research**.

Paper date: 14 August 2026 in the supplied PDF; arXiv identifier `2608.13331v1` dated 13 August 2026.

Rights note from the supplied paper: © 2026 Inherent Laboratories, all rights reserved. Treat the paper as a research reference, **not** a source-code donor.

Classification for Winds: **Tier S strategic research**.

#### Core result

The paper introduces Replica, a scalable research-paper replication task space, and post-trains a 27B "AI Scientist" agent called Faraday. The central architectural idea is especially relevant to Winds: Faraday acts as a higher-level policy that uses a frontier coding agent as a tool.

Preserved paper details:

- Replica contains 310 figure-replication tasks from 100 ML and AI-for-science papers;
- 242 tasks are used for training and 68 for held-out AI-for-science testing;
- each task hides a result figure and asks the agent to reproduce the underlying experiment under time/compute constraints;
- the environment is containerized and includes research libraries, internet access, a GPU allocation, and a coding-agent binary;
- a task-specific rubric is auto-generated;
- a coding agent judge can inspect the workspace, code, Git history, rollout trace, generated plot, and original gold plot;
- the judge can re-execute code and returns dimension scores;
- multiple independent judge samples reduce reward variance;
- turn-level credit weights redistribute learning credit across a long trajectory;
- Faraday is post-trained from a 27B base model using a modified GRPO recipe;
- Faraday can invoke Codex through a wrapper, resume previous sessions, reset context, or run multiple coding agents in parallel;
- the harness is intentionally simple rather than a large hand-coded research workflow;
- the paper reports Faraday above Claude/Codex baselines on many train and held-out replication tasks under its evaluation protocol;
- qualitative analysis emphasizes implementing the actual mechanism instead of hard-coding an expected result or taking a shortcut.

#### Five rubric dimensions preserved from the paper

1. visual fidelity to the target result;
2. whether the replication supports the scientific claim;
3. whether the implementation actually tests the described mechanism;
4. effective use of the available compute budget;
5. scientific integrity / avoidance of cheating.

#### Why this matters to Winds

The important abstraction is not "build Faraday". It is:

```text
higher-level policy
        ↓
frontier coding agent(s) as tools
```

A future Winds analogue could be:

```text
Winds fleet policy
        ↓
Codex / Claude Code / Goose / Gemini CLI / Kilo / OpenCode / Pi / ...
        ↓
authoritative execution evidence
        ↓
independent verification
```

Faraday strengthens the case that a smaller/specialized control policy can extract better task behavior from a more powerful coding model by deciding how and when to use it.

Winds divergence is critical:

- Winds should not learn from a single judge score as if that were product truth;
- Winds verification already has stronger exact-snapshot evidence requirements;
- a future training/routing loop should preserve deterministic evidence and human authority;
- rubric judges are valuable signals but remain reviewer evidence, not unquestionable truth;
- reward hacking and judge bias must be treated as explicit risks.

#### Research opportunity

Faraday suggests a future route where Winds learns a **fleet policy** rather than training a full coding model:

- when to delegate;
- which agent/model to choose;
- when to resume versus reset context;
- when to run agents in parallel;
- when to escalate to a stronger model;
- when a result needs independent verification;
- when to stop spending compute;
- how to allocate limited resources among workers.

This must remain a future research program until the execution/evidence spine is mature enough to supply trustworthy training/evaluation data.

### 7.4 LeWorldModel / LeWM — arXiv:2603.19312

Source:

- https://arxiv.org/pdf/2603.19312

Associated project observed during research:

- https://github.com/lucas-maes/le-wm

Classification for Winds: **PARK now; Tier A future predictive-execution research**.

The paper is primarily about a learned visual world model, not coding-agent orchestration. The direct model is therefore not a current Winds dependency.

The important abstraction is:

```text
current state + action -> predicted next state
```

#### Proposed Winds research translation

This is an inference for Winds, not a claim made by the LeWM paper:

```text
repository/execution state + candidate agent action
        ↓
predicted next repository/execution state
```

Possible future inputs:

- repo tree / exact Git state;
- current diff;
- dependency graph;
- compiler/test diagnostics;
- active workers;
- execution ledger;
- verified past experience;
- task intent and constraints.

Possible future candidate actions:

- delegate to a particular agent;
- ask for a patch;
- run a verifier;
- switch model/provider;
- reset context;
- spawn an independent reviewer;
- run a test subset;
- escalate to a larger model.

Possible predicted outputs:

- probability of verified success;
- likely tests/files affected;
- regression risk;
- expected time/tokens/cost;
- likelihood of needing another repair cycle;
- expected information gain;
- uncertainty.

#### Execution surprise

A particularly useful future concept is **prediction error / surprise**.

Examples:

```text
Claim: "docs-only change"
Observed: binary behavior changed
=> high surprise -> increase verification
```

```text
Expected: package A test repair
Observed: unrelated packages fail
=> high surprise -> trigger broader diagnosis/review
```

```text
Expected: narrow 3-file repair
Observed: 87 files touched
=> anomalous trajectory -> stop/escalate
```

A future policy could use surprise as one input to increase verification depth, never as a replacement for deterministic evidence.

#### Latent planning idea

Instead of blindly launching every available agent, a cheap predictive model could eventually estimate which agent/topology is worth the compute before expensive execution.

Example research target:

```text
Codex + state X                 -> 0.81 expected verified success
Claude + state X                -> 0.74
Codex + Claude in parallel      -> 0.87, 2.4x cost
Codex + independent reviewer    -> 0.84 verified-success probability
```

This is explicitly future research. Do not add PyTorch/JEPAs/CEM/world-model dependencies to Winds core now.

### 7.5 mHC — arXiv:2512.24880

Source:

- https://arxiv.org/pdf/2512.24880

Classification for Winds: **PARK**.

This work is primarily model-architecture/training research rather than an immediate control-plane primitive. Preserve it in the research register in case Winds later trains a native/specialized routing or coding model, but it is not a justification for adding model-training infrastructure to current Winds.

Potential related implementation reference observed during research:

- https://github.com/deepseek-ai/TileKernels

Any future use requires a fresh license/provenance/fit audit.

### 7.6 TimesFM

Previously supplied Google TimesFM research/code was reviewed as low-value for current Spec 003.

Classification: **PARK**.

Potential future use exists only if Winds accumulates meaningful longitudinal time-series telemetry where forecasting/anomaly detection materially improves capacity planning, cost forecasting, queue behavior, or operational anomaly detection.

Do not add a time-series model merely because one exists.

---

## 8. Memory architecture research direction

Future Winds memory should not be a single undifferentiated database of conversation chunks.

A research decomposition:

### 8.1 Working memory

Short-lived information needed for the current execution:

- task brief;
- active workspace identity;
- current agent sessions;
- recent tool results;
- temporary hypotheses;
- unresolved findings.

### 8.2 Episodic execution memory

Durable trajectories:

- task -> attempts -> failures -> repairs -> verification outcome;
- exact agent/model/toolchain versions;
- exact repository snapshots;
- timing/resource usage;
- independent review findings;
- recovery actions.

### 8.3 Semantic project memory

Stable facts supported by evidence:

- project languages/toolchain;
- authoritative build/test commands;
- repository policies;
- platform constraints;
- recurring architecture invariants;
- known subsystem ownership.

### 8.4 Procedural memory / verified skills

Reusable workflows promoted only after evidence:

```text
Symptom
  -> diagnostic sequence
  -> repair pattern
  -> verification sequence
  -> known limits
```

A skill should include scope, prerequisites, evidence provenance, version applicability, and invalidation conditions.

### 8.5 Shared fleet memory

Multi-agent memory requires explicit access controls:

- what is private to one worker;
- what is shared with a role/team;
- what is globally reusable;
- what contains secrets/sensitive source;
- what is untrusted agent prose versus verified evidence.

The safest default is not "every agent sees everything."

---

## 9. Learning and self-improvement direction

The future target is **system-level improvement**, not uncontrolled recursive mutation by one worker.

Winds should eventually be able to answer with evidence:

- which agent is strongest for Rust debugging in this repository class;
- which agent is strongest for tests, refactors, docs, security review, or research;
- which model/provider combination has the best verified success/cost tradeoff;
- when parallel execution is worth the extra compute;
- when a fresh context performs better than session continuation;
- which reviewer catches a given defect family most reliably;
- which repair patterns repeatedly pass independent verification;
- which prompts/skills produce large diffs without benefit;
- which workers tend to claim success before deterministic checks agree;
- when the scheduler should stop, escalate, or ask the human.

### 9.1 Training-data firewall

Never train or optimize a future routing policy directly on raw chat success labels.

Prefer signals such as:

- exact deterministic gate outcomes;
- exact candidate identity;
- independently observed command/process results;
- independent reviewer findings;
- human accept/reject decisions;
- explicit cost/time/resource observations;
- evidence completeness.

Agent self-reports remain features/telemetry at most, never the sole target truth.

### 9.2 Risks to design against

- memory poisoning;
- stale knowledge promoted as current truth;
- prompt-injection persistence into future tasks;
- reward hacking;
- judge/reviewer bias becoming self-reinforcing;
- routing policies that overfit one repository/model generation;
- false causality from correlated successful runs;
- privacy leakage between workspaces/users/projects;
- uncontrolled skill mutation;
- optimization for cheap success claims rather than verified outcomes;
- feedback loops where the same model authors, judges, and trains on itself without independent evidence.

---

## 10. Future fleet scheduler research

A future scheduler should avoid naive all-to-all communication and indiscriminate fan-out.

Potential responsibilities:

- task decomposition;
- dependency graph;
- worker role selection;
- model/provider selection;
- concurrency budget;
- worktree allocation;
- information visibility;
- retry/escalation policy;
- deadline/resource budget;
- cancellation/backpressure;
- verifier assignment;
- independent reviewer assignment;
- evidence aggregation;
- human decision gate.

Potential topology examples:

```text
Planner
  ├── Implementer A
  ├── Implementer B
  └── Researcher
        ↓
Deterministic verifier
        ↓
Independent reviewer
        ↓
Human selection
```

or

```text
Implementer
   ↓ failure evidence
Diagnoser
   ↓ proposed repair
Implementer
   ↓
Verifier
```

Topology should be selected from evidence/task needs rather than fixed role theater.

---

## 11. Future evaluation program

Winds should evaluate fleet intelligence against **verified outcomes**, not attractive demos.

Potential metric families:

### Outcome

- verified task success rate;
- first-pass verified success;
- repair cycles to verified success;
- regression rate;
- human rejection rate after apparent success.

### Efficiency

- wall-clock time;
- agent/model calls;
- token use where reliably reported;
- compute/resource usage;
- monetary cost with pricing-version provenance;
- worktree/storage churn.

### Evidence quality

- evidence completeness;
- stale/mismatched snapshot rate;
- agent-claim vs observed-fact disagreement rate;
- false-positive verification rate;
- false-negative blocking rate.

### Memory

- useful retrieval rate;
- false-memory rate;
- stale-memory rate;
- contradiction rate;
- procedure reuse success;
- selective forgetting/invalidation correctness.

### Routing

- regret versus best available agent after the fact;
- cost-adjusted verified success;
- escalation precision;
- unnecessary parallelism rate;
- predicted vs actual success calibration.

### Surprise / prediction

- next-state prediction error;
- anomaly precision/recall;
- calibration of high-surprise escalations;
- information gain from chosen actions.

---

## 12. Proposed future spec sequence — research only

This is a parking-order suggestion, **not authorization**.

1. **Agent Fleet Core**
   - heterogeneous worker registry/session/lifecycle;
   - no learned routing required.

2. **Structured Agent Interoperability**
   - ACP/vendor-native transports;
   - capability negotiation;
   - relay fallback only where needed.

3. **Fleet Scheduling and Delegation**
   - dependency-aware tasks;
   - concurrency/backpressure;
   - explicit visibility and role topology.

4. **Verified Experience Memory**
   - working/episodic/semantic/procedural memory;
   - provenance and invalidation;
   - poisoning/privacy controls.

5. **Evidence-Aware Routing**
   - heuristic/statistical routing first;
   - transparent features and offline evaluation.

6. **Predictive Execution Model**
   - state/action/outcome modeling;
   - surprise/anomaly escalation;
   - learned planning only if data quality is sufficient.

7. **Fleet Policy Learning / Self-Improvement**
   - optimize delegation from verified experience;
   - maintain human authority and reproducible evaluation.

8. **Research/Science Mode**
   - optional specialized higher-level policy inspired by Faraday-style coding-agent-as-tool research;
   - only after the core execution/evidence platform is mature.

Each must begin with Constitution -> Spec -> Plan -> Tasks and pass the normal Winds review gates.

---

## 13. Donor/provenance rules remain mandatory

No research enthusiasm overrides donor controls.

Before copying source:

1. pin exact repository and commit;
2. pin exact paths/lines;
3. inspect path-level license headers and repository license;
4. inspect generated/vendor/transitive provenance;
5. record why integration or smaller reimplementation is insufficient;
6. preserve notices/attribution;
7. create Winds-semantic tests;
8. run correctness/safety review;
9. run Ponytail simplicity review;
10. obtain independent review.

Research papers are not source-code licenses. "All rights reserved" papers are references only unless separately published code has explicit reuse terms.

---

## 14. Source register preserved in this archive

### Research

- https://arxiv.org/abs/2602.06052
- https://arxiv.org/pdf/2504.01990
- https://arxiv.org/pdf/2608.13331
- https://www.alphaxiv.org/pdf/2608.13331v1
- https://inherentlabs.ai/research/training-to-replicate
- https://arxiv.org/pdf/2603.19312
- https://arxiv.org/pdf/2512.24880

### Agent/control-plane references

- https://goose-docs.ai
- https://github.com/aaif-goose/goose
- https://github.com/Kilo-Org/kilocode
- https://github.com/zed-industries/zed
- https://github.com/kirodotdev/Kiro
- https://github.com/amElnagdy/delegate-skills
- https://github.com/earendil-works/pi

### Additional implementation references mentioned during research

- https://github.com/lucas-maes/le-wm
- https://github.com/deepseek-ai/TileKernels

### Existing donor archive

See `docs/research/006-agent-fleet-donor-audit.md` for the larger ACP, worktree, terminal, usage/cost, and verification donor set and its pinned snapshots/decisions.

---

## 15. Durable decisions from this research pass

1. **Do not replace Winds' verification identity with generic orchestration.** Fleet orchestration must sit on top of evidence, not instead of it.
2. **Do not make agent prose authoritative.** Future memory and learning inherit the evidence ontology.
3. **Verified Experience is the central bridge between verification and learning.**
4. **Prefer heterogeneous interchangeable agents over a locked single-model architecture.**
5. **Avoid all-to-all multi-agent chatter.** Explicit topology, visibility, and backpressure should be scheduler concerns.
6. **Prefer structured agent transports over PTY/TUI scraping.**
7. **Preserve failures and exact trajectories because failed attempts are valuable learning evidence.**
8. **Use learned routing only after enough trustworthy execution data exists.** Start with explicit policies and measurable baselines.
9. **Prediction/surprise is a promising future verification amplifier, not a replacement for deterministic checks.**
10. **Faraday-style higher-level intelligence above coding agents is strategically aligned with Winds, but should be learned/evaluated from Winds' stronger evidence model.**
11. **World-model research is parked until the execution ledger contains enough high-quality trajectories.**
12. **No research item in this archive changes T049/T050 scope.**

---

## 16. Short product thesis to preserve

> **Winds is the evidence-first control plane for the agentic developer. It can eventually operate an interchangeable fleet of coding agents, preserve verified experience, and improve delegation over time — without turning agent claims into truth or making Winds itself the bottleneck.**

This thesis is preserved for future Spec Kit work. It is not an implementation claim about the current repository.
