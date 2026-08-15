# Feature Specification: Workspace Execution Spine

**Feature Branch**: `spec/003-workspace-execution-spine`

**Created**: 2026-08-15

**Status**: Authorized for specification, planning, and implementation

**Input**: Evolve Winds 0.2 toward a verification-native developer workspace where opening or cloning a repository gives the user first-class terminal control, easy host-shell/WSL switching, accurate workspace organization, and one local execution record that can later unify shell, SQL, and LLM activity including time, tokens, cost, and exact workspace identity.

## Product North Star

Winds should answer five questions for every meaningful execution:

1. **Where did it run?** Exact workspace, repository identity, execution domain, shell/database/model endpoint, and working directory.
2. **What ran?** Command now; typed SQL query and LLM request in follow-on specifications.
3. **What happened?** Exit/result state, bounded artifacts/output, errors, and before/after repository observations when available.
4. **How long and how much did it consume?** Duration for every execution; SQL/database timing and LLM token/latency/cost metrics in their typed domain records.
5. **What authority does the record have?** Winds-observed process/repository facts remain distinct from caller input, shell-reported telemetry, model claims, and explicit human decisions.

The goal is not to become a generic IDE with unrelated tabs. The goal is a **verification-native workspace** in which terminal, SQL, and LLM work can share exact workspace identity and a trustworthy local execution timeline without weakening the existing `winds verify` evidence model.

## User Scenarios & Testing

### User Story 1 - Open or Clone a Repository Into an Exact Workspace (Priority: P1)

A developer can open an existing Git worktree or clone a repository into Winds and immediately see the exact repository root, current HEAD, branch/detached state, dirty-state summary, platform/execution domain, and available shell profiles.

**Why this priority**: Every terminal, SQL, LLM, and verification action needs a trustworthy workspace identity before Winds can organize or measure anything else.

**Independent Test**: Open a fixture repository through a symlinked path and through its canonical path, then clone the same fixture into a new destination. Confirm Winds resolves one canonical worktree identity per destination, records exact Git identity, does not place persistent Winds state inside the repository, and does not automatically execute repository tool/environment configuration.

**Acceptance Scenarios**:

1. **Given** an existing non-bare Git worktree, **When** the user opens it in Winds, **Then** Winds records its canonical worktree root, Git common directory, HEAD OID, branch or detached state, and current dirty-state observation.
2. **Given** a Git remote and explicit destination, **When** the user asks Winds to clone it, **Then** Winds uses system Git, records the resulting exact repository identity, and sanitizes persisted/displayed remote identity so credentials are not stored in the workspace ledger.
3. **Given** repository files such as `.envrc`, `.mise.toml`, `devcontainer.json`, shell rc files, or other executable environment configuration, **When** a workspace is first opened, **Then** Winds may detect their presence but MUST NOT execute or trust them automatically.
4. **Given** a non-Git directory, bare repository, missing path, or ambiguous worktree root, **When** open is requested, **Then** Winds fails explicitly rather than inventing a Git workspace identity.

### User Story 2 - Use a Real Interactive Terminal With Full Lifecycle Control (Priority: P1)

A developer can start an interactive shell in the workspace and Winds owns a real PTY/ConPTY session that supports input, output streaming, resize, interrupt, termination, and explicit close semantics.

**Why this priority**: A pipe-backed command runner cannot provide the terminal behavior required by shells, REPLs, TUIs, debuggers, database clients, and agent CLIs.

**Independent Test**: Launch supported shells under a PTY, run an interactive command, resize the terminal, send interrupt, verify exit behavior, then close the session. Repeat with a full-screen/alternate-screen fixture and confirm no directly owned child session is silently orphaned during normal close.

**Acceptance Scenarios**:

1. **Given** a supported shell profile, **When** the user starts a terminal, **Then** the shell receives a real terminal device and starts in the requested mapped workspace directory.
2. **Given** an active terminal, **When** input, resize, interrupt, or terminate is requested, **Then** Winds applies the operation to that exact terminal session and records the observed result.
3. **Given** a shell or child program that exits, **When** Winds observes termination, **Then** the session has one final observed state and duration rather than remaining falsely active.
4. **Given** a Winds process crash or forced exit, **When** the workspace is opened again, **Then** persisted sessions that Winds can no longer prove it owns are reconciled as `OWNERSHIP_LOST`/interrupted with process state explicitly unknown. Winds MUST NOT claim those processes are live or dead and MUST NOT blindly signal a persisted PID that may have been reused.

### User Story 3 - Switch Easily Between Host Shells and WSL/Ubuntu (Priority: P1)

On a machine with multiple execution environments, especially Windows with WSL2, a developer can choose another shell or WSL distribution without manually reconstructing the repository path or losing track of which environment is active.

**Why this priority**: Real projects routinely require PowerShell/cmd on Windows and Bash/Zsh/Linux tooling inside WSL. Winds should make the boundary explicit and easy rather than hiding it.

**Independent Test**: On Windows with WSL2 installed, discover installed distributions, start one host shell and one Ubuntu/WSL shell against the same logically accessible repository, and verify each session reports its exact execution domain, shell executable, initial working directory, and Git HEAD. If path mapping is not safe/available, verify Winds reports that fact and does not fake equivalence.

**Acceptance Scenarios**:

1. **Given** multiple discovered shell profiles, **When** the user switches profile, **Then** Winds creates a new explicit terminal session with the selected executable/domain rather than pretending to transform the already-running shell process.
2. **Given** Windows with WSL installed, **When** Winds discovers WSL, **Then** installed distributions are obtained through the supported WSL command surface and are represented separately from native Windows shells.
3. **Given** a workspace path that can be mapped into a selected WSL distribution, **When** the WSL shell is started, **Then** it starts at the mapped workspace path and the mapping result is recorded.
4. **Given** a workspace path that cannot be mapped safely, **When** the WSL shell is started, **Then** Winds uses an explicit fallback location and surfaces the mismatch; it MUST NOT silently claim the same working directory.

### User Story 4 - Inspect a Trustworthy Local Execution Timeline (Priority: P1)

A developer can inspect when a terminal session started, where it ran, which shell/domain it used, how long it lived, how it ended, and—when reliable shell integration is available—which commands completed with which exit code and duration.

**Why this priority**: The execution ledger is the common spine that later allows SQL and LLM activity to live in the same workspace without inventing incompatible logging systems.

**Independent Test**: Run terminal sessions and supported shell command fixtures, inspect the SQLite-backed records, and prove that timestamps/status/domain/cwd/repository observations have explicit sources: direct Winds process/Git observations are distinguishable from caller-entered intent and shell-reported command telemetry.

**Acceptance Scenarios**:

1. **Given** a terminal session, **When** it starts and ends, **Then** Winds records stable session identity, workspace identity, execution domain, shell profile, start/end timestamps, duration, and observed final state.
2. **Given** a shell for which a reliable non-invasive lifecycle hook is implemented, **When** commands run, **Then** Winds may record command text, cwd, exit code, and duration as shell-reported command telemetry without inferring commands from arbitrary raw keystrokes. Hook-reported facts are not `WINDS_OBSERVED` unless Winds has an independently protected observation channel that proves them.
3. **Given** a command that changes the repository, **When** before/after Git observations are available, **Then** Winds records the directly observed change in workspace state but does not represent the interactive command as verified candidate evidence.
4. **Given** unavailable/ambiguous lifecycle information, **When** an execution record is shown, **Then** missing facts remain unknown rather than synthesized.

### User Story 5 - Understand the Environment Without Auto-Executing It (Priority: P2)

A developer can see the operating system, architecture, relevant shell profiles, WSL distributions, Git identity, and detected project tool/environment manifests without Winds silently running untrusted project configuration.

**Why this priority**: A well-organized workspace must explain its environment while preserving an explicit trust boundary.

**Independent Test**: Open fixtures containing `.mise.toml`, `.tool-versions`, `.python-version`, `.nvmrc`, `.envrc`, `rust-toolchain.toml`, and `devcontainer.json`; confirm Winds inventories file presence and safe metadata only and does not execute those files or persist secret environment values.

### User Story 6 - Grow Into SQL and LLM Work Without Rebuilding the Foundation (Priority: P2)

A developer later using Winds SQL or LLM surfaces should see those executions in the same workspace timeline, with domain-specific details rather than a generic untyped blob.

**Why this priority**: SQL and LLM are explicit product goals, but implementing them before workspace/session identity is proven would duplicate state, timing, secret handling, and history infrastructure.

**Independent Test for this feature**: Inspect the execution data model and migration strategy and confirm shell records use a common execution identity plus a typed terminal record, while future SQL/LLM data can be added through typed child tables without putting token/query/provider fields into the shell schema.

**Acceptance Scenarios**:

1. **Given** the Spec 003 schema, **When** future SQL support is designed, **Then** it can reuse workspace/execution identity while storing database-specific facts in a typed SQL record.
2. **Given** the Spec 003 schema, **When** future LLM support is designed, **Then** it can reuse workspace/execution identity while storing provider/model/token/cost/latency facts in a typed LLM record.
3. **Given** shell-only Spec 003 implementation, **When** a user inspects it, **Then** Winds does not pretend SQL or LLM execution is already implemented.

### Edge Cases

- Repository paths contain spaces, Unicode, symlinks, junctions, or case variations.
- The opened path is a linked worktree whose Git common directory is elsewhere.
- Clone authentication is interactive or the remote URL contains user-info/query material.
- Git clone/checkout honors user-level Git behavior such as credential helpers or filters; Winds must not overclaim hostile-checkout isolation.
- A shell executable disappears after discovery or exits before integration is ready.
- A PTY child forks, changes process groups, runs a TUI/alternate screen, emits invalid UTF-8, emits very large output, or ignores a polite interrupt.
- Resize/input/exit happen concurrently.
- A child survives an unexpected Winds crash; the old PID may later be reused by an unrelated process, so restart reconciliation cannot use blind PID signaling as proof or cleanup.
- The machine has no WSL, multiple WSL distributions, a stopped distribution, WSL1 plus WSL2, or a distribution name containing spaces.
- A Windows path maps to `/mnt/<drive>` but the repository is actually stored in a WSL-native filesystem, or vice versa.
- A shell changes cwd independently of the workspace root.
- Shell command hooks are unsupported, disabled, altered by user configuration, or spoofed by child output.
- Command text/output contains credentials or secrets.
- SQLite write fails while a terminal remains active.
- Winds exits while terminal records are partially persisted.
- The workspace is dirty before a terminal starts and is dirtier/cleaner afterward.
- An interactive command intentionally mutates the primary checkout; this is user workspace behavior, not a violation of the isolated `winds verify` invariant.

## Requirements

### Workspace Identity and Organization

- **FR-001**: Winds MUST support opening an existing non-bare Git worktree and cloning a Git repository to an explicit destination using the system Git executable.
- **FR-002**: Workspace registration MUST canonicalize and validate the Git worktree root and Git common-directory boundary before persistent workspace state is created.
- **FR-003**: A workspace record MUST include the canonical worktree path, Git common-directory path, current HEAD OID when present, branch/detached state, and a current dirty-state observation or explicit unknown state.
- **FR-004**: Clone/open MUST NOT automatically execute repository environment/tool configuration, shell rc fragments from the repository, `mise`, `direnv`, devcontainer hooks, package install scripts, or similar bootstrap behavior. Detection is allowed; execution requires a later explicit user action/specification.
- **FR-005**: Remote identity persisted by Winds MUST exclude embedded credentials and other obvious secret-bearing URL components. Winds MUST NOT claim that system Git cloning is an OS sandbox or hostile checkout containment boundary.
- **FR-006**: Persistent workspace/session state MUST remain under Winds-owned storage outside the repository checkout and Git common-directory boundary.

### Environment and Shell Discovery

- **FR-007**: Winds MUST report the execution host OS and architecture and discover usable shell profiles with exact executable/argument identity rather than display-name guesses.
- **FR-008**: On Windows, Winds MUST discover installed WSL distributions through Microsoft's supported WSL command surface and keep native Windows and WSL execution domains distinct.
- **FR-009**: Winds MAY detect project environment manifests such as `.mise.toml`, `.tool-versions`, `.python-version`, `.nvmrc`, `.envrc`, `rust-toolchain.toml`, and devcontainer configuration, but MUST NOT evaluate executable project configuration merely to populate inventory.
- **FR-010**: Winds MUST NOT persist the full process environment or secret values by default. Any future environment-value persistence requires an explicit allowlist/secret policy.

### Interactive Terminal Control

- **FR-011**: Interactive terminal sessions MUST use a real PTY on POSIX and a native pseudoconsole/ConPTY-equivalent path on supported Windows hosts; pipe-only emulation is insufficient.
- **FR-012**: Winds MUST support terminal session create, input, output streaming, resize, interrupt, terminate, and close operations, each bound to an exact session identity.
- **FR-013**: A terminal profile MUST bind an execution domain, exact shell executable, arguments, and initial working-directory resolution strategy. Profiles MUST NOT be inferred solely from a human-readable shell name.
- **FR-014**: Switching shell/profile MUST create a new explicit session. Winds MUST NOT represent an already-running shell as having changed operating-system domain.
- **FR-015**: When a workspace directory can be proven to map into the selected execution domain, Winds SHOULD preserve that logical cwd; when it cannot, Winds MUST expose the mismatch and use an explicit fallback.
- **FR-016**: WSL launch behavior MUST use supported `wsl.exe` semantics and bind the selected distribution explicitly. Winds MUST NOT parse localized human-readable output when a stable machine-usable form is available.
- **FR-017**: Spec 003 expands Winds beyond the 0.1 terminal/native-Windows guardrail specifically for workspace terminal execution. It does **not** authorize a daemon, public IPC/runtime protocol, generic plugin runtime, remote shell service, or broad sandbox framework.
- **FR-018**: The first Spec 003 implementation MUST NOT build a custom terminal emulator/VT renderer in the Rust core. It should prove PTY/session mechanics first and leave UI rendering/embedding to a later UI slice unless implementation evidence shows a renderer is required sooner.
- **FR-019**: Initial terminal sessions are owned by the running Winds process. Cross-restart live-session persistence is deferred. On restart, a persisted session for which Winds cannot prove continuing ownership MUST become `OWNERSHIP_LOST`/interrupted with process state unknown. Winds MUST NOT infer liveness/death from a stored PID alone and MUST NOT blindly signal or kill such a PID because of PID reuse risk.
- **FR-020**: Winds MUST NOT modify the user's persistent shell dotfiles/profile merely to instrument a session. Any shell integration must be injected ephemerally and be removable with the session.
- **FR-021**: Winds MUST NOT infer exact shell commands by scraping arbitrary keystrokes. Command-level records require a reliable shell lifecycle integration or an explicit Winds-run command surface. Telemetry supplied by an instrumented shell MUST remain source-labeled shell-reported data unless a protected independent observation proves the fact directly.

### Execution Ledger

- **FR-022**: Winds MUST add a local execution ledger in the existing SQLite/WAL store using forward-only, reviewable schema changes. The existing verification/evidence tables remain semantically unchanged.
- **FR-023**: Each execution MUST have stable identity, workspace identity, execution kind, authority/source classification, execution domain, start time, observed final status, and end/duration fields when known.
- **FR-024**: Terminal-specific fields MUST live in a typed terminal/session record rather than a generic plugin payload. Future SQL and LLM fields MUST likewise use typed domain records.
- **FR-025**: Winds SHOULD record before/after Git HEAD and working-tree state observations around command boundaries when those observations can be obtained reliably and cheaply. Missing observations MUST remain explicitly unknown.
- **FR-026**: Interactive terminal activity MAY intentionally mutate the primary checkout because it is user workspace behavior. Such activity MUST remain clearly separated from `winds verify` candidate worktrees and MUST NOT become `ELIGIBLE` verification evidence merely because it appears in the execution ledger.
- **FR-027**: Process/session lifecycle facts directly observed by Winds may be `WINDS_OBSERVED`; caller-entered commands/profile selections are caller intent; shell-hook command/cwd/exit telemetry is shell-reported unless independently observed. None of those sources may be elevated into authenticated-human or verification authority merely because they are persisted.
- **FR-028**: Persistent terminal output/history MUST be bounded by an explicit local retention/quota policy. Winds MUST NOT allow an unbounded PTY stream to grow SQLite or blob storage indefinitely.
- **FR-029**: Partial database writes, process-launch failures, interrupted sessions, and lost process ownership MUST reconcile to explicit failed/interrupted/ownership-unknown state. No execution may remain falsely successful or falsely live because persistence failed. Restart recovery MUST be conservative and MUST NOT attempt destructive cleanup of a process whose identity/ownership is ambiguous.

### SQL and LLM Follow-On Contract

- **FR-030**: SQL Studio is a follow-on feature, not Spec 003 implementation scope. Its typed execution record is expected to include at minimum connection identity without secret DSN material, database/dialect identity, query/statement identity, transaction/write mode, row/result metadata, client/server timing when available, and optional EXPLAIN/plan artifacts.
- **FR-031**: LLM Observatory is a follow-on feature, not Spec 003 implementation scope. Its typed execution record MUST be designed around provider-reported facts where available and align with current OpenTelemetry GenAI semantic concepts for provider/model, input/output/cache/reasoning tokens, operation duration, and streaming latency. Cost MUST identify its pricing source and MUST NOT be fabricated when price/usage is unknown.
- **FR-032**: Future SQL/LLM credentials MUST use secret references/environment/provider credential mechanisms; raw API keys, database passwords, or credential-bearing DSNs MUST NOT be written to the execution ledger by default.
- **FR-033**: Spec 003 MUST NOT introduce generic SQL-provider, LLM-provider, MCP/A2A, or plugin interfaces merely to anticipate those follow-on features.

### Compatibility and Safety

- **FR-034**: Existing `winds verify`, `winds promote`, and `winds recover` behavior and evidence authority MUST remain regression-tested and unchanged unless a separate requirement explicitly modifies them.
- **FR-035**: Terminal/workspace support targets Linux, macOS, and native Windows. WSL2 integration is an additional Windows execution-domain capability and requires real-environment proof before a support claim.
- **FR-036**: Native Windows workspace/terminal support does not automatically extend the 0.1 `verify/promote` support claim to native Windows; that verification claim requires its own passing platform evidence.
- **FR-037**: Workspaces and terminals are not security sandboxes. Required documentation MUST state that commands run with the launching user's permissions and may access network, secrets, and filesystem locations available to that user.

## Key Artifacts

- **Workspace registry**: canonical repository/worktree identity plus safe environment inventory.
- **Shell profile inventory**: exact host/WSL execution-domain and shell launch definitions.
- **PTY session controller**: in-process terminal lifecycle control with no public runtime protocol.
- **Execution ledger**: SQLite/WAL common execution identity plus typed terminal records/events with explicit observation source.
- **CLI proof surface**: minimal commands that prove open/clone/inspect and shell profile/session behavior before a GUI is relied upon.
- **Cross-platform fixtures**: Linux/macOS/Windows terminal lifecycle tests plus WSL-specific integration evidence when available.
- **Follow-on boundary**: explicit data/authority contract for later SQL Studio and LLM Observatory specifications.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Opening an existing fixture through canonical and symlinked paths yields the correct canonical Git worktree/common-dir identity, exact HEAD, and no persistent Winds state inside the repository.
- **SC-002**: Cloning a fixture through Winds yields a usable registered workspace with exact resulting Git identity while no repository environment/bootstrap manifest is automatically executed.
- **SC-003**: Supported Linux, macOS, and Windows terminal fixtures can create a PTY-backed shell, exchange input/output, resize, interrupt a child command, observe exit, and close without orphaning the directly owned terminal child during normal lifecycle control.
- **SC-004**: On a Windows+WSL2 environment used for acceptance, Winds discovers installed distributions and can launch an explicitly selected Ubuntu/WSL session at the mapped repository cwd when that mapping is valid; mismatches fail visibly rather than silently.
- **SC-005**: A deterministic 100-cycle create/input/resize/command/close soak completes with zero falsely-live terminal records, zero leaked directly owned child processes during controlled close, and zero corruption of existing verification tables.
- **SC-006**: Execution-ledger fixtures prove start/end/duration/final-status data is internally consistent; interrupted persistence cases reconcile explicitly; and simulated restart/lost-ownership cases become `OWNERSHIP_LOST`/unknown without blind PID signaling.
- **SC-007**: Secret-safety fixtures prove full environment values, credential-bearing clone URLs, API keys, and database/password placeholders are not persisted by default.
- **SC-008**: The full pre-existing 0.1 deterministic suite remains green; terminal/workspace changes do not weaken isolated verification/promotion behavior.
- **SC-009**: Spec 003 adds no daemon, public IPC/runtime protocol, generic plugin system, remote execution service, or custom terminal renderer to the first implementation slice.

## Assumptions

- Winds 0.2 remains verification-first; workspace capabilities strengthen observability and control rather than replacing `winds verify` as the authoritative verification primitive.
- A graphical workspace can later be built on the in-process terminal/workspace APIs. UI layout, terminal rendering technology, and persistent detached sessions are separate slices.
- `portable-pty` from the WezTerm ecosystem is a strong implementation candidate for PTY/ConPTY portability, but dependency adoption requires an exact-version/license/provenance audit before code is added.
- WSL discovery/launch should follow then-current Microsoft WSL command documentation rather than reverse-engineered registry state.
- SQL Studio and LLM Observatory will receive their own specifications after the execution spine is proven. This feature records only the common identity/timing/authority boundary they will reuse.
- OpenTelemetry GenAI conventions are evolving; the LLM specification must pin the convention version/schema it adopts rather than assuming `main` is stable forever.
