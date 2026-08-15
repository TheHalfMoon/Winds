# Contributing to Winds

Winds is built around a small trust boundary, so contribution quality is measured by reproducible evidence and preserved invariants rather than by implementation volume.

## Before changing code

Read, in order:

1. [`AGENTS.md`](AGENTS.md)
2. [`.specify/memory/constitution.md`](.specify/memory/constitution.md)
3. the active feature's `spec.md`
4. its `plan.md`
5. its `tasks.md`

Repository canonical documents outrank chat, agent memory, or reviewer summaries.

Do not begin product work that is not authorized by an active task. If the desired behavior is outside the current specification, amend the specification first rather than hiding scope expansion inside implementation.

## Required development sequence

Every accepted implementation slice follows:

```text
Constitution
    ↓
Specification
    ↓
Plan
    ↓
Tasks
    ↓
Implementation
    ↓
Deterministic checks
    ↓
Correctness / safety review
    ↓
Ponytail simplicity review
    ↓
Independent review
    ↓
Evidence reconciliation
```

A passing agent message is not test evidence. A review bound only to an older commit is not final-head evidence.

## Local deterministic checks

Use the pinned toolchain and committed lockfile:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Feature-specific tasks may require additional focused tests, fault injection, or an explicit pre-release soak. Follow the active `tasks.md` rather than inventing replacement gates.

## Safety invariants

Changes must preserve the current product boundary unless an active specification explicitly changes it.

In particular:

- do not trust agent-reported success as promotion evidence;
- do not mutate the primary checkout while verifying/promoting a candidate;
- do not force-clean or force-remove candidate worktrees;
- do not recursively delete ambiguously owned paths;
- do not auto-select a winner;
- do not add merge/rebase/cherry-pick/push/PR automation as 0.1 product behavior;
- do not describe worktree isolation as OS/network/secret sandboxing.

When a safety state is ambiguous, prefer a bounded failure or `MANUAL_RECOVERY_REQUIRED` over automatic repair.

## Simplicity gate

Winds uses the Ponytail review discipline: delete speculative abstractions and prefer the smallest implementation that proves the active requirement.

A new daemon, public protocol, plugin system, sandbox framework, orchestration layer, generic runtime abstraction, or large dependency needs explicit specification-level justification. “We may need it later” is not sufficient.

## External code and provenance

Before copying or adapting code from another project, update [`docs/provenance/donors.md`](docs/provenance/donors.md) with:

- exact upstream repository and path;
- exact source commit/tag;
- upstream license;
- what was copied or adapted;
- Winds modifications;
- update strategy.

Ordinary package dependencies also need compatible license/provenance treatment for release artifacts. Do not paste third-party code first and reconcile provenance later.

## Pull requests

Keep pull requests bounded to an authorized task or tightly coupled task group. The PR description should state:

- exact scope;
- relevant spec/task IDs;
- important non-goals;
- deterministic evidence on the current head;
- correctness/safety and simplicity review status;
- independent-review status;
- any remaining gate.

Do not mark a task complete because implementation merely exists. Update task truth only after the evidence required by that task exists.

## Security reports

Do not open a public issue containing vulnerability details. Follow [`SECURITY.md`](SECURITY.md) for the coordinated-disclosure path.

## License of contributions

Unless explicitly stated otherwise, contributions intentionally submitted to Winds are accepted under the same dual license as Winds-authored source: **MIT OR Apache-2.0**.
