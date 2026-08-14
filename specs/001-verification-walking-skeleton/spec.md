# Feature Specification: Verification Walking Skeleton

**Feature Branch**: `agent/verification-walking-skeleton`

**Created**: 2026-08-14

**Status**: Implementation complete; acceptance review in progress

**Input**: Build the smallest end-to-end Winds slice that independently verifies agent-made or existing Git candidates and promotes an explicitly selected verified snapshot without touching the source checkout.

## User Scenarios & Testing

### User Story 1 - Verify One Candidate (Priority: P1)

A repository maintainer gives Winds a clean repository, a pinned base ref, a candidate ref, and one explicit repository-owned check. Winds verifies the candidate in an isolated worktree and produces an Evidence Report from directly observed Git/process facts.

**Why this priority**: Independent verification is the product wedge. If this does not work safely, orchestration and UI are irrelevant.

**Independent Test**: Create a fixture repository with a known passing candidate, run Winds verification, and prove the check ran against the exact candidate snapshot while the source checkout remained byte-for-byte and Git-state unchanged.

**Acceptance Scenarios**:

1. **Given** a clean repository and valid base/candidate refs, **When** a required check exits 0 without mutating the candidate tree, **Then** Winds records exact base/candidate identities and reports the candidate eligible.
2. **Given** a valid candidate, **When** the required check exits non-zero, **Then** Winds records the exact command, exit result, bounded output evidence, and reports the candidate blocked.
3. **Given** a dirty source checkout, **When** verification is requested, **Then** Winds fails closed before provisioning a candidate worktree.

### User Story 2 - Compare Two Candidates (Priority: P2)

A maintainer verifies two candidates from the same pinned base and can compare their objective evidence without a model-generated winner score.

**Why this priority**: Comparison is useful only after single-candidate evidence is trustworthy.

**Independent Test**: Verify one passing and one failing candidate against the same contract and confirm the reports share the same base/check identity while retaining candidate-specific evidence.

**Acceptance Scenarios**:

1. **Given** two candidate refs and one pinned base/check contract, **When** both are verified, **Then** Winds produces independent objective evidence for each and leaves selection to the caller.

The walking skeleton does not add a dedicated `compare` command or comparison UI. P2 is satisfied at this layer by two independently produced JSON Evidence Reports; a comparison surface remains deferred until there is a second concrete product need.

### User Story 3 - Promote a Verified Snapshot (Priority: P3)

After reviewing evidence, the maintainer explicitly requests promotion of one eligible candidate to a dedicated Winds-selected branch without merging, rebasing, pushing, or touching the source checkout.

**Why this priority**: The product must bind evidence to the exact state selected for downstream integration.

**Independent Test**: Promote an eligible fixture candidate and confirm the selected branch points at the verified snapshot, required checks are revalidated against that snapshot, recheck evidence is persisted, and the source checkout branch/index/worktree are unchanged.

**Acceptance Scenarios**:

1. **Given** an eligible evidence record whose candidate state has not changed, **When** the caller explicitly requests promotion, **Then** Winds creates only a dedicated Winds-selected ref and records the request/promotion.
2. **Given** stale evidence or a changed candidate, **When** promotion is attempted, **Then** Winds blocks promotion until verification is rerun.

### Edge Cases

- Source checkout becomes dirty after a run starts.
- Candidate ref or base ref cannot be resolved to an exact commit.
- Worktree creation partially succeeds.
- Required check times out, cannot start, produces oversized output, or mutates tracked/untracked files.
- Git reports a worktree/path/ref mismatch during recovery.
- Candidate contains commits made by an authoring agent.
- Disk or database writes fail mid-transition.

## Requirements

### Functional Requirements

- **FR-001**: Winds MUST resolve and persist an exact base commit OID before candidate provisioning.
- **FR-002**: Winds MUST reject a dirty source checkout by default for the walking skeleton.
- **FR-003**: Winds MUST create candidate workspaces using system Git and MUST NOT modify the source checkout. Verification worktrees MUST be detached because this slice does not author candidate changes.
- **FR-004**: Winds MUST execute explicit repository-owned checks with a timeout and capture observed exit status, duration, and bounded output metadata.
- **FR-005**: Winds MUST distinguish evidence authority. The walking skeleton persists `WINDS_OBSERVED` for directly observed facts and `CALLER_REQUESTED` for an explicit promotion request. It MUST NOT claim authenticated human identity. `AGENT_REPORTED` and `INFERRED` remain reserved until a real source exists.
- **FR-006**: Winds MUST persist run/evidence/promotion state using SQLite WAL, keep event/projection transitions transactional, and store large/raw streams outside SQLite. A standalone `Task` persistence entity is intentionally deferred until multi-candidate task behavior exists; in this slice the run carries the pinned base/check identity directly.
- **FR-007**: Winds MUST fail closed when required checks fail, time out, cannot run, or evidence no longer matches the candidate snapshot. A request that fails before an Evidence Report exists cannot be eligible.
- **FR-008**: Winds MUST NOT compute an overall winner score or automatically select a candidate.
- **FR-009**: Promotion MUST require an explicit caller request, revalidate the exact candidate, persist the promotion recheck evidence, and bind the selected ref to the exact promoted commit/tree.
- **FR-010**: Winds MUST NOT merge, rebase, cherry-pick into the source branch, push, open PRs, auto-resolve conflicts, force-clean, force-remove worktrees, or automatically delete ambiguous/dirty candidate state.
- **FR-011**: Winds MUST retain ambiguous state and surface manual recovery rather than guessing ownership or deleting user data. An interrupted `PROVISIONING` run MUST NOT be auto-promoted to a ready lifecycle state by recovery.
- **FR-012**: The first implementation MUST have no real Claude/Codex integration; candidate generation is fixture/existing-ref based until the Git/evidence invariants pass.
- **FR-013**: Repository mutation operations and recovery reconciliation MUST serialize per Git common directory.
- **FR-014**: `WINDS_HOME` and persisted repository/worktree paths MUST be canonical UTF-8 paths in this slice; unsupported non-UTF-8 paths fail closed rather than being stored lossily.

### Key Entities

- **CandidateRun**: Pinned base/check identity, candidate source/ref, detached worktree identity, process/check disposition, snapshot identity.
- **EvidenceReport**: Exact candidate snapshot plus observed check/Git/process evidence and completeness warnings.
- **Decision event**: Explicit caller-requested promotion intent; it is not proof of authenticated human identity.
- **Promotion**: Dedicated selected ref bound to an exact verified snapshot.

A standalone **Task** entity is deferred by design. Ponytail review found that the original walking-skeleton schema made `Task` an alias for `run_id` with no independent behavior. It must not return until a second concrete caller (for example, multi-candidate comparison under one immutable brief) proves the boundary.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Before a 0.1 release claim, 100 fixture create/verify/promote/reconcile cycles complete with zero source-checkout mutations. This is a pre-release soak gate, not a 100x loop in every PR CI run.
- **SC-002**: Dirty or ambiguous candidate workspaces are retained in 100% of safety tests; no forced worktree removal is used.
- **SC-003**: A required failing or timed-out check blocks eligibility in 100% of tests.
- **SC-004**: Evidence records the exact base/candidate identity and exact required check definition used for verification; promotion records a fresh recheck before selected-ref creation.
- **SC-005**: Recovery must never delete or auto-adopt state whose ownership/lifecycle cannot be proven. Crash-injection coverage for partial Git/DB transitions remains a pre-release hardening gate.

## Assumptions

- Initial fixture development targets ordinary non-bare, single-root Git repositories without submodules, sparse/partial clones, or unverified Git LFS behavior.
- Initial implementation targets Unix-like execution semantics (Linux/macOS/WSL2 Linux domain); native Windows is deferred.
- Project-owned checks may be non-hermetic; Winds records that limitation rather than claiming determinism.
- Git worktrees isolate checkout/index state but do not provide OS, network, secret, service, or container isolation.
