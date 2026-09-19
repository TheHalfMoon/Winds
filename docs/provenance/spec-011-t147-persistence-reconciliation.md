# Spec 011 T147 — Persistent Runtime Persistence and Restart Reconciliation

## Scope

T147 adds only the persistence boundary authorized by Spec 011:

- migration `0013_persistent_runtime_owner.sql`,
- Store schema installation and exact schema validation,
- runtime namespace and owner-generation persistence,
- bounded restart reconciliation,
- focused deterministic tests.

It does not add an endpoint, socket, named pipe, owner process, IPC, PTY or ConPTY ownership change, renderer behavior, provider execution, service manager, new dependency, remote/public control, or plugin behavior.

## Canonical predecessor

T146 merged as:

- merge commit: `ab781dae92d7a6c1cc9a44cba9a4fd26da68e3be`
- merge tree: `f9f4c296415e8287440047f8c0e46d107c02f98e`
- parent 1: `a90575133bfccd8e2879f45930d50497e50aa504`
- parent 2: `448a63a62124ee6fd94f9950d3e63c0f3fc028ae`
- GitHub merge signature: verified/valid

All eight actually-triggered post-merge push workflows for that exact merge completed successfully on attempt 1 before T147 implementation began.

T147 was created from exact `origin/main` at that merge in isolated worktree `.winds-t147`.

## Persistence boundary

Migration 0013 persists:

- immutable canonical runtime namespace ID,
- mutable non-authoritative runtime alias,
- optional canonical workspace/session/terminal references,
- owner-generation history/reference,
- separated ownership, process-liveness, endpoint-availability, and continuity truth,
- bounded lifecycle event kind,
- lifecycle/reconciliation timestamps,
- bounded recovery reason.

It intentionally does not persist:

- PID or process handle as authority,
- raw PTY/ConPTY handles,
- restart-surviving controller lease authority,
- full process environment,
- credentials or tokens,
- terminal transcript/history,
- arbitrary agent/model prose,
- Git verification or human-acceptance truth.

A persisted `LIVE_OWNED` row is historical metadata only. T147's Store reader refuses to return it as live even when the caller knows the matching owner-generation ID, because an ID is not the owner-held ownership primitive. A later separately authorized owner task must retain and independently prove the real ownership primitive before any live projection can exist.

## Reconciliation rule

When a previous persisted owner generation is no longer the explicitly proven current generation, reconciliation changes only prior `LIVE_OWNED` runtime rows to:

- ownership: `OWNERSHIP_LOST`,
- process liveness: `UNKNOWN`,
- endpoint availability: `UNKNOWN`,
- continuity: `UNKNOWN`,
- lifecycle event: `OWNERSHIP_LOST`,
- bounded recovery reason: `OWNER_GENERATION_CHANGED` or `OWNER_GENERATION_UNPROVEN`.

The prior owner-generation reference is retained as historical explanation. Rows tied to an explicitly supplied current generation may remain unchanged as persisted historical classification, but T147 still refuses to project them as live ownership. Reconciliation performs no process scan, PID attachment, PID signal, PID kill, or liveness-to-ownership upgrade.

## Schema integrity

T147 follows the existing transactional migration pattern:

1. inventory expected schema objects in an isolated in-memory SQLite database,
2. inventory observed schema objects,
3. fail closed on missing, unexpected, or definition-mismatched objects,
4. install migration 0013 inside `BEGIN IMMEDIATE`,
5. validate before commit,
6. rollback on installation or validation failure,
7. validate the existing schema again on every reopen.

Partial or corrupt 0013 schema is not silently repaired.

Canonical-reference foreign keys and cross-reference scope triggers prevent invalid workspace/session/terminal bindings. Runtime deletion has no reverse cascade into canonical workflow, execution, or terminal truth.

## Frozen predecessor migrations

Pre-implementation and post-implementation SHA-256 values remain:

- `migrations/0011_model_mesh_continuity.sql`: `9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d`
- `migrations/0012_desktop_presentation.sql`: `b1972c19531fdc3d358abdaf1d948037cd7067e9eb49717ff3b5211ddc971f41`

No dependency or lockfile change was made.

## Local deterministic evidence

Before candidate commit:

- `git diff --check`: PASS
- `cargo fmt --all -- --check`: PASS
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS
- focused T147 tests: 11 passed / 0 failed in the library target and 11 passed / 0 failed in the binary target
- full `cargo test --locked --all-targets --all-features -- --test-threads=1`: 1360 passed / 0 failed / 9 ignored across all targets

The nine ignored tests are pre-existing explicitly governed live-proof or soak tests; T147 does not convert them into passing evidence.

## Review repair history

The first published candidate head `6a15373491075d1cde2b87c636f9df0c95ddf2ac` received a fresh CodeRabbit review that identified one material schema-integrity gap: the schema inventory selected T147 indexes/triggers by naming convention, so a differently named trigger or index attached directly to a persistent-runtime table could evade the unexpected-object check.

The repair broadens observed inventory to every non-internal SQLite schema object whose `tbl_name` is either T147 table, while retaining the exact isolated expected-inventory comparison. A dedicated differently named trigger fixture now proves reopening fails closed. The finding is preserved rather than relabelled as passing evidence; the repaired successor head requires fresh exact-head qualification and review.

## Nonclaims

T147 does not establish any of the following:

- a live persistent-runtime owner process,
- endpoint availability,
- PID/process reattachment,
- controller lease mechanics,
- PTY/ConPTY handoff,
- provider execution,
- remote/public control,
- Git landing authority,
- human acceptance,
- `T079_LIVE_PASS`,
- `T080_LIVE_PASS`,
- `T082_WORKER_LIVE_PASS`,
- `REAL_CLAUDE_EXECUTION`,
- `REAL_CODEX_WORKER_EXECUTION`.

Exact-head GitHub CI, independent review, guarded merge identity, and post-merge workflow evidence are candidate-bound gates and must be recorded from live GitHub truth after this document is committed.
