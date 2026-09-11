# Implementation Plan: Model Mesh & Explicit Multi-Provider Continuity

## Summary

Build the smallest one-process, local Model Mesh continuity layer that satisfies canonical Spec 009 while preserving the accepted Spec 003/006/007/008 Git, evidence, authority, runtime-continuity, terminal-lifecycle, workflow, decision, privacy, reviewer-independence, platform, and human-landing boundaries.

The first implementation program will:

1. introduce closed target-request and identity-claim domain types that keep runtime, provider, model, canonical workflow/stage, Winds session, and provider-native session identities separate;
2. reuse the accepted `RuntimeKind::{Codex, Claude}`, runtime discovery, runtime/session bindings, `WorkflowRun`, `StageRun`, actor bindings, reconstruction reports, context capsules, authority evaluation, exact candidate/evidence baselines, and Winds-owned SQLite store rather than create a provider platform or second runtime model;
3. match explicit target requests only against qualified source-labelled observations and fail closed on unknown, unavailable, ambiguous, stale, conflicting, or unauthorized target state;
4. record explicit target-request and continuity history as bounded append-only local records tied to exact `StageRun` attempts and existing actor/runtime bindings;
5. generalize continuity evaluation so `NATIVE_RESUME`, `RECONSTRUCTED`, `REASSIGNED`, `HANDOFF`, `OWNERSHIP_LOST`, `UNAVAILABLE`, and `UNPROVEN` remain separate proof levels without upgrading the still-open Spec 006 live-runtime nonclaims;
6. reuse canonical structured context/reconstruction machinery for direction-neutral Codex/Claude handoff semantics while keeping the existing T081 Claude-Planner -> Codex-Worker contract unchanged until a separately authorized implementation slice proves a safe general seam;
7. make runtime/provider/model/native-session/workflow/stage/candidate/authority drift deterministically stale the exact claims that depend on moved identity;
8. expose non-mutating status, target-resolution, continuity, and why-blocked projections from canonical state rather than persist competing UI truth;
9. keep authentication readiness, credential access, provider/model availability, selection authority, execution authority, verification authority, and human acceptance distinct;
10. retain optional structured usage/cost observations only when an already-authorized local structured source exists; otherwise report unknown and add no billing/pricing/provider dependency;
11. qualify deterministic/adversarial behavior first on the already admitted Codex and Claude runtime families before any broader runtime/provider proposal can claim necessity;
12. prove exact-candidate regression/platform/evidence integrity before final reconciliation.

This Plan selects the existing Winds one-process Rust architecture, existing SQLite/rusqlite store, and existing runtime/workflow/context seams for later Task-stage use. This Plan PR itself adds no migration, source, dependency, lockfile, runtime/provider execution, credential mechanism, gateway, daemon/IPC, remote/browser/learning behavior, or automatic landing behavior.

## Constitution Check

The implementation program MUST preserve:

```text
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
AGENT_COMPLETION_IS_NOT_VERIFICATION
VERIFICATION_IS_NOT_ACCEPTANCE
ACCEPTANCE_IS_NOT_LANDING
VERIFY_THE_EXACT_CANDIDATE
CURRENT_ONE_PROCESS_ARCHITECTURE_PRESERVED
WINDS_SESSION != RUNTIME_SESSION
WORKFLOW_RUN != NATIVE_SESSION
STAGE_RUN != NATIVE_TURN
RUNTIME != PROVIDER
PROVIDER != MODEL
REQUESTED_TARGET != OBSERVED_TARGET
HANDOFF != NATIVE_RESUME
RECONSTRUCTED != RESUMED
REASSIGNED != RESUMED
RUNTIME_DISCOVERY != EXECUTION_AUTHORITY
PROVIDER_AVAILABILITY != AUTHENTICATION_READINESS
MODEL_SELECTION != VERIFICATION_AUTHORITY
TARGET_SELECTION_CANNOT_EXPAND_AUTHORITY
NO_AUTOMATIC_PROVIDER_ROUTING
NO_SILENT_MODEL_SWITCH
NO_MAGIC_WINNER
NO_SILENT_LANDING
```

Additional constitutional constraints:

- canonical workspace, workstream/task, workflow, stage-attempt, session, candidate, artifact, evidence, and decision identities remain inherited truth; Spec 009 does not create a second work identity namespace;
- `RuntimeKind` remains the accepted concrete Codex/Claude runtime family boundary in the first implementation program;
- runtime display names, executable names, native session IDs, provider/model prose, environment labels, transcript content, paths, catalog declarations, and model self-report never determine authority or canonical identity by themselves;
- provider/model target selection never grants execution, delegation, filesystem, network, secret, verification, Git, landing, or human-decision authority;
- repository-native Git/evidence observations remain verification authority;
- Spec 006 durable runtime mapping and native-resume proof rules are reused and never silently promoted;
- Spec 008 workflow/stage/candidate freshness, reconstruction, decision, and human-acceptance semantics are reused and never duplicated into a competing state machine;
- no persistent background owner, daemon, socket, local server, IPC/RPC protocol, remote execution route, browser runtime, generic provider/plugin platform, learning system, semantic/vector memory, or automatic Git landing is introduced;
- every implementation slice must start from exact then-current canonical `main`, remain dependency-ordered, pass deterministic gates, receive author correctness/safety review, Ponytail/YAGNI review, and fresh independent substantive review before guarded landing;
- HEAD/TREE movement invalidates prior exact-candidate qualification.

## Canonical Baseline

Planning base:

```text
BASE=a8525b68d7d5ca83cfd3c475cbae18e131700e07
BASE_TREE=bba395516cead404582eed8183b2be9c9b03e159
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=IN_QUALIFICATION
SPEC_009_TASKS_AUTHORIZED=NO
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
POST_SPEC_MERGE_QUALITY=#1203 ATTEMPT_1 PASS
```

Inherited canonical seams to reuse rather than duplicate:

- `RuntimeKind::{Codex, Claude}` and bounded runtime discovery in `src/agentic_runtime.rs`;
- `RuntimeDiscovery`, `RuntimeExecutableIdentity`, `RuntimeVersionEvidence`, source-labelled capability evidence, and `AuthReadiness::Unknown` truth;
- `RuntimeSessionBinding`, native session identity, ownership truth, and exact runtime identity revalidation;
- `WorkflowRuntimeContinuationObservation`, which intentionally cannot manufacture physical `RESUMED` truth from a durable candidate;
- `WorkflowRunIdentity` and `StageRunIdentity` in `src/workflow.rs`;
- append-only actor/reconstruction/decision truth and exact candidate/artifact baseline freshness from Spec 008;
- `ContextCapsule`, transfer report, hidden/private-state boundary, and authority-bounded cross-runtime handoff primitives in `src/agentic_context.rs`;
- existing delegation/approval/authority evaluation paths;
- existing `Store`, `winds.db`, WAL mode, foreign-key enforcement, schema-integrity validation, append-only/no-update/no-delete trigger precedents, and migration ordering through `0010_resumable_workflow_ledger.sql`;
- exact Git candidate identity and repository `quality` evidence;
- existing native Windows/ConPTY, WSL2, Linux PTY, and macOS PTY qualification boundaries.

The Spec 006 live-runtime nonclaims remain unchanged unless a separate live-runtime acceptance lane independently proves otherwise:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

## Architecture Decision: One Process, Seven Narrow Seams

Spec 009 adds no generic provider framework. The first implementation should be seven concrete Winds-owned seams inside the existing process.

### 1. Closed Model Mesh target domain

Add a small domain module, expected to be `src/model_mesh.rs` unless Tasks find a clearer existing module boundary.

Logical first-slice types:

```text
ModelMeshTargetDescriptorV1 {
  schema_version: 1,
  workspace_id,
  workstream_id,
  workflow_run_id,
  stage_run_id,
  actor_binding_id,
  winds_session_id,
  actor_role,
  runtime: RuntimeKind,
  provider: UNSPECIFIED | EXACT(ExactProviderId),
  model: UNSPECIFIED | EXACT(ExactModelId),
}

TargetRequest {
  descriptor: ModelMeshTargetDescriptorV1,
  target_descriptor_digest: SHA256(canonical_descriptor_json),
  selector: HUMAN,
}

IdentityClaim {
  dimension: RUNTIME | PROVIDER | MODEL | NATIVE_SESSION,
  value?,
  source: WINDS_LOCALLY_OBSERVED | VENDOR_DECLARED | CATALOG_DECLARED | AGENT_REPORTED | HUMAN_DECIDED | UNAVAILABLE,
  observation_basis?,
}

TargetResolution {
  EXACT_MATCH,
  UNKNOWN,
  UNAVAILABLE,
  AMBIGUOUS,
  CONFLICT,
  STALE,
  AUTHENTICATION_UNKNOWN,
  CAPABILITY_UNAVAILABLE,
  AUTHORITY_DENIED,
  POLICY_NOT_AUTHORIZED,
}

ModelMeshContinuityPermissionDescriptorV1 {
  schema_version: 1,
  workflow_run_id,
  stage_run_id,
  source_actor_binding_id?,
  destination_actor_binding_id?,
  continuity_class,
  target_descriptor_digest,
  context_digest: NOT_APPLICABLE | EXACT(SHA256),
}

ModelMeshAuthorityEnvelopeV1 {
  schema_version: 1,
  purpose: TARGET_SELECTION | CONTINUITY_PERMISSION,
  workspace_id,
  workstream_id,
  session_id,
  workflow_run_id,
  stage_run_id,
  actor_binding_id,
  actor_role,
  target_descriptor_digest,
  continuity_permission_digest?,
}
```

The exact target authority binding is content-bound, not label-bound:

- `ModelMeshTargetDescriptorV1` has one deterministic canonical JSON representation; normalization includes exact workspace/workstream/workflow/stage/actor-binding/Winds-session/actor-role scope and uses explicit `UNSPECIFIED` rather than omitted/guessed provider/model values;
- `target_descriptor_digest` is SHA-256 of that canonical representation and is stored with the target request;
- the first implementation accepts **human selection only**. `EXPLICIT_POLICY` remains specification vocabulary but is fail-closed as `POLICY_NOT_AUTHORIZED` until a separately accepted Plan/Tasks amendment defines an actual policy and its exact inputs/authority semantics;
- human target selection and any authorized mutating continuity operation require a content-bound `agentic_delegation_approvals` row whose `canonical_content_json` uses a dedicated versioned `ModelMeshAuthorityEnvelopeV1` canonicalizer and includes the exact target descriptor digest;
- `ModelMeshContinuityPermissionDescriptorV1` also has one deterministic canonical JSON representation; `CONTINUITY_PERMISSION` approval content includes its digest, binding continuity class, exact source/destination actor bindings as applicable, target descriptor digest, and explicit `NOT_APPLICABLE` versus exact context digest;
- the existing approval table is reused unchanged; Tasks may add a narrow Model Mesh approval canonicalizer/write/load/revalidation helper, but MUST NOT reinterpret generic T076 `ApprovalContent` or a generic workflow decision as Model Mesh target authorization;
- `load_human_approval`-style digest validation remains required: stored `content_digest` must equal SHA-256 of the exact canonical approval JSON before any Model Mesh authority basis is applicable;
- the resolver must rebuild the descriptor from canonical joins (`workflow_actor_bindings` -> `workflow_stage_runs` -> `workflow_runs` -> workstream/workspace plus `winds_sessions`) plus the persisted immutable target-request `actor_role`, and compare the approval envelope's exact actor binding/session/work/role scope and descriptor digest with the persisted request/event. Any role, binding, session, stage, workflow, workstream, workspace, or digest mismatch returns `STALE` or `AUTHORITY_DENIED` and never mutates historical records;
- generic `workflow_decisions` may remain explanatory/history evidence, but they are **not** a first-slice Model Mesh selection/permission authority path because current canonical workflow-decision storage has no accepted structured human target-decision schema/load path. Adding such a path later requires an explicit canonical Plan/Tasks amendment.


Rules:

- `RuntimeKind` is reused; no `Provider` trait hierarchy or generic adapter registry is created;
- provider/model IDs are bounded normalized exact identifiers, not display labels and not arbitrary executable names;
- canonical target descriptor serialization and digesting are deterministic; two semantically identical normalized targets on the same exact actor binding/Winds session and canonical work scope produce the same descriptor digest, while runtime/provider/model, workspace/workstream/workflow/stage, actor binding, Winds session, or actor-role movement produces a different digest;
- unspecified target dimensions stay explicitly `UNSPECIFIED` and stay unspecified;
- a runtime kind never implies provider/model identity;
- model/provider text emitted by an agent remains `AGENT_REPORTED` and cannot satisfy a request requiring Winds-observed identity;
- equality/matching uses stable normalized exact values and source/proof applicability, never fuzzy matching, ranking, reputation, cost, latency, or prior success;
- ambiguous/conflicting/unknown dimensions remain explicit and cannot select a winner;
- target resolution is pure/deterministic for identical canonical inputs;
- target resolution has no side effects and grants no authority.

### 2. Observation adapter over accepted runtime truth

Reuse `RuntimeDiscovery` and `RuntimeSessionBinding` as the runtime identity basis.

Plan decision:

- runtime executable path/hash/version observations remain existing Winds-local runtime truth;
- provider/model observations are optional source-labelled claims and may be admitted only from an already-authorized structured local observation path;
- do not infer `OpenAI` merely because `RuntimeKind::Codex` is selected;
- do not infer `Anthropic` merely because `RuntimeKind::Claude` is selected;
- the existing Codex test-only structured `modelProvider` field or any future vendor field does not become production identity until a Task proves an accepted structured observation path and exact source class;
- Claude provider/model identity remains unknown if no accepted structured observation exists;
- missing provider/model observation is a valid `UNKNOWN` result, not a reason to add a provider API;
- catalog/vendor declarations remain declarations and may conflict with local observations without being silently discarded;
- `AuthReadiness::Unknown` remains unknown unless a separately authorized accepted local observation exists;
- no authenticated network probe is required or authorized just to turn unknown identity/readiness into known truth.

The first implementation must therefore be useful when only runtime identity is locally proven: a request scoped only to `RuntimeKind::Codex` or `RuntimeKind::Claude` can be resolved if otherwise authorized, while a request that additionally requires an unproven provider/model identity fails truthfully rather than guessing or falling back.

### 3. Append-only target-request and identity-claim persistence

Reuse `winds.db` and `rusqlite`; expected migration order is `0011_model_mesh_continuity.sql` if Tasks confirm no intervening canonical migration.

Minimal logical relational shape:

```text
model_mesh_target_requests
  target_request_id PK
  stage_run_id FK -> workflow_stage_runs
  actor_binding_id FK -> workflow_actor_bindings
  actor_role
  runtime_kind
  requested_provider_id?
  requested_model_id?
  selector_class              # HUMAN in first slice
  target_descriptor_digest
  selection_approval_id FK -> agentic_delegation_approvals
  created_unix_ms

model_mesh_identity_claims
  identity_claim_id PK
  target_request_id? FK -> model_mesh_target_requests
  actor_binding_id? FK -> workflow_actor_bindings
  claim_subject              # REQUEST_TARGET | ACTOR
  dimension
  normalized_value?
  source_class
  observation_basis?
  runtime_binding_id? FK -> runtime_session_bindings
  observed_unix_ms

model_mesh_continuity_events
  continuity_event_id PK
  target_request_id FK -> model_mesh_target_requests
  source_actor_binding_id? FK -> workflow_actor_bindings
  destination_actor_binding_id? FK -> workflow_actor_bindings
  continuity_class
  context_digest?
  completeness_state
  continuity_permission_digest?
  authority_approval_id? FK -> agentic_delegation_approvals
  authority_claim             # REQUIRED | NO_AUTHORITY_CLAIM
  created_unix_ms

model_mesh_continuity_identity_claims
  continuity_event_id FK -> model_mesh_continuity_events
  actor_role                # SOURCE | DESTINATION
  identity_claim_id FK -> model_mesh_identity_claims
  PRIMARY KEY (continuity_event_id, actor_role, identity_claim_id)
```

Required constraints:

- all four relations are historical append-only records; updates/deletes are rejected by schema-level protection;
- every target request is bound to exactly one canonical `StageRun` attempt and exactly one existing `workflow_actor_bindings.binding_id` for that stage, and persists one bounded immutable normalized `actor_role` token as part of that request's canonical scope;
- the target request's actor binding must have a concrete `winds_session_id`; bindings with unknown/unavailable session identity cannot support an authorized human target selection and fail closed as `UNAVAILABLE`/`AUTHORITY_DENIED` as applicable;
- `actor_role` is durable request scope, not inferred actor identity or authority: insertion validates it as bounded canonical text, stores it immutably, includes it in `ModelMeshTargetDescriptorV1`, and requires exact equality with `ModelMeshAuthorityEnvelopeV1.actor_role`; current execution/delegation authority remains separately evaluated and cannot be expanded by the role token;
- insertion validates through joins that the actor binding belongs to the request stage, the stage belongs to its canonical workflow/workstream/workspace hierarchy, and the bound Winds session belongs to that same canonical workstream/workspace context;
- every target request persists the exact canonical `target_descriptor_digest`, rebuilt from that joined canonical scope, and is not applicable unless `selection_approval_id` resolves to a digest-valid `ModelMeshAuthorityEnvelopeV1` with purpose `TARGET_SELECTION`, the identical actor binding and Winds session, identical workspace/workstream/workflow/stage/actor-role scope, and the identical target descriptor digest;
- the first slice rejects `EXPLICIT_POLICY`; no policy label/reference/digest is accepted as selection authority until a separately canonical amendment defines a real policy path;
- identity claims use an exclusive subject binding: `REQUEST_TARGET` requires `target_request_id` and forbids `actor_binding_id`, while `ACTOR` requires exactly one `actor_binding_id` and does not borrow target identity from display-name coincidence;
- any runtime binding referenced by an actor identity claim must belong to the same canonical Winds session/actor context; cross-workflow or cross-actor coincidence is rejected;
- any source/destination actor binding referenced by a continuity event must belong to the relevant stage lineage or an explicitly qualified source-stage handoff relation selected later by Tasks;
- every continuity-event actor identity is role-specific: `model_mesh_continuity_identity_claims.actor_role=SOURCE` may reference only claims bound to that event's `source_actor_binding_id`, and `DESTINATION` only claims bound to its `destination_actor_binding_id`; SQL constraints/triggers must reject mismatched actor, runtime-binding, workflow/stage lineage, or event associations;
- provider/model unknown state for either actor is preserved by an explicit source-labelled unknown/unavailable claim rather than inferred from the target request or the opposite actor;
- any continuity event that claims an authorized mutating continuation uses `authority_claim=REQUIRED`, persists the canonical `continuity_permission_digest`, and must reference one digest-valid `authority_approval_id` whose `ModelMeshAuthorityEnvelopeV1` has purpose `CONTINUITY_PERMISSION`, the identical target descriptor digest, identical canonical operation descriptor digest, and matching canonical work/stage/actor scope;
- observation-only `UNAVAILABLE`/`UNPROVEN` records may omit authority approval only with `authority_claim=NO_AUTHORITY_CLAIM`; such rows cannot be projected as permitted execution;
- generic workflow-decision text, safe rationale, decision type/result labels, runtime labels, or an approval that lacks the exact Model Mesh descriptor digest can never satisfy Model Mesh selection/permission authority;
- target request replay uses an operation/idempotency identity selected in Tasks; replay may not create duplicate current truth;
- provider/model identifiers and all canonical target/continuity descriptor digests are bounded and validated before persistence;
- raw prompts, terminal transcripts, environment dumps, credentials, provider-private memory, arbitrary provider response payloads, and model output are not persisted in these tables;
- runtime executable/version facts remain referenced from accepted runtime bindings/discovery rather than copied into a competing identity table;
- no “current target” mutable singleton row is selected; current/applicable target truth is a deterministic projection over append-only stage-bound records plus revalidated content-bound approval authority basis.

The initial migration does **not** need a usage/cost table. P2 usage/cost observation may be added only in a later authorized task if an already-existing structured observation proves a concrete need; otherwise unknown satisfies the contract.

### 4. Exact target resolver and authority gate

Create one pure resolver that evaluates an explicit `TargetRequest` against qualified identity claims, fresh runtime identity, exact actor-binding/Winds-session scope, capability state, authentication-readiness truth, the digest-valid Model Mesh human approval basis, and the independently current execution/delegation authority ceiling.

Evaluation order should remain explicit:

```text
1. validate canonical StageRun/work context
2. validate target request structure
3. revalidate runtime identity/binding freshness
4. collect source-labelled provider/model/native claims
5. detect unknown/unavailable/ambiguous/conflicting/stale identity
6. check required capability state
7. retain authentication readiness as its own truth dimension
8. recompute the canonical target/continuity descriptor from exact actor-binding/session/work joins and revalidate the exact content-bound Model Mesh human approval envelope
9. evaluate the current existing execution/delegation authority ceiling independently; the approval cannot exceed it
10. return resolution; do not execute
```

Rules:

- exact matching never invokes a provider/model;
- `EXACT_MATCH` means only identity/capability/request applicability; it does not mean authenticated, authorized to execute, verified, accepted, or landed;
- unavailable requested target never silently selects another target;
- ambiguous target never triggers a score/rank/winner function;
- `EXPLICIT_POLICY` is not authorized in the first implementation and resolves fail-closed as `POLICY_NOT_AUTHORIZED`; enabling policy selection requires a separately accepted Plan/Tasks amendment satisfying FR-024..FR-029;
- human selection becomes stale/denied if the referenced Model Mesh approval content digest, target descriptor digest, exact actor binding/Winds session, canonical work scope, or current authority ceiling no longer revalidates;
- alternate-target selection is a new explicit request/event, preserving the failed/unavailable original request;
- usage/cost observations cannot affect first-slice resolution.

### 5. Continuity classifier and drift evaluator

Extend workflow-facing continuity semantics without rewriting Spec 006 native ownership truth.

First-slice Model Mesh continuity vocabulary:

```text
NATIVE_RESUME
RECONSTRUCTED
REASSIGNED
HANDOFF
OWNERSHIP_LOST
UNAVAILABLE
UNPROVEN
```

Mapping rules:

- `NATIVE_RESUME` is emitted only if the pre-existing accepted native-resume proof actually supports `RESUMED`; a durable runtime/session candidate alone remains insufficient;
- a new native session initialized from canonical state is `RECONSTRUCTED`;
- a new actor/target replacing the prior actor is `REASSIGNED` where that distinction matters;
- cross-runtime transfer is `HANDOFF`, optionally accompanied by reconstruction/loss metadata, never native resume;
- ownership loss remains explicit and cannot be repaired by display-name/native-ID coincidence;
- unknown or unsupported continuity proof is `UNPROVEN`/`UNAVAILABLE` rather than guessed.

Drift evaluation must deterministically cover:

- runtime executable identity or version movement;
- provider/model identity conflict or movement;
- native-session identity/ownership movement;
- `WorkflowRun` / `StageRun` attempt movement;
- candidate/artifact/evidence movement through existing Spec 008 freshness evaluation;
- content-bound Model Mesh approval applicability/content digest, exact target/continuity descriptor digest, actor-binding/Winds-session scope, or current authority-ceiling movement.

Drift invalidates only the claims or permissions whose observation/authority basis depends on the moved identity. Historical target requests, actor-specific identity claims, continuity-event role associations, approvals, generic decisions, and failed/stale permission results remain append-only and inspectable.

### 6. Direction-neutral continuity context over existing context/reconstruction seams

Do not delete or silently generalize the accepted T081 Claude-Planner -> Codex-Worker contract in place.

Instead, Tasks should introduce one lower-level provider-agnostic continuity-context projection assembled from existing canonical inputs:

```text
ModelMeshContinuityContext {
  workflow_run_id,
  stage_run_id,
  canonical work identity,
  exact target request/reference,
  exact candidate/artifact/evidence references,
  content-bound Model Mesh approval authority basis plus separately re-evaluated current authority ceiling,
  bounded reconstruction/transfer report,
  unavailable/redacted/material-loss markers,
}
```

Rules:

- the projection is built from canonical Winds structured state and exact references;
- provider-private memory remains `UNAVAILABLE`/`NO_LONGER_TRANSFERABLE` unless separately and safely observed;
- source-agent persuasion/confidence/winner recommendation is excluded from independent-review handoff by default;
- context digest binds the continuity event to the exact structured context used;
- redaction/omission is explicit and may block/downgrade continuity if required material is missing;
- no raw transcript replay is selected as canonical continuity;
- direction-neutral evaluation may cover Codex -> Claude, Claude -> Codex, Codex -> Codex, and Claude -> Claude fixtures, but real execution remains separately authorized and Spec 006 live-runtime nonclaims stay unchanged;
- existing T081 behavior remains a compatibility/regression surface until a later implementation task can prove safe reuse of the lower-level seam without expanding authority.

### 7. Read-only operator projections

Generate deterministic projections instead of persisting competing UI state.

Initial surfaces:

```text
ModelMeshTargetProjection
ModelMeshContinuityProjection
ModelMeshWhyBlockedProjection
ReviewerContinuityProjection
```

At minimum expose:

- exact workflow/stage-attempt identity plus the target request's exact actor binding and Winds session;
- requested runtime/provider/model and selector source;
- source-labelled observed/declared/agent-reported identity claims, with provider/model claims separated by exact source versus destination actor role for continuity events;
- target resolution and explicit blocker category;
- runtime binding/native-session identity and freshness where applicable;
- continuity class plus source/destination actor/runtime identity;
- context digest and bounded transfer/reconstruction completeness/loss summary;
- candidate/artifact/evidence freshness where applicable;
- authentication readiness as separate truth;
- exact content-bound Model Mesh approval basis, target/continuity descriptor digest match, approval revalidation state, and the current existing authority ceiling/result as separate truth; generic workflow decisions remain history/explanation only;
- verification and human acceptance as separate existing authority states when in view;
- optional usage/cost as unknown unless a qualified observation exists.

Projection functions are side-effect free. Merely opening status/why-blocked cannot execute, resume, hand off, request credentials, change target, mutate authority, create a Git change, or land a candidate.

## Identity Source and Conflict Policy

The first implementation uses source classes, not a global source-priority winner.

```text
WINDS_LOCALLY_OBSERVED
VENDOR_DECLARED
CATALOG_DECLARED
AGENT_REPORTED
HUMAN_DECIDED
UNAVAILABLE
```

Interpretation:

- source class describes provenance, not blanket precedence;
- a requested identity dimension is satisfied only by a source class explicitly accepted for that dimension and operation by the deterministic resolver;
- `AGENT_REPORTED` never satisfies a Winds-observed provider/model requirement;
- `HUMAN_DECIDED` may select a requested target but does not transform an unobserved provider/model identity into Winds-observed fact;
- declaration/local conflicts remain visible `CONFLICT` truth until re-observed or explicitly resolved by an accepted deterministic rule;
- duplicate equivalent claims remain source-labelled history and do not create extra authority;
- empty/unknown values are represented explicitly and never normalized into a guessed value.

Tasks must define exact bounded identifier normalization and permitted source/dimension combinations before source implementation lands.

## Authentication and Secret Policy

Plan decision:

```text
RAW_PROVIDER_CREDENTIAL_PERSISTENCE=NO
CREDENTIAL_SCRAPING=NO
AUTOMATED_LOGIN=NO
REFRESH_TOKEN_BROKER=NO
AUTHENTICATED_PROBE_FOR_DISCOVERY=NO_BY_DEFAULT
EXECUTABLE_PRESENT_IMPLIES_AUTH_READY=NO
TARGET_SELECTION_GRANTS_SECRET_ACCESS=NO
```

Rules:

- existing runtime executable discovery may coexist with `AuthReadiness::Unknown`;
- target resolution never requests or prints a credential just because identity matches;
- no raw environment credential, token, password, cookie, key, or provider-private login state enters Model Mesh durable records;
- secret-like content from agent/model/provider output remains untrusted and subject to existing redaction/history protections;
- if a later Task proves a non-secret credential reference is required, that requires explicit scope/lifecycle semantics and security review before implementation;
- an authenticated API/provider SDK/credential store integration requires a separately accepted Plan amendment or follow-on governance unit; it is not authorized by this Plan.

## Usage / Cost Observation Policy

Spec 009 P2 usage/cost visibility remains deliberately non-blocking.

Plan decision:

- do not add a pricing database, billing API, telemetry SaaS, gateway, or provider SDK;
- do not compute cost from guessed/current public prices and call it observed cost;
- do not infer token usage from prose;
- an already-authorized structured local runtime event may later be adapted into a bounded source-labelled observation after an explicit task qualifies its schema/provenance;
- the existing Codex test protocol includes structured token-usage fields, but those tests do not by themselves authorize a new production usage/cost persistence path;
- when no qualified observation exists, projection returns `UNKNOWN`;
- usage/cost never influences routing, winner selection, verification, human acceptance, or Git authority.

## Durable State Versioning and Recovery

Model Mesh persisted rows must use bounded known enums and existing schema-integrity patterns.

Before target/continuity records are treated as applicable, Winds must validate:

- referenced `StageRun` and parent workflow exist and match canonical work identity;
- referenced runtime/actor bindings exist and belong to the applicable canonical context;
- each actor-scoped identity claim is bound to exactly one actor binding, and each continuity-event SOURCE/DESTINATION association references a claim bound to the matching event actor;
- each applicable target request has a persisted immutable `actor_role` and an actor binding that joins to the same exact Winds session and canonical workspace/workstream/workflow/stage scope encoded by a digest-valid `TARGET_SELECTION` Model Mesh approval; the approval's role must exactly match the stored request role and its canonical target descriptor digest must exactly match the recomputed request descriptor;
- each authorized mutating continuity event has a digest-valid `CONTINUITY_PERMISSION` Model Mesh approval whose canonical target and operation descriptor digests exactly match the event; `NO_AUTHORITY_CLAIM` events cannot project execution permission;
- no first-slice record is applicable through generic workflow-decision text or policy selection;
- runtime/provider/model identifiers and persisted target-request actor roles are structurally valid and bounded;
- selector/source/dimension/continuity/completeness enums are known;
- provider/model/native claims carry the required source and observation basis for their proof level;
- context digest format is valid where present;
- source/destination handoff relationships are internally consistent;
- no duplicate current/applicable request/event is manufactured by replay;
- candidate/artifact/evidence references are freshly evaluated rather than trusted from persisted presentation state.

Recovery posture:

- unknown/unsupported enum or schema shape -> explicit incompatible/recovery-required truth;
- orphan/mismatched stage/runtime/actor reference -> fail closed while preserving historical data;
- corrupt/partial schema -> fail schema integrity checks rather than silently initializing clean state;
- physical SQLite corruption/open failure -> leave `winds.db` untouched and fail;
- migration is idempotent and preserves every historical request, mismatch, unavailable state, and failed continuity event;
- no automatic destructive repair or historical rewrite is authorized.

## Concurrency and Transaction Boundaries

The architecture remains one Winds process, but SQLite/WAL and command concurrency still require deterministic transactions.

Required transaction rules:

- target-request insertion plus canonical stage validation occur atomically;
- identity-claim insertion validates its exclusive request-or-actor subject plus any runtime binding in the same transaction;
- target-request insertion validates and persists the immutable bounded actor role, joins and validates the exact actor binding/Winds session/work scope, recomputes the canonical target descriptor digest from that joined truth plus stored role, and validates an exact digest-valid `TARGET_SELECTION` Model Mesh approval in the same transaction;
- continuity-event insertion recomputes the canonical operation descriptor digest and validates request, source/destination actor bindings, exact target digest, and digest-valid `CONTINUITY_PERMISSION` Model Mesh approval atomically;
- continuity-event role/identity-claim association validates that each SOURCE/DESTINATION claim belongs to the exact matching event actor before commit;
- replay/idempotency checks occur within the write transaction;
- no transaction may hold a database write lock while waiting on a runtime child, provider, network call, reviewer, human, or terminal interaction;
- identity observation happens before/after the transaction as appropriate, and only the bounded qualified result is persisted;
- crash between external effect and durable observation cannot be guessed successful; existing recovery/reconstruction semantics apply.

## CLI / Workbench Integration Plan

Do not add a new application shell or provider dashboard.

First expose the Model Mesh layer through narrow existing-style surfaces after domain/storage qualification. Exact spelling belongs to Tasks after CLI collision review.

Required first user capabilities:

- inspect admitted runtime target availability without executing;
- submit/record an explicit stage-bound target request;
- inspect target resolution and why blocked;
- inspect source-labelled requested/declared/observed/agent-reported identity truth;
- inspect continuity class, drift/staleness, material context loss, and source/destination binding;
- record a later explicit alternate target request without rewriting the original failed/unavailable request;
- generate reviewer continuity context using existing exact candidate/evidence and reviewer-independence rules.

Workbench/TUI may later render the same read-only projections, but must not become a second identity/selection/continuity authority and must not land before the deterministic domain/projection seam is qualified.

No first-slice CLI command may silently execute a target merely because resolution is `EXACT_MATCH`.

## Testing Strategy

### Pure target-domain tests first

Prove without SQLite/runtime execution:

- exact identifier normalization and bounds;
- unspecified dimensions remain unspecified;
- runtime != provider != model identity;
- source-labelled identity claims remain distinct under equal and conflicting values;
- exact target matching;
- unknown/unavailable/ambiguous/conflict/stale/authentication-unknown/capability-unavailable/authority-denied results;
- no ranking/winner/silent fallback;
- human vs explicit-policy selector source remains visible;
- target resolution has no side effects and cannot expand authority.

### Persistence and schema fixtures

Prove:

- migration idempotence and expected schema-object integrity;
- target request -> exact StageRun/workflow/workspace/workstream binding;
- identity claim -> request/runtime binding integrity;
- continuity event -> request/source/destination actor integrity;
- append-only/no-update/no-delete behavior;
- replay/idempotency does not manufacture duplicate current truth;
- restart preserves exact history;
- corrupt/partial/unknown state fails closed without recreating a false clean state;
- existing database contents remain preserved on Model Mesh schema failure.

### Observation and conflict fixtures

Prove:

- runtime observation is derived only from accepted RuntimeDiscovery/binding truth;
- runtime label cannot manufacture provider/model identity;
- agent self-report remains agent-reported;
- catalog/vendor/local claims retain provenance;
- provider/model missing -> unknown;
- declaration/local mismatch -> conflict;
- changed runtime executable/version makes dependent claims stale;
- provider/model movement makes only the applicable identity-bound continuity stale.

### Target and authority fixtures

Prove:

- canonical `ModelMeshTargetDescriptorV1` serialization/digest is stable for identical normalized input on the same exact actor binding/Winds session and changes for runtime/provider/model/workspace/workstream/workflow/stage/actor-binding/session/actor-role movement;
- only a digest-valid `ModelMeshAuthorityEnvelopeV1` `TARGET_SELECTION` approval with exact descriptor digest/scope can authorize an applicable human target request;
- only a digest-valid `CONTINUITY_PERMISSION` approval with exact target + continuity permission descriptor digests can authorize a mutating continuity event;
- a generic `workflow_decision`, generic T076 approval content, mismatched approval, stale approval digest, same-stage approval for a different provider/model target, or approval for another actor binding/Winds session in the same stage/role cannot authorize selection/continuity;
- two immutable actor bindings with the same stage and logical role but different `winds_session_id` values produce different target descriptor digests; an approval for binding/session A is rejected for binding/session B;
- with binding, Winds session, stage, runtime/provider/model target, and all other scope held identical, changing only the approval envelope `actor_role` must fail exact revalidation as `STALE` or `AUTHORITY_DENIED`;
- `EXPLICIT_POLICY` fails closed as not authorized in the first slice;
- exact request match never grants execution/delegation/filesystem/network/secret/Git/verification/human authority;
- unavailable target never silently falls back;
- ambiguous target requires explicit new decision/request;
- authentication unknown remains separate from availability;
- changed Model Mesh approval content/descriptor digest or current authority ceiling invalidates prior permission without rewriting identity history;
- alternate target success preserves original target failure/unavailability evidence.

### Continuity and handoff fixtures

Prove:

- exact native proof -> `NATIVE_RESUME` only where existing accepted proof supports it;
- durable native identifier alone never yields `NATIVE_RESUME`;
- new session from canonical state -> `RECONSTRUCTED`;
- actor replacement -> `REASSIGNED` where applicable;
- Codex -> Claude and Claude -> Codex deterministic fixtures -> `HANDOFF`, never native resume;
- same-runtime new-session fixtures remain reconstruction/reassignment rather than false resume;
- provider-private memory unavailable remains explicit;
- missing/redacted required context blocks/downgrades continuity truth;
- context digest movement/stage attempt movement/candidate movement invalidates exact prior applicability;
- existing T081 Claude-Planner -> Codex-Worker tests remain green and authority-bounded.

### Reviewer/evidence fixtures

Prove:

- reviewer context includes exact candidate/artifact/evidence/authority identity;
- builder/source-agent persuasion/confidence/winner recommendation is excluded by default;
- stale candidate invalidates reviewer-context applicability without deleting history;
- forged agent `PASS`, `VERIFIED`, `ACCEPTED`, provider/model identity, capability, continuity, or authority text cannot promote canonical truth;
- target/continuity success never implies stage/workflow verification or human acceptance.

### Secret/adversarial fixtures

Prove:

- secret-like provider payloads are not persisted into Model Mesh rows/projections;
- model output cannot request/authorize credential exposure;
- malformed/unknown source/continuity enum fails closed;
- cross-workflow or cross-stage ID replay is rejected;
- native-session ID reuse after ownership loss does not revive prior live state;
- ambiguous duplicate claims cannot silently select a canonical provider/model;
- provider/model drift cannot attach old evidence to a new stage/candidate.

### Regression and platform gates

Every implementation candidate must run repository `quality` plus applicable existing Spec 003/006/007/008 regression/security/platform gates for the exact changed behavior.

Platform claims remain domain-specific:

- native Windows/ConPTY only when directly exercised;
- real WSL2 only when directly exercised;
- Linux PTY only when directly exercised;
- macOS PTY only when directly exercised;
- deterministic target-domain/storage fixtures do not become physical provider/runtime acceptance on every platform;
- Spec 006 live-runtime nonclaims remain `NO` unless separately proven.

## Requirement-to-Implementation Map

| Spec 009 contract | Planned seam | Primary evidence |
| --- | --- | --- |
| FR-001..FR-012 | closed target domain + exact request resolver | identity/target pure fixtures |
| FR-013..FR-022 | observation adapter/source-labelled claims | declaration/local/agent conflict fixtures |
| FR-023..FR-032 | no-fallback resolver + existing authority gate | routing-negative/authority fixtures |
| FR-033..FR-045 | continuity classifier + canonical continuity context | resume/reconstruction/reassignment/handoff/adversarial fixtures |
| FR-046..FR-058 | stage-bound append-only records + drift evaluator | workflow/stage/candidate/authority movement fixtures |
| FR-059..FR-067 | existing secret/redaction posture + auth truth | credential/adversarial fixtures |
| FR-068..FR-074 | read-only projections + optional observation hook | projection/unknown-usage fixtures |
| FR-075..FR-082 | reviewer/evidence integration | forged-truth/reviewer-independence fixtures |
| FR-083..FR-095 | scope/dependency/platform/governance gates | negative-scope + exact-candidate/platform reconciliation |
| SC-001..SC-018 | combined deterministic qualification campaign | focused domain/storage/continuity/adversarial tests |
| SC-019..SC-025 | regression/review/scope/platform/history reconciliation | exact final candidate + final reconciliation artifact |

## Implementation Slice Strategy

Tasks should authorize small dependency-ordered slices. This Plan does not itself authorize any implementation slice.

Recommended sequence:

1. target/identity/source/continuity domain types plus pure exact resolver and full negative-state matrix;
2. reuse adapters from `RuntimeDiscovery`, `RuntimeSessionBinding`, workflow/stage/actor identity, and existing authority truth without persistence changes;
3. qualify one idempotent `0011_model_mesh_continuity.sql` migration for append-only stage-bound target/claim/continuity records and Store methods;
4. deterministic drift/staleness evaluator for runtime/version/provider/model/native-session/stage/candidate/authority movement;
5. direction-neutral continuity-context projection over existing Spec 008 reconstruction/context seams while preserving T081 compatibility;
6. continuity-event persistence and exact context-digest/source/destination binding;
7. target/status/why-blocked/reviewer projections;
8. narrow CLI inspection/request surfaces with zero implicit execution;
9. workbench read-only rendering only if still necessary after CLI qualification;
10. secret/adversarial/replay/corruption/history campaign;
11. applicable platform and Spec 003/006/007/008 regression qualification;
12. optional P2 usage observation only if an already-authorized structured local source proves necessity; otherwise close the implementation program with usage/cost `UNKNOWN` support;
13. final Spec 009 reconciliation and exact-candidate acceptance.

Tasks may split a recommended slice further for reviewability. They may not combine later authority, opportunistically admit a new runtime/provider/dependency, or skip a predecessor whose invariant the later slice relies on.

## Dependency Decision Record

No dependency changes are required or selected.

Existing direct dependencies already cover the planned first implementation needs:

- `rusqlite = "=0.40.2"` with bundled SQLite: existing durable local relational state;
- `serde` / `serde_json`: existing bounded structured serialization where required;
- `sha2`: existing digest primitive for content-bound context/identity support where a Task proves hashing appropriate;
- existing runtime/workflow/context/authority/Git modules: current canonical observations and proof boundaries.

Plan decision:

```text
NEW_DIRECT_DEPENDENCIES=0
NEW_PROVIDER_SDK=NO
NEW_PROVIDER_API=NO
NEW_GATEWAY=NO
NEW_ASYNC_RUNTIME=NO
NEW_DATABASE=NO
NEW_ORM=NO
NEW_PLUGIN_ABI=NO
NEW_DAEMON_OR_IPC=NO
NEW_CREDENTIAL_STORE=NO
```

If a later Task discovers a genuinely missing capability, it must stop and obtain an explicit accepted amendment/dependency qualification rather than silently adding a crate, provider service, gateway, SDK, or protocol.

## Failure and Recovery Semantics

Fail closed whenever canonical truth is insufficient.

Examples:

- unknown/malformed target -> invalid request;
- provider/model unobserved -> unknown;
- requested target unavailable -> unavailable with no fallback;
- multiple applicable targets/claims -> ambiguous/conflict;
- runtime executable/version moved -> stale/requalification required;
- authentication unproven -> authentication unknown, not authenticated;
- requested capability unproven -> unavailable/unsupported as applicable;
- missing/mismatched/non-Model-Mesh approval, changed descriptor digest, or actor-binding/Winds-session mismatch -> target identity may still match, but selection/continuity permission is stale or denied;
- generic workflow decision or generic T076 approval -> explanatory/history evidence only, never exact Model Mesh target authorization;
- authority denied -> identity may still be an exact match, but execution remains denied;
- native resume unproven -> reconstructed/reassigned/unproven/unavailable as applicable, never guessed resumed;
- provider/model/native-session/stage/candidate drift -> exact dependent continuity/evidence stale;
- malformed/orphan durable Model Mesh state -> recovery/incompatible while preserving history;
- missing required context -> incomplete/blocked rather than clean handoff;
- forged agent/provider labels -> source-labelled non-authoritative input only;
- target/continuity success with verification pending -> verification remains pending;
- human acceptance missing -> never implied;
- failed/unavailable prior target followed by successful alternate -> both historical records remain visible.

## Security / Privacy Posture

- Model Mesh state is local Winds-owned structured state, not a trust grant to repository/provider/model output;
- runtime/provider/model output is untrusted and may contain secrets or prompt injection;
- durable Model Mesh records store the minimum bounded identity/source/reference data required for truth;
- no raw credential/token/password/cookie/key storage;
- no provider credential brokerage;
- no background listener/socket/server;
- no remote continuation/control plane;
- no browser automation/profile/CDP integration;
- no arbitrary network/provider call is granted by target resolution;
- no arbitrary shell execution is granted by a target request or continuity event;
- provider/model identity cannot become verification or human authority;
- repository paths/display labels/native IDs are context, not authority;
- corrupted state is preserved rather than destructively rewritten into clean success.

## Evidence Integrity

Every acceptance artifact must bind to exact candidate state. At minimum record commit/tree plus command/runner/platform where applicable.

No evidence may be current if:

- HEAD changed after it was produced;
- the tested tree differs;
- the runtime executable/version/provider/model/native-session observation basis moved;
- workflow/stage attempt moved;
- required candidate/artifact baseline moved;
- authority/policy basis moved;
- platform/domain differs from the claim;
- result is model/terminal/agent prose rather than an accepted Winds/CI observation;
- review applies to an older candidate;
- historical failed/unavailable/stale/reconstructed state is merely relabelled by later success.

Final reconciliation must keep historical target failures, identity conflicts, ownership loss, stale observations, reconstruction loss, failed continuity attempts, and superseded target decisions inspectable and must not collapse `TARGET_MATCH`, `EXECUTION_AUTHORIZED`, `CONTINUITY_PROVEN`, `VERIFIED`, `HUMAN_ACCEPTED`, and `LANDED` into one status.

## Ponytail / YAGNI Boundaries

Explicitly reject during the first Spec 009 implementation program unless an accepted amendment changes scope:

- generic provider/plugin trait hierarchy merely for hypothetical providers;
- provider marketplace/integration SDK/dynamic code loading;
- first-slice policy engine or generic workflow-decision-as-target-authority path;
- broad provider fleet admission;
- mandatory LiteLLM or equivalent provider gateway/proxy;
- provider billing/pricing database or telemetry SaaS;
- credential broker/login automation/secret vault integration;
- daemon/process supervisor architecture;
- IPC/socket/RPC/local server;
- remote/mobile/team continuation;
- browser/CDP runtime;
- MCP/ACP/A2A expansion;
- learning/skill optimization/training/fine-tuning/RL/learned routing;
- semantic/vector/RAG memory;
- second database/state store;
- new ORM/persistence framework;
- message queue/distributed lease service;
- persisted provider-private transcript/prompt memory;
- fuzzy model/provider aliases that become identity authority;
- automatic target ranking/winner selection;
- automatic candidate winner selection;
- automatic Git mutation/PR creation/landing.

Prefer closed enums, concrete structs, exact normalized IDs, explicit Store methods/SQL constraints, pure deterministic resolution/drift functions, and direct projections until multiple proven use cases justify a broader abstraction.

## External Research Provenance

The accepted Spec 009 entry research and prior Model Mesh/product comparisons inform product gaps only. No donor source, runtime, dependency, gateway, provider SDK, adapter implementation, credential mechanism, or external code is admitted by this Plan.

Implementation remains independently designed against canonical Winds Constitution/Spec/Plan and compatible existing dependencies. Any future donor-code/dependency/runtime/provider admission requires a separately accepted provenance/license/security/platform/YAGNI decision.

## Plan Acceptance Gate

This Plan can land only if all are true on its exact final candidate:

- canonical base is the post-Spec-009-spec merge `a8525b68d7d5ca83cfd3c475cbae18e131700e07` or an explicitly reconciled newer canonical `main` descendant;
- changed scope is planning/documentation only;
- no `Cargo.toml`, lockfile, migration, source, workflow semantic, runtime/provider execution, credential, gateway, browser, daemon, IPC, remote, learning, plugin, or automatic landing change occurs;
- the Plan proves direct reuse of accepted runtime/workflow/context/SQLite seams and selects no new dependency;
- runtime/provider/model/canonical/native identities remain distinct and no display label becomes authority;
- exact target selection/continuity permission is content-bound through versioned Model Mesh approval canonicalization and descriptor digests that include exact actor-binding/Winds-session/work scope; generic decisions/approvals cannot be reused across targets, actors, or sessions;
- no automatic provider/model routing, ranking, winner, or silent fallback is introduced;
- the Plan provides a plausible implementation/evidence path for FR-001..FR-095 and SC-001..SC-025;
- Spec 006 live-runtime nonclaims remain unchanged;
- exact-head repository `quality` succeeds;
- correctness/safety/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads remain;
- exact main/base/head/tree/scope/ruleset/mergeability are reconciled immediately before guarded merge;
- guarded expected-head normal merge succeeds;
- post-merge canonical main/tree/ordered parents/signature and every actually-triggered applicable push check are verified.

## Downstream Authorization

Canonical acceptance and post-merge verification of this Plan authorize **Spec 009 Tasks creation only**.

They do not authorize implementation, migration landing, source changes, runtime/provider execution, provider API/SDK/gateway admission, credential mechanisms, new dependencies, daemon/IPC, browser/remote execution, learning, automatic routing/fallback/winner selection, automatic Git actions, or later specifications.

Only after this Plan lands and is post-merge verified may repository truth state:

```text
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN=CLOSED_CANONICAL
SPEC_009_TASKS_AUTHORIZED=YES
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
```
