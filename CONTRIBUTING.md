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

Feature-specific tasks may require additional focused tests, fault injection, platform jobs, real-environment integration, release-bundle checks, or an explicit soak. Follow the active `tasks.md` rather than inventing replacement gates.

For Spec 003 workspace-terminal changes, a local Unix pass alone is not sufficient when the touched surface includes native Windows/ConPTY or WSL behavior. Preserve the accepted official-Windows touched-surface gate and require real Windows+WSL2 evidence before making or widening a WSL support claim.

## Safety invariants

Changes must preserve the current product boundary unless an active specification explicitly changes it.

For the authoritative verification path:

- do not trust agent-reported success as promotion evidence;
- do not mutate the primary checkout while verifying/promoting a candidate;
- do not force-clean or force-remove candidate worktrees;
- do not recursively delete ambiguously owned paths;
- do not auto-select a winner;
- do not add merge/rebase/cherry-pick/push/PR automation as verification product behavior without explicit specification authorization;
- do not describe worktree isolation as OS/network/secret sandboxing.

When a verification safety state is ambiguous, prefer a bounded failure or `MANUAL_RECOVERY_REQUIRED` over automatic repair.

### Spec 003 workspace-terminal invariants

Workspace-terminal activity has a deliberately different mutation boundary from `winds verify`, but it must not weaken verification authority.

When changing the accepted Spec 003 surface:

- treat workspace terminals and explicit `winds run` commands as user-directed activity that may mutate the primary checkout;
- keep workspace execution/history records separate from candidate eligibility, promotion authority, and verification evidence;
- preserve exact workspace, execution-domain, executable/profile, and cwd identity instead of relying on UX display names;
- do not auto-execute repository environment/bootstrap configuration merely to open or inventory a workspace;
- keep native Windows and each WSL distribution as distinct execution domains and expose mapping/identity mismatch rather than inventing equivalence;
- treat PTY/ConPTY ownership as lifecycle ownership only for resources Winds can prove it owns, not as a sandbox or proof of descendant ownership;
- after restart, reconcile an unprovably owned terminal to `OWNERSHIP_LOST`/unknown rather than trusting a stored PID or blindly signaling it;
- do not scrape arbitrary PTY keystrokes to invent exact command history;
- keep shell-reported telemetry source-labeled unless Winds independently proves the fact through an accepted observation path;
- do not persist the full process environment or claim perfect secret detection;
- keep native-Windows authoritative `verify`/`promote` required-check execution fail-closed unless a later specification separately proves and authorizes it.

The detailed user-facing boundary is documented in [`specs/003-workspace-execution-spine/terminal-trust-boundary.md`](specs/003-workspace-execution-spine/terminal-trust-boundary.md).

## Simplicity gate

Winds uses the Ponytail review discipline: delete speculative abstractions and prefer the smallest implementation that proves the active requirement.

A new daemon, public protocol, plugin/provider system, sandbox framework, orchestration layer, generic runtime abstraction, terminal renderer, persistent detached-session owner, or large dependency needs explicit specification-level justification. “We may need it later” is not sufficient.

For Spec 003 specifically, do not smuggle SQL Studio, LLM Observatory, Agent Fleet, MCP/ACP/A2A, remote execution, a custom multiplexer/renderer, or environment-manager machinery into a workspace-terminal change.

## External code and provenance

Before copying or adapting code from another project, update [`docs/provenance/donors.md`](docs/provenance/donors.md) with:

- exact upstream repository and path;
- exact source commit/tag;
- upstream license;
- what was copied or adapted;
- Winds modifications;
- update strategy.

Ordinary package dependencies also need compatible license/provenance treatment for release artifacts. Do not paste third-party code first and reconcile provenance later.

The current direct PTY dependency has a dedicated exact-lock audit at [`docs/provenance/portable-pty-0.9.0-lock-audit.md`](docs/provenance/portable-pty-0.9.0-lock-audit.md). A future dependency-graph change must not silently reuse that audit as evidence for a different lock graph.

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

Documentation changes are subject to the same authority rule: distinguish released behavior, accepted-but-unreleased `main` behavior, future design, and explicit non-goals. A README claim can widen user expectations just as effectively as code, so platform/security/support language must be backed by the active task evidence.

## Security reports

Do not open a public issue containing vulnerability details. Follow [`SECURITY.md`](SECURITY.md) for the coordinated-disclosure path.

## License of contributions

Unless explicitly stated otherwise, contributions intentionally submitted to Winds are accepted under the same dual license as Winds-authored source: **MIT OR Apache-2.0**.
