# Winds

**Independent verification runtime for agent-generated software.**

Coding agents are good at producing changes. Winds is for the separate question: **what can we independently prove about an exact candidate before a human selects it?**

Winds resolves exact Git snapshots, verifies them in detached candidate worktrees, runs repository-owned checks independently of the authoring agent, records content-addressed evidence with explicit authority, and promotes only an explicitly selected verified snapshot to a dedicated Winds ref—without mutating the primary checkout.

The current public release is **v0.1.0**. Current `main` also contains the accepted implementation slices of Spec 003's workspace-execution spine: exact workspace registration, bounded environment/profile discovery, PTY/ConPTY terminal lifecycle mechanics, WSL execution-domain support, and a local typed execution ledger. That development surface does **not** make Spec 003 a released 0.2 product by itself and does not widen the existing verification authority model.

## The 0.1 verification contract

The authoritative verification path exposes three commands:

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

Reconciles persisted Winds candidate-worktree metadata against Git's machine-readable worktree inventory. Ambiguous, missing, dirty, mismatched, or interrupted state fails closed as `MANUAL_RECOVERY_REQUIRED`; Winds does not auto-adopt or force-delete uncertain state.

## Evidence authorities

Winds keeps authority explicit:

- **`AGENT_REPORTED`** — a claim originating from an authoring agent. It is not authoritative verification evidence.
- **`WINDS_OBSERVED`** — Git/process/check facts observed directly by Winds through an accepted observation path.
- **`HUMAN_DECIDED`** — explicit human selection or policy decisions.

The current CLI records promotion intent as `CALLER_REQUESTED`; it does not claim that CLI possession proves an authenticated human identity. Promotion policy is based on Winds-observed verification evidence plus explicit selection, never on an agent saying its own work passed.

Workspace execution records preserve the same distinction. Caller-entered commands/profile choices remain requested values, and persisted terminal or command history does not become verification evidence merely because Winds stores it.

## Workspace-terminal behavior on current `main`

Spec 003 adds a deliberately small, flat proof surface rather than a GUI terminal product or generic runtime protocol:

```text
winds workspace-open  --repo PATH [--home PATH]
winds workspace-clone --remote REMOTE --destination ABSOLUTE_PATH [--home PATH]
winds profiles        --repo PATH [--home PATH]
winds run             --repo PATH --execution-id ID --executable ABSOLUTE_EXECUTABLE [--args-json JSON] [--history command|disabled] [--home PATH]
winds terminal-proof  --repo PATH --execution-id ID --profile-id PROFILE_ID [--rows N] [--cols N] [--home PATH]
winds execution       --repo PATH --execution-id ID [--home PATH]
```

These commands are backend proof surfaces:

- **`workspace-open`** canonicalizes and registers an existing non-bare Git worktree with exact worktree/Git-common-directory identity and current Git observations.
- **`workspace-clone`** uses system Git to clone into an explicit absolute destination, persists credential-safe remote identity, keeps Winds state outside the repository boundary, and does not auto-execute project environment/bootstrap configuration.
- **`profiles`** reports safe workspace inventory plus concrete native shell launch profiles; on Windows it also reports WSL discovery status and distributions.
- **`run`** launches one explicit absolute executable with structured argv, records typed lifecycle facts plus lightweight before/after Git observations when available, and allows supported command-history persistence to be disabled with `--history disabled`.
- **`terminal-proof`** starts a selected **native** shell profile through the accepted PTY/ConPTY lifecycle path and performs a focused start/terminate/record proof. It is not a terminal renderer, multiplexer, or persistent detached session UI.
- **`execution`** returns deterministic JSON for one execution/session record and binds inspection to the requested canonical workspace.

The underlying Spec 003 backend also contains explicit selected-distribution WSL launch/path-identity mechanics. Real Windows Server 2025 + Ubuntu WSL2 integration evidence proves distribution discovery, mapped and fallback launch behavior, cwd/repository identity checks, and visible mapping mismatch. The current flat CLI should not be interpreted as a full interactive WSL terminal UX.

### Execution ledger and ownership truth

Workspace-terminal activity uses Winds' local SQLite/WAL store but remains separate from candidate verification authority.

Accepted behavior includes:

- stable workspace and execution identity;
- typed terminal-session and explicit-command records rather than generic plugin payloads;
- execution domain, profile/executable identity, requested cwd, lifecycle status, timing when proven, and source labels;
- bounded local history controls with transcript persistence default-off and no full process-environment persistence;
- conservative restart reconciliation: a session whose continuing process ownership cannot be proven becomes `OWNERSHIP_LOST` with process state unknown;
- no persisted PID is treated as sufficient process identity and no stale PID is blindly signaled or killed;
- terminal/command Git observations remain workspace history and cannot make a candidate `ELIGIBLE` or promotable.

See [`specs/003-workspace-execution-spine/terminal-trust-boundary.md`](specs/003-workspace-execution-spine/terminal-trust-boundary.md) for the explicit trust boundary.

## Platform claims

The verification and workspace-terminal surfaces intentionally have different platform claims.

### Verification path

The released 0.1 verification support claim remains Linux and macOS, plus WSL2 when Winds, Git, the repository, and required checks all run inside the Linux domain. On native Windows, authoritative required-check execution used by `winds verify` and `winds promote` is **not supported** and fails closed before verification/promotion mutation. Spec 003's native-Windows workspace-terminal evidence does not by itself certify `winds verify`, `winds promote`, or `winds recover` as supported native-Windows verification product surfaces.

### Workspace / terminal execution

Spec 003 has accepted platform evidence for:

- Linux x86-64 PTY lifecycle and integration;
- macOS arm64 PTY lifecycle and integration;
- native Windows workspace/ConPTY touched-surface behavior on official Windows runners;
- real Windows + Ubuntu WSL2 distribution discovery, selected-domain launch, cwd mapping, Git identity validation, and explicit mismatch behavior.

This is a workspace-terminal support claim, not a native-Windows verification claim.

## Quick start from source

### Requirements

- Git 2.36 or newer;
- Rust 1.97.1 (also pinned by `rust-toolchain.toml`);
- a platform compatible with the command surface you intend to use, as described above.

Build with the committed lockfile:

```bash
cargo build --locked --release
```

Run the default deterministic suite:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Feature acceptance uses additional platform, fault, soak, and release-candidate gates recorded in the active specification; the three commands above are not a replacement for those gates.

The binary is then available at `target/release/winds` (or the platform-equivalent executable path).

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

By default Winds stores persistent state under `$HOME/.winds`. `WINDS_HOME` or `--home` may choose another path, but Winds rejects persistent state located inside the relevant repository checkout or Git common-directory boundary.

## What Winds proves today

For the released 0.1 verification flow, Winds has deterministic fixtures and CI evidence covering exact candidate identity, dirty-source rejection, hostile inherited Git context, timeout/failure handling, bounded output, candidate mutation, evidence corruption, stale evidence, worktree reconciliation, promotion recheck, prohibited downstream Git operations, fault-injected partial transitions, and a 100-cycle create/verify/promote/reconcile soak with zero observed primary-checkout mutations.

For the accepted Spec 003 development surface on current `main`, repository evidence additionally covers exact workspace open/clone, safe environment/profile inventory, Unix PTY and Windows ConPTY lifecycle behavior, explicit WSL domain/path truth, durable execution/session state, command/Git observations, bounded privacy controls, negative/fault/race fixtures, a cross-platform 100-cycle terminal lifecycle soak, and a complete regression gate proving the pre-existing verification authority was not weakened.

See [`specs/001-verification-walking-skeleton/`](specs/001-verification-walking-skeleton/), [`specs/002-public-release-readiness/`](specs/002-public-release-readiness/), and [`specs/003-workspace-execution-spine/`](specs/003-workspace-execution-spine/) for canonical task/evidence truth.

## Safety boundary

A Winds verification worktree is **checkout/index isolation**, not a security sandbox. Workspace terminals and explicit `winds run` commands are even more direct: they execute with the permissions of the launching user and may intentionally mutate the primary checkout or access that user's filesystem, network, environment, and credentials.

Winds does not claim to provide:

- OS/process sandboxing;
- network isolation;
- secret or credential isolation;
- protection from arbitrary filesystem writes by launched commands or hostile checks;
- hostile Git clean/smudge/filter containment;
- container, service, database, or port isolation;
- authenticated human identity from local CLI possession;
- automatic winner scoring or autonomous promotion;
- cross-restart ownership of live terminal processes.

Treat required checks and workspace commands as code running with the permissions of the user who launched Winds. PTY/ConPTY ownership is lifecycle ownership for resources Winds can prove it owns; it is not descendant-process or security confinement.

## Intentionally deferred

Current Winds does not include:

- SQL Studio or SQL execution runtime;
- LLM Observatory, model/provider routing, or token/cost accounting runtime;
- a GUI terminal renderer or terminal emulator UI;
- persistent detached terminals or cross-restart live-session attachment;
- a daemon / `windsd`, public IPC/runtime protocol, or remote terminal service;
- generic plugin/provider runtime abstractions;
- ACP/MCP/A2A or Agent Fleet runtime behavior;
- broad OS sandboxing;
- automatic downstream merge/rebase/cherry-pick/push/PR automation;
- package-manager installers, auto-update, or signed-attestation infrastructure.

These are not hidden features waiting behind flags. They require new specifications and evidence before they become product scope.

## Development and security

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — spec-driven contribution and review workflow.
- [`SECURITY.md`](SECURITY.md) — supported security boundary and vulnerability reporting.
- [`specs/003-workspace-execution-spine/terminal-trust-boundary.md`](specs/003-workspace-execution-spine/terminal-trust-boundary.md) — workspace-terminal authority and isolation boundary.
- [`docs/provenance/donors.md`](docs/provenance/donors.md) — material donor/process provenance.
- [`docs/provenance/portable-pty-0.9.0-lock-audit.md`](docs/provenance/portable-pty-0.9.0-lock-audit.md) — exact Spec 003 PTY dependency lock/license audit.
- [`docs/release/license-audit.md`](docs/release/license-audit.md) — v0.1 release dependency/license audit.
- [`CHANGELOG.md`](CHANGELOG.md) — release-facing and unreleased change summary.

## License

Winds-authored source is available under your choice of the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE).

Third-party dependencies retain their own licenses and notice obligations; see the release and dependency-specific provenance audits for the applicable dependency set.
