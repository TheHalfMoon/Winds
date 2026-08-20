# Tasks: Agentic Terminal & Local Delegation Control Plane

## Canonical Inputs

- Constitution 1.1.0: canonical.
- Spec 006 specification: canonical via PR #67.
- Spec 006 implementation Plan: canonical via PR #68.
- Tasks planning base: `d37f4f869f76d0715c0a0aee3d818c775771affe`.
- Canonical Plan tree: `d30b617e7cf038c91d5cdbff03f73142ebe07609`.

This file decomposes Spec 006 into independently reviewable slices. It does not itself execute an Agent, send a prompt, call a model/provider API, install a runtime, accept terms, duplicate credentials, enable MCP, add a daemon, or authorize remote execution.

## Global Execution Rules

1. Tasks execute strictly in dependency order unless this file explicitly marks independence.
2. Only one acceptance-critical task may be active for canonical landing at a time unless the tasks are explicitly documentation/review-only and cannot mutate overlapping product state.
3. Every implementation task starts from exact then-canonical `main`; stale handoff SHA values never override live repository truth.
4. Every task must preserve the canonical invariants:
   - `RUNTIME != MODEL`;
   - `NEW_SESSION != NEW_TASK`;
   - native resume != canonical Winds continuity;
   - live process ownership != durable native session identity;
   - imported history != canonical evidence;
   - worktree / ACP root != sandbox;
   - Planner direct authority != delegation ceiling;
   - child authority <= explicit child/delegation/team/human ceilings;
   - agent completion != verified/accepted;
   - changed candidate invalidates earlier candidate-bound review/evidence applicability;
   - no automatic winner or landing.
5. No task may add a generic runtime/plugin framework, public IPC, local network listener, remote execution, MCP runtime, ACP draft v2, recursive fleet scheduler, custom renderer, SQL Studio, LLM Observatory, vector/RAG memory system, or model gateway.
6. No task may add a dependency unless that exact task explicitly authorizes it and includes fresh exact-version/checksum/license/MSRV/platform/YAGNI review. The currently planned T070–T086 program requires no ACP dependency.
7. `agent-client-protocol` remains pinned provenance only. It is not an implementation dependency in any task below. A future ACP-speaking runtime requires a newly reviewed task amendment.
8. Protected Winds authority/trust state must not be writable as ordinary governed worktree content by the actor it controls.
9. `WINDS_ENFORCED` may be claimed only for operations actually mediated by Winds. Runtime-native restrictions remain `AGENT_NATIVE_ENFORCED` or weaker unless stronger enforcement is independently proven.
10. Any branch/head movement after review or deterministic evidence invalidates merge-ready status until exact-head gates are rerun.

## Standard Acceptance Gate for Every Implementation Task

Unless a task explicitly requires stronger evidence, acceptance requires on the exact final candidate:

- repository `quality` workflow SUCCESS;
- focused deterministic tests for the task's changed surface;
- relevant Linux/macOS/Windows behavior only where the task claims it;
- author correctness/safety/evidence-integrity review;
- Ponytail/YAGNI review;
- at least one independent reviewer pass on the exact acceptance-critical candidate or a review stack whose final delta reaches that exact head;
- zero unresolved material review findings;
- exact changed-file reconciliation against the task scope;
- no unauthorized dependency/runtime/protocol/authority expansion;
- guarded merge against the expected head SHA;
- post-merge canonical main/tree verification before the next dependent task starts.

Historical evidence remains historical and must not be rewritten as current exact-head evidence.

## Implementation Authorization Ladder

Canonical acceptance of this `tasks.md` authorizes **T070 only** to begin implementation.

Each subsequent task becomes authorized only after all listed dependencies are `CLOSED_CANONICAL` and the current repository truth still satisfies this file. This avoids turning the whole program into one blanket mutation authorization.

Two tasks carry additional explicit real-runtime gates:

```text
FIRST_REAL_CODEX_PROMPT_TASK=T079
FIRST_REAL_CLAUDE_PROMPT_TASK=T080
```

No task before T079 may launch a real Codex Agent/App Server for Agent work or send a Codex prompt.
No task before T080 may launch a real Claude Code Agent for Agent work or send a Claude prompt.
Safe non-Agent executable discovery/version inspection remains allowed in T072 because `DISCOVERY != AGENT_EXECUTION`.

---

## Phase 1 — Canonical Identity, Fixture-Only

### [ ] T070 — Workstream and Winds-session persistence substrate

**Purpose**: Add the minimum structural identity chain required for canonical work continuity without any Agent process.

**Exact intended product paths**:

- `migrations/0006_agentic_identity.sql` — new forward-only migration;
- `src/domain.rs` — typed workstream/session records and fail-closed vocabularies only as required;
- `src/store.rs` — minimal Store create/read/update/list operations;
- `src/t070_agentic_identity_tests.rs` — deterministic migration/Store/identity tests;
- `src/main.rs` only if required to register the focused test module; no user-facing Agent command is required in T070.

**Required schema invariant**:

```text
workspaces(workspace_id)
  -> workstreams(workstream_id, workspace_id)
      -> winds_sessions(session_id, workstream_id)
```

`winds_sessions` MUST NOT duplicate an independent `workspace_id` column in this slice.

**Acceptance requirements**:

- stable opaque workstream/session IDs independent of display names;
- rename does not change identity or orphan relationships;
- at least 20 fixture sessions across at least 5 workstreams in one workspace;
- duplicate/case/Unicode display names do not collide with identity;
- cross-workspace session/workstream mismatch is structurally impossible through the schema path;
- invalid/unknown persistence values fail closed;
- no Agent runtime discovery, launch, prompt, model/provider call, runtime binding, delegation, or candidate acceptance behavior.

**Dependencies**: canonical Tasks only.

**Authorization after closure**: T071.

### [ ] T071 — Continue / fork / new-session / new-task canonical semantics

**Purpose**: Prove `NEW_SESSION != NEW_TASK` and explicit canonical relationships before runtime-native continuity exists.

**Intended paths**:

- `src/agentic_identity.rs` — concrete canonical workstream/session operations;
- `src/cli_workspace.rs` and/or `src/main.rs` — smallest CLI proof surface only if needed;
- `src/store.rs` / `src/domain.rs` — only minimal extensions required by accepted semantics;
- `src/t071_agentic_continuity_tests.rs`.

**Acceptance requirements**:

- create/list/rename/select Winds sessions deterministically;
- new session can continue the same workstream without creating a new task;
- explicit new task creates a distinct workstream even with identical display text;
- fork records origin while creating a distinct Winds session identity;
- ambiguous continuation returns explicit candidates rather than choosing by recency alone;
- no runtime-native session ID is required;
- no Agent process or prompt.

**Dependencies**: T070 `CLOSED_CANONICAL`.

**Authorization after closure**: T072.

---

## Phase 2 — Runtime Discovery and Native-Binding Truth, Fixture-Only

### [ ] T072 — Codex/Claude safe runtime discovery with fake executables

**Purpose**: Represent exact local runtime identity and capability provenance without turning discovery into trust or Agent execution.

**Intended paths**:

- `src/agentic_runtime.rs` — closed `CODEX` / `CLAUDE` discovery model and concrete discovery functions;
- `src/t072_agentic_runtime_discovery_tests.rs`;
- `src/main.rs` / CLI only for a minimal read-only discovery proof if justified.

**Acceptance requirements**:

- absent/present/unsupported-version/changed-after-discovery fake executable cases;
- exact executable path/identity plus safely observable version;
- declared vs locally observed vs unavailable capability provenance;
- runtime identity remains separate from model/provider identity;
- auth readiness remains unknown/unavailable when it cannot be safely observed;
- no auto-install/update/auth/terms acceptance;
- no prompt/model/provider call;
- discovery cannot create a Winds execution that claims Agent work occurred;
- launch-significant identity can be revalidated before a later real start/resume.

**Dependencies**: T071 `CLOSED_CANONICAL`.

**Authorization after closure**: T073.

### [ ] T073 — Runtime-session binding persistence and truthful continuity states

**Purpose**: Persist only the native mapping facts needed to decide `RESUMED` vs `RECONSTRUCTED` vs `OWNERSHIP_LOST` without pretending a durable native ID is live ownership.

**Intended paths**:

- `migrations/0007_runtime_session_bindings.sql`;
- `src/domain.rs` / `src/store.rs` — typed binding record/state;
- `src/agentic_runtime.rs` — mapping/revalidation decision seam;
- `src/t073_runtime_binding_tests.rs`.

**Acceptance requirements**:

- binding links to `winds_sessions(session_id)` and stores concrete runtime kind plus exact executable/version provenance and optional native ID;
- exact valid mapping may be eligible for future native resume;
- stale/missing/ambiguous mapping never yields false `RESUMED`;
- replacement executable/version invalidates launch-significant mapping applicability;
- persisted PID/native ID alone cannot establish `LIVE` after Winds restart;
- ownership loss is explicit; no blind process/native-session attachment;
- no real Agent process or prompt.

**Dependencies**: T072 `CLOSED_CANONICAL`.

**Authorization after closure**: T074.

---

## Phase 3 — Canonical Context, Fixture-Only

### [ ] T074 — Deterministic canonical context capsule and transfer report

**Purpose**: Prove cross-session/runtime continuity without building a transcript database or claiming inaccessible private-state transfer.

**Intended paths**:

- `src/agentic_context.rs`;
- `src/domain.rs` / `src/store.rs` only for the smallest bounded canonical fact/reference persistence proven necessary;
- optional `migrations/0008_agentic_context_facts.sql` only if T074 demonstrates core workstream fields are insufficient; otherwise do not create it;
- `src/t074_agentic_context_tests.rs`.

**Acceptance requirements**:

- deterministic schema/versioned serialization and SHA-256 digest for identical canonical input/policy;
- stable ordering and explicit normalization/omission rules;
- canonical workspace/workstream/session/objective/constraints/decisions/candidate/evidence references;
- per-fact provenance retained;
- imported runtime/provider history cannot overwrite `WINDS_OBSERVED` / `HUMAN_DECIDED` facts;
- prompt/tool-like imported text remains data and grants no authority;
- transfer report distinguishes transferred, reconstructed/derived, omitted-by-policy/budget, and unavailable state;
- provider-private hidden state/private reasoning is never claimed as transferred;
- compaction does not mutate canonical work/evidence truth;
- no vector DB, embeddings, retrieval service, tokenizer, Agent process, or prompt.

**Dependencies**: T073 `CLOSED_CANONICAL`.

**Authorization after closure**: T075.

---

## Phase 4 — Authority and Approval, Fixture-Only

### [ ] T075 — Pure authority/delegation evaluator

**Purpose**: Freeze deterministic authority semantics before any live Agent can request an operation.

**Intended paths**:

- `src/agentic_authority.rs`;
- `src/domain.rs` for minimal typed authority/enforcement values;
- `src/t075_agentic_authority_tests.rs`.

**Acceptance requirements**:

- Planner direct authority and delegation ceiling are independent values;
- Worker effective authority cannot exceed explicit Worker grant ∩ Planner delegation ceiling ∩ applicable team policy ∩ human ceiling;
- explicit deny precedence is fail-closed;
- repo/model/tool/imported text cannot self-expand authority;
- evaluator returns decision/reason/required-human-action and performs no operation itself;
- all required enforcement-quality labels exist and cannot overclaim `WINDS_ENFORCED`;
- worktree/additional-root visibility never implies authorization or sandboxing;
- one Planner -> one Worker model only; no recursive fleet;
- no real Agent process or prompt.

**Dependencies**: T074 `CLOSED_CANONICAL`.

**Authorization after closure**: T076.

### [ ] T076 — Content-bound human approval digest and delegation audit substrate

**Purpose**: Bind explicit human approval to the exact normalized contract so changed content/resources cannot reuse stale approval.

**Intended paths**:

- `migrations/0008_agentic_delegation_audit.sql` if T074 did not consume 0008; otherwise use the next forward-only migration number resolved from live canonical truth;
- `src/agentic_authority.rs`;
- `src/store.rs` / `src/domain.rs`;
- `src/t076_agentic_approval_tests.rs`.

**Normalized approval identity must include, when applicable**:

- workstream/session IDs;
- requested Worker role/runtime;
- workspace/worktree/root identity;
- requested capability/resource/path scope;
- context capsule digest;
- delegation ceiling + Worker grant;
- budget fields when used;
- exact candidate/base identity when known.

**Acceptance requirements**:

- stable digest for identical normalized content;
- material normalized-content change invalidates approval and returns to ask/deny policy;
- approval audit contains no credential/token/full-environment duplication;
- protected policy/approval state is not ordinary governed repo content;
- no PKI/signing dependency is introduced for same-user approval;
- no real Agent process or prompt.

**Dependencies**: T075 `CLOSED_CANONICAL`.

**Authorization after closure**: T077.

---

## Phase 5 — Codex Structured Control, Fake Before Real

### [ ] T077 — Fake Codex App Server protocol client

**Purpose**: Prove the exact structured protocol state machine and bounds without starting real Codex or sending a model prompt.

**Intended paths**:

- `src/agentic_codex.rs` — narrow vendor-specific typed JSONL/stdio envelope/state machine;
- `src/t077_codex_protocol_tests.rs`;
- existing `process_scope.rs` only if a minimal reusable ownership seam is required and its current terminal/verification semantics remain unchanged.

**Mandatory handshake**:

```text
initialize request
  -> successful initialize response
  -> initialized notification
  -> only then thread/start | thread/resume | thread/fork | accepted later methods
```

**Acceptance requirements**:

- pre-handshake thread/turn methods fail closed;
- malformed/unknown/oversized JSONL is bounded and cannot grant authority;
- fake server exit during handshake is truthful failure;
- structured notifications are source-labelled Agent/runtime events, not verification evidence;
- approval request cannot self-authorize and is routed only to the authority seam;
- native thread ID remains distinct from canonical Winds session/workstream identity;
- controlled child cleanup uses proven ownership only;
- no generic JSON-RPC crate/framework, async runtime, real Codex process, or prompt/model call.

**Dependencies**: T076 `CLOSED_CANONICAL`.

**Authorization after closure**: T078.

### [ ] T078 — Fake Claude structured CLI construction/parser

**Purpose**: Prove exact resume, structured-output, and permission construction before launching real Claude.

**Intended paths**:

- `src/agentic_claude.rs`;
- `src/t078_claude_structured_tests.rs`.

**Acceptance requirements**:

- accepted construction uses structured `--print` plus `--output-format json|stream-json` as required by the tested path;
- exact `--resume <session-id>` only for a revalidated binding;
- `--continue` is never used for canonical Winds continuation;
- `--dangerously-skip-permissions` cannot appear in accepted command construction;
- malformed/truncated/oversized structured output fails truthfully;
- permission restrictions are labelled `AGENT_NATIVE_ENFORCED` or weaker unless independent stronger mediation exists;
- reconstructed new native session is labelled `RECONSTRUCTED`, not `RESUMED`;
- no real Claude process or prompt/model call.

**Dependencies**: T077 `CLOSED_CANONICAL`.

**Authorization after closure**: T079.

---

## Phase 6 — First Real Runtime Proofs

### [ ] T079 — FIRST REAL CODEX PROMPT AUTHORIZED TASK: bounded connected App Server proof

**Special authorization**: This is the first task in Spec 006 permitted to launch the exact locally discovered/revalidated real Codex App Server and send a Codex prompt. No earlier task may do so.

**Purpose**: Prove that the fixture-tested client interoperates with the real local App Server without granting editing/delegation authority yet.

**Required safety boundary**:

- use an explicit disposable/fixture repository or non-mutating proof context;
- one bounded proof session/turn only unless exact-version behavior requires a narrowly documented retry;
- no primary-checkout mutation;
- no network/credential escalation beyond the user's already configured local runtime state;
- no terms/access request or credential harvesting by Winds;
- deny/decline unexpected write/tool approval requests unless this exact task's reviewed proof contract explicitly allows a harmless fixture operation;
- no automatic PR/push/merge.

**Intended paths**:

- `src/agentic_codex.rs`;
- `src/execution.rs`, `src/domain.rs`, `src/store.rs` only for the minimum accepted Agent-execution child record if real execution persistence is proven necessary;
- corresponding forward-only migration only if the task actually lands that typed child record;
- `src/t079_codex_connected_tests.rs` for deterministic/fake coverage plus a separately recorded real proof artifact/process outside unit-test assumptions;
- minimal CLI proof command only if necessary to execute the accepted local proof.

**Acceptance evidence**:

- exact runtime executable/version and revalidation;
- complete initialize/initialized handshake;
- exact Winds session <-> native thread binding provenance;
- one bounded structured response;
- truthful enforcement/source labels;
- directly owned process cleanup or explicit `OWNERSHIP_LOST` if proof fails;
- no claim that the model result is verified/accepted.

**Dependencies**: T078 `CLOSED_CANONICAL`.

**Authorization after closure**: T080.

### [ ] T080 — FIRST REAL CLAUDE PROMPT AUTHORIZED TASK: bounded Planner/read-plan proof

**Special authorization**: This is the first task in Spec 006 permitted to launch exact locally discovered/revalidated real Claude Code and send a Claude prompt. No earlier task may do so.

**Purpose**: Prove one real Claude-backed Planner Winds session using the strongest exact-version read/plan-oriented restriction that can be truthfully represented without MCP or dangerous bypass.

**Required safety boundary**:

- one bounded Planner prompt against an explicit fixture/disposable or read-only planning context;
- no `--continue` for canonical continuity;
- no `--dangerously-skip-permissions`;
- no workaround through MCP;
- no primary-checkout mutation;
- no credential/terms automation;
- if stronger non-interactive write/tool mediation cannot be proven, remain Planner-only;
- runtime restriction must be labelled `AGENT_NATIVE_ENFORCED` or weaker unless stronger enforcement is independently proven.

**Intended paths**:

- `src/agentic_claude.rs`;
- runtime-binding / Agent-execution persistence surfaces only as already justified by T079 or minimally required here;
- `src/t080_claude_planner_tests.rs` plus separately recorded real proof evidence;
- minimal CLI proof command only if required.

**Acceptance evidence**:

- exact executable/version revalidation;
- exact native session ID provenance;
- structured Planner result retained as `AGENT_REPORTED`;
- exact resume can be demonstrated only if the native binding remains valid;
- a reconstructed new native session is reported as `RECONSTRUCTED`;
- no claim that Planner output itself is verified/accepted.

**Dependencies**: T079 `CLOSED_CANONICAL`.

**Authorization after closure**: T081.

---

## Phase 7 — Cross-Runtime Handoff and One Bounded Delegation

### [ ] T081 — Canonical Claude-Planner -> Codex-Worker handoff contract

**Purpose**: Connect the accepted real-runtime paths through deterministic Winds context and one inspectable delegation proposal without yet requiring an editing Worker run.

**Intended paths**:

- `src/agentic_context.rs`;
- `src/agentic_authority.rs`;
- `src/agentic_claude.rs` / `src/agentic_codex.rs` only for the smallest coordination seam;
- optional concrete `src/agentic.rs` coordinator only if duplication otherwise exists; do not introduce a generic runtime trait/plugin host;
- `src/t081_cross_runtime_handoff_tests.rs`.

**Acceptance requirements**:

- same canonical workstream survives Claude -> Codex runtime change;
- transfer report enumerates transferred/derived/omitted/unavailable facts;
- Planner Worker proposal remains `AGENT_REPORTED`;
- human sees exact normalized delegation contract before approval;
- child-over-ceiling and changed-approval-content paths fail closed;
- no recursive delegation/fleet;
- no automatic Worker execution from Planner prose alone.

**Dependencies**: T080 `CLOSED_CANONICAL`.

**Authorization after closure**: T082.

### [ ] T082 — One human-approved Codex Worker edit in exact isolated worktree

**Purpose**: Prove the differentiated P1 walking skeleton with exactly one Planner -> one Worker and one human-approved edit scope.

**Live Agent scope**: real Codex Worker prompt is allowed under the already-established T079 runtime path, but only after the normalized T081 contract is explicitly approved through the accepted human-decision path.

**Intended paths**:

- existing system-Git/worktree surfaces in `src/git.rs` or the smallest compatible extension;
- `src/agentic_authority.rs` / `src/agentic_codex.rs` / coordinator seam;
- Agent execution typed child persistence if not already landed;
- `src/t082_worker_worktree_tests.rs`.

**Acceptance requirements**:

- explicit exact base/candidate parent and Winds-owned Worker worktree;
- Worker bound to exact worktree root/common-dir identity;
- worktree is never called an OS sandbox;
- operation/resource scope cannot exceed approved Worker grant;
- unexpected approval request cannot self-authorize;
- dirty/failed/ambiguous Worker state is retained for recovery, never force-cleaned;
- Worker completion remains `AGENT_REPORTED` until Git/evidence is independently observed;
- no primary-checkout mutation, automatic merge/push/PR, or automatic winner.

**Dependencies**: T081 `CLOSED_CANONICAL`.

**Authorization after closure**: T083.

---

## Phase 8 — Exact Candidate Review and Verification Authority

### [ ] T083 — Candidate binding, review staleness, and existing `winds verify` integration

**Purpose**: Preserve Winds' verification-native authority after Agentic editing.

**Intended paths**:

- existing `src/git.rs`, `src/store.rs`, `src/domain.rs`, `src/main.rs` only where required;
- minimal Agentic candidate/review-state seam in the already-proven modules;
- `src/t083_agentic_candidate_evidence_tests.rs`.

**Acceptance requirements**:

- exact candidate OID/tree, not branch/display name, is the acceptance identity;
- independent-review context includes exact candidate/diff/criteria/canonical constraints/evidence and excludes builder confidence/persuasion as authority;
- candidate A review/checks become `STALE` for candidate B while A remains traceable;
- Agent `done/tests passed` cannot satisfy deterministic verification;
- existing `winds verify` runs on the exact candidate and resulting evidence is referenced, not duplicated as Agent truth;
- existing verify/promote/recover semantics and Spec 003 authority regressions stay green;
- final landing remains explicit human decision.

**Dependencies**: T082 `CLOSED_CANONICAL`.

**Authorization after closure**: T084.

---

## Phase 9 — P2 Findability

### [ ] T084 — Deterministic session/path findability without authority escalation

**Purpose**: Add the smallest useful P2 selection UX only after the P1 continuity/delegation/verification loop is proven.

**Intended paths**:

- reuse existing CLI/workspace selection surfaces where practical;
- optional `src/agentic_find.rs` only if a concrete separate seam is required;
- `src/t084_agentic_findability_tests.rs`.

**Acceptance requirements**:

- deterministic partial/fuzzy session selection with explicit disambiguation;
- exact canonical path resolution before context/execution use;
- Unicode/case/similar-name fixtures;
- changed/recent/test/symbol-derived candidates retain selection provenance;
- unavailable semantic/symbol intelligence remains unavailable;
- visibility in search/picker never grants read/send/modify authority;
- prefer simple deterministic matching; no fuzzy/search dependency unless measured evidence and a fresh task amendment justify it.

**Dependencies**: T083 `CLOSED_CANONICAL`.

**Authorization after closure**: T085.

---

## Phase 10 — Hardening and Acceptance

### [ ] T085 — Cross-platform negative/fault/repetition regression campaign

**Purpose**: Stress the accepted implementation without inventing expensive model soaks or broad unsupported platform claims.

**Required deterministic campaign**:

- corrupt/unknown workstream/session/native-binding values;
- cross-workspace mismatch attempts;
- deleted/changed workspace identity;
- runtime executable replacement after discovery;
- malformed/oversized runtime output;
- Codex exit before initialize response and before initialized completion;
- unknown protocol message;
- Claude resume rejection/reuse;
- imported-history injection attempts;
- context truncation/omission markers;
- child-over-ceiling / explicit-deny / approval-replay attempts;
- runtime success claim conflicting with Git observation;
- candidate movement during review/check;
- dirty/failed Worker worktree recovery;
- no blind PID/native-session attachment after restart;
- bounded fake Codex/Claude lifecycle repetition;
- deterministic context hash repetition;
- Spec 003 verification/store regression suite.

Real-runtime repetition, if used, must be bounded from measured cost and cannot be presented as a 100-cycle model soak unless separately justified.

Each platform/execution-domain claim must have direct evidence. Native Windows Agentic runtime proof does not imply native-Windows authoritative `winds verify` support.

**Dependencies**: T084 `CLOSED_CANONICAL`.

**Authorization after closure**: T086.

### [ ] T086 — Spec 006 acceptance documentation, exact-head independent review, and canonical evidence reconciliation

**Purpose**: Reconcile the entire accepted Spec 006 first implementation program without adding new runtime scope.

**Intended paths**:

- `specs/006-agentic-terminal-local-delegation-control-plane/tasks.md` task-state updates;
- focused Spec 006 acceptance/evidence artifact(s) only as required by repository precedent;
- user-facing README/docs corrections only for claims actually proven by canonical implementation/evidence.

**Acceptance requirements**:

- every T070–T085 task status reconciled against canonical merge truth;
- all Spec FR/SC requirements either proven by canonical evidence or explicitly recorded as deferred/non-claimed according to scope;
- no stale older-head review/check represented as current;
- final exact implementation candidate receives correctness/safety, Ponytail, and fresh independent review;
- zero unresolved material findings;
- final deterministic gates pass on exact candidate;
- real-runtime/platform claims are limited to actual evidence;
- no automatic landing; canonical merge remains a guarded explicit repository action;
- MCP, daemon/IPC, remote execution, generic plugin framework, ACP dependency, recursive fleets, custom renderer, SQL Studio, and LLM Observatory remain unstarted unless a later separately authorized specification changes scope.

**Dependencies**: T085 `CLOSED_CANONICAL`.

**Canonical completion condition**:

```text
T070..T086=CLOSED_CANONICAL
SPEC_006_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

No later agentic phase is implicitly authorized by closing T086.

---

## Task Dependency Chain

```text
T070 identity persistence
  -> T071 canonical session/task semantics
  -> T072 safe runtime discovery fixtures
  -> T073 native-binding/continuity truth
  -> T074 deterministic context
  -> T075 pure authority evaluator
  -> T076 human approval digest/audit
  -> T077 fake Codex protocol
  -> T078 fake Claude structured path
  -> T079 FIRST REAL CODEX PROMPT
  -> T080 FIRST REAL CLAUDE PROMPT
  -> T081 cross-runtime handoff contract
  -> T082 one approved Worker edit
  -> T083 exact-candidate review/verification bridge
  -> T084 P2 findability
  -> T085 hardening/regression
  -> T086 acceptance/reconciliation
```

## Non-Authorization Summary

Until this Tasks file itself is accepted and merged canonically:

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
