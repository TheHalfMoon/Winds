# Feature Specification: Verification Walking Skeleton

**Feature Branch**: `agent/verification-walking-skeleton`

**Created**: 2026-08-14

**Status**: Approved for implementation planning

**Input**: Build the smallest end-to-end Winds slice that independently verifies agent-made or existing Git candidates and promotes a human-selected verified snapshot without touching the primary checkout.

## User Scenarios & Testing

### User Story 1 - Verify One Candidate (Priority: P1)

A repository maintainer gives Winds a clean repository, a pinned base ref, a candidate ref, and one explicit repository-owned check. Winds verifies the candidate in an isolated worktree and produces an Evidence Report from directly observed Git/process facts.

**Why this priority**: Independent verification is the product wedge. If this does not work safely, orchestration and UI are irrelevant.

**Independent Test**: Create a fixture repository with a known passing candidate, run Winds verification, and prove the check ran against the exact candidate snapshot while the primary checkout remained byte-for-byte and Git-state unchanged.

**Acceptance Scenarios**:

1. **Given** a clean repository and valid base/candidate refs, **When** a required check exits 0 without mutating the candidate tree, **Then** Winds records exact base/candidate identities and reports the candidate eligible.
2. **Given** a valid candidate, **When** the required check exits non-zero, **Then** Winds records the exact command, exit result, bounded output evidence, and reports the candidate blocked.
3. **Given** a dirty primary checkout, **When** verification is requested, **Then** Winds fails closed before provisioning a candidate worktree.

### User Story 2 - Compare Two Candidates (Priority: P2)

A maintainer verifies two candidates from the same pinned base and sees objective evidence side-by-side without a model-generated winner score.

**Why this priority**: Comparison is useful only after single-candidate evidence is trustworthy.

**Independent Test**: Verify one passing and one failing candidate against the same contract and confirm the reports share the same base/contract identity while retaining candidate-specific evidence.

**Acceptance Scenarios**:

1. **Given** two candidate refs and one pinned base/check contract, **When** both are verified, **Then** Winds exposes objective check/Git/process evidence for each and leaves selection to the human.

### User Story 3 - Promote a Verified Snapshot (Priority: P3)

After reviewing evidence, the maintainer explicitly promotes one eligible candidate to a dedicated Winds-selected branch without merging, rebasing, pushing, or touching the primary checkout.

**Why this priority**: The product must bind evidence to the exact state the human selected.

**Independent Test**: Promote an eligible fixture candidate and confirm the selected branch points at the verified snapshot, required checks are revalidated against that snapshot, and the primary checkout branch/index/worktree are unchanged.

**Acceptance Scenarios**:

1. **Given** an eligible evidence record whose candidate state has not changed, **When** the human explicitly promotes it, **Then** Winds creates/updates only a dedicated Winds-selected ref and records the decision/promotion.
2. **Given** stale evidence or a changed candidate, **When** promotion is attempted, **Then** Winds blocks promotion until verification is rerun.

### Edge Cases

- Source checkout becomes dirty after a run starts.
- Candidate ref or base ref cannot be resolved to an exact commit.
- Worktree creation partially succeeds.
- Required check times out, crashes, produces oversized output, or mutates tracked/untracked files.
- Git reports a worktree/path/ref mismatch during cleanup or recovery.
- Candidate contains commits made by an authoring agent.
- Disk or database writes fail mid-transition.

## Requirements

### Functional Requirements

- **FR-001**: Winds MUST resolve and persist an exact base commit OID before candidate provisioning.
- **FR-002**: Winds MUST reject a dirty primary checkout by default for the walking skeleton.
- **FR-003**: Winds MUST create candidate workspaces using system Git and MUST NOT modify the primary checkout.
- **FR-004**: Winds MUST execute explicit repository-owned checks with a timeout and capture observed exit status, duration, and bounded output metadata.
- **FR-005**: Winds MUST distinguish `WINDS_OBSERVED`, `AGENT_REPORTED`, and `INFERRED` evidence authority.
- **FR-006**: Winds MUST persist task/run/evidence/promotion state transactionally using SQLite WAL and store large/raw streams outside SQLite.
- **FR-007**: Winds MUST invalidate eligibility when required checks fail, time out, are not run, or evidence no longer matches the candidate snapshot.
- **FR-008**: Winds MUST NOT compute an overall winner score or automatically select a candidate.
- **FR-009**: Promotion MUST require explicit human action and MUST bind the Evidence Report to the exact promoted commit/tree.
- **FR-010**: Winds MUST NOT merge, rebase, cherry-pick into the primary branch, push, open PRs, auto-resolve conflicts, force-clean, force-remove worktrees, or automatically delete ambiguous/dirty candidate state.
- **FR-011**: Winds MUST retain ambiguous state and surface manual recovery rather than guessing ownership or deleting user data.
- **FR-012**: The first implementation MUST have no real Claude/Codex integration; candidate generation is fixture/existing-ref based until the Git/evidence invariants pass.

### Key Entities

- **Task**: Pinned base, immutable brief/contract identity, lifecycle status.
- **CandidateRun**: Candidate source/ref, worktree identity, process/check disposition, snapshot identity.
- **EvidenceReport**: Exact candidate snapshot plus observed check/Git/process evidence and completeness warnings.
- **Decision**: Human-selected candidate and optional rationale/override.
- **Promotion**: Dedicated selected ref bound to an exact verified snapshot.

## Success Criteria

### Measurable Outcomes

- **SC-001**: 100 fixture create/verify/promote/reconcile cycles complete with zero primary-checkout mutations.
- **SC-002**: Dirty or ambiguous candidate workspaces are retained in 100% of safety tests; no forced worktree removal is used.
- **SC-003**: A required failing or timed-out check blocks eligibility in 100% of tests.
- **SC-004**: Evidence always records the exact base/candidate identity and the exact required check definition used for the decision.
- **SC-005**: Restart/recovery fixtures never delete state whose ownership cannot be proven.

## Assumptions

- Initial fixture development targets ordinary non-bare, single-root Git repositories without submodules, sparse/partial clones, or unverified Git LFS behavior.
- Initial implementation targets Unix-like execution semantics (Linux/macOS/WSL2 Linux domain); native Windows is deferred.
- Project-owned checks may be non-hermetic; Winds records that limitation rather than claiming determinism.
- Git worktrees isolate checkout/index state but do not provide OS, network, secret, service, or container isolation.
