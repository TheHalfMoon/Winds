# Winds

**Independent verification runtime for agent-generated software.**

Coding agents are good at producing changes. Winds is for the separate question: **what can we independently prove about an exact candidate before a human selects it?**

Winds resolves exact Git snapshots, verifies them in detached candidate worktrees, runs repository-owned checks independently of the authoring agent, records content-addressed evidence with explicit authority, and promotes only an explicitly selected verified snapshot to a dedicated Winds ref—without mutating the primary checkout.

Winds 0.1 is intentionally small. It is a verification boundary, not another coding agent and not a general sandbox or merge bot.

## The 0.1 contract

Winds exposes three commands:

```text
winds verify  --repo PATH --base REF --candidate REF --check COMMAND [--timeout-secs N] [--home PATH]
winds promote --repo PATH --run RUN_ID [--home PATH]
winds recover --repo PATH [--home PATH]
```

### `winds verify`

- requires a clean primary checkout, including visible non-ignored untracked files;
- resolves the base and candidate to exact Git object IDs;
- creates a detached, locked candidate worktree outside the source checkout and Git common directory;
- runs the explicit repository check with a timeout and bounded output capture;
- records an immutable Evidence Report bound to the exact candidate/check definition;
- returns `ELIGIBLE` only when the observed candidate still matches the verified snapshot and the required check passes cleanly.

### `winds promote`

- accepts an explicit caller request for one previously `ELIGIBLE` run;
- revalidates candidate identity and cleanliness;
- reruns the original required check;
- persists fresh promotion-recheck evidence;
- creates `refs/heads/winds/selected/<run-id>` at the exact verified candidate commit.

Promotion does **not** merge, rebase, cherry-pick, push, open a pull request, or switch/reset the primary checkout.

### `winds recover`

Reconciles persisted Winds workspace metadata against Git's machine-readable worktree inventory. Ambiguous, missing, dirty, mismatched, or interrupted state fails closed as `MANUAL_RECOVERY_REQUIRED`; Winds does not auto-adopt or force-delete uncertain state.

## Evidence authorities

Winds keeps authority explicit:

- **`AGENT_REPORTED`** — a claim originating from an authoring agent. It is not authoritative verification evidence.
- **`WINDS_OBSERVED`** — Git/process/check facts observed directly by Winds.
- **`HUMAN_DECIDED`** — explicit human selection or policy decisions.

The current CLI records promotion intent as `CALLER_REQUESTED`; it does not claim that CLI possession proves an authenticated human identity. Promotion policy is based on Winds-observed evidence plus explicit selection, never on an agent saying its own work passed.

## Quick start from source

### Requirements

- Linux, macOS, or WSL2 operating in its Linux domain;
- Git 2.36 or newer;
- Rust 1.97.1 (also pinned by `rust-toolchain.toml`).

Native Windows execution semantics are **not** part of the 0.1 support claim.

Build with the committed lockfile:

```bash
cargo build --locked --release
```

Run the deterministic test suite:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The binary is then available at `target/release/winds`.

### Verify and select a candidate

Start from a clean repository containing existing refs/commits for the base and candidate:

```bash
./target/release/winds verify \
  --repo . \
  --base main \
  --candidate my-candidate \
  --check 'cargo test --locked'
```

The JSON output contains a `run_id`, exact base/candidate object IDs, observed check evidence, and eligibility. If the result is `ELIGIBLE`, explicitly select that exact run:

```bash
./target/release/winds promote --repo . --run '<RUN_ID>'
```

Inspect/reconcile Winds-owned candidate worktrees at any time:

```bash
./target/release/winds recover --repo .
```

By default Winds stores persistent state under `$HOME/.winds`. `WINDS_HOME` or `--home` may choose another path, but Winds rejects persistent state located inside the repository checkout or Git common-directory boundary.

## What Winds 0.1 proves

For the supported walking-skeleton flow, Winds has deterministic fixtures and CI evidence covering exact candidate identity, dirty-source rejection, hostile inherited Git context, timeout/failure handling, bounded output, candidate mutation, evidence corruption, stale evidence, worktree reconciliation, promotion recheck, prohibited downstream Git operations, fault-injected partial transitions, and a 100-cycle create/verify/promote/reconcile soak with zero observed primary-checkout mutations.

See [`specs/001-verification-walking-skeleton/`](specs/001-verification-walking-skeleton/) for the as-built contract and [`specs/002-public-release-readiness/`](specs/002-public-release-readiness/) for release-readiness work.

## Safety boundary

A Winds worktree is **checkout/index isolation**, not a security sandbox. Winds 0.1 does not claim to provide:

- OS/process sandboxing;
- network isolation;
- secret or credential isolation;
- protection from arbitrary filesystem writes by hostile checks;
- hostile Git clean/smudge/filter containment;
- container, service, database, or port isolation;
- authenticated human identity from local CLI possession;
- automatic winner scoring or autonomous promotion;
- native Windows execution semantics.

Treat required checks as code running with the permissions of the user who launched Winds.

## Intentionally deferred

The 0.1 release does not include agent adapters, `winds race`, ACP/MCP/A2A, a daemon, terminal emulator, TUI/dashboard, generic plugin/runtime abstractions, Graphify/Jujutsu integration, broad sandboxing, package-manager installers, or automatic downstream Git/PR automation.

These are not hidden features waiting behind flags; they require new specifications and evidence before they become product scope.

## Development and security

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — spec-driven contribution and review workflow.
- [`SECURITY.md`](SECURITY.md) — supported security boundary and vulnerability reporting.
- [`docs/provenance/donors.md`](docs/provenance/donors.md) — material donor/process provenance.
- [`docs/release/license-audit.md`](docs/release/license-audit.md) — 0.1 dependency/license audit.
- [`CHANGELOG.md`](CHANGELOG.md) — release-facing change summary.

## License

Winds-authored source is available under your choice of the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE).

Third-party dependencies retain their own licenses and notice obligations; see the release licensing audit for the current dependency set.
