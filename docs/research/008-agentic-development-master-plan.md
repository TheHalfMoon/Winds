# Winds Agentic Development Master Plan

**Status:** Pre-spec product/architecture plan. Research and future direction only.

**Research freeze:** 2026-08-20

**Canonical Winds base inspected:** `048cf59e8bdbe3a757b4d3ead214099ce18369bd`

**Authority boundary:** This document does **not** amend the Constitution, Spec 003, or the active T068 reconciliation. It does **not** authorize T069 or any Agentic/Agent Fleet product code. Any implementation that introduces a daemon/session owner, public/local IPC, ACP/MCP runtime, generic agent adapters, remote execution, plugin/runtime system, broader sandbox, or other post-Spec-003 runtime capability must first pass the repository's required Constitution -> Spec -> Plan -> Tasks process and any necessary explicit constitutional/spec amendment.

**Research basis:** `006-agent-fleet-donor-audit.md` plus `007-agentic-cli-final-landscape.md`.

---

## 1. North Star

Winds should become the environment a developer opens **before** running Claude Code, Codex, Pi, Goose, Droid, Junie, Gemini CLI, Copilot CLI, OpenCode, Cursor, Kimi, Qoder, Cline, Mistral Vibe, or whatever coding agent appears next.

### Product definition

> **Winds is the verified local runtime for agentic software development.**
>
> It organizes human terminals and AI agents under named workspaces and connected sessions; preserves canonical task/work/evidence state across sessions and runtimes; lets human-approved planners delegate to heterogeneous agent teams; gates consequential access through external local authority; and independently verifies exact candidate results before human landing decisions.

### Short positioning

> **Any agent. One runtime. Verified work.**

### User promise

> **Run any agent. Build any team. Keep it alive. Gate its authority. Verify its work.**

### Product anti-positioning

Winds is **not**:

- another provider-specific AI chat;
- a model marketplace disguised as a terminal;
- a generic multi-agent launcher;
- a Superset clone;
- a cmux/Herdr/pane-multiplexer clone;
- a Conductor/worktree dashboard clone;
- a transcript-search product;
- a claim that a worktree is a sandbox;
- a magic agent-ranking/winner system;
- a replacement for native agent authentication/configuration;
- a system that trusts agent prose as proof;
- a giant plugin platform before there is product need.

The durable product distinction is not breadth of agent support. It is the ability to preserve **canonical continuity, authority truth, exact candidate identity, independently observed evidence, reviewer independence, and human decision provenance** while the underlying agents change.

---

## 2. The simple user mental model

The visible product model should stay simple even if the runtime becomes sophisticated:

```text
Workspace
  └── Sessions
       ├── Human terminal
       ├── Planner agent
       ├── Builder agent
       ├── Reviewer agent
       ├── Consultant / researcher
       ├── Service / database / container
       └── Remote terminal
```

A normal user should need to remember only:

1. workspace name;
2. session name;
3. what they want to accomplish.

They should not need to remember:

- native agent session IDs;
- opaque internal Winds IDs;
- routine absolute repository paths;
- provider-specific resume syntax;
- provider-specific path flags;
- which agent happens to be the current model/runtime implementation detail.

### Non-negotiable UX rules

```text
WORKSPACE_NAME != WORKSPACE_ID
SESSION_NAME   != SESSION_ID
RENAME_NEVER_BREAKS_IDENTITY
NEW_SESSION != NEW_TASK
NEW_AGENT   != NEW_TASK
PATH_SELECTION_MUST_BE_INTERACTIVE_AND_FUZZY
USER_SHOULD_NOT_NEED_NATIVE_SESSION_IDS
```

---

## 3. Workspace model

A **Workspace** is the durable human container for development activity.

Minimum future identity:

```text
Workspace
- stable Winds workspace ID
- editable display name
- primary root
- optional additional roots
- canonical repository/worktree identity where Git applies
- trust/policy state
- execution authority domain
- created / last-active metadata
- sessions[]
```

Changing the display name must never break session history, Git identity, evidence references, native agent session mappings, worktrees, or task continuity.

### Target launcher UX

```text
Winds

Recent Workspaces
────────────────────────────────────────────
> Winds Core          ~/code/Winds          8 sessions
  ProtocolWISE        ~/code/ProtocolWISE   4 sessions
  Playground          ~/playground          2 sessions

[o] Open folder
[n] New workspace
[/] Search everything
```

`winds open` should support both fuzzy navigation and explicit power-user paths:

```text
winds open .
winds open ..
winds open ~/code
winds open C:\Projects\foo
```

Path-heavy underlying agent flags must not leak into routine UX.

---

## 4. Session model

A **Session** is a named execution/conversation unit inside a Workspace.

Future metadata should be able to represent:

```text
Session
- stable Winds session ID
- editable display name/title
- workspace ID
- optional parent/continuation lineage
- role
- runtime
- selected/requested model
- native runtime session/thread ID when available
- transport
- cwd
- additional roots
- worktree/capsule identity
- canonical task/workstream identity
- context checkpoint
- authority profile
- lifecycle state
- evidence references
- created / last-active metadata
```

### Continue by meaning, not ID

The normal flow should be:

```text
winds continue
winds continue "T068"
```

with fuzzy selection if ambiguous. Native IDs remain diagnostics, not routine UX.

### `continue`, `fork`, `new`

- **continue** — same canonical task/workstream, latest known Winds state;
- **fork** — alternate path retaining explicit lineage;
- **new** — intentionally unrelated unless the user explicitly links it.

Do not collapse these into vendor-specific resume behavior.

---

## 5. Connected Sessions: the core continuity architecture

This remains the primary future differentiator.

Opening a new CLI/agent session must not silently reset the developer's work.

### Continuity invariant

```text
SESSION_CONTINUITY_INVARIANT

Creating, replacing, resuming, forking, attaching, or changing
an agent session MUST NOT silently discard canonical task,
workspace, authority, or evidence state.

Vendor-native conversation state SHOULD be resumed when it is
provably available.

When native continuation is unavailable, Winds MUST provide an
explicit bounded handoff derived from canonical Winds state.

Lossy model-context compaction MUST affect only the model view,
never canonical task/evidence truth.
```

### Five continuation layers

Winds should attempt continuation in this order:

1. **Live process reattach** — original owned PTY/process still exists and can be reattached.
2. **Native runtime resume** — the runtime exposes a proven native session/thread continuation.
3. **Winds context continuation** — a new runtime process receives a bounded canonical task capsule.
4. **Cross-runtime handoff** — the same canonical task continues under another runtime/model.
5. **Fail-closed ambiguity** — if identity cannot be established safely, ask rather than guess.

### Continuity proof vocabulary

Do not use one vague `restored=true` flag.

- `LIVE` — original process/session remains owned.
- `RESUMED` — a new process resumed a proven vendor-native session/thread.
- `RECONSTRUCTED` — Winds restored task/work/context state but could not prove original native conversation continuation.
- `OWNERSHIP_LOST` — previously live ownership can no longer be proven.
- `STOPPED` — known terminated/closed.

### Transfer report

Every cross-session/runtime handoff should be able to explain:

```text
Transferred:
- canonical task objective
- active constraints
- accepted decisions
- exact workspace/candidate identity
- selected files/symbols/context
- required gates
- authority ceiling
- evidence references

Not transferred / unavailable:
- vendor-private hidden state
- native transcript portions not exportable
- unavailable tool state
- unsupported runtime configuration

Reconstructed from:
- Winds canonical work state
- Winds evidence ledger
- selected repository context
- cited imported history, if explicitly used
```

A handoff is not “lossless” merely because a new agent received a summary.

---

## 6. Winds memory: conversation, work, evidence, and imported history are different

Winds must not treat a transcript as the entire memory system.

### 6.1 Conversation memory

What humans/agents said, including native transcripts where available and permitted.

### 6.2 Work memory

Structured current truth needed to continue the task:

- objective;
- acceptance criteria;
- constraints;
- decisions;
- open questions;
- task status/dependencies;
- assigned roles;
- relevant files/symbols;
- expected deliverables;
- current candidate/worktree state.

### 6.3 Evidence memory

What actually happened or was independently observed:

- exact Git base/candidate/tree;
- commands Winds owned/observed;
- deterministic check results;
- artifacts;
- reviewer identity/scope/result;
- authority grants/escalations;
- explicit human decisions.

### 6.4 Imported history

History imported from Claude/Codex/Pi/Goose/etc. — whether through native APIs, files, or a ctx-style local index — is useful retrieval context but remains source-labelled imported history.

It does **not** automatically become canonical Winds task/evidence truth.

### Canonical context capsule

A continuation should be generated from structured state rather than replaying an enormous transcript:

```yaml
workspace:
  id: ws_...
  name: Winds Core

task:
  id: ...
  objective: ...
  state: active

constraints:
  - ...

decisions:
  - ...

workspace_state:
  base: ...
  candidate: ...
  worktree: ...
  dirty_paths: ...

completed:
  - ...

pending:
  - ...

open_questions:
  - ...

evidence:
  focused_tests: ...
  full_tests: ...
  review: ...

authority:
  ...
```

### Compaction and reconciliation rule

Canonical Winds state and imported history must stay on different provenance paths:

```text
WINDS-OBSERVED FACTS + EXPLICIT HUMAN DECISIONS
      ↓
canonical Winds work/evidence state
      ↓
retrieval / bounded context view
      ↓
MODEL CONTEXT

IMPORTED HISTORY / VENDOR TRANSCRIPTS
      ↓
source-labelled retrieval context
      ├──────────────→ MODEL CONTEXT
      └→ explicit reconciliation proposal
              ↓
        validation / human decision
              ↓
        canonical state update
```

Imported history may inform a proposal to update canonical work state, but it must never overwrite canonical work/evidence truth merely because it was retrieved. Compaction may be lossy for conversational detail. It must never rewrite evidence or silently erase active constraints/decisions.

### ctx-informed direction

A future Winds implementation should seriously consider whether local history search should integrate with, study, or reuse a ctx-style approach rather than rebuild every vendor-history parser.

The value Winds must add above raw history search is:

- canonical task identity;
- explicit source/provenance;
- transfer/loss reporting;
- authority state;
- exact candidate/evidence bindings.

---

## 7. Context inheritance policy

Not every child receives all parent context.

### Planner -> Builder

Typically inherit:

- objective;
- accepted plan;
- constraints;
- relevant repository context;
- required evidence/gates;
- authority ceiling.

### Planner -> Consultant / Researcher

Typically inherit:

- precise question;
- required background;
- evidence/report format;
- read-only authority where possible.

### Builder -> Independent Reviewer

Intentionally inherit only what the review needs:

- requirements/acceptance criteria;
- exact base/candidate identity;
- exact diff/artifacts;
- deterministic evidence available to the reviewer;
- review contract.

Exclude by default:

- builder self-assessment;
- builder confidence;
- persuasive implementation rationale not needed for understanding;
- planner preference for one candidate;
- hidden/private reasoning.

This preserves stronger reviewer independence.

---

## 8. Context and navigation UX

Winds should be easier than every underlying CLI for choosing files, folders, symbols, diffs, and prior work.

### `@` picker

Inside a prompt:

```text
Fix the race in @hist
```

should fuzzy-search:

- files;
- folders;
- symbols/classes/functions when intelligence is available;
- recent files;
- changed/staged files;
- tests;
- prior session artifacts;
- cited imported-history results when explicitly requested.

Selected context should become inspectable chips rather than opaque prompt text.

### Context Inspector

`winds context` should show at least:

- workspace/task/session identity;
- continuation lineage and proof level;
- active objective/constraints/decisions;
- active context sources and provenance;
- repository/worktree identity;
- evidence state;
- authority state;
- model-context budget;
- native history availability;
- imported-history sources;
- Winds context checkpoint.

### Context diff

A future:

```text
winds context diff <session-a> <session-b>
```

should explain what was added, omitted, changed, transferred, reconstructed, or still retained outside active model context.

### Code-intelligence hierarchy

Prefer:

1. connected IDE semantic/index intelligence when available;
2. LSP;
3. tree-sitter/symbol map;
4. deterministic filesystem/Git search.

Winds must remain fully usable without an IDE.

---

## 9. Runtime != model

The product must represent agent harness/runtime separately from the underlying model/provider.

```text
Agent Runtime
  └── Model / Provider
```

Never hard-code assumptions such as “Claude is the builder” or “Codex is the planner.”

Roles, runtimes, models, workflows, context, and authority are separate axes.

---

## 10. Universal Agent Runtime and capability truth

### Goal

If an agent CLI can legitimately run on the user's machine, Winds should have a path to supervise/integrate it without forcing the user to abandon the agent's native authentication/subscription/configuration.

### Transport priority

1. **ACP**.
2. **Vendor-native structured API/app server/SDK**.
3. **Machine-readable CLI mode**.
4. **Compatibility relay**.
5. **PTY/TUI observation/automation as a last-resort compatibility path**.

### ACP state to plan around

As of 2026-08-20, ACP v1 has stabilized/completed capabilities relevant to future Winds adapters including:

- session config options;
- session list;
- session info updates;
- session resume;
- session close/delete;
- additional workspace roots via `additionalDirectories`;
- message IDs;
- usage updates;
- model config category;
- request cancellation;
- boolean config options;
- elicitation;
- 1.0 Rust/TypeScript SDK foundations.

However:

- ACP v2 remains Draft;
- Streamable HTTP/WebSocket remote transports remain evolving/Active;
- implementation must still pin an exact protocol/SDK revision.

`additionalDirectories` declares session workspace scope but is **not** a sandbox. Winds authority claims still require real enforcement.

### Capability registry

Normalize capabilities such as:

- launch/readiness;
- executable path/version;
- auth readiness;
- structured transport;
- session list/load/resume/fork/close/delete;
- session title metadata;
- additional roots;
- message IDs;
- usage updates;
- model/mode/reasoning/config selection;
- permission requests/elicitation;
- request cancellation;
- read-only enforcement;
- workspace write support;
- worktree support;
- subagents;
- terminal/tool events;
- usage/cost reporting;
- remote control;
- host-platform support.

### Declared vs observed

```text
CATALOG_DECLARED
VENDOR_DECLARED
WINDS_LOCALLY_OBSERVED
```

Local launch-time truth wins. Catalog presence is not proof of local installation, authentication, version compatibility, or enforceable isolation.

### Enforcement quality

For every safety-relevant capability, record how it is achieved:

- `WINDS_ENFORCED`
- `OS_SANDBOX_ENFORCED`
- `AGENT_NATIVE_ENFORCED`
- `BEST_EFFORT_TRIPWIRE`
- `OBSERVATION_ONLY`
- `UNAVAILABLE`

Never collapse these into `safe=true` or `sandboxed=true`.

---

## 11. Integration sequence

Broad compatibility is the long-term goal. The initial product proof remains deliberately narrow.

### First formal proof after Spec 003 is accepted

The Constitution already identifies Codex and Claude Code as the first authoring-agent targets. Preserve that discipline.

Prove:

1. one structured Codex session;
2. one structured Claude session;
3. session identity/list/resume where supported;
4. exact workspace/candidate binding;
5. approval/event mapping;
6. explicit transferred/not-transferred continuity report;
7. no product-specific credential duplication.

### Second wave

Add runtimes that exercise genuinely different capabilities:

- Pi — session trees/model flexibility/extensions;
- Goose — custom-agent delegation/recipes/MCP;
- Droid — resumable subagents/autonomy separation;
- Junie — session UX/IDE bridge;
- Gemini CLI — trust/sandbox expansion;
- OpenCode — granular permissions;
- Copilot CLI — fleet/session/remote-control patterns.

### Long tail

Cursor, Kimi, Qoder, Cline, Mistral Vibe, Kilo, Aider, Amp, Hermes, and future runtimes should be adapter/capability additions rather than architecture rewrites.

---

## 12. Agent Teams: one planner, heterogeneous workers

The default team UX should let the human communicate primarily with one Planner while workers report to the Planner.

```text
USER
  ↓
PLANNER
  ├── Builder
  ├── Reviewer
  ├── Consultant
  ├── Researcher
  └── Tester
```

### Team proposal gate

Before workers launch, Winds should present the proposed team contract. The contract distinguishes the Planner's own execution authority from the maximum authority it may delegate:

```text
Planner:                    Codex
Planner direct authority:   orchestrate/read
Delegation ceiling:         worktree write/test + read-only research/review

Builder:                    Claude       worktree write/test
Research:                   Gemini       read-only
Reviewer:                   Codex fresh  read-only exact candidate

Parallelism: 3
Max workers: 4
Max delegation depth: 2
Network: ask
Commit: denied
Push: denied
Merge: denied
Budget: ...

[Approve team] [Edit] [Cancel]
```

A read-only Planner may therefore coordinate a write-enabled Builder only because the human-approved delegation ceiling explicitly authorizes that child capability. The Planner does not acquire the Builder's write authority for its own execution.

No worker starts before the approved contract exists unless a standing policy explicitly authorizes that exact class of delegation.

### Dynamic team changes

A Planner may request another specialist but must state:

- role;
- runtime/model;
- task;
- requested child execution authority;
- budget/concurrency impact;
- reason.

The request auto-passes only when it stays within an already approved delegation policy/ceiling; otherwise it escalates to the human. The Planner cannot self-expand that ceiling.

### Delegation limits

Default product policy should be bounded:

- finite workers;
- finite delegation depth;
- bounded concurrency;
- time/token/cost ceilings where measurable;
- no recursive unbounded spawning.

Agent count is not a success metric.

---

## 13. Structured inter-agent work protocol

Winds does not need private chain-of-thought to orchestrate reliably.

Workers should return structured state such as:

```json
{
  "task_id": "...",
  "from": "builder-1",
  "to": "planner",
  "status": "blocked",
  "summary": "...",
  "artifacts": [],
  "questions": [],
  "requested_action": "planner_decision"
}
```

The durable record should preserve:

- assignment;
- status transitions;
- externally stated claims;
- artifacts;
- questions/decisions;
- authority requests;
- execution/evidence references.

Do not store hidden reasoning as a product requirement.

---

## 14. Local Authority Plane

This is a core Winds moat and must exist outside model reasoning.

### Authority invariant

Direct execution authority and delegation authority are distinct. A Planner may have narrow direct authority while being authorized by a human-approved team contract to delegate a different or broader bounded capability to a child.

```text
CHILD_EXECUTION_AUTHORITY
  ⊆ PLANNER_DELEGATION_CEILING
  ⊆ APPROVED_TEAM_AUTHORITY
  ⊆ HUMAN_GRANTED_AUTHORITY

PLANNER_EXECUTION_AUTHORITY
  ⊆ HUMAN_GRANTED_AUTHORITY
```

No Planner can self-expand its delegation ceiling. No worker can grant itself authority. A Planner may delegate a capability it does not directly possess only when that capability is already inside the explicit human-approved delegation/team ceiling. A model statement that an operation is safe has no policy authority.

### Candidate capability vocabulary

The formal spec should consider explicit resources/actions for:

- filesystem read/write;
- shell/process execution;
- Git local mutation;
- Git remote/network operations;
- network destination/port;
- browser/web;
- MCP server/tool;
- skill/hook/plugin invocation;
- secrets/credential handles;
- Docker/container;
- Kubernetes;
- database connection/query classes;
- remote host;
- subagent delegation;
- worktree creation/removal.

### Policy semantics

Prefer deterministic:

```text
deny > ask > allow
```

An explicit deny cannot be overridden by a transient convenience approval.

### Protect the policy plane

Winds-managed authority/trust files must not be writable by agents through the same policy they govern.

### Content-bound approvals

If an approved hook/command/MCP endpoint/skill/plugin materially changes, prior approval must not silently carry forward.

### Complete mediation and truthful downgrade

Every consequential access that Winds claims to govern must pass through a real enforcement point. If a third-party runtime has direct host access that Winds cannot mediate, report that fact and downgrade enforcement quality rather than pretending Winds blocked it.

---

## 15. Workspace Capsules for workers

A worktree is useful isolation for Git edits but is not a security sandbox.

A future worker **Workspace Capsule** should bind:

- base/candidate identity;
- worktree/cwd;
- allowed roots;
- environment allowlist;
- allowed secret handles;
- network policy;
- MCP/tool set;
- port namespace where needed;
- process/time/token/cost limits;
- delegation ceiling;
- required gates.

### Failure/recovery rule

Dirty, failed, live, or ambiguous worker state is retained for inspection. Do not force-clean or force-remove it.

### No automatic winner

Parallel candidates may be compared/reviewed. Human selection remains explicit.

---

## 16. Persistent Session Runtime

Superset, cmux, Herdr, VS Code background sessions, remote-control systems, and modern agent CLIs validate persistent/background execution as important product substrate.

But persistence is no longer sufficient differentiation.

It still requires an explicit future architecture/spec amendment because current Spec 003 intentionally excludes a daemon/public runtime.

### Product requirement

Closing the Winds client UI should not necessarily terminate approved long-running agent/terminal work.

### Separation of concerns

Never conflate:

- live process persistence;
- native agent conversation/session resume;
- workspace reconstruction;
- transcript replay/import;
- canonical task-memory reconstruction.

Each has a different proof level.

### Future session-owner responsibilities

A future long-lived local owner would need to own/version:

- PTYs/process identity;
- attach/detach;
- resize/input/control ownership;
- lifecycle events;
- terminal state replay/scrollback strategy;
- crash/restart semantics;
- authenticated local client control;
- session resource cleanup;
- exact ownership-loss behavior.

Do not expose a broad public API merely because a local owner exists. Start with the narrowest versioned private control surface required by the product.

### Resource-aware hibernation

cmux demonstrates a useful future pattern: an idle restorable agent can be stopped to reclaim resources and later resumed with native session identity.

Winds should only consider analogous behavior when it can prove:

- exact process generation;
- session/workspace identity;
- idle/restorable state source;
- no pending unconfirmed user input;
- native resume capability;
- preserved canonical task/evidence state.

Hibernation must never turn a heuristic “looks idle” into a destructive action without a stronger policy/proof contract.

---

## 17. Human Take Over / Hand Back

Any agent using a real terminal should eventually support clear control handoff:

```text
Agent owns PTY input
     ↓
[Take Over]
     ↓
Human owns PTY input
     ↓
[Hand Back]
     ↓
Agent may continue
```

Control transitions are Winds-observed events and should be recorded in the execution timeline.

---

## 18. Agent state authority

Do not treat every `idle` signal equally.

Preferred source order:

1. structured native/ACP lifecycle event;
2. installed lifecycle hook with explicit source label;
3. Winds-owned process observation;
4. terminal/screen heuristic;
5. `UNKNOWN`.

```text
screen appears idle
```

must never silently become:

```text
task succeeded
```

### Separate state machines

- **process**: running / exited / ownership_lost;
- **agent turn**: working / blocked / idle / settled;
- **task**: queued / running / blocked / candidate_ready / failed;
- **evidence**: unverified / verifying / verified / failed / stale;
- **decision**: awaiting_human / accepted / rejected.

Core invariant:

```text
IDLE != DONE != VERIFIED != ACCEPTED
```

---

## 19. Attention Router / `winds inbox`

The human should manage authority and decisions, not panes.

Competitors already provide notifications, agent-status badges, and task dashboards. Winds must make the inbox **authority/evidence aware**, not merely notification aggregation.

A future inbox should prioritize:

- **P0 Authority** — credential/network/destructive/escalation request;
- **P0 Evidence invalidated** — exact candidate changed after verification/review;
- **P1 Decision** — Planner requires architecture/product choice;
- **P1 Review** — exact candidate ready for fresh human/independent review;
- **P1 Stale review** — candidate changed and old review no longer applies;
- **P2 Blocked** — worker needs input;
- **P2 Recovery** — ownership lost/dirty/ambiguous state requires action;
- **P3 Information** — research/gate/worker completed.

A terminal “finished” notification is low-level input to this router, not automatic task completion.

---

## 20. Evidence Plane remains the moat

The constitutional distinction remains foundational:

```text
AGENT_REPORTED
!=
WINDS_OBSERVED
!=
HUMAN_DECIDED
```

### Exact-candidate acceptance

A team workflow is not accepted because a Planner says “done.”

Required evidence should bind to exact candidate identity, including as applicable:

- exact base SHA;
- exact candidate SHA/tree or formally defined snapshot identity;
- deterministic repo-native gates independently run/observed by Winds;
- exact reviewer scope/candidate identity;
- unresolved-finding reconciliation;
- explicit human landing decision.

### Staleness rule

```text
CANDIDATE_CHANGED
=> PRIOR_CANDIDATE_CHECKS_STALE
=> PRIOR_CANDIDATE_REVIEW_STALE
```

No UI should imply old review/checks still prove the new candidate.

### `winds explain`

A future explain surface should reconstruct observable provenance:

```text
human request
→ canonical task
→ approved plan/team
→ delegations
→ authority grants/escalations
→ session/runtime actions
→ workspace/candidate changes
→ deterministic gates
→ independent reviews
→ human decision
```

This is evidence replay, not hidden chain-of-thought replay.

---

## 21. Security model

### Hard truths

- Worktree != sandbox.
- Agent permission prompt != OS isolation.
- ACP session identity != trusted output.
- `additionalDirectories` != sandbox.
- MCP output != trusted instructions.
- Imported transcript/history != canonical evidence.
- Repository configuration != trusted merely because it is local.
- Screen-state detection != authoritative completion.
- Planner reasoning != permission policy.

### Prompt-injection boundary

Web, MCP, files, terminals, other agents, issue trackers, and databases can contain adversarial instructions. Those inputs may influence model reasoning but cannot bypass external authority checks.

### Least privilege

Agents request capabilities; deterministic policy/human grants decide them.

### Remote hosts

Authority is host-scoped:

```text
permission on laptop
!= permission on workstation
!= permission on production host
```

Never automatically copy local approvals/secrets to another execution domain.

---

## 22. Protocol and extension model

Use existing standards before inventing Winds-specific public protocols.

### Agent interoperability

**ACP first** where supported.

Formal implementation must pin exact protocol/SDK revisions. ACP v1 has a meaningful stabilized surface, but ACP v2 and remote transport work continue to evolve.

### Tool/data interoperability

**MCP** for external tools/data, pinned to an exact implementation-time specification/SDK revision.

MCP is not the source of truth for:

- Winds session identity;
- Winds authority;
- canonical task/work state;
- candidate verification.

### Extension layering

Prefer small composable surfaces:

1. instructions / AGENTS-style context;
2. Skills;
3. Hooks;
4. MCP tools/data;
5. ACP/native agent adapters;
6. compatibility CLI relays.

Do not start with a giant arbitrary-code plugin runtime.

### Third-party extension trust

Discovery/installation does not equal execution authorization.

---

## 23. CLI/TUI surface

Keep the power-user command set small:

```text
winds
winds open
winds continue
winds sessions
winds agents
winds team
winds inbox
winds context
winds explain
winds attach
```

The TUI should make common actions discoverable without requiring command memorization.

### `winds agents`

Target UX:

```text
Agents on this computer

✓ Claude Code       Ready
✓ Codex             Ready
✓ Pi                Ready
✓ Goose             Ready
✓ Droid             Ready
✓ Gemini CLI        Ready
○ Cursor Agent      Login required
○ Aider             Not installed
```

Winds should prefer existing local installations/authentication. Discovery must never auto-install or execute without authorization.

### `winds team`

Create/inspect/continue a human-approved team.

### `winds attach`

Future persistent/remote attachment to an owned session. This command is not authorized by current Spec 003.

---

## 24. Remote execution direction

After local persistence/authority are proven, Winds may extend the same model to:

```text
local
wsl://distribution
ssh://workstation
container://...
```

A remote session remains an **Execution Authority Domain** with its own:

- workspace identity;
- executable/capability observations;
- authority grants;
- secret access;
- lifecycle/ownership;
- evidence.

Remote execution must not create an implicit cloud dependency for local users.

---

## 25. Observability / usage / cost

Winds' internal ledger remains product truth. OpenTelemetry is an export/interoperability surface, not the authority model.

ACP `usage_update` is useful standardized telemetry, but its values remain source-labelled agent/protocol reports unless independently observable.

Future usage/cost facts should retain provenance:

- provider-reported;
- agent/ACP-reported;
- locally parsed;
- Winds-observed wall-clock/process facts;
- derived from pinned pricing;
- unknown.

Never fabricate token counts or historical costs.

---

## 26. Product roadmap after Spec 003

This is sequencing guidance, not an authorized task list.

### Gate 0 — finish the current foundation

- Reconcile and close T068.
- Complete T069 exactly as canonical Spec 003 requires.
- Accept/close Spec 003 with all required deterministic and independent-review evidence.
- Do not mix future Agentic scope into those tasks.

### Phase A — formalize future product semantics

- amend constitutional/product wording where required;
- create formal Spec 006;
- freeze user scenarios for named workspaces/sessions, continuation, runtime discovery, and one controlled delegation;
- define security boundaries and non-goals before implementation.

**Recommended working title:**

> **Spec 006 — Agentic Terminal & Local Delegation Control Plane**

### Phase B — workspace/session identity UX

Prove without multi-agent complexity:

- editable workspace/session names;
- workspace -> sessions history;
- fuzzy session selection;
- fuzzy directory/file selection;
- `NEW_SESSION != NEW_TASK`;
- explicit continue/fork/new semantics.

### Phase C — runtime discovery and structured adapters

- capability registry;
- local executable/version/auth readiness;
- exact ACP/native transport pinning;
- first Codex adapter;
- first Claude adapter;
- source-labelled capability truth.

### Phase D — connected session continuity

- native continuation where provable;
- canonical Winds context capsule;
- cross-runtime handoff;
- explicit transferred/not-transferred report;
- context inspector;
- imported-history provenance;
- independent-review context policy.

### Phase E — single Planner -> Worker delegation

- Planner proposes one worker/task;
- user approves team contract with separate Planner direct authority and delegation ceiling;
- worker reports structured result;
- Planner can continue the same resumable worker session;
- no broad swarm yet.

### Phase F — worker worktree capsule

- exact worktree assignment;
- cwd/root binding;
- failed/dirty preservation;
- no primary-checkout mutation by Winds verification;
- no automatic landing.

### Phase G — Local Authority Broker

- capability/resource schema;
- deny/ask/allow;
- direct-execution, delegation, team, and human ceilings;
- content-bound approvals;
- protected policy plane;
- truthful enforcement-quality reporting;
- adversarial bypass/self-escalation tests.

### Phase H — exact-candidate review and verification

- candidate identity binding;
- deterministic repo-native gates independently run/observed;
- fresh independent reviewer on exact candidate;
- evidence reconciliation;
- stale evidence invalidation;
- human final landing gate.

At this point Winds has the smallest version of the real moat.

### Phase I — persistent session owner

Only after explicit architecture/spec amendment:

- local long-lived owner;
- PTY/process persistence;
- attach/detach;
- `LIVE/RESUMED/RECONSTRUCTED/OWNERSHIP_LOST` proof;
- crash/restart recovery;
- Human Take Over / Hand Back;
- optionally, evidence-backed resource hibernation for provably restorable idle agents.

This phase should occur **before** scaling to large fleets.

### Phase J — heterogeneous teams and attention routing

- multiple workers;
- multiple runtimes/models;
- bounded task graph;
- concurrency/depth/budget controls;
- Planner/Builder/Reviewer/Consultant/Researcher/Tester roles;
- authority/evidence-aware `winds inbox`;
- alternate candidates without automatic winner selection.

### Phase K — context intelligence

- repository/symbol map;
- LSP;
- optional IDE semantic bridges;
- changed/recent/test/symbol pickers;
- context retrieval/budget accounting;
- cross-session context search;
- evaluate integration/study of ctx-style local history indexing before building redundant provider-history parsers.

### Phase L — broader runtime compatibility

Add runtimes by concrete user demand and capability diversity rather than marketing count.

### Phase M — remote domains

- WSL/SSH attach/control with host-specific authority;
- remote session identity/reconnect;
- no implicit credential/permission propagation.

### Phase N — rich terminal/product ecosystem

Only after runtime/evidence semantics are proven:

- richer rendering/UI choices;
- extension marketplace if justified;
- advanced service/database surfaces;
- broader observability/export.

---

## 27. Required research-informed test program

The formal spec should include deterministic/adversarial tests for product claims.

### Continuity

- native resume succeeds and preserves exact mapping;
- native resume unavailable -> explicit `RECONSTRUCTED` handoff;
- cross-runtime handoff preserves canonical objective/constraints/work/evidence without claiming vendor-private state transfer;
- transfer report identifies unavailable/lost state;
- compaction never rewrites canonical evidence;
- rename never breaks identity;
- ambiguous `continue` fails into user selection rather than guessing;
- imported history cannot silently overwrite canonical work/evidence truth.

### Authority

- child execution authority cannot exceed the approved Planner delegation/team/human ceiling;
- Planner direct execution authority may be narrower than its approved delegation ceiling and must not expand merely because it delegates;
- Planner cannot self-expand the delegation ceiling;
- explicit deny cannot be overridden by transient approval;
- policy files cannot be modified by governed agents;
- changed approved content requires reapproval;
- direct/unmediated capability is labelled truthfully;
- prompt-injected tool output cannot directly grant authority;
- ACP `additionalDirectories` cannot bypass root/OS enforcement claims.

### Workspace/worktree

- parallel workers do not silently share an edit worktree when isolation is required;
- dirty/failed/ambiguous state is preserved;
- no force-clean/remove;
- exact candidate remains bound through gates/review.

### Agent lifecycle

- structured events outrank heuristics;
- replacement process cannot satisfy an old wait by identity confusion;
- `idle` never implies `verified`;
- hibernation/restart paths revalidate exact identity before acting;
- UI close / owner crash / machine restart produce truthful states.

### Review independence

- reviewer receives exact candidate/acceptance criteria;
- builder persuasion/confidence excluded under independent policy;
- changed candidate invalidates stale review automatically.

### Cross-platform

- Windows, WSL, Linux, and macOS execution-domain semantics tested where claimed;
- path/root canonicalization fails closed;
- remote authority never inherits silently.

### Security evaluation

Include adversarial tool-data/prompt-injection and permission-boundary cases. These are product-security tests, not claims of solving prompt injection generally.

---

## 28. Measurable outcomes for the formal spec

Future acceptance criteria should include at least:

1. workspace/session rename never breaks identity/history/evidence linkage;
2. one workspace can contain many independently resumable/searchable sessions;
3. a new session can continue canonical task state without the user restating the objective/constraints/evidence;
4. cross-runtime handoff explicitly reports what transferred and what did not;
5. routine continuation does not require a native session ID;
6. routine file/folder selection does not require a full absolute path;
7. `@`/picker selects files/folders and symbols where intelligence exists;
8. one Planner can delegate a bounded task to one approved worker and continue that worker later;
9. a child's execution authority cannot exceed the approved delegation/team/human ceilings, while the Planner's own direct execution authority remains independently bounded;
10. every safety-relevant capability reports actual enforcement quality;
11. imported/vendor history retains provenance and cannot become canonical evidence by implication;
12. Winds independently runs/observes required gates against the exact candidate;
13. a fresh independent reviewer can be bound to the exact candidate with an independence-preserving context policy;
14. stale checks/reviews invalidate automatically when candidate identity changes;
15. failed/dirty/ambiguous workspaces remain recoverable;
16. heuristic status cannot close task/evidence gates;
17. a future persistent owner distinguishes live ownership from native resume and reconstruction;
18. `winds inbox` can distinguish authority/evidence/decision urgency from ordinary notifications.

Performance/latency targets should be benchmarked against current leading products during formal specification rather than invented here.

---

## 29. Explicit non-goals for the first formal Agentic slice

Do **not** attempt all of the following at once:

- every coding agent;
- a cloud scheduler;
- an agent marketplace;
- a full Docker/Kubernetes sandbox platform;
- custom model serving;
- a proprietary public agent protocol;
- automatic merge/push/PR creation without explicit future policy;
- automatic winner scoring;
- unlimited recursive teams;
- a giant plugin runtime;
- browser-cookie harvesting;
- automatic trust of project hooks/MCP/skills;
- a new GPU terminal renderer before runtime value is proven;
- SQL/DB product surfaces in the same first slice;
- rebuilding every third-party agent-history parser before testing a ctx-style donor/integration path.

The first walking skeleton must prove the differentiated loop, not breadth.

---

## 30. The first walking skeleton that matters

After all required governance gates permit implementation:

```text
Human opens named Workspace
  ↓
continues/creates named Planner Session
  ↓
Winds binds canonical task/work/evidence state
  ↓
Planner proposes one Builder + direct/delegated authority + budget
  ↓
Human approves
  ↓
Builder runs in isolated worktree/capsule
  ↓
Builder reports to Planner
  ↓
Planner sends follow-up to same resumable Builder session
  ↓
Winds reports transferred/not-transferred context truth
  ↓
Exact candidate is bound
  ↓
Fresh independent Reviewer receives review-safe context
  ↓
Winds independently runs exact-candidate deterministic gates
  ↓
Externally visible findings/evidence are reconciled
  ↓
Human inspects and makes final landing decision
```

Initial proof should use Codex/Claude structured integrations because the existing Constitution already names them as the first authoring-agent targets.

A second proof should swap one role to Pi/Goose/Droid or another runtime **without changing the Winds workspace/session/task/authority/evidence mental model**.

---

## 31. Final product principles

Freeze these as the design direction until stronger evidence justifies change:

```text
RUN ANYTHING THAT CAN LEGITIMATELY RUN LOCALLY.

RUNTIME != MODEL.

WORKSPACE HAS MANY NAMED SESSIONS.

NEW_SESSION != NEW_TASK.

NEW_AGENT != NEW_TASK.

CONTINUITY IS A WINDS RESPONSIBILITY, NOT A VENDOR ACCIDENT.

NATIVE RESUME != CANONICAL TASK CONTINUITY.

IMPORTED HISTORY != CANONICAL EVIDENCE.

MODEL CONTEXT MAY COMPACT; CANONICAL WORK/EVIDENCE TRUTH MUST NOT.

CHILD EXECUTION AUTHORITY CAN NEVER EXCEED ITS APPROVED DELEGATION/TEAM/HUMAN CEILING.

PLANNER DIRECT EXECUTION AUTHORITY != PLANNER DELEGATION CEILING.

DISCOVERY != TRUST.

WORKTREE != SANDBOX.

ACP ADDITIONAL ROOTS != SANDBOX.

AGENT CLAIM != WINDS OBSERVATION != HUMAN DECISION.

IDLE != DONE != VERIFIED != ACCEPTED.

USE ACP/NATIVE STRUCTURED CONTROL BEFORE TERMINAL SCRAPING.

THE USER SHOULD NOT HAVE TO MEMORIZE PATHS OR NATIVE SESSION IDS.

FAILED OR AMBIGUOUS STATE IS RETAINED FOR RECOVERY.

CHANGED CANDIDATE INVALIDATES STALE EVIDENCE/REVIEW.

NO AUTOMATIC WINNER.

NO AUTOMATIC AUTHORITY ESCALATION.

VERIFY THE EXACT CANDIDATE.
```

---

## 32. Formal Spec 006 entry criteria

Do not convert this research plan directly into implementation.

Formal specification work begins only when:

1. Spec 003 is canonically accepted/closed, including T068/T069 and required review evidence.
2. Repository truth confirms no conflicting active slice.
3. Constitution/product wording is amended where necessary for post-0.1 agentic runtime goals.
4. Exact ACP protocol/SDK revision is pinned and audited.
5. Any MCP use pins the exact current specification/SDK revision.
6. Persistent-owner/IPC requirements have an explicit threat model and versioned lifecycle design before coding.
7. User scenarios and measurable acceptance outcomes are written before architecture.
8. Authority/trust boundaries and non-goals are explicit before adapters can execute local tools.
9. Deterministic continuity/security/recovery/review-staleness tests are specified before implementation.
10. Implementation begins with the smallest Codex/Claude connected-session + single-delegation walking skeleton, not a broad fleet.

---

## 33. Final verdict

The final market sweep strengthens, rather than weakens, the focused strategy.

Superset demonstrates that heterogeneous agents + worktrees + remote workspaces + automation can be a product. cmux demonstrates agent-oriented terminal attention and resumability. Conductor demonstrates task-centered workspaces and review flows. ctx demonstrates broad local cross-agent history retrieval.

Therefore Winds should **not** try to win by reproducing those features and calling the bundle a moat.

Winds should become the environment that can truthfully answer, across all of them:

- what canonical task continued;
- what context actually transferred;
- what was unavailable/reconstructed;
- what authority each actor really held;
- what exact workspace/candidate was affected;
- what happened versus what an agent merely claimed;
- which checks and review apply to the exact current candidate;
- whether review independence was preserved;
- what the human ultimately decided.

That yields the durable distinction:

> **Winds remembers the work across agents, owns or truthfully describes the execution boundary, constrains delegated authority, routes evidence-aware human attention, and preserves exact proof of what actually happened.**

That is the basis for becoming the first-choice terminal/CLI environment for AI-native software development.
