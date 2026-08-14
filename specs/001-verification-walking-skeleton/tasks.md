# Tasks: Verification Walking Skeleton

## Phase 1 - Repository and Test Harness

- [ ] **T001** Create one unpublished Rust package and minimal `winds` binary composition root.
- [ ] **T002** Add fixture helpers that initialize temporary Git repositories with exact base/pass/fail candidate commits.
- [ ] **T003** Add a safety test proving the primary checkout branch/index/worktree remain unchanged during candidate verification.

## Phase 2 - Git Candidate Workspace

- [ ] **T004** Implement exact base/candidate OID resolution through system Git.
- [ ] **T005** Reject dirty primary checkout before provisioning.
- [ ] **T006** Create a Winds-owned candidate worktree outside the primary checkout and record its ownership metadata.
- [ ] **T007** Inspect candidate Git state using machine-readable output only; retain unknown/dirty state.

## Phase 3 - Persistence and Evidence

- [ ] **T008** Add SQLite schema/migrations for the minimum task/run/event/evidence/promotion records.
- [ ] **T009** Persist append-only events and current projections transactionally.
- [ ] **T010** Implement explicit required-check execution with timeout/error/pass/fail disposition and bounded output capture.
- [ ] **T011** Build an Evidence Report bound to the exact candidate snapshot and check definition.
- [ ] **T012** Add tests proving fail/timeout/not-run/stale evidence blocks eligibility.

## Phase 4 - Human Decision and Promotion

- [ ] **T013** Record explicit human candidate selection separately from verification evidence.
- [ ] **T014** Revalidate candidate state and required checks before promotion.
- [ ] **T015** Create a dedicated Winds-selected branch/ref at the exact verified snapshot without touching the primary checkout.
- [ ] **T016** Add tests proving Winds does not merge, rebase, cherry-pick, push, force-clean, force-remove, or delete ambiguous state.

## Phase 5 - Recovery and Review

- [ ] **T017** Reconcile persisted workspace metadata with `git worktree list --porcelain -z` after interrupted/partial operations.
- [ ] **T018** Add crash/failure fixtures for partial worktree creation, failed DB write, failed check, and ambiguous ownership.
- [ ] **T019** Add deterministic CI quality workflow.
- [ ] **T020** Run correctness/safety review and resolve blocking findings.
- [ ] **T021** Run Ponytail review; remove unjustified code, dependencies, and abstractions without weakening safety.
- [ ] **T022** Run independent reviewer pass and reconcile findings against the spec/constitution.

## Explicitly Deferred

- Claude/Codex drivers, ACP, MCP, A2A, Pi/OpenCode.
- `winds race` orchestration.
- TUI/dashboard and v0 UI integration.
- Terminal emulator/PTY ownership and `windsd`.
- Port/service/container/database isolation.
- Graphify/semantic brain/Jujutsu dependency.
- Broad sandbox/security claims, secret broker, MCP firewall.
- Automatic winner scoring, merge/rebase/push/PR automation.
