# Tasks: Agentic Terminal & Local Delegation Control Plane

## Canonical Inputs

- Constitution 1.1.0: canonical.
- Spec 006 specification: canonical via PR #67.
- Spec 006 Plan: canonical via PR #68.
- Tasks base: `d37f4f869f76d0715c0a0aee3d818c775771affe`.
- Canonical Plan tree: `d30b617e7cf038c91d5cdbff03f73142ebe07609`.

This file decomposes Spec 006 into independently reviewable slices. It does not itself execute an Agent, send a prompt, call a model/provider API, install a runtime, accept terms, duplicate credentials, enable MCP, add a daemon, or authorize remote execution.

## Global Rules

1. Execute tasks in dependency order. Only the next dependency-satisfied task is authorized.
2. Every implementation task starts from exact then-canonical `main`; live repository truth overrides stale handoffs.
3. Preserve: `RUNTIME != MODEL`, `NEW_SESSION != NEW_TASK`, native resume != canonical continuity, live ownership != durable native ID, imported history != canonical evidence, worktree/ACP root != sandbox, Planner direct authority != delegation ceiling, child authority <= approved child/delegation/team/human ceilings, Agent completion != verification/acceptance, and candidate movement invalidates earlier candidate-bound review/evidence applicability.
4. No automatic winner, merge, rebase, cherry-pick, push, PR creation, force-clean, or autonomous landing.
5. No generic runtime/plugin framework, public IPC, listener, remote execution, MCP runtime, ACP v2, recursive fleet, custom renderer, SQL Studio, LLM Observatory, vector/RAG memory system, or model gateway.
6. No new dependency unless the exact task explicitly authorizes it with fresh exact-version/checksum/license/MSRV/platform/YAGNI review. T070–T086 do not authorize `agent-client-protocol`.
7. Protected Winds policy/trust state must not be ordinary governed worktree content writable by the actor it controls.
8. `WINDS_ENFORCED` is reserved for operations actually mediated by Winds; vendor restrictions are `AGENT_NATIVE_ENFORCED` or weaker unless stronger enforcement is independently proven.
9. Any head movement after evidence/review invalidates merge-ready state until exact-head gates are rerun.
10. **Focused-test registration rule**: whenever a task authorizes a new `src/tNNN_*.rs` focused test module or a new `src/agentic_*.rs` module that must be reachable from the binary crate, that task also authorizes the smallest necessary `src/main.rs` edit solely to add the corresponding module declaration / `#[cfg(test)]` test-module registration. This rule does not authorize unrelated CLI/runtime behavior. The focused acceptance gate MUST prove the named test module was compiled and executed; a test file merely existing under `src/` is insufficient.

## Standard Acceptance Gate

Every implementation task requires on its exact final candidate:

- repository `quality` = SUCCESS;
- focused deterministic tests for the changed surface, with evidence that every task-authorized `src/tNNN_*.rs` module is registered in the crate test graph and actually executed;
- platform evidence only for platforms/domains claimed;
- author correctness/safety/evidence-integrity review;
- Ponytail/YAGNI review;
- independent review on the exact candidate, or a review stack whose final delta reaches it;
- zero unresolved material findings;
- exact changed-file/scope reconciliation;
- no unauthorized dependency/runtime/protocol/authority expansion;
- expected-head guarded merge;
- post-merge canonical main/tree verification before the next task starts.

Historical evidence remains historical.

## Authorization Ladder

Canonical acceptance of this file authorizes **T070 only**. Each later task is dependency-gated.

```text
FIRST_REAL_CODEX_PROMPT_TASK=T079
FIRST_REAL_CLAUDE_PROMPT_TASK=T080
```

Before T079: no real Codex Agent/App Server Agent work and no Codex prompt.
Before T080: no real Claude Agent work and no Claude prompt.
T072 may perform documented non-Agent executable/version discovery because `DISCOVERY != AGENT_EXECUTION`.

---

## Phase 1 — Canonical Identity, Fixture-Only

### [ ] T070 — Workstream and Winds-session persistence substrate

**Purpose**: add structural canonical identity without any Agent process.

**Authorized paths**:
- `migrations/0006_agentic_identity.sql`
- `src/domain.rs`
- `src/store.rs`
- `src/t070_agentic_identity_tests.rs`
- `src/main.rs` only for the minimal module/test registration permitted by Global Rule 10

**Required schema**:
```text
workspaces(workspace_id)
  -> workstreams(workstream_id, workspace_id)
      -> winds_sessions(session_id, workstream_id)
```

`winds_sessions` MUST NOT duplicate an independent `workspace_id`.

**Acceptance**:
- stable opaque IDs independent of display names;
- rename preserves identity/links;
- >=20 sessions across >=5 workstreams in one fixture workspace;
- duplicate/case/Unicode names do not collide with identity;
- cross-workspace session/workstream mismatch structurally impossible;
- invalid/unknown values fail closed;
- T070 focused tests are registered and demonstrably executed;
- no runtime discovery/binding, Agent launch, prompt, delegation, or candidate acceptance behavior.

**Depends on**: canonical Tasks. **Closes to authorize**: T071.

### [ ] T071 — Continue / fork / new-session / new-task semantics

**Authorized paths**:
- `src/agentic_identity.rs`
- `src/cli_workspace.rs` and/or `src/main.rs` only for the smallest proof/module-registration surface
- minimal `src/store.rs` / `src/domain.rs` extensions
- `src/t071_agentic_continuity_tests.rs`

**Acceptance**:
- deterministic create/list/rename/select;
- new session can continue same workstream without creating a task;
- explicit new task creates a distinct workstream even with identical display text;
- fork records origin with a distinct Winds session ID;
- ambiguous continuation returns explicit candidates, not recency guessing;
- focused tests registered/executed;
- no native runtime ID or Agent process/prompt.

**Depends on**: T070 `CLOSED_CANONICAL`. **Closes to authorize**: T072.

---

## Phase 2 — Runtime Discovery / Binding Truth, Fixture-Only

### [ ] T072 — Codex/Claude safe discovery with fake executables

**Authorized paths**:
- `src/agentic_runtime.rs`
- `src/t072_agentic_runtime_discovery_tests.rs`
- `src/main.rs` only for minimal read-only proof/module/test registration

**Acceptance**:
- absent/present/unsupported/replaced-after-discovery fixtures;
- exact executable identity/path + safely observable version;
- declared vs locally observed vs unavailable capability provenance;
- runtime identity remains separate from model/provider identity;
- auth readiness remains unknown when not safely observable;
- focused tests registered/executed;
- no install/update/auth/terms/prompt/model call;
- discovery cannot claim Agent execution occurred;
- launch-significant identity can be revalidated later.

**Depends on**: T071. **Closes to authorize**: T073.

### [ ] T073 — Runtime-session binding persistence and continuity truth

**Authorized paths**:
- `migrations/0007_runtime_session_bindings.sql`
- minimal `src/domain.rs` / `src/store.rs`
- `src/agentic_runtime.rs`
- `src/t073_runtime_binding_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- binding -> `winds_sessions(session_id)` with concrete runtime kind, exact executable/version provenance, optional native ID;
- exact valid mapping may become a future resume candidate;
- stale/missing/ambiguous mapping never yields false `RESUMED`;
- executable/version replacement invalidates mapping applicability;
- persisted PID/native ID alone cannot establish `LIVE` after restart;
- ownership loss explicit; no blind attachment;
- focused tests registered/executed;
- no real Agent process/prompt.

**Depends on**: T072. **Closes to authorize**: T074.

---

## Phase 3 — Canonical Context, Fixture-Only

### [ ] T074 — Deterministic context capsule and transfer report

**Authorized paths**:
- `src/agentic_context.rs`
- minimal existing `src/domain.rs` / `src/store.rs` use only; **no migration is authorized in T074**
- `src/t074_agentic_context_tests.rs`
- `src/main.rs` only under Global Rule 10

T074 must first prove the canonical capsule using existing workstream/session fields plus in-memory typed fixture facts/references. If a new persistent context-fact table is demonstrably required, **STOP T074 and amend/review Tasks before adding a migration**. Do not opportunistically consume migration `0008`.

**Acceptance**:
- deterministic versioned serialization + SHA-256 for identical canonical input/policy;
- stable ordering/normalization/omission rules;
- workspace/workstream/session/objective/constraints/decisions/candidate/evidence references;
- per-fact provenance;
- imported history cannot overwrite `WINDS_OBSERVED` / `HUMAN_DECIDED` facts;
- prompt/tool-like imported text remains data;
- transfer report distinguishes transferred, derived/reconstructed, omitted, unavailable;
- no private hidden-state/reasoning transfer claim;
- compaction does not mutate canonical truth;
- focused tests registered/executed;
- no vector DB/embedding/retrieval/tokenizer/Agent process/prompt.

**Depends on**: T073. **Closes to authorize**: T075.

---

## Phase 4 — Authority / Approval, Fixture-Only

### [ ] T075 — Pure authority/delegation evaluator

**Authorized paths**:
- `src/agentic_authority.rs`
- minimal authority/enforcement types in `src/domain.rs`
- `src/t075_agentic_authority_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- Planner direct authority and delegation ceiling are independent;
- Worker authority <= Worker grant ∩ Planner delegation ceiling ∩ team policy ∩ human ceiling;
- deny precedence fail-closed;
- repo/model/tool/imported text cannot self-escalate;
- evaluator returns decision/reason/human-action and performs no operation;
- enforcement labels cannot overclaim `WINDS_ENFORCED`;
- worktree/root visibility is not authorization/sandboxing;
- one Planner -> one Worker only;
- focused tests registered/executed;
- no real Agent process/prompt.

**Depends on**: T074. **Closes to authorize**: T076.

### [ ] T076 — Content-bound human approval digest / audit substrate

**Authorized paths**:
- `migrations/0008_agentic_delegation_audit.sql`
- `src/agentic_authority.rs`
- minimal `src/store.rs` / `src/domain.rs`
- `src/t076_agentic_approval_tests.rs`
- `src/main.rs` only under Global Rule 10

Approval identity includes, as applicable: workstream/session IDs, requested Worker role/runtime, workspace/worktree/root, capability/resource/path scope, context digest, delegation ceiling/Worker grant, budgets, and exact base/candidate identity.

**Acceptance**:
- stable digest for identical normalized content;
- material change invalidates approval and returns to ask/deny;
- no credential/token/full-environment duplication;
- protected policy/approval state is outside ordinary governed repo content;
- no PKI/signing dependency;
- focused tests registered/executed;
- no real Agent process/prompt.

**Depends on**: T075. **Closes to authorize**: T077.

---

## Phase 5 — Structured Runtime Clients, Fake Before Real

### [ ] T077 — Fake Codex App Server protocol client

**Authorized paths**:
- `src/agentic_codex.rs`
- `src/t077_codex_protocol_tests.rs`
- `src/process_scope.rs` only if a minimal reusable ownership seam is required without weakening existing semantics
- `src/main.rs` only under Global Rule 10

**Mandatory handshake**:
```text
initialize request
  -> successful initialize response
  -> initialized notification
  -> only then thread/start | thread/resume | thread/fork | accepted later methods
```

**Acceptance**:
- pre-handshake methods fail closed;
- malformed/unknown/oversized JSONL bounded and non-authorizing;
- fake server exit during handshake truthful failure;
- notifications remain Agent/runtime evidence, not verification evidence;
- approval request cannot self-authorize;
- native thread ID != Winds session/workstream ID;
- cleanup only for proven owned child;
- focused tests registered/executed;
- no JSON-RPC framework/async dependency/real Codex/prompt.

**Depends on**: T076. **Closes to authorize**: T078.

### [ ] T078 — Fake Claude structured CLI construction/parser

**Authorized paths**:
- `src/agentic_claude.rs`
- `src/t078_claude_structured_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- structured `--print` with `--output-format json|stream-json` as required;
- exact `--resume <session-id>` only for revalidated binding;
- `--continue` never canonical continuation;
- `--dangerously-skip-permissions` impossible in accepted construction;
- malformed/truncated/oversized output fails truthfully;
- restrictions labelled `AGENT_NATIVE_ENFORCED` or weaker unless stronger mediation proven;
- reconstructed new native session = `RECONSTRUCTED`;
- focused tests registered/executed;
- no real Claude/prompt.

**Depends on**: T077. **Closes to authorize**: T079.

---

## Phase 6 — First Real Runtime Proofs

### [ ] T079 — FIRST REAL CODEX PROMPT: bounded App Server proof

**Special authorization**: first task permitted to launch the exact locally discovered/revalidated real Codex App Server for Agent work and send a Codex prompt.

**Safety boundary**:
- explicit disposable/fixture repo or non-mutating proof context;
- one bounded proof session/turn unless exact-version behavior requires a narrowly documented retry;
- no primary-checkout mutation;
- no Winds-driven credential/terms/access escalation;
- decline unexpected write/tool approvals unless the reviewed task contract explicitly allows a harmless fixture operation;
- no PR/push/merge automation.

**Authorized paths**:
- `src/agentic_codex.rs`
- minimal `src/execution.rs` / `src/domain.rs` / `src/store.rs` only if an Agent-execution child record is proven necessary
- a forward-only migration **only if** exact T079 preflight proves that child persistence is necessary; if a migration is needed but not already exact in this Tasks contract, STOP and amend/review Tasks before creating it
- `src/t079_codex_connected_tests.rs`
- `src/main.rs` only for minimal proof/module/test registration

**Acceptance evidence**:
- exact executable/version revalidation;
- complete initialize/initialized handshake;
- exact Winds-session <-> native-thread provenance;
- one bounded structured result;
- truthful authority/enforcement/source labels;
- focused deterministic tests registered/executed;
- proven child cleanup or explicit ownership loss;
- model result is not verified/accepted by implication.

**Depends on**: T078. **Closes to authorize**: T080.

### [ ] T080 — FIRST REAL CLAUDE PROMPT: bounded Planner/read-plan proof

**Special authorization**: first task permitted to launch exact locally discovered/revalidated real Claude Code and send a Claude prompt.

**Safety boundary**:
- one bounded Planner prompt in explicit fixture/disposable or read-only planning context;
- no canonical `--continue`;
- no `--dangerously-skip-permissions`;
- no MCP workaround;
- no primary-checkout mutation;
- no credential/terms automation;
- if stronger safe mediation cannot be proven, remain Planner-only;
- enforcement = `AGENT_NATIVE_ENFORCED` or weaker unless stronger mediation proven.

**Authorized paths**:
- `src/agentic_claude.rs`
- runtime-binding/Agent-execution persistence only if already canonically justified by prior tasks
- `src/t080_claude_planner_tests.rs`
- `src/main.rs` only for minimal proof/module/test registration

**Acceptance evidence**:
- exact executable/version revalidation;
- exact native-session provenance;
- structured Planner result remains `AGENT_REPORTED`;
- exact resume only if binding remains valid;
- reconstructed session labelled `RECONSTRUCTED`;
- focused deterministic tests registered/executed;
- no verification/acceptance claim from Planner output.

**Depends on**: T079. **Closes to authorize**: T081.

---

## Phase 7 — Cross-Runtime Handoff / One Bounded Delegation

### [ ] T081 — Claude-Planner -> Codex-Worker handoff contract

**Authorized paths**:
- `src/agentic_context.rs`
- `src/agentic_authority.rs`
- minimal `src/agentic_claude.rs` / `src/agentic_codex.rs` coordination seam
- optional concrete `src/agentic.rs` only if duplication proves a coordinator is needed; no generic runtime trait/plugin host
- `src/t081_cross_runtime_handoff_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- same canonical workstream survives runtime change;
- transfer report explicit;
- Planner Worker proposal remains `AGENT_REPORTED`;
- human sees exact normalized contract before approval;
- over-ceiling and changed-content approvals fail closed;
- no recursive delegation;
- focused tests registered/executed;
- Planner prose cannot automatically start Worker execution.

**Depends on**: T080. **Closes to authorize**: T082.

### [ ] T082 — One human-approved Codex Worker edit in exact worktree

Real Codex Worker is allowed only under the canonical T079 path after explicit approval of the T081 normalized contract.

**Authorized paths**:
- smallest existing system-Git/worktree extension in `src/git.rs`
- proven `src/agentic_authority.rs` / `src/agentic_codex.rs` / coordinator seams
- already-canonical Agent-execution persistence only
- `src/t082_worker_worktree_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- explicit exact base and Winds-owned Worker worktree;
- Worker bound to exact worktree/common-dir identity;
- worktree never called an OS sandbox;
- operation scope <= approved grant;
- approval requests cannot self-authorize;
- dirty/failed/ambiguous state retained, never force-cleaned;
- completion remains `AGENT_REPORTED` until Git/evidence observation;
- focused tests registered/executed;
- no primary-checkout mutation or automatic merge/push/PR.

**Depends on**: T081. **Closes to authorize**: T083.

---

## Phase 8 — Exact Candidate Review / Verification

### [ ] T083 — Candidate binding, review staleness, `winds verify` bridge

**Authorized paths**:
- existing `src/git.rs`, `src/store.rs`, `src/domain.rs`, `src/main.rs` only where required
- minimal already-proven Agentic candidate/review seam
- `src/t083_agentic_candidate_evidence_tests.rs`

**Acceptance**:
- exact OID/tree is acceptance identity;
- independent-review context contains exact candidate/diff/criteria/canonical constraints/evidence and excludes builder persuasion as authority;
- candidate A evidence becomes `STALE` for B while A remains traceable;
- Agent `done/tests passed` cannot satisfy verification;
- existing `winds verify` supplies deterministic authority; evidence referenced, not copied as Agent truth;
- focused tests registered/executed;
- verify/promote/recover and Spec 003 regressions remain green;
- landing remains human-decided.

**Depends on**: T082. **Closes to authorize**: T084.

---

## Phase 9 — P2 Findability

### [ ] T084 — Deterministic session/path findability

**Authorized paths**:
- existing CLI/workspace selection surfaces where practical
- optional `src/agentic_find.rs` only if a concrete separate seam is proven necessary
- `src/t084_agentic_findability_tests.rs`
- `src/main.rs` only under Global Rule 10

**Acceptance**:
- deterministic partial/fuzzy session selection with explicit disambiguation;
- exact canonical path before context/execution use;
- Unicode/case/similar-name fixtures;
- changed/recent/test/symbol-derived candidates retain provenance;
- unavailable semantic intelligence stays unavailable;
- picker visibility never grants authority;
- focused tests registered/executed;
- no fuzzy/search dependency without a later reviewed amendment.

**Depends on**: T083. **Closes to authorize**: T085.

---

## Phase 10 — Hardening / Acceptance

### [ ] T085 — Cross-platform negative/fault/repetition campaign

Cover deterministically: corrupt/unknown identity/binding values, cross-workspace attempts, changed workspace identity, executable replacement, malformed/oversized runtime output, Codex handshake exits/unknown messages, Claude resume rejection/reuse, imported-history injection, explicit context omissions, over-ceiling/deny/approval-replay cases, runtime success conflicting with Git, candidate movement during review/check, dirty Worker recovery, no blind PID/native attachment, bounded fake runtime repetitions, deterministic capsule hashes, and Spec 003 verification/store regressions.

Any new focused test module added by T085 must obey Global Rule 10 and be proven executed.

Real-runtime repetition, if any, is bounded by measured cost; do not invent a 100-cycle model soak. Platform claims require direct evidence. Native Windows Agentic evidence does not imply native-Windows authoritative `winds verify` support.

**Depends on**: T084. **Closes to authorize**: T086.

### [ ] T086 — Spec 006 acceptance / exact-head review / evidence reconciliation

**Authorized paths**:
- task-state updates in this `tasks.md`
- focused Spec 006 acceptance/evidence artifacts following repository precedent
- README/docs corrections only for claims actually proven

**Acceptance**:
- T070–T085 reconciled against canonical merges;
- every Spec FR/SC either proven or explicitly deferred/non-claimed within scope;
- no stale older-head evidence represented as current;
- final exact implementation candidate receives correctness/safety, Ponytail, and fresh independent review;
- zero unresolved material findings and all deterministic gates pass;
- real-runtime/platform claims limited to actual evidence;
- final landing guarded and explicit;
- ACP dependency, MCP, daemon/IPC, remote execution, generic plugin framework, recursive fleets, custom renderer, SQL Studio, and LLM Observatory remain unstarted unless separately respecified.

**Depends on**: T085.

**Canonical completion condition**:
```text
T070..T086=CLOSED_CANONICAL
SPEC_006_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

Closing T086 does not authorize a later Agentic phase.

---

## Dependency Chain

```text
T070 identity persistence
 -> T071 canonical session/task semantics
 -> T072 safe discovery fixtures
 -> T073 runtime binding/continuity truth
 -> T074 deterministic context
 -> T075 pure authority
 -> T076 approval digest/audit
 -> T077 fake Codex protocol
 -> T078 fake Claude path
 -> T079 FIRST REAL CODEX PROMPT
 -> T080 FIRST REAL CLAUDE PROMPT
 -> T081 cross-runtime handoff
 -> T082 one approved Worker edit
 -> T083 exact-candidate verification bridge
 -> T084 P2 findability
 -> T085 hardening
 -> T086 acceptance
```

## Non-Authorization Summary

Until this file is canonical:
```text
T070=NOT_AUTHORIZED_TO_IMPLEMENT
T071_PLUS=NOT_AUTHORIZED_TO_IMPLEMENT
REAL_CODEX_PROMPT=NOT_AUTHORIZED
REAL_CLAUDE_PROMPT=NOT_AUTHORIZED
```

After canonical Tasks acceptance:
```text
T070=AUTHORIZED_TO_START
T071_PLUS=DEPENDENCY_GATED
REAL_CODEX_PROMPT=BLOCKED_UNTIL_T079
REAL_CLAUDE_PROMPT=BLOCKED_UNTIL_T080
ACP_DEPENDENCY=NOT_AUTHORIZED
MCP=NOT_AUTHORIZED
DAEMON_IPC=NOT_AUTHORIZED
REMOTE_EXECUTION=NOT_AUTHORIZED
AUTOMATIC_LANDING=NOT_AUTHORIZED
```
