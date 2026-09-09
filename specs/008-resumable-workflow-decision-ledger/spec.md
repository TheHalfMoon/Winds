# Feature Specification: Resumable Workflow & Decision Ledger

**Feature Branch**: `spec/008-resumable-workflow-decision-ledger`

**Created**: 2026-09-09

**Canonical Base**: `97bc89ad08531f479fcac0cdf8ec5a2b0aa7a96d`

**Canonical Entry Authority**: `docs/research/017-spec-008-entry-gate.md`, PR #140, merge `97bc89ad08531f479fcac0cdf8ec5a2b0aa7a96d`

**Status**: Authorized for specification only; planning, tasks, dependencies, migrations, runtime implementation, storage-engine selection, provider/browser execution, daemon/IPC work, learning, remote execution, and product-source changes are NOT authorized by this file alone

**Input**: Give Winds a first-class, proof-carrying workflow and decision-state contract so a developer can leave and return to multi-stage work and truthfully determine which exact workflow/stage attempt is active, which actor/candidate/artifacts/evidence apply, what is stale or blocked, what can resume, what can only be reconstructed, and what remains unverified or unaccepted.

## Product North Star

Winds should preserve workflow truth independently of any one terminal, transcript, agent/provider session, UI lifetime, or mutable display label.

For every resumable workflow the user should be able to answer:

1. **Which exact workflow and stage attempt is this?** Workflow/stage identity is canonical and separate from session or display identity.
2. **Which actor/run instance is bound to it?** A stale or replaced runtime cannot inherit stage authority by name or identifier coincidence.
3. **Which exact inputs and artifacts were prepared for this attempt?** Stale artifacts cannot silently satisfy a newer stage gate.
4. **What happened to prior attempts and decisions?** Failed, rejected, reverted, superseded, and stale history remains auditable.
5. **What can actually resume?** Native/runtime resume, Winds reconstruction, reassignment, and ownership loss remain distinguishable proof levels.
6. **Why is progress blocked or retrying?** Retry/no-progress budgets, blockers, approvals, and external conditions are explicit rather than inferred from prose.
7. **What evidence is complete, omitted, or redacted?** Redaction never masquerades as complete evidence.
8. **Is the workflow merely procedurally complete, or is its candidate verified and human-accepted?** Completion, verification, and acceptance remain separate truths.

The differentiated loop is:

> **canonical workflow state -> exact stage attempt -> fresh artifacts -> bounded action -> evidence/decision record -> truthful resume/reconstruction -> explicit verification/human acceptance**

not “replay a transcript and call it resumed,” not “trust the current agent’s narrative,” and not “persist enough state to imply authority.”

## Frozen Product Invariants

```text
WORKFLOW_RUN != CHAT_SESSION
STAGE_RUN != AGENT_SESSION
DISPLAY_NAME != IDENTITY
STAGE_ATTEMPT_HAS_UNIQUE_RUN_INSTANCE

STAGE_COMPLETE != CANDIDATE_VERIFIED
WORKFLOW_COMPLETE != HUMAN_ACCEPTED
VERIFICATION_PENDING != VERIFIED

PREPARED_STAGE_HAS_ARTIFACT_BASELINE
REUSED_STALE_ARTIFACT => STAGE_GATE_FAILS
CHANGED_CANDIDATE_INVALIDATES_STALE_STAGE_EVIDENCE

STALE_ACTOR_BINDING => BLOCK_OR_RECONSTRUCT
RESUMED != RECONSTRUCTED
RECONSTRUCTED_STAGE_MUST_REPORT_LOSS
OWNERSHIP_LOST != RESUMED

RETRY_REQUIRES_BOUNDED_POLICY
NO_PROGRESS_WITHIN_BUDGET => PAUSE_OR_BLOCK
AMBIGUOUS_NON_IDEMPOTENT_EFFECT => NO_SILENT_RETRY

DECISION_RECORDS_ARE_APPEND_ONLY
SUPERSEDED_DECISION_REMAINS_AUDITABLE
FAILED_OR_REVERTED_ATTEMPT_IS_EVIDENCE

REDACTED_DATA_MUST_RETAIN_EXPLICIT_OMISSION_MARKERS
REDACTION != EVIDENCE_COMPLETENESS

AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
SOURCE_FOUND != SOURCE_ADMITTED
NO_AUTOMATIC_WINNER
NO_SILENT_LANDING
VERIFY_THE_EXACT_CANDIDATE
CURRENT_ONE_PROCESS_ARCHITECTURE_PRESERVED
NO_DAEMON_OR_IPC_IN_SPEC_008
```

Existing Spec 003/006/007 evidence, authority, terminal-lifecycle, recovery, platform, privacy, and human-landing invariants remain unchanged. Historical failed evidence remains historical and MUST NOT be rewritten as success.

Spec 006 live-runtime nonclaims also remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Spec 008 may specify continuation proof levels and deterministic fixtures, but nothing in this specification constitutes or upgrades those separate live provider/runtime acceptance lanes.

## User Scenarios & Testing

### User Story 1 - Return to a Multi-Stage Workflow and See Exact Current Truth (Priority: P1)

A developer returns to Winds after leaving a multi-stage task and sees the exact `WorkflowRun`, current `StageRun` attempt, lifecycle state, actor binding, candidate/artifact bindings, blockers, and allowed continuation semantics without relying on transcript replay or mutable labels.

**Independent Test**: Create a deterministic multi-stage fixture, advance it through prepare/start/wait/block/complete states, restart the Winds process between transitions, and verify the same canonical workflow/stage identity and truthful continuation state are recovered without claiming native/runtime resume where only reconstruction is proven.

**Acceptance Scenarios**:

1. A prepared stage has a unique stage-attempt identity and explicit lifecycle state before work begins.
2. Mutable stage/workflow display labels do not change canonical identity.
3. Process restart reconstructs canonical workflow state from accepted durable state but does not imply the original runtime/session is live.
4. A stage cannot become active if required canonical bindings are missing, stale, malformed, or ambiguous.
5. A completed stage does not automatically make the workflow candidate `VERIFIED` or `HUMAN_ACCEPTED`.

### User Story 2 - Reject Stale Artifacts and Moved Candidates (Priority: P1)

A developer prepares a stage against exact upstream artifacts/candidate state. If those inputs change before the stage consumes or verifies them, Winds marks the affected baseline stale/not-applicable rather than silently treating old material as current.

**Independent Test**: Prepare stage B from stage A artifact/candidate X, mutate the required artifact or move to candidate Y, then attempt start/complete/verify. Verify stale inputs fail the applicable gate while historical evidence remains inspectable.

**Acceptance Scenarios**:

1. Stage preparation records an explicit artifact/input baseline sufficient to revalidate freshness.
2. Changed required input identity/hash/reference makes the prepared baseline stale.
3. Candidate movement invalidates prior candidate-bound stage evidence without deleting history.
4. An unchanged artifact from an older attempt cannot be presented as newly produced work merely because a stage restarted.
5. Freshness failure is explicit and cannot be overridden by terminal/agent prose.

### User Story 3 - Retry Only Within Bounded, Truthful Progress Rules (Priority: P1)

A developer can see why a stage is retrying, how much retry/no-progress budget remains, and when Winds pauses or blocks instead of repeating the same ineffective or ambiguous action indefinitely.

**Independent Test**: Exercise repeated same-candidate hashes, repeated diff signatures, repeated failure classes, repeated checkpoints with no material progress, exhausted retry budget, and ambiguous non-idempotent effects. Verify Winds stops/requires reconciliation at the defined boundary rather than silently retrying forever.

**Acceptance Scenarios**:

1. Every retryable stage uses an explicit finite policy/budget selected later by Plan/Tasks.
2. Repeated non-progress observations consume a bounded budget and eventually produce an explicit blocked/paused outcome.
3. A retry never erases or rewrites the failed attempt that caused it.
4. An ambiguous non-idempotent side effect is not silently repeated without proven idempotency or an explicitly accepted at-least-once/reconciliation rule.
5. Retry exhaustion is visible and does not imply stage completion.

### User Story 4 - Keep Decisions Append-Only and Auditable (Priority: P1)

A developer can inspect every material workflow decision, who or what made it, its authority/source class, exact candidate/evidence basis, rationale, result, and supersession lineage. Changing a decision adds history rather than rewriting it.

**Independent Test**: Record accepted, rejected, reverted, and superseding decisions across multiple stage attempts. Verify prior records remain immutable/auditable and no author/agent can convert its own recommendation into human acceptance or verification authority.

**Acceptance Scenarios**:

1. Every canonical decision has stable identity and workflow/stage binding where applicable.
2. Supersession creates a new decision relationship; it does not mutate away the old decision.
3. Rejected/reverted decisions and failed attempts remain inspectable.
4. Decision source/authority distinguishes agent-reported, Winds-observed/policy, and human-decided truth where applicable.
5. Decision references to candidate/evidence become stale/not-applicable when their exact subject changes.

### User Story 5 - Recover From Restart, Stale Ownership, and Corrupt State Without Overclaim (Priority: P1)

A developer can distinguish a truly resumed native/runtime session from a new Winds reconstruction, reassignment, unavailable continuation, and ownership loss. Corrupt or incompatible durable workflow state fails closed and remains inspectable instead of silently resetting to a clean workflow.

**Independent Test**: Exercise valid reconstruction after Winds restart, exact runtime/native resume where existing accepted runtime semantics prove it, stale actor mapping, missing runtime mapping, partial/corrupt workflow state, unknown schema version, and interrupted durable write. Verify each outcome reports only the proof actually established.

**Acceptance Scenarios**:

1. `RESUMED` requires the existing accepted proof applicable to the runtime/native identity in use.
2. A new actor/runtime created from canonical Winds state is `RECONSTRUCTED`, not `RESUMED`.
3. Reconstruction reports material state that was unavailable, omitted, or not transferable.
4. Stale/ambiguous actor or native identity yields blocked/reconstruction/ownership-loss truth rather than identifier-coincidence attachment.
5. Unreadable/incompatible state is preserved for diagnosis and produces explicit recovery-required/failure truth rather than silent reinitialization.

### User Story 6 - Hand Off Only Canonical Stage Context and Preserve Reviewer Independence (Priority: P1)

A stage actor or reviewer receives the minimum structured state and exact artifacts needed for its role. Transcript history, prior author persuasion/confidence, hidden runtime state, and unrelated secrets are not silently promoted into canonical handoff authority.

**Independent Test**: Build handoff fixtures containing canonical state plus persuasive author prose, stale transcript material, hidden-state claims, unrelated artifacts, and reviewer-targeted conclusions. Verify stage/reviewer handoff is derived from canonical structured state and required exact artifacts, with omissions/unavailable information explicit.

**Acceptance Scenarios**:

1. Stage handoff identifies exact workflow/stage/candidate/artifact context.
2. Handoff does not depend on an unqualified transcript dump as canonical state.
3. Independent reviewer input excludes author confidence/persuasion by default unless explicitly required as source-labelled evidence.
4. Hidden provider/runtime memory that Winds cannot prove is unavailable rather than reconstructed as fact.
5. Handoff minimization never removes required evidence, authority, recovery, or provenance context.

### User Story 7 - Persist Redacted State Without Pretending Evidence Is Complete (Priority: P1)

A developer can inspect durable workflow history without Winds silently persisting protected secrets or presenting redacted/omitted material as complete evidence.

**Independent Test**: Feed deterministic secret-like/protected-data fixtures into stage outputs and decision context before durable write. Verify configured protected classes are redacted/omitted before persistence, explicit loss/completeness markers survive, and affected evidence cannot qualify as complete when required material is missing.

**Acceptance Scenarios**:

1. Protected durable fields are redacted/omitted according to explicit policy before durable persistence.
2. Redaction/omission leaves source-aware markers sufficient to explain lost information.
3. Missing/redacted required evidence cannot silently satisfy a complete-evidence gate.
4. Search/status/history surfaces preserve the same incompleteness truth.
5. Redaction does not rewrite unrelated historical evidence or decision provenance.

### User Story 8 - Explain Status, Resume, and Why-Blocked Without Creating New Authority (Priority: P1)

A developer can inspect a concise operator-facing projection answering current stage, actor, candidate/artifact/evidence binding, blocker/approval/external condition, retry state, and what a resume action would actually do. The projection is derived from canonical state and cannot itself mutate or upgrade authority.

**Independent Test**: Render status/resume/why-blocked projections for active, waiting, blocked, stale, failed, completed, recovery-required, reconstructed, and ownership-lost fixtures; inject forged terminal/agent labels and verify the projection remains canonical and source-labelled.

**Acceptance Scenarios**:

1. Status identifies the exact workflow/stage attempt and its canonical lifecycle state.
2. Resume preview distinguishes live/native resume, Winds reconstruction, reassignment, unavailable continuation, and ownership loss as applicable.
3. Why-blocked identifies the blocker and the explicit approval/external condition/action needed where known.
4. UI/CLI labels cannot convert agent/terminal claims into verification, acceptance, or authority.
5. Inspecting status/resume/why-blocked is non-mutating unless a later accepted task explicitly authorizes one narrow state transition.

## Workflow Safety Edge Cases

- Workflow/stage display names collide or change while IDs remain stable.
- A stage is prepared, then its required artifact changes before start.
- Candidate identity changes after stage evidence/review is gathered.
- An unchanged old artifact is re-presented as if produced by a new attempt.
- A stale actor/runtime has the same display name or reused native identifier as a new process/session.
- Winds restarts after durable state exists but the original runtime cannot be proven live.
- Durable workflow state is truncated, partially written, malformed, from an unknown future schema version, or internally inconsistent.
- A duplicate/replayed completion or decision event arrives after the original was already recorded.
- A retry follows a failure with the same candidate/diff/failure class and no material progress.
- A crash occurs after a non-idempotent side effect but before its completion checkpoint is proven.
- An author-supplied rationale contains persuasive reviewer instructions or forged acceptance language.
- Terminal/agent output contains `PASS`, `VERIFIED`, `ACCEPTED`, decision-shaped JSON, or fake workflow labels.
- Redaction removes material content required to judge evidence completeness.
- A superseded/reverted decision is referenced by a later stage.
- A stage completes procedurally while candidate verification remains stale, missing, or failed.
- Workflow completion is reached while explicit human acceptance/landing has not occurred.

## Functional Requirements

### Canonical workflow and stage identity

- **FR-001**: Winds MUST represent a workflow run with a stable canonical identity distinct from chat/session/runtime/UI identity.
- **FR-002**: Winds MUST represent each stage attempt with a stable canonical identity distinct from stage display name and actor/session identity.
- **FR-003**: Every new stage attempt MUST have a unique run-instance/attempt identity; retry/reconstruction MUST NOT silently reuse a prior attempt identity as new work.
- **FR-004**: Mutable workflow/stage labels MUST remain presentation-only and MUST NOT determine canonical identity or authority.
- **FR-005**: A stage MUST expose an explicit lifecycle state from a deterministic state model covering at minimum prepared, active, waiting approval, waiting external condition, blocked, failed, stale, cancelled, completed, and recovery-required truth where applicable.
- **FR-006**: Legal stage transitions MUST be explicit and deterministic for identical canonical inputs; invalid transitions MUST fail closed rather than being coerced to the requested state.
- **FR-007**: Workflow lifecycle truth MUST be derived from canonical workflow/stage state and MUST NOT be inferred solely from agent/terminal prose or UI presence.
- **FR-008**: `STAGE_COMPLETE`, `WORKFLOW_COMPLETE`, `VERIFICATION_PENDING`, `VERIFIED`, and `HUMAN_ACCEPTED` MUST remain distinct states/claims; no earlier state may imply a later one.
- **FR-009**: Historical attempts MUST remain bound to their original canonical workflow/stage identities even after retry, reconstruction, cancellation, or supersession.

### Artifact baselines, candidate binding, and freshness

- **FR-010**: Preparing a stage MUST establish an explicit baseline for every required upstream artifact/input whose freshness affects stage validity.
- **FR-011**: The baseline MUST carry stable reference/identity information sufficient for later deterministic freshness comparison; the specification does not require a particular hashing/storage implementation.
- **FR-012**: A material change to a required baseline input before the applicable gate MUST make the prepared stage stale/not-applicable until explicitly re-prepared or otherwise resolved by an accepted rule.
- **FR-013**: A stale artifact from an older attempt MUST NOT silently satisfy a newly prepared stage gate merely because its path/name/content presentation is similar.
- **FR-014**: Candidate-bound stage evidence/review MUST remain attached to the exact candidate identity for which it was gathered.
- **FR-015**: Candidate movement MUST invalidate applicability of earlier candidate-bound stage evidence/review without deleting historical evidence.
- **FR-016**: Freshness status MUST be source-labelled and MUST NOT be promoted by terminal/agent claims.
- **FR-017**: The operator-facing state MUST explain which required baseline is stale/missing/ambiguous when that condition blocks progress.

### Actor binding, continuation, and reconstruction truth

- **FR-018**: An active stage MUST bind to explicit Winds actor and stage run-instance identity; Winds session/runtime/native identity MUST also be bound where applicable to the actor kind so stale replacement or ambiguous continuation can be detected.
- **FR-019**: Actor binding MUST NOT be inferred from display name, PID reuse, native-session identifier coincidence, or transcript continuity alone.
- **FR-020**: `RESUMED` MUST require the applicable existing accepted proof for an exact runtime/native-session mapping and supported resume path revalidated at use time.
- **FR-021**: A new actor/runtime initialized from canonical Winds workflow state MUST be classified `RECONSTRUCTED`, not `RESUMED`.
- **FR-022**: Reconstruction MUST identify material context that was reconstructed/derived, omitted, unavailable, or no longer transferable.
- **FR-023**: Stale/missing/ambiguous actor/runtime mapping MUST fail closed to blocked, reconstruction-required, ownership-lost, or explicit unavailable truth as applicable; it MUST NOT produce a false live/resumed claim.
- **FR-024**: Reassignment to another actor/run instance MUST create an explicit new binding/attempt relationship rather than silently changing the historical actor of a prior attempt.
- **FR-025**: Restarting the Winds process MUST NOT by itself prove live external/native runtime ownership.
- **FR-026**: Spec 008 MUST reuse rather than weaken accepted Spec 006 continuity semantics for `LIVE`, `RESUMED`, `RECONSTRUCTED`, `OWNERSHIP_LOST`, and `STOPPED` where those runtime states apply.

### Retry, no-progress, and ambiguous side effects

- **FR-027**: Every retryable stage behavior MUST operate under an explicit finite retry/no-progress policy selected by a later Plan/Tasks unit; unbounded retry is forbidden.
- **FR-028**: Every stage retry MUST create/preserve distinct attempt history and MUST NOT rewrite the failure that caused the retry.
- **FR-029**: The future Plan/Tasks MUST define deterministic no-progress signals sufficient to cover repeated candidate/artifact signatures, repeated failure classes, repeated checkpoints without material progress, and budget consumption where applicable.
- **FR-030**: Reaching the configured no-progress or retry budget MUST pause/block/fail explicitly rather than silently resetting the budget or continuing indefinitely.
- **FR-031**: Retry state MUST expose the reason, attempt/budget state, and last material-progress basis sufficient for operator inspection.
- **FR-032**: If a prior action may have caused a non-idempotent side effect but completion is ambiguous, Winds MUST NOT silently repeat it unless idempotency is proven or an explicit accepted semantics/reconciliation rule authorizes the risk.
- **FR-033**: Duplicate/replayed transition or completion input MUST NOT create duplicate canonical completion/decision truth or silently advance authority.
- **FR-034**: Retry success MUST NOT erase or relabel earlier failed/reverted attempts as successful evidence.

### Append-only decision ledger

- **FR-035**: Every canonical decision record MUST have stable identity and, where applicable, explicit workflow/stage/workstream binding.
- **FR-036**: A canonical decision record MUST identify its source/actor/authority class, decision type, rationale/result, creation identity/time, and exact candidate/evidence references where applicable.
- **FR-037**: Decision records MUST be append-only at the semantic level; changing a decision MUST create a new record/relationship rather than deleting or rewriting the prior decision into a different historical claim.
- **FR-038**: Supersession MUST preserve explicit predecessor/successor lineage and the superseded decision MUST remain auditable.
- **FR-039**: Rejected, failed, reverted, stale, and superseded decisions/attempts MUST remain historically inspectable.
- **FR-040**: Decision source truth MUST preserve `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED` where applicable.
- **FR-041**: An agent/terminal/user-interface string claiming approval MUST NOT create a human decision record without the existing accepted human-decision path.
- **FR-042**: Candidate/evidence movement MUST make affected decision references stale/not-applicable as required without deleting the record.
- **FR-043**: Append-only decision history MUST NOT itself grant mutation, execution, verification, merge, or landing authority.

### Structured handoff and reviewer independence

- **FR-044**: Stage handoff MUST be assembled from canonical structured workflow/stage state plus exact required artifacts/evidence, not from an unqualified transcript as canonical authority.
- **FR-045**: Handoff MUST identify exact workflow/stage/attempt identity, actor role/binding as applicable, candidate/artifact baseline, authority ceiling, and required evidence/blocker context needed for the receiving role.
- **FR-046**: Context minimization MUST NOT remove required evidence completeness, provenance, authority, recovery, or safety facts.
- **FR-047**: Material unavailable/omitted context MUST be explicitly represented rather than silently reconstructed.
- **FR-048**: Independent reviewer handoff MUST exclude author confidence/persuasion and proposed verdict language by default unless a later accepted rule requires that material as explicitly source-labelled evidence.
- **FR-049**: Hidden provider/runtime reasoning or inaccessible native state MUST NOT be reconstructed as canonical fact.
- **FR-050**: Spec 008 MUST NOT treat transcript continuity as workflow continuity proof.

### Durable-state validation, corruption recovery, and redaction

- **FR-051**: Canonical workflow/decision state intended to survive process restart MUST have an explicit versioned logical schema/format contract selected by later Plan/Tasks without this specification selecting a storage engine.
- **FR-052**: Durable state MUST be validated before use for required identity, version compatibility, internal consistency, and references necessary to the attempted operation.
- **FR-053**: Malformed, partial, internally inconsistent, or unsupported-version state MUST fail closed to explicit recovery-required/failure truth rather than being silently treated as a new clean workflow.
- **FR-054**: Unreadable/corrupt state MUST be preserved or quarantined for bounded diagnosis/recovery as later Plan/Tasks define; recovery MUST NOT destroy the only available historical evidence by default.
- **FR-055**: Migration/recovery MUST preserve stable canonical identity, decision history, stale/failed attempt history, and evidence/candidate provenance or explicitly report any proven loss.
- **FR-056**: Protected data classes selected by later Plan/Tasks MUST be redacted/omitted before durable persistence when policy requires it.
- **FR-057**: Every material redaction/omission MUST leave an explicit source-aware loss/completeness marker sufficient to prevent the remaining material from masquerading as complete evidence.
- **FR-058**: Missing/redacted required evidence MUST NOT satisfy a complete-evidence, verification, or acceptance gate.
- **FR-059**: Redaction MUST NOT mutate unrelated canonical evidence, approvals, decision source, or candidate identity.
- **FR-060**: Spec 008 MUST NOT introduce semantic/vector/RAG memory or treat durable workflow history as a general learned memory engine.

### Operator-facing workflow truth

- **FR-061**: Winds MUST provide a deterministic operator-facing projection of current workflow/stage identity and lifecycle truth suitable for later CLI/TUI presentation.
- **FR-062**: The projection MUST identify current actor/binding truth, candidate/artifact/evidence applicability, blocker/approval/external-condition state, retry/no-progress state, and continuation/reconstruction semantics where applicable.
- **FR-063**: A resume preview MUST state whether the action would use an already-proven live/native resume path, reconstruct from Winds state, reassign, fail/block, or operate with ownership unavailable as applicable.
- **FR-064**: A why-blocked projection MUST identify the known blocker and explicit approval/external condition/recovery action needed where the canonical state can prove it.
- **FR-065**: Operator-facing projections MUST distinguish source-labelled Agent-reported material from Winds-observed/policy and human-decided state.
- **FR-066**: Inspecting status/resume/why-blocked MUST be non-mutating unless a later accepted task explicitly authorizes one narrow state transition.
- **FR-067**: UI/CLI presentation MUST NOT be a second workflow authority; canonical state remains authoritative.

### Scope, platform, and governance boundaries

- **FR-068**: Spec 008 MUST preserve the current one-process architecture and MUST NOT add a persistent owner, daemon, server, socket, HTTP/SSE/WebSocket endpoint, new IPC/control protocol, or new child-process/multi-process owner or coordinator for workflow state. Existing previously accepted child-process execution surfaces are not expanded by this requirement.
- **FR-069**: Spec 008 MUST NOT add remote execution, remote/mobile/team continuation, cloud orchestration, or service control-plane behavior.
- **FR-070**: Spec 008 MUST NOT add provider/model APIs, Model Mesh, automatic provider routing, credential brokerage, or new provider authentication/execution authority.
- **FR-071**: Spec 008 MUST NOT add browser automation/profile/CDP runtime, browser credentials, screenshots as verification authority, or Browser Twin.
- **FR-072**: Spec 008 MUST NOT add MCP runtime, ACP dependency expansion, A2A, generic workflow/plugin/runtime frameworks, integration SDK, or marketplace architecture.
- **FR-073**: Spec 008 MUST NOT add verified-learning activation, skill optimization/promotion, experiment-plane activation, protected-holdout execution, model training/fine-tuning/RL, learned routing, or automatic policy mutation.
- **FR-074**: Spec 008 MUST NOT automatically select a candidate winner, merge, rebase, cherry-pick, push, create a PR, or land changes.
- **FR-075**: LoopForge, SkillHone, and other research sources remain design references only; source discovery/research MUST NOT become donor code/runtime/dependency admission without a separate exact-source provenance decision.
- **FR-076**: Every new dependency, persistence technology, schema engine, migration mechanism, serialization crate, or runtime abstraction remains a Plan/Tasks decision requiring exact need/version/license/MSRV/platform/provenance/security/YAGNI review before implementation may rely on it.
- **FR-077**: Platform claims MUST be limited to directly exercised domains; no native Windows, WSL2, Linux, or macOS behavior may inherit proof solely from another platform.
- **FR-078**: Spec 008 MUST preserve all applicable accepted Spec 003/006/007 Git, evidence, authority, recovery, platform, privacy, terminal-lifecycle, and human-landing invariants.

## Success Criteria

- **SC-001**: Deterministic workflow fixtures prove stable `WorkflowRun` and unique stage-attempt identity across label changes, retries, and Winds process restart with zero identity collision or silent attempt reuse.
- **SC-002**: The defined stage-state fixtures accept every specified legal transition and reject every specified illegal transition without silent coercion or authority upgrade.
- **SC-003**: Changed required artifact/candidate fixtures make prepared state/evidence stale or not-applicable while preserving historical records; zero stale artifact silently satisfies a newer stage gate.
- **SC-004**: Exact native/runtime resume, Winds reconstruction, stale mapping, unavailable mapping, and ownership-loss fixtures produce truthful distinct continuation outcomes with zero false `RESUMED` claims.
- **SC-005**: Retry/no-progress fixtures terminate within the configured finite policy, preserve every failed attempt, and produce explicit blocked/failed truth at exhaustion rather than unbounded retry.
- **SC-006**: Ambiguous non-idempotent-effect fixtures produce explicit reconciliation/block truth unless an accepted idempotency/at-least-once rule applies; no silent duplicate retry is performed.
- **SC-007**: Duplicate/replayed transition/completion fixtures create zero duplicate canonical completion/decision truth and zero authority escalation.
- **SC-008**: Decision-ledger fixtures preserve accepted/rejected/reverted/superseded history append-only, with exact source/authority/candidate/evidence lineage and zero mutation of prior historical meaning.
- **SC-009**: Reviewer-handoff fixtures omit author persuasion/confidence by default while retaining required exact candidate/artifact/evidence/provenance/authority facts.
- **SC-010**: Restart/reconstruction fixtures preserve canonical workflow/decision state while making unavailable runtime/native state explicit rather than claiming it was restored.
- **SC-011**: Malformed/partial/incompatible/corrupt durable-state fixtures fail closed and preserve or quarantine the unreadable state for recovery/diagnosis; zero case silently initializes a new successful workflow over corrupt history.
- **SC-012**: Redaction fixtures prove protected durable content is removed/omitted before persistence according to the accepted policy while explicit loss/completeness markers remain; missing required evidence never qualifies as complete.
- **SC-013**: Status/resume/why-blocked projections deterministically explain active stage, actor, freshness/applicability, blocker, retry state, and continuation semantics from canonical state for every accepted lifecycle fixture.
- **SC-014**: Forged terminal/agent `PASS`/`VERIFIED`/`ACCEPTED`/decision/workflow labels produce zero promotion into canonical verification, human decision, stage authority, or workflow authority.
- **SC-015**: Stage/workflow completion fixtures never imply candidate verification or human acceptance; verification/acceptance require their existing accepted paths and exact-candidate applicability.
- **SC-016**: Existing repository `quality` and applicable Spec 003/006/007 regression/platform/security gates remain green on the final exact implementation candidate.
- **SC-017**: Correctness/safety, Ponytail/YAGNI, and fresh independent review reach the final exact implementation candidate with zero unresolved material findings.
- **SC-018**: Final reconciliation truthfully proves no daemon/IPC, remote execution, provider mesh, browser runtime, MCP/ACP expansion, generic plugin/workflow engine, learning subsystem, semantic/vector memory, donor code/dependency admission, automatic winner, or automatic landing path was introduced.
- **SC-019**: Every claimed native Windows/WSL2/Linux/macOS behavior is backed by direct applicable evidence for that domain; unexercised behavior remains explicitly unclaimed.
- **SC-020**: Historical failed/rejected/reverted/stale evidence remains inspectable after retries, reconstruction, supersession, and final workflow completion; zero historical failure is rewritten as current success.

## Explicit Non-Goals

Spec 008 does not authorize:

- persistent background ownership, daemon/server/socket/IPC/control API, or detach/reattach across Winds process exit;
- remote execution, SSH/mobile/team continuation, cloud control plane, or service orchestration;
- provider/model APIs, Model Mesh, provider credential brokerage, silent/automatic routing, or new provider execution authority;
- browser automation, browser profile/credential management, CDP, screenshots as verification evidence, or Browser Twin;
- MCP runtime, ACP dependency expansion, A2A, generic workflow/runtime/plugin framework, integration SDK, marketplace, or third-party executable extension host;
- verified-learning activation, skill mutation/optimization/promotion, protected-holdout execution, experiment-plane activation, training/fine-tuning/RL, or automatic policy mutation;
- vector/embedding/RAG memory, semantic memory engine, or transcript-as-canonical-memory architecture;
- automatic candidate comparison/winner selection, automatic Git mutation/PR creation, or automatic landing;
- donor code/runtime/dependency import from LoopForge, SkillHone, or another research source without a separately accepted provenance/admission decision;
- a storage engine, database, schema framework, serialization crate, migration library, provider SDK, or runtime architecture choice at specification stage;
- weakening or replacing accepted Spec 003/006/007 exact-candidate, evidence, authority, reviewer-independence, terminal-lifecycle, recovery, platform, privacy, or human-landing truth.

## Assumptions

- Existing canonical workspace/workstream/session/runtime/evidence identities remain starting truth and are referenced rather than replaced.
- Existing Spec 006 continuation vocabulary and proof requirements remain authoritative where runtime/native session continuity applies.
- Existing verification/human-decision/landing paths remain authoritative; Spec 008 supplies workflow state around them but does not replace them.
- A later Plan may choose the smallest durable local persistence representation only after exact need/provenance/dependency/security/YAGNI review; this specification intentionally does not choose SQLite or any other engine.
- The Plan may define bounded default retry/no-progress budgets and concrete schema/versioning mechanics but may not authorize unbounded retries or weaken the fail-closed/redaction/freshness invariants above.
- The Plan may tighten workflow performance/boundedness budgets based on measured current behavior; any material weakening of the semantic invariants requires an accepted spec amendment.

## Governance / Downstream Authorization

Canonical acceptance of this specification authorizes **Spec 008 Plan creation only**.

It does not authorize Tasks, implementation, dependencies, migrations, storage-engine selection, source changes, provider/browser execution, daemon/IPC, remote execution, learning, donor code admission, automatic landing, or later specifications.

Only after this specification lands and is post-merge verified may repository truth state:

```text
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN_AUTHORIZED=YES
SPEC_008_TASKS_AUTHORIZED=NO
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```
