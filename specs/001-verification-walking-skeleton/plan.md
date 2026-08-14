# Implementation Plan: Verification Walking Skeleton

## Summary

Build one unpublished Rust package with a small CLI composition root and private application/domain seams. Use system Git for repository/worktree/ref authority, SQLite WAL for transactional metadata/events/projections, filesystem blobs for potentially large raw output, and an explicit check executor. No real coding-agent integration is included in this slice.

## Constitution Check

- Evidence authority is explicit and promotion cannot trust agent prose: PASS.
- Primary checkout is never mutated by Winds-controlled flows: REQUIRED.
- Spec/plan/tasks precede code: PASS.
- Ponytail/YAGNI gate: one process, one package, no daemon/protocol/UI/terminal/VCS abstraction beyond required private ports.
- Independent review required before acceptance: REQUIRED.

## Technical Context

- **Language**: Rust stable; package unpublished initially.
- **State**: SQLite WAL through `rusqlite`; bounded filesystem blobs for raw output if needed.
- **VCS**: system Git >= 2.36, machine-readable porcelain only.
- **Process execution**: standard process APIs; Unix process-group/timeout behavior may be introduced only as required by acceptance tests.
- **Testing**: Rust integration fixtures construct temporary Git repositories and assert Git/ref/worktree invariants.
- **UI**: out of scope; v0 is handled separately and consumes future stable application contracts.

## Architecture

One OS process:

`CLI -> application use cases -> private domain -> adapters`

Private ports are limited to current needs:

- `VcsWorkspace`: resolve base/candidate, create/inspect/retain worktree, create selected ref, reconcile.
- `CheckExecutor`: execute an exact check and return observed disposition.
- `Store`: transactionally persist events/projections/evidence metadata.
- `BlobStore`: persist bounded raw output/artifacts when DB rows are inappropriate.
- `Clock/Ids`: test seams only if deterministic tests require them.

Do NOT introduce `Runtime`, `TerminalEngine`, IPC DTOs, daemon/service boundaries, public protocol types, generic plugin abstractions, custom VCS semantics, or agent capability matrices in this slice.

## Data and Evidence

SQLite tables may start with the minimum fields needed for `tasks`, `candidate_runs`, `events`, `evidence_reports`, `decisions`, and `promotions`. Events are append-only; current projections may be mutable. Every evidence-producing record includes an authority classification. Large stdout/stderr or native streams are stored as bounded blobs referenced by metadata.

Eligibility states: `ELIGIBLE`, `WARNING`, `BLOCKED`.
Check states: `PASS`, `FAIL`, `TIMEOUT`, `ERROR`, `NOT_RUN`.

## Safety Strategy

- Resolve and persist exact OIDs before provisioning.
- Reject dirty primary checkout by default.
- Serialize Winds Git mutations per common Git directory.
- Never parse human Git output when porcelain/plumbing exists.
- Never force-remove a worktree or recursively delete a path based only on DB ownership.
- Quiesce candidate/check processes before snapshot/promotion/cleanup.
- Retain ambiguous state and mark manual recovery required.
- Promotion creates a dedicated Winds-selected ref only; normal Git/PR/CI performs downstream integration.

## Dependencies

Keep dependencies minimal. `rusqlite` is justified by the accepted SQLite persistence decision. Additional crates require a concrete task-level need and Ponytail review. Prefer system Git over a Git library for mutation semantics in 0.1.

## Review Strategy

1. Deterministic CI: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`.
2. Correctness/safety review against spec acceptance and Git/data-loss invariants.
3. Ponytail review focused only on deletable complexity/dependencies/abstractions.
4. Independent reviewer pass; authoring agent cannot self-certify.
5. Optional external review bots/services add findings when connected but are not release authority.

## Delivery

Implementation lives on `agent/verification-walking-skeleton` as a Draft PR. `main` receives implementation only after all blocking review findings are resolved and deterministic checks pass.
