# Implementation Plan: Resumable Workflow & Decision Ledger

## Summary

Build the smallest one-process, local durable workflow/decision layer that satisfies Spec 008 while preserving the accepted Spec 003/006/007 identity, evidence, runtime-continuity, terminal-lifecycle, platform, privacy, review-independence, and human-landing boundaries.

The first implementation program will:

1. add stable `WorkflowRun` and unique `StageRun` attempt identity bound to the existing canonical workspace + workstream/task hierarchy;
2. reuse the existing Winds-owned `winds.db` SQLite store and existing `rusqlite` dependency rather than introduce another database or persistence crate;
3. add deterministic stage lifecycle transitions that never infer authority from prose, labels, paths, session presence, or runtime coincidence;
4. add exact artifact/candidate baselines whose applicability becomes stale when required identity moves;
5. reuse Spec 006 runtime/session binding truth for `LIVE`, `RESUMED`, `RECONSTRUCTED`, `OWNERSHIP_LOST`, and unavailable continuation states without upgrading the still-open live-runtime nonclaims;
6. implement finite retry/no-progress state with distinct attempt history and fail-closed treatment of ambiguous non-idempotent effects;
7. add an append-only decision ledger that preserves source/authority/candidate/evidence lineage and supersession history;
8. generate structured handoff and operator status/resume/why-blocked projections from canonical state rather than persist a second competing source of truth;
9. validate versioned durable workflow state before use, preserve unreadable/corrupt state, and make redaction/completeness loss explicit;
10. prove deterministic, regression, recovery, adversarial, and directly applicable platform behavior before final reconciliation.

This Plan selects the existing Winds SQLite persistence substrate for later Task-stage use. This Plan PR itself adds no migration, source, dependency, lockfile, workflow semantic, runtime behavior, provider/browser integration, daemon/IPC, remote execution, learning system, or automatic landing behavior.

## Constitution Check

The implementation program MUST preserve:

```text
AGENT_COMPLETION_IS_NOT_VERIFICATION
VERIFICATION_IS_NOT_ACCEPTANCE
ACCEPTANCE_IS_NOT_LANDING
VERIFY_THE_EXACT_CANDIDATE
CURRENT_ONE_PROCESS_ARCHITECTURE_PRESERVED
NO_DAEMON_OR_IPC_IN_SPEC_008
CANONICAL_WORK_IDENTITY_PRECEDES_SESSION_OR_RUNTIME_IDENTITY
RESUMED_IS_NOT_RECONSTRUCTED
HISTORICAL_FAILURE_IS_NOT_REWRITTEN_AS_SUCCESS
```

Additional constitutional constraints:

- canonical workspace and workstream/task identity remain inherited truth; Spec 008 does not create a second task identity namespace;
- every workflow operation validates its bound canonical workspace/workstream/task relationship before use;
- display names, repository paths, Winds sessions, native runtime IDs, PIDs, transcript continuity, and UI presence never determine workflow identity or authority;
- repository-native Git/evidence observations remain verification authority;
- `AGENT_REPORTED`, `WINDS_OBSERVED`, and `HUMAN_DECIDED` remain distinct source classes;
- Spec 006 runtime/native continuation semantics are reused, not redefined or silently promoted;
- no persistent background owner, daemon, socket, local server, IPC/RPC protocol, remote execution route, provider/model router, browser runtime, generic plugin/workflow runtime, learning subsystem, semantic/vector memory, or automatic Git landing is introduced;
- every implementation slice must start from exact then-current canonical `main`, remain dependency-ordered, pass deterministic gates, receive author correctness/safety review, Ponytail/YAGNI review, and fresh independent substantive review before guarded landing;
- HEAD/TREE movement invalidates prior exact-candidate qualification.

## Canonical Baseline

Planning base:

```text
BASE=5f25341c2f317f528e542d4d90a112c784ab24dc
BASE_TREE=f0412553a8e4a5231f58a33303ba7bc8c9091e0d
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=IN_QUALIFICATION
SPEC_008_TASKS_AUTHORIZED=NO
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```

Inherited canonical seams to reuse rather than duplicate:

- `WorkspaceRecord.workspace_id` + canonical Git workspace identity;
- `WorkstreamRecord.workstream_id -> workspace_id` as the existing canonical workstream/task relationship;
- `WindsSessionRecord.session_id -> workstream_id` for Winds session identity;
- Spec 006 runtime/native binding records and continuation classifications;
- exact Git candidate identity (`candidate_oid`, `candidate_tree`) and candidate-bound evidence/review applicability;
- accepted human-decision/approval paths and explicit human landing authority;
- existing `Store`, `winds.db`, WAL mode, foreign-key enforcement, schema-integrity validation precedents, and append-only audit-trigger precedent;
- existing native Windows/ConPTY, WSL2, Linux PTY, and macOS PTY/platform qualification boundaries.

The live-runtime nonclaims remain unchanged throughout Spec 008 unless separately governed:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

## Architecture Decision: One Process, Six Narrow Seams

Spec 008 adds no general workflow engine. The first implementation should be six concrete Winds-owned seams inside the existing process.

### 1. Canonical workflow domain state

Add closed domain types for workflow identity and stage-attempt state.

Logical shape:

```text
WorkflowRunIdentity {
  workflow_run_id,
  workspace_id,
  workstream_id,
}

StageRunIdentity {
  stage_run_id,
  workflow_run_id,
  stage_key,
  attempt_ordinal,
  predecessor_stage_run_id?,
}
```

Rules:

- `workflow_run_id` is stable and never derived from a label/path/session/runtime ID;
- `workspace_id` + `workstream_id` are immutable after `WorkflowRun` creation;
- creation and every later workflow load/use validate that the workstream belongs to the bound workspace in the existing canonical hierarchy;
- no separate `task_id` column is invented because Spec 006 already defines workstream/task as the canonical work identity;
- `stage_key` identifies the logical stage within one workflow and may be human-readable but is not sufficient authority by itself;
- each actual attempt receives a unique `stage_run_id` and monotonically increasing attempt ordinal for its logical stage;
- retry, reconstruction, reassignment, and recovery create a new stage attempt where the work is materially new; historical attempt identity is never rewritten.

### 2. Existing SQLite persistence adapter

Reuse the current `Store` + `winds.db` + `rusqlite = 0.40.2` persistence substrate.

Plan decision:

- no new database;
- no new persistence/ORM crate;
- no new async runtime;
- no second state directory;
- no event-sourcing framework;
- no generic workflow framework;
- later Tasks may add one idempotent Spec 008 migration, expected to follow the existing numeric migration convention after `0009_agentic_delegation_audit.sql`;
- workflow methods should use a dedicated `ensure_workflow_schema()` / schema-integrity validation seam following the accepted Spec 006 approval-schema precedent rather than silently assuming partially-created objects are valid;
- physical SQLite/database-open failure must leave the existing database file untouched and fail explicitly; Spec 008 must not auto-delete/recreate `winds.db` as recovery.

Minimal logical relational shape for Tasks to refine into exact DDL:

```text
workflow_runs
  workflow_run_id PK
  workspace_id FK
  workstream_id FK
  schema_version
  terminal_state?         # NULL or COMPLETED / CANCELLED / RECOVERY_REQUIRED
  created_unix_ms

workflow_stage_runs
  stage_run_id PK
  workflow_run_id FK
  stage_key
  attempt_ordinal
  predecessor_stage_run_id?
  relation_kind?          # RETRY_OF / RECOVERY_OF / REASSIGNMENT_OF where applicable
  lifecycle_state
  failure_class?
  checkpoint_identity?
  material_progress_basis?
  outcome_reason?
  created_unix_ms
  updated_unix_ms

workflow_artifact_baselines
  baseline_id PK
  stage_run_id FK
  baseline_kind
  stable_reference
  candidate_oid?
  candidate_tree?
  created_unix_ms

workflow_actor_bindings
  binding_id PK
  stage_run_id FK
  winds_session_id?
  runtime_binding_id?
  continuation_class
  bound_unix_ms

workflow_reconstruction_reports
  reconstruction_report_id PK
  binding_id FK UNIQUE
  schema_version
  canonical_report_json
  created_unix_ms

workflow_decisions
  decision_id PK
  workflow_run_id FK
  stage_run_id?
  source_class
  authority_class
  decision_type
  decision_result
  predecessor_decision_id?
  candidate_oid?
  candidate_tree?
  evidence_reference?
  content_state
  safe_rationale?
  created_unix_ms
```

Required identity/schema constraints:

- because `workflow_runs` deliberately stores both `workspace_id` and `workstream_id`, its insert path MUST use a 0009-style fail-closed hierarchy constraint/trigger (or an equivalently strong database-enforced relation) proving the workstream belongs to that workspace; application-side checking alone is insufficient;
- Tasks must enforce uniqueness of `(workflow_run_id, stage_key, attempt_ordinal)` and require any predecessor to belong to the same workflow and logical stage lineage unless the explicit relation kind is a separately authorized cross-stage relation;
- child rows retain canonical work context through their immutable `workflow_run_id` parent rather than duplicating workspace/workstream columns everywhere;
- every child operation must load and revalidate the parent workflow context before the row can be treated as applicable;
- if Tasks prove that a duplicated identity column is required for an audit row, it must use the existing 0009-style fail-closed hierarchy trigger and may not become a second authority source;
- no persisted handoff table is selected for the first slice; handoff is a deterministic projection from canonical workflow/stage/artifact/decision state unless a later Task proves a durable record is required;
- artifact-baseline, actor-binding, and reconstruction-report rows are immutable historical facts after creation; refreshed baseline/actor/reconstruction truth uses a new row/attempt relationship rather than rewriting prior history;
- `workflow_reconstruction_reports.canonical_report_json` is a bounded, schema-versioned structured metadata report only; it stores category/source/transfer/completeness/reference truth, not raw provider-private state, transcript, prompts, credentials, or environment dumps;
- no persisted free-form transcript, raw prompt, environment dump, credential, token, or provider-private state is added.

### 3. Exact artifact/candidate baseline evaluator

A baseline is a structured identity reference, not a pathname snapshot.

Initial baseline kinds should be closed and concrete, for example:

- exact Git candidate: OID + tree;
- existing Winds verification evidence: run/evidence identity + candidate identity;
- prior workflow stage output: producing `stage_run_id` + stable output identity;
- canonical decision: `decision_id` plus its candidate/evidence applicability where relevant;
- explicit bounded file/blob artifact only where Tasks can prove a stable content identity using already-available SHA-256 support.

Rules:

- path/name similarity never proves freshness;
- candidate movement immediately makes candidate-bound baseline/evidence/review applicability stale/not-applicable without deleting history;
- required baseline absence/ambiguity blocks applicability;
- stale state is recomputed from canonical referenced identity, not copied forward as presentation state;
- baseline comparison must be deterministic for the same stored canonical inputs.

### 4. Actor/continuation binding adapter

Reuse Spec 006 runtime/session mapping and do not create a competing runtime owner.

The workflow layer may bind a stage attempt to:

- a Winds session identity;
- an existing runtime/native binding identity where one is actually applicable;
- an explicit continuation classification.

Continuation classification must remain closed and source-truthful, including as applicable:

```text
RESUMED
RECONSTRUCTED
OWNERSHIP_LOST
UNAVAILABLE
UNPROVEN
```

Rules:

- `RESUMED` requires the existing exact runtime/native mapping and accepted runtime-supported resume proof revalidated at use time;
- `RECONSTRUCTED` means a new actor/runtime was initialized from canonical Winds state;
- every `RECONSTRUCTED` binding MUST be created atomically with one bounded immutable `ReconstructionReport`; a reconstructed binding is not applicable if the required report is absent, malformed, incomplete for required categories, or bound to a different StageRun/actor binding;
- the reconstruction report uses a closed material-context category set in the first slice: `CANONICAL_WORK_CONTEXT`, `OBJECTIVE_CONSTRAINTS`, `DECISIONS`, `CANDIDATE_EVIDENCE`, `PRIOR_STAGE_OUTPUTS`, `RUNTIME_NATIVE_CONTEXT`, and `PROVIDER_PRIVATE_STATE`;
- each category records source/provenance plus exactly one transfer state from `PRESERVED_REFERENCE`, `RECONSTRUCTED`, `DERIVED`, `OMITTED`, `UNAVAILABLE`, or `NO_LONGER_TRANSFERABLE`, and an explicit completeness/content state where material content may be redacted or absent;
- provider-private state is never fabricated as transferred; when it cannot be proven/exported it is recorded `UNAVAILABLE` or `NO_LONGER_TRANSFERABLE` without persisting the private payload;
- the same deterministic reconstruction-context evaluation used by execution is exposed in side-effect-free resume preview before a reconstruction action; after reconstruction, the persisted report becomes the historical source for status/handoff truth;
- restart of Winds alone proves no live native ownership;
- actor/session replacement creates a new binding/attempt relationship rather than changing the historical actor;
- an unavailable or ambiguous runtime/native mapping fails closed and cannot be upgraded by transcript or UI continuity;
- no new provider/model API or real Claude/Codex execution authority is created by this Plan.

### 5. Append-only decision ledger

Decision records are evidence-bearing historical facts, not mutable current configuration rows.

Plan decision:

- inserts create new immutable decision records;
- accepted/rejected/reverted/superseded history remains queryable;
- semantic modification is represented by a new decision record with explicit predecessor/supersession lineage;
- later Tasks should use database triggers or equivalent schema-level protection following the existing `agentic_delegation_approvals` no-update/no-delete precedent;
- replay of the same completion/decision operation must be idempotent or rejected without creating duplicate canonical truth;
- a decision stores source and authority class explicitly;
- agent/terminal/UI text cannot create `HUMAN_DECIDED` truth without the existing accepted human-decision path;
- decision history never grants execution, verification, candidate selection, merge, or landing authority.

### 6. Read-only handoff and operator projections

Generate deterministic projections rather than persist competing summary truth.

Initial projection surfaces:

```text
WorkflowStatusProjection
ResumePreviewProjection
WhyBlockedProjection
ReviewerHandoffProjection
```

Every projection must include enough stable identity to prove which workflow/stage/work context it describes and must revalidate the workflow's canonical workspace/workstream binding before returning an applicable result.

`WorkflowStatusProjection` should expose at minimum:

- workflow/stage/attempt identity;
- current stage lifecycle state;
- actor/continuation truth;
- the persisted reconstruction report category/source/transfer/completeness summary whenever the applicable binding is `RECONSTRUCTED`;
- required baseline freshness/applicability;
- blocker/approval/external-condition state;
- retry/no-progress state;
- exact candidate/evidence applicability where relevant;
- separate verification and human-acceptance state when those existing authorities are in view.

`ResumePreviewProjection` must say whether the next action would be exact native resume, Winds reconstruction, reassignment, blocked/unavailable, or ownership-lost handling. When reconstruction is the prospective action, the preview must include the deterministic material-context transfer/loss evaluation that would become the immutable reconstruction report if the action proceeds. It must never execute a resume/reconstruction merely because the preview was requested.

`WhyBlockedProjection` must return a canonical known blocker and required explicit approval/external/recovery action where provable; it must use `UNKNOWN`/unavailable truth rather than invent a reason.

`ReviewerHandoffProjection` must include exact candidate/artifact/evidence/authority context but exclude builder persuasion/confidence and untrusted transcript as acceptance guidance by default. If the current/relevant actor binding is reconstructed, the handoff must include the bounded reconstruction report summary so omitted/unavailable/derived context is visible to the reviewer.

## Stage Lifecycle Model

Use one explicit closed stage-attempt state machine:

```text
PREPARED
ACTIVE
WAITING_APPROVAL
WAITING_EXTERNAL
BLOCKED
FAILED
STALE
CANCELLED
COMPLETED
RECOVERY_REQUIRED
```

Allowed first-slice transitions:

```text
PREPARED -> ACTIVE | STALE | CANCELLED | RECOVERY_REQUIRED
ACTIVE -> WAITING_APPROVAL | WAITING_EXTERNAL | BLOCKED | FAILED | STALE | CANCELLED | COMPLETED | RECOVERY_REQUIRED
WAITING_APPROVAL -> ACTIVE | BLOCKED | FAILED | STALE | CANCELLED | RECOVERY_REQUIRED
WAITING_EXTERNAL -> ACTIVE | BLOCKED | FAILED | STALE | CANCELLED | RECOVERY_REQUIRED
BLOCKED -> ACTIVE | FAILED | STALE | CANCELLED | RECOVERY_REQUIRED
```

`FAILED`, `STALE`, `CANCELLED`, `COMPLETED`, and `RECOVERY_REQUIRED` are terminal for that `StageRun` attempt in the first implementation. New work after one of these states uses a new attempt with explicit lineage rather than resurrecting the historical attempt.

Consequences:

- duplicate requests for an already-reached state are idempotent no-change only when the operation identity and canonical preconditions match;
- every invalid transition fails closed;
- stage completion means only that stage's procedural contract completed;
- stage completion does not imply candidate verification, human acceptance, workflow landing, or success of another stage;
- `CANCELLED` is reached only by an explicit stage-cancel compare-and-set mutation whose caller/authority input passes the applicable accepted Winds authority/human-decision path; terminal/agent/UI prose and a decision row by itself can never cancel a stage;
- an `AGENT_REPORTED` decision may explain/request cancellation but is never cancellation authority; a later decision record may reference an already-authorized cancellation as history without causing the transition;
- workflow-level nonterminal status is derived from canonical workflow + stage records rather than maintained as an independently mutable UI summary;
- `WorkflowRun.terminal_state` is nullable and may become only `COMPLETED`, `CANCELLED`, or `RECOVERY_REQUIRED` through an explicit compare-and-set terminal action;
- workflow `COMPLETED` is accepted only when no latest logical-stage attempt is `PREPARED`, `ACTIVE`, `WAITING_APPROVAL`, `WAITING_EXTERNAL`, `BLOCKED`, or `RECOVERY_REQUIRED`, and every logical stage is resolved by either a latest applicable `COMPLETED` attempt or a latest `CANCELLED` attempt produced by the explicit authorized stage-cancel transition above; decision records, including `AGENT_REPORTED`, stale, superseded, forged, or otherwise non-applicable decisions, never satisfy workflow-completion resolution by themselves;
- duplicate workflow completion is idempotent no-change for the same exact canonical preconditions; conflicting terminal mutation is rejected;
- workflow completion remains purely procedural and still does not imply candidate verification, human acceptance, or landing.

## Retry and No-Progress Policy

The first implementation must be finite and deliberately conservative.

Plan decision:

```text
AUTOMATIC_RETRY=DISABLED
MAX_CONSECUTIVE_NO_PROGRESS_RETRIES=2
SECOND_NO_PROGRESS_RETRY_FAILURE=RETRY_BUDGET_EXHAUSTED
RETRY_AFTER_BUDGET_EXHAUSTION=REJECTED_WITHOUT_NEW_STAGE_RUN
AMBIGUOUS_NON_IDEMPOTENT_RETRY=FORBIDDEN_WITHOUT_EXPLICIT_RECONCILIATION
```

Interpretation:

- retry is an explicit workflow action, never an invisible background loop;
- the initial attempt is not counted as a retry;
- the second consecutive retry that still proves no material progress ends that `StageRun` as `FAILED` with explicit `RETRY_BUDGET_EXHAUSTED` reason; the budget does not silently reset;
- any further retry request in the same no-progress lineage is rejected without creating another `StageRun` until an explicit accepted recovery/re-preparation condition establishes a new applicable basis;
- material progress must be deterministically evidenced, not asserted by prose;
- each retryable attempt persists/derives a normalized failure class, checkpoint identity where applicable, and material-progress basis sufficient for later deterministic comparison and operator projection;
- deterministic no-progress signals include unchanged required candidate/artifact signatures plus the same normalized failure class, repeated checkpoint identity without a newer accepted checkpoint, and exhausted explicit task budget where one applies;
- a changed candidate/artifact alone is not automatically "progress" if the failure class/checkpoint remains materially unresolved; Tasks must define the exact comparator for each retryable slice;
- any prior possibly-completed non-idempotent side effect with ambiguous completion moves to `RECOVERY_REQUIRED`/explicit reconciliation, not automatic retry;
- every retry creates a distinct `StageRun` and preserves the failed predecessor.

The numeric no-progress cap is a first-slice safety ceiling, not a performance target. A later accepted amendment may tighten or broaden it only with explicit evidence and no relaxation of the unbounded-retry prohibition.

## Durable State Versioning and Recovery

Select a Spec 008 logical schema version beginning at `1`.

Before a workflow row or dependent record is used, Winds must validate:

- supported logical schema version;
- required identifiers are non-empty and structurally valid;
- `workflow_run_id` exists;
- bound `workstream_id` still belongs to bound `workspace_id` in canonical Winds identity tables;
- dependent stage/baseline/actor/decision references point to the expected workflow/stage lineage;
- lifecycle/continuation/source/authority enums are known;
- candidate OID/tree values are well-formed where present;
- supersession/predecessor relationships do not create self-reference or impossible lineage;
- required redaction/completeness markers are present where material content may be omitted;
- every applicable `RECONSTRUCTED` actor binding has exactly one schema-valid reconstruction report bound to the same binding/StageRun, with all first-slice material-context categories represented once and no unknown transfer state silently accepted.

Recovery posture:

- unsupported schema version -> explicit `RECOVERY_REQUIRED` / incompatible state;
- malformed or internally inconsistent row -> explicit failure/recovery-required truth;
- physical SQLite corruption/open failure -> leave `winds.db` unchanged and fail; do not create a fresh empty database over it;
- partial schema objects -> schema integrity check fails rather than silently treating the schema as complete;
- migration must be idempotent and preserve canonical workflow/decision/attempt identity and historical failed/stale state;
- no automatic destructive repair, recursive state deletion, or historical rewrite is authorized by the first slice.

## Redaction and Completeness Policy

Persist the minimum canonical structured data necessary for deterministic workflow truth.

Do not persist in Spec 008 workflow tables by default:

- raw terminal transcript;
- full model prompts/responses;
- provider-private hidden state;
- credentials/tokens/secrets;
- full environment dumps;
- arbitrary repository file contents;
- builder persuasion/confidence text in reviewer handoffs.

Material text-bearing fields must carry an explicit content state such as:

```text
FULL
REDACTED
OMITTED
UNAVAILABLE
```

Rules:

- redaction happens before durable workflow persistence;
- `REDACTED`/`OMITTED`/`UNAVAILABLE` remain visible completeness facts;
- missing required evidence content cannot be relabelled complete because a safe summary exists;
- do not persist hashes of secret/highly sensitive omitted values merely to prove omission unless a later Task proves that doing so is safe and necessary;
- prefer stable references to existing Winds evidence/artifacts over copying evidence payload into the workflow ledger.

## Concurrency and Transaction Boundaries

The first implementation remains one Winds process but SQLite/WAL and concurrent commands still require deterministic transactions.

Required transaction rules:

- workflow creation + canonical work-context validation occur in one transaction;
- stage attempt creation + attempt ordinal/lineage validation occur in one transaction;
- lifecycle transition uses expected-current-state compare-and-set semantics;
- retry attempt creation and predecessor final-state validation are atomic;
- decision insertion + predecessor/supersession validation are atomic;
- candidate/artifact baseline capture is atomic with stage preparation where applicability depends on it;
- no transaction may hold a database write lock while waiting on a terminal child, reviewer, external process, provider, network service, or human input;
- a process crash between side effect and durable checkpoint must be treated according to the explicit idempotency/recovery rule, never guessed successful from intent.

## CLI / TUI Integration Plan

Do not create a new application shell or nested generic workflow framework.

Tasks should first expose the workflow layer through small existing-style Winds surfaces, with exact command spelling selected in Tasks after CLI collision review.

Required first user capabilities:

- create/open a workflow under an exact canonical workspace + workstream/task;
- prepare/start one stage attempt;
- record explicit blocked/waiting/failure/completion transitions;
- inspect workflow status;
- inspect resume/reconstruction preview without side effects;
- inspect why-blocked truth;
- record/query canonical decisions through accepted authority paths;
- generate reviewer handoff context;
- retry/recover by creating a new attempt with explicit lineage.

Workbench/TUI may later render the same read-only projections, but must not become a second workflow authority or land before the underlying deterministic state/projection seam is qualified.

## Testing Strategy

### Pure model tests first

Prove without SQLite or process/runtime coupling:

- workflow/stage identity normalization and immutability;
- full legal/illegal stage transition matrix;
- retry/no-progress state and budget exhaustion;
- idempotent duplicate transition/completion behavior;
- decision supersession/reversion lineage;
- redaction/completeness enums;
- deterministic status/resume/why-blocked projection logic.

### Persistence and schema fixtures

Prove:

- migration idempotence;
- schema-object integrity validation;
- workflow -> canonical workspace/workstream binding and fail-closed mismatch;
- child row -> workflow/stage lineage integrity;
- immutable/append-only decision history;
- restart preserves stable workflow/stage/decision identity;
- corrupt/partial/unknown-version state does not initialize a clean false workflow;
- existing database contents remain preserved on workflow-schema failure;
- WAL/foreign-key behavior remains compatible with existing Store tests.

### Freshness/evidence fixtures

Prove:

- candidate A evidence/baselines are not applicable to moved candidate B;
- stale path/name-equivalent artifacts do not satisfy a new stage;
- historical evidence remains inspectable;
- terminal/agent forged `PASS`/`VERIFIED`/`ACCEPTED`/decision JSON does not promote canonical state.

### Continuation fixtures

Reuse accepted Spec 006 seams to prove:

- exact revalidated native mapping -> truthful `RESUMED` only where already supported;
- new runtime/session from canonical workflow state -> `RECONSTRUCTED` plus an immutable bounded reconstruction report;
- reconstruction-report fixtures cover preserved canonical references, derived state, omitted artifacts/context, unavailable provider-private state, no-longer-transferable runtime/native context, and stale native mappings;
- missing/malformed/incomplete reconstruction report makes a `RECONSTRUCTED` binding non-applicable;
- stale/missing/ambiguous mapping -> fail-closed unavailable/ownership-lost/recovery truth;
- process restart alone -> no live-ownership upgrade;
- Spec 006 live-runtime nonclaims remain `NO`.

### Retry and side-effect fixtures

Prove:

- two consecutive no-progress retries are preserved and the second failure records `RETRY_BUDGET_EXHAUSTED`;
- a further retry request is rejected without creating a new attempt until an explicit accepted recovery/re-preparation condition establishes a new applicable basis;
- material progress comparator is deterministic;
- ambiguous non-idempotent completion is not silently retried;
- retry success preserves earlier failed attempts.

### Reviewer/handoff fixtures

Prove:

- canonical exact candidate/evidence/authority context is retained;
- builder persuasion/confidence is omitted by default;
- reviewer independence remains intact;
- stale candidate invalidates handoff applicability without deleting history;
- a reconstructed actor handoff exposes reconstruction-loss/omission/unavailable truth and cannot imply full context transfer.

### Regression and platform gates

Every implementation candidate must run repository `quality` plus the applicable existing Spec 003/006/007 regression/security/platform workflows for the exact changed behavior.

Platform claims remain domain-specific:

- native Windows/ConPTY only when directly exercised;
- real WSL2 only when directly exercised;
- Linux PTY only when directly exercised;
- macOS PTY only when directly exercised;
- persistence/domain-only behavior proven on a portable deterministic fixture is not automatically a native runtime-resume claim on every platform.

## Requirement-to-Implementation Map

| Spec 008 contract | Planned seam | Primary evidence |
| --- | --- | --- |
| FR-001..FR-009 | workflow/stage domain + canonical context validator | pure identity/lifecycle + persistence fixtures |
| FR-010..FR-017 | artifact baseline evaluator | candidate/artifact movement fixtures |
| FR-018..FR-026 | actor/continuation adapter over Spec 006 | continuation/nonclaim fixtures |
| FR-027..FR-034 | finite retry/no-progress controller | retry/exhaustion/non-idempotent fixtures |
| FR-035..FR-043 | append-only decision ledger | schema immutability + lineage/replay fixtures |
| FR-044..FR-050 | deterministic reviewer handoff projection | independence/context fixtures |
| FR-051..FR-060 | workflow schema validator + recovery/redaction | migration/corruption/redaction fixtures |
| FR-061..FR-067 | status/resume/why-blocked projections | deterministic projection fixtures |
| FR-068..FR-078 | governance/platform/scope boundaries | negative scope + exact-candidate/platform gates |
| SC-001..SC-015 | combined deterministic acceptance campaign | focused integration/adversarial fixtures |
| SC-016..SC-020 | final regression/review/scope/platform/history reconciliation | exact final candidate + final reconciliation artifact |

## Implementation Slice Strategy

Tasks should authorize small dependency-ordered slices. This Plan does not itself authorize any slice.

Recommended sequence:

1. workflow/stage domain types, canonical workspace/workstream validator, and complete transition matrix;
2. existing-SQLite schema/migration qualification and persistent workflow/stage identity;
3. artifact/candidate baseline storage and freshness evaluator;
4. actor binding + truthful continuation/reconstruction adapter over accepted Spec 006 seams;
5. finite retry/no-progress + ambiguous side-effect recovery rules;
6. append-only decision ledger and replay/supersession protections;
7. deterministic handoff + status/resume/why-blocked projections;
8. durable-state schema validation, corruption/restart recovery, and redaction/completeness campaign;
9. narrow CLI integration over the qualified domain/projection seams;
10. workbench/TUI read-only projection integration only if still necessary after CLI qualification;
11. cross-platform/applicable Spec 003/006/007 regression qualification;
12. adversarial forged-truth/stale-candidate/replay/recovery campaign;
13. final Spec 008 reconciliation and exact-candidate acceptance.

Tasks may split a recommended slice further for reviewability. They may not combine later authority, add a dependency opportunistically, or skip a predecessor whose invariant the later slice relies on.

## Dependency Decision Record

No dependency changes are required or selected.

Existing direct dependencies already cover the planned implementation needs:

- `rusqlite = "=0.40.2"` with bundled SQLite: existing durable local relational state;
- `serde` / `serde_json`: existing bounded structured serialization where required;
- `sha2`: existing content/candidate-supporting digest primitive where a Task proves hashing is appropriate;
- existing Git/process/runtime/terminal modules: current exact candidate, evidence, runtime/native, and platform observations.

Plan decision:

```text
NEW_DIRECT_DEPENDENCIES=0
NEW_ASYNC_RUNTIME=NO
NEW_DATABASE=NO
NEW_ORM=NO
NEW_WORKFLOW_ENGINE=NO
```

If a later Task discovers a genuinely missing capability, it must stop and obtain an explicit accepted amendment/dependency qualification rather than silently adding a crate.

## Failure and Recovery Semantics

Fail closed whenever canonical truth is insufficient.

Examples:

- unknown workflow/stage -> explicit not-found;
- workspace/workstream mismatch -> identity failure;
- missing/stale/ambiguous baseline -> stale/blocked/not-applicable;
- invalid transition -> rejected with no state mutation;
- retry budget exhausted -> blocked;
- ambiguous non-idempotent effect -> recovery required;
- stale actor/runtime binding -> unavailable/ownership-lost/reconstruction-required as applicable;
- corrupt/unsupported workflow state -> recovery required while preserving existing data;
- candidate movement -> prior candidate-bound evidence/review/decision applicability stale;
- reviewer handoff with missing required evidence -> incomplete/blocked, not a clean handoff;
- forged/stale/superseded/`AGENT_REPORTED` cancellation decisions -> cannot transition a stage to `CANCELLED` or satisfy workflow completion;
- workflow/stage completion with verification pending -> completion remains separate from verification;
- human acceptance missing -> never implied by workflow state.

## Security / Privacy Posture

- workflow state is local Winds-owned state, not a trust grant to repository content or agent output;
- terminal/model output is untrusted and may contain secrets or prompt injection;
- durable workflow records store the minimum structured state and references needed for truth;
- no provider credential brokerage or credential duplication;
- no background listener/socket/server;
- no remote continuation/control plane;
- no automatic external-open/browser action;
- no arbitrary shell execution is granted by workflow transitions/decisions;
- no decision record changes human authority merely because its type/name resembles approval;
- repository paths are context, not authority;
- corrupted state is preserved rather than destructively rewritten into a clean state.

## Evidence Integrity

Every acceptance artifact must bind to exact candidate state. At minimum record commit/tree plus command/runner/platform where applicable.

No evidence may be current if:

- HEAD changed after it was produced;
- the tested tree differs;
- the required candidate/artifact baseline moved;
- platform/domain differs from the claim;
- result is terminal/agent prose rather than the accepted Winds/CI observation;
- review applies to an older candidate;
- a historical failed/reverted/stale attempt is merely relabelled by later success.

Final reconciliation must keep historical failures and superseded decisions inspectable and must not collapse `COMPLETED`, `VERIFIED`, `HUMAN_ACCEPTED`, and `LANDED` into one status.

## Ponytail / YAGNI Boundaries

Explicitly reject during the first Spec 008 implementation program unless an accepted amendment changes scope:

- daemon/process supervisor architecture;
- IPC/socket/RPC/local server;
- remote/mobile/team workflow continuation;
- provider/model router or credential broker;
- browser/CDP runtime;
- generic workflow engine/framework;
- plugin SDK/marketplace;
- MCP/ACP/A2A expansion;
- learning/skill optimization/training/RL;
- semantic/vector/RAG memory;
- second database/state store;
- new persistence/ORM dependency;
- event-sourcing framework;
- message queue;
- distributed lock/lease service;
- generic DAG scheduler;
- persisted full transcript/model prompt history;
- speculative handoff table;
- automatic candidate winner selection;
- automatic Git mutation/PR creation/landing.

Prefer closed enums, concrete Store methods, explicit SQL constraints/triggers, deterministic pure transition functions, and direct projections until two proven use cases justify a more generic abstraction.

## External Research Provenance

The accepted Spec 008 entry research, including LoopForge and SkillHone comparisons, informs product gaps only. No donor source, runtime, dependency, workflow, template, or implementation code is admitted by this Plan.

Implementation must remain independently designed against canonical Winds Spec/Plan and compatible existing dependencies. Any future donor-code/dependency admission requires a separately accepted provenance/license/security/YAGNI decision.

## Plan Acceptance Gate

This Plan can land only if all are true on its exact final candidate:

- canonical base is the post-Spec-008-spec merge `main`;
- changed scope is planning/documentation only;
- no `Cargo.toml`, lockfile, migration, source, workflow semantic, runtime, provider, browser, daemon, IPC, remote, learning, plugin, or automatic landing change occurs;
- the existing SQLite/rusqlite choice is proven to reuse accepted repository infrastructure and introduces no new dependency;
- no second canonical workspace/workstream/task identity is created;
- the plan provides a plausible implementation/evidence path for FR-001..FR-078 and SC-001..SC-020;
- Spec 006 live-runtime nonclaims remain unchanged;
- exact-head repository `quality` succeeds;
- correctness/safety/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads remain;
- exact main/base/head/tree/scope/ruleset/mergeability are reconciled immediately before guarded merge;
- guarded expected-head merge succeeds;
- post-merge canonical main/tree and every actually-triggered applicable push check are verified.

## Downstream Authorization

Canonical acceptance and post-merge verification of this Plan authorize **Spec 008 Tasks creation only**.

They do not authorize implementation, migration landing, source changes, runtime changes, provider/browser execution, daemon/IPC, remote execution, learning, donor code admission, new dependencies, automatic landing, or later specifications.

Only after this Plan lands and is post-merge verified may repository truth state:

```text
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=CLOSED_CANONICAL
SPEC_008_TASKS_AUTHORIZED=YES
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```
