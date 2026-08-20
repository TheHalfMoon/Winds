# Feature Specification: Agentic Terminal & Local Delegation Control Plane

**Feature Branch**: `spec/006-agentic-terminal-local-delegation-control-plane`

**Created**: 2026-08-21

**Canonical Base**: `ef933a6a1f4d2f926f9f2e2f89ebb6972e4785f1`

**Status**: Authorized for specification only; planning, tasks, dependencies, migrations, runtime implementation, prompt execution, and model/provider calls are NOT authorized by this file alone

**Input**: Extend Winds from a verification-native workspace into a local agentic development control plane where a human can keep many named sessions per workspace, continue one canonical task across changing agent runtimes, discover Codex/Claude capabilities without silently trusting or installing them, delegate one bounded task under explicit authority ceilings, preserve truthful context-transfer provenance, and bind independent review plus deterministic verification to the exact current candidate without automatic landing.

## Product North Star

Winds should let a developer move between terminal sessions and coding-agent runtimes without losing the identity, authority, or evidence of the work.

For every meaningful agentic action Winds should be able to answer:

1. **What canonical work is this?** Stable workspace, workstream/task, and Winds session identity independent of display names or vendor-native session IDs.
2. **Which runtime is acting?** Exact local runtime/harness identity and capability observations, kept separate from model/provider identity.
3. **What context actually transferred?** Canonical task facts, evidence, files, constraints, and imported/native history with explicit provenance and explicit unavailable/lost state.
4. **What authority did this actor actually have?** Direct execution authority, delegation ceiling, team/human ceiling, and actual enforcement quality.
5. **What happened versus what was merely claimed?** `AGENT_REPORTED`, `WINDS_OBSERVED`, and `HUMAN_DECIDED` facts remain distinct.
6. **Which exact candidate do the checks and review apply to?** Changed candidate identity invalidates stale deterministic evidence and stale independent review.
7. **What did the human decide?** Winds may organize and verify work but must not silently choose a winner or land changes.

The differentiated loop is not “run many agents.” It is **canonical continuity + explicit authority + exact-candidate evidence across heterogeneous local runtimes**.

## Frozen Product Invariants

The following invariants are requirements, not slogans:

```text
RUNTIME != MODEL
WORKSPACE_HAS_MANY_NAMED_SESSIONS
NEW_SESSION != NEW_TASK
NEW_AGENT != NEW_TASK
NATIVE_RESUME != CANONICAL_TASK_CONTINUITY
LIVE_PROCESS_OWNERSHIP != NATIVE_RUNTIME_RESUME
IMPORTED_HISTORY != CANONICAL_EVIDENCE
MODEL_CONTEXT_MAY_COMPACT; CANONICAL_WORK_EVIDENCE_TRUTH_MUST_NOT
DISCOVERY != TRUST
WORKTREE != SANDBOX
ACP_ADDITIONAL_ROOTS != SANDBOX
AGENT_CLAIM != WINDS_OBSERVATION != HUMAN_DECISION
IDLE != DONE != VERIFIED != ACCEPTED
PLANNER_DIRECT_AUTHORITY != PLANNER_DELEGATION_CEILING
CHILD_AUTHORITY <= APPROVED_DELEGATION_TEAM_HUMAN_CEILINGS
CHANGED_CANDIDATE_INVALIDATES_STALE_EVIDENCE_AND_REVIEW
NO_AUTOMATIC_WINNER
NO_AUTOMATIC_AUTHORITY_ESCALATION
NO_SILENT_LANDING
VERIFY_THE_EXACT_CANDIDATE
```

## User Scenarios & Testing

### User Story 1 - Keep Many Named Sessions Under One Stable Workspace (Priority: P1)

A developer can organize one repository/workspace into many independently named Winds sessions without equating a new session with a new task. Display names may change, but stable identity, history, evidence links, and task relationships remain intact.

Examples include a long-lived planner session, a focused bug-fix session, a review session, and a new terminal session that continues the same canonical workstream.

**Why this priority**: If session identity depends on a title, path, runtime, or provider-native ID, every later continuity/delegation feature can silently lose or fork the work.

**Independent Test**: Create one fixture workspace with multiple Winds sessions attached to one canonical task/workstream. Rename workspace/session display names, create a new session that continues the same task, and verify stable identifiers and evidence/task links are unchanged. Verify a newly created unrelated task receives a distinct canonical identity even if its title is identical.

**Acceptance Scenarios**:

1. **Given** an existing workspace, **When** multiple Winds sessions are created, **Then** each has a stable Winds identity and a user-editable display name, and the workspace can list/select them independently.
2. **Given** a session rename, **When** history/evidence/task relationships are inspected afterward, **Then** rename changes UX text only and does not create a new identity or orphan prior links.
3. **Given** the user chooses “new session” for the current task, **When** the session is created, **Then** Winds preserves the canonical task/workstream identity rather than silently creating a new task.
4. **Given** the user explicitly chooses “new task,” **When** a session is created for it, **Then** Winds creates a distinct canonical work identity even if workspace/runtime/display text overlaps.
5. **Given** multiple plausible sessions, **When** a continuation request is ambiguous, **Then** Winds returns/selects from explicit candidates rather than guessing from recency alone.

### User Story 2 - Continue, Fork, or Start New Work Without Memorizing Native Session IDs (Priority: P1)

A developer can explicitly continue canonical work, fork from known state, or begin a new task without needing to remember a provider/runtime-native session identifier.

Winds may use a native resume primitive when it can prove the mapping, but routine continuity belongs to Winds and survives the absence of native resume.

**Why this priority**: Vendor-native resume is useful but cannot be the canonical work identity if Winds is to support heterogeneous runtimes and honest handoff.

**Independent Test**: Exercise three fixture paths: native resume available and exact, native resume unavailable, and ambiguous/stale native mapping. Verify Winds reports the correct continuity state, never invents a successful native resume, and can reconstruct a new runtime session from canonical state when policy allows.

**Acceptance Scenarios**:

1. **Given** a Winds session with a valid exact native-session mapping and a runtime that proves resume support, **When** continue is requested, **Then** Winds may use native resume and labels the result `RESUMED`.
2. **Given** no usable native-session mapping, **When** continue is requested, **Then** Winds may construct a new runtime session from the canonical Winds context and labels the result `RECONSTRUCTED`; it MUST NOT report `RESUMED`.
3. **Given** a currently owned connected process/session in the same Winds lifetime, **When** continuation occurs through that exact live owner, **Then** the relationship may be labelled `LIVE` and MUST remain distinct from provider-native resume.
4. **Given** persisted process/native identity that can no longer be proven, **When** continuation is inspected, **Then** Winds reports `OWNERSHIP_LOST` or an explicit unavailable mapping rather than attaching to a replacement process/session by identifier coincidence.
5. **Given** the user chooses fork, **When** a new session is created, **Then** Winds records its origin/reference point while giving it a distinct Winds session identity.

### User Story 3 - Discover Codex and Claude Runtime Capabilities Without Turning Discovery Into Trust (Priority: P1)

A developer can see whether supported local authoring runtimes are present and what Winds can actually observe about their version, structured-control support, native continuation support, authentication readiness when safely observable, and execution domain.

The first formal targets are Codex and Claude Code. Discovery must not auto-install, auto-update, accept terms, duplicate credentials, start an agent, send a prompt, or claim access that Winds did not observe.

**Why this priority**: Structured control is only safe and useful when Winds distinguishes runtime identity, model identity, declared capabilities, local observations, and unavailable facts.

**Independent Test**: Use fake/local fixture executables representing absent, present, unsupported-version, capability-declared, capability-unobservable, and changed-between-discovery-and-use runtimes. Verify discovery performs only authorized non-agentic/version/capability observations, records provenance, and requires revalidation before use.

**Acceptance Scenarios**:

1. **Given** no supported runtime installed, **When** discovery runs, **Then** Winds reports unavailable state and does not install or execute an agent workflow.
2. **Given** Codex or Claude is installed, **When** Winds observes exact executable/version/capability information through an accepted safe path, **Then** those facts are source-labelled `WINDS_LOCALLY_OBSERVED` and tied to the exact observed runtime identity.
3. **Given** a runtime/vendor declares support for a capability Winds has not exercised or independently observed, **When** the capability is displayed, **Then** its source remains `VENDOR_DECLARED` or `CATALOG_DECLARED`, not `WINDS_LOCALLY_OBSERVED`.
4. **Given** a runtime exposes model/provider choices, **When** Winds represents them, **Then** runtime/harness identity remains distinct from model/provider identity.
5. **Given** a runtime binary/version changes after discovery, **When** a structured session is about to start/resume, **Then** Winds revalidates launch-significant identity or fails closed; stale discovery MUST NOT silently authorize use.

### User Story 4 - Transfer Canonical Context Truthfully Across Sessions and Runtimes (Priority: P1)

A developer can continue work in a new session or different supported runtime with a deterministic Winds context capsule that carries the canonical objective, constraints, decisions, current work state, exact workspace/candidate facts, and applicable evidence without claiming to transfer inaccessible provider-private state.

**Why this priority**: Cross-runtime continuity is valuable only if the user can tell what was transferred, what was reconstructed, and what was unavailable.

**Independent Test**: Build a fixture canonical task with source-labelled facts, a prior runtime/native history artifact, exact candidate/evidence links, and intentionally unavailable vendor-private state. Produce a handoff to a second fixture runtime and verify the transfer report is deterministic, provenance-preserving, bounded, and explicit about omissions.

**Acceptance Scenarios**:

1. **Given** canonical Winds task state, **When** a context capsule is prepared, **Then** it includes stable task/workspace/session references, current objective/constraints, explicit human decisions, exact candidate/evidence references, and selected relevant work facts with source labels.
2. **Given** imported provider/runtime history, **When** it is included, **Then** it remains imported/source-labelled and cannot overwrite `WINDS_OBSERVED` or `HUMAN_DECIDED` canonical facts by implication.
3. **Given** state that cannot be exported or proven, **When** a handoff occurs, **Then** the transfer report identifies it as unavailable/not transferred rather than claiming full context preservation.
4. **Given** context compaction/budget limits, **When** a smaller model-facing capsule is produced, **Then** canonical Winds work/evidence state remains unchanged and the compact view is traceable to its source state.
5. **Given** a cross-runtime handoff, **When** the receiving session starts, **Then** Winds records the source and destination runtime/session identities plus the transfer result without claiming vendor-private memory transfer.

### User Story 5 - Approve One Bounded Planner to Worker Delegation (Priority: P1)

A human can approve a single Planner -> Worker contract in which the Planner's own direct execution authority is distinct from the authority it may delegate, the Worker receives only bounded task/context/resource authority, and no actor can self-expand those ceilings.

The Worker can return a structured result and may later continue the same resumable Worker session when its exact identity remains valid.

**Why this priority**: Delegation without explicit authority semantics turns a useful control plane into an uncontrolled swarm. One bounded relationship is sufficient to prove the model before fleet complexity.

**Independent Test**: Use a fixture Planner/Worker adapter pair and a protected Winds policy fixture. Exercise allow, ask, deny, child-over-ceiling, Planner-self-escalation, changed-approved-content, prompt-injected “grant,” and unavailable enforcement paths. No real model/provider call is required to prove the policy semantics.

**Acceptance Scenarios**:

1. **Given** a Planner proposal for one Worker task, **When** the human reviews it, **Then** Winds shows the bounded task, workspace/root, relevant context, budget where applicable, Planner direct authority, delegation ceiling, Worker authority, and enforcement quality before approval.
2. **Given** an approved delegation contract, **When** the Worker requests an operation outside its child/delegation/team/human ceiling, **Then** Winds denies or re-prompts according to policy and MUST NOT infer permission from the Planner's prose.
3. **Given** the Planner has a delegation ceiling broader than its own direct execution authority, **When** it delegates within the approved ceiling, **Then** the Planner's own direct authority does not expand as a side effect.
4. **Given** policy `deny`, **When** transient request/prose/tool output suggests allow, **Then** deny remains authoritative until the protected policy is explicitly changed through the authorized human path.
5. **Given** approved content/resource identity changes materially before execution, **When** the delegated operation is attempted, **Then** stale content-bound approval is invalidated and reapproval is required where policy requires it.
6. **Given** a third-party runtime can access resources outside Winds mediation, **When** Winds cannot enforce a requested ceiling, **Then** the UI/evidence reports the actual weaker enforcement quality instead of claiming `WINDS_ENFORCED`.
7. **Given** a completed Worker response, **When** it returns to the Planner, **Then** agent claims remain `AGENT_REPORTED` until independently observed or human-decided; Worker completion MUST NOT imply task verification/acceptance.

### User Story 6 - Bind Work, Review, and Verification to the Exact Candidate (Priority: P1)

A developer can bind a candidate produced through an agentic session to exact Git identity, obtain a fresh independent reviewer pass with independence-preserving context, run deterministic repository-native verification against the same candidate, and see stale review/evidence invalidated automatically when candidate identity changes.

**Why this priority**: Winds' existing verification authority is the differentiator that must survive the addition of agentic sessions and delegation.

**Independent Test**: Create candidate A, attach deterministic check evidence and independent review, then change to candidate B. Verify A's review/check results remain historical but are not applicable to B. Verify the independent review context excludes builder persuasion/confidence and includes exact candidate identity/acceptance criteria.

**Acceptance Scenarios**:

1. **Given** an agent-produced candidate, **When** it enters the acceptance path, **Then** Winds binds review/check evidence to exact candidate identity rather than branch/display name alone.
2. **Given** a reviewer session, **When** review-safe context is prepared, **Then** it includes exact candidate/diff, acceptance criteria, relevant canonical constraints/evidence, and excludes builder confidence/persuasion as authoritative input under the independent policy.
3. **Given** deterministic gates and reviewer evidence for candidate A, **When** candidate identity changes to B, **Then** Winds marks A evidence/review stale/not-applicable for B until B is freshly checked/reviewed.
4. **Given** an agent says the candidate is done or verified, **When** no accepted Winds-observed gates exist, **Then** Winds does not elevate the claim to verified/accepted.
5. **Given** all accepted gates pass, **When** the candidate is presented, **Then** Winds still requires the explicit human landing decision and does not automatically choose/merge/push/open a PR under this specification.

### User Story 7 - Find the Right Session and Context Without Memorizing Paths (Priority: P2)

A developer can locate a workspace/session and select files, directories, recent/changed items, and symbols where intelligence is available without memorizing full paths or provider-native IDs.

This UX supports the P1 continuity loop but must not introduce an opaque autonomous retrieval authority.

**Independent Test**: Populate a fixture workspace with similarly named sessions/files/directories, ambiguous matches, Unicode paths, changed/recent items, and optional symbol metadata. Verify deterministic candidate lists, explicit disambiguation, and provenance for semantic/symbol-derived results.

**Acceptance Scenarios**:

1. **Given** many named sessions, **When** the user searches by partial/fuzzy text, **Then** Winds presents deterministic identifiable candidates rather than silently continuing the top heuristic match when ambiguity is material.
2. **Given** file/directory selection, **When** a picker/search is used, **Then** selection resolves to exact canonical path identity before context/execution use.
3. **Given** symbol intelligence is unavailable, **When** symbol selection is requested, **Then** Winds reports unavailable capability rather than inventing semantic confidence.
4. **Given** a selected context item outside an approved authority/root boundary, **When** it would be sent to an actor, **Then** policy/authority rules still apply; picker visibility is not access authorization.

## Edge Cases

- Workspace/session display names collide, contain Unicode, differ only by case, or are renamed repeatedly.
- A runtime-native session ID is reused, disappears, belongs to a different executable/version/account/domain, or becomes ambiguous.
- The runtime binary is replaced between discovery and session start.
- A runtime advertises structured-control or resume support but fails when exercised.
- Runtime authentication state cannot be safely observed without prompting or network access.
- Provider/vendor history contains facts that conflict with canonical Winds observations or human decisions.
- Imported history is enormous, corrupt, malicious, duplicated, or contains prompt-injection/tool-like text.
- Context compaction removes a constraint or evidence reference from the model-facing view; canonical state must remain intact and omissions visible where material.
- A Planner requests a Worker with authority broader than its delegation ceiling.
- A Worker requests a path/capability visible to the runtime but denied by Winds policy.
- A runtime bypasses Winds mediation because it already has direct host credentials/filesystem/network access.
- Approved file/task content changes between approval and execution.
- A policy file is modified by the governed worktree/agent rather than the protected human policy plane.
- A Planner or tool output contains text that claims to grant itself more authority.
- Two sessions edit the same worktree when the future task requires isolation; no isolation must be inferred from naming alone.
- Candidate branch name stays constant while its commit/tree changes.
- Review/check evidence finishes after the candidate moved.
- Reviewer context accidentally includes builder confidence or a “please approve” summary.
- Session status is idle but process/task/evidence state is unresolved.
- Winds process exits; current Spec 006 must not claim a daemon preserved process ownership.
- Windows, WSL, Linux, or macOS path/runtime-domain facts differ; claims must remain platform-evidence-specific.
- ACP additional roots include paths outside the repository; they do not become sandboxed or trusted by declaration.

## Requirements

### Canonical Workspace, Work, and Session Identity

- **FR-001**: Winds MUST preserve stable workspace identity separately from user-editable workspace display name.
- **FR-002**: Winds MUST support multiple stable Winds session identities under one workspace.
- **FR-003**: Winds MUST preserve a canonical workstream/task identity separately from Winds session identity, runtime-native session identity, and display names.
- **FR-004**: Creating a new Winds session MUST NOT implicitly create a new canonical task/workstream.
- **FR-005**: Changing runtime or agent MUST NOT implicitly create a new canonical task/workstream.
- **FR-006**: Session/workspace rename MUST NOT invalidate or rewrite historical evidence, task relationships, native-session provenance, or exact candidate links.
- **FR-007**: Winds MUST expose explicit `continue`, `fork`, and `new task` semantics wherever implicit interpretation could change canonical work identity.
- **FR-008**: Materially ambiguous continuation MUST fail into explicit candidate selection rather than silently choosing by recency or heuristic score.

### Continuity and Native Session Truth

- **FR-009**: Winds MUST distinguish at least `LIVE`, `RESUMED`, `RECONSTRUCTED`, `OWNERSHIP_LOST`, and `STOPPED` continuity/lifecycle states where applicable; it MUST NOT collapse these into an unqualified restored/resumed boolean.
- **FR-010**: `LIVE` MUST require exact currently proven Winds-owned/connected session identity in the current ownership domain.
- **FR-011**: `RESUMED` MUST require an exact runtime/native-session mapping plus a runtime-supported resume path whose applicable identity is revalidated at use time.
- **FR-012**: `RECONSTRUCTED` MUST mean a new runtime/native session was initialized from canonical Winds state; it MUST NOT imply inaccessible native/provider memory was restored.
- **FR-013**: Persisted PID/native-session identifiers alone MUST NOT prove live ownership or valid resume when identity may have changed/reused.
- **FR-014**: Canonical task continuity MUST remain possible without a native runtime session ID when sufficient canonical Winds state exists and policy permits reconstruction.
- **FR-015**: Winds MUST report what state transferred and what state was unavailable/not transferred for cross-session/runtime continuation.

### Runtime, Model, and Capability Discovery

- **FR-016**: Winds MUST represent runtime/harness identity separately from model/provider identity.
- **FR-017**: The first formal supported agentic runtime targets MUST be Codex and Claude Code; adding other runtimes requires later task-level evidence and MUST NOT force a generic plugin platform into the first implementation.
- **FR-018**: Runtime discovery MUST NOT automatically install, update, authenticate, accept terms, execute an agent task, send a prompt, or call a model/provider merely to populate discovery state.
- **FR-019**: Capability state MUST retain provenance at least across `CATALOG_DECLARED`, `VENDOR_DECLARED`, `WINDS_LOCALLY_OBSERVED`, and explicit unknown/unavailable state.
- **FR-020**: Launch-significant runtime identity/capability observations MUST be revalidated before structured start/resume when stale replacement could change semantics.
- **FR-021**: Winds MUST NOT represent authentication readiness as observed when determining it would require unauthorized prompting, credential access, terms acceptance, or model/provider calls.
- **FR-022**: Winds SHOULD prefer a pinned structured ACP/vendor-native control path over terminal scraping when the required stable capability exists and passes the later dependency/transport landing gates.

### ACP and External Protocol Boundary

- **FR-023**: Formal Spec 006 planning MUST target stable ACP wire protocol `1`, stable schema `schema-v1.20.0` at commit `5e89c71497fe07dd4ae633c181a17224f4a8956d`, and official Rust SDK `2.0.0` at commit `ce023279824149008659dd8f4b8b70266a7e8210`, subject to the separate exact dependency landing audit before any crate is added.
- **FR-024**: ACP draft protocol v2 and `unstable_protocol_v2` MUST remain disabled in the first implementation slice unless a later explicit reviewed task changes that decision.
- **FR-025**: Unstable MCP-over-ACP and MCP runtime behavior are NOT authorized by this specification's first slice.
- **FR-026**: ACP workspace/additional-root declarations MUST be treated as scope/capability declarations, not OS sandbox proof or automatic Winds authority grants.
- **FR-027**: ACP permission/elicitation messages MUST be treated as protocol requests/events that remain subject to Winds/human authority policy; protocol transport MUST NOT self-grant authority.
- **FR-028**: This specification MUST NOT require a public Winds runtime protocol, network listener, HTTP/SSE/WebSocket control plane, or remote execution transport.

### Canonical Context and Provenance

- **FR-029**: Winds MUST be able to prepare a bounded canonical context capsule for continuation/delegation that identifies canonical workspace, work/task, source session, objective, current constraints/decisions, selected work state, and applicable candidate/evidence references.
- **FR-030**: Every safety/evidence-relevant fact in a context capsule MUST preserve its authority/source classification rather than becoming trustworthy solely because it appears in the capsule.
- **FR-031**: Imported runtime/provider history MUST retain provenance and MUST NOT silently overwrite canonical `WINDS_OBSERVED` or `HUMAN_DECIDED` state.
- **FR-032**: Model-facing context compaction MUST NOT mutate or delete canonical Winds work/evidence truth.
- **FR-033**: A transfer report MUST identify material context that was transferred, reconstructed/derived, omitted by policy/budget, or unavailable.
- **FR-034**: Winds MUST NOT claim transfer of provider-private hidden state, inaccessible model context, or private reasoning it cannot observe/export.
- **FR-035**: Tool output, imported history, repository text, and model prose that contains instructions MUST remain data unless an authorized policy/action path explicitly promotes an action; prompt-injected text MUST NOT directly grant authority.

### Local Authority and Delegation

- **FR-036**: Winds MUST represent Planner direct execution authority separately from Planner delegation ceiling.
- **FR-037**: A Worker/child's effective execution authority MUST NOT exceed the intersection of its explicit child grant, approved Planner delegation ceiling, applicable team policy, and human ceiling.
- **FR-038**: No Planner, Worker, model, tool output, repository file, hook, imported history, runtime, or delegated child may self-expand an authority ceiling.
- **FR-039**: Where policy uses `deny`, `ask`, and `allow`, precedence MUST be fail-closed such that explicit deny cannot be overridden by transient lower-authority text/request.
- **FR-040**: The first delegation slice MUST support at most one Planner -> one Worker relationship per demonstrated walking-skeleton path; broad recursive fleets are out of scope.
- **FR-041**: A delegation proposal MUST be inspectable before human approval and include bounded task identity, relevant workspace/root/context, requested capabilities/resources, applicable budget limits where used, Planner direct authority, delegation ceiling, Worker authority, and reported enforcement quality.
- **FR-042**: Content-bound/resource-bound approvals MUST be invalidated when the approved identity/content changes materially according to the later plan's deterministic binding rule.
- **FR-043**: Winds-managed policy/trust state MUST reside in a protection domain not writable as ordinary governed worktree content by the actor whose authority it controls.
- **FR-044**: Winds MUST represent enforcement quality truthfully using at least `WINDS_ENFORCED`, `OS_SANDBOX_ENFORCED`, `AGENT_NATIVE_ENFORCED`, `BEST_EFFORT_TRIPWIRE`, `OBSERVATION_ONLY`, and `UNAVAILABLE` where applicable.
- **FR-045**: Winds MUST NOT label an operation `WINDS_ENFORCED` when the third-party runtime can bypass the claimed restriction through direct host access that Winds does not mediate.
- **FR-046**: A Worker's structured completion/result is `AGENT_REPORTED` unless independently observed or explicitly human-decided; completion MUST NOT imply verification or acceptance.

### Workspace / Candidate Isolation Truth

- **FR-047**: Git worktree separation MUST NOT be represented as an OS/process/network/secret sandbox.
- **FR-048**: When a later task requires an isolated Worker edit worktree, Winds MUST bind that Worker to exact worktree/repository identity and retain dirty/failed/ambiguous state for recovery rather than force-cleaning it.
- **FR-049**: This specification MUST NOT authorize automatic winner selection, merge, rebase, cherry-pick, push, PR creation, conflict resolution, or primary-checkout mutation by an autonomous actor.
- **FR-050**: Candidate identity used for acceptance MUST be exact Git identity; mutable branch/name labels alone are insufficient.

### Review Independence and Verification

- **FR-051**: Independent reviewer context MUST bind to the exact candidate and applicable acceptance criteria.
- **FR-052**: Under the independent-review policy, builder confidence, persuasion, self-assessment, or “tests passed” prose MUST NOT substitute for independently observed evidence and SHOULD be excluded from the review-safe context when not needed to reproduce the work.
- **FR-053**: Winds MUST independently run/observe required deterministic repository-native gates against the exact acceptance candidate rather than accepting agent-reported check results.
- **FR-054**: Candidate identity movement MUST automatically make prior candidate-bound deterministic evidence and independent review stale/not-applicable to the new candidate.
- **FR-055**: Historical stale evidence/review MUST remain traceable rather than being rewritten as though it applied to the new candidate.
- **FR-056**: `IDLE`, `DONE`, agent completion, or Worker success MUST NOT imply `VERIFIED` or `ACCEPTED`.
- **FR-057**: Final landing/selection MUST remain an explicit human decision under this specification.

### Findability and Context Selection

- **FR-058**: Winds SHOULD support deterministic fuzzy/searchable selection of named workspaces/sessions without requiring native session IDs.
- **FR-059**: Winds SHOULD support file/directory selection that resolves to exact canonical path identity before use.
- **FR-060**: Changed/recent/test/symbol-derived context candidates MUST retain provenance for how they were selected; unavailable semantic intelligence MUST remain explicitly unavailable.
- **FR-061**: Visibility in a picker/search MUST NOT itself authorize an actor to read/send/modify that resource.

### Persistence, Privacy, Recovery, and Platform Truth

- **FR-062**: Persistent canonical state needed for session/task/context/evidence continuity MUST remain local-first and MUST NOT require an implicit cloud control plane.
- **FR-063**: Secret-bearing credentials/tokens and full process environments MUST NOT be duplicated into canonical session/context records by default merely to enable continuation.
- **FR-064**: Failed, dirty, interrupted, ownership-lost, stale, or ambiguous state MUST be retained truthfully for recovery rather than normalized into success or automatically deleted.
- **FR-065**: Platform/runtime-domain claims MUST be limited to platforms and execution domains proven by deterministic evidence for the accepted implementation slice.
- **FR-066**: Native Windows workspace/ConPTY support MUST NOT imply native-Windows authoritative verification support while that separate authority remains unsupported.
- **FR-067**: The initial Spec 006 implementation MUST NOT require a persistent daemon/session owner; Winds process restart cannot be claimed to preserve live process ownership under this slice.

## Explicit Non-Goals for the First Spec 006 Implementation Program

The formal specification deliberately does **not** authorize these as part of the first walking skeleton:

- a persistent `windsd`/daemon or cross-restart live PTY/session owner;
- a public Winds IPC/runtime protocol;
- local network listeners, HTTP/SSE/WebSocket agent control, or remote execution;
- SSH/container/cloud scheduler domains;
- MCP runtime/tool execution or unstable MCP-over-ACP;
- ACP draft protocol v2;
- a generic plugin/provider marketplace or runtime framework;
- every coding-agent integration;
- a large heterogeneous fleet, recursive delegation, automatic team topology, or automatic winner selection;
- automatic merge/rebase/cherry-pick/push/PR creation or autonomous conflict resolution;
- an OS sandbox platform, Docker/Kubernetes sandbox manager, or claims that worktrees/ACP roots are sandboxes;
- custom model serving, model training/fine-tuning, provider routing, or credential harvesting;
- browser-cookie harvesting or silent reuse/duplication of credentials;
- trusting repository hooks, skills, MCP servers, configuration, or imported histories merely because they exist;
- a custom GPU terminal renderer/emulator;
- SQL Studio/database execution scope;
- LLM Observatory/token-cost telemetry scope;
- broad remote workspace persistence;
- rebuilding every provider-native history parser before proving the canonical Winds context/handoff model.

A later persistent-owner phase requires a separate explicit threat model plus versioned lifecycle/ownership/authenticated-local-control design before coding.

A later MCP phase requires exact then-current MCP specification/SDK pinning plus authority/enforcement analysis before execution is enabled.

## Deterministic & Adversarial Test Requirements Before Implementation Acceptance

The eventual plan/tasks MUST map implementation to deterministic tests covering at least the following; implementation may not weaken these into manual-only claims.

### Identity and Continuity

- rename workspace/session without identity or evidence-link breakage;
- new session continues same task without silently creating a task;
- new task creates distinct canonical identity despite identical display text;
- ambiguous continuation fails to explicit selection;
- exact native resume succeeds only with exact revalidated mapping;
- unavailable/stale native resume yields `RECONSTRUCTED` or fail-closed state, never false `RESUMED`;
- persisted/reused native/process ID cannot satisfy old ownership/resume by coincidence;
- context compaction does not modify canonical evidence/work state.

### Runtime and Capability Truth

- absent runtime discovery does not install/start anything;
- runtime replacement after discovery is detected before structured use;
- declared capability remains declared until locally observed;
- runtime/model identity cannot collapse into one opaque string without provenance;
- unobservable auth readiness remains unknown/unavailable without credential probing or model call.

### Context and Imported History

- imported history cannot overwrite a conflicting Winds-observed fact;
- imported/tool text containing authority instructions cannot self-grant capability;
- handoff reports unavailable provider-private state;
- transfer report differentiates canonical/transferred/derived/omitted/unavailable state;
- bounded context generation is deterministic for the same canonical inputs/policy.

### Authority

- child authority cannot exceed approved child/delegation/team/human ceiling;
- Planner direct authority does not expand because its delegation ceiling is broader;
- Planner cannot modify the protected policy plane to self-expand authority;
- explicit deny outranks transient ask/allow text;
- changed content-bound approval invalidates stale authorization;
- runtime direct host access is labelled with weaker enforcement quality rather than false Winds enforcement;
- ACP additional roots cannot bypass Winds/root/OS enforcement claims.

### Workspace and Recovery

- isolated Worker path, when later implemented, binds exact worktree identity;
- failed/dirty/ambiguous Worker state is retained and not force-cleaned;
- no automatic primary-checkout mutation by Winds verification;
- worktree presence never proves OS sandboxing.

### Review and Evidence

- reviewer context binds exact candidate/criteria;
- builder persuasion/confidence is excluded from independent authority;
- candidate mutation invalidates prior deterministic evidence and independent review;
- agent “done/tests passed” claim cannot close verification gate;
- final landing remains human-decided.

### Cross-Platform

- Linux/macOS/Windows/WSL claims are tested only where claimed by the task;
- path/root canonicalization fails closed;
- execution-domain identity is explicit;
- no remote authority is inherited because a path/runtime appears similar.

## Measurable Success Criteria

- **SC-001**: Renaming a workspace or session changes display text while 100% of stable identity, canonical task links, candidate links, and historical evidence references remain unchanged in deterministic fixtures.
- **SC-002**: One workspace supports at least 20 fixture sessions across at least 5 canonical tasks with deterministic listing/selection and no identity collision or task/session conflation.
- **SC-003**: `continue`, `fork`, and `new task` fixtures produce distinct expected canonical relationships with zero heuristic silent task creation.
- **SC-004**: Native-resume-present, native-resume-absent, and stale/ambiguous-mapping fixtures produce truthful `RESUMED`, `RECONSTRUCTED`/explicit failure, and `OWNERSHIP_LOST`/unavailable outcomes with zero false resume claims.
- **SC-005**: For identical canonical input and policy, context capsule + transfer-report fixtures are byte-for-byte deterministic except explicitly excluded timestamps/non-authoritative runtime data, and every included safety/evidence fact retains provenance.
- **SC-006**: A cross-runtime fixture proves canonical task continuation without claiming transfer of unavailable provider-private state and explicitly enumerates transferred/not-transferred categories.
- **SC-007**: Runtime discovery fixtures for Codex/Claude prove zero agent-task execution, prompt/model calls, auto-install, auto-update, terms acceptance, or credential duplication during discovery.
- **SC-008**: Authority adversarial tests prove 100% rejection/reapproval of child-over-ceiling, Planner self-escalation, explicit-deny bypass, prompt-injected grants, and materially changed content-bound approvals.
- **SC-009**: Every safety-relevant capability in the first accepted runtime path reports one explicit enforcement-quality value; no tested bypassable host access is labelled `WINDS_ENFORCED`.
- **SC-010**: Candidate mutation fixtures invalidate 100% of prior candidate-bound deterministic gate/reviewer applicability while preserving historical traceability.
- **SC-011**: Independent-review fixtures prove exact candidate/criteria binding and demonstrate that builder confidence/prose alone cannot satisfy acceptance.
- **SC-012**: The first accepted delegation walking skeleton contains exactly one Planner -> one Worker relationship and no recursive fleet dependency is required to prove the full canonical-work -> delegation -> result -> exact-candidate -> independent-review -> deterministic-verification -> human-decision loop.
- **SC-013**: No accepted first-slice change requires a daemon, public network control plane, MCP runtime, remote executor, generic plugin marketplace, custom renderer, SQL Studio, or LLM Observatory.
- **SC-014**: Existing Spec 003 verification authority regressions remain green; adding agentic session/delegation state does not make agent reports eligible verification evidence or weaken exact-candidate checks.

Performance/latency targets for runtime discovery, session selection, context preparation, and structured-control startup MUST be benchmarked during Plan against current repository/platform baselines rather than invented in this specification.

## Protocol / Dependency Constraints for Future Plan

This specification records the governance pin but lands no dependency:

```text
ACP_WIRE_PROTOCOL=1
ACP_SCHEMA=schema-v1.20.0
ACP_SCHEMA_COMMIT=5e89c71497fe07dd4ae633c181a17224f4a8956d
ACP_RUST_SDK=2.0.0
ACP_RUST_SDK_COMMIT=ce023279824149008659dd8f4b8b70266a7e8210
UNSTABLE_PROTOCOL_V2=DISABLED
UNSTABLE_MCP_OVER_ACP=DISABLED
MCP_RUNTIME=NOT_AUTHORIZED_IN_FIRST_SLICE
PERSISTENT_OWNER_IPC=NOT_AUTHORIZED_IN_FIRST_SLICE
REMOTE_EXECUTION=NOT_AUTHORIZED_IN_FIRST_SLICE
```

The future Plan MUST reopen the ACP dependency choice rather than silently work around it if the exact dependency landing gates in `docs/provenance/agent-client-protocol-v1-sdk-2.0.0-entry-audit.md` fail under the canonical Winds toolchain/platform/license requirements.

## Specification Acceptance Boundary

Accepting this `spec.md` means only that Winds has frozen user scenarios, product semantics, authority/trust boundaries, non-goals, and measurable acceptance requirements sufficiently to begin a separate **Plan** review.

It does **not** by itself authorize:

- `Cargo.toml` / `Cargo.lock` changes;
- database migrations;
- source/runtime implementation;
- starting Codex/Claude or any other agent;
- sending prompts or calling model/provider APIs;
- credential/terms access;
- ACP transport execution;
- persistent daemon/IPC;
- MCP;
- remote execution;
- workflow-semantic expansion;
- implementation tasks.

The required repository sequence remains:

```text
Constitution 1.1.0 (canonical)
  -> Spec 006 spec.md (this gate)
  -> Plan
  -> Tasks
  -> explicitly authorized implementation slice
  -> deterministic CI / correctness-safety / Ponytail / independent review
  -> exact-candidate evidence reconciliation
  -> explicit human landing decision
```
