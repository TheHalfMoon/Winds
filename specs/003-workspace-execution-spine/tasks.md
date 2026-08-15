# Tasks: Workspace Execution Spine

This checklist records implementation/evidence truth for Spec 003. A checked item requires repository evidence on the accepted branch/head; nearby work does not imply completion. SQL Studio and LLM Observatory are explicit follow-on specifications and are not silently included by completing this checklist.

## Phase 1 - Canonical Baseline and Dependency Decision

- [x] **T042** Establish Spec 003 with `spec.md`, `plan.md`, `tasks.md`, and `research.md` on a dedicated branch rooted at canonical main `8e92c5612a9ddc32996ed5e08475e3c9baa5e161`. The specification explicitly authorizes the 0.2 workspace/PTY/native-Windows expansion while retaining the no-daemon/no-public-protocol/no-plugin-system boundary.
- [x] **T043** Complete the PTY dependency/provenance decision. `portable-pty` crate `0.9.0`, from `wezterm/wezterm` published-source commit `f8921727a11b9f8b073e8c24821d72fd41283500` path `pty/`, MIT, is accepted as the preferred first direct-dependency candidate. The audit covers required PTY/ConPTY API shape, Linux/macOS/Windows implementation direction, Rust 1.97.1 compatibility posture, direct/transitive-footprint risks, license/notice obligations, mandatory `serial2` footprint, and alternatives (`xpty`, `rust-pty`, `portable-pty-psmux`, Unix-only PTY crates). Evidence is recorded in `pty-dependency-decision.md` and `docs/provenance/donors.md`. No donor runtime code was copied and the crate is intentionally **not yet landed**: the first runtime PR that actually uses it must request exact `portable-pty = "=0.9.0"`, commit the Winds-resolved `Cargo.lock`, compile/clippy/test that exact graph under pinned Rust 1.97.1, and re-audit the exact locked transitive/license set. If those landing gates fail, the candidate decision must be reopened rather than silently worked around.

## Phase 2 - Workspace and Execution Persistence

- [x] **T044** Add a forward-only SQLite migration and store/domain types for a minimal workspace registry plus common execution ledger and typed terminal-session records. Keep existing candidate/evidence/promotion table semantics unchanged; do not add speculative SQL/LLM columns or a generic plugin payload. **Canonical evidence:** PR #10 merged as `870431c63790878f5fc7ddd60a14b14544557512`; exact implementation head `413b9a05ad1f98f8199012f784bf79690a093d58` passed quality #138, release-candidate #24, Ubuntu/macOS format+Clippy+tests, SC-001 100-cycle soak, Linux/macOS release builds, correctness/safety and Ponytail review, Qodo exact-head re-review with 0 bugs / 0 rule violations, and zero unresolved review threads. Post-merge quality #139 passed on exact canonical main `870431c63790878f5fc7ddd60a14b14544557512`. The accepted slice adds only `workspaces`, `executions`, `execution_events`, and `terminal_sessions` persistence plus typed Store/domain records and deterministic tests; it does not land `portable-pty`, PTY/terminal runtime, CLI, SQL/LLM-specific schema, Agent Fleet runtime, or changes to existing verify/promote/recover semantics.
- [ ] **T045** Implement existing-worktree open/inspect: canonical worktree root, Git common directory, exact HEAD, branch/detached state, deterministic dirty-state observation, and safe workspace registration outside the repository boundary. Reject non-Git, bare, missing, or ambiguous workspace identity.
- [ ] **T046** Implement explicit system-Git clone + registration with destination validation, failure-before-registration behavior, and credential-safe persisted remote identity. Do not auto-run repository environment/bootstrap configuration after clone and do not claim hostile-clone sandboxing.

## Phase 3 - Environment and Shell Profiles

- [ ] **T047** Implement non-executing workspace environment inventory: OS/architecture, exact Git paths, discovered shell candidates, and presence of selected tool/environment manifests (`.mise.toml`, `.tool-versions`, `.python-version`, `.nvmrc`, `.envrc`, `rust-toolchain.toml`, devcontainer config). Prove no manifest/bootstrap execution and no full environment-value persistence.
- [ ] **T048** Implement concrete shell-profile discovery for Linux/macOS and native Windows with exact executable/argument/domain identity. Display names are UX only; launch identity must be explicit and validated at use time.
- [ ] **T049** Implement WSL distribution discovery on Windows through supported `wsl.exe` machine-usable behavior. Record distribution identity/version/state when available and fail explicitly when WSL discovery is unavailable or ambiguous.

## Phase 4 - PTY Terminal Lifecycle

- [ ] **T050** Implement the smallest PTY-backed interactive terminal controller on Linux/macOS using the accepted T043 dependency/primitive. Support exact session identity, start cwd, input/output streaming, resize, interrupt, observed exit, terminate/close, and reaping only for resources/processes Winds can prove it owns.
- [ ] **T051** Extend the terminal controller to native Windows using the accepted pseudoconsole/ConPTY path without weakening existing Unix process/check safety. Add compile/test gates on an official Windows runner before claiming native Windows workspace/terminal support.
- [ ] **T052** Implement explicit WSL terminal launch by selected distribution/profile. Validate effective cwd and repository identity after launch when a workspace path is mapped; expose/fail safe on mapping mismatch rather than silently claiming equivalence.
- [ ] **T053** Persist terminal lifecycle transitions into the execution ledger, including start/final state/timing/domain/profile/cwd and explicit failed/interrupted states. After restart, any session whose continuing process ownership cannot be proven MUST reconcile to `OWNERSHIP_LOST` with process state unknown. A persisted PID alone is not process identity; recovery MUST NOT blindly signal/kill it. No daemon or cross-restart process attachment is allowed in this slice.

## Phase 5 - Command Observability and Privacy

- [ ] **T054** Add command-level records only for shells with a reliable ephemeral lifecycle integration or an explicit Winds-run command. Capture command/cwd/exit/duration where available without scraping arbitrary PTY keystrokes or editing persistent shell profiles. Hook-emitted command telemetry MUST remain explicitly shell-reported rather than `WINDS_OBSERVED` unless an independently protected Winds observation proves the fact; test marker spoof/confusion by child output.
- [ ] **T055** Add before/after lightweight Git observations around supported command boundaries where reliable, using system Git machine-readable state rather than recursive repository hashing. Interactive mutations remain workspace history and MUST NOT become verification eligibility evidence.
- [ ] **T056** Implement local history/transcript retention and secret-safety policy before broad automatic persistence: bounded storage/quota, truncation/retention metadata, sanitized clone URLs/launch metadata, no full environment values, and a clear way to disable command/transcript history for a session. Do not claim perfect secret detection.

## Phase 6 - Minimal User/CLI Proof Surface

- [ ] **T057** Add the minimum CLI surface needed to prove the backend without a GUI: open/inspect workspace, clone/register, list shell/execution profiles, launch a selected terminal/profile or focused interactive proof command, and inspect execution/session metadata in deterministic JSON. Keep command structure small and consistent with existing `winds` CLI behavior.
- [ ] **T058** Document the explicit workspace-terminal trust boundary: terminal commands run with the launching user's permissions and may mutate the primary checkout/network/filesystem/secrets; `winds verify` remains the isolated authoritative verification path.

## Phase 7 - Deterministic, Fault, Platform, and Soak Evidence

- [ ] **T059** Add deterministic negative fixtures for invalid/bare/symlinked workspaces, credential-bearing clone URLs, clone failure, disappearing shell executable, PTY allocation/start failure, immediate shell exit, huge output bound, and environment manifests that must not auto-execute.
- [ ] **T060** Add lifecycle race/fault fixtures covering input/resize concurrent with exit, interrupt/close escalation while ownership is proven, output-reader failure, SQLite failure before/after child spawn, partial execution persistence, stale ACTIVE session reconciliation to `OWNERSHIP_LOST`, explicit PID-reuse/no-blind-signal behavior, and shell-hook marker spoofing. No false-success, falsely-live, falsely-owned, or falsely-`WINDS_OBSERVED` record is acceptable.
- [ ] **T061** Add Linux/macOS terminal integration coverage and a native Windows CI job proving the Spec 003 touched surface. Do not weaken or skip existing 0.1 quality/safety checks solely to make Windows compile.
- [ ] **T062** Obtain real Windows+WSL2 integration evidence before claiming WSL support: distribution discovery, explicit Ubuntu/selected-distro launch, cwd mapping verification, Git identity verification, and visible mismatch behavior. A mocked command-construction test alone is insufficient for the support claim.
- [ ] **T063** Run a deterministic 100-cycle controlled terminal lifecycle soak: create -> command/input -> resize -> observe exit -> close/reap -> reconcile store. Require zero leaks of directly owned children during controlled lifecycle, zero falsely-live terminal rows, bounded retained output, and no corruption/regression of candidate/evidence/promotion tables. Unexpected-crash recovery is separately proven by `OWNERSHIP_LOST` truth rather than claiming unknown survivors can always be killed.
- [ ] **T064** Run the complete pre-existing `winds verify/promote/recover` deterministic suite on the exact implementation head and prove no regression in existing evidence authority, non-destructive candidate worktrees, promotion recheck, or recovery behavior.

## Phase 8 - Acceptance and Review

- [ ] **T065** Update README/CONTRIBUTING/SECURITY/relevant docs for accepted 0.2 workspace-terminal behavior only. Do not describe SQL Studio, LLM Observatory, persistent detached terminals, terminal renderer, or native-Windows verification as implemented unless separately proven.
- [ ] **T066** Complete correctness/safety review on the exact final implementation head, explicitly covering PTY/process ownership, stale PID reuse, Windows/Unix close semantics, WSL path/domain truth, SQLite partial transitions, shell-telemetry source attribution, secret/history persistence, and separation from verification authority.
- [ ] **T067** Complete Ponytail v4.9.0 simplicity review on the exact final implementation head. Challenge every dependency/module/protocol; remove custom multiplexer/renderer/plugin/provider/environment-manager machinery not required by Spec 003.
- [ ] **T068** Obtain and reconcile at least one independent reviewer pass on the exact final implementation head. External summaries or reviews bound only to older heads do not satisfy this task.
- [ ] **T069** Reconcile deterministic CI, platform/WSL evidence, soak results, correctness/safety, Ponytail, and independent-review findings into final canonical task truth before making the Spec 003 completion claim.

## Explicit Follow-On Specifications

These are product goals, **not hidden unchecked work inside Spec 003**:

- **Spec 004 - SQL Studio**: secret-safe connection profiles, schema/catalog intelligence, query editor/history, explicit transaction/write safety, dialect-aware parsing, timing/rows/results/export, EXPLAIN/plan evidence, and workspace-linked query executions.
- **Spec 005 - LLM Observatory**: provider/model traces, input/output/cache/reasoning tokens, total/streaming latency, retries/errors/rate limits, exact or unknown cost with pricing provenance, budgets/aggregates, tool/subagent spans, payload privacy, and OpenTelemetry-aligned export.
- **Later UI slice**: graphical workspace organization, panes/tabs, embedded terminal renderer, SQL data grid/plan visualization, and LLM timeline dashboards built on the proven backend records.
- **Later persistence slice**: detached live terminal sessions across Winds restarts only if real user need justifies an external PTY owner/daemon and a versioned reconnection protocol.
