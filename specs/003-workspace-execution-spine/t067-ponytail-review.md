# T067 Ponytail v4.9.0 Simplicity Review

## Status

**PASS — no required code/dependency/module removal identified.**

This review closes only the **review work** for Spec 003 T067. It does not mark `tasks.md`, does not satisfy T068 independent review, and does not make the T069 / Spec 003 completion claim.

## Review binding

- Repository: `TheHalfMoon/Winds`
- Canonical main at review start: `fa1a1793309936798409c12b411d8e4393d3df0e`
- Exact final Spec 003 implementation head reviewed: `8601b7dbb44582a284813bbd50a44aeb1afd24f1`
- Exact implementation tree reviewed: `1d056bead423f02c62ace10b798ceb5c1a1c191c`
- Ponytail process reference: `DietrichGebert/ponytail` v4.9.0, commit `0a4dd63ad4541f4f655c4108a295916f3c1d8fda`
- Spec/plan authority: `specs/003-workspace-execution-spine/spec.md` and `plan.md`

The implementation tree is the authoritative target for simplicity judgment. The later T066 closeout commit only records task truth and does not change the runtime tree under review.

## Ponytail questions applied

The review challenged the implementation in this order:

1. Can the capability be deleted because Spec 003 does not require it?
2. Can custom code be replaced by the standard library/platform?
3. Can an already-adopted dependency replace custom machinery?
4. Is a new dependency/module/protocol/framework justified by a concrete current use case?
5. Does any abstraction exist only for hypothetical SQL/LLM/Agent Fleet/plugin/remote-runtime work?
6. Can any simplification be made without weakening validation, process ownership, Git/data safety, evidence authority, WSL truth, recovery, privacy, or deterministic evidence?

## Findings

### P67-01 — direct dependency surface is already minimal and justified

`Cargo.toml` has six direct dependencies:

- `libc = "=0.2.189"`
- `portable-pty = "=0.9.0"`
- `rusqlite = { version = "=0.40.2", features = ["bundled"] }`
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`
- `sha2 = "0.10"`

Disposition: **retain all six**.

Rationale:

- `libc` is the smallest existing low-level primitive used by the existing required-check/process-group safety and by Unix PTY foreground-process-group ownership validation/SIGINT. Replacing it with custom FFI would be strictly more code and more risk.
- `portable-pty` is the exact-pinned T043 decision that avoids maintaining separate hand-written Unix PTY and Windows ConPTY implementations. Removing it would recreate precisely the platform abstraction Spec 003 chose not to build.
- `rusqlite` is the pre-existing local persistence primitive and is reused for the workspace/execution ledger and coordination instead of introducing a second database/lock framework.
- `serde` / `serde_json` are already used for typed deterministic JSON persistence/CLI output and execution-domain data. Replacing them would add hand-written serialization/parsing.
- `sha2` supplies deterministic collision-resistant identities/digests used by existing Git/workspace/history/ownership surfaces. A process-seeded or implementation-defined standard-library hash would not be a stable persisted identity format.

No async runtime, CLI framework, tracing framework, UUID crate, plugin SDK, RPC stack, ORM, terminal renderer, or extra locking dependency has been introduced.

### P67-02 — module seams are concrete, not a generic runtime framework

The final source tree contains concrete modules for current responsibilities: Git/check/store/domain, workspace open/clone/inventory, shell profiles, WSL discovery/launch, PTY terminal lifecycle, execution persistence, explicit-command/history behavior, CLI proof wiring, and deterministic acceptance/fault/soak tests.

Disposition: **retain current module boundaries**.

The following splits were challenged and retained because each maps directly to already-accepted Spec 003 work rather than hypothetical extensibility:

- `workspace.rs`, `workspace_clone.rs`, `workspace_inventory.rs`
- `shell_profiles.rs`
- `wsl.rs`, `wsl_launch.rs`
- `terminal.rs`
- `execution.rs`
- `command.rs`, `command/history.rs`
- `store_git_observation.rs`
- T059/T060/T063 and Windows acceptance-test modules

Folding these back into `git.rs`, `store.rs`, or `main.rs` would reduce file count but increase responsibility mixing and make safety-critical platform/persistence behavior harder to review. Splitting them further behind interfaces/traits/services would add abstraction with no current consumer. The current concrete-file shape is the smaller design.

### P67-03 — no custom plugin/provider/runtime protocol abstraction exists

Repository/source inspection found no product implementation of:

- daemon / `windsd`;
- public IPC or RPC protocol;
- TCP/Unix socket server;
- generic plugin/provider runtime;
- MCP/ACP/A2A runtime;
- remote execution service;
- terminal multiplexer;
- custom VT/terminal renderer;
- SQL provider abstraction;
- LLM provider/router abstraction;
- Agent Fleet/Herdr/Pi runtime transplant.

Disposition: **PASS**.

Donor/research material remains documentation-only and does not create runtime authority or an implementation dependency.

### P67-04 — no custom trait framework to delete

A source search found no custom public trait layer for workspace, terminal, execution, provider, plugin, or runtime dispatch. Dynamic trait objects in the PTY path are concrete `portable-pty` / standard I/O objects required by that dependency's API rather than a Winds-created extensibility framework.

Disposition: **PASS**.

### P67-05 — module-wide `dead_code` allowances were challenged and retained

`main.rs` allows `dead_code` for the concrete `command` and `execution` backend modules. `git.rs` similarly allows it for PTY/WSL launch backend modules.

This initially looked like possible speculative surface. Review against the accepted Spec 003 requirements shows that the relevant backend operations are intentionally broader than the minimal T057 CLI proof surface: PTY/session lifecycle includes input, output, resize, interrupt/terminate/close and exact ownership behavior even when every operation is not separately exposed as a stable public CLI command.

Disposition: **retain the module-level allowance; no T067 code change**.

Replacing the module-level allowance with numerous item-level lint exceptions would add annotation ceremony without deleting runtime code, dependencies, states, or abstractions. Deleting the underlying lifecycle operations merely to make the binary's minimal CLI call graph smaller would violate the already-accepted terminal lifecycle requirements and would remove deterministic safety/fault coverage.

Two historical reason strings in `git.rs` still say persistence/CLI callers “land in T053/T057”; those tasks have since landed. That wording is **advisory documentation debt**, not an over-engineering blocker and not a reason to create a runtime change in T067. It can be refreshed opportunistically when those module declarations next change.

### P67-06 — the T066 ownership-lease repair is complex but irreducible under current safety invariants

The per-execution retained SQLite coordination lease was challenged because it adds lifecycle machinery.

Disposition: **retain**.

Why a simpler-looking replacement is rejected:

- removing cross-process ownership proof reopens stale/live reconciliation races found during T066;
- unlinking/recreating a held lease path reopens pathname/inode split-brain;
- one global lease would serialize unrelated executions and create avoidable coupling;
- hand-written OS file-lock code or a new lock dependency adds a second platform primitive and a new dependency surface;
- blind PID-based recovery is explicitly forbidden because PID reuse is not ownership proof.

The accepted design reuses the existing SQLite dependency, keeps leases private/local, uses exact execution identity, and grants no process-signaling authority. Its complexity is safety-driven rather than speculative.

### P67-07 — persistence remains typed; no speculative SQL/LLM schema was smuggled into Spec 003

The migration sequence contains concrete workspace, execution, terminal, shell-command, clone-origin, and Git-observation data required by Spec 003. No nullable generic payload table or SQL/LLM/provider/plugin schema was added for future work.

Disposition: **PASS**.

Future SQL Studio and LLM Observatory remain follow-on specifications rather than extension points hidden in the current runtime.

### P67-08 — large concrete files are not a reason to create more abstraction

`store.rs`, `command.rs`, `command/history.rs`, and `wsl_launch.rs` are relatively large concrete modules.

Disposition: **no split/refactor required by T067**.

A size-only refactor would move code without reducing states, dependencies, runtime behavior, or safety surface. New repositories/services/interfaces/build crates are explicitly worse under Ponytail unless a present correctness or ownership boundary requires them. Existing extracted modules already isolate the strongest concrete seams.

## Deletion / dependency / abstraction ledger

| Candidate | Decision | Reason |
|---|---|---|
| `libc` | KEEP | Smaller/safer than custom Unix FFI; required for current process-group safety |
| `portable-pty` | KEEP | Exact accepted cross-platform PTY/ConPTY primitive; avoids custom implementations |
| `rusqlite` | KEEP | Existing single local persistence/coordination primitive |
| `serde` | KEEP | Typed serialization already required |
| `serde_json` | KEEP | Deterministic CLI/persistence JSON already required |
| `sha2` | KEEP | Stable collision-resistant persisted identities/digests |
| `workspace_*` modules | KEEP | Concrete current responsibilities |
| `wsl*` modules | KEEP | Concrete Windows/WSL current support |
| `terminal` / `execution` | KEEP | Required lifecycle + persistence boundaries |
| `command/history` | KEEP | Current bounded local-history/privacy behavior |
| module-level `dead_code` allowances | KEEP | Required backend surface exceeds deliberately minimal proof CLI; per-item allowances add ceremony only |
| daemon / public protocol / plugin/provider layer | ABSENT | Correctly not built |
| renderer / multiplexer | ABSENT | Correctly deferred |
| SQL/LLM runtime abstractions | ABSENT | Correctly deferred |
| Agent Fleet/Herdr/Pi runtime | ABSENT | Research/donor-only |

## Final Ponytail verdict

**T067_REVIEW_PASS_NO_REQUIRED_REMOVALS**

The final Spec 003 implementation is not minimal in line count, but its remaining complexity tracks concrete safety, cross-platform terminal, WSL truth, persistence, history/privacy, and verification-authority requirements. No dependency, module, protocol, service boundary, generic interface, or runtime subsystem can be removed today without either deleting an accepted requirement or replacing existing concrete code with more framework/platform machinery.

The correct Ponytail action is therefore **not to refactor for aesthetics**.

## Boundaries preserved

This review does **not**:

- start or satisfy T068 independent review;
- start or satisfy T069 final evidence reconciliation;
- claim Spec 003 complete;
- authorize daemon/server/socket/public protocol/plugin/provider/MCP/ACP/A2A behavior;
- authorize Agent Fleet/Herdr/Pi runtime work;
- authorize SQL Studio or LLM Observatory implementation;
- broaden native-Windows authoritative `winds verify` support;
- change runtime code, dependencies, migrations, workflows, CLI behavior, or verification authority.
