# Spec 008 Formal Entry Gate — Resumable Workflow & Decision Ledger

**Status:** Governance entry candidate. Specification-only authority if canonically accepted.

**Canonical base at creation:** `cf0df0254bdc752b63c078343db6d2843f6a6b21`

**Date:** 2026-09-09

## 1. Purpose

Spec 007's first implementation program is canonically closed. The post-Spec-007 research/provenance reconciliation is also canonically landed through PR #116 without creating successor implementation authority.

This entry gate reconciles the current post-Spec-007 research against that exact repository state and records the Founder decision for the next formal Spec Kit sequence.

This document does not itself create an implementation task. It may authorize only a separate Spec 008 specification candidate after this entry gate is independently qualified and canonically landed.

## 2. Canonical inputs

The entry decision is bounded by:

- Winds Constitution 1.1.0;
- canonical Spec 007 closeout through T100;
- `docs/research/014-loopforge-skillhone-roadmap-reconciliation.md`;
- `docs/research/015-selective-code-adoption-master-plan.md`;
- `docs/research/016-post-spec-007-roadmap-status-reconciliation.md`;
- research RFC #91, which remains non-canonical input rather than implementation authority;
- Draft PR #21, which remains a research archive and not an implementation-acceptance surface.

The accepted post-Spec-007 truth remains:

```text
T087..T100=CLOSED_CANONICAL
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS=CLOSED_CANONICAL
SPEC_007_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_008_FORMAL_SPEC_AUTHORIZED=NO
SPEC_008_PLAN_AUTHORIZED=NO
SPEC_008_TASKS_AUTHORIZED=NO
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```

Historical failed first-attempt CI evidence from the Spec 007 closeout/repair chain remains historical and must not be rewritten, deleted, relabeled as flake evidence, or represented as current success.

## 3. Founder decision

The Founder authorizes the next canonical planning sequence to proceed through the repository's mandatory `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing` discipline.

The next formal specification is:

```text
SPEC_008_NAME=RESUMABLE_WORKFLOW_AND_DECISION_LEDGER
SPEC_008_FORMAL_SPEC_AUTHORIZED=YES_AFTER_THIS_ENTRY_LANDS
SPEC_008_PLAN_AUTHORIZED=NO
SPEC_008_TASKS_AUTHORIZED=NO
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```

This decision is ordinary project authority only. It does not waive exact-head CI, correctness/safety review, Ponytail review, independent review, evidence reconciliation, guarded landing, or any source/dependency/license/security/platform gate.

The Founder's general instruction to continue the project does not itself authorize Spec 008 implementation. Canonical implementation authority may exist only after this entry lands and the separate Spec, Plan, and Tasks stages are each independently qualified and canonically accepted in dependency order.

## 4. Why Spec 008 is next

The current research identifies a missing foundation between Winds' canonical task/context/authority model and later multi-provider, durable-owner, browser, learning, and remote-continuation programs.

The missing layer is:

> **Proof-Carrying Resumable Workflow and Decision State**

Winds already has strong exact-candidate identity, session/task separation, explicit authority, evidence provenance, reviewer independence, and human-decision semantics. It does not yet have a general first-class workflow contract that deterministically answers:

- which workflow stage is active;
- which exact stage attempt and actor/run instance owns that stage;
- which artifacts and evidence were prepared for that stage;
- whether required artifacts are current or stale;
- which exact upstream facts may flow into a stage;
- how retry, waiting, blocked, failed, stale, cancelled, and recovery-required states differ;
- which decisions were accepted, rejected, superseded, or reverted;
- which state may be resumed and which state can only be reconstructed;
- whether procedural workflow completion is distinct from candidate verification and human acceptance.

This foundation must exist before later Model Mesh or durable runtime work so provider/runtime history never becomes the canonical workflow truth merely because a session can be resumed.

## 5. Spec 008 specification scope

The specification may define implementation-agnostic user scenarios, invariants, security/non-goals, and measurable acceptance criteria for:

- first-class `WorkflowRun` and `StageRun` identity and lifecycle semantics;
- unique stage-attempt identity and explicit actor/run-instance binding;
- deterministic stage states and transition rules for prepared, active, waiting approval, waiting external condition, blocked, failed, stale, cancelled, completed, and recovery-required truth;
- explicit proof levels for live continuation, native-runtime resume, Winds reconstruction, reassignment, and ownership loss;
- artifact-baseline and freshness rules so stale artifacts cannot silently satisfy a newly prepared stage;
- stage-specific context/handoff minimization from canonical structured state and exact upstream artifacts rather than replaying transcript history as authority;
- bounded retry and no-progress semantics, including explicit treatment of repeated candidate hashes, repeated failure classes, repeated non-progress checkpoints, and exhausted budgets;
- append-only `DecisionRecord` semantics with actor/source identity, workflow/stage binding, candidate/evidence references, rationale, outcome, and supersession lineage;
- preservation of failed, rejected, reverted, superseded, and stale decisions as auditable history;
- versioned workflow-state validation, corruption detection, migration/recovery requirements, and fail-closed handling that preserves unreadable state rather than silently reinitializing it;
- durable-write redaction requirements and explicit omission/incompleteness markers so redaction never masquerades as complete evidence;
- explicit separation among `STAGE_COMPLETE`, `WORKFLOW_COMPLETE`, `VERIFICATION_PENDING`, `VERIFIED`, and `HUMAN_ACCEPTED`;
- operator-facing CLI/TUI requirements that explain active stage, owner, candidate/evidence binding, blocker, required approval/external condition, resume behavior, and reconstruction loss;
- deterministic/platform acceptance only where the exact accepted slice directly exercises the claimed platform/domain.

The specification must remain implementation-agnostic. It may require durable local workflow-state semantics, but it must not select a database, schema engine, background service, provider SDK, or framework merely because research references use one.

## 6. Explicit Spec 008 non-goals

Unless a later accepted amendment changes this boundary, Spec 008 does not authorize:

- persistent background owner, daemon, server, socket, IPC, HTTP/SSE/WebSocket control plane, or public runtime protocol;
- remote execution, remote/mobile continuation, team/cloud control plane, or service orchestration;
- provider/model APIs, Model Mesh, automatic provider routing, credential brokerage, or new provider authentication;
- browser automation, browser profiles, CDP, screenshots as verification, or Browser Twin;
- MCP runtime, ACP dependency expansion, A2A, or generic runtime/plugin frameworks;
- learning, skill optimization/promotion, experiment-plane activation, protected-holdout execution, model training, fine-tuning, RL, or automatic policy mutation;
- vector/RAG memory, semantic memory engine, or transcript-as-canonical-memory behavior;
- automatic candidate selection, merge, rebase, cherry-pick, push, PR creation, or landing;
- donor runtime/code import merely because LoopForge, SkillHone, or another research source informed the design;
- dependency, lockfile, migration, schema-runtime, or storage-engine adoption before a later accepted Plan/Tasks unit explicitly qualifies the exact need, version, license, provenance, security boundary, portability, and removal path;
- weakening any accepted Spec 003/006/007 Git, evidence, authority, platform, privacy, recovery, terminal-lifecycle, or human-landing invariant.

## 7. Research-source boundary

LoopForge and SkillHone remain design references, not admitted runtime dependencies or source code.

Any later reuse requires a separate exact-source provenance decision that records, at minimum:

- exact upstream repository and immutable revision;
- exact paths/slices inspected or proposed for reuse;
- license and notice obligations;
- whether reuse is conceptual, adapted, or copied;
- the smallest coherent reuse mode;
- security and authority impact;
- deterministic/platform tests for the Winds-owned behavior;
- an explicit removal/replacement path where the dependency is load-bearing.

`SOURCE_FOUND != SOURCE_ADMITTED` remains binding.

## 8. Provisional future sequence

This is a governance ordering decision, not Tasks authorization for downstream specifications:

```text
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

Later specifications remain non-authorized research ordering until each receives its own accepted entry/specification authority.

## 9. Entry acceptance gate

This entry gate may land only if the exact final candidate proves:

- changed scope is governance/research documentation only;
- canonical base includes the complete Spec 007 T100 closeout and the accepted post-Spec-007 research reconciliation;
- no production source, dependency, lockfile, migration, workflow-semantic implementation, runtime, provider, browser, daemon, IPC, learning, remote-execution, or automatic-landing mutation;
- repository `quality` succeeds on the exact final head;
- correctness/governance/evidence-integrity review passes;
- Ponytail/YAGNI review finds no unjustified scope or premature architecture selection;
- an independent reviewer challenges the exact candidate, scope boundary, roadmap ordering, and non-authorization claims;
- zero unresolved material findings/threads;
- final base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head landing;
- post-merge canonical `main`/tree verification and every actually triggered applicable push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_007_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_008_FORMAL_SPEC_AUTHORIZED=YES
SPEC_008_PLAN_AUTHORIZED=NO
SPEC_008_TASKS_AUTHORIZED=NO
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IPC_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
BROWSER_RUNTIME_AUTHORIZED=NO
PROVIDER_MESH_IMPLEMENTATION_AUTHORIZED=NO
LEARNING_IMPLEMENTATION_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

## 10. Entry effect

If this entry gate closes canonically, the next authorized repository unit is a separate Spec 008 `spec.md` candidate only.

The specification candidate must be written from the then-current canonical main, must not assume this entry gate is implementation authority, and must pass the repository's exact-candidate acceptance gates before any Plan may begin.
