# Tasks: Resumable Workflow & Decision Ledger

## Canonical Inputs

- Constitution 1.1.0: canonical.
- Spec 008 specification: canonical.
- Spec 008 Plan: canonical via PR #142.
- Tasks base: `7993b496393beac282cf2ca611a2021e268ef2f6`.
- Canonical Plan tree: `2c609af3b8e6b0aa7dd1601ff9623ad26b917438`.
- Canonical T057 repair prerequisite: PR #146, merge `cd94b9940fa1ecfa390838a6e1cd7c4cf0c593aa`.

This file decomposes Spec 008 into independently reviewable, dependency-ordered implementation slices. It does not itself add a dependency, migration, source/runtime behavior, provider/browser route, daemon/IPC path, remote execution, learning subsystem, donor code, or automatic landing behavior.

At this Tasks base:

```text
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=CLOSED_CANONICAL
SPEC_008_TASKS_AUTHORIZED=YES
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```

Canonical acceptance and post-merge verification of this file authorize **T101 only**. Every later task remains unauthorized until its exact predecessor closes canonically.

The historical first Plan failure remains material evidence:

```text
HISTORICAL_PLAN_FAILURE_RUN=34395694562
HISTORICAL_PLAN_FAILURE_CLASS=MATERIAL
HISTORICAL_PLAN_FAILURE_FLAKE=NO
```

Spec 006 live-runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Nothing in Spec 008 upgrades those separate lanes.

## Global Rules

1. Execute tasks strictly in dependency order. Only the next dependency-satisfied task is authorized.
2. Every implementation task starts from exact then-current canonical `main`; live repository/GitHub truth overrides this file's recorded historical hashes when main legitimately moves.
3. HEAD/TREE movement invalidates candidate-bound CI, review, benchmark, and merge-ready evidence. Requalify the new exact candidate rather than inheriting stale evidence.
4. Preserve canonical workspace/workstream identity. `WorkflowRun`, `StageRun`, Winds session, runtime/native identity, display labels, paths, PIDs, transcripts, and UI state remain distinct.
5. Preserve `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`; stage/workflow completion never implies candidate verification, human acceptance, or landing.
6. No daemon, persistent owner, server, socket, HTTP/SSE/WebSocket surface, new IPC/RPC/control protocol, remote execution/control plane, browser/CDP runtime, provider/model router, credential broker, MCP/ACP/A2A expansion, generic workflow/plugin runtime, learning/training/RL, semantic/vector/RAG memory, automatic winner selection, or automatic Git landing.
7. No new direct dependency is authorized by T101-T113. Existing `rusqlite`, `serde`, `serde_json`, `sha2`, Git/process/runtime/terminal modules may be reused only within their accepted roles. A genuinely missing dependency stops the task for a canonical Plan/Tasks amendment and exact dependency qualification.
8. Existing `winds.db` is the only selected durable store. No second database, ORM, persistence layer, state directory, event-sourcing framework, queue, distributed lock/lease service, or generic DAG scheduler.
9. Durable workflow state must fail closed on malformed, ambiguous, stale, incompatible, partial, or corrupt truth. The only available historical database/evidence must not be deleted or silently replaced by a clean state.
10. No raw provider-private state, credentials, tokens, full prompts/responses, terminal transcript, environment dump, or arbitrary repository content may be introduced as default durable workflow payload.
11. Candidate/artifact/evidence applicability is exact and freshness-sensitive. Candidate movement invalidates stale candidate-bound applicability without deleting history.
12. Retry is explicit and finite. Automatic retry remains disabled; ambiguous possibly-completed non-idempotent effects require reconciliation rather than silent repetition.
13. Decision history is append-only at the semantic level. A decision record alone cannot create execution, verification, human-acceptance, cancellation, merge, or landing authority.
14. Status/resume/why-blocked/reviewer handoff are projections from canonical state, not independently mutable authority stores.
15. Platform claims remain domain-specific and require direct applicable evidence. Native Windows, WSL2, Linux PTY, and macOS PTY proof do not substitute for each other.
16. Implementation tasks may make the smallest `src/main.rs` module-registration or exact CLI dispatch edit needed to compile/execute that task's explicitly authorized surface. No unrelated CLI behavior is authorized by this rule.
17. Every new `src/tNNN_*.rs` focused test module must be registered and demonstrably executed. File existence is not test evidence.
18. Historical failed/rejected/reverted/stale attempts and decisions remain inspectable. Later success does not rewrite their historical meaning.
19. No task may relax the Plan's lifecycle, retry, freshness, redaction, reconstruction, evidence, authority, platform, or YAGNI constraints. Relaxation requires a canonical Plan/Spec amendment first.
20. Each task closes only after expected-head guarded landing and post-merge verification on canonical `main`. Successor authority is asserted only after that closure.
21. T102 owns the single Plan-selected Spec 008 schema migration `0010_resumable_workflow_ledger.sql`. After T102 lands canonically, that migration is immutable historical schema. T103-T113 MUST NOT edit it; a later proven schema defect or missing canonical constraint requires an explicit Tasks amendment authorizing a new next-numbered migration, never mutation of the landed migration.

## Standard Acceptance Gate

Every implementation task requires on its exact final candidate:

- repository `quality` = SUCCESS on every platform job actually defined by that workflow;
- focused deterministic tests for the task-authorized surface, with proof each new focused module executed;
- applicable existing Spec 003/006/007 regression/security/platform checks remain green;
- platform evidence only for directly claimed platform/runtime behavior;
- author correctness/safety/governance/evidence-integrity review;
- Ponytail/YAGNI review;
- fresh independent substantive review by a different reviewer/model/human on the exact final candidate;
- zero unresolved material findings and zero unresolved material review threads;
- exact changed-file/scope/dependency/authority reconciliation;
- exact current main/base/head/tree/ruleset/mergeability reconciliation immediately before landing;
- expected-head guarded normal merge;
- merge commit/tree/ordered parents/signature verification;
- every actually-triggered applicable post-merge push workflow checked before successor authority is asserted.

Documentation-only or evidence-only tasks use the same review/race/landing discipline. Focused implementation tests are N/A only when the task proves it changes no implementation surface.

## Authorization Ladder

Canonical acceptance of this file authorizes T101 only.

```text
T101 -> T102 -> T103 -> T104 -> T105 -> T106 -> T107
     -> T108 -> T109 -> T110 -> T111 -> T112 -> T113
```

A task closes only after guarded landing and post-merge verification. Closing T113 authorizes no later Spec 008 implementation phase.

## Requirement Coverage Map

| Canonical contract | Primary task path | Final evidence path |
| --- | --- | --- |
| FR-001..FR-009 | T101 domain truth -> T102 persistence | T112 adversarial -> T113 reconciliation |
| FR-010..FR-017 | T103 exact baseline/freshness | T112 -> T113 |
| FR-018..FR-026 | T104 continuation/reconstruction | T111 platform/nonclaim + T112 -> T113 |
| FR-027..FR-034 | T105 retry/no-progress/recovery | T112 -> T113 |
| FR-035..FR-043 | T106 append-only decisions | T112 -> T113 |
| FR-044..FR-050 | T107 reviewer handoff | T112 -> T113 |
| FR-051..FR-060 | T102 schema substrate -> T108 validation/redaction/recovery | T112 -> T113 |
| FR-061..FR-067 | T107 pure projections -> T109 CLI | T110 necessity gate -> T112 -> T113 |
| FR-068..FR-078 | Global Rules + T111 inherited/platform qualification | T112 -> T113 |
| SC-001..SC-015 | T101..T109 focused deterministic acceptance | T112 combined adversarial campaign |
| SC-016 | T111 exact regression/platform qualification | T113 |
| SC-017 | each task Standard Acceptance Gate | T113 final exact review stack |
| SC-018 | Global Rules/YAGNI + T110/T111 | T113 final scope reconciliation |
| SC-019 | T111 direct platform-only evidence | T113 |
| SC-020 | Global Rule 18 + every task history preservation | T112 -> T113 |

No row grants authority to skip a dependency. The map identifies the primary implementation/evidence path only; each task remains bound by the complete canonical Spec and Plan.

---

## Phase 1 — Canonical Workflow Domain

### [x] T101 — Workflow/stage domain, canonical context validation, and transition matrix

**Purpose**: establish the smallest pure deterministic workflow/stage truth model before persistence or CLI integration.

**Authorized paths**:
- `src/workflow.rs`
- `src/t101_workflow_domain_tests.rs`
- `src/main.rs` only for module/test registration under Global Rule 16

**Required implementation**:
- closed `WorkflowRun`/`StageRun` identity and lifecycle types;
- stable workflow identity bound to canonical `workspace_id` + `workstream_id` inputs;
- unique stage-attempt identity with logical `stage_key`, attempt ordinal, and optional predecessor lineage;
- the exact first-slice lifecycle states selected by the Plan;
- one pure legal-transition validator implementing the complete Plan transition matrix;
- deterministic duplicate/idempotent no-change behavior only for exact matching operation/preconditions;
- explicit invalid-transition failure without mutation;
- pure workflow completion eligibility that keeps `COMPLETED`, `VERIFIED`, `HUMAN_ACCEPTED`, and `LANDED` separate;
- no persistence, terminal child, provider/runtime action, or CLI mutation.

**Acceptance**:
- exhaustive legal and illegal transition matrix tests;
- display-label/path/session/runtime/PID coincidence cannot define identity;
- terminal stage attempts cannot be resurrected as the same attempt;
- retry/reconstruction/reassignment material new work requires a distinct attempt identity;
- `AGENT_REPORTED`/decision-shaped input cannot create cancellation or completion authority;
- workflow completion eligibility rejects unresolved latest logical-stage attempts and remains procedural only;
- no new dependency or persistence surface;
- focused T101 tests registered and executed.

**Depends on**: canonical Tasks acceptance. **Closes to authorize**: T102.

---

## Phase 2 — Existing SQLite Persistence

### [x] T102 — Spec 008 schema qualification and persistent workflow/stage identity

**Purpose**: qualify the complete minimal Plan-selected Spec 008 SQLite schema once, while exposing only canonical workflow/stage identity and lifecycle Store behavior in this slice.

**Authorized paths**:
- `migrations/0010_resumable_workflow_ledger.sql`
- `src/store.rs`
- `src/workflow.rs`
- `src/t102_workflow_store_tests.rs`
- `src/main.rs` only for module/test registration
- a narrowly scoped Spec 008 T102 evidence artifact if required

**Required implementation**:
- one idempotent `0010_resumable_workflow_ledger.sql` following existing migration conventions and creating the complete minimal Plan-selected Spec 008 schema: `workflow_runs`, `workflow_stage_runs`, `workflow_artifact_baselines`, `workflow_actor_bindings`, `workflow_reconstruction_reports`, and `workflow_decisions`;
- all schema columns, indexes, foreign keys, uniqueness constraints, append-only guards, hierarchy constraints, and immutable-history relationships already required by the canonical Plan are defined now, even though T102 exposes Store behavior only for workflow/stage identity and lifecycle;
- stage rows include the Plan-selected failure/checkpoint/material-progress fields needed by later retry semantics, while T105 remains the first task authorized to use them behaviorally;
- reconstruction and decision tables are inert persistence structure until T104/T106 respectively authorize their Store/domain behavior;
- database-enforced workspace/workstream hierarchy validation using a 0009-style fail-closed relation/trigger or equally strong existing-SQLite mechanism;
- uniqueness of `(workflow_run_id, stage_key, attempt_ordinal)`;
- predecessor lineage constrained to the same workflow/logical-stage lineage unless no cross-stage relation is selected;
- schema-integrity validation before workflow state is treated as applicable;
- atomic creation/transition compare-and-set operations;
- no database write lock held across external process/human/network waits;
- no T103-T113 behavior is activated merely because its Plan-selected table/column exists.

**Acceptance**:
- migration applies and reapplies idempotently and proves the complete Plan-selected table/index/trigger/constraint inventory;
- partial/missing/modified required schema objects fail closed;
- orphan or mismatched workspace/workstream workflows are rejected at database/application boundaries;
- duplicate stage ordinal/invalid predecessor lineage is rejected;
- restart preserves stable workflow/stage identities and lifecycle history;
- invalid CAS loses cleanly without overwriting newer state;
- existing WAL/foreign-key/store regression behavior remains green;
- no second database or dependency;
- focused T102 tests registered and executed.

**Depends on**: T101 `CLOSED_CANONICAL`. **Closes to authorize**: T103.

---

## Phase 3 — Exact Artifact and Candidate Baselines

### [x] T103 — Artifact/candidate baseline persistence and freshness evaluator

**Purpose**: make prepared-stage applicability depend on exact structured input/candidate identity rather than paths, labels, or stale artifacts.

**Authorized paths**:
- `src/store.rs`
- `src/workflow.rs`
- `src/t103_workflow_baseline_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- durable artifact-baseline identity bound to exact stage attempt;
- closed baseline kinds selected from the Plan's concrete first-slice references;
- candidate OID + tree handling where candidate-bound;
- deterministic freshness/applicability evaluator;
- stale/missing/ambiguous baseline truth that blocks applicability without deleting history;
- no arbitrary path/name similarity as freshness proof;
- no repository payload copying when stable canonical references suffice.

**Acceptance**:
- candidate A evidence/baselines do not apply to moved candidate B;
- path/name-equivalent older artifacts do not satisfy a new stage;
- missing or ambiguous required baseline fails closed and identifies the blocker;
- historical stale baselines remain inspectable;
- exact identical canonical inputs produce deterministic identical freshness results;
- no automatic Git mutation or candidate selection;
- focused T103 tests registered and executed.

**Depends on**: T102 `CLOSED_CANONICAL`. **Closes to authorize**: T104.

---

## Phase 4 — Truthful Actor Continuation and Reconstruction

### [x] T104 — Actor binding and reconstruction truth over accepted Spec 006 seams

**Purpose**: bind stage attempts to existing Winds/runtime identity truth without creating a competing runtime owner or upgrading live-runtime nonclaims.

**Authorized paths**:
- `src/workflow.rs`
- `src/agentic_runtime.rs` only for the smallest read/revalidation adapter required to reuse accepted Spec 006 truth
- `src/store.rs`
- `src/t104_workflow_continuation_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- closed continuation classification required by Plan (`RESUMED`, `RECONSTRUCTED`, `OWNERSHIP_LOST`, `UNAVAILABLE`, `UNPROVEN` as applicable);
- exact stage actor binding to existing Winds session/runtime binding identities where applicable;
- `RESUMED` only from already-accepted revalidated Spec 006 native/runtime proof;
- `RECONSTRUCTED` only with a new binding/attempt relationship and one atomic immutable bounded reconstruction report;
- exact first-slice reconstruction category, transfer-state, source/provenance, and completeness/content-state sets;
- provider-private state represented only as unavailable/no-longer-transferable when not provable, never persisted as hidden payload;
- side-effect-free reconstruction preview using the same deterministic context evaluation as the later action;
- no provider call, new runtime owner, daemon, IPC, or live-runtime acceptance upgrade.

**Acceptance**:
- exact accepted runtime mapping can produce `RESUMED` only where current Spec 006 truth permits it;
- process restart alone never proves live ownership;
- new actor initialized from Winds canonical state is `RECONSTRUCTED`, never `RESUMED`;
- absent/malformed/incomplete/mismatched reconstruction report makes a reconstructed binding non-applicable;
- stale/ambiguous mapping fails closed to unavailable/ownership-lost/reconstruction-required truth;
- all required reconstruction categories appear exactly once with explicit transfer/completeness truth;
- live-runtime nonclaims remain unchanged;
- focused T104 tests registered and executed.

**Depends on**: T103 `CLOSED_CANONICAL`. **Closes to authorize**: T105.

---

## Phase 5 — Finite Retry and Ambiguous Side Effects

### [x] T105 — Retry/no-progress lineage and explicit recovery

**Purpose**: add bounded explicit retry semantics without background loops or ambiguous side-effect repetition.

**Authorized paths**:
- `src/workflow.rs`
- `src/store.rs`
- `src/t105_workflow_retry_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:

```text
AUTOMATIC_RETRY=DISABLED
MAX_CONSECUTIVE_NO_PROGRESS_RETRIES=2
SECOND_NO_PROGRESS_RETRY_FAILURE=RETRY_BUDGET_EXHAUSTED
RETRY_AFTER_BUDGET_EXHAUSTION=REJECTED_WITHOUT_NEW_STAGE_RUN
AMBIGUOUS_NON_IDEMPOTENT_RETRY=FORBIDDEN_WITHOUT_EXPLICIT_RECONCILIATION
```

- normalized failure class, checkpoint identity where applicable, and deterministic material-progress basis;
- explicit same-lineage retry budget and predecessor history;
- changed candidate/artifact alone is insufficient progress when the failure/checkpoint remains materially unresolved;
- ambiguous possibly-completed non-idempotent effect moves to `RECOVERY_REQUIRED`/explicit reconciliation;
- successful later retry preserves earlier failures.

**Acceptance**:
- no automatic/background retry path exists;
- first and second no-progress retries are distinct attempts; the second no-progress retry failure records budget exhaustion;
- a third same-lineage retry is rejected without creating an attempt until a new accepted recovery/re-preparation basis exists;
- progress comparator is deterministic for identical canonical inputs;
- ambiguous side-effect fixture never silently re-executes;
- failed predecessor history remains inspectable after eventual success;
- focused T105 tests registered and executed.

**Depends on**: T104 `CLOSED_CANONICAL`. **Closes to authorize**: T106.

---

## Phase 6 — Append-Only Decision Ledger

### [x] T106 — Canonical decision persistence, lineage, replay, and authority ceiling

**Purpose**: persist material decisions as immutable historical facts without granting lifecycle or human authority by record existence.

**Authorized paths**:
- `src/workflow.rs`
- `src/store.rs`
- `src/t106_workflow_decision_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- stable decision identity with workflow/stage binding where applicable;
- explicit source class, authority class, decision type/result, candidate/evidence references, safe rationale/content state, predecessor/supersession lineage;
- database-level update/delete prevention or equivalently strong accepted append-only enforcement;
- idempotent/rejected replay semantics preventing duplicate canonical truth;
- current applicability evaluator for candidate/evidence-bound decisions;
- no decision row by itself may cancel a stage, satisfy workflow completion, create `HUMAN_DECIDED`, verify a candidate, select a winner, merge, or land.

**Acceptance**:
- accepted/rejected/reverted/stale/superseded decisions remain queryable;
- semantic change creates a new decision record rather than mutating historical meaning;
- self/cyclic/impossible predecessor/supersession lineage fails closed;
- forged agent/terminal/UI approval strings cannot create human authority;
- candidate movement marks affected decision applicability stale without deleting history;
- duplicate replay creates zero duplicate canonical decision truth;
- focused T106 tests registered and executed.

**Depends on**: T105 `CLOSED_CANONICAL`. **Closes to authorize**: T107.

---

## Phase 7 — Deterministic Operator and Reviewer Projections

### [x] T107 — Status, resume-preview, why-blocked, and reviewer-handoff projections

**Purpose**: expose canonical workflow truth without persisting a competing presentation authority.

**Authorized paths**:
- `src/workflow.rs`
- `src/workflow_projection.rs`
- `src/t107_workflow_projection_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- deterministic `WorkflowStatusProjection`;
- side-effect-free `ResumePreviewProjection`;
- deterministic `WhyBlockedProjection` with explicit unknown/unavailable truth when canonical cause is not provable;
- deterministic `ReviewerHandoffProjection` containing exact identity/candidate/artifact/evidence/authority context while excluding builder persuasion/confidence by default;
- reconstructed actor projections surface persisted reconstruction report loss/transfer/completeness truth;
- every projection revalidates canonical workspace/workstream binding before becoming applicable;
- no new handoff/status persistence table.

**Acceptance**:
- every accepted lifecycle fixture projects exact workflow/stage/attempt identity and source-labelled truth;
- resume preview distinguishes proven resume, reconstruction, reassignment, unavailable/blocked, and ownership-loss paths without executing them;
- why-blocked never invents a blocker;
- stale candidate/baseline/evidence invalidates projection applicability as required;
- reviewer handoff excludes persuasive verdict/confidence text and includes material omission/reconstruction facts;
- inspection is non-mutating;
- focused T107 tests registered and executed.

**Depends on**: T106 `CLOSED_CANONICAL`. **Closes to authorize**: T108.

---

## Phase 8 — Durable-State Validation, Corruption, and Redaction

### [x] T108 — Workflow schema integrity, recovery, and completeness campaign

**Purpose**: close the durable-state safety boundary before exposing mutating CLI commands.

**Authorized paths**:
- `src/workflow.rs`
- `src/store.rs`
- `src/t108_workflow_recovery_tests.rs`
- `src/main.rs` only for module/test registration
- narrowly scoped T108 deterministic evidence artifacts if needed

**Required implementation/evidence**:
- logical schema version beginning at 1;
- validation of required identifiers, supported enums/version, canonical work-context relationship, parent/child lineage, candidate format, predecessor/supersession integrity, content-state markers, and reconstructed-binding report completeness;
- physical database-open/corruption failure preserves the existing database and fails explicitly;
- partial required schema never masquerades as initialized;
- protected content is redacted/omitted before workflow persistence where policy requires it;
- `FULL`, `REDACTED`, `OMITTED`, `UNAVAILABLE` completeness truth remains explicit;
- missing/redacted required evidence cannot satisfy complete-evidence, verification, or acceptance claims;
- no secret/highly-sensitive hash is added solely to prove omission without separate authorization.

**Acceptance**:
- malformed/partial/unsupported-version/corrupt fixtures fail closed without silent clean reinitialization;
- historical database bytes/state are preserved or quarantined as defined without destructive repair;
- restart/migration preserves stable workflow/stage/decision identity and failed/stale history;
- redaction fixtures persist no forbidden raw payload and retain explicit loss markers;
- unrelated evidence/authority/candidate truth is unchanged by redaction;
- focused T108 tests registered and executed.

**Depends on**: T107 `CLOSED_CANONICAL`. **Closes to authorize**: T109.

---

## Phase 9 — Narrow CLI Integration

### [x] T109 — Existing-style CLI workflow operations over qualified domain seams

**Purpose**: expose only the first-slice user operations required by the Plan through the existing Winds CLI without creating a new shell or generic workflow runtime.

**Authorized paths**:
- `src/workflow_cli.rs`
- `src/main.rs` for exact workflow CLI registration/dispatch only
- `src/workflow.rs`
- `src/workflow_projection.rs`
- `src/store.rs` only for already-qualified workflow methods needed by CLI dispatch
- `src/t109_workflow_cli_tests.rs`
- narrowly scoped CLI fixture/evidence artifacts if needed

**Required user capabilities**:
- create/open a workflow under exact canonical workspace + workstream;
- prepare/start one stage attempt;
- record only explicitly authorized lifecycle transitions;
- inspect status;
- inspect resume/reconstruction preview without side effects;
- inspect why-blocked;
- record/query decisions through accepted authority/source inputs;
- generate reviewer handoff;
- request explicit retry/recovery that creates new attempt lineage where authorized.

**Acceptance**:
- command spelling is collision-checked against existing CLI surface;
- all mutating commands validate exact canonical identity/preconditions before Store mutation;
- inspection commands are side-effect-free;
- terminal/agent strings cannot promote verification/human acceptance/cancellation/landing;
- no provider/browser/network call, daemon, IPC, automatic Git mutation, or automatic retry;
- stable machine-readable output is bounded and source-labelled where provided;
- focused T109 tests registered and executed.

**Depends on**: T108 `CLOSED_CANONICAL`. **Closes to authorize**: T110.

---

## Phase 10 — TUI Necessity Gate

### [x] T110 — Workbench/TUI necessity decision and read-only integration only if proven necessary

**Purpose**: enforce the Plan's YAGNI condition that TUI integration is not built unless the qualified CLI is demonstrably insufficient for a current required Spec 008 capability.

**Authorized paths by default**:
- `specs/008-resumable-workflow-decision-ledger/t110-tui-necessity.md`
- no production/runtime source

**Authority ceiling**:
- T110 is documentation/evidence-only. It authorizes no workbench or other production source edit.
- if T110 evidence proves a concrete unmet current Spec 008 requirement that cannot be satisfied by the qualified CLI/projection seams, T110 MUST stop open and obtain a canonical Tasks amendment defining a new exact source slice before any TUI implementation begins.
- absent such proven need, T110 closes as `TUI_NOT_REQUIRED_FIRST_SLICE` with no production change.

**Acceptance**:
- every P1 operator capability is reconciled against T109 CLI/projections;
- presentation preference alone is not sufficient need;
- no second workflow authority, durable UI state, or mutation path is introduced;
- default/no-need result is documentation-only `TUI_NOT_REQUIRED_FIRST_SLICE`;
- any genuinely required read-only integration remains unauthorized until a canonical Tasks amendment creates an explicit later source task; presentation preference alone cannot trigger that amendment.

**Depends on**: T109 `CLOSED_CANONICAL`. **Closes to authorize**: T111.

---

## Phase 11 — Applicable Platform and Inherited Regression Qualification

### [x] T111 — Spec 003/006/007 regression and platform-bound qualification

**Purpose**: prove the full canonical Spec 008 implementation has not weakened inherited terminal, identity, evidence, authority, workbench, or platform truths.

**Authorized paths**:
- focused `src/t111_workflow_regression_tests.rs` if a deterministic aggregation fixture is necessary
- `src/main.rs` only for focused-test registration
- Spec 008 T111 evidence artifacts
- minimal existing implementation repair only if a newly proven regression requires a forward-only fix; any repair moves the candidate and restarts the task's exact-head qualification

**Required evidence**:
- repository `quality`;
- directly applicable Spec 003 terminal lifecycle/recovery tests;
- Spec 006 identity/continuity/runtime/evidence/authority regressions;
- Spec 007 workbench truth regressions only where Spec 008 integration can affect shared surfaces;
- native Windows/ConPTY only if directly affected/claimed;
- real WSL2 only if directly affected/claimed;
- Linux/macOS PTY only if directly affected/claimed;
- persistence/domain-only behavior is not relabelled native resume proof.

**Acceptance**:
- no inherited canonical invariant is weakened;
- any unexercised platform/runtime claim remains explicitly unclaimed;
- Spec 006 live-runtime nonclaims remain unchanged;
- all actually applicable workflows complete on the exact final candidate;
- focused T111 tests, if added, are registered and executed.

**Depends on**: T110 `CLOSED_CANONICAL`. **Closes to authorize**: T112.

---

## Phase 12 — Adversarial Workflow Truth Campaign

### [x] T112 — Forged truth, stale candidate, replay, corruption, and recovery campaign

**Purpose**: attack the complete Spec 008 truth model before final acceptance.

**Authorized paths**:
- `src/t112_workflow_adversarial_tests.rs`
- `src/main.rs` only for focused-test registration
- Spec 008 T112 evidence artifacts
- minimal existing Spec 008 implementation repair only for forward-only fixes proven necessary by the campaign

**Campaign coverage**:
- duplicate/case/Unicode workflow/stage display labels;
- stale/missing/mismatched workspace/workstream bindings;
- invalid/cyclic predecessor and decision supersession lineage;
- candidate movement after baseline/evidence/review/decision capture;
- path/name-equivalent stale artifact replay;
- forged terminal/agent `PASS`, `VERIFIED`, `ACCEPTED`, `HUMAN_DECIDED`, cancellation, completion, and decision-shaped JSON;
- duplicate/replayed transition/completion/decision operations;
- retry budget exhaustion and attempted silent reset;
- ambiguous non-idempotent completion;
- stale/reused runtime/native identifiers and restart without live proof;
- missing/malformed/incomplete reconstruction report;
- provider-private/unavailable context claims;
- malformed/partial/unknown-version/corrupt durable state;
- redacted/omitted required evidence presented as complete;
- projection/handoff persuasion or stale-review contamination;
- historical failure/reverted/stale evidence after later success.

**Acceptance**:
- zero false identity attachment, false `RESUMED`, false `VERIFIED`, false `HUMAN_ACCEPTED`, false completion/cancellation authority, silent retry, destructive recovery, stale evidence promotion, or automatic landing;
- no newly discovered material defect remains unresolved;
- full repository regression and applicable platform workflows green on the exact final candidate;
- focused T112 tests registered and executed.

**Depends on**: T111 `CLOSED_CANONICAL`. **Closes to authorize**: T113.

---

## Phase 13 — Final Spec 008 Reconciliation and Closeout

### [x] T113 — Spec 008 final acceptance, evidence reconciliation, and program closeout

**Checked-state note**: this T113 check records the final closeout candidate. It becomes `CLOSED_CANONICAL` only after this exact candidate lands by guarded normal merge and every actually-triggered post-merge push check succeeds.

**Purpose**: reconcile every Spec 008 requirement/success criterion against canonical implementation evidence and close only what is actually proven.

**Authorized paths**:
- this `tasks.md` for final checked-state reconciliation;
- `specs/008-resumable-workflow-decision-ledger/t113-final-reconciliation.md`;
- focused Spec 008 acceptance/evidence artifacts following repository precedent;
- README/docs corrections only for claims proven by canonical evidence;
- no production/runtime/dependency behavior change.

**Acceptance**:
- T101..T112 reconciled against exact canonical merges and post-merge verification;
- FR-001..FR-078 each classified as `PROVEN_DETERMINISTIC`, `PROVEN_PLATFORM_BOUND`, `PROVEN_GOVERNANCE_BOUNDARY`, or explicit truthful nonclaim/deferment where the canonical Spec permits it;
- SC-001..SC-020 each reconciled to exact evidence;
- all selected schema objects/migrations and exact existing dependency use reconciled; no unauthorized dependency or durable store remains;
- candidate/artifact/evidence freshness, decision append-only lineage, bounded retry, reconstruction loss, redaction completeness, and corrupt-state preservation all remain proven;
- platform claims remain limited to directly exercised domains;
- Spec 006 live-runtime nonclaims remain separate and unchanged unless independently governed elsewhere;
- no older-head CI/review evidence represented as current;
- historical failed/rejected/reverted/stale evidence remains material and inspectable, including Plan failure run `34395694562`;
- final exact implementation state has repository quality, applicable regression/platform/security evidence, author review, Ponytail/YAGNI review, and fresh independent substantive review with zero unresolved material findings;
- no daemon/IPC, remote execution, provider/model router, browser runtime, MCP/ACP/A2A expansion, generic plugin/workflow engine, learning subsystem, semantic/vector memory, second database, new dependency, automatic winner, or automatic landing path introduced;
- exact main/base/head/tree/scope/ruleset/mergeability reconciliation before guarded landing;
- guarded normal landing with exact expected head;
- post-merge canonical main/tree and every actually-triggered push check verified.

**Completion state, only if proven after canonical landing**:

```text
T101..T113=CLOSED_CANONICAL
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=CLOSED_CANONICAL
SPEC_008_TASKS=CLOSED_CANONICAL
SPEC_008_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

Closing T113 authorizes no later Spec 008 phase and does not automatically authorize daemon/IPC, remote execution, provider/browser orchestration, learning, generic plugins/workflow engines, new dependencies, automatic landing, or any later specification.

**Depends on**: T112 `CLOSED_CANONICAL`. **Closes to authorize**: no successor implementation task.

## Tasks Acceptance Gate

This Tasks file may land only if all are true on its exact final candidate:

- canonical base is post-Plan `main` `7993b496393beac282cf2ca611a2021e268ef2f6` unless live truth legitimately moves before candidate creation, in which case this file/base metadata must be reconciled forward-only and all candidate-bound qualification restarted;
- changed scope is exactly `specs/008-resumable-workflow-decision-ledger/tasks.md` unless an explicitly justified governance-only correction is needed;
- no implementation, Cargo/lockfile, source, migration, workflow, runtime, provider/browser, daemon/IPC, remote, learning, plugin, ACP/MCP/A2A, or automatic landing change occurs in the Tasks PR;
- no new direct dependency is authorized by T101-T113;
- every FR-001..FR-078 and SC-001..SC-020 has a plausible dependency-ordered implementation/evidence path;
- canonical acceptance authorizes T101 only; T102..T113 remain dependency-blocked;
- Spec 006 live-runtime nonclaims remain unchanged;
- exact-head repository `quality` succeeds;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- final exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- expected-head guarded normal merge;
- canonical merge identity/tree/ordered parents/signature verified;
- every actually-triggered applicable post-merge push check succeeds before T101 authority is asserted.

Only after this Tasks file lands canonically and is post-merge verified may repository truth state:

```text
SPEC_008_TASKS=CLOSED_CANONICAL
SPEC_008_IMPLEMENTATION_AUTHORIZED=T101_ONLY
T101=AUTHORIZED
T102..T113=BLOCKED_BY_DEPENDENCY
```
