# Changelog

This changelog records release-facing product behavior. Exact CI/review run identifiers remain in pull-request/release evidence so this file does not become stale operational metadata.

## 0.1.0 - Unreleased

### Added

- Exact Git base/candidate resolution through the system Git executable.
- Clean-primary preflight that includes non-ignored untracked files even when repository config attempts to hide them.
- Detached, locked candidate worktrees outside the primary checkout and Git common directory.
- Explicit repository-owned required checks with timeout handling, bounded output capture, process-group termination, and bounded stream shutdown.
- SQLite-backed run/event/evidence/promotion state with append-only recovery-required observations.
- Immutable Evidence Reports bound to exact base/candidate identities and check definitions, with content-addressed SHA-256 output blobs.
- Explicit evidence authority separating Winds-observed facts from caller selection intent.
- Fresh candidate/check revalidation before promotion.
- Dedicated `winds/selected/<run-id>` refs for explicitly selected verified candidates without changing the primary checkout.
- Conservative recovery reconciliation that reports ambiguous, missing, dirty, mismatched, or interrupted state as `MANUAL_RECOVERY_REQUIRED` rather than auto-adopting or force-cleaning it.
- Cross-platform deterministic CI for the supported source/test surface on Ubuntu and macOS.
- Negative fixtures for prohibited downstream Git operations and fault-injection fixtures for partial Git/SQLite transitions.
- Explicit 100-cycle create/verify/promote/reconcile soak evidence with zero observed primary-checkout mutations.
- Public-release readiness metadata, dual `MIT OR Apache-2.0` source licensing, and dependency/provenance audit.

### Security and trust boundary

- Agent-reported success is not authoritative verification evidence.
- Worktree separation isolates checkout/index state; it is not an OS, network, process, filesystem, credential, or secret sandbox.
- Local CLI possession is not represented as authenticated human identity.
- Winds 0.1 does not automatically choose winners or integrate selected changes into a source branch.

### Explicitly not included

- merge, rebase, cherry-pick, push, or pull-request automation as product behavior;
- agent adapters or `winds race` orchestration;
- native Windows execution semantics;
- daemon/public runtime protocol, ACP/MCP/A2A, terminal emulator, TUI/dashboard;
- generic plugin system or sandbox framework;
- network/secret/container/service/database isolation;
- package-manager installers, crates.io publication, auto-update, signing/attestation infrastructure.

These deferred surfaces require new specifications and evidence rather than being implied by version 0.1.
