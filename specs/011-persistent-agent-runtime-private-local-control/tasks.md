# Tasks: Persistent Agent Runtime & Private Local Control

## Canonical Inputs

```text
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=IN_QUALIFICATION
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO

ENTRY_MERGE=4ba393ea68bdd49477a633bc4f7e2bd072d287ff
ENTRY_TREE=6335db9bb9f872ea6b996450646139ebba6af54b
POST_ENTRY_QUALITY=35336085572 SUCCESS ATTEMPT_1

SPEC_MERGE=5734dbb67becf386fe1e1340c2d585fcdf9f6c13
SPEC_TREE=af698f702fbd7669753111b657ff6580e4254c09
POST_SPEC_QUALITY=35337765623 SUCCESS ATTEMPT_1

PLAN_MERGE=f36189becfb3f6322ffe3ff5b72ef7ba5e373a49
PLAN_TREE=08c76c501a3a8f8d7b61a5d1c1a4856cd7654f09
POST_PLAN_QUALITY=35342365164 SUCCESS ATTEMPT_1
```

Tasks creation-time Herdr reconciliation:

```text
HERDR_TASKS_CREATION_PIN=7201907b654aefdb9b636b17ea7ca3cddc873d7b
HERDR_TASKS_CREATION_TREE=cae1bf61ea858e64f9b5bf8928ca066c968b17ab
AGENT_FAMILIES=24
PUBLIC_SERIALIZED_API_METHODS=104
FROZEN_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INTEGRATIONS=1 (letta)
```

The Herdr movement after the Spec/Plan reference pin remains agent-detection/test-policy work and does not expand Spec 011. Herdr is research/test evidence only. No donor source is implementation-admitted by this Tasks file.

Inherited live-runtime nonclaims remain binding:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

## Global Rules

1. Live canonical repository truth overrides this Tasks file if `main`, accepted dependencies, platform evidence, or upstream research moves.
2. Each implementation task starts from then-current exact canonical `main` only after its predecessor is `CLOSED_CANONICAL`.
3. Canonical acceptance of this Tasks file authorizes **T146 only**. No later task is authorized until the immediately preceding task closes canonically with successful post-merge verification.
4. No force-push, rebase, shared-history rewrite, rerun-to-green, stale-head review reuse, hidden failure suppression, or fabricated evidence.
5. Candidate movement invalidates candidate-bound CI, security, performance, platform, review, and qualification evidence.
6. `PERSISTED_PID != PROCESS_IDENTITY`. No task may signal, kill, attach, or claim ownership based only on a persisted PID or process scan.
7. `PERSISTED_ROW != LIVE_OWNERSHIP`. A live-owned claim requires the accepted owner generation to retain the real ownership primitive.
8. `RUNTIME_NAMESPACE != CANONICAL_WINDS_SESSION != WORKSTREAM_OR_TASK != PROVIDER_NATIVE_SESSION != GIT_CANDIDATE`.
9. `OBSERVER_AUTHORITY != CONTROLLER_AUTHORITY` and `CLIENT_FOCUS != CONTROLLER_LEASE`.
10. Exactly one runtime is targeted by every mutating control action. Multi-runtime broadcast is forbidden.
11. Terminal/agent output is untrusted data and can never become local-control protocol, verification, approval, acceptance, or host-action authority.
12. The Spec 011 control plane is local-only. No TCP, HTTP, WebSocket, LAN, SSH, cloud relay, remote-control path, browser-origin endpoint, or public SDK enters this program.
13. The first program does not claim isolation from arbitrary malicious code running as the same effective OS user/principal.
14. The existing PTY/ConPTY implementation remains the process primitive; no second terminal backend, tmux-style engine, or Herdr PTY transplant is authorized.
15. Existing native-Windows fail-closed interrupt semantics remain unchanged unless a separately accepted exact implementation proves a stronger ownership-scoped primitive.
16. Linux, macOS, native Windows, and WSL are distinct evidence domains. No platform claim is inferred from another.
17. The Tauri/WebView renderer never connects directly to the persistent owner. Desktop access stays behind an allowlisted Rust host/client seam.
18. No generic command dispatcher, shell/filesystem/Git/SQL passthrough, plugin protocol, provider gateway, or arbitrary method registry may be introduced.
19. Existing migrations `0011_model_mesh_continuity.sql` and `0012_desktop_presentation.sql` remain immutable. The only planned new migration is `0013_persistent_runtime_owner.sql`, owned by T147.
20. No transcript/history database, semantic/vector memory, full process environment, raw credential, bearer token, controller lease, or OS PID may be persisted merely to support reattach.
21. Replay is presentation continuity only and never canonical evidence.
22. The first protocol uses versioned bounded length-prefixed JSON over platform-private local transport; no generic RPC/IDL framework is authorized.
23. No Tokio/async runtime is admitted merely for local IPC. The accepted first design uses bounded blocking concurrency.
24. No service-manager/autostart integration is authorized. The owner uses the shipped Winds binary in an internal owner mode and exits after the accepted idle grace when no clients/live runtimes remain.
25. Herdr source reuse requires a later exact task that records immutable source revision/path, license/NOTICE/vendor provenance, destination, modifications, threat-model delta, tests, and removal/update path. No task below currently authorizes direct/adapted Herdr copying.
26. No task below authorizes real direct Codex/Claude execution unless a separately accepted governance change supersedes the inherited nonclaims with exact evidence.
27. Persistent process authority never grants verification, human acceptance, delegation-ceiling expansion, Git mutation/landing, remote authority, plugin authority, or credential authority.
28. Failed/rejected/superseded candidates and failed workflows remain historical evidence and must not be relabelled.
29. Every task must preserve exact error/source/proof distinctions; generic `connected`, `running`, or `success` must not hide ownership loss, protocol mismatch, stale generation, replay gaps, or cleanup uncertainty.
30. The program may close only after T161 reconciles all 70 FR and 25 SC to exact canonical evidence and truthful nonclaims.

## Standard Acceptance Gate

Every implementation-bearing task must satisfy on its exact final candidate:

- task-focused deterministic tests PASS;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS for Rust-touched candidates;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS for Rust-touched candidates;
- complete applicable repository tests under `quality` PASS;
- applicable `windows-terminal` and any task-owned platform/security/stress workflows PASS when the task touches or claims those domains;
- no unrelated source/dependency/migration/workflow mutation;
- author correctness/safety/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact final candidate with zero unresolved material findings;
- zero unresolved review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge push workflow succeeds.

Documentation/governance-only candidates use the same exact identity/review/landing discipline and repository `quality` without inventing irrelevant runtime tests.

A task that changes dependency, platform, security, or migration truth must also preserve an exact provenance/evidence artifact for that change.

## Dependency Order

```text
T146 -> T147 -> T148 -> T149 -> T150 -> T151 -> T152 -> T153
     -> T154 -> T155 -> T156 -> T157 -> T158 -> T159 -> T160 -> T161
```

Canonical acceptance and successful post-merge verification of this Tasks file authorize **T146 only**.

---

## Phase 1 — Domain, Identity, and Event Contract

### [ ] T146 — Freeze persistent-runtime domain, identity entropy contract, continuity classes, and lifecycle event schema

**Purpose**: define the authority model before any persistence, transport, process owner, or endpoint exists.

**Authorized paths**:
- new Winds-owned `src/persistent_runtime/mod.rs` and `src/persistent_runtime/domain.rs`;
- `src/lib.rs` only for module/test registration;
- focused `src/t146_persistent_runtime_domain_tests.rs`;
- focused provenance/design evidence under `docs/provenance/`;
- no Cargo dependency, migration, transport, process launch, endpoint, desktop, Git, provider, or workflow mutation except a focused task workflow if proven necessary.

**Required domain types/invariants**:
- immutable `RuntimeNamespaceId` and `OwnerGenerationId`;
- mutable alias represented separately from immutable ID;
- `OwnershipState` with an explicit `OWNERSHIP_LOST` state;
- liveness/endpoint availability represented separately from ownership;
- `ContinuityClass` distinguishing retained live process, provider-native resume, Winds reconstruction/reassignment, fresh process, and unavailable/unknown as required by the Spec;
- observer vs controller role/authority vocabulary;
- controller lease identity vocabulary without implementing leases yet;
- typed local-control error vocabulary including protocol mismatch, principal denial, stale generation, unknown runtime, ownership loss, controller conflict, malformed/oversized frame, duplicate/out-of-order mutation, slow client/backpressure, unsupported operation/platform, and `OUTCOME_UNKNOWN`;
- lifecycle event schema with exact runtime namespace, owner generation, event kind, monotonic sequence/order identity, source/proof class, optional controller/client identity, and timestamp where valid;
- event schema excludes raw credentials, full process environment, secret material, arbitrary agent prose, and canonical verification/acceptance authority.

**Entropy contract frozen here**:
- both owner generation and runtime namespace IDs use 128 bits from an OS CSPRNG and encode to a canonical lower-case fixed-width representation;
- Linux implementation seam: `getrandom(2)` through the accepted libc/platform boundary;
- macOS implementation seam: `getentropy(3)` through the accepted libc/platform boundary;
- native-Windows implementation seam: `BCryptGenRandom` through the exact Win32 dependency seam qualified in T150;
- all-zero/short/failed entropy results fail closed;
- aliases, timestamps, counters, PIDs, paths, or database rowids are insufficient as identity entropy;
- no UUID/random crate is admitted by T146.

**Acceptance**:
- deterministic parsing/serialization/equality tests;
- duplicate aliases never create identity equivalence;
- ownership/liveness/continuity truth cannot be conflated by type conversions;
- event schema has deterministic ordering and no authority derived from display text;
- no process, socket, pipe, database migration, endpoint, or persistent owner exists after T146.

**Primary FR coverage**: FR-001, FR-005, FR-009, FR-012, FR-033, FR-035, FR-062, FR-063, FR-067, FR-068.

**Closes to authorize**: T147.

---

## Phase 2 — Durable Namespace Metadata and Fail-Closed Restart Reconciliation

### [ ] T147 — Add migration 0013 and Store reconciliation without PID/process reattachment

**Purpose**: persist only enough namespace/generation/lifecycle metadata to explain restart truth while keeping live ownership non-persistent.

**Authorized paths**:
- new `migrations/0013_persistent_runtime_owner.sql` only;
- `src/store.rs` and narrowly required domain/persistence support;
- `src/persistent_runtime/persistence.rs`;
- focused `src/t147_persistent_runtime_persistence_tests.rs`;
- migration/provenance evidence;
- no endpoint, process owner, IPC, PTY move, renderer, provider, or new dependency.

**Required persistence**:
- runtime namespace immutable ID + mutable alias;
- optional accepted canonical workspace/session/terminal references;
- owner-generation reference/history sufficient for reconciliation;
- last proven ownership/lifecycle/continuity classification;
- lifecycle timestamps/recovery reason where proven;
- schema/version constraints and foreign-key integrity where applicable.

**Explicitly prohibited durable state**:
- OS PID/process handle as authority;
- raw PTY handles;
- controller lease as restart-surviving authority;
- full process environment;
- credentials/tokens;
- terminal transcript/history;
- arbitrary agent/model text;
- duplicated candidate verification/human acceptance truth.

**Startup/recovery rule**:
- any row requiring a prior owner generation to prove live ownership is reconciled to `OWNERSHIP_LOST` when that generation is not the currently proven owner;
- process liveness may remain separately unknown;
- no process scan, PID lookup, signal, kill, or attach is performed by reconciliation;
- partial/corrupt state fails closed and preserves recoverable evidence rather than silently reinitializing ambiguous live rows.

**Acceptance**:
- fresh DB, migrated DB, restart, partial-row, corrupt-row, duplicate-alias, stale-generation fixtures;
- migration 0011 and 0012 digests unchanged;
- removing presentation state still does not remove canonical execution/workflow/evidence truth;
- zero PID recovery code.

**Primary FR coverage**: FR-002, FR-004, FR-047, FR-049, FR-050.

**Primary SC coverage**: SC-002.

**Closes to authorize**: T148.

---

## Phase 3 — Versioned Bounded Protocol Codec

### [ ] T148 — Implement protocol v1 framing, typed messages, correlation, authority classes, and replay rejection over in-memory streams only

**Purpose**: prove the wire contract and bounded parsing before a real OS endpoint exists.

**Authorized paths**:
- `src/persistent_runtime/protocol.rs` and domain support;
- focused `src/t148_persistent_runtime_protocol_tests.rs`;
- no Unix socket/named pipe, owner process, PTY move, migration, dependency, renderer, or network listener.

**Frozen framing**:
- little-endian `u32` payload length followed by UTF-8 JSON;
- protocol version `1`;
- maximum inbound control frame 256 KiB;
- output event chunk <= 64 KiB before framing;
- claimed lengths are validated before allocation/read;
- unknown version/kind/required field/malformed JSON/UTF-8 fails deterministically.

**Message ceiling**:
- connection/observer-safe: `HELLO`, `PING/PONG`, `LIST_RUNTIMES`, `RUNTIME_SNAPSHOT`, `ATTACH_OBSERVER`, `DETACH`, `RUNTIME_EVENT`, `OUTPUT_EVENT`, `HISTORY_GAP`, `OWNER_STATUS`;
- owner-to-client state only: `HELLO_ACK`, `ERROR`, `CONTROL_STATE`;
- bounded authority transition: `REQUEST_CONTROL`;
- active-controller/revocation path: `RELEASE_CONTROL`;
- controller-only: `INPUT`, `RESIZE`, `INTERRUPT`, `STOP`;
- no `SHUTDOWN_IF_IDLE` wire method;
- no generic method dispatcher.

**Sequencing/correlation**:
- exact owner generation + connection ID + runtime namespace where applicable;
- request sequence strictly increases within a connection;
- repeated/lower sequence is a typed reject and never re-executes a mutation;
- no consequential mutation result cache;
- connection loss with uncertain mutation result becomes `OUTCOME_UNKNOWN` and requires state reconciliation before any new explicit action;
- response/event binding cannot be reassigned to another runtime/generation/request.

**Acceptance**:
- round-trip/golden fixtures;
- malformed/oversized/truncated/unknown-version tests;
- duplicated/out-of-order mutation tests;
- terminal/agent strings shaped like JSON/control frames remain data;
- zero listener/socket/pipe exists.

**Primary FR coverage**: FR-020 through FR-026, FR-028.

**Primary SC coverage**: SC-007, SC-008.

**Closes to authorize**: T149.

---

## Phase 4 — POSIX Private Local Transport

### [ ] T149 — Implement secure Unix-domain local transport for Linux and macOS

**Purpose**: introduce the first real endpoint with platform-local principal checks and stale-path defenses, without process ownership migration.

**Authorized paths**:
- `src/persistent_runtime/transport/mod.rs` and `transport/unix.rs`;
- `src/persistent_runtime/peer.rs` POSIX portion;
- focused Linux/macOS tests;
- exact provenance/security artifact;
- no Windows pipe, PTY persistent owner, renderer, remote/network transport, or new dependency.

**Required endpoint behavior**:
- Unix-domain stream socket only;
- absolute user-owned runtime directory selected by one deterministic resolver;
- directory ownership/mode validated before bind/connect;
- no group/world writable endpoint parent;
- symlink/path substitution rejected;
- stale endpoint removed only after proving no live matching owner;
- live endpoint is never destructively replaced based only on pathname existence;
- collision/path-length/fallback handling deterministic.

**Principal checks**:
- Linux: direct peer-credential proof using accepted kernel/libc primitive;
- macOS: direct peer effective-user proof using accepted platform/libc primitive;
- both separately tested; Linux evidence cannot substitute for macOS;
- same-effective-user malicious-code isolation remains explicitly unclaimed.

**Entropy implementation**:
- Linux namespace/generation IDs use the T146 `getrandom(2)` contract;
- macOS uses the T146 `getentropy(3)` contract;
- entropy failure prevents endpoint/runtime creation.

**Acceptance**:
- same-user happy path;
- wrong-user fixture where CI/environment permits;
- stale socket, regular-file collision, symlink substitution, ownership/mode mismatch, live-owner collision, entropy-failure fixtures;
- no TCP/HTTP/WebSocket/network listener exists.

**Primary FR coverage**: FR-029, FR-031, FR-032, FR-034, FR-036.

**Primary SC coverage**: POSIX portion of SC-016.

**Closes to authorize**: T150.

---

## Phase 5 — Native-Windows Private Local Transport

### [ ] T150 — Implement named-pipe current-user security and qualify the exact Win32 dependency seam

**Purpose**: add native-Windows endpoint security without inferring it from Unix or WSL.

**Authorized paths**:
- `Cargo.toml` / `Cargo.lock` only if direct `windows-sys = 0.61.2` is required;
- exact minimum Win32 features only;
- `src/persistent_runtime/transport/windows.rs` and Windows peer/security support;
- focused official-Windows tests/workflow changes strictly required for the slice;
- `docs/provenance/` dependency/security record;
- no PTY persistent owner yet, no remote transport, no service installation.

**Dependency gate if `windows-sys` becomes direct**:
- exact 0.61.2 checksum/source/license/MSRV/transitive graph recorded;
- minimal required Win32 feature modules documented from compiled calls;
- no broad umbrella features;
- removal/update path recorded;
- no second abstraction crate added.

**Required Windows behavior**:
- named pipe only, no loopback TCP;
- explicit security descriptor/DACL limited to current accepted user principal plus minimum required system principals;
- current-user SID/principal binding directly proven;
- wrong-principal denial directly proven with the strongest available official-runner fixture;
- pipe-name collision/stale-owner/generation mismatch fail closed;
- T146 identity entropy implemented with `BCryptGenRandom`;
- same-user malicious-code isolation remains unclaimed.

**Acceptance**:
- official native Windows compile/tests;
- effective DACL and allowed/denied principal evidence;
- named-pipe collision/generation fixtures;
- Windows and WSL claims kept distinct;
- existing `windows-terminal` behavior remains green.

**Primary FR coverage**: native-Windows portions of FR-029 through FR-036, FR-057.

**Primary SC coverage**: native-Windows portions of SC-016 and SC-017.

**Closes to authorize**: T151.

---

## Phase 6 — Owner Process Shell and Singleton Generation

### [ ] T151 — Add same-binary owner mode, singleton arbitration, readiness, and mandatory idle exit with zero child ownership

**Purpose**: prove a narrow persistent process lifecycle before moving any PTY/ConPTY child into it.

**Authorized paths**:
- `src/persistent_runtime/owner.rs` and narrowly required CLI/internal entry wiring;
- shared platform singleton support;
- focused tests and task evidence;
- no persistent PTY launch, no desktop renderer change, no provider launch.

**Required behavior**:
- one active owner per canonical Winds home/accepted principal;
- same shipped `winds` binary internal owner mode; not a public general daemon command contract;
- exact owner generation created from T146/T149/T150 entropy seam;
- startup order: singleton -> generation -> Store/migration reconciliation -> secure endpoint -> ready;
- endpoint never advertises ready before reconciliation succeeds;
- concurrent launches deterministically produce one owner and bounded losers/connectors;
- singleton primitive named and proven per platform in the task evidence;
- no launchd/systemd/Windows Service/login/autostart integration;
- with no connected clients and no live runtimes, owner MUST exit after 300 seconds;
- no wire request can force idle shutdown;
- no PID persisted as authority.

**Acceptance**:
- concurrent-start race fixture;
- stale singleton/endpoint/generation fixture;
- startup failure before ready;
- clean idle exit and immediate non-idle preservation;
- zero child process/PTY is persistently owned by this task.

**Primary FR coverage**: FR-008, FR-049, FR-054, FR-065.

**Closes to authorize**: T152.

---

## Phase 7 — Persistent PTY/ConPTY Ownership

### [ ] T152 — Move/reuse accepted terminal ownership under the persistent owner for bounded shell runtimes

**Purpose**: deliver the first real detach-survives-client capability without adding provider execution authority.

**Authorized paths**:
- owner/runtime registry and current execution/terminal seams only as required;
- focused terminal persistence tests;
- existing platform workflows where touched;
- no new terminal backend, no direct Codex/Claude launch, no remote/plugin behavior.

**Required behavior**:
- existing POSIX PTY and native-Windows ConPTY creation/input/output/resize/exit/terminate/close semantics reused;
- runtime namespace owns the exact retained process/PTY primitive inside the current owner generation;
- closing/detaching all presentation clients does not kill an owner-preserved runtime;
- reattach within the same proven owner generation observes the same live-owned runtime;
- process exit while detached is observed and retained as lifecycle truth;
- no PID-based reconstruction after owner loss;
- native-Windows interrupt remains explicitly fail-closed unless current accepted backend proves otherwise;
- no provider/model identity inferred from shell/process title/output.

**Acceptance**:
- long-running shell survives complete client detachment and reattaches to exact same runtime/owner generation;
- process exit while detached is reported accurately;
- terminate/reap only resources Winds proves it owns;
- inherited terminal regression suites and official Windows terminal gate remain green;
- inherited real Codex/Claude nonclaims unchanged.

**Primary FR coverage**: FR-003, FR-006, FR-055, FR-056.

**Primary SC coverage**: SC-001.

**Closes to authorize**: T153.

---

## Phase 8 — Multiple Observers, Bounded Replay, and Backpressure

### [ ] T153 — Add read-only observers, bounded in-memory replay, explicit gaps, and slow-client protection

**Purpose**: make detach/reattach useful without an unbounded transcript or resource leak.

**Authorized paths**:
- `src/persistent_runtime/replay.rs` plus owner/protocol support;
- focused observer/replay/backpressure tests;
- no controller mutation beyond the existing owner-only internal path.

**Frozen bounds**:
- <= 8 MiB retained output/event bytes per runtime;
- <= 10,000 retained events per runtime;
- <= 64 MiB default aggregate owner replay budget;
- <= 4 MiB queued outbound payload per client;
- deterministic fair eviction when aggregate replay budget is reached;
- explicit earliest-available sequence and `HISTORY_GAP`/truncation semantics.

**Required behavior**:
- multiple observers receive bounded runtime/lifecycle/output state;
- observers cannot input/resize/interrupt/stop/request generic mutation;
- replay preserves source/proof/order;
- replayed `VERIFIED`/approval/protocol-like strings remain untrusted presentation data;
- slow observer cannot block PTY reader, controller, peer observer, or owner authority loop;
- queue overflow has deterministic drop/gap/disconnect behavior by message class;
- no terminal transcript is persisted.

**Acceptance**:
- >=8 concurrent observers fixture;
- >8 MiB/runtime and >64 MiB aggregate replay eviction fixtures;
- slow reader/queue overflow fixture;
- forged evidence/protocol-shaped replay fixture;
- no unbounded memory growth.

**Primary FR coverage**: FR-013, FR-025, FR-027, FR-044 through FR-046.

**Primary SC coverage**: SC-004 observer path, SC-010, SC-011.

**Closes to authorize**: T154.

---

## Phase 9 — Explicit Controller Lease and Race Semantics

### [ ] T154 — Implement one explicit controller lease per runtime and deterministic takeover/revocation

**Purpose**: permit writable terminal control without accidental authority inheritance or broadcast.

**Authorized paths**:
- `src/persistent_runtime/controller.rs` plus owner/protocol support;
- focused deterministic race tests;
- no new mutation classes beyond accepted terminal operations.

**Frozen first-program policy**:
- at most one controller lease per runtime;
- lease bound to owner generation + runtime namespace + client connection;
- heartbeat target 10 seconds;
- expiry 30 seconds;
- client focus/recency/visibility never grants authority;
- controller disconnect/expiry yields no automatic observer promotion;
- `REQUEST_CONTROL` is explicit and serialized;
- `RELEASE_CONTROL` only active controller or owner-defined revocation;
- INPUT/RESIZE/INTERRUPT/STOP require the active exact lease;
- no multi-runtime broadcast.

**Required races**:
- simultaneous takeover;
- input during takeover;
- resize during takeover;
- stop/interrupt during takeover;
- disconnect vs renewal;
- expiry vs renewal;
- observer mutation attempt.

**Acceptance**:
- exactly one deterministic controller outcome;
- no duplicate consequential action;
- prior controller disposition attributable;
- peer runtimes unaffected;
- lease never survives owner generation restart as authority.

**Primary FR coverage**: FR-014 through FR-019.

**Primary SC coverage**: SC-004 controller path, SC-005, SC-006.

**Closes to authorize**: T155.

---

## Phase 10 — Shared Rust Local-Control Client Seam

### [ ] T155 — Implement one Rust-owned client for CLI/TUI/Desktop host integration

**Purpose**: keep endpoint/security/protocol/reconnect logic out of renderer/UI domains and prevent competing client implementations.

**Authorized paths**:
- `src/persistent_runtime/client.rs`;
- bounded CLI/TUI/Desktop Rust-host adapters/projections needed to exercise the client;
- focused tests;
- no renderer-direct socket/pipe API and no generic dispatcher.

**Required behavior**:
- endpoint resolution + principal/generation handshake;
- protocol version validation;
- connection/request correlation;
- observer attach;
- explicit controller acquire/release APIs;
- generation mismatch and protocol mismatch surfaced distinctly;
- alias search requires exact ID or explicit disambiguation before consequential action;
- uncertain mutation response becomes `OUTCOME_UNKNOWN` and triggers state reconciliation before new action;
- remote/network endpoint syntax rejected;
- renderer only receives bounded typed projections through existing trusted Rust host.

**Acceptance**:
- exact runtime/generation reattach;
- duplicate aliases cannot mis-target;
- stale client cannot treat replacement generation as continuation;
- no WebView JavaScript can open the owner endpoint directly through a privileged generic bridge;
- no raw filesystem/Git/SQL/shell method is exposed.

**Primary FR coverage**: FR-007, FR-023, FR-038, FR-039, FR-063, FR-064.

**Closes to authorize**: T156.

---

## Phase 11 — Truthful Detach/Reattach and Continuity Projections

### [ ] T156 — Surface ownership, continuity, controller, replay, and mismatch states across accepted clients

**Purpose**: make the new runtime truth usable without visually upgrading process continuity into provider/evidence truth.

**Authorized paths**:
- typed CLI/TUI/Desktop Rust projections and bounded existing presentation components/tests;
- no new provider launcher/model router/verification mutation.

**Required visible states**:
- live retained-process continuation;
- provider-native resume (when already known from accepted source);
- Winds reconstruction/reassignment;
- fresh process;
- ownership lost;
- process liveness unknown/unavailable;
- protocol incompatible;
- owner-generation stale/mismatch;
- observer/controller status;
- replay truncated/gap;
- `OUTCOME_UNKNOWN` where applicable.

**Required invariants**:
- native resume != retained live process;
- reconstruction != resume;
- process exit != verified/accepted;
- agent-reported runtime/continuity cannot become Winds-observed truth;
- display alias never replaces immutable runtime target.

**Acceptance**:
- deterministic fixture matrix for every state;
- no optimistic generic `connected` state hides loss/mismatch/gap;
- historical Spec 006/009/010 provider nonclaims remain unchanged.

**Primary FR coverage**: FR-010 through FR-012, FR-042, FR-043, FR-070.

**Primary SC coverage**: SC-014.

**Closes to authorize**: T157.

---

## Phase 12 — Stop, Crash, Restart, and Recovery Truth

### [ ] T157 — Prove fail-closed owner/runtime stop and recovery without blind process reconstruction

**Purpose**: close the dangerous lifecycle edges before adversarial broadening.

**Authorized paths**:
- owner/persistence/client recovery logic and focused tests;
- no process enumeration/PID attach, remote/plugin/update behavior.

**Required campaigns**:
- owner crash/kill with live child;
- client crash during mutation;
- owner restart/new generation;
- stale endpoint after crash;
- stale PID/reused PID;
- partial metadata write/corrupt active row;
- explicit runtime stop healthy/stuck/already-terminal;
- repeated stop;
- stop response lost/connection loss -> `OUTCOME_UNKNOWN`;
- owner shutdown with multiple runtimes and per-runtime disposition.

**Required truth**:
- prior-generation live ownership becomes `OWNERSHIP_LOST` when not provable;
- possible surviving process remains liveness unknown, not silently adopted;
- no blind signal/kill;
- cleanup success only when termination/reap/finalization is proven;
- uncertain cleanup preserves recovery evidence;
- endpoint removal only with proven endpoint ownership.

**Acceptance**:
- zero unrelated-process control in PID-reuse fixtures;
- repeated stop is safe/deterministic;
- crash/restart converges to explicit truthful states;
- canonical Store/evidence/Git truth unaffected.

**Primary FR coverage**: FR-049 through FR-054 plus restart/recovery portions of FR-004, FR-008, FR-024, FR-031, FR-032.

**Primary SC coverage**: SC-002 recovery path, SC-003, SC-013, SC-015.

**Closes to authorize**: T158.

---

## Phase 13 — Adversarial Protocol and Authority Campaign

### [ ] T158 — Attack endpoint, protocol, renderer, authority, and secret boundaries

**Purpose**: prove the local owner is private/least-authority rather than merely functional.

**Required campaign**:
- wrong local principal;
- stale/attacker-created endpoint path;
- symlink/reparse substitution;
- guessed namespace/generation IDs;
- malformed/oversized/truncated frames;
- unknown protocol version and downgrade attempt;
- repeated/lower/out-of-order mutation sequences;
- terminal/agent text shaped like control JSON, approvals, `VERIFIED`, runtime labels, host actions;
- observer attempting INPUT/RESIZE/INTERRUPT/STOP;
- client/controller generation confusion;
- renderer attempting generic control access;
- remote/TCP/HTTP/WebSocket endpoint attempts;
- secret-shaped strings/full environment fixtures;
- Git/verification/acceptance escalation attempts;
- generic plugin/method-dispatch attempts.

**Required nonclaims**:
- same-effective-user malicious code is not claimed isolated;
- worktree/PTTY/endpoint locality is not an OS sandbox;
- persistent owner does not prove provider/model truth;
- no perfect secret detector is claimed.

**Acceptance**:
- all attacks fail closed/bounded;
- no public listener or generic host API exists;
- no secret/full environment persisted in owner metadata/history;
- no Git/verification/human-decision mutation through the owner.

**Primary FR coverage**: FR-026, FR-028 through FR-043, FR-048, FR-066.

**Primary SC coverage**: SC-009, SC-012, SC-022, SC-023.

**Closes to authorize**: T159.

---

## Phase 14 — Stress, Performance, and Resource Qualification

### [ ] T159 — Prove bounded resources, high output, observer scale, and reconnect churn

**Purpose**: make persistence lightweight and prevent slow-client/output pressure from becoming an availability or authority problem.

**Frozen Plan ceilings/campaigns**:
- idle owner CPU with zero live runtimes <= 1% of one logical core p95;
- idle owner RSS <= 64 MiB;
- local reattach handshake + snapshot <= 100 ms p95;
- cached observer attach <= 100 ms p95;
- controller input dispatch overhead <= 10 ms p95 excluding child processing;
- resize dispatch overhead <= 25 ms p95 excluding child processing;
- >= 10 MiB and >= 100,000 logical lines output;
- >= 8 concurrent observers to one runtime;
- >= 32 idle runtime namespaces;
- >= 500 attach/detach reconnect cycles;
- replay/client queue bounds from T153 remain enforced.

**Required evidence**:
- exact candidate/environment/build/runtime provenance;
- raw/lossless-enough metrics;
- no correctness/security checks disabled for performance;
- no ambiguous generations, leaked clients/controllers/runtime records, leaked owned handles/fds after churn.

**Acceptance**:
- every ceiling met or a separate Plan amendment lands before any relaxation;
- exact target/controller ownership preserved under output and churn;
- slow observer cannot degrade peer/control correctness.

**Primary FR coverage**: FR-059 through FR-061 plus stress portions of FR-025, FR-027.

**Primary SC coverage**: SC-019, SC-020, SC-021.

**Closes to authorize**: T160.

---

## Phase 15 — Native Platform Qualification

### [ ] T160 — Directly qualify Linux, macOS, native Windows, and any explicitly claimed WSL domain

**Purpose**: prevent cross-platform substitution for endpoint/process security and continuity.

**Required Linux evidence**:
- Unix socket ownership/permissions and peer credential;
- entropy seam;
- detach/reattach;
- observer/controller;
- crash/recovery;
- replay/backpressure/resource campaigns.

**Required macOS evidence**:
- independent Unix socket parent/path/peer behavior;
- entropy seam;
- PTY continuity;
- lifecycle/recovery/resource evidence.

**Required native-Windows evidence**:
- named-pipe security descriptor/DACL;
- current-user access and wrong-principal denial fixture;
- entropy seam;
- name/generation collision behavior;
- ConPTY detach/reattach/input/output/resize/exit/stop;
- preserved fail-closed interrupt limitation;
- controller/crash/reconnect/resource campaigns on official Windows.

**WSL**:
- WSL is not a native-Windows endpoint claim;
- no Windows-host↔WSL owner bridge is introduced;
- a WSL/Linux owner may be claimed only if directly exercised; otherwise `SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED`.

**Acceptance**:
- every released platform claim has exact-candidate direct evidence;
- unsupported limitations remain explicit;
- Linux/macOS/native Windows/WSL evidence is not substituted across domains.

**Primary FR coverage**: FR-030, FR-055 through FR-058.

**Primary SC coverage**: SC-016, SC-017, SC-018.

**Closes to authorize**: T161.

---

## Phase 16 — Final Spec 011 Reconciliation

### [ ] T161 — Reconcile every requirement/evidence boundary and close the first persistent-runtime program

**Purpose**: prevent “it works” from becoming an unproven parity/security/authority claim.

**Authorized paths**:
- this `tasks.md` checked-state reconciliation;
- `specs/011-persistent-agent-runtime-private-local-control/t161-final-reconciliation.md`;
- focused acceptance/provenance evidence artifacts;
- proven README/docs updates;
- no new product behavior.

**Required reconciliation**:
- exact canonical merges/post-merge verification for T146–T160;
- FR-001..FR-070 each mapped to deterministic/platform/security/performance/governance evidence or an explicit truthful nonclaim where permitted;
- SC-001..SC-025 each mapped to exact evidence;
- migration inventory proves 0011/0012 unchanged and 0013 exact;
- dependency/provenance inventory exact, including any direct `windows-sys` adoption;
- owner generation/runtime namespace/canonical Session/workflow/provider/candidate/evidence identities remain separate;
- no PID-as-authority or blind process recovery entered;
- observer/controller and zero-broadcast proofs intact;
- protocol authority classes/version/bounds/replay semantics exact;
- endpoint principal/path/DACL proofs exact by platform;
- replay remains non-evidence and bounded;
- inherited real Codex/Claude nonclaims preserved unless separately superseded by accepted exact evidence;
- Herdr donor-code ledger remains empty unless a later exact task actually admitted a slice;
- remote/plugin/marketplace/live-handoff/public-network/automatic-Git scope remains absent;
- every historical failed/rejected/superseded candidate remains inspectable and not relabelled;
- final exact implementation state passes repository/platform/security/stress/performance gates;
- fresh author/Ponytail/independent review reports zero material findings;
- guarded expected-head landing and post-merge push verification complete.

Only after canonical landing and post-merge proof may repository truth state:

```text
T146..T161=CLOSED_CANONICAL
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=CLOSED_CANONICAL
SPEC_011_FIRST_PERSISTENT_RUNTIME_PROGRAM=CLOSED_CANONICAL
```

Closing T161 authorizes no Spec 012 implementation automatically. Spec 012 requires its own canonical Entry -> Spec -> Plan -> Tasks sequence.

**Closes to authorize**: no successor implementation task inside Spec 011.

---

# Requirement-to-Task Coverage Matrix

This matrix assigns a primary closure task. Supporting tasks may provide additional evidence; T161 performs final exact reconciliation.

| Requirement | Primary task |
| --- | --- |
| FR-001 | T146 |
| FR-002 | T147 |
| FR-003 | T152 |
| FR-004 | T147 / T157 |
| FR-005 | T146 |
| FR-006 | T152 |
| FR-007 | T155 |
| FR-008 | T151 / T157 |
| FR-009 | T146 |
| FR-010 | T156 |
| FR-011 | T156 |
| FR-012 | T146 / T156 |
| FR-013 | T153 |
| FR-014 | T154 |
| FR-015 | T154 |
| FR-016 | T154 |
| FR-017 | T154 |
| FR-018 | T154 |
| FR-019 | T154 |
| FR-020 | T148 |
| FR-021 | T148 |
| FR-022 | T148 |
| FR-023 | T148 / T155 |
| FR-024 | T148 / T157 |
| FR-025 | T148 / T153 / T159 |
| FR-026 | T148 / T158 |
| FR-027 | T153 / T159 |
| FR-028 | T148 / T158 |
| FR-029 | T149 / T150 / T158 |
| FR-030 | T149 / T150 / T160 |
| FR-031 | T149 / T150 / T157 |
| FR-032 | T149 / T150 / T157 |
| FR-033 | T146 |
| FR-034 | T149 / T150 |
| FR-035 | T146 |
| FR-036 | T149 / T150 / T158 |
| FR-037 | T158 |
| FR-038 | T155 / T158 |
| FR-039 | T155 |
| FR-040 | T158 |
| FR-041 | T158 |
| FR-042 | T156 / T158 |
| FR-043 | T156 |
| FR-044 | T153 |
| FR-045 | T153 / T156 |
| FR-046 | T153 |
| FR-047 | T147 |
| FR-048 | T158 |
| FR-049 | T147 / T157 |
| FR-050 | T147 / T157 |
| FR-051 | T157 |
| FR-052 | T157 |
| FR-053 | T157 |
| FR-054 | T151 / T157 |
| FR-055 | T152 / T160 |
| FR-056 | T152 / T160 |
| FR-057 | T150 / T160 |
| FR-058 | T160 |
| FR-059 | T159 |
| FR-060 | T159 |
| FR-061 | T159 |
| FR-062 | T146 |
| FR-063 | T146 / T155 |
| FR-064 | T155 |
| FR-065 | T151 |
| FR-066 | T158 |
| FR-067 | T146 / T161 |
| FR-068 | T146 / T161 |
| FR-069 | T161 |
| FR-070 | T156 |

# Success-Criterion-to-Task Coverage Matrix

| Criterion | Primary task |
| --- | --- |
| SC-001 | T152 |
| SC-002 | T147 / T157 |
| SC-003 | T157 / T158 |
| SC-004 | T153 / T154 |
| SC-005 | T154 |
| SC-006 | T154 |
| SC-007 | T148 |
| SC-008 | T148 |
| SC-009 | T158 |
| SC-010 | T153 / T159 |
| SC-011 | T153 |
| SC-012 | T158 |
| SC-013 | T157 |
| SC-014 | T156 |
| SC-015 | T157 |
| SC-016 | T149 / T150 / T160 |
| SC-017 | T150 / T160 |
| SC-018 | T160 |
| SC-019 | T159 |
| SC-020 | T159 |
| SC-021 | T159 |
| SC-022 | T158 |
| SC-023 | T158 |
| SC-024 | T161 |
| SC-025 | T161 |

## Tasks Acceptance Gate

This Tasks candidate may land only if its exact final candidate proves:

- changed scope is exactly this `tasks.md` unless a separately justified governance-only correction is required;
- Entry, Spec, and Plan are canonically closed with successful post-merge quality on their exact merges;
- dependency order is explicit and canonical Tasks acceptance authorizes T146 only;
- all 70 FR and 25 SC appear in the coverage matrices with a primary implementation/evidence owner;
- no source, dependency, lockfile, migration, workflow-semantic implementation, process owner, socket/pipe endpoint, renderer behavior, provider/runtime execution, remote/plugin/update behavior, or donor-code copy is introduced by this Tasks candidate;
- migration 0013 is owned by T147 and prior migrations remain frozen;
- any direct `windows-sys` adoption is owned only by T150 and is not pre-admitted by this file;
- identity entropy, protocol authority, observer/controller, replay bounds, recovery, and platform security are each owned by dependency-ordered slices;
- inherited live Codex/Claude nonclaims remain explicit;
- immediate pre-landing Herdr movement is reconciled without expanding Spec 011;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review finds no over-broad task, generic framework, hidden dependency, or premature downstream scope;
- fresh independent substantive review confirms no omitted FR/SC, unsafe ordering, authority ambiguity, or impossible evidence gate;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical Tasks landing may repository truth state:

```text
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=CLOSED_CANONICAL

T146=AUTHORIZED
T147..T161=BLOCKED_BY_PREDECESSOR

SPEC_011_IMPLEMENTATION_AUTHORIZED=T146_ONLY
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_LOCAL_CONTROL_IMPLEMENTATION_AUTHORIZED=NO
```

T146 is domain-only. No persistent owner process or private endpoint becomes implementation-authorized merely because the Tasks document lands.
