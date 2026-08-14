# Implementation Plan: Verification Walking Skeleton

## Summary

Build one unpublished Rust package with a small CLI composition root and private modules. Use system Git for repository/worktree/ref authority, SQLite WAL for transactional metadata/events/projections, filesystem blobs for bounded raw output, and an explicit check executor. No real coding-agent integration is included in this slice.

## Constitution Check

- Evidence authority is explicit and promotion cannot trust agent prose: PASS.
- Source checkout is never mutated by Winds-controlled verification/promotion flows: REQUIRED.
- Spec/plan/tasks precede implementation: PASS.
- Ponytail/YAGNI gate: one process, one package, detached verification worktrees, no daemon/protocol/UI/terminal/VCS abstraction beyond current needs.
- Independent review required before acceptance: REQUIRED.

## Technical Context

- **Language/toolchain**: Rust `1.97.1`, pinned in `rust-toolchain.toml`; package unpublished.
- **Dependency resolution**: committed `Cargo.lock`; CI uses `--locked`.
- **State**: SQLite WAL through `rusqlite = 0.40.2` with bundled SQLite; bounded filesystem blobs for raw output.
- **VCS**: system Git >= 2.36, plumbing/porcelain output only; verification worktrees are detached and locked.
- **Process execution**: `/bin/sh -c` in a dedicated Unix process group; explicit timeout, bounded stdout/stderr, and descendant termination.
- **Testing**: Rust integration fixtures construct temporary Git repositories and assert Git/ref/worktree/evidence invariants on Ubuntu and macOS.
- **UI**: out of scope; v0 UI work consumes future stable application contracts rather than shaping this slice.

## Architecture

One OS process:

`CLI -> domain/use-case functions -> git/check/store modules`

The current code intentionally does not introduce public port traits or one-implementation abstractions. Concrete module ownership is:

- `git`: exact ref/OID resolution, detached worktree creation/inspection, selected-ref creation, common-dir mutation lock, worktree inventory.
- `check`: bounded check execution and process-group timeout/termination.
- `store`: SQLite lifecycle/evidence/event persistence plus filesystem evidence blobs.
- `domain`: serializable evidence/report state shared by the composition root and store.

Do NOT introduce `Runtime`, `TerminalEngine`, IPC DTOs, daemon/service boundaries, public protocol types, generic plugin abstractions, custom VCS semantics, agent capability matrices, standalone `Task`, or separate `BlobStore` abstractions in this slice.

## Data and Evidence

SQLite contains only the tables proven by the walking skeleton:

- `candidate_runs`
- `events`
- `evidence_reports`
- `promotions`

Events are append-only. Lifecycle projections may be updated transactionally with their corresponding events. Recovery-required observations are appended as events and do not rewrite lifecycle state.

Large stdout/stderr streams are stored as bounded filesystem blobs referenced by metadata containing relative path, SHA-256, captured byte count, and truncation state.

Eligibility states: `ELIGIBLE`, `WARNING`, `BLOCKED`.
Observed check states in this slice: `PASS`, `FAIL`, `TIMEOUT`. Check startup/execution failures return an error and never create eligible evidence rather than inventing unused persisted `ERROR`/`NOT_RUN` enum variants.

Evidence authorities currently persisted:

- `WINDS_OBSERVED` for facts Winds directly observed.
- `CALLER_REQUESTED` for the explicit promotion request.

`AGENT_REPORTED` and `INFERRED` remain reserved until real producers exist.

## Safety Strategy

- Resolve and persist exact base/candidate OIDs before provisioning.
- Reject dirty source checkout by default.
- Serialize Git mutations and recovery reconciliation per Git common directory.
- Create verification worktrees detached at exact candidate commits; no verification branch is created.
- Never parse human Git output when porcelain/plumbing exists.
- Never force-remove a worktree or recursively delete a path based only on DB ownership.
- Quiesce check process groups before final evidence/promotion state is accepted.
- A successful check that mutates tracked or untracked candidate state is still `BLOCKED`.
- Retain ambiguous state and surface manual recovery required.
- Never auto-adopt an interrupted `PROVISIONING` run as ready.
- Promotion persists a fresh recheck observation before selected-ref creation.
- Promotion creates a dedicated Winds-selected ref only; normal Git/PR/CI performs downstream integration.
- Canonicalize `WINDS_HOME`; reject non-UTF-8 persisted paths rather than store lossy identities.

## Dependencies

Keep dependencies minimal:

- `rusqlite` + bundled SQLite: justified by accepted transactional local-state decision.
- `serde` / `serde_json`: evidence/report serialization and event payloads.
- `sha2`: evidence blob integrity metadata.

No Git library, CLI framework, async runtime, UUID crate, process-control crate, logging framework, or agent SDK is required for this slice.

## Deterministic CI

CI is read-only and pins external GitHub Actions by commit SHA. It pins Rust `1.97.1` and runs on Ubuntu + macOS:

1. `cargo fmt --all -- --check`
2. `cargo clippy --locked --all-targets --all-features -- -D warnings`
3. `cargo test --locked --all-targets --all-features`

`actions/checkout` runs with `persist-credentials: false`.

## Review Strategy

1. Deterministic CI on the exact PR head.
2. Correctness/safety review against spec acceptance, Git/data-loss invariants, evidence authority, and recovery behavior.
3. Ponytail v4.9.0 review focused only on deletable complexity/dependencies/abstractions.
4. Independent reviewer pass; authoring agent cannot self-certify.
5. Connected external review services add findings when they actually produce a review. A bot summary, skipped run, or rate-limit response is not approval.

## Delivery

Implementation remains on `agent/verification-walking-skeleton` until deterministic/internal gates pass. The PR may then move from Draft to Ready for Review to trigger external reviewers, but `main` receives implementation only after blocking findings are reconciled and an explicit merge decision is made separately.

## Pre-release Gates Not Claimed by This PR

- 100-cycle soak required by SC-001.
- Crash/fault injection for partial worktree creation, DB write failure, and interrupted cross-resource transitions.
- Native Windows process/worktree semantics.
- Sandboxing, network/secret isolation, agent integration, or remote execution.
