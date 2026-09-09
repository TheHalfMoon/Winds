# Winds Roadmap Reconciliation — LoopForge + SkillHone

**Status:** Research-only product/architecture plan. Non-authorizing.

**Prepared:** 2026-09-08

**Canonical Winds base inspected:** `9d6300ff5566a40ba9a90247e164e0761a91fe65`

**Planning branch:** `docs/014-loopforge-skillhone-roadmap-reconciliation`

**Authority firewall:** This document does **not** amend the Constitution, accepted Spec 007 scope, `spec.md`, `plan.md`, `tasks.md`, T095, or any exact-candidate evidence. It does not authorize implementation, provider/model calls, persistent ownership, daemon/IPC, browser control, remote execution, automatic landing, self-modification, training, or learning activation. Current implementation must continue in canonical dependency order. In particular, this planning branch MUST NOT be merged while doing so would invalidate an exact-head candidate gate such as the currently qualified T095 candidate. Any future implementation still requires `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing`.

---

## 1. Research inputs and exact provenance

### Tencent/LoopForge

- URL: <https://github.com/Tencent/LoopForge>
- Exact inspected commit: `09c765286f549624dd95434e1e6ef2249657cbeb`
- License: MIT, with explicit third-party attribution for vendored/adapted `obra/superpowers` material.
- High-value inspected paths:
  - `LICENSE`
  - `THIRD_PARTY_NOTICES.md`
  - `README.md`
  - `skills/devflow/SKILL.md`
  - `skills/devflow/references/workflow-contract.md`
  - `skills/devflow/scripts/workflow_state.py`
  - `skills/devflow/tests/test_regressions.py`

### Tencent/SkillHone

- URL: <https://github.com/Tencent/SkillHone>
- Exact inspected commit: `7d565839fb4dc74f9c77f09ace660e1c0484e048`
- License: MIT.
- High-value inspected paths:
  - `LICENSE`
  - `README.md`
  - `skills/skillhone/SKILL.md`
  - `skills/skillhone-optimization/SKILL.md`
  - `skills/skillhone/references/evaluation.md`
  - `skills/skillhone/references/optim.md`
  - `skills/skillhone/scripts/optim.py`
  - `skills/skillhone/scripts/core/git_ops.py`
  - `skills/skillhone/scripts/core/redaction.py`

### Reuse classification

Both projects are useful research and selective design references. This plan recommends **Winds-authored implementation** for future admitted capabilities rather than transplanting either project's Python orchestration/runtime architecture.

No donor runtime code is admitted by this document.

---

## 2. Executive decision

The new sources expose a missing foundation between Winds' already-proven canonical task/context/authority model and the planned Model Mesh / durable runtime / learning programs.

The missing layer is:

> **Proof-Carrying Resumable Workflow and Decision State**

Winds already has strong primitives for exact candidate identity, context capsules, authority ceilings, execution evidence, reviewer independence, and human decisions. What it does not yet have as a first-class product contract is a deterministic workflow-run state machine that says:

- which stage is active;
- which exact actor/run instance owns that stage;
- which artifacts were prepared for that stage;
- whether those artifacts are fresh or stale;
- which exact upstream facts may flow into the stage;
- how retry/block/recovery state is represented;
- which decisions were made, rejected, superseded, or reverted;
- which stage completion is merely procedural versus candidate verification;
- what can be resumed versus only reconstructed.

LoopForge demonstrates the value of explicit stage state, artifact baselines, role separation, bounded retries, resume semantics, and stale-runtime protection. SkillHone demonstrates the value of persistent decision history, redacted failure observations, score provenance, negative-experiment retention, one-change attribution, and evaluator separation.

The Winds-specific synthesis should be stricter than both:

```text
WORKFLOW_CONTINUITY
+ EXACT_CANDIDATE_EVIDENCE
+ EXPLICIT_AUTHORITY
+ ACTOR_IDENTITY
+ ARTIFACT_FRESHNESS
+ DECISION_PROVENANCE
+ MECHANICALLY_ENFORCED_EVALUATION_BOUNDARIES
= PROOF_CARRYING_RESUMABLE_WORKFLOW
```

---

## 3. What to adopt, adapt, defer, and reject

### Adopt as Winds-native concepts

From LoopForge:

1. Explicit workflow state and stage transitions.
2. Resume from persisted state rather than re-inferring the whole workflow from chat history.
3. Stage-specific actor identity and role binding.
4. Artifact baselines/freshness: old artifacts cannot satisfy a newly prepared stage gate silently.
5. Bounded retry counts and explicit blocked truth.
6. Distinct prepare/start/finish/approve semantics.
7. Conditional knowledge/distillation rather than mandatory ceremony.
8. Stage-specific context handoff instead of replaying all prior context.
9. Negative tests for stale actor/team reuse and invalid executor identity.
10. Atomic/transactional state persistence.

From SkillHone:

1. Persistent decision history.
2. Failure diagnosis before changing behavior.
3. Separation of optimizer/proposer from evaluator/acceptance authority.
4. Practice/probe evidence distinct from protected acceptance evidence.
5. Score/evidence provenance: baseline, iteration, validation, final acceptance cannot be mixed.
6. Redacted trajectory/failure observations.
7. One attributable candidate revision per experiment/cycle.
8. Retention of failed/reverted experiments.
9. Early-stop/no-progress policies and explicit budgets.
10. Whole-skill evolution: a reusable skill is a versioned package, not just one prompt string.

### Adapt rather than copy

- LoopForge filesystem JSON workflow state -> Winds should prefer the accepted SQLite transactional store and canonical IDs.
- LoopForge agent/team identifiers -> Winds should bind stage actors to canonical Winds session/runtime/run-instance identities.
- SkillHone Forgejo issue/PR/wiki observability -> Winds should use its own evidence/decision ledger and existing GitHub/repository governance where applicable.
- SkillHone skill folder model -> Winds should use an immutable `SkillBundleVersion` manifest/hash spanning instructions, scripts, references, assets, and tests where admitted.
- SkillHone redaction -> Winds should define source-aware secret/eval-data redaction plus explicit evidence-completeness flags; redaction must not silently transform missing evidence into success.

### Defer

- automatic skill optimization;
- learned routing;
- automatic provider selection;
- persistent background owner;
- public/local daemon control protocol;
- browser automation;
- remote/mobile continuation;
- automatic PR merge/landing;
- model-weight training/fine-tuning/RL.

### Reject for current/foundation architecture

- wholesale Python orchestration runtime transplant;
- Forgejo as a required product control plane;
- LiteLLM as an implicit provider gateway dependency;
- prompt-only security boundaries;
- automatic reviewer merge as acceptance authority;
- a single aggregate learning/winner score;
- hidden evaluator state exposed to the optimizer while still being described as protected;
- generic plugin/runtime abstractions before concrete need.

---

## 4. Gap analysis against the existing Winds roadmap

### G1 — No first-class WorkflowRun / StageRun object

Spec 006 gives Winds canonical workstream/session/context semantics, but there is no general workflow object with stage lifecycle, attempt identity, blocker truth, approvals, and explicit resume behavior.

**Required improvement:** introduce a future `WorkflowRun` / `StageRun` contract rather than inferring process state from transcripts or ad-hoc task prose.

### G2 — No artifact-freshness gate for workflow stages

Exact candidate evidence already becomes stale when the candidate changes. The same principle is not yet generalized to stage artifacts.

**Required improvement:** every prepared stage records an artifact baseline/digest set. A completion gate must prove the required artifact was created or materially changed after preparation unless the stage explicitly consumes an immutable upstream artifact.

### G3 — No stale actor/run-instance protection at workflow level

Winds distinguishes canonical session/task identity from runtime/native identity, but a resumed workflow also needs to reject stale actor bindings.

**Required improvement:** bind a stage attempt to a unique run-instance/actor identity. A new run with the same human-readable name must not inherit an old actor/team lease implicitly.

### G4 — No explicit stage handoff minimization contract

Current context capsules preserve canonical task context, but the roadmap does not yet define stage-specific context minimization.

**Required improvement:** each stage receives an allowlisted handoff assembled from canonical structured state and exact upstream artifacts. Independent reviewers should not automatically inherit builder confidence, persuasive rationale, or irrelevant transcript history.

### G5 — Retry, blocked, restart, and recovery semantics are underspecified

A durable agent product needs more truth than `RUNNING/DONE`.

**Required improvement:** define stage states and transitions for prepared, active, waiting approval, waiting external condition, blocked, failed, stale, cancelled, completed, and recovery-required. Bound automatic retries. Do not normalize repeated failure into progress.

### G6 — Decision history is not yet a first-class append-only object

Winds records human decisions and evidence, but future long-running workflows and learning need a durable sequence of why a path was chosen, rejected, reverted, or superseded.

**Required improvement:** define `DecisionRecord` entries bound to source, actor, task/workflow/stage, candidate/evidence references, rationale, outcome, and supersession lineage.

### G7 — Failure-source diagnosis is not a formal precondition to learning/proposals

A failed task may be caused by infrastructure, runtime/tool invocation, candidate code, verifier design, stale evidence, policy, or model behavior.

**Required improvement:** future learning/optimization must classify the failure layer before creating a reusable lesson. `wrong_answer` or a low score alone is not enough.

### G8 — Protected evaluation lacks a concrete enforcement model

The Verified Learning roadmap correctly requires protected evaluation, but it does not yet freeze the exact mechanical boundary.

The inspected SkillHone public bundle is a useful caution: its documentation describes private eval separation, while `optim.py` also places the eval clone path in run configuration/prompt-visible workspace state and uses a high-permission agent mode. That does not prove a hard filesystem/capability isolation boundary for the public bundle.

**Required improvement:** Winds must define protection by enforcement, not wording. If the optimizer can read the holdout or evaluator secret material, the holdout is not protected.

### G9 — No holdout access audit / leakage invalidation lifecycle

The roadmap says leakage invalidates learning, but it does not yet define the event trail.

**Required improvement:** every protected evaluation run must record evaluator identity, allowed data boundary, access attempts, leakage status, and invalidation reason. A contaminated experiment cannot later become valid by deleting logs.

### G10 — Skill identity is too abstract for whole-skill evolution

`SkillVersion` is useful but should explicitly cover the entire executable/consulted skill surface.

**Required improvement:** define `SkillBundleVersion` with an immutable manifest covering `SKILL.md`-equivalent instructions, helper scripts, references, assets/templates, tests, declared tool requirements, and content hashes.

### G11 — No explicit score/evidence provenance graph

A metric can refer to baseline, visible probe, PR validation, held-out acceptance, canary, or live monitoring.

**Required improvement:** every score/result must identify task-set version, split, evaluator version, candidate version, runtime/model/toolchain, budget, repetitions, and whether the result is visible/protected.

### G12 — Attribution can be lost when several improvements are bundled

Whole-skill changes may legitimately touch several files, but multiple unrelated hypotheses in one candidate destroy attribution.

**Required improvement:** one experiment candidate may be an atomic multi-file bundle, but it should correspond to one explicit hypothesis unless a dependency relation requires otherwise.

### G13 — No-progress / loop detection needs to be broader than retry count

Long-running agents can oscillate, repeatedly rewrite the same diff, repeatedly compact context, or consume budget without semantic progress.

**Required improvement:** track stage attempts, repeated candidate hashes/diff signatures, repeated failure classes, repeated context checkpoints without material progress, and budget consumption. Pause/block rather than endlessly retry.

### G14 — Durable redaction and privacy rules are incomplete

Long-lived trajectories, workflow logs, and decision history can accidentally persist credentials or protected evaluation data.

**Required improvement:** define redaction-before-durable-write for secrets and protected eval content, plus explicit loss markers so redaction does not masquerade as complete evidence.

### G15 — Workflow-state schema migration/corruption recovery is not planned

A multi-day product must survive product upgrades and damaged local state.

**Required improvement:** version workflow state, make migrations explicit and reversible where possible, validate before use, preserve corrupted state for recovery, and never silently reinitialize an existing workflow over unreadable state.

### G16 — Resume proof levels need to apply to workflow stages, not only sessions

Existing Winds vocabulary distinguishes live/resumed/reconstructed/ownership-lost semantics. Workflow continuation needs the same rigor.

**Required improvement:** a resumed `StageRun` records whether the original actor remains live, the native runtime resumed, the workflow was reconstructed with a new actor, or ownership is lost.

### G17 — External side-effect retry policy is not explicit enough for a durable owner

When future background execution arrives, a crash between an external write and checkpoint can duplicate the write on retry.

**Required improvement:** checkpoint before external wait where possible; require idempotency keys/deduplication or explicit at-least-once semantics; never claim exactly-once delivery when the external system cannot prove it.

### G18 — Workflow completion can be confused with candidate verification

A stage or workflow may be procedurally complete while the software candidate remains unverified or unaccepted.

**Required improvement:** preserve separate terminal states such as `WORKFLOW_COMPLETED`, `VERIFICATION_PENDING`, `VERIFIED`, and `HUMAN_ACCEPTED`.

### G19 — Resume/block truth needs a first-class daily UX

The roadmap contains rich workbench UX but not enough operator-facing explanation for long-running workflow state.

**Required improvement:** future UI/CLI should answer:

```text
What is this workflow doing?
Which stage is active?
Who owns it?
What exact candidate/evidence applies?
Why is it blocked?
What will resume do?
What will NOT be restored?
Which approval or external condition is required?
```

### G20 — New sources need exact provenance in repository planning

**Required improvement:** preserve the exact URLs, pins, licenses, inspected paths, and admission state. Presence in research remains non-authorizing.

---

## 5. Proposed research invariants

These are proposed future requirements, not current constitutional law.

```text
WORKFLOW_RUN != CHAT_SESSION
STAGE_RUN != AGENT_SESSION
STAGE_COMPLETE != CANDIDATE_VERIFIED
WORKFLOW_COMPLETE != HUMAN_ACCEPTED

PREPARED_STAGE_HAS_ARTIFACT_BASELINE
REUSED_STALE_ARTIFACT => STAGE_GATE_FAILS

STAGE_ATTEMPT_HAS_UNIQUE_RUN_INSTANCE
STALE_ACTOR_BINDING => BLOCK_OR_RECONSTRUCT

RESUMED != RECONSTRUCTED
RECONSTRUCTED_STAGE_MUST_REPORT_LOSS

RETRY_REQUIRES_BOUNDED_POLICY
EXTERNAL_WRITE_RETRY_REQUIRES_IDEMPOTENCY_OR_EXPLICIT_AT_LEAST_ONCE_SEMANTICS

DECISION_RECORDS_ARE_APPEND_ONLY
SUPERSEDED_DECISION_REMAINS_AUDITABLE
FAILED_OR_REVERTED_ATTEMPT_IS_EVIDENCE

OPTIMIZER != EVALUATOR
VISIBLE_PROBE != PROTECTED_HOLDOUT
EVAL_PATH_VISIBLE_TO_OPTIMIZER => HOLDOUT_NOT_PROTECTED
HOLDOUT_ACCESS_OR_LEAKAGE => AFFECTED_LEARNING_EVIDENCE_INVALID

SKILL_VERSION_HASH_COVERS_WHOLE_SKILL_BUNDLE
CHANGED_SKILL_BUNDLE_HASH => PRIOR_SKILL_APPROVAL_STALE

SCORE_WITHOUT_SPLIT_EVALUATOR_CANDIDATE_AND_BUDGET_PROVENANCE => NON_CANONICAL_FOR_PROMOTION

NO_PROGRESS_WITHIN_BUDGET => PAUSE_OR_BLOCK

REDACTED_DATA_MUST_RETAIN_EXPLICIT_OMISSION_MARKERS
REDACTION != EVIDENCE_COMPLETENESS
```

---

## 6. Revised post-Spec-007 sequence

The previous Spec 008–012 numbering was explicitly provisional research ordering. Based on the new evidence, the recommended sequence becomes:

```text
SPEC_007  Native Agentic Terminal UX Foundation
    ↓
SPEC_008  Resumable Workflow & Decision Ledger
    ↓
SPEC_009  Model Mesh / explicit multi-provider session continuity
    ↓
SPEC_010  Winds Continuum + durable local runtime owner
    ↓
SPEC_011  Verified Browser + browser evidence
    ↓
SPEC_012  Proof-Carrying Reality Branches / candidate comparison
    ↓
SPEC_013  Verified Learning + Experiment Plane
    ↓
SPEC_014  Remote/mobile/team continuation, only after local truth is strong
```

### Why insert Spec 008 before Model Mesh?

Multi-provider continuity should not make the provider/session transcript the workflow truth. A stable workflow/stage/decision ledger gives provider switching a canonical handoff target and lets Winds prove whether work was resumed, reconstructed, blocked, or reassigned.

The new Spec 008 should remain **single-process** and build on accepted local persistence. It does not need a daemon, model API, provider gateway, browser, or remote control.

### Why schedule Verified Learning after the execution/verification substrate?

Learning is safest after Winds has:

- exact workflow/stage identity;
- exact actor/run identity;
- durable decision provenance;
- provider/runtime identity;
- durable execution/recovery semantics;
- exact candidate/evidence identity;
- comparative candidate infrastructure.

This makes learning an evidence consumer, not a new source of authority.

---

## 7. Proposed Spec 008 — Resumable Workflow & Decision Ledger

### Product goal

A developer can leave and return to a multi-stage Winds task and get an exact, truthful answer about where the workflow is, which actor/candidate/evidence applies, what is blocked, and what can safely resume.

### Candidate data concepts

```text
WorkflowRun
- workflow_run_id
- canonical workstream_id
- workflow kind/schema version
- current stage
- lifecycle status
- created/updated timestamps
- current canonical candidate reference, if applicable
- blocker/approval summary
- policy version

StageRun
- stage_run_id
- workflow_run_id
- stage name/kind
- attempt number
- prepared_at / started_at / completed_at
- status
- actor binding
- context capsule reference/hash
- artifact baseline
- produced artifact references/hashes
- applicable authority ceiling
- applicable evidence references
- blocker/failure class
- resume proof level

ActorBinding
- Winds session ID
- runtime identity
- native runtime/session identity if proven
- run-instance ID
- role
- binding/lease state
- proof level

DecisionRecord
- decision_id
- workflow/stage/workstream references
- source/actor/authority class
- decision type
- exact evidence/candidate references
- rationale
- result
- supersedes / superseded-by
- created_at
```

### Required behavior

1. Deterministic stage state machine.
2. Explicit prepare/start/finish/approve/block/cancel/recover semantics.
3. Transactional local persistence through existing accepted storage architecture.
4. Artifact freshness baselines.
5. Unique stage-attempt/run-instance identity.
6. No implicit attachment to stale actor/team identity.
7. Stage-specific context handoff assembled from canonical structured state.
8. Reviewer handoff excludes author confidence/persuasion by default.
9. Bounded retry and no-progress detection.
10. Explicit restart/reconstruction report.
11. Append-only decision history.
12. Readable `status` / `resume` / `why-blocked` projection.
13. Secret/protected-data redaction before durable trajectory storage.
14. Versioned state schema and fail-closed migration/validation.

### Required adversarial fixtures

- stale actor identity with same display name;
- changed artifact that does not match prepared baseline;
- unchanged stale artifact presented as new work;
- actor attempts to self-expand stage authority;
- retry after ambiguous external side effect;
- duplicate/replayed completion event;
- corrupted/partial workflow state;
- reconstructed stage falsely claiming native resume;
- builder rationale contaminating independent reviewer input;
- changed candidate after stage evidence was gathered;
- repeated same failure/diff without progress;
- secrets/eval content present in raw tool output before durable logging.

### Non-goals

- no daemon;
- no IPC/public runtime protocol;
- no provider/model API calls;
- no browser control;
- no remote continuation;
- no learned routing;
- no automatic PR/merge/push/landing;
- no generic workflow-plugin engine;
- no hidden evaluator yet.

---

## 8. Proposed Spec 009 — Model Mesh improvements

Model Mesh should consume Spec 008 rather than invent its own continuity state.

Additional requirements:

1. Provider/model switch occurs at an explicit workflow checkpoint.
2. A new provider receives a bounded canonical handoff, not an unqualified transcript dump.
3. Actor binding changes create a new run-instance identity.
4. Native resume and Winds reconstruction remain separate proof levels.
5. Model/provider switch cannot automatically mark a stage complete.
6. Provider-specific hidden state is listed as unavailable when it cannot be transferred.
7. Cost/usage evidence is source-labelled and stage/workflow attributable.
8. Direct user selection precedes learned/automatic routing.

---

## 9. Proposed Spec 010 — Durable Continuum improvements

LoopForge's resume discipline becomes more important once Winds introduces a persistent owner.

Future durable-owner requirements should include:

- journaled/transactional lifecycle state;
- ownership leases/heartbeats;
- crash-safe checkpoint/recovery;
- unique action/attempt IDs;
- duplicate-event detection;
- idempotency policy for external writes;
- explicit at-least-once semantics where exactly-once is impossible;
- checkpoint before external waits when feasible;
- monotonic retry budgets;
- stale lease recovery;
- workflow schema migrations;
- interrupted upgrade recovery;
- human-visible recovery report;
- no silent restart of failed destructive/consequential actions.

---

## 10. Proposed Spec 013 — Verified Learning + Experiment Plane improvements

### Whole-skill identity

Replace the ambiguous idea of a prompt-only skill with:

```text
SkillBundleVersion
- stable skill ID
- immutable version ID
- manifest version
- content hash
- instruction files
- scripts/helpers
- references
- assets/templates
- tests
- declared tools/dependencies
- required authority/capabilities
- provenance
- applicability constraints
- status
```

### Evaluation boundary

Define three surfaces:

```text
PUBLIC / OPTIMIZER_VISIBLE
- task spec
- repository tests
- public compiler/linter feedback
- permitted redacted observations

PRACTICE / PROBE
- evaluation items available for iterative improvement
- explicit non-final score provenance

PROTECTED ACCEPTANCE
- held-out items/invariants
- evaluator-only configuration
- leakage/access audit
- independent evaluator identity
```

The protection mechanism must be testable. A prompt instruction saying "do not read the eval repo" is not sufficient.

### Evaluation evidence

Every result should bind:

- exact skill bundle version;
- exact task-set/split version;
- evaluator version;
- runtime/model/toolchain;
- candidate/worktree where relevant;
- budget;
- repetitions/seeds where justified;
- failure taxonomy;
- raw evidence references;
- redaction/omission status;
- leakage status;
- outcome vector.

### Experiment discipline

1. One explicit hypothesis per candidate where practical.
2. Candidate may atomically modify several files in one skill bundle.
3. Preserve rejected and reverted candidates.
4. Diagnose infrastructure/verifier/tool failures before changing skill behavior.
5. Use public compilers/linters as development evidence when they are the relevant real toolchain.
6. Stop/pause on no progress or budget exhaustion.
7. No promotion from visible probe gain alone.
8. Optimizer and acceptance evaluator remain independent.
9. Canary before broad activation.
10. Rollback target is fixed before activation.
11. Drift/invalidation when repository/runtime/model/toolchain/evaluator assumptions change.
12. No learning change may expand authority or weaken verification gates.

---

## 11. UX additions to the North Star

Future Winds should expose workflow truth as clearly as candidate truth.

### Workflow rail

```text
Task: checkout-redesign
Workflow: feature-delivery

✓ REQUIREMENT     confirmed
✓ DESIGN          completed
→ IMPLEMENT       active · Codex · attempt 2
! REVIEW          blocked by implementation
· TEST            pending
· ACCEPTANCE      pending

Candidate: 251dd272...
Evidence: stale after candidate move? NO
Authority: builder-write / no-land
Resume proof: RECONSTRUCTED
```

### Resume preview

Before a consequential resume, Winds should be able to show:

```text
Will resume:
- canonical task/workstream
- stage: IMPLEMENT / attempt 2
- accepted decisions
- exact candidate/worktree
- authority ceiling
- required validation gates

Will not resume:
- prior provider-private hidden context
- stale actor ownership
- expired tool/session state

Reconstructed from:
- canonical WorkflowRun
- StageRun artifacts
- context capsule
- decision history
- evidence references
```

### Decision history view

Developers should be able to inspect not only what is active, but what was rejected and why. This is essential for avoiding repeated failed approaches and for trustworthy future learning.

---

## 12. Required planning gates before any future implementation

### Gate A — Exact post-Spec-007 reconciliation

Do not author Spec 008 from this document until Spec 007 is canonically closed and the repository is re-read from its landed main/tree.

### Gate B — No duplication of existing Spec 006/007 primitives

Spec 008 must reuse accepted workspace/workstream/session/context/evidence/authority primitives. It should add only missing workflow/stage/decision semantics.

### Gate C — Storage decision

Prefer existing SQLite transactional persistence. A new database, service, daemon, or generic event bus requires separate proof of necessity.

### Gate D — State machine first

Freeze lifecycle states/transitions and invariants before UI or automation.

### Gate E — Adversarial recovery fixtures

Stale actor, stale artifact, duplicate event, corrupted state, retry ambiguity, authority escalation, and reconstruction-overclaim tests are required before live resume UX can claim correctness.

### Gate F — Learning isolation threat model

Before protected evaluation is implemented, define exactly which process/actor can access which paths/data/capabilities and how that is enforced/tested.

### Gate G — Provenance

If any upstream code is copied/adapted later, add the exact source path, source commit, license text/notice obligations, Winds modifications, and update strategy to `docs/provenance/donors.md` before acceptance.

---

## 13. Recommended execution order after current canonical work closes

```text
1. Finish Spec 007 exactly as currently authorized.
2. Reconcile landed main, close T100/program if evidence permits.
3. Canonically land the external-source provenance update.
4. Author a formal Spec 008 entry gate from exact post-Spec-007 truth.
5. Freeze Spec 008 user scenarios/non-goals/security/recovery semantics.
6. Plan the smallest SQLite-backed workflow/decision architecture.
7. Task-slice StageRun state, artifact freshness, actor identity, handoff, recovery, and UX projections.
8. Qualify Spec 008 through deterministic/adversarial/review gates.
9. Only then author Model Mesh Spec 009.
10. Durable owner remains later Spec 010 with a dedicated threat model.
11. Browser/reality/learning/remote programs remain downstream and separately authorized.
```

---

## 14. Definition of planning completeness

This research plan is complete only when it answers all of the following without hand-waving:

- What is canonical workflow state?
- What is merely runtime/native state?
- What exactly can resume?
- What is only reconstructed?
- How does Winds prevent stale actor reuse?
- How does Winds prove stage artifacts are fresh?
- How does a reviewer receive independent context?
- How are retries bounded?
- How are external side effects deduplicated or truthfully classified?
- How are failed/reverted decisions retained?
- How are secrets and protected eval data kept out of durable logs?
- What mechanically protects holdout evaluation?
- What event invalidates learning evidence?
- What exactly constitutes a skill version?
- What score/evidence provenance is required?
- How does the user see why work is blocked?
- Which future spec owns each capability?
- Which surfaces remain deliberately unavailable?

The roadmap should be considered stronger when the answer is "not yet available" with an explicit boundary rather than a simulated capability or an unverifiable promise.

---

## 15. Final recommendation

Winds should not copy LoopForge or SkillHone as products. It should use them to sharpen a stronger architecture:

> **LoopForge contributes resumable workflow discipline. SkillHone contributes decision/evaluation discipline. Winds contributes exact software reality, authority truth, and independent evidence.**

The resulting product direction is:

> **A proof-carrying agentic development runtime where workflows can resume without identity amnesia, decisions remain inspectable, evaluation cannot be gamed by mere prompt convention, and no agent can turn its own narrative into verification or authority.**
