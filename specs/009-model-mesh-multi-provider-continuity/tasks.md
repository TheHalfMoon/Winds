# Tasks: Model Mesh & Explicit Multi-Provider Continuity

## Canonical Inputs

- Constitution 1.1.0: canonical.
- Spec 009 specification: canonical via PR #167.
- Spec 009 Plan: canonical via PR #168.
- Tasks base: `3e1d13730b5ccae41e997515653fb1546ef66f70`.
- Canonical Plan tree: `b90e0357437f97bf39eaafd51b658109336d46b8`.
- Post-Plan `quality #1209` attempt 1: PASS on `ubuntu-latest` and `macos-latest`.

This file decomposes Spec 009 into independently reviewable, dependency-ordered implementation slices. It does not itself add a dependency, migration, source/runtime behavior, provider API/SDK, credential mechanism, gateway, daemon/IPC path, remote/browser execution, learning subsystem, policy engine, donor code, or automatic landing behavior.

At this Tasks base:

```text
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=CLOSED_CANONICAL
SPEC_009_TASKS_AUTHORIZED=YES
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
```

Canonical acceptance and post-merge verification of this file authorize **T114 only**. Every later task remains unauthorized until its exact predecessor closes canonically.

Spec 006 live-runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Nothing in Spec 009 upgrades those separate lanes.

## Global Rules

1. Execute tasks strictly in dependency order. Only the next dependency-satisfied task is authorized.
2. Every implementation task starts from exact then-current canonical `main`; live repository/GitHub truth overrides recorded historical hashes when main legitimately moves.
3. HEAD/TREE movement invalidates candidate-bound CI, review, benchmark, and merge-ready evidence. Requalify the new exact candidate rather than inheriting stale evidence.
4. Preserve canonical workspace/workstream/workflow/stage/session/candidate/artifact/evidence/decision identities. Model Mesh adds no second work identity namespace.
5. Preserve `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`; target/continuity success never implies candidate verification, human acceptance, or landing.
6. Preserve `RuntimeKind::{Codex, Claude}` as the first-slice runtime-family boundary. Runtime, provider, model, Winds session, actor binding, native session, workflow, and stage identities remain distinct.
7. Provider/model identity MUST NOT be inferred from runtime kind, executable name, display label, model output, native session ID, environment label, or mutable UI text.
8. Explicit target selection never expands execution, delegation, filesystem, network, secret, verification, Git, landing, or human-decision authority.
9. No silent provider/model fallback, ranking, score, winner selection, availability-driven alternate selection, or hidden target mutation.
10. First-slice target selection is HUMAN only. `EXPLICIT_POLICY` remains fail-closed as `POLICY_NOT_AUTHORIZED`; enabling policy selection requires a canonical Spec/Plan/Tasks amendment.
11. Exact target authority is content-bound. Generic T076 approval content, generic workflow decisions, labels, rationale text, or same-stage approvals for another target/binding/session/role cannot authorize Model Mesh selection or continuity.
12. `ModelMeshTargetDescriptorV1` must remain reconstructible from durable canonical truth: canonical work/session joins plus immutable target-request scope, including exact actor binding, Winds session, actor role, runtime, provider, and model dimensions.
13. The target-request `actor_role` is bounded immutable scope only. It does not create actor identity or authority; the current authority ceiling remains independently evaluated.
14. Provider/model/native-session claims remain source-labelled. Unknown/unavailable/conflicting/stale truth is explicit and never replaced by a guess.
15. Native resume, reconstruction, reassignment, handoff, ownership loss, unavailable, and unproven continuity remain distinct. Cross-runtime handoff is never native resume.
16. Spec 006 durable runtime/session mappings alone do not prove live native ownership after restart; existing live-runtime nonclaims remain unchanged.
17. Existing T081 Claude-Planner -> Codex-Worker behavior remains a compatibility surface until an authorized task proves safe reuse of the lower-level direction-neutral seam.
18. Existing `winds.db`/rusqlite is the only selected durable store. No second database, ORM, state service, queue, distributed lease, or event-sourcing framework.
19. T116 owns the single Plan-selected migration `0011_model_mesh_continuity.sql`. After T116 lands canonically, that migration is immutable historical schema. T117-T126 MUST NOT edit it; a later proven schema defect requires an explicit Tasks amendment authorizing a new next-numbered migration.
20. All Model Mesh history is append-only at the semantic level. Later success never rewrites failed, unavailable, conflicting, stale, reconstructed, or superseded historical truth.
21. No raw credential/token/password/cookie/key, provider-private memory, transcript dump, full environment dump, arbitrary provider response, or hidden model state may enter Model Mesh durable records.
22. Authentication readiness remains distinct from runtime/provider/model availability and identity. Executable presence does not imply authenticated readiness.
23. Optional usage/cost metadata remains `UNKNOWN` unless T125 proves an already-authorized structured local observation source. No billing API, pricing DB, provider SDK, external telemetry service, or guessed public price is authorized.
24. No new direct dependency is authorized by T114-T126. Existing `rusqlite`, `serde`, `serde_json`, `sha2`, and accepted runtime/workflow/context/authority/Git modules may be reused only within their accepted roles.
25. No daemon, server, socket, IPC/RPC control plane, remote execution, browser/CDP runtime, MCP/ACP/A2A expansion, generic plugin/provider framework, semantic/vector/RAG memory, training/fine-tuning/RL, learned routing, automatic policy mutation, automatic candidate winner, or automatic Git landing.
26. CLI/status/why-blocked/reviewer/workbench views are projections from canonical state, never independently mutable authority stores.
27. Reviewer handoff excludes source-agent persuasion/confidence/winner recommendations by default while preserving exact required evidence/provenance/authority facts.
28. Every new focused `src/tNNN_*.rs` test module must be registered and demonstrably executed. File existence is not test evidence.
29. Platform claims remain domain-specific and require direct applicable evidence. Linux/macOS/Windows/WSL proof do not substitute for one another.
30. Each task closes only after expected-head guarded normal merge and post-merge verification on canonical `main`. Successor authority is asserted only after that closure.
31. Minimal `src/main.rs` edits are authorized only for exact module/test registration or the task's explicitly authorized CLI dispatch; no unrelated CLI behavior is allowed.
32. A genuinely missing dependency, provider interface, policy mechanism, schema capability, credential path, runtime family, or protocol stops the affected task for canonical amendment rather than being silently added.

## Standard Acceptance Gate

Every implementation task requires on its exact final candidate:

- repository `quality` = SUCCESS on every platform job actually defined by that workflow;
- focused deterministic tests for the task-authorized surface, with proof each new focused module executed;
- applicable existing Spec 003/006/007/008 regression/security/platform checks remain green;
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

Canonical acceptance of this file authorizes T114 only.

```text
T114 -> T115 -> T116 -> T117 -> T118 -> T119 -> T120
     -> T121 -> T122 -> T123 -> T124 -> T125 -> T126
```

A task closes only after guarded landing and post-merge verification. Closing T126 authorizes no later Spec 009 implementation phase.

## Requirement Coverage Map

| Canonical contract | Primary task path | Final evidence path |
| --- | --- | --- |
| FR-001..FR-012 | T114 domain/descriptor/resolver -> T115 observation adapter -> T116 persistence | T123 adversarial -> T126 |
| FR-013..FR-022 | T115 source-labelled runtime/provider/model observation adapters | T123 -> T124 -> T126 |
| FR-023..FR-032 | T114 no-fallback resolver + T115 exact authority adapter + T116 approval persistence | T123 -> T126 |
| FR-033..FR-045 | T118 continuity context/classification -> T119 continuity persistence | T123 -> T124 -> T126 |
| FR-046..FR-058 | T116 append-only stage-bound substrate -> T117 drift/staleness -> T119 | T123 -> T126 |
| FR-059..FR-067 | Global Rules + T115/T116 secret-safe authority/observation boundaries | T123 -> T126 |
| FR-068..FR-074 | T120 read-only projections -> T125 usage/cost necessity gate | T126 |
| FR-075..FR-082 | T120 reviewer projections -> T123 forged-truth campaign -> T124 inherited verification regressions | T126 |
| FR-083..FR-095 | Global Rules + T122/T124/T125 scope/platform/necessity gates | T126 |
| SC-001..SC-018 | T114..T123 focused deterministic qualification | T124 -> T126 |
| SC-019..SC-025 | T124 regression/platform/history + every Standard Acceptance Gate | T126 final reconciliation |

No row grants authority to skip a dependency. The map identifies the primary implementation/evidence path only; every task remains bound by the complete canonical Spec and Plan.

---

## Phase 1 — Closed Model Mesh Domain

### [x] T114 — Canonical target/identity/continuity domain and pure exact resolver

**Purpose**: establish the smallest closed deterministic Model Mesh truth model before adapters, persistence, CLI, or runtime action.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/t114_model_mesh_domain_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- closed exact IDs/types for provider/model identity and explicit `UNSPECIFIED` dimensions;
- versioned `ModelMeshTargetDescriptorV1` containing exact workspace/workstream/workflow/stage, actor binding, Winds session, bounded immutable actor role, runtime kind, provider dimension, and model dimension;
- deterministic canonical serialization and SHA-256 digest helpers using existing `serde`/`serde_json`/`sha2` only;
- closed `TargetRequest`, `IdentityClaim`, source class, identity dimension, authentication/capability truth, and `TargetResolution` states selected by the Plan;
- versioned `ModelMeshContinuityPermissionDescriptorV1` and `ModelMeshAuthorityEnvelopeV1` pure domain/canonicalization types, without persistence;
- pure exact target resolver over explicit input truth, including `UNKNOWN`, `UNAVAILABLE`, `AMBIGUOUS`, `CONFLICT`, `STALE`, `AUTHENTICATION_UNKNOWN`, `CAPABILITY_UNAVAILABLE`, `AUTHORITY_DENIED`, and `POLICY_NOT_AUTHORIZED`;
- exact resolver input keeps target identity applicability, content-bound approval applicability, authentication readiness, capability, and current execution authority as separate dimensions;
- no provider invocation, store access, process spawn, network, credential access, or Git mutation.

**Acceptance**:
- canonical descriptor bytes/digest stable for identical normalized input and change on every material target/scope dimension;
- actor-role-only movement changes the digest;
- provider/model unspecified stays unspecified and cannot be guessed from runtime kind;
- agent-reported identity cannot satisfy a Winds-observed requirement;
- unavailable/ambiguous/conflicting targets never choose an alternate winner;
- first-slice `EXPLICIT_POLICY` always fails closed;
- exact identity match never creates execution, verification, acceptance, or landing authority;
- complete negative-state matrix is deterministic and focused T114 tests are registered/executed;
- no persistence, source adapter, CLI behavior, or new dependency.

**Depends on**: canonical Tasks acceptance. **Closes to authorize**: T115.

---

## Phase 2 — Accepted Observation and Authority Adapters

### [x] T115 — Runtime/actor/source-labelled observation and authority revalidation adapters

**Purpose**: adapt already-accepted Winds runtime/workflow/authority truth into pure Model Mesh inputs without creating new observations, persistence, or execution authority.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/agentic_runtime.rs` only for narrow read/revalidation adapters over accepted truth
- `src/workflow.rs` only for narrow canonical identity/actor-context adapters
- `src/agentic_authority.rs` only for narrow Model Mesh approval parsing/revalidation helpers that do not write state
- `src/t115_model_mesh_adapter_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- derive runtime identity only from accepted `RuntimeDiscovery`/runtime-binding truth;
- reconstruct exact workspace/workstream/workflow/stage/actor-binding/Winds-session scope from canonical existing identities where available;
- convert already-authorized structured provider/model observations into source-labelled claims only when their provenance is explicit; otherwise return unknown;
- never infer OpenAI from Codex or Anthropic from Claude;
- parse and digest-validate dedicated `ModelMeshAuthorityEnvelopeV1` canonical JSON from an already-loaded approval record while rejecting generic T076 approval JSON as Model Mesh authority;
- compare envelope target/role/binding/session/work scope to exact reconstructed/current target descriptor input;
- expose current authority-ceiling evaluation as a separate input; an approval cannot exceed it;
- no new provider call, login, credential read, database write, runtime owner, or policy engine.

**Acceptance**:
- runtime label/model prose/native ID cannot manufacture provider/model identity;
- declaration/local/agent claim conflicts remain source-labelled conflict rather than winner selection;
- missing provider/model observation remains unknown;
- approval for target/binding/session/role A cannot authorize B;
- malformed/wrong-schema/generic/stale-digest Model Mesh approval fails closed;
- exact approval match does not override a denied current authority ceiling;
- existing runtime/workflow/authority truth is reused without semantic duplication;
- focused T115 tests registered/executed; no persistence schema change or new dependency.

**Depends on**: T114 `CLOSED_CANONICAL`. **Closes to authorize**: T116.

---

## Phase 3 — Append-Only SQLite Substrate

### [x] T116 — `0011_model_mesh_continuity.sql`, exact target/claim persistence, and Model Mesh approval storage

**Purpose**: qualify the complete Plan-selected Spec 009 schema once and expose only the persistence behavior needed by target requests/identity claims/approval basis in this slice.

**Authorized paths**:
- `migrations/0011_model_mesh_continuity.sql`
- `src/store.rs`
- `src/model_mesh.rs`
- `src/agentic_authority.rs` only for the narrow dedicated Model Mesh approval write/load/revalidation path over the existing approval table
- `src/t116_model_mesh_store_tests.rs`
- `src/main.rs` only for module/test registration
- a narrowly scoped T116 evidence artifact if required

**Required implementation**:
- one idempotent migration creating the complete minimal Plan-selected relations: `model_mesh_target_requests`, `model_mesh_identity_claims`, `model_mesh_continuity_events`, and `model_mesh_continuity_identity_claims`;
- complete indexes, foreign keys, bounded known enums, exclusive request-vs-actor claim subject constraints, append-only update/delete guards, and hierarchy/role association triggers selected by the Plan;
- `model_mesh_target_requests` persists exact stage, actor binding, bounded immutable actor role, runtime/provider/model request dimensions, HUMAN selector, target descriptor digest, exact selection approval, and creation time;
- insertion validates actor binding -> stage -> workflow -> workstream/workspace and concrete Winds session consistency atomically;
- insertion recomputes descriptor digest from durable request fields plus canonical joins and validates exact Model Mesh `TARGET_SELECTION` approval content in the same transaction;
- dedicated Model Mesh approval canonicalization/write/load/revalidation reuses `agentic_delegation_approvals` unchanged and remains distinct from generic T076 `ApprovalContent`;
- identity claims persist exact source/dimension/value/basis and optional accepted runtime binding with exclusive request/actor subject;
- `model_mesh_continuity_events` is created now with the complete frozen Plan-selected substrate: `continuity_event_id` primary key, `target_request_id` foreign key, optional `source_actor_binding_id` and `destination_actor_binding_id` foreign keys, closed `continuity_class`, optional bounded SHA-256 `context_digest`, bounded known `completeness_state`, optional bounded SHA-256 `continuity_permission_digest`, optional `authority_approval_id` foreign key to `agentic_delegation_approvals`, closed `authority_claim` (`REQUIRED` | `NO_AUTHORITY_CLAIM`), and `created_unix_ms`;
- `model_mesh_continuity_identity_claims` is created now with `continuity_event_id`, closed `actor_role` (`SOURCE` | `DESTINATION`), `identity_claim_id`, and primary key `(continuity_event_id, actor_role, identity_claim_id)`;
- 0011 schema constraints/triggers prove that every continuity-event source/destination actor binding is in the applicable stage lineage or qualified source-stage handoff relation, and reject orphan/cross-workflow/cross-stage coincidence;
- role-association constraints/triggers require a `SOURCE` association to reference an ACTOR-subject identity claim bound to the event's exact `source_actor_binding_id`, and a `DESTINATION` association to reference an ACTOR-subject claim bound to the exact `destination_actor_binding_id`; any referenced runtime binding must remain consistent with that actor's canonical Winds-session/runtime context;
- continuity authority columns have a closed nullability contract: `authority_claim=REQUIRED` requires both `continuity_permission_digest` and `authority_approval_id`; `authority_claim=NO_AUTHORITY_CLAIM` requires both to be absent and cannot represent execution permission; digest columns, when present, are exactly bounded lowercase SHA-256 hex values;
- 0011 contains the structural foreign-key/check/trigger substrate needed to bind a required continuity approval and operation digest, while T119 remains the first task authorized to expose production Store/domain continuity-event insertion/revalidation behavior;
- T116 schema-qualification fixtures exercise continuity tables directly at the migration/schema boundary only: they prove a valid REQUIRED row shape and a valid NO_AUTHORITY_CLAIM observation row shape are representable, while wrong actor-role association, wrong source/destination actor, orphan binding/approval, invalid authority-claim nullability, malformed digest, and invalid lineage are rejected before 0011 can freeze;
- continuity-event insertion behavior remains otherwise inert until T119; T119 MUST be implementable over the frozen 0011 schema with zero schema modification;
- schema-integrity validation recognizes every required 0011 table/column/index/foreign-key/check/trigger object and fails closed on missing/modified/partial definitions;
- replay/idempotency identity for target requests/claims is deterministic and cannot manufacture duplicate current truth.

**Acceptance**:
- migration applies/reapplies idempotently and proves complete table/index/trigger/constraint inventory;
- append-only updates/deletes rejected at database boundary;
- cross-stage/cross-workflow/cross-session actor coincidence rejected;
- role mismatch and approval-target/binding/session mismatch rejected;
- complete frozen continuity-event column inventory is proven, including source/destination actor bindings, context/completeness, continuity permission digest, approval foreign key, authority claim, and created time;
- continuity association primary key plus SOURCE/DESTINATION actor-claim triggers are proven, including rejection of wrong actor, wrong claim subject, wrong runtime-binding context, and invalid stage/workflow lineage;
- `REQUIRED` authority rows reject missing approval/digest and malformed digests; `NO_AUTHORITY_CLAIM` rows reject approval/digest presence and remain structurally incapable of claiming permission;
- schema-bound fixtures prove both valid continuity row classes can be represented without exposing T119 production event APIs, and prove T119 requires no 0011 mutation;
- generic approval/decision text cannot authorize a target request;
- request replay with identical canonical identity is idempotent; collision with different meaning fails;
- restart preserves exact target/claim history and stale/failed history;
- no credential/provider-private payload persisted;
- no second database/dependency;
- focused T116 tests registered/executed.

**Depends on**: T115 `CLOSED_CANONICAL`. **Closes to authorize**: T117.

**Schema freeze after closure**: once T116 is `CLOSED_CANONICAL`, `0011_model_mesh_continuity.sql` is immutable. Later correction requires a canonical Tasks amendment and next-numbered migration.

---

## Phase 4 — Deterministic Drift and Staleness

### [x] T117 — Exact target/identity/authority drift evaluator

**Purpose**: make applicability fail truthfully when any observation or authority basis moves, without deleting historical records.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/store.rs` only for read/query helpers over the landed 0011 schema; no migration edit
- `src/t117_model_mesh_drift_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- pure/deterministic drift evaluation for runtime executable identity/version, provider/model claim movement/conflict, native-session identity/ownership, workflow/stage attempt, actor binding/Winds session/role, candidate/artifact/evidence freshness, Model Mesh approval digest/applicability, and current authority ceiling;
- staleness applies only to dependent claims/permission and preserves historical records;
- exact candidate/artifact/evidence freshness reuses Spec 008 evaluators rather than copying their logic;
- current target projection derives from applicable append-only records; no mutable singleton.

**Acceptance**:
- each material identity/scope/authority movement deterministically stales the exact dependent truth;
- irrelevant movement does not stale unrelated claims;
- historical failed/unavailable/conflicting/stale requests remain inspectable after requalification;
- same inputs always produce same drift result;
- no re-execution, fallback, destructive repair, or schema mutation;
- focused T117 tests registered/executed.

**Depends on**: T116 `CLOSED_CANONICAL`. **Closes to authorize**: T118.

---

## Phase 5 — Direction-Neutral Continuity Context

### [x] T118 — Continuity classifier and canonical cross-runtime context projection

**Purpose**: generalize continuity evaluation over accepted Spec 006/008 context/reconstruction seams without manufacturing live ownership or provider-private memory.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/agentic_context.rs` only for the smallest lower-level provider-neutral projection seam
- `src/workflow.rs` only for accepted reconstruction/actor inputs
- `src/agentic_runtime.rs` only for accepted native-resume observation inputs
- `src/t118_model_mesh_continuity_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- closed continuity classes `NATIVE_RESUME`, `RECONSTRUCTED`, `REASSIGNED`, `HANDOFF`, `OWNERSHIP_LOST`, `UNAVAILABLE`, `UNPROVEN`;
- `NATIVE_RESUME` only when existing accepted Spec 006 proof actually supports it;
- cross-runtime continuity always classified as handoff/reconstruction as applicable, never native resume;
- provider-neutral continuity context from canonical workflow/stage/work identity, target reference, exact candidate/artifact/evidence refs, content-bound Model Mesh approval basis/current authority ceiling, bounded reconstruction/transfer report, and explicit unavailable/redacted/material-loss markers;
- deterministic context digest; provider-private state unavailable unless safely/provably observed;
- source persuasion/confidence/winner recommendation excluded from independent reviewer context by default;
- T081 existing behavior preserved as a regression/compatibility contract.

**Acceptance**:
- Codex->Claude, Claude->Codex, Codex->Codex, Claude->Claude deterministic fixtures retain exact provenance and truthful continuity class;
- restart/native-ID coincidence never becomes native resume proof;
- missing required context blocks/downgrades continuity rather than being filled from prose;
- context digest moves on material structured-context movement;
- T081 compatibility remains green;
- no provider call, transcript replay, credential/private-memory extraction, or new dependency;
- focused T118 tests registered/executed.

**Depends on**: T117 `CLOSED_CANONICAL`. **Closes to authorize**: T119.

---

## Phase 6 — Continuity Event Persistence

### [x] T119 — Append-only continuity events, role-specific actor claims, and permission binding

**Purpose**: activate the already-landed continuity schema with exact source/destination identity and content-bound permission semantics.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/store.rs` using the immutable landed 0011 schema only
- `src/agentic_authority.rs` only for exact Model Mesh continuity-approval validation
- `src/t119_model_mesh_event_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- atomic continuity-event insertion bound to exact target request, source/destination actor bindings, continuity class, context digest/completeness, authority-claim state, and optional exact continuity permission digest/approval;
- `SOURCE` identity-claim associations only to the event source actor and `DESTINATION` only to destination actor;
- mutating continuation requires `authority_claim=REQUIRED` and an exact digest-valid `CONTINUITY_PERMISSION` Model Mesh approval matching target descriptor, operation descriptor, actor scope, and applicable context digest;
- observation-only unavailable/unproven events may use `NO_AUTHORITY_CLAIM` and can never project execution permission;
- replay/idempotency cannot duplicate current ownership/handoff truth;
- no 0011 migration mutation.

**Acceptance**:
- cross-actor/runtime-binding/stage-lineage claim association rejected;
- unknown provider/model remains explicit per actor;
- approval for a different continuity class/context/target/actor fails closed;
- no-authority observation cannot be mistaken for authorized execution;
- failed/stale continuity event remains historical after later success;
- restart preserves event/association identity;
- focused T119 tests registered/executed.

**Depends on**: T118 `CLOSED_CANONICAL`. **Closes to authorize**: T120.

---

## Phase 7 — Read-Only Operator and Reviewer Projections

### [x] T120 — Target/status/why-blocked/continuity/reviewer projections

**Purpose**: expose deterministic Model Mesh truth without creating a second mutable UI/authority state.

**Authorized paths**:
- `src/model_mesh.rs`
- `src/workflow.rs` / existing evidence modules only for narrow read-only projection reuse
- `src/t120_model_mesh_projection_tests.rs`
- `src/main.rs` only for module/test registration

**Required implementation**:
- side-effect-free `ModelMeshTargetProjection`, `ModelMeshContinuityProjection`, `ModelMeshWhyBlockedProjection`, and `ReviewerContinuityProjection` or equivalent narrow concrete structures;
- expose exact workflow/stage/actor-binding/Winds-session/role and requested runtime/provider/model;
- source-labelled requested/observed/declared/agent identity truth and explicit blocker category;
- runtime/native-session freshness, continuity class, source/destination identity, context completeness/loss, candidate/artifact/evidence freshness;
- authentication readiness, exact Model Mesh approval basis/digest match, current authority ceiling, verification, and human acceptance remain separate fields;
- usage/cost remains explicit unknown unless later T125 qualifies a source;
- reviewer view excludes source persuasion/winner recommendations by default.

**Acceptance**:
- opening/projection cannot execute/resume/handoff/request credentials/change target/mutate authority/Git state;
- stale/unknown/unavailable/conflict/denied states are distinguishable;
- `TARGET_MATCH`, `EXECUTION_AUTHORIZED`, `CONTINUITY_PROVEN`, `VERIFIED`, `HUMAN_ACCEPTED`, and `LANDED` are never collapsed;
- reviewer projection remains candidate/evidence freshness-sensitive;
- focused T120 tests registered/executed.

**Depends on**: T119 `CLOSED_CANONICAL`. **Closes to authorize**: T121.

---

## Phase 8 — Narrow CLI Integration

### [x] T121 — Existing-style Model Mesh CLI inspection and explicit request surfaces

**Purpose**: make qualified Model Mesh truth usable from the existing CLI without implicit provider/runtime execution.

**Authorized paths**:
- `src/main.rs` for exact collision-reviewed dispatch
- `src/model_mesh.rs`
- a narrow new `src/model_mesh_cli.rs` if existing CLI organization requires it
- `src/t121_model_mesh_cli_tests.rs`
- no migration edit

**Required implementation**:
- collision-reviewed commands to inspect admitted runtime target availability without executing;
- explicit stage/actor-bound HUMAN target-request recording only through exact authorized persistence path;
- inspect target resolution/why blocked, source-labelled identity truth, drift/staleness, continuity class/context loss/source/destination binding, and reviewer continuity summary;
- machine-readable output bounded and deterministic where existing CLI conventions support it;
- no implicit target fallback, provider call, credential operation, runtime launch, handoff execution, Git mutation, or acceptance action.

**Acceptance**:
- command spelling does not collide with existing CLI surfaces;
- malformed/oversized/unknown input fails closed;
- status/why-blocked are non-mutating;
- explicit request cannot exceed exact current authority/approval scope;
- HUMAN vs unavailable policy path remains visible;
- focused T121 CLI tests registered/executed.

**Depends on**: T120 `CLOSED_CANONICAL`. **Closes to authorize**: T122.

---

## Phase 9 — Workbench Necessity Gate

### [x] T122 — Decide whether read-only Workbench Model Mesh rendering is necessary

**Purpose**: prevent speculative TUI expansion; add a read-only surface only if CLI qualification proves a concrete operator gap.

**Authorized paths**:
- a T122 necessity/evidence artifact;
- `src/workbench/*` only if the evidence artifact proves the read-only surface necessary;
- `src/t122_model_mesh_workbench_tests.rs` only if implementation is selected;
- `src/main.rs` only for registration needed by that selected surface

**Required decision**:
- compare T121 CLI capabilities against Spec 009 operator requirements;
- if CLI satisfies requirements, record `WORKBENCH_MODEL_MESH_NECESSARY=NO` and add no Workbench behavior;
- if a material discoverability/inspection gap remains, authorize only the smallest read-only rendering of already-canonical projections;
- no Workbench-owned target state, provider execution, credential UI, routing controls, daemon/session owner, or automatic action.

**Acceptance**:
- necessity decision is evidence-backed, not aesthetic preference;
- if NO, no Workbench source changes;
- if YES, rendering is read-only, keyboard/findability/accessibility conventions remain intact, and focused tests are registered/executed;
- existing Spec 007 workbench lifecycle/host-safety/accessibility truth remains unchanged.

**Depends on**: T121 `CLOSED_CANONICAL`. **Closes to authorize**: T123.

---

## Phase 10 — Adversarial Model Mesh Truth Campaign

### [x] T123 — Forged identity, authority replay, secret, corruption, and history campaign

**Purpose**: attack the complete deterministic Model Mesh truth model before platform/final qualification.

**Authorized paths**:
- `src/t123_model_mesh_adversarial_tests.rs`
- `src/main.rs` only for focused-test registration
- T123 evidence artifacts
- minimal forward-only repair within already-authorized Spec 009 source surfaces if the campaign proves a material implementation defect; any repair moves the candidate and restarts exact-head qualification
- no migration edit

**Campaign coverage**:
- runtime labels/model prose forging provider/model identity;
- provider/model Unicode/case/whitespace/oversize identifier collisions;
- unspecified dimensions being silently filled;
- ambiguous/unavailable target with tempting alternate;
- generic T076 approval, generic workflow decision, forged approval-shaped JSON, stale digest, wrong purpose, wrong target, wrong role, wrong binding/session/stage/work scope;
- two bindings in same stage/role but different sessions;
- same binding/session/target with role-only mismatch;
- current authority ceiling reduced after approval;
- replay/duplicate target requests, claims, events, and role associations;
- provider/model/runtime/native-session/stage/candidate/artifact/authority drift;
- forged native resume after restart or ownership loss;
- missing/redacted/incomplete context and provider-private-memory claims;
- secret-like values in durable Model Mesh payload attempts;
- malformed/unknown enum/schema/partial/orphan/corrupt state;
- source-agent `PASS`, `VERIFIED`, `ACCEPTED`, cost, winner, authority, and continuity labels;
- later target success attempting to erase earlier unavailable/failure history.

**Acceptance**:
- zero false target match, false observed identity, false authority, false native resume, false verified/accepted/landed truth, silent fallback, secret persistence, destructive recovery, or historical rewrite;
- no newly discovered material defect remains unresolved;
- focused T123 tests registered/executed and full regression green on exact candidate.

**Depends on**: T122 `CLOSED_CANONICAL`. **Closes to authorize**: T124.

---

## Phase 11 — Inherited Regression and Platform Qualification

### [x] T124 — Spec 003/006/007/008 regression and platform-bound qualification

**Purpose**: prove Model Mesh integration does not weaken inherited canonical terminal/runtime/workbench/workflow/evidence/authority boundaries.

**Authorized paths**:
- T124 evidence artifacts;
- `src/t124_model_mesh_regression_tests.rs` if focused glue is needed;
- `src/main.rs` only for test registration;
- minimal forward-only repair inside already-authorized Spec 009 implementation surfaces only if a new regression is proven; candidate movement restarts qualification
- no migration edit

**Required evidence**:
- repository `quality`;
- applicable Spec 003 terminal lifecycle/recovery regression;
- Spec 006 runtime identity/session/continuity/evidence/authority regression and unchanged live-runtime nonclaims;
- Spec 007 workbench host-safety/navigation/accessibility regression only where shared surfaces are affected;
- Spec 008 workflow/stage/actor/reconstruction/candidate/evidence/decision/reviewer regression;
- platform claims limited to directly exercised domains;
- persistence/domain fixture success is not relabelled real provider/model execution or native-resume proof.

**Acceptance**:
- no inherited invariant weakened;
- all unexercised runtime/provider/platform claims remain explicit nonclaims;
- no real Claude/Codex worker execution claim manufactured;
- all actually applicable workflows green on exact candidate;
- focused T124 tests, if any, registered/executed.

**Depends on**: T123 `CLOSED_CANONICAL`. **Closes to authorize**: T125.

---

## Phase 12 — Optional Usage/Cost Observation Necessity Gate

### [x] T125 — Qualify an existing structured local usage source or close with `UNKNOWN`

**Purpose**: satisfy optional observability truth without creating billing/provider infrastructure or speculative telemetry.

**Authorized paths**:
- a T125 necessity/provenance/evidence artifact;
- `src/model_mesh.rs` and read-only existing runtime observation adapter only if an already-authorized structured local source is proven suitable;
- `src/t125_model_mesh_usage_tests.rs` only if an observation adapter is selected;
- `src/main.rs` only for test registration;
- no migration edit unless a separately accepted Tasks amendment explicitly authorizes a new next-numbered migration

**Required decision**:
- inspect only already-authorized structured local observations available in canonical code/runtime fixtures;
- if no production-qualified source exists, record `MODEL_MESH_USAGE_OBSERVATION=UNKNOWN` and implement no new collection path;
- if a suitable source exists, adapt only bounded source-labelled observation with explicit provenance/freshness;
- no public-price guessing, prose-derived tokens, billing API, pricing DB, provider SDK, external telemetry, or routing authority.

**Acceptance**:
- unknown remains truthful and is sufficient when no source is qualified;
- observed usage/cost, if any, never becomes verification, human acceptance, routing, winner, or execution authority;
- any adapter is deterministic, source-labelled, bounded, and secret-safe;
- no unauthorized persistence/dependency/provider call;
- focused tests registered/executed if implementation occurs.

**Depends on**: T124 `CLOSED_CANONICAL`. **Closes to authorize**: T126.

---

## Phase 13 — Final Spec 009 Reconciliation and Closeout

### [x] T126 — Spec 009 final acceptance, evidence reconciliation, and program closeout

**Purpose**: reconcile every Spec 009 requirement/success criterion against canonical implementation evidence and close only what is actually proven.

**Authorized paths**:
- this `tasks.md` for final checked-state reconciliation;
- `specs/009-model-mesh-multi-provider-continuity/t126-final-reconciliation.md`;
- focused Spec 009 acceptance/evidence artifacts following repository precedent;
- README/docs corrections only for claims proven by canonical evidence;
- no production/runtime/dependency/schema behavior change.

**Acceptance**:
- T114..T125 reconciled against exact canonical merges and post-merge verification;
- FR-001..FR-095 each classified as deterministic, platform-bound, governance-boundary, or explicit truthful nonclaim/deferment where the canonical Spec permits it;
- SC-001..SC-025 each reconciled to exact evidence;
- canonical target descriptors remain reconstructible and exact authority remains content-bound to target/binding/session/role/current ceiling;
- source-labelled identity, no-fallback routing, drift/staleness, continuity classification/context, append-only history, secret/auth posture, reviewer independence, and corrupt-state preservation remain proven;
- migration 0011 and all selected schema objects reconciled; no unauthorized schema mutation, dependency, provider API/SDK, gateway, policy engine, daemon/IPC, remote/browser path, learning/routing subsystem, second store, or automatic Git action exists;
- optional usage/cost state reconciled truthfully, including `UNKNOWN` if T125 proves no qualified source;
- platform/runtime/provider claims limited to directly exercised evidence and Spec 006 live-runtime nonclaims remain unchanged unless independently governed elsewhere;
- historical failed/rejected/superseded/stale evidence remains material and inspectable, including Plan review findings and superseded Plan heads;
- final exact implementation state has repository quality, applicable regression/platform/security evidence, author review, Ponytail/YAGNI review, and fresh independent substantive review with zero unresolved material findings;
- exact main/base/head/tree/scope/ruleset/mergeability reconciliation before guarded landing;
- guarded normal landing with exact expected head;
- post-merge canonical main/tree/parents/signature and every actually-triggered push check verified.

**Completion state, only if proven after canonical landing**:

```text
T114..T126=CLOSED_CANONICAL
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=CLOSED_CANONICAL
SPEC_009_TASKS=CLOSED_CANONICAL
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

Closing T126 authorizes no later Spec 009 phase and does not automatically authorize broader providers, policy routing, provider APIs/SDKs, credentials, gateway, daemon/IPC, remote/browser execution, learning, semantic memory, new dependencies, automatic winner selection, or automatic landing.

**Depends on**: T125 `CLOSED_CANONICAL`. **Closes to authorize**: no successor implementation task.

## Tasks Acceptance Gate

This Tasks file may land only if all are true on its exact final candidate:

- canonical base is post-Plan `main` `3e1d13730b5ccae41e997515653fb1546ef66f70` unless live truth legitimately moves before candidate creation, in which case this file/base metadata must be reconciled forward-only and all candidate-bound qualification restarted;
- changed scope is exactly `specs/009-model-mesh-multi-provider-continuity/tasks.md` unless an explicitly justified governance-only correction is needed;
- no implementation, Cargo/lockfile, source, migration, workflow, runtime/provider execution, credential, gateway, browser, daemon/IPC, remote, learning, policy-engine, plugin, protocol, or automatic landing change occurs in the Tasks PR;
- no new direct dependency is authorized by T114-T126;
- every FR-001..FR-095 and SC-001..SC-025 has a plausible dependency-ordered implementation/evidence path;
- canonical acceptance authorizes T114 only; T115..T126 remain dependency-blocked;
- migration ownership is exactly T116 and landed 0011 is immutable afterward;
- first-slice `EXPLICIT_POLICY` remains fail-closed and Spec 006 live-runtime nonclaims remain unchanged;
- exact-head repository `quality` succeeds;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- final exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- expected-head guarded normal merge;
- canonical merge identity/tree/ordered parents/signature verified;
- every actually-triggered applicable post-merge push check succeeds before T114 authority is asserted.

Only after this Tasks file lands canonically and is post-merge verified may repository truth state:

```text
SPEC_009_TASKS=CLOSED_CANONICAL
SPEC_009_IMPLEMENTATION_AUTHORIZED=T114_ONLY
T114=AUTHORIZED
T115..T126=BLOCKED_BY_DEPENDENCY
```
