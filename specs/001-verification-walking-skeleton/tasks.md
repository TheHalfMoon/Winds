# Tasks: Verification Walking Skeleton

This checklist records **as-built truth** for the verification walking skeleton and its bounded hardening follow-ups. A checked item has implementation/evidence in merged or active review history. Open items are not implied complete by nearby work and remain gates for the stated scope.

## Phase 1 - Repository and Test Harness

- [x] **T001** Create one unpublished Rust package and minimal `winds` binary composition root.
- [x] **T002** Add fixture helpers that initialize temporary Git repositories with exact base/pass/fail candidate commits.
- [x] **T003** Add a safety test proving the source checkout branch/index/worktree remain unchanged during candidate verification and promotion.

## Phase 2 - Git Candidate Workspace

- [x] **T004** Implement exact base/candidate OID resolution through system Git.
- [x] **T005** Reject dirty source checkout before provisioning, including when repository config attempts to hide untracked files.
- [x] **T006** Create a detached, locked Winds-owned candidate worktree outside the source checkout and Git common directory, and record its ownership metadata without creating a verification branch.
- [x] **T007** Inspect candidate Git state using machine-readable output only; explicitly include non-ignored untracked files and retain unknown/dirty state.

## Phase 3 - Persistence and Evidence

- [x] **T008** Add SQLite migration for the minimum proven run/event/evidence/promotion records; standalone `Task`/`Decision` tables are intentionally not present.
- [x] **T009** Persist lifecycle projections with corresponding events transactionally; keep recovery-required observations append-only; enforce run ownership for events and one-shot primary Evidence Reports.
- [x] **T010** Implement explicit required-check execution with pass/fail/timeout disposition, bounded output capture, process-group termination, and bounded stream shutdown.
- [x] **T011** Build an Evidence Report bound to the exact base/candidate snapshot and check definition with content-addressed SHA-256 blob metadata; fail closed if an existing digest path contains different bytes.
- [x] **T012** Add tests proving fail, timeout, dirty/not-run preflight, check mutation, hostile inherited Git context, repository-local Winds state rejection, whitespace-preserving repository paths, corrupt evidence, and stale candidate evidence fail closed.

## Phase 4 - Explicit Selection and Promotion

- [x] **T013** Record explicit `CALLER_REQUESTED` promotion intent separately from `WINDS_OBSERVED` verification evidence; do not claim authenticated human identity.
- [x] **T014** Revalidate candidate state and required checks before promotion, persist fresh promotion-recheck evidence, and serialize the shared-worktree recheck/promotion sequence per Git common directory.
- [x] **T015** Create a dedicated Winds-selected branch/ref at the exact verified snapshot without touching the source checkout.
- [x] **T016** Add explicit negative fixtures for every prohibited downstream Git operation (merge, rebase, cherry-pick, push, force-clean, force-remove). PR #2 adds an instrumented Git shim that traces the real `verify -> promote -> recover` lifecycle and fails on any prohibited operation while proving the trace observed Winds worktree activity; quality run #80 passed format, Clippy `-D warnings`, and tests on Ubuntu/macOS before this task-truth update.

## Phase 5 - Recovery and Review

- [x] **T017** Reconcile persisted workspace metadata with `git worktree list --porcelain -z`; dirty/missing/mismatched/interrupted-provisioning state reports `MANUAL_RECOVERY_REQUIRED` without cleanup or lifecycle auto-adoption.
- [x] **T018** Add fault-injection fixtures for partial worktree creation, failed DB write, and interrupted cross-resource transitions. PR #3 injects a `git worktree lock` failure after successful add, a SQLite-trigger failure on the `READY` lifecycle write, and a promotion-row write failure after selected-ref creation; each path proves fail-closed/manual-recovery or idempotent retry behavior without production fault hooks. Quality run #87 passed format, Clippy `-D warnings`, and tests on Ubuntu/macOS before this task-truth update.
- [x] **T019** Add deterministic read-only CI with pinned GitHub Action SHAs, Rust `1.97.1`, committed `Cargo.lock`, `--locked`, rustfmt, Clippy `-D warnings`, and tests on Ubuntu/macOS.
- [x] **T020** Re-run correctness/safety review after DeepCode review-agent and Qodo findings. The reconciled code head `1d4390af48c668542b4b8b9ad2d6ba3b34d10543` passed read-only quality run #71: rustfmt, Clippy `-D warnings`, and tests on Ubuntu/macOS. Blocking findings around repository-local state, corrupt evidence blobs, Git config-hidden untracked files, path whitespace, promotion recheck concurrency, and Winds-owned clean-check/provision serialization were resolved.
- [x] **T021** Re-run Ponytail v4.9.0 after the external-review corrections. The repo-wide Git lock remains intentionally simpler than adding a per-run locking subsystem; no additional abstraction or dependency can be removed without weakening a current invariant. Final verdict remains `Lean already. Ship.`
- [x] **T022** Complete independent/external review reconciliation on the final Ready-for-Review head. Qodo explicitly updated its final review through exact PR #1 head `1f868d75568a851394dbae17928325bbc0b93e10` with `0 Bugs / 0 Rule violations / 0 Skill insights`; CodeRabbit reported no actionable comments on the final incremental review, all review threads were resolved, and no Greptile result became available to inspect. PR #1 was then merged by merge commit `1b5885faa019567657a2527cb2b6c54497af9077`, preserving the reviewed head as a direct parent.

## Pre-release Soak

- [x] **T023** Execute the SC-001 100-cycle create/verify/promote/reconcile soak with zero source-checkout mutations before any 0.1 release claim. PR #4 adds an ignored explicit pre-release harness plus a dedicated path-scoped/manual workflow rather than multiplying the soak into ordinary PR CI. Pre-release-soak run #2 checked out exact head `25a22006597402ab8589b54f9bf47339b370eab2` and completed `100/100` fresh create/verify/promote/reconcile cycles with zero observed primary-checkout mutations; quality run #96 also passed Format, Clippy `-D warnings`, and Tests on Ubuntu/macOS on that exact head. Final task-truth-head gates must re-run before merge.

## Explicitly Deferred Product Scope

- Claude/Codex drivers, ACP, MCP, A2A, Pi/OpenCode.
- `winds race` orchestration and dedicated candidate-comparison UI/command.
- TUI/dashboard and v0 UI integration.
- Terminal emulator/PTY ownership and `windsd`.
- Port/service/container/database isolation.
- Graphify/semantic brain/Jujutsu dependency.
- Broad sandbox/security claims, secret broker, MCP firewall.
- Automatic winner scoring, merge/rebase/push/PR automation.
- Native Windows execution semantics.
