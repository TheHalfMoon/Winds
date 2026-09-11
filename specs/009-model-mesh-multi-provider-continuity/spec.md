# Feature Specification: Model Mesh & Explicit Multi-Provider Continuity

**Feature Branch**: `spec/009-model-mesh-multi-provider-continuity`

**Created**: 2026-09-11

**Canonical Base**: `77933ae166b885c09f9d1c1e9971610fe179b230`

**Canonical Entry Authority**: `docs/research/018-spec-009-entry-gate.md`, PR #166, merge `77933ae166b885c09f9d1c1e9971610fe179b230`, post-merge `quality #1201` successful on attempt 1

**Status**: Authorized for specification only; planning, tasks, dependencies, provider SDKs/APIs, gateways, credentials, runtime implementation, daemon/IPC work, browser execution, remote execution, learning, automatic routing, and product-source changes are NOT authorized by this file alone

**Input**: Give Winds a first-class, explicit, proof-carrying continuity contract across the currently admitted Codex and Claude runtime families so a developer can select a target deliberately, distinguish runtime/provider/model/native-session identity, preserve exact workflow and evidence provenance across handoff or reconstruction, detect identity drift, explain unavailable or unknown state truthfully, and never let provider/model selection silently expand authority or verification claims.

## Product North Star

Winds should make changing or continuing work across agent runtimes/providers/models as explicit and trustworthy as changing an exact Git candidate: the user can see what identity was requested, what Winds actually observed, what continuity was proven, what was reconstructed or lost, and which authority/evidence remains applicable.

For every admitted continuity operation the user should be able to answer:

1. **What exact target did I request?** Runtime, provider, model, and native-session identity are represented separately where observable rather than collapsed into one label.
2. **What did Winds actually prove?** Catalog declarations, vendor/runtime declarations, local observations, agent prose, and unknown state remain distinct evidence classes.
3. **Did Winds continue, reconstruct, or hand off?** Native resume, same-runtime reconstruction, cross-runtime handoff, reassignment, and unavailable continuation are different proof levels.
4. **Did anything drift?** Runtime executable/version, provider/model identity, native-session identity, workflow attempt, candidate, artifacts, authority, or policy movement can invalidate prior continuity applicability.
5. **What context crossed the boundary?** Handoff is assembled from canonical structured work state and exact artifacts/evidence, not from an assumed provider-private transcript or memory.
6. **Did selection change authority?** Choosing a different runtime/provider/model never grants execution, delegation, verification, Git, secret, network, or human-acceptance authority by itself.
7. **What is unavailable or unknown?** Winds reports `UNKNOWN`/`UNAVAILABLE` instead of guessing identity, authentication readiness, capability, cost, or resume state.
8. **Can I audit what happened later?** Source identity, destination identity, workflow/stage binding, observed continuity result, material loss, and applicable evidence remain inspectable without rewriting historical failures.

The differentiated loop is:

> **canonical workflow/stage truth -> explicit target request -> locally qualified identity/capability truth -> authority check -> minimal structured continuity context -> observed execution/continuation result -> provenance-bound evidence -> truthful drift/reconstruction status**

not “pick the best model automatically,” not “trust a provider label in model output,” not “resume because a native identifier still exists,” and not “switch providers silently when the requested target is unavailable.”

## Frozen Product Invariants

```text
WINDS_SESSION != RUNTIME_SESSION
WORKFLOW_RUN != NATIVE_SESSION
STAGE_RUN != NATIVE_TURN

RUNTIME != PROVIDER
PROVIDER != MODEL
RUNTIME_LABEL != PROVIDER_IDENTITY
MODEL_LABEL != MODEL_IDENTITY
MODEL_OUTPUT_TEXT != MODEL_IDENTITY

CATALOG_DECLARED != VENDOR_DECLARED
VENDOR_DECLARED != WINDS_OBSERVED
WINDS_OBSERVED != AGENT_REPORTED
AGENT_REPORTED != HUMAN_DECIDED

RUNTIME_DISCOVERY != EXECUTION_AUTHORITY
PROVIDER_AVAILABILITY != AUTHENTICATION_READINESS
MODEL_AVAILABILITY != MODEL_SELECTION_AUTHORITY
MODEL_SELECTION != VERIFICATION_AUTHORITY

REQUESTED_TARGET != OBSERVED_TARGET
REQUESTED_TARGET_UNAVAILABLE != SILENT_FALLBACK
AMBIGUOUS_TARGET => FAIL_CLOSED_OR_REQUIRE_EXPLICIT_CHOICE

HANDOFF != NATIVE_RESUME
RECONSTRUCTED != RESUMED
REASSIGNED != RESUMED
NATIVE_SESSION_CANDIDATE != PROVEN_LIVE_SESSION

PROVIDER_OR_MODEL_DRIFT => PRIOR_PROVIDER_BOUND_CONTINUITY_STALE
RUNTIME_OR_VERSION_DRIFT => REQUALIFY_APPLICABLE_CONTINUITY
WORKFLOW_OR_STAGE_ATTEMPT_DRIFT => PRIOR_STAGE_BOUND_CONTINUITY_STALE
CANDIDATE_OR_ARTIFACT_DRIFT => PRIOR_CANDIDATE_BOUND_EVIDENCE_STALE

CONTEXT_TRANSFER != PRIVATE_PROVIDER_MEMORY_TRANSFER
UNOBSERVABLE_PRIVATE_STATE => UNAVAILABLE
REDACTED_OR_OMITTED_CONTEXT != COMPLETE_CONTEXT

TARGET_SELECTION_CANNOT_EXPAND_AUTHORITY
TARGET_SELECTION_CANNOT_SELF_AUTHORIZE_SECRETS
TARGET_SELECTION_CANNOT_SELF_AUTHORIZE_GIT_LANDING
TARGET_SELECTION_CANNOT_SELF_AUTHORIZE_NETWORK_SCOPE

NO_AUTOMATIC_PROVIDER_ROUTING
NO_SILENT_MODEL_SWITCH
NO_MAGIC_WINNER
NO_SILENT_LANDING
VERIFY_THE_EXACT_CANDIDATE
CURRENT_ONE_PROCESS_ARCHITECTURE_PRESERVED
```

Existing Spec 003/006/007/008 Git, evidence, authority, terminal-lifecycle, recovery, platform, privacy, workflow, decision, and human-landing invariants remain unchanged. Historical failed, rejected, stale, reconstructed, or unavailable evidence remains historical and MUST NOT be rewritten as success.

Spec 006 live-runtime nonclaims also remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Spec 009 may specify deterministic identity, continuity, handoff, and drift proof levels. Nothing in this specification constitutes or upgrades a separate live provider/runtime acceptance lane.

## User Scenarios & Testing

### User Story 1 - Select an Exact Admitted Target Without Silent Routing (Priority: P1)

A developer explicitly chooses an admitted runtime/provider/model target for an authorized stage and can see the exact requested identity before execution. Winds either uses a target that satisfies the explicit request and authority ceiling or reports why it cannot; it never silently switches to another provider/model because the preferred target is unavailable.

**Independent Test**: Create deterministic target fixtures covering one exact match, multiple ambiguous matches, unavailable runtime, unavailable model, provider/model unknown, stale catalog declaration, and a tempting alternate target. Verify exact match proceeds only within existing authority, ambiguity requires explicit resolution, and every unavailable case stays unavailable unless the user or an already-accepted explicit policy selects a different target.

**Acceptance Scenarios**:

1. A target request distinguishes runtime, provider, and model fields where those identities are applicable and observable.
2. An exact qualified match does not imply authority beyond the existing stage/delegation ceiling.
3. Multiple plausible targets do not trigger automatic winner selection.
4. An unavailable requested target does not silently fall back to another provider/model/runtime.
5. An explicit later choice of a different target is recorded as a new target decision rather than rewriting the original request.

### User Story 2 - See Requested, Declared, and Observed Identity Separately (Priority: P1)

A developer can distinguish what they requested, what a catalog says, what a runtime/vendor declares, what Winds locally observes, and what an agent merely claims about itself. Winds does not promote self-reported model/provider prose into observed identity.

**Independent Test**: Feed fixtures where catalog metadata, runtime declarations, structured local observations, environment labels, and agent output agree, disagree, or are missing. Verify every source remains labelled and conflicts or missing observations yield explicit mismatch/unknown truth rather than an invented canonical identity.

**Acceptance Scenarios**:

1. Requested target identity and observed target identity remain separate fields/truth claims.
2. Catalog and vendor/runtime declarations remain source-labelled declarations until an accepted local observation proves the applicable fact.
3. Agent text such as “I am model X” does not become Winds-observed model identity.
4. Conflicting identity sources remain a visible conflict and cannot be collapsed by source priority unless a later Plan defines an accepted deterministic rule consistent with this spec.
5. Unknown provider/model identity remains `UNKNOWN` rather than inferred from runtime display names.

### User Story 3 - Hand Off Between Codex and Claude Without Pretending Native Resume (Priority: P1)

A developer can hand an authorized workflow stage from one currently admitted runtime family to the other using canonical Winds context and exact artifacts/evidence. The destination is a new actor/continuity event unless an independently proven native-resume rule applies; cross-runtime handoff is never called native resume.

**Independent Test**: Build deterministic Codex-to-Claude and Claude-to-Codex handoff fixtures containing exact workflow/stage/candidate/artifact bindings, minimal role context, stale transcript material, provider-private-memory claims, and a source native-session identifier. Verify the destination receives only qualified structured context, preserves provenance, reports unavailable private state, and records `HANDOFF`/`RECONSTRUCTED` semantics rather than `RESUMED`.

**Acceptance Scenarios**:

1. Cross-runtime continuity identifies source and destination runtime/provider/model identity at the proof level actually available.
2. Handoff binds to the exact `WorkflowRun`, `StageRun`, candidate, artifacts, evidence, and authority ceiling applicable to the destination role.
3. Provider-private memory/transcript state not captured in canonical Winds context is reported unavailable rather than implied transferred.
4. A new destination native session is not labelled native resume merely because it receives prior canonical context.
5. Reviewer independence and evidence provenance survive the handoff boundary.

### User Story 4 - Detect Provider, Model, Runtime, and Native-Session Drift (Priority: P1)

A developer can tell when the identity underlying a prior continuity event has changed. Winds marks affected provider/model/runtime/native-session-bound continuity or evidence stale/not-applicable and requires requalification where needed rather than attaching new work to old identity by display-name coincidence.

**Independent Test**: Record an observed target and continuity event, then mutate runtime executable/version, provider/model identity, native-session identity, workflow/stage attempt, candidate, artifact, or authority binding one variable at a time. Verify every material drift invalidates exactly the applicable prior claims while preserving historical evidence.

**Acceptance Scenarios**:

1. Provider/model drift invalidates prior provider/model-bound continuity applicability without deleting history.
2. Runtime executable/version drift triggers requalification for continuity claims that depend on that runtime observation.
3. Native-session identifier reuse without live ownership proof does not revive a prior session.
4. Workflow/stage attempt drift prevents old continuity evidence from becoming current stage evidence.
5. Candidate/artifact movement preserves historical target/continuity records but invalidates stale candidate-bound applicability.

### User Story 5 - Distinguish Native Resume, Reconstruction, Reassignment, and Unavailable Continuation (Priority: P1)

A developer returning to work sees the strongest continuity proof Winds actually has: proven native resume, Winds reconstruction from canonical state, reassignment to a new actor/target, handoff, ownership loss, or unavailable continuation. Native identifiers alone are never sufficient proof of resume.

**Independent Test**: Exercise fixtures for exact accepted native-resume proof, persisted native identifier without live ownership, missing runtime, changed runtime version, cross-runtime handoff, same-runtime new session from canonical state, stale actor binding, and corrupt continuity metadata. Verify each maps to a distinct truthful result and nothing weaker is promoted to `RESUMED`.

**Acceptance Scenarios**:

1. `RESUMED` requires the pre-existing accepted proof applicable to the native runtime/session in use.
2. A new session built from canonical Winds state is `RECONSTRUCTED` or `REASSIGNED` as applicable, not `RESUMED`.
3. Cross-runtime transfer is `HANDOFF`, optionally with reconstruction semantics, never native resume.
4. Missing or ambiguous ownership produces explicit ownership-loss/unavailable/recovery-required truth.
5. Corrupt or incompatible continuity metadata fails closed and remains inspectable for diagnosis.

### User Story 6 - Bind Target Continuity to Exact Workflow and Stage Truth (Priority: P1)

A developer can see which runtime/provider/model/native-session observation belongs to each workflow stage attempt. Target identity cannot float globally and silently apply to a different stage, retry, candidate, or authority context.

**Independent Test**: Create two workflows and multiple stage attempts with overlapping display names and reused target labels. Attempt to replay a continuity observation from one stage into another. Verify exact canonical binding prevents cross-workflow/cross-attempt attachment and leaves the original record auditable.

**Acceptance Scenarios**:

1. Every canonical continuity event carries exact workflow and stage-attempt identity where stage-scoped.
2. Display labels, model names, runtime names, or native IDs cannot substitute for canonical workflow/stage identity.
3. Retry creates a new stage-attempt binding and does not silently inherit stale target evidence.
4. Reassignment records the new actor/target binding while retaining the prior attempt history.
5. Workflow completion, candidate verification, and human acceptance remain independent of provider/model continuity success.

### User Story 7 - Keep Authentication and Secrets Outside Canonical Work Truth (Priority: P1)

A developer can use an already configured admitted runtime without Winds persisting raw provider credentials into canonical workflow, evidence, transcript, or repository state. Winds reports authentication readiness only at the proof level actually available and does not scrape, acquire, broker, or expose credentials to make a target work.

**Independent Test**: Exercise deterministic fixtures containing secret-like values, credential references, missing credentials, misleading environment labels, an agent request to print credentials, and a target that appears available but whose authentication readiness is unknown. Verify raw secrets are not persisted/promoted and unknown readiness remains unknown until an accepted observation proves otherwise.

**Acceptance Scenarios**:

1. Provider availability and authentication readiness remain separate truth claims.
2. Raw credentials are never required as canonical workflow/continuity fields.
3. Secret-like provider material is redacted/omitted from durable evidence according to existing policy boundaries.
4. A target cannot self-authorize credential access or ask Winds to scrape login/token material through model output.
5. Unknown authentication readiness cannot be represented as authenticated merely because the runtime executable exists.

### User Story 8 - Explain Continuity and Failure Without Creating New Authority (Priority: P1)

A developer can inspect a concise projection showing requested target, observed target, identity source/proof level, workflow/stage binding, continuity result, drift/staleness, unavailable context, authority ceiling, and why execution/continuation is blocked. Inspection itself does not mutate state or grant authority.

**Independent Test**: Render projections for exact match, mismatch, unknown identity, unavailable target, reconstructed, handed-off, resumed, stale, ownership-lost, authentication-unknown, and authority-denied fixtures. Inject forged agent/provider labels and verify the projection remains source-labelled and canonical.

**Acceptance Scenarios**:

1. Status explains both requested and observed identity without collapsing them.
2. Status identifies the exact continuity proof level and any material loss or unavailable context.
3. Why-blocked distinguishes identity mismatch, target unavailable, authentication unknown, capability unavailable, stale state, and authority denial where applicable.
4. Agent output cannot change the displayed canonical proof level or authority ceiling.
5. Inspection is non-mutating unless a later accepted task explicitly authorizes one narrow transition.

### User Story 9 - Retain Observed Usage/Cost Metadata Without Guessing (Priority: P2)

When an admitted runtime supplies structured trustworthy usage or cost observations for a continuity event, a developer can inspect that observed metadata with its source and scope. When it is not supplied or cannot be qualified, Winds reports unknown rather than estimating usage/cost as fact.

**Independent Test**: Exercise structured usage present, partially present, absent, conflicting, stale, and agent-prose-only fixtures. Verify only qualified structured observations become observed metadata and missing values remain unknown.

**Acceptance Scenarios**:

1. Usage/cost metadata is optional and source-labelled.
2. Missing structured observations remain unknown rather than inferred from token-count prose or pricing assumptions.
3. Stale usage/cost observations remain attached to their original continuity/run event.
4. Usage/cost metadata cannot become verification, routing, or winner-selection authority.
5. This story does not require adding a billing API, telemetry service, provider SDK, or pricing database.

## Model Mesh Safety Edge Cases

- Runtime display name implies a provider/model that Winds cannot independently observe.
- Agent output claims a different model/provider than the structured local observation.
- Catalog metadata is stale while the runtime is present locally.
- The requested runtime exists but authentication readiness is unknown.
- A requested model is unavailable while another model would be easy to use.
- Multiple admitted targets satisfy a broad request and no explicit deterministic selection policy has been authorized.
- A native session ID is reused after process death or runtime upgrade.
- Runtime executable path remains the same while version or implementation changes.
- Provider/model identity changes mid-workflow or between stage attempts.
- A workflow retry reuses the same target label but has a new stage-attempt identity.
- A candidate/artifact changes after provider/model-bound evidence is gathered.
- Cross-runtime handoff includes persuasive source-agent conclusions intended to bias an independent reviewer.
- Provider-private transcript/history is unavailable but source agent claims it was transferred.
- Context redaction removes material required to judge continuity completeness.
- A target asks for secrets, broader filesystem access, network access, delegation, or Git actions beyond the existing authority ceiling.
- Provider/model output embeds fake `PASS`, `VERIFIED`, `HUMAN_ACCEPTED`, identity, capability, or authority labels.
- Runtime discovery finds an executable supplied by an untrusted workspace path or otherwise ambiguous provenance.
- Structured usage/cost metadata conflicts with agent prose or an earlier event.
- Continuity metadata is malformed, partially written, duplicated, replayed, from an unsupported schema, or internally contradictory.
- Winds restarts after durable continuity metadata exists but the underlying runtime/native session cannot be proven live.

## Functional Requirements

### Canonical identity and explicit target requests

- **FR-001**: Winds MUST represent runtime/harness identity separately from provider identity, model identity, canonical Winds session identity, workflow/stage identity, and provider-native session/turn identity.
- **FR-002**: Winds MUST NOT derive provider or model identity solely from a runtime display label, executable name, native-session identifier, model output text, or mutable UI label.
- **FR-003**: A target request MUST preserve the explicit identity dimensions the user or already-accepted policy specified and MUST leave unspecified dimensions unspecified rather than silently filling them from guesses.
- **FR-004**: Requested target identity and observed target identity MUST remain distinct canonical truth claims.
- **FR-005**: Ambiguous target requests MUST fail closed or require an explicit deterministic choice; Winds MUST NOT select a winner based on undocumented preference, model reputation, cost guess, latency guess, or prior success.
- **FR-006**: If an explicitly requested target is unavailable or cannot be qualified, Winds MUST report that condition without silently switching provider, model, or runtime.
- **FR-007**: A later explicit target change MUST be recorded as a new decision/event and MUST NOT rewrite the original request as if the new target had always been requested.
- **FR-008**: Target selection MUST NOT expand execution, delegation, filesystem, network, secret, verification, Git, landing, or human-decision authority.
- **FR-009**: Target selection MUST remain bounded by the exact workflow/stage actor role and authority ceiling already applicable to the operation.
- **FR-010**: Canonical target identity MUST use stable internal identity/reference semantics selected later by Plan and MUST NOT rely on mutable display names for equality.
- **FR-011**: Unknown provider/model/native identity MUST be representable explicitly as `UNKNOWN` or an equivalent fail-truthfully state rather than replaced by a guessed value.
- **FR-012**: An unavailable target MUST be distinguishable from an unknown target, an unauthorized target, an authentication-unknown target, and an ambiguous target.

### Discovery, declarations, observation, and capability truth

- **FR-013**: Winds MUST keep catalog-declared, vendor/runtime-declared, Winds-locally-observed, agent-reported, and human-decided identity/capability facts source-labelled where the distinction affects trust.
- **FR-014**: Runtime discovery MUST NOT itself grant execution authority, authentication readiness, provider/model trust, or capability authority.
- **FR-015**: A catalog declaration MUST NOT be promoted to local availability merely because it exists in configuration or documentation.
- **FR-016**: A runtime/vendor declaration MUST NOT be promoted to Winds-observed model/provider identity unless the later accepted Plan defines a concrete local observation that proves that claim at the needed level.
- **FR-017**: Agent-reported provider/model/capability text MUST remain agent-reported and MUST NOT become Winds-observed truth.
- **FR-018**: Conflicting qualified identity sources MUST produce explicit conflict/mismatch truth until an accepted deterministic rule resolves applicability; the conflict MUST NOT be silently discarded.
- **FR-019**: Capability state MUST remain separate from authority state; an observed capability does not authorize its use.
- **FR-020**: Authentication readiness MUST remain separate from executable presence, provider availability, model availability, and execution authority.
- **FR-021**: Provider/model availability observations MUST be scoped to the exact runtime/configuration context for which they were observed and MUST become stale when their material observation basis changes.
- **FR-022**: The specification MUST NOT require probing a remote provider, sending user data, or performing an authenticated call merely to turn an unknown catalog/declaration into an observation; any such behavior requires later explicit Plan/Tasks authority.

### Routing, fallback, and authority ceilings

- **FR-023**: Spec 009 MUST prohibit automatic provider/model winner selection and silent fallback.
- **FR-024**: Winds MAY later support an explicit deterministic target policy only if a separately accepted Plan/Tasks slice defines its exact inputs, user-visible semantics, authority ceiling, and no-silent-fallback behavior consistent with this specification.
- **FR-025**: A policy-authorized target choice MUST remain inspectable as policy-selected rather than human-selected.
- **FR-026**: No target policy may treat provider/model output quality claims as verification authority.
- **FR-027**: No target policy may silently cross an execution/delegation/network/filesystem/secret/Git authority boundary to satisfy availability.
- **FR-028**: Failure to use the requested target MUST preserve the original failure/unavailability evidence even if a later explicit target succeeds.
- **FR-029**: An alternate target MUST require explicit user selection or a separately accepted explicit policy; availability alone is insufficient authorization.
- **FR-030**: Usage/cost/latency observations MUST NOT create automatic routing authority in Spec 009.
- **FR-031**: Target selection MUST NOT create a magic score, ranking, or “best model” claim as canonical Winds truth.
- **FR-032**: A target-selection explanation MUST identify the source of the choice and the authority under which it was made.

### Continuity, handoff, reconstruction, and native resume

- **FR-033**: Winds MUST represent native resume, same-runtime reconstruction, cross-runtime handoff, reassignment, ownership loss, and unavailable continuation as distinct continuity outcomes.
- **FR-034**: `RESUMED` MUST require the existing accepted proof applicable to the exact runtime/native-session identity; persisted identifiers alone MUST NOT prove live ownership or resume.
- **FR-035**: A new native session initialized from canonical Winds state MUST be `RECONSTRUCTED` or `REASSIGNED` as applicable, not `RESUMED`.
- **FR-036**: Cross-runtime transfer MUST be identified as `HANDOFF` and MUST NOT be represented as native resume.
- **FR-037**: A continuity event MUST identify source and destination runtime/provider/model/native-session identity at the exact proof level available; unknown dimensions MUST remain unknown.
- **FR-038**: Handoff context MUST be derived from canonical structured Winds state plus exact required artifacts/evidence, not from an unqualified transcript dump.
- **FR-039**: Provider-private memory or history that Winds cannot observe and bind MUST be represented as unavailable rather than inferred or claimed transferred.
- **FR-040**: Continuity context MUST preserve exact source provenance and distinguish transferred facts from source-agent recommendations or prose.
- **FR-041**: Independent reviewer handoff MUST exclude author/source-agent persuasion, confidence, or winner recommendations by default while retaining required exact evidence/provenance/authority facts.
- **FR-042**: Redacted or omitted continuity context MUST carry explicit incompleteness/loss markers where that omission could affect interpretation.
- **FR-043**: Missing required context MUST block or downgrade the applicable continuity claim rather than being silently reconstructed from model prose.
- **FR-044**: Duplicate/replayed continuity events MUST NOT create duplicate canonical current ownership, resume, or handoff truth.
- **FR-045**: Corrupt, partial, contradictory, or unsupported continuity metadata MUST fail closed and remain available for recovery/diagnosis rather than silently reinitializing successful continuity.

### Workflow, stage, candidate, artifact, and staleness binding

- **FR-046**: Every stage-scoped target/continuity event MUST bind to the exact canonical `WorkflowRun` and `StageRun` attempt for which it applies.
- **FR-047**: A continuity event MUST NOT attach to a workflow/stage by display name, runtime label, provider/model label, filesystem path, or native identifier coincidence.
- **FR-048**: Retry MUST create or use the new canonical stage-attempt binding and MUST NOT silently inherit prior attempt target/continuity applicability.
- **FR-049**: Reassignment MUST record a new actor/target binding while preserving prior actor/target history.
- **FR-050**: Material runtime executable/version drift MUST make continuity observations that depend on that runtime basis stale pending requalification.
- **FR-051**: Material provider or model identity drift MUST make prior provider/model-bound continuity applicability stale without deleting history.
- **FR-052**: Native-session identity or ownership drift MUST invalidate prior live/resume applicability unless the existing accepted native proof re-establishes it.
- **FR-053**: Workflow or stage-attempt movement MUST invalidate prior stage-bound target/continuity applicability.
- **FR-054**: Candidate or required artifact movement MUST invalidate prior candidate/artifact-bound continuity evidence according to existing Spec 008 freshness semantics.
- **FR-055**: Authority/policy movement MUST invalidate prior continuity permission when the new authority ceiling no longer permits the operation.
- **FR-056**: Staleness MUST be explicit, source-labelled, and deterministic for identical canonical inputs.
- **FR-057**: Historical stale/drifted target and continuity records MUST remain inspectable and MUST NOT be rewritten as current success after requalification.
- **FR-058**: Stage/workflow completion, candidate verification, and human acceptance MUST remain separate from target/continuity success.

### Authentication, credentials, privacy, and secret boundaries

- **FR-059**: Spec 009 MUST NOT require raw provider credentials as canonical workflow, stage, target, continuity, evidence, decision, or repository fields.
- **FR-060**: Winds MUST NOT acquire credentials, scrape passwords/tokens, automate login, broker refresh tokens, or persist raw secrets merely to make a target available.
- **FR-061**: Provider/model selection MUST NOT grant an actor new credential or secret access.
- **FR-062**: Existing protected-data/redaction rules MUST apply before durable continuity/evidence persistence; redaction MUST NOT be represented as complete evidence when required material is missing.
- **FR-063**: Authentication readiness MAY be represented only at the proof level an accepted local observation can establish; absent proof remains unknown.
- **FR-064**: Runtime executable presence MUST NOT imply authentication readiness.
- **FR-065**: Agent/provider output requesting secret disclosure MUST NOT create authority to expose or persist those secrets.
- **FR-066**: Credential references, if later selected by Plan, MUST be non-secret references with explicit scope/lifecycle semantics and MUST NOT expand authority by identifier possession alone.
- **FR-067**: Any future authenticated provider API, credential store integration, login flow, or secret broker requires separate explicit Plan/Tasks authority and security review; this specification alone does not authorize it.

### Observability, usage, cost, and operator truth

- **FR-068**: Winds MUST provide a non-mutating operator-facing projection of requested target, observed target, source/proof level, workflow/stage binding, continuity result, staleness/drift, material context loss, and applicable authority ceiling.
- **FR-069**: The projection MUST distinguish at minimum target unavailable, target unknown, target ambiguous, authentication unknown, capability unavailable, stale identity, ownership lost, and authority denied where those states apply.
- **FR-070**: Forged model/runtime output MUST NOT change canonical projection truth unless independently admitted through an accepted observation path.
- **FR-071**: Structured usage/cost metadata MAY be retained only when supplied by a qualified trustworthy observation selected later by Plan; the metadata MUST remain bound to its exact event/source/scope.
- **FR-072**: Missing usage/cost metadata MUST remain unknown and MUST NOT be estimated as observed fact from prose, stale price tables, or guessed token counts.
- **FR-073**: Usage/cost metadata MUST NOT become candidate verification, human acceptance, automatic routing, or winner-selection authority.
- **FR-074**: Spec 009 MUST NOT require a billing API, pricing database, telemetry service, external observability platform, or provider SDK merely to satisfy optional usage/cost visibility.

### Verification, reviewer independence, and evidence authority

- **FR-075**: Provider/model identity and continuity evidence MUST remain subject to the Constitution distinction `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`.
- **FR-076**: Target/continuity success MUST NOT imply candidate verification, correctness, safety, review acceptance, or human landing acceptance.
- **FR-077**: Verification MUST remain bound to the exact Git base/candidate and applicable evidence under existing accepted rules regardless of which target produced the candidate.
- **FR-078**: A provider/model/runtime MUST NOT verify or human-accept its own output merely because it produced a continuity record or execution result.
- **FR-079**: Independent review context MUST preserve reviewer independence across cross-runtime handoff and MUST source-label any author/provider/model recommendations that are intentionally included.
- **FR-080**: Agent-generated `PASS`, `VERIFIED`, `ACCEPTED`, identity, capability, cost, authority, or continuity labels MUST remain non-authoritative until independently observed/admitted by an accepted path.
- **FR-081**: Historical target failures, unavailable states, mismatches, stale observations, and reconstruction losses MUST remain evidence and MUST NOT be erased by a later successful target.
- **FR-082**: Exact-candidate review movement MUST invalidate stale review/evidence applicability according to existing governance; provider/model continuity MUST NOT weaken that rule.

### Scope, platform, dependency, and governance boundaries

- **FR-083**: The first Spec 009 implementation program MUST prefer the already admitted Codex and Claude runtime families and MUST NOT add a broader provider/runtime fleet merely to claim “multi-provider” support.
- **FR-084**: Spec 009 MUST NOT add a generic plugin/provider marketplace, arbitrary adapter ABI, dynamic code loading, third-party executable extension host, or broad integration SDK.
- **FR-085**: Spec 009 MUST NOT require a central provider gateway such as LiteLLM or any equivalent proxy; a later Plan may consider a dependency only if a concrete accepted current need proves existing runtime surfaces insufficient and the dependency passes exact qualification.
- **FR-086**: Spec 009 MUST preserve the current one-process architecture and MUST NOT add a persistent owner, daemon, server, socket, HTTP/SSE/WebSocket endpoint, or new IPC/control plane.
- **FR-087**: Spec 009 MUST NOT add remote execution, SSH/mobile/team continuation, cloud orchestration, or service control-plane behavior.
- **FR-088**: Spec 009 MUST NOT add browser automation/profile/CDP runtime, browser credentials, screenshots as verification authority, or Browser Twin.
- **FR-089**: Spec 009 MUST NOT add learning/training/fine-tuning/RL, automatic skill/policy mutation, learned routing, protected-holdout execution, or experiment-plane activation.
- **FR-090**: Spec 009 MUST NOT add MCP runtime expansion, ACP dependency expansion, A2A, or a generic runtime/tool protocol merely to implement Model Mesh continuity.
- **FR-091**: Spec 009 MUST NOT automatically select a candidate winner, merge, rebase, cherry-pick, push, create a PR, or land changes.
- **FR-092**: Every new dependency, provider API, SDK, gateway, catalog, persistence/schema mechanism, or secret integration remains a Plan/Tasks decision requiring exact need/version/revision/license/MSRV/platform/provenance/security/removal-path/YAGNI review before implementation may rely on it.
- **FR-093**: Platform claims MUST be limited to directly exercised domains; no native Windows, WSL2, Linux, or macOS provider/runtime continuity behavior may inherit proof solely from another platform.
- **FR-094**: Runtime/provider/model identity observations MUST not treat workspace roots, worktrees, runtime permission UX, catalog entries, or configuration declarations as OS sandbox proof.
- **FR-095**: Spec 009 MUST preserve all applicable accepted Spec 003/006/007/008 Git, evidence, authority, recovery, platform, privacy, terminal-lifecycle, workflow, decision, and human-landing invariants.

## Success Criteria

- **SC-001**: Deterministic identity fixtures preserve distinct Winds session, workflow/stage, runtime, provider, model, and native-session/turn identities with zero equality based solely on mutable display labels.
- **SC-002**: Requested/declared/observed/agent-reported identity fixtures retain source labels under agreement, conflict, and missing-data cases; zero agent self-report is promoted to Winds-observed identity.
- **SC-003**: Exact-target fixtures use only the explicitly admitted qualified target, while ambiguous/unavailable fixtures produce explicit choice/block truth with zero silent provider/model/runtime fallback.
- **SC-004**: Authority fixtures prove changing target identity grants zero additional execution, delegation, filesystem, network, secret, verification, Git, landing, or human-decision authority.
- **SC-005**: Cross-runtime Codex-to-Claude and Claude-to-Codex fixtures preserve exact workflow/stage/candidate/artifact/evidence provenance and always report handoff/reconstruction semantics rather than false native resume.
- **SC-006**: Exact native-resume, reconstruction, reassignment, handoff, ownership-loss, and unavailable-continuation fixtures produce distinct truthful outcomes with zero persisted-identifier-only `RESUMED` claims.
- **SC-007**: Runtime/version, provider/model, native-session, workflow/stage, candidate/artifact, and authority drift fixtures invalidate exactly the applicable prior continuity/evidence state while preserving history.
- **SC-008**: Cross-workflow and cross-stage replay fixtures create zero attachment of target/continuity state by runtime label, provider/model label, native identifier, display name, or filesystem-path coincidence.
- **SC-009**: Handoff-context fixtures transfer only required canonical structured state and exact artifacts/evidence; unavailable provider-private memory remains unavailable and persuasive source-agent prose is excluded from independent-review context by default.
- **SC-010**: Missing/redacted required continuity context produces explicit incompleteness/block/downgrade truth; zero omission is represented as complete transfer.
- **SC-011**: Authentication fixtures distinguish executable presence, provider/model availability, authentication readiness, and authority with zero false authenticated-ready claim when proof is absent.
- **SC-012**: Secret fixtures prove raw provider credentials are not persisted into canonical workflow/target/continuity/evidence/repository state and agent requests cannot self-authorize secret disclosure.
- **SC-013**: Duplicate/replayed/corrupt/incompatible continuity fixtures create zero duplicate current ownership/resume truth and fail closed with diagnosable recovery state.
- **SC-014**: Operator projections deterministically explain requested versus observed target, proof source, continuity result, drift/staleness, loss, blocker, and authority ceiling for every accepted fixture state.
- **SC-015**: Forged agent/runtime/provider `PASS`/`VERIFIED`/`ACCEPTED`/identity/capability/authority/continuity labels produce zero promotion into canonical Winds-observed, verification, human-decision, or authority truth.
- **SC-016**: Structured usage/cost fixtures preserve qualified observations when present and report unknown when absent/conflicting; zero guessed estimate is presented as observed fact.
- **SC-017**: Usage/cost fixtures create zero automatic routing, winner-selection, verification, or human-acceptance authority.
- **SC-018**: Workflow/stage completion fixtures never imply candidate verification or human acceptance because target/continuity succeeded.
- **SC-019**: Existing repository `quality` and applicable Spec 003/006/007/008 regression/platform/security/workflow gates remain green on the final exact implementation candidate.
- **SC-020**: Correctness/safety, Ponytail/YAGNI, and fresh independent review reach the final exact implementation candidate with zero unresolved material findings.
- **SC-021**: Final reconciliation truthfully proves no generic plugin/provider framework, broad provider fleet, central gateway requirement, daemon/IPC, remote execution, browser runtime, MCP/ACP expansion, learning subsystem, credential broker, automatic routing, automatic winner, or automatic landing path was introduced unless a separately accepted later amendment explicitly authorizes one.
- **SC-022**: Every claimed native Windows/WSL2/Linux/macOS target/continuity behavior is backed by direct applicable evidence for that domain; unexercised behavior remains explicitly unclaimed.
- **SC-023**: Historical failed/unavailable/mismatched/stale/reconstructed continuity evidence remains inspectable after later success; zero historical failure is rewritten as current success.
- **SC-024**: First implementation qualification demonstrates the accepted continuity contract on the already admitted Codex and Claude runtime families before any proposal to admit another runtime/provider family can claim necessity.
- **SC-025**: Spec 006 live-runtime nonclaims remain unchanged unless and until a separate exact live-runtime acceptance lane independently proves them.

## Explicit Non-Goals

Spec 009 does not authorize:

- automatic provider/model routing, learned routing, silent fallback, magic winner scoring, or autonomous “best model” selection;
- broad third-party provider/runtime admission merely to increase provider count;
- a generic plugin/provider marketplace, arbitrary adapter ABI, dynamic code loading, third-party extension host, or integration SDK;
- a mandatory provider gateway/proxy, billing service, pricing database, observability SaaS, or provider API/SDK;
- credential acquisition, password/token scraping, automated login, refresh-token brokerage, raw-secret durable storage, or new secret-disclosure authority;
- persistent background ownership, daemon/server/socket/IPC/control API, or detach/reattach across Winds process exit;
- remote execution, SSH/mobile/team continuation, cloud control plane, or service orchestration;
- browser automation, browser profile/credential management, CDP, screenshots as verification evidence, or Browser Twin;
- MCP runtime expansion, ACP dependency expansion, A2A, or a generic runtime/tool protocol layer;
- verified-learning activation, skill mutation/optimization/promotion, experiment-plane activation, protected-holdout execution, training/fine-tuning/RL, or automatic policy mutation;
- vector/embedding/RAG memory, semantic memory engine, provider-private-memory extraction, or transcript-as-canonical-memory architecture;
- automatic candidate comparison/winner selection, automatic Git mutation/PR creation, or automatic landing;
- provider/model output becoming verification, correctness, reviewer, human-acceptance, or Git authority;
- claiming native resume from persisted native identifiers without the existing accepted live ownership proof;
- choosing a persistence engine, provider SDK, API protocol, credential store, catalog schema, gateway, telemetry backend, or adapter architecture at specification stage;
- weakening or replacing accepted Spec 003/006/007/008 exact-candidate, evidence, authority, reviewer-independence, terminal-lifecycle, recovery, platform, privacy, workflow, decision, or human-landing truth.

## Assumptions

- Existing canonical workspace/workstream/session/runtime/evidence/workflow/stage identities remain starting truth and are referenced rather than replaced.
- Existing `RuntimeKind` remains bounded to the already accepted Codex and Claude families at specification entry; this spec defines continuity semantics and does not itself admit another runtime family.
- Existing Spec 006 runtime discovery, native-session ownership/resume, authority, delegation, context-transfer, and live-runtime nonclaims remain authoritative where applicable.
- Existing Spec 008 `WorkflowRun`, `StageRun`, candidate/artifact freshness, decision lineage, retry/reconstruction, redaction, and human-acceptance semantics remain authoritative and are bound into Model Mesh rather than duplicated.
- A later Plan may choose the smallest identity/reference and continuity-event representation that satisfies these requirements, but it may not introduce a generic provider abstraction simply for hypothetical future providers.
- A later Plan may define narrow deterministic target-selection UX/policy mechanics only within the explicit no-silent-routing and authority constraints above.
- A later Plan may identify already-existing structured runtime/vendor observations sufficient for provider/model identity or usage metadata; if such proof does not exist, the correct value remains unknown rather than requiring a new provider API by default.
- Any new dependency, provider SDK/API, credential mechanism, protocol, persistence choice, or runtime family requires separately accepted necessity and qualification before implementation may rely on it.

## Governance / Downstream Authorization

Canonical acceptance of this specification authorizes **Spec 009 Plan creation only**.

It does not authorize Tasks, implementation, dependencies, provider SDKs/APIs, gateways, credential mechanisms, new runtime/provider admission, storage/schema changes, source changes, daemon/IPC, remote execution, browser execution, learning, automatic routing, automatic fallback, automatic winner selection, or automatic landing.

Only after this specification lands and is post-merge verified may repository truth state:

```text
SPEC_009_ENTRY=CLOSED_CANONICAL
SPEC_009_SPEC=CLOSED_CANONICAL
SPEC_009_PLAN_AUTHORIZED=YES
SPEC_009_TASKS_AUTHORIZED=NO
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
```
