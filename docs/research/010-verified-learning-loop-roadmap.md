# Winds Verified Learning Loop Roadmap

**Status:** Research-backed future product roadmap. Non-normative and non-authorizing.

**Prepared:** 2026-08-26

**Canonical base inspected:** `06e515471cf91a0f1d5b257d6e9820096d9a0197`

**Current implementation firewall:** Spec 006 / T079 remains the active implementation program. This roadmap MUST NOT widen T079, T080, or any current Spec 006 task, MUST NOT authorize new runtime/model calls, and MUST NOT be used to bypass the Constitution -> Spec -> Plan -> Tasks -> Implement sequence.

**Purpose:** Reconcile Winds' existing evidence-first architecture and historical learning research with newer evidence from autonomous experimentation, procedural skill learning, evaluator design, and self-improving agent systems so the next post-Spec-006 program is explicit, ordered, testable, and easy to follow.

---

## 1. Executive Decision

Winds should not become another agent that merely remembers conversations or rewrites its own core.

The recommended post-Spec-006 direction is:

> **Verified work -> Verified experience -> Learning proposal -> Bounded experiment -> Learning evidence -> Explicit promotion -> Canary -> Monitor -> Rollback.**

The long-term product distinction should become:

> **Any agent. Verified work. Verified learning.**

Winds already has the strongest prerequisite: exact-candidate identity, independent evidence, explicit authority, stale-evidence invalidation, and human decision provenance. The missing product layer is a canonical scientific loop that can prove whether a reusable skill, routing rule, context policy, reviewer strategy, or verification strategy actually improves future outcomes.

This roadmap therefore adds a future **Experiment Plane** and a controlled **Verified Learning Loop** without weakening existing Winds authority or verification semantics.

---

## 2. Current Winds Strengths To Preserve

The future learning system MUST inherit, not replace, the existing constitutional boundaries:

```text
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
RUNTIME != MODEL
NEW_SESSION != NEW_TASK
NEW_AGENT != NEW_TASK
WORKTREE != SANDBOX
IDLE != DONE != VERIFIED != ACCEPTED
CHANGED_CANDIDATE => PRIOR_CHECKS_AND_REVIEW_STALE
NO_AUTOMATIC_WINNER
NO_AUTOMATIC_AUTHORITY_ESCALATION
NO_SILENT_LANDING
VERIFY_THE_EXACT_CANDIDATE
```

The existing Spec 006 program correctly prioritizes identity, context, authority, runtime provenance, one bounded delegation, exact candidate review, and independent verification before broad fleet complexity. That sequence remains valid and MUST finish before this roadmap becomes an implementation program.

---

## 3. Gap Assessment

### G1 — No canonical experiment object

Current Winds can verify an exact candidate but does not yet define a first-class experiment that binds:

- hypothesis;
- baseline/champion;
- challenger;
- task set;
- visible vs protected evaluation surfaces;
- runtime/model/toolchain identity;
- time/cost/resource budget;
- repeated trials/seeds where applicable;
- deterministic checks;
- independent evaluator evidence;
- outcome uncertainty;
- regression/safety results;
- promotion decision.

Without this object, self-improvement risks becoming anecdotal memory rather than reproducible evidence.

### G2 — Verified experience is research, not canonical product behavior

Historical Winds research already proposes `VerifiedExperience`, procedural memory, evidence-aware routing, predictive execution, and learning. That direction remains strong, but it is currently research-only and predates the accepted Spec 006 architecture.

Required action: reconcile the historical archive against post-Spec-006 canonical truth rather than merging the old archive unchanged.

### G3 — No promotion firewall between task success and reusable learning

A task that passes current verification is not automatically evidence that its method should become a general skill.

Required distinction:

```text
VERIFIED_TASK_SUCCESS != PROMOTABLE_GENERAL_SKILL
```

### G4 — No protected holdout model for learning promotion

Visible repository tests can be optimized or accidentally overfit. Future learning promotion needs evaluation surfaces that the optimizer cannot fully inspect or influence during proposal generation.

### G5 — No evaluator-independence contract for learning

Current Winds correctly separates authoring and independent review. The same principle must extend to learning:

```text
OPTIMIZER != EVALUATOR
```

A model/agent that proposes a skill or routing change cannot be the sole authority that certifies the improvement.

### G6 — No skill lifecycle with exact identity

Future procedural skills need exact content identity, provenance, evaluation evidence, status, invalidation, rollback, and applicability boundaries.

### G7 — No learning rollback/drift contract

A promoted strategy can become stale when repositories, models, tools, dependencies, policies, or runtime versions change. Learning must be reversible and continuously scoped to the conditions under which it was proven.

### G8 — No canonical negative-experiment memory

Failed experiments are valuable evidence. The system should retain them so agents do not repeatedly rediscover known-bad strategies.

### G9 — No evidence-aware routing baseline

Winds research already anticipates choosing agents/reviewers based on verified outcomes, but the product needs a transparent heuristic/statistical baseline before learned routing.

### G10 — No daily compounding user surface

Verification alone is valuable near acceptance time. Learning should make Winds more useful every day by exposing what has been learned, why it is trusted, where it applies, and when it has been invalidated.

---

## 4. Research Synthesis

### 4.1 Autoresearch — borrow experimental discipline, not a single magic metric

High-value pattern:

```text
bounded mutable surface
+ fixed budget
+ objective evaluation
+ rapid modify/run/measure/keep-or-discard loop
```

Winds translation:

- explicitly define what a learning candidate may change;
- freeze evaluator/policy surfaces during an experiment;
- compare against a known baseline;
- bind all evidence to exact identities;
- preserve failures;
- never collapse multidimensional software quality into one unqualified winner score.

### 4.2 Hermes — borrow inspectable procedural skills and approval-aware writes

High-value pattern:

- procedural knowledge is represented as inspectable skills;
- skills can evolve separately from model weights;
- memory/skill writes can require approval.

Winds translation:

- agents may propose learning;
- proposals remain non-authoritative;
- a skill becomes active only after evidence-backed evaluation and allowed promotion;
- learning must not directly mutate Winds' authority or verification core.

### 4.3 Reflexion / Voyager — learn before fine-tuning

High-value pattern:

- improve behavior from episodic feedback and reusable skill libraries without weight updates.

Winds translation:

- begin with verified procedural skills and routing/context policies;
- do not begin with RL, fine-tuning, or a native learned world model.

### 4.4 ACE — evolve playbooks incrementally

High-value pattern:

- treat context/playbooks as evolving structured state rather than repeatedly rewriting a lossy summary.

Winds translation:

- skill changes should be content-addressed deltas or new versions;
- preserve provenance and prior versions;
- avoid destructive global rewriting that loses constraints or applicability details.

### 4.5 Darwin Gödel Machine — preserve a diverse archive, but do not self-modify the trusted core first

High-value pattern:

- explore multiple alternative policies/agents rather than following only one greedy improvement lineage.

Winds translation:

- maintain champion/challenger archives;
- preserve failed and alternative variants;
- defer self-modifying Winds core until much stronger evidence and governance exist.

### 4.6 PaperBench / SpecBench / RE-Bench — evaluator quality, holdouts, and budget are first-class

High-value patterns:

- complex work needs decomposed evaluation rather than one superficial score;
- evaluators themselves need validation;
- visible tests can diverge from held-out correctness;
- comparisons are invalid when compute/time budgets are not controlled.

Winds translation:

- protected evaluation surfaces;
- evaluator provenance/versioning;
- repeated trials where stochasticity matters;
- explicit time/cost budgets;
- regression and adversarial gates;
- no promotion from a single lucky run.

### 4.7 Faraday / Replica — higher-level policy above coding agents is viable

High-value pattern:

- a higher-level policy can use coding agents as interchangeable tools and learn orchestration behavior.

Winds translation:

- long-term learned fleet policy is strategically aligned;
- learning reward must remain separate from canonical Winds verification authority;
- learned routing begins only after the Verified Experience and Experiment planes are trustworthy.

---

## 5. Proposed New Invariants

These are recommendations for a future constitutional/specification amendment after Spec 006 closes. They are NOT current constitutional rules yet.

```text
EXPERIENCE != LESSON

LESSON != SKILL

SKILL_CANDIDATE != ACTIVE_SKILL

MEMORY_WRITE != TRUSTED_MEMORY

VISIBLE_TEST_PASS != GENERALIZED_SUCCESS

VERIFIED_TASK_SUCCESS != PROMOTABLE_GENERAL_SKILL

OPTIMIZER != EVALUATOR

SAME_AGENT_AUTHORS_AND_CERTIFIES => NON_CANONICAL_FOR_PROMOTION

SAME_JUDGE_TRAINS_AND_ACCEPTS => INSUFFICIENT_FOR_CANONICAL_PROMOTION

LEARNING_CANNOT_EXPAND_AUTHORITY

LEARNING_CANNOT_MUTATE_THE_POLICY_PLANE

LEARNING_CANNOT_WEAKEN_VERIFICATION_REQUIREMENTS_BY_SELF-ASSERTION

CHANGED_SKILL_HASH => PRIOR_APPROVAL_STALE

CHANGED_EVALUATOR => PRIOR_COMPARISON_NOT_AUTOMATICALLY_COMPARABLE

FAILED_EXPERIENCE_IS_EVIDENCE

NEGATIVE_EXPERIMENTS_ARE_RETAINED

NO_SINGLE_MAGIC_LEARNING_SCORE

PROMOTED_LEARNING_MUST_BE_REVERSIBLE

CANARY_FAILURE_MUST_NOT_BE_NORMALIZED_AS_SUCCESS

HOLDOUT_LEAKAGE_INVALIDATES_THE_AFFECTED_LEARNING_CLAIM
```

---

## 6. Target Architecture

```text
Human Intent
    │
    ▼
Canonical Task / Workstream
    │
    ▼
Agent Runtime / Team
    │
    ▼
Execution + Authority Plane
    │
    ▼
Exact Candidate
    │
    ▼
Evidence Plane
    │
    ▼
Verification Plane
    │
    ▼
Verified Experience
    │
    ├── successful trajectory
    └── failed / repaired trajectory
    │
    ▼
Reflection / Distillation
    │
    ▼
LearningProposal
    │
    ├── ProceduralSkillCandidate
    ├── ContextPolicyCandidate
    ├── RoutingPolicyCandidate
    ├── ReviewerPolicyCandidate
    └── VerificationPolicyCandidate
    │
    ▼
Experiment Plane
    │
    ├── champion vs challenger
    ├── exact task-set identity
    ├── protected holdouts
    ├── repeated trials
    ├── adversarial/regression gates
    ├── fixed resource budget
    ├── independent evaluator
    └── uncertainty/effect evidence
    │
    ▼
Learning Evidence
    │
    ▼
Explicit Promotion Decision / Bounded Policy
    │
    ▼
Canary
    │
    ▼
Active Version
    │
    ▼
Monitor / Drift / Invalidation / Rollback
    │
    └──────────────────────────────────────────────↺
```

---

## 7. Proposed Future Data Concepts

These are conceptual shapes only. Do not create migrations from this document.

### 7.1 VerifiedExperience

```text
VerifiedExperience
- stable experience ID
- canonical task/workstream ID
- workspace/repository identity
- exact base/candidate/tree identity
- runtime/model/toolchain identity
- source-labelled trajectory summary
- observed commands/process facts
- observed Git/file effects
- deterministic verification evidence
- independent review evidence
- human decision, if any
- cost/time/resource observations with provenance
- failure/repair lineage
- evidence completeness flags
```

### 7.2 LearningProposal

```text
LearningProposal
- proposal ID
- source experience IDs
- proposed learning type
- normalized proposed content
- claimed applicability
- claimed expected benefit
- known limitations
- proposer identity/source
- no activation authority
```

### 7.3 SkillVersion

```text
SkillVersion
- stable skill ID
- immutable version ID
- content hash
- normalized procedural content
- provenance
- applicability constraints
- required authority/capabilities
- evaluation references
- status: candidate | canary | active | deprecated | revoked
- supersedes / superseded-by lineage
```

### 7.4 Experiment

```text
Experiment
- experiment ID
- hypothesis
- baseline/champion identity
- challenger identity
- exact task-set identity
- protected-evaluation identity
- evaluator version/identity
- runtime/model/toolchain versions
- budget
- repetition/seed policy
- outcome dimensions
- safety/regression dimensions
- raw evidence references
- statistical/uncertainty summary where justified
- final non-automatic decision state
```

### 7.5 LearningDecision

```text
LearningDecision
- candidate identity
- exact applicable evidence set
- decision authority
- activate | canary | reject | defer | revoke
- scope/applicability
- rollback target
- rationale
```

---

## 8. Evaluation Dimensions

Future learning MUST be evaluated as a vector of evidence, not a single score.

### Outcome

- verified task success rate;
- first-pass verified success;
- repair cycles to verified success;
- regression rate;
- human rejection rate after apparent success.

### Efficiency

- wall-clock time;
- model/agent calls;
- token usage where source provenance is reliable;
- compute/resource use;
- monetary cost with pricing-version provenance;
- storage/worktree churn.

### Safety and authority

- authority violations;
- attempted over-ceiling operations;
- policy bypass attempts;
- unsafe diff expansion;
- unmediated capability exposure;
- secret/privacy boundary events.

### Evidence quality

- evidence completeness;
- stale/mismatched snapshot rate;
- agent-claim vs observed-fact disagreement;
- false-positive verification rate;
- false-negative blocking rate.

### Generalization

- visible-task performance;
- held-out task performance;
- adversarial task performance;
- repository-family transfer;
- runtime/model-version transfer;
- degradation outside claimed applicability.

### Learning quality

- useful skill reuse rate;
- false/stale memory rate;
- contradiction rate;
- skill invalidation correctness;
- rollback rate;
- champion/challenger regret;
- calibration of routing predictions.

---

## 9. Protected Evaluation Requirements

Before any learning system may autonomously promote behavior, the future spec should define protected evaluation.

Recommended model:

```text
AGENT / OPTIMIZER VISIBLE
- task specification
- ordinary repository tests
- compiler/linter output
- allowed observed evidence

PROTECTED EVALUATOR
- hidden composition tests where appropriate
- mutation/resilience tests
- adversarial authority tests
- invariant checks
- holdout tasks
- independent evaluator configuration
```

Rules:

1. protected evaluation identity is versioned and access-controlled;
2. leakage is an explicit evidence-invalidating event;
3. evaluation changes invalidate naive historical comparability;
4. optimizer output cannot alter evaluator policy in the same experiment;
5. evaluator success is not accepted merely because the evaluator self-reports correctness;
6. deterministic repository-owned gates remain authoritative where applicable.

---

## 10. Skill Lifecycle

Recommended lifecycle:

```text
raw experience
    ↓
verified experience
    ↓
learning proposal
    ↓
skill candidate
    ↓
offline experiment
    ↓
reject / revise / canary
    ↓
canary
    ↓
active
    ↓
monitor
    ├── healthy -> remain active
    ├── stale -> deprecated
    └── regression -> revoked / rollback
```

A skill change MUST create a new immutable content identity. Silent in-place mutation of an already evaluated skill should not preserve prior approval/evaluation claims.

Initial self-improvement targets SHOULD be limited to:

```text
ALLOWED EARLY TARGETS
- procedural skills/playbooks
- context-selection strategies
- prompt templates
- transparent routing heuristics
- reviewer selection heuristics
- verification-depth heuristics

DEFERRED TARGETS
- Winds authority engine
- Winds verification engine
- protected policy plane
- privilege boundaries
- executable self-rewrite
- automatic constitutional/spec mutation
```

---

## 11. Future Program Sequence

This is sequencing guidance, not an authorized Tasks file.

### L0 — Verified Experience Ledger

**Goal:** canonicalize successful, failed, and repaired execution trajectories as immutable provenance-rich records.

**Entry:** Spec 006 canonically closed.

**Required proof:**

- exact task/workspace/candidate identities;
- source-labelled agent claims versus Winds observations;
- failures retained;
- review/check/human-decision references are linked, not rewritten;
- no learning action yet.

**Closes to authorize:** L1 planning.

### L1 — Learning Proposals Only

**Goal:** allow an agent or deterministic analysis to propose reusable lessons without activating them.

**Required proof:**

- proposal cannot mutate active behavior;
- proposal cites exact source experiences;
- conflicting experiences are surfaced;
- imported/unverified text cannot silently become trusted learning;
- memory-poisoning/adversarial fixtures.

**Closes to authorize:** L2 planning.

### L2 — Verified Procedural Skills

**Goal:** content-addressed skill versions with candidate/canary/active/deprecated/revoked lifecycle.

**Required proof:**

- exact content hash/version;
- applicability boundaries;
- provenance;
- approval/evaluation staleness after content change;
- rollback target;
- no authority expansion from skill text.

**Closes to authorize:** L3 planning.

### L3 — Experiment Plane

**Goal:** prove whether a skill/policy candidate is better than the current baseline under controlled conditions.

**Required proof:**

- champion/challenger identity;
- fixed budget;
- exact task-set identity;
- protected holdouts;
- repeated trials where required;
- multidimensional outcome vector;
- independent evaluator;
- regression/safety gates;
- no automatic promotion from one metric.

**This is the minimum viable verified learning loop.**

### L4 — Evidence-Aware Routing Baseline

**Goal:** transparent rules/statistics select agents/reviewers based on verified historical outcomes.

Start with explainable heuristic/statistical policies before training a model.

Required proof:

- routing features have provenance;
- exact applicability scope;
- counterfactual/offline evaluation where feasible;
- cost-adjusted verified success;
- no authority change caused by routing;
- explicit fallback when evidence is insufficient.

### L5 — Automatic Failure-Family Curriculum

**Goal:** identify recurring verified failure classes and generate bounded evaluation sets/learning proposals.

Required proof:

- clustering/classification does not become truth without evidence;
- synthetic/derived tasks retain provenance;
- curriculum generation cannot alter canonical verification gates;
- protected holdout contamination controls.

### L6 — Alternative Policy / Skill Archive

**Goal:** maintain diverse challenger variants rather than one greedy self-improvement lineage.

Required proof:

- immutable lineage;
- exact comparison evidence;
- pruning does not erase historically relevant failures;
- no self-selection bypass of experiment gates.

### L7 — Prediction and Execution Surprise

**Goal:** predict risk/cost/success or detect anomalous trajectories from verified history.

Use prediction only as a verification amplifier or scheduling input, never as a substitute for deterministic evidence.

### L8 — Learned Routing

**Goal:** train an optional routing/policy model only after enough high-quality verified experience exists.

Required proof:

- training-data firewall;
- held-out and temporal generalization;
- calibrated uncertainty;
- model/version rollback;
- no opaque authority escalation.

### L9 — Optional Policy Training / Fine-Tuning

Only after L0–L8 prove that non-weight learning is insufficient for a concrete measured need.

### L10 — Research / AI-Scientist Mode

Optional specialized mode where a higher-level policy uses coding/research agents as tools under Winds evidence and authority control.

This is a late research product, not a prerequisite for core developer value.

---

## 12. Dependency Chain

```text
SPEC_006_CLOSED_CANONICAL
    ↓
L0 VERIFIED EXPERIENCE
    ↓
L1 LEARNING PROPOSALS
    ↓
L2 VERIFIED PROCEDURAL SKILLS
    ↓
L3 EXPERIMENT PLANE
    ↓
L4 EVIDENCE-AWARE ROUTING
    ↓
L5 FAILURE CURRICULUM
    ↓
L6 ALTERNATIVE POLICY ARCHIVE
    ↓
L7 PREDICTION / SURPRISE
    ↓
L8 LEARNED ROUTING
    ↓
L9 OPTIONAL TRAINING
    ↓
L10 RESEARCH MODE
```

No downstream phase is implied to be required merely because it appears here. Each phase must earn entry through measured need and the normal Spec Kit process.

---

## 13. First Walking Skeleton After Spec 006

Do not begin with learned routing, RL, a world model, or a giant memory subsystem.

The first end-to-end proof should be:

```text
At least several independently verified similar experiences
    ↓
Generate one LearningProposal
    ↓
Human inspects proposal and source evidence
    ↓
Create one content-addressed SkillCandidate
    ↓
Run champion vs challenger experiment
    ↓
Evaluate on exact visible + protected holdout task sets
    ↓
Run safety/regression checks
    ↓
Produce LearningEvidence
    ↓
Human or explicit bounded policy selects reject/canary
    ↓
Canary skill executes under unchanged authority ceiling
    ↓
Observed future outcomes are measured
    ↓
Regression => explicit rollback/revocation path
```

Success means Winds can prove whether one reusable procedural improvement helped. It does NOT mean Winds is generally self-improving.

---

## 14. Repository Integration Plan

To keep the project easy to follow, future work should be represented in repository truth rather than chat-only plans.

### While Spec 006 is active

- keep this document research-only;
- do not add learning tasks to `specs/006-.../tasks.md`;
- do not amend T079/T080 scope;
- do not introduce learning migrations/code/dependencies;
- use this roadmap only as a post-Spec-006 reference.

### After Spec 006 closes canonically

1. re-read live `main`, Constitution, accepted Spec 006 evidence, and all active PRs;
2. reconcile historical PR #21 learning research against current repository truth;
3. perform a fresh research update for changed external systems/protocols;
4. amend the Constitution only where the learning invariants require normative authority;
5. create one formal Spec Kit package for the smallest L0–L3 walking skeleton;
6. write measurable user stories before schema/runtime design;
7. create `spec.md -> plan.md -> tasks.md` with explicit dependencies and authorized paths;
8. implement sequentially from the exact task ordering;
9. require deterministic CI, safety/correctness review, Ponytail, and independent review at each accepted slice;
10. keep L4+ unstarted until L3 demonstrates real product value.

### Rule for future improvements

Any major improvement discovered during implementation or research MUST land in one of these repository truth categories before being considered part of the project plan:

- Constitution invariant/amendment;
- active Spec requirement;
- active Plan architecture decision;
- active Tasks item;
- explicitly non-authorizing Research roadmap entry;
- ADR/decision artifact if that pattern becomes canonical.

Chat-only decisions are handoff context, not durable project authority.

---

## 15. Proposed Post-Spec-006 Product Outcome

A developer should eventually be able to ask:

```text
winds learn
```

and receive an evidence-oriented view, conceptually:

```text
Candidate learning proposals

1. Rust dependency diagnosis skill
   Source: 14 verified experiences
   Conflicts: 2
   Candidate hash: ...
   Current champion: skill@...
   Holdout evaluation: not run
   Status: PROPOSAL_ONLY

2. Reviewer routing policy
   Source: 38 exact-head reviews
   Claim: reviewer X detects lifecycle defects more often
   Confounders: repository family / task class
   Status: INSUFFICIENT_EVIDENCE
```

After evaluation:

```text
Skill candidate: rust-dependency-diagnosis@sha256:...

Champion verified success:    ...
Challenger verified success:  ...
First-pass delta:              ...
Repair-cycle delta:            ...
Cost delta:                    ...
Safety regressions:            0
Protected holdout:             PASS / FAIL / INCONCLUSIVE
Evaluator:                     exact version/source
Evidence completeness:        ...

Decision: CANARY / REJECT / DEFER
```

The user should never need to trust a sentence like "the agent learned this successfully" without inspectable evidence.

---

## 16. Research / Donor Register For Fresh Revalidation

Before formalizing the future learning spec, revalidate exact current versions, licenses, and claims for:

- `karpathy/autoresearch` — bounded autonomous experiment loop;
- Hermes Agent official documentation — procedural skills and approval-aware memory/skill writes;
- Reflexion — episodic reflective learning without weight updates;
- Voyager — reusable skill library and self-verification;
- ACE — evolving playbook/context refinement;
- Darwin Gödel Machine — archive-based open-ended self-improvement;
- PaperBench — decomposed rubric evaluation and evaluator validation;
- RE-Bench — budget/time-sensitive agent evaluation;
- SpecBench — visible-test vs held-out generalization / reward-hacking risk;
- Faraday / Replica — higher-level policy using coding agents as tools;
- current agent-memory and multi-agent evaluation surveys relevant at implementation time.

Research papers are references, not source-code licenses. No source may be copied without the existing Winds provenance/license process.

---

## 17. Explicit Non-Goals For The First Learning Program

Do NOT start with:

- model fine-tuning;
- RL;
- a neural world model;
- executable self-rewrite;
- automatic Constitution/Spec edits;
- learning that changes authority ceilings;
- learning that can disable verification gates;
- an unbounded memory vector database;
- every-agent fleet optimization;
- automatic winner/merge/push/PR decisions;
- cloud control plane dependency;
- silent cross-workspace learning leakage;
- private reasoning capture as required training data.

---

## 18. Final Roadmap Decision

The next strategic Winds program after Spec 006 should not be "more agents" for its own sake.

It should prove this differentiated loop:

```text
Winds observes exact work
→ independently verifies the outcome
→ preserves verified experience
→ proposes a reusable learning artifact
→ evaluates that artifact against a baseline and protected holdout
→ keeps optimizer and evaluator authority separate
→ promotes only through explicit evidence-backed policy
→ canary-observes the result
→ invalidates or rolls back stale/regressing learning
```

That turns Winds from a verification runtime into a **verified learning control plane for software agents** without sacrificing the evidence model that makes Winds different.

Until Spec 006 is closed and a future formal Spec Kit package authorizes implementation, this roadmap remains research guidance only.
