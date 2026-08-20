# Winds Agentic Development Master Plan

**Status:** Pre-spec product/architecture plan. Research and future direction only.

**Research freeze:** 2026-08-20

**Canonical Winds base inspected:** `048cf59e8bdbe3a757b4d3ead214099ce18369bd`

**Authority boundary:** This document does **not** amend the Constitution, Spec 003, or the active T068 reconciliation. It does **not** authorize T069 or any Agentic/Agent Fleet product code. Any implementation that introduces a daemon/session owner, public/local IPC, ACP/MCP runtime, generic agent adapters, remote execution, plugin/runtime system, or broader sandbox must first pass the repository's required Constitution -> Spec -> Plan -> Tasks process and any necessary explicit constitutional/spec amendment.

**Research basis:** `006-agent-fleet-donor-audit.md` plus `007-agentic-cli-final-landscape.md`.

---

## 1. North Star

Winds should become the default environment a developer opens **before** running Claude Code, Codex, Pi, Goose, Droid, Junie, Gemini CLI, Copilot CLI, OpenCode, Cursor, Kimi, Qoder, Cline, Mistral Vibe, or whatever coding agent appears next.

### Product definition

> **Winds is the verified local runtime for agentic software development.**
>
> It organizes human terminals and AI agents under named workspaces and connected sessions; preserves work across sessions and runtimes; allows human-approved planners to delegate to heterogeneous agent teams; gates consequential access to the local or remote machine; and independently verifies exact candidate results before human landing decisions.

### Short positioning

> **Any agent. One runtime. Verified work.**

### User promise

> **Run any agent. Build any team. Keep it alive. Gate its authority. Verify its work.**

### Product anti-positioning

Winds is **not**:

- another provider-specific AI chat;
- a model marketplace disguised as a terminal;
- a Herdr clone / pane multiplexer;
- a Claude Teams clone;
- a generic plugin platform before there is product need;
- a claim that a worktree is a sandbox;
- a magic agent-ranking/winner system;
- a replacement for native agent authentication/configuration;
- a system that trusts agent prose as proof.

---

## 2. The simple user mental model

The visible product model must remain simple even if the internal runtime is sophisticated:

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

1. **workspace name**;
2. **session name**;
3. **what they want to accomplish**.

They should not need to remember:

- native agent session IDs;
- worktree paths;
- absolute repository paths in routine flows;
- provider-specific resume syntax;
- opaque model IDs when the runtime can present friendly choices;
- which CLI uses `--add-dir`, `--workspace`, `--cwd`, or another equivalent flag.

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
- optional additional roots (later)
- canonical repository/worktree identity where Git applies
- trust/policy state
- created / last-active metadata
- sessions[]
```

### Rename behavior

Changing:

`Winds` -> `Winds Core`

changes only the human display name. It must never break session history, Git identity, evidence references, worktrees, native agent session mapping, or task continuity.

### Home experience

The target interaction is closer to a project launcher than a raw chat prompt:

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

`winds open` should provide a fuzzy directory picker over recent roots and normal filesystem navigation while still supporting power-user forms such as:

```text
winds open .
winds open ..
winds open ~/code
winds open C:\Projects\foo
```

No product flow should force an ordinary user to paste an absolute path merely because an underlying CLI requires one.

---

## 4. Session model

A **Session** is a named execution/conversation unit inside a Workspace.

Future session metadata should be able to represent:

```text
Session
- stable Winds session ID
- editable display name/title
- workspace ID
- optional parent/continuation session
- role
- runtime
- selected/requested model
- native runtime session/thread ID when available
- transport
- cwd
- worktree/capsule identity
- task/workstream identity
- context checkpoint
- authority profile
- lifecycle state
- evidence references
- created / last-active metadata
```

### Session names

Winds should auto-suggest a concise title after the initial task becomes clear, while preserving explicit user names. Names must be searchable, editable, and persistent.

Example:

```text
Sessions — Winds Core
──────────────────────────────────────────────────
● T068 filesystem repair       Claude      12m ago
○ Agent runtime design         Pi          yesterday
○ Herdr research               Codex       yesterday
○ Terminal experiments         zsh         3 days ago
○ ACP prototype                Goose       5 days ago
```

### Continue by meaning, not ID

The normal flow should be:

```text
winds continue
winds continue "T068"
```

with fuzzy selection if ambiguous. Native IDs remain inspectable diagnostics, not routine UX.

---

## 5. Connected Sessions: the core continuity architecture

This is a primary differentiator.

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

1. **Live process reattach** — original PTY/process is still owned and can be reattached.
2. **Native runtime resume** — Claude/Codex/Pi/Goose/etc. exposes a proven native session/thread resume.
3. **Winds context continuation** — new process/session receives a bounded canonical task context capsule.
4. **Cross-runtime handoff** — task continues under another agent runtime/model using the same Winds task/workspace/evidence state.
5. **Fail-closed ambiguity** — if the target task/workspace/session cannot be identified safely, ask the user rather than guessing.

### Lifecycle vocabulary

Do not use a vague single `restored` flag. Distinguish:

- `LIVE` — original process/session remains owned.
- `RESUMED` — a new process resumed a proven vendor-native session/thread.
- `RECONSTRUCTED` — Winds restored workspace/task/context state, but the original native conversation could not be resumed.
- `OWNERSHIP_LOST` — Winds cannot prove ownership/continuity of a stored live process.
- `STOPPED` — known terminated/closed.

### `continue`, `fork`, `new`

These are semantically different:

- **continue** — same task, latest canonical state.
- **fork** — alternate path retaining an explicit lineage to the original.
- **new** — intentionally unrelated session/task unless the user explicitly links it.

Do not collapse them into vendor-specific resume behavior.

---

## 6. Winds memory: conversation, work, and evidence are different things

Winds should never treat a transcript as the whole memory system.

### 6.1 Conversation memory

What users/agents said, including native transcripts where available and permitted.

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

What actually happened or was observed:

- exact Git base/candidate/tree;
- commands Winds owned/observed;
- deterministic check results;
- artifacts;
- reviewer identity/scope/result;
- authority grants/escalations;
- explicit human decisions.

### Canonical context capsule

A cross-session/runtime continuation should be generated from structured state, not by blindly replaying an enormous transcript.

Illustrative shape:

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

### Compaction rule

Full local history may remain available according to retention/privacy policy while model context is compressed/selectively retrieved.

```text
FULL LOCAL RECORD
      ↓
canonical work/evidence state
      ↓
retrieval / bounded context view
      ↓
MODEL CONTEXT
```

Compaction may be lossy for conversational detail. It must never rewrite evidence or silently erase active constraints/decisions.

---

## 7. Context inheritance policy

Not every child should receive all parent context.

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

Intentionally exclude by default:

- builder self-assessment;
- builder confidence;
- persuasive implementation rationale not required for understanding;
- planner preference for one candidate;
- hidden/private reasoning.

This preserves stronger reviewer independence.

---

## 8. Context and file/directory UX

Winds should be easier than every underlying CLI for choosing files, folders, symbols, diffs, and previous work.

### `@` picker

Inside a prompt:

```text
Fix the race in @hist
```

should fuzzy-search:

- files;
- folders;
- symbols/classes/functions when code intelligence is available;
- recent files;
- changed/staged files;
- tests;
- prior session artifacts.

Selected context should become inspectable chips rather than opaque prompt text.

### Context commands / TUI actions

Power-user forms may include:

```text
/add
/drop
/context
/cd
```

but the TUI must make the same actions discoverable without command memorization.

### Context Inspector

`winds context` should show at least:

- workspace/task/session identity;
- inherited-from lineage;
- active objective/constraints/decisions;
- active context sources;
- repository/worktree identity;
- evidence state;
- model-context budget;
- native history availability;
- Winds context checkpoint.

### Context diff

A future `winds context diff <session-a> <session-b>` should explain what was added, changed, omitted from active model context, and still retained in canonical history.

### Code intelligence sources

Prefer an opportunistic hierarchy:

1. connected IDE semantic/index intelligence when available (JetBrains/VS Code-like bridge);
2. LSP;
3. tree-sitter/symbol map;
4. deterministic filesystem/Git search.

Winds must remain fully usable without an IDE.

---

## 9. Runtime != model

The product must represent agent harness/runtime separately from the LLM model/provider.

```text
Agent Runtime
  └── Model / Provider
```

Examples:

- Pi can use different providers/models.
- Goose custom agents can prefer different models.
- Droid can route subagent complexity to different models.
- Junie supports different auth/model routes including BYOK.
- Claude Code and Codex have their own runtime/session semantics.

Never hard-code product assumptions such as "Claude is the builder" or "Codex is the planner" as permanent truth.

---

## 10. Universal Agent Runtime and capability truth

### Goal

If an agent CLI can legitimately run on the user's machine, Winds should have a path to supervise/integrate it without making the user abandon the agent's native authentication/subscription/configuration.

### Transport priority

Preserve and strengthen the existing research order:

1. **ACP**.
2. **Vendor-native structured API/app server/SDK**.
3. **Machine-readable CLI mode**.
4. **Compatibility relay**.
5. **PTY/TUI observation/automation as a last-resort compatibility path**.

### Capability registry

Do not write product logic as endless `if agent == claude` branches. Normalize capabilities such as:

- launch/readiness;
- executable path/version;
- auth readiness;
- structured transport;
- session list/load/resume/fork/close;
- session title metadata;
- model/mode/reasoning selection;
- permission request support;
- read-only enforcement;
- workspace write support;
- worktree support;
- subagents;
- terminal/tool events;
- usage/cost reporting;
- additional roots;
- remote control;
- supported host platforms.

### Declared vs observed

```text
CATALOG_DECLARED
VENDOR_DECLARED
WINDS_LOCALLY_OBSERVED
```

Local launch-time truth wins. A catalog entry is not proof of local installation, authentication, version compatibility, or enforceable isolation.

### Enforcement quality

For every safety-relevant capability, record how it is achieved:

- `WINDS_ENFORCED`
- `OS_SANDBOX_ENFORCED`
- `AGENT_NATIVE_ENFORCED`
- `BEST_EFFORT_TRIPWIRE`
- `OBSERVATION_ONLY`
- `UNAVAILABLE`

Never collapse these into a misleading `safe=true` or `sandboxed=true`.

---

## 11. Integration sequence

The long-term goal is broad compatibility, but initial product proof must remain narrow.

### First proof after Spec 003 is accepted

The Constitution already identifies Codex and Claude Code as the first authoring-agent targets. Keep that discipline for the first formal agentic slice.

Prove:

1. one structured Codex session;
2. one structured Claude session;
3. session identity/list/resume where supported;
4. exact workspace/candidate binding;
5. approval/event mapping;
6. no product-specific credential duplication.

### Second wave

Add runtimes that exercise genuinely different capabilities:

- **Pi** — session trees/model flexibility/extensions;
- **Goose** — custom-agent delegation/recipes/MCP;
- **Droid** — Mission/subagent resume/autonomy separation;
- **Junie** — session UX/IDE bridge/BYOK;
- **Gemini CLI** — trust/sandbox expansion;
- **OpenCode** — granular permissions;
- **Copilot CLI** — fleet/session/remote-control patterns.

### Long tail

Cursor, Kimi, Qoder, Cline, Mistral Vibe, Kilo, Aider, Devin, Amp, Grok, Hermes, and future runtimes should be adapter/capability additions, not architecture rewrites.

---

## 12. Agent Teams: one planner, heterogeneous workers

The default team UX should let the human communicate primarily with one **Planner** while workers report to the Planner.

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

The Planner may itself be Codex, Claude, Pi, Goose, or another capable runtime/model.

### Team proposal gate

Before the Planner launches workers, Winds should present a complete proposed team contract:

```text
Planner:   GPT/Codex          orchestrate/read
Builder:   Claude             worktree write/test
Research:  Gemini             read-only
Reviewer:  Codex fresh        read-only exact candidate

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

No worker starts before the approved contract exists, unless the user has explicitly configured a standing policy that authorizes that exact class of delegation.

### Dynamic team changes

If the Planner discovers that another specialist is needed, it submits a team-change request explaining:

- role;
- runtime/model;
- task;
- authority;
- budget/concurrency impact;
- reason.

The request is automatically allowed only if it remains within an explicitly pre-approved dynamic-team policy. Otherwise it escalates to the human.

### Delegation limits

Default product policy should be bounded:

- finite maximum workers;
- finite delegation depth;
- bounded concurrency;
- time/token/cost ceilings where measurable;
- no recursive unbounded spawning.

---

## 13. Structured inter-agent work protocol

Winds does not need private model chain-of-thought to orchestrate reliably.

Workers should return structured work state such as:

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

- task assignment;
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

```text
CHILD_AUTHORITY
  ⊆ PARENT_AUTHORITY
  ⊆ APPROVED_TEAM_AUTHORITY
  ⊆ HUMAN_GRANTED_AUTHORITY
```

No Planner can grant a capability it does not possess. No worker can grant itself a capability. A model's statement that an operation is safe has no policy authority.

### Candidate capability vocabulary

The formal spec should consider explicit resources/actions for:

- filesystem read/write;
- shell/process execution;
- Git local mutation;
- Git remote network operations;
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

Prefer deterministic `deny / ask / allow` rules with the conservative precedence:

`deny > ask > allow`

where an explicit deny cannot be overridden by an ephemeral convenience approval.

### Protect the policy plane

Winds-managed policy/trust/authority files must not be writable by agents through the same policy they govern. Follow the principle demonstrated by Kiro's hard denial of agent writes to its permission configuration.

### Content-bound approvals

Approval should be tied to the thing actually approved. If a repository hook, command, MCP endpoint, skill, plugin manifest, or similar executable content materially changes, the previous approval should not silently apply.

### Complete mediation

Every consequential access that Winds claims to govern must pass through an enforcement point. If a third-party CLI has direct host access that Winds cannot mediate, Winds must state that limitation explicitly and downgrade the enforcement quality rather than pretending the action was blocked by Winds.

---

## 15. Workspace Capsules for workers

A worktree is necessary for parallel Git edits but is not sufficient as an isolation policy.

A future worker **Workspace Capsule** should bind:

- base/candidate identity;
- worktree/cwd;
- allowed roots;
- environment allowlist;
- allowed secrets handles;
- network policy;
- MCP/tool set;
- port namespace where needed;
- process/time/token/cost limits;
- delegation ceiling;
- required gates.

### Failure/recovery rule

Dirty, failed, live, or ambiguous worker state is retained for inspection. Do not force-clean or force-remove it.

### No automatic winner

Parallel candidates may be compared and reviewed. Winds still does not choose a magic winner. Human selection remains explicit.

---

## 16. Persistent Session Runtime

Market evidence from Herdr, VS Code background sessions, remote control, and modern agent CLIs makes persistent ownership strategically important.

This requires an explicit future architecture/spec amendment because current Spec 003 intentionally excludes a daemon/public runtime.

### Product requirement

Closing the Winds client UI should not necessarily terminate approved long-running agent/terminal work.

### Separation of concerns

Never conflate:

- live process persistence;
- native agent conversation/session resume;
- workspace reconstruction;
- transcript replay;
- task-memory reconstruction.

Each has a different proof level.

### Session owner responsibilities

A future long-lived local owner (name/architecture to be specified later) would need to own/version:

- PTYs/process identity;
- attach/detach;
- resize/input/control ownership;
- lifecycle events;
- terminal state replay/scrollback strategy;
- crash/restart semantics;
- authenticated local client control;
- session resource cleanup;
- exact ownership-loss behavior.

Do not expose a public API merely because a local owner exists. Start with the narrowest versioned private control surface required by the product.

---

## 17. Human Take Over / Hand Back

Any agent using a real terminal should eventually support a clear control handoff:

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

The same PTY/process remains visible. Control transitions are Winds-observed events and should be recorded in the execution timeline.

This is valuable for debuggers, database shells, REPLs, editors, dev servers, SSH sessions, and long-running interactive commands.

---

## 18. Agent state authority

Do not treat every `idle` indicator equally.

Preferred source order:

1. structured native/ACP lifecycle event;
2. installed lifecycle hook with explicit source label;
3. Winds-owned process observation;
4. terminal/screen heuristic;
5. `UNKNOWN`.

Example:

```text
screen appears idle
```

must never silently become:

```text
task succeeded
```

### Separate state machines

Future design should distinguish:

- **process state**: running / exited / ownership_lost;
- **agent-turn state**: working / blocked / idle / settled;
- **task state**: queued / running / blocked / candidate_ready / failed;
- **evidence state**: unverified / verifying / verified / failed;
- **decision state**: awaiting_human / accepted / rejected.

This prevents false-success semantics.

---

## 19. Attention Router / `winds inbox`

The human should manage attention, not panes.

A future inbox should aggregate across workspaces/teams/sessions and prioritize, for example:

- **P0 Authority** — credential/network/destructive access request;
- **P1 Decision** — Planner requires a product/architecture choice;
- **P1 Review** — exact candidate ready for human landing review;
- **P2 Blocked** — worker needs input;
- **P3 Information** — research/gate/worker completed.

`winds inbox` should be a primary navigation primitive once teams can run in parallel/background.

---

## 20. Evidence Plane remains the moat

The current constitutional distinction remains foundational:

```text
AGENT_REPORTED
!=
WINDS_OBSERVED
!=
HUMAN_DECIDED
```

### Exact-candidate acceptance

A future team workflow is not accepted because a Planner says "done".

Required evidence should bind to the exact candidate identity, including when applicable:

- exact base SHA;
- exact candidate SHA/tree or uncommitted snapshot identity if that becomes a formal supported primitive;
- deterministic repo-native gates Winds independently runs/observes;
- exact reviewer scope/candidate identity;
- unresolved finding reconciliation;
- explicit human landing decision.

### `winds explain`

A future explain surface should reconstruct the externally observable chain:

```text
human request
→ approved plan/team
→ delegations
→ authority grants/escalations
→ agent/session actions
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
- ACP authentication != trustworthy agent output.
- MCP tool output != trusted instructions.
- Repository files/configuration != trusted merely because they are local.
- Screen-state detection != authoritative agent completion.
- Planner reasoning != permission policy.

### Prompt injection boundary

Data from web, MCP, files, terminals, other agents, issue trackers, and databases can contain adversarial instructions. Those inputs may influence model reasoning but cannot bypass Winds' external authority checks.

### Least privilege

AuthBench's 2026 results strengthen a design requirement: do not ask the model to generate and enforce its own least-privilege boundary as the sole protection. Let agents request capabilities; let deterministic policy/human grants decide them.

### Remote hosts

Authority is host-scoped:

```text
permission on laptop != permission on workstation != permission on production host
```

Never automatically copy local approvals/secrets to a remote execution domain.

---

## 22. Protocol and extension model

Use existing standards before inventing Winds-specific public protocols.

### Agent interoperability

**ACP first** where supported.

Pin the exact protocol/SDK revision in the formal implementation spec. ACP is evolving quickly (session list/info/close/config are stabilized while additional roots/remote transports continue evolving).

### Tool/data interoperability

**MCP** for external tools/data, pinned to an exact spec revision at implementation time. The 2026-07-28 MCP release is materially different from older stateful assumptions, so Winds persistence must not blindly mirror protocol details.

### Extension layering

Prefer small composable surfaces:

1. instructions / AGENTS-style context;
2. Skills;
3. Hooks;
4. MCP tools/data;
5. ACP/native agent adapters;
6. compatibility CLI relays.

Do not begin with a giant arbitrary-code plugin runtime.

### Third-party extension trust

Future extensions that execute local code require explicit capability/trust treatment. Installation/discovery must never equal execution authorization.

---

## 23. CLI/TUI surface

The power-user command set should stay small and memorable:

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

The TUI should make every common operation discoverable so memorizing these commands is optional.

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

Winds should prefer the user's existing local installation/authentication. Discovery must never auto-install or execute an agent without authorization.

### `winds team`

Create/inspect/continue a human-approved agent team.

### `winds attach`

Future persistent/remote attachment to an owned session/runtime. This command is not authorized by the current spec; it is a future product target only.

---

## 24. Remote execution direction

After local persistence/authority is proven, Winds may extend the same model to:

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

Remote is not a reason to create a cloud dependency for local users. Local-first remains the default architecture/product posture.

---

## 25. Observability / cost

Winds' internal ledger remains product truth. OpenTelemetry is an export/interoperability surface, not the internal authority model.

Future usage/cost facts must retain source/provenance, e.g.:

- provider-reported;
- agent-reported;
- locally parsed;
- Winds-observed wall clock/process facts;
- derived from pinned pricing;
- unknown.

Do not fabricate token counts or historical costs.

When external telemetry is implemented, pin current OpenTelemetry CLI/GenAI semantic-convention revisions rather than cloning evolving attributes into permanent SQLite columns.

---

## 26. Product roadmap after Spec 003

This is sequencing guidance, not an authorized task list.

### Gate 0 — finish the current foundation

- Reconcile and close T068.
- Complete T069 exactly as the canonical Spec 003 requires.
- Accept/close Spec 003 with all required deterministic and independent review evidence.
- Do not mix future Agentic scope into those tasks.

### Phase A — formalize future product semantics

- Amend product/constitutional wording as required for the new post-0.1 direction.
- Create formal Spec 006 (working title below).
- Freeze user scenarios for named workspaces/sessions, continuation, runtime discovery, and one controlled delegation.
- Define exact security/non-goals before implementation.

**Recommended formal working title:**

> **Spec 006 — Agentic Terminal & Local Delegation Control Plane**

The existing `Agent Fleet & Delegation Control Plane` name may remain historical research terminology.

### Phase B — workspace/session identity UX

Prove without multi-agent complexity:

- editable workspace display names;
- editable session titles;
- workspace -> sessions history;
- fuzzy session selection;
- fuzzy directory/file selection;
- `NEW_SESSION != NEW_TASK` data model;
- explicit continue/fork/new semantics.

### Phase C — runtime discovery and structured adapters

- capability registry;
- local executable/version/auth readiness;
- ACP/native transport pinning;
- first Codex adapter;
- first Claude adapter;
- source-labelled capability truth.

### Phase D — connected session continuity

- live/native session continuation where possible;
- canonical Winds context capsule;
- cross-runtime handoff;
- context inspector;
- explicit transferred/not-transferred report;
- independent-review context policy.

### Phase E — single Planner -> Worker delegation

- Planner proposes one worker/task;
- user approves team contract;
- worker reports only structured result to Planner by default;
- Planner may send a follow-up to the same resumable worker session;
- no parallel swarm yet.

### Phase F — worker worktree capsule

- exact worktree assignment;
- cwd/root binding;
- failed/dirty preservation;
- no primary-checkout mutation by Winds verification;
- no automatic landing.

### Phase G — Local Authority Broker

- capability/resource schema;
- `deny/ask/allow` rules;
- parent/team/human ceilings;
- content-bound approvals;
- protected policy plane;
- truthful enforcement-quality reporting;
- adversarial tests for bypass/self-escalation.

### Phase H — exact-candidate review and verification

- candidate identity binding;
- deterministic repo-native gates independently run/observed by Winds;
- fresh independent reviewer on exact candidate;
- evidence reconciliation;
- human final landing gate.

At this point Winds has the smallest version of its real moat.

### Phase I — persistent session owner

Only after explicit architecture/spec amendment:

- local long-lived owner;
- PTY/process persistence;
- attach/detach;
- `LIVE/RESUMED/RECONSTRUCTED/OWNERSHIP_LOST` proof;
- crash/restart recovery;
- Human Take Over / Hand Back.

This phase should occur **before** scaling to very large fleets; a large team that dies with the UI is structurally weak.

### Phase J — heterogeneous teams and attention routing

- multiple workers;
- multiple runtimes/models;
- bounded task graph/dependencies;
- concurrency/depth/budget controls;
- Planner/Builder/Reviewer/Consultant/Researcher/Tester roles;
- `winds inbox`;
- optional alternate candidates, never auto winner.

### Phase K — context intelligence

- repository/symbol map;
- LSP integration;
- optional JetBrains/VS Code semantic bridge;
- changed/recent/test/symbol pickers;
- context retrieval and budget accounting;
- cross-session context search.

### Phase L — broader runtime compatibility

- Pi;
- Goose;
- Droid;
- Junie;
- Gemini;
- OpenCode;
- Copilot CLI;
- Cursor;
- Kimi;
- Qoder;
- Cline;
- Mistral Vibe;
- Kilo;
- Aider;
- long-tail relay adapters.

Order should follow concrete user demand and capability diversity, not marketing count.

### Phase M — remote domains

- WSL/SSH attach/control with host-specific authority;
- remote session identity/reconnect;
- no implicit credential/permission propagation.

### Phase N — rich terminal/product ecosystem

Only after the runtime and evidence model are proven:

- richer terminal UI/rendering choices;
- extension marketplace/discovery if justified;
- advanced service/database surfaces;
- broader observability/export.

---

## 27. Required research-informed test program

The formal spec should include deterministic/adversarial tests for the product claims, not just happy-path UI tests.

### Continuity

- native resume succeeds and preserves exact session identity mapping;
- native resume unavailable -> explicit reconstructed handoff;
- cross-runtime handoff preserves objective/constraints/work/evidence without claiming vendor-private state transfer;
- compaction never rewrites canonical evidence;
- rename never breaks identity;
- ambiguous `continue` fails into user selection rather than guessing.

### Authority

- child cannot exceed parent/team ceiling;
- explicit deny cannot be overridden by ephemeral approval;
- policy files cannot be modified by governed agents;
- changed approved command/hook/plugin content requires reapproval;
- direct/unmediated agent capability is labelled truthfully;
- prompt-injected tool output cannot directly grant authority.

### Workspace/worktree

- parallel workers cannot silently share an edit worktree when isolation is required;
- dirty/failed/ambiguous state is preserved;
- no force-clean/remove;
- exact candidate remains bound through gates/review.

### Agent lifecycle

- structured events outrank screen heuristics;
- replacement process cannot satisfy an old wait by identity confusion;
- `idle` never implies `verified`;
- UI close / owner crash / machine restart produce truthful states.

### Review independence

- reviewer receives exact candidate and acceptance criteria;
- builder persuasion/self-confidence excluded under independent policy;
- changed candidate invalidates stale review automatically.

### Cross-platform

- Windows, WSL, Linux, and macOS execution-domain semantics tested where claimed;
- path canonicalization/root boundaries fail closed;
- remote host authority never inherits silently.

### Security evaluation

Include prompt-injection/adversarial tool-data cases inspired by AgentDojo and permission-boundary cases inspired by AuthBench. These are product security tests, not claims of solving prompt injection generally.

---

## 28. Measurable product outcomes for the formal spec

Future acceptance criteria should include at least these user-visible outcomes:

1. A developer can rename a workspace or session without losing any identity/history/evidence linkage.
2. A workspace can contain many independently resumable/searchable sessions.
3. A new session can continue an existing canonical task without the user restating its objective, active constraints, current workspace state, and known evidence.
4. A cross-runtime handoff explicitly reports what context/state transferred and what could not transfer.
5. Normal continuation does not require a native session ID.
6. Normal file/folder selection does not require typing a full absolute path.
7. `@`/picker navigation can select files/folders and, when intelligence exists, symbols.
8. A Planner can delegate a bounded task to an approved worker and continue that same worker in a later turn.
9. A child cannot acquire authority beyond the approved parent/team/human ceiling.
10. Every safety-relevant capability reports its actual enforcement quality.
11. Winds can independently run/observe required gates against the exact candidate instead of trusting worker prose.
12. A fresh independent reviewer can be bound to the exact candidate with an independence-preserving context policy.
13. Stale checks/reviews are invalidated when the candidate changes.
14. Failed/dirty/ambiguous workspaces remain recoverable instead of being force-cleaned.
15. Heuristic agent status is labelled heuristic and cannot close a task/evidence gate.
16. A future persistent session owner can distinguish live process ownership from native session resume and context reconstruction.

Performance/latency targets should be measured against current leading CLIs during the formal spec rather than invented in this pre-spec document.

---

## 29. Explicit non-goals for the first formal Agentic slice

Do **not** attempt all of the following at once:

- every coding agent on day one;
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
- a new GPU terminal renderer before the agentic runtime proves value;
- SQL/DB product surfaces in the same first Agentic slice.

The first walking skeleton should prove the differentiated loop, not breadth.

---

## 30. The first walking skeleton that matters

After all required governance gates permit implementation, the smallest product loop that proves Winds' future is:

```text
Human opens named Workspace
  ↓
continues/creates named Planner Session
  ↓
Planner proposes one Builder + authority + budget
  ↓
Human approves
  ↓
Builder runs in isolated worktree/capsule
  ↓
Builder reports to Planner
  ↓
Planner sends follow-up to same resumable Builder session
  ↓
Exact candidate is bound
  ↓
Fresh independent Reviewer receives review-safe context
  ↓
Winds independently runs exact-candidate deterministic gates
  ↓
Planner reconciles externally visible findings/evidence
  ↓
Human inspects and makes final landing decision
```

Initial proof should use Codex/Claude structured integrations because the existing Constitution already names them as the first authoring-agent targets. A second proof should demonstrate that one role can be swapped to Pi/Goose/Droid or another runtime without changing the Winds workspace/session/team/evidence mental model.

---

## 31. Final product principles

Freeze these as the design direction until new evidence justifies changing them:

```text
RUN ANYTHING THAT CAN LEGITIMATELY RUN LOCALLY.

RUNTIME != MODEL.

WORKSPACE HAS MANY NAMED SESSIONS.

NEW_SESSION != NEW_TASK.

NEW_AGENT != NEW_TASK.

CONTINUITY IS A WINDS RESPONSIBILITY, NOT A VENDOR ACCIDENT.

MODEL CONTEXT MAY COMPACT; CANONICAL WORK/EVIDENCE TRUTH MUST NOT.

CHILD AUTHORITY CAN NEVER EXCEED ITS APPROVED PARENT/TEAM/HUMAN CEILING.

DISCOVERY != TRUST.

WORKTREE != SANDBOX.

AGENT CLAIM != WINDS OBSERVATION != HUMAN DECISION.

IDLE != DONE != VERIFIED != ACCEPTED.

USE ACP/NATIVE STRUCTURED CONTROL BEFORE TERMINAL SCRAPING.

THE USER SHOULD NOT HAVE TO MEMORIZE PATHS OR NATIVE SESSION IDS.

FAILED OR AMBIGUOUS STATE IS RETAINED FOR RECOVERY.

NO AUTOMATIC WINNER.

NO AUTOMATIC AUTHORITY ESCALATION.

VERIFY THE EXACT CANDIDATE.
```

---

## 32. Formal Spec 006 entry criteria

Do not convert this research document directly into implementation.

Formal specification work begins only when:

1. Spec 003 is canonically accepted/closed, including T068/T069 and required review evidence.
2. Repository truth confirms no conflicting active slice.
3. The Constitution/product wording is amended where necessary for post-0.1 agentic runtime goals.
4. The exact ACP SDK/protocol revision is pinned and audited.
5. Any MCP use pins the then-current exact specification/SDK revision.
6. Persistent-owner/IPC requirements have an explicit threat model and versioned lifecycle design before coding.
7. User scenarios and measurable acceptance outcomes are written before architecture.
8. Authority/trust boundaries and non-goals are explicit before adapters can execute local tools.
9. Deterministic continuity/security/recovery tests are specified before implementation.
10. Implementation begins with the smallest Codex/Claude connected-session + single-delegation walking skeleton, not a broad fleet.

---

## 33. Final verdict

The final market and research sweep supports a focused strategy:

> Winds should not try to be the agent with the smartest model. It should become the **best environment in which every serious coding agent can live, continue, cooperate, be constrained, and have its work independently proven**.

If Winds executes this plan well, the durable product distinction is not "we support Claude + Codex + Pi + Goose + Droid + Junie." Competitors can copy a provider list.

The durable distinction is:

> **Winds remembers the work across agents, owns the execution boundary, routes human attention, constrains delegated authority, and preserves exact evidence of what actually happened.**

That is the basis for becoming the first-choice terminal/CLI environment for developers who build with AI.
