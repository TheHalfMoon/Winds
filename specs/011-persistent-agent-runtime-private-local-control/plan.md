# Implementation Plan: Persistent Agent Runtime & Private Local Control

**Branch**: `plan/011-persistent-agent-runtime-private-local-control`
**Created**: 2026-09-18
**Status**: Plan candidate only. Tasks, implementation, dependency adoption, lockfile mutation, migrations, owner-process creation, private endpoint creation, service installation, donor-code reuse, remote execution, plugins, marketplace, live owner handoff, and product-source changes are NOT authorized by this file alone.

## Canonical Baseline

This Plan begins from the canonically accepted Spec 011 specification:

```text
ENTRY_MERGE=4ba393ea68bdd49477a633bc4f7e2bd072d287ff
ENTRY_POST_MERGE_QUALITY=35336085572 SUCCESS ATTEMPT_1
SPEC_MERGE=5734dbb67becf386fe1e1340c2d585fcdf9f6c13
SPEC_TREE=af698f702fbd7669753111b657ff6580e4254c09
SPEC_POST_MERGE_QUALITY=35337765623 SUCCESS ATTEMPT_1

SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN_AUTHORIZED=YES
SPEC_011_TASKS_AUTHORIZED=NO
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO
```

The accepted specification contains 70 functional requirements and 25 success criteria. This Plan must make each requirement implementable without weakening earlier Winds authority, evidence, Git, process, platform, privacy, or human-decision semantics.

Inherited live-runtime nonclaims remain binding unless a later exact Tasks slice independently changes them with direct evidence:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Spec 011 does not silently authorize real Codex/Claude launch merely because it introduces persistent process ownership.

## Planning Principle

The smallest architecture that satisfies Spec 011 is:

> one narrow local owner process that owns only state which genuinely must outlive presentation clients, using OS-native private local transport and the existing Winds process/store/evidence model.

This is intentionally **not** a general daemon framework, public server, plugin host, remote plane, workflow scheduler, model gateway, or second authority system.

```text
PERSISTENT_OWNER_AUTHORITY
  = OWNED_LOCAL_PROCESS_LIFECYCLE
  + PRIVATE_LOCAL_CONTROL
  + BOUNDED_RUNTIME_REPLAY
  + EXPLICIT_CONTROLLER_LEASE
  + RUNTIME_NAMESPACE_LIFECYCLE

PERSISTENT_OWNER_AUTHORITY
  != VERIFICATION_AUTHORITY
  != HUMAN_ACCEPTANCE
  != GIT_LANDING_AUTHORITY
  != REMOTE_AUTHORITY
  != PLUGIN_AUTHORITY
```

---

# Architecture Decisions

## AD-011-01 — One owner process per Winds home/principal

The first implementation program uses at most one active Winds persistent owner per canonical Winds home and accepted local OS principal.

The owner may manage multiple runtime namespaces, but it is not a machine-wide service and is not shared across OS users.

The owner is launched on demand by an accepted Winds client using the same shipped `winds` executable in an internal owner mode. The first program does **not** install launchd, systemd, Windows Service Control Manager entries, login agents, scheduled tasks, or boot-time services.

When there are no connected clients and no live persistent runtimes, the owner SHOULD exit after a bounded idle grace period selected by Tasks. There is no requirement for an always-running background process merely because Winds is installed.

Why:

- one owner generation gives a single serialization point for runtime/controller authority;
- reusing the shipped binary avoids a second package/service lifecycle;
- no service manager is needed to prove detach/reattach while the owner remains live;
- service installation/update/handoff belongs to later separately governed work.

## AD-011-02 — Owner generation is explicit and never reconstructed from PID

Each owner start creates an immutable owner-generation identity.

A runtime namespace is bound to exactly one owner generation while live-owned. A later process using the same endpoint path, executable, PID, or display name is not the same owner generation.

The implementation MUST NOT persist a PID as live authority.

On owner-generation loss:

- every runtime previously requiring that generation becomes `OWNERSHIP_LOST` unless a separately accepted proof demonstrates continuing ownership;
- process liveness MAY remain unknown independently;
- stale generation state cannot be promoted by endpoint-file existence or PID reuse.

Generation identifiers MUST be collision-resistant. Tasks must qualify the smallest OS-appropriate entropy mechanism; this Plan does not authorize a random/UUID dependency by itself.

## AD-011-03 — Reuse current PTY/ConPTY ownership instead of replacing it

Existing accepted `portable-pty = 0.9.0` PTY/ConPTY lifecycle semantics remain the process primitive.

The persistent owner will host/reuse the existing Rust terminal ownership controller rather than introducing a second terminal implementation.

Preserved semantics include:

- canonical cwd/profile validation before launch;
- exact terminal/session identity;
- one owned output consumer;
- input/output/resize/current-size;
- owned-child exit observation/termination/reaping;
- POSIX ownership-scoped interrupt where already proven;
- fail-closed native-Windows interrupt where ownership-scoped foreground interrupt remains unproven;
- `OWNERSHIP_LOST` rather than blind PID recovery.

Spec 011 does not authorize a custom terminal engine, tmux-like process model, or Herdr PTY transplant.

## AD-011-04 — OS-native local transport only

The local-control transport is platform-specific and private:

### Linux/macOS

Use a Unix-domain stream socket owned by the accepted local principal.

The endpoint resolver MUST choose an absolute user-owned runtime directory and MUST verify its ownership and permissions before bind/connect. Tasks must define exact precedence and fallback rules, including path-length handling.

Required security properties:

- parent directory restricted to the effective user;
- endpoint not world/group writable;
- symlink/path substitution rejected;
- stale endpoint removed only after the implementation proves there is no matching live owner generation;
- peer effective user identity checked using platform primitives where available;
- Linux and macOS peer-credential code paths are qualified independently.

### Native Windows

Use a Windows named pipe, not TCP loopback.

The pipe MUST use an explicit security descriptor/DACL limited to the accepted current-user principal plus only the minimum system principals required by the implementation. Default permissive pipe security is not acceptable.

Native Windows must directly prove:

- effective DACL;
- wrong-user denial;
- current-user connection;
- stale/colliding name behavior;
- owner-generation mismatch behavior.

### Explicit non-goals

No TCP, UDP, HTTP, HTTPS, WebSocket, QUIC, local-network listener, browser-origin bridge, cloud relay, or public RPC transport is allowed in Spec 011.

## AD-011-05 — OS principal is the first-program authentication boundary

The first program authenticates local clients using the OS principal enforced by endpoint permissions/ACL plus peer identity where the platform exposes it.

No additional reusable bearer token is required in the first program because:

- the Spec explicitly does not claim isolation from malicious code already running as the same effective OS user;
- a same-user-readable token would not create a stronger meaningful boundary under that threat model;
- extra credential storage/rotation would add complexity without defending the admitted boundary.

Owner-generation identity is still required to prevent stale endpoint/generation confusion, but it is identity, not a secret credential.

A later stronger same-user isolation claim requires a separate accepted threat-model amendment.

## AD-011-06 — Versioned length-prefixed JSON protocol, not a generic RPC framework

The first protocol is Winds-owned and deliberately small.

Wire shape:

```text
u32 little-endian payload length
UTF-8 JSON payload
```

The payload MUST include:

```text
protocol_version
message_kind
connection_id
sequence
runtime_namespace_id when applicable
owner_generation_id
request/response/event body
```

The initial protocol version is `1`.

Tasks must freeze exact frame limits before implementation. Plan targets:

- maximum inbound control frame: 256 KiB;
- terminal/output event chunk: <= 64 KiB payload before framing;
- malformed length/JSON/version closes or rejects the request deterministically;
- no unbounded allocation from a claimed frame length;
- terminal bytes and agent text are always encoded data fields and are never parsed as control messages.

Existing direct `serde`/`serde_json` dependencies are sufficient for the first codec. Do not add protobuf, gRPC, Cap'n Proto, MessagePack, bincode, Tokio codecs, or a generic RPC framework unless Tasks prove a concrete requirement this protocol cannot meet.

## AD-011-07 — Blocking bounded concurrency; do not introduce Tokio for IPC

The current terminal design is synchronous/blocking and deliberately avoided Tokio solely for terminal I/O. Spec 011 preserves that simplicity.

The planned owner model is:

- one listener/accept boundary;
- bounded per-client reader/writer handling;
- one owner authority loop that serializes mutating lifecycle/controller decisions;
- existing terminal output ownership threads feeding bounded owner events;
- bounded `std::sync` queues/channels or another standard-library-equivalent mechanism where sufficient.

A client connection must never allocate an unbounded thread/queue/resource count.

Tokio or another async runtime is rejected for the first program unless a Tasks amendment proves the blocking model cannot meet measured multi-client/output requirements.

## AD-011-08 — Explicit observer/controller lease model

Each runtime namespace may have multiple observers but at most one active controller lease.

Observer authority is read-only.

Controller-only operations include at minimum:

- terminal input;
- resize;
- interrupt where supported;
- terminate/stop;
- other mutating runtime actions admitted by exact Tasks.

Plan target lease behavior:

- controller identity is explicit;
- lease is bound to owner generation + runtime namespace + client connection;
- regular liveness/heartbeat renewal is required;
- a disconnected/expired controller does not transfer authority automatically;
- takeover is an explicit accepted action;
- simultaneous takeover requests are serialized by the owner authority loop;
- the prior controller receives deterministic revocation/expiry semantics where still connected;
- observers cannot mutate merely because they hold UI focus.

Tasks must freeze exact heartbeat/lease durations. Recommended initial engineering target: 10-second heartbeat and 30-second expiry, subject to deterministic testability and platform scheduling behavior.

## AD-011-09 — Mutating-request uncertainty fails closed

Every connection receives an owner-assigned connection identity and monotonically increasing request sequence.

Within one connection:

- repeated sequence numbers are rejected or return the already-recorded deterministic disposition where Tasks choose bounded idempotency caching;
- out-of-order mutation is rejected if it violates the protocol contract.

Across connection loss:

- the client MUST NOT blindly replay a consequential mutation;
- if a mutating response was lost, the result is `OUTCOME_UNKNOWN` until the client re-queries runtime/controller state;
- the user/client may explicitly issue a new action only after reconciliation.

This avoids a persistent global idempotency ledger solely for retries while satisfying the Spec requirement that duplicated/replayed mutations never execute consequential actions blindly.

## AD-011-10 — Runtime namespace identity is durable metadata, not live ownership

A runtime namespace has an immutable Winds-generated ID and optional mutable display alias.

The runtime namespace may bind:

- canonical workspace identity where applicable;
- existing Winds Session/workstream references where applicable;
- terminal/session identity;
- execution domain/profile/cwd;
- current owner generation when proven;
- lifecycle/continuity classification;
- provider-native session identity as a separate optional reference;
- timestamps and bounded recovery metadata.

The namespace MUST NOT replace canonical Winds Session, Workstream, Project, provider-native session, terminal identity, candidate, or evidence identity.

Aliases never target consequential actions without immutable ID binding.

## AD-011-11 — Migration 0013 is the planned persistence boundary

If Tasks confirm the Plan persistence design, the next schema migration is planned as:

`migrations/0013_persistent_runtime_owner.sql`

It should persist only what is needed to reconcile namespace/lifecycle truth across client/owner restarts.

Planned durable concepts:

- runtime namespace identity and mutable alias;
- accepted workspace/session/terminal bindings;
- owner-generation record and lifecycle timestamps;
- last proven lifecycle/continuity classification;
- explicit ownership-loss/recovery reason;
- safe bounded metadata needed for restart reconciliation.

Explicitly prohibited durable data in 0013:

- OS PID as authority;
- raw process handles;
- full environment values;
- bearer credentials;
- controller lease as restart-surviving authority;
- unbounded terminal transcript;
- raw secret material;
- Git verification/acceptance authority duplicated into owner tables.

A new owner generation must reconcile previously live-owned namespaces conservatively before presenting them.

## AD-011-12 — Replay is bounded and in-memory in the first program

The first program does not persist terminal transcript/history merely to support reattach.

The owner keeps a bounded in-memory ring per live runtime for presentation continuity.

Plan targets:

- <= 8 MiB retained output/event bytes per runtime;
- <= 10,000 retained events per runtime;
- <= 64 MiB aggregate owner replay budget by default;
- explicit earliest-available sequence;
- explicit truncation/gap marker;
- deterministic ordering;
- source/trust labels preserved;
- replayed content never becomes verification or human-decision evidence.

If a runtime exceeds its per-runtime budget, oldest presentation events are evicted deterministically.

If the global owner budget is exceeded, Tasks must define deterministic fair eviction without evicting canonical lifecycle truth.

Transcript persistence, search indexing, semantic memory, or cloud history remain outside Spec 011.

## AD-011-13 — Slow clients cannot grow owner memory without bound

Each client receives a bounded outbound queue.

Plan target:

- <= 4 MiB queued payload or a smaller Tasks-frozen event-count cap per client;
- lifecycle/control truth has priority over bulk output;
- once the slow-client limit is exceeded, the owner records an explicit gap and either drops bounded presentation output or disconnects the slow observer according to the exact message class;
- a slow observer cannot stall the controller, another observer, PTY reader, or owner authority loop.

Backpressure policy must be deterministic and adversarially tested.

## AD-011-14 — No automatic owner resurrection of process authority

On owner crash/restart, the first program does not attempt to reacquire arbitrary existing child processes by PID, terminal device, provider-native ID, or OS process enumeration.

A new owner generation:

1. opens canonical store;
2. reconciles records tied to a prior generation;
3. marks unprovable live ownership `OWNERSHIP_LOST`;
4. preserves lifecycle/recovery evidence;
5. creates no new process until an explicit accepted user/client action requires it.

Native provider-session resume remains a separate future/accepted runtime adapter behavior and is never labelled retained-process continuation.

## AD-011-15 — Client integration goes through one Rust local-control client seam

Desktop, TUI, and CLI must not implement independent IPC semantics.

Create one Rust-owned local-control client seam that owns:

- endpoint resolution;
- principal/generation handshake;
- protocol versioning;
- request/response correlation;
- observer/controller semantics;
- reconnect/state reconciliation;
- bounded replay decoding.

The Tauri renderer never connects to the owner endpoint directly. Desktop uses the existing trusted Rust host boundary, which calls the shared Rust client seam and returns bounded typed projections to the renderer.

Terminal/agent text remains untrusted through every layer.

## AD-011-16 — No new provider execution authority

The persistent owner may host already-authorized process kinds only.

The first implementation program MUST NOT turn fixture-only or unauthorized direct Codex/Claude execution into live provider authority.

Spec 006/009/010 nonclaims remain unchanged until separately authorized and directly proven.

Persistence of a shell/PTY process proves persistent process ownership only; it does not prove provider/model identity or agent correctness.

## AD-011-17 — Herdr remains design/test evidence; no donor code is admitted by this Plan

The accepted Entry/Spec research pin is Herdr `7df919d00e5bf8f6ed43a1781e81340cc0bbce8d`, tree `647a010ad83d983943ecfe00f82b6a376f1bfbf1`.

The Plan uses Herdr only to challenge missing behavior and tests:

- persistent server ownership;
- detach/reattach;
- observer/controller distinction;
- local socket/named-pipe security;
- stale endpoint handling;
- version mismatch;
- native session resume distinction;
- bounded state transfer.

No Herdr source file is admitted for direct/adapted copying by this Plan.

Any later Tasks proposal for reuse must record exact source path, exact upstream revision, Apache-2.0 notice/copyright requirements, underlying vendor provenance, destination path, modification record, threat-model delta, tests, and removal/update path.

## AD-011-18 — Direct dependency posture

No dependency is adopted by this Plan.

Tasks should first attempt implementation using current direct dependencies and standard library primitives.

Current relevant direct dependencies include:

- `libc = 0.2.189`;
- `portable-pty = 0.9.0`;
- `rusqlite = 0.40.2`;
- `serde` + `serde_json`.

For native Windows pipe creation/security, Tasks MAY evaluate adding direct `windows-sys = 0.61.2`, which is already present transitively in the current lockfile, with the smallest exact Win32 feature set needed for named pipes, security descriptors/SIDs, handles, and process/user identity.

Transitive presence is not direct admission. Tasks must still audit exact checksum/license/MSRV/platform graph and explain why direct Win32 FFI through the already-observed package is smaller/safer than another abstraction.

Do not add `tokio`, `interprocess`, gRPC, protobuf, a service-manager crate, a generic RPC crate, or a UUID/random crate unless an exact task proves necessity.

---

# Local-Control Protocol Model

## Handshake

Before any runtime metadata or control operation is exposed:

1. transport-level OS-principal restriction succeeds;
2. client sends supported protocol range and expected owner generation if reconnecting;
3. owner returns exact protocol version, owner generation, and assigned connection ID;
4. generation mismatch is explicit;
5. the client must request runtime inventory under observer authority before requesting control;
6. no automatic protocol downgrade below the accepted version.

A stale client that expected an old generation cannot silently attach to a new generation as if continuity were live.

## Message classes

The first protocol may define only the minimum typed classes needed by Tasks, grouped conceptually as:

```text
HELLO / HELLO_ACK / ERROR
LIST_RUNTIMES / RUNTIME_SNAPSHOT
ATTACH_OBSERVER / DETACH
REQUEST_CONTROL / RELEASE_CONTROL / CONTROL_STATE
INPUT / RESIZE / INTERRUPT / STOP
RUNTIME_EVENT / OUTPUT_EVENT / HISTORY_GAP
PING / PONG
OWNER_STATUS / SHUTDOWN_IF_IDLE
```

This list is a planning ceiling, not automatic Tasks authority. Tasks may reduce it.

Do not expose arbitrary shell execution, generic filesystem operations, generic Git commands, raw SQL, environment dump, plugin calls, remote calls, or a generic method dispatcher.

## Errors

Errors are typed and source-owned, with at least conceptual distinctions for:

- protocol mismatch;
- authentication/principal denial;
- stale owner generation;
- unknown runtime;
- ownership lost;
- observer lacks control;
- controller conflict;
- malformed/oversized frame;
- duplicate/replayed mutation;
- slow client/backpressure;
- unsupported operation/platform;
- outcome unknown after connection loss;
- internal owner failure without false success.

String matching must not be the authority contract.

---

# Persistence and Recovery Model

## Store ownership

The existing Winds SQLite store remains canonical for durable runtime metadata.

The owner may hold a store connection or a bounded store facade, but it does not become a second database authority.

Clients do not mutate 0013 owner/runtime rows directly while an owner generation is live. Mutations flow through accepted Rust domain methods so lifecycle transitions remain serialized.

## Startup reconciliation

Owner startup must be explicit:

1. acquire/validate local owner singleton boundary;
2. create new generation identity;
3. open/migrate store if exact Tasks authorized migration 0013;
4. reconcile all prior-generation active runtime rows to fail-closed truth;
5. create private endpoint;
6. expose ready state only after endpoint security and generation/store reconciliation succeed.

No endpoint may advertise ready before stale-state reconciliation completes.

## Shutdown

A clean owner shutdown:

- stops accepting new control operations;
- resolves or rejects in-flight mutations deterministically;
- preserves or stops child runtimes according to the exact accepted shutdown action;
- records every runtime disposition independently;
- removes endpoint state only when ownership of that endpoint is proven;
- never reports clean stop for unresolved child ownership.

The first program does not perform live binary handoff.

---

# Threat Model Implementation Plan

## Different local user

Linux/macOS qualification must attempt wrong-UID connection where the test environment allows it.

Windows qualification must directly inspect and exercise named-pipe DACL denial for an unauthorized principal or equivalent controlled fixture.

Passing same-user tests is not evidence for cross-user isolation.

## Stale endpoint / path replacement

Tests must cover:

- stale socket path;
- attacker-created regular file;
- symlink path;
- directory ownership mismatch;
- live old owner;
- dead old owner;
- new generation collision;
- Windows named-pipe collision.

Deletion/replacement fails closed when identity is ambiguous.

## Malformed protocol

Fuzz-like deterministic fixtures cover:

- zero/oversized length;
- truncated body;
- invalid UTF-8/JSON;
- unknown message kind;
- wrong protocol version;
- missing/incorrect generation;
- invalid sequence;
- runtime ID mismatch;
- terminal bytes resembling JSON/control frames.

## Controller races

Deterministic barriers exercise:

- simultaneous takeover;
- input concurrent with takeover;
- resize concurrent with takeover;
- stop concurrent with takeover;
- controller disconnect;
- lease expiry/renewal race;
- observer attempting mutation.

Exactly one authoritative mutation ordering must result.

## Resource exhaustion

Campaigns cover:

- many observers;
- slow reader;
- high-output PTY;
- repeated reconnects;
- replay-budget exhaustion;
- malformed-frame floods within test bounds;
- idle owner with no runtimes;
- many idle runtimes.

---

# Platform Plan

## Linux

Claim only after direct qualification of:

- Unix socket path/permissions;
- peer credential behavior;
- PTY continuity;
- observer/controller semantics;
- owner crash/restart;
- replay/backpressure;
- resource budgets.

## macOS

Qualify separately from Linux:

- Unix socket directory/path behavior;
- peer identity mechanism;
- path-length/fallback behavior;
- PTY continuity;
- lifecycle/resource campaigns.

Linux evidence must not substitute for macOS.

## Native Windows

Claim only after official Windows CI/runtime proof of:

- named-pipe security descriptor/DACL;
- current-user connection and wrong-principal denial fixture where feasible;
- generation collision/stale name behavior;
- ConPTY create/input/output/resize/exit/stop continuity;
- explicit preserved fail-closed interrupt limitation;
- owner/client crash and reconnect;
- resource cleanup.

## WSL2

WSL remains a distinct execution domain. Spec 011 does not create a Windows-host ↔ WSL owner bridge.

A Linux owner running inside WSL may be treated as a Linux-domain owner only after direct WSL evidence. Native-Windows named-pipe claims do not transfer to WSL Unix-socket claims.

---

# Performance and Resource Budgets

Tasks must freeze reproducible reference environments. Initial Plan ceilings:

```text
owner idle CPU with zero live runtimes: <= 1% of one logical core p95
owner idle RSS with zero live runtimes: <= 64 MiB
reattach handshake + runtime snapshot local p95: <= 100 ms
cached observer attach local p95: <= 100 ms
controller input dispatch overhead p95: <= 10 ms excluding child processing
resize dispatch overhead p95: <= 25 ms excluding child processing
per-runtime replay budget: <= 8 MiB
aggregate replay budget default: <= 64 MiB
per-client queued outbound payload: <= 4 MiB
output stress: >= 10 MiB and >= 100,000 logical lines
observer stress: >= 8 concurrent local observers to one runtime
runtime stress: >= 32 idle runtime namespaces
reconnect churn: >= 500 attach/detach cycles in deterministic campaign
```

These are engineering ceilings, not product marketing claims. Tasks may tighten them; relaxation requires a documented Plan amendment with evidence.

No performance optimization may weaken principal checks, generation checks, controller authority, bounds, stale-state rejection, or lifecycle truth.

---

# Dependency and Provenance Gates

Before any dependency or donor slice is adopted, the owning Task must prove:

- exact version/revision and checksum;
- license and notice obligations;
- MSRV compatibility with Rust 1.97.1;
- supported platform/domain;
- transitive graph impact;
- security advisories relevant to admitted features;
- why standard library/current dependencies are insufficient;
- smallest feature set;
- failure/upgrade/removal path.

For `windows-sys 0.61.2`, if directly adopted, record exact selected Win32 feature modules and prove no broad feature expansion.

For Herdr, if any later exact slice is proposed, root Apache-2.0 is not blanket admission for vendored/third-party material.

---

# Implementation-Boundary Map

The Plan expects Tasks to create or adapt Winds-owned modules approximately along these seams; exact filenames are Tasks decisions:

```text
runtime_owner/
  domain        immutable IDs, generation, lifecycle, continuity
  owner         authority loop / runtime registry
  protocol      versioned typed messages + framing limits
  transport     POSIX Unix socket / Windows named pipe
  peer          principal identity checks
  replay        bounded in-memory history
  controller    observer/controller lease
  persistence   0013 mappings and restart reconciliation
  client        shared Rust client seam
```

Existing modules remain authoritative for:

- PTY/ConPTY lifecycle: current execution/terminal code;
- canonical workspace/session/workflow/model-mesh identity;
- SQLite store/migrations;
- Git/check/verification;
- desktop trusted Rust bridge;
- evidence/human-decision truth.

No duplicate verification store or duplicate terminal backend is planned.

---

# Proposed Dependency-Ordered Implementation Slices

Tasks should decompose the implementation into the smallest independently reviewable units, approximately:

1. **Domain freeze** — runtime namespace, owner generation, continuity/ownership states, observer/controller types, protocol error vocabulary; no process or IPC.
2. **Persistence/reconciliation** — migration 0013 and fail-closed prior-generation reconciliation; no endpoint/process continuation yet.
3. **Protocol codec** — version-1 bounded framing/messages and malformed/duplicate fixtures over in-memory streams.
4. **POSIX private transport** — secure Unix endpoint + Linux/macOS peer identity and stale-path defenses; no PTY ownership migration yet.
5. **Windows private transport** — named-pipe DACL/current-user semantics on official Windows; no cross-platform inference.
6. **Owner process shell** — singleton generation/start/readiness/idle shutdown with zero child authority.
7. **Persistent PTY ownership** — move/reuse accepted PTY/ConPTY ownership under owner for a bounded shell fixture; preserve interrupt limitations.
8. **Observer/replay** — multiple read-only clients, bounded history, gap/backpressure semantics.
9. **Controller lease** — explicit controller, takeover/release/expiry/race tests; no implicit transfer/broadcast.
10. **Shared Rust client** — attach/reconnect/generation reconciliation used by CLI/TUI/Desktop Rust host; renderer remains indirect.
11. **Detach/reattach UX/projections** — exact continuity classes and `OWNERSHIP_LOST` surfaced through existing clients without new provider claims.
12. **Stop/crash/recovery** — owner/client crash, uncertain outcome, cleanup, stale endpoint, no blind PID recovery.
13. **Adversarial protocol/security** — wrong principal, path replacement, malformed/oversized/replayed frames, forged terminal content.
14. **Stress/performance** — high output, slow observers, many clients/runtimes, reconnect churn, resource ceilings.
15. **Platform qualification** — Linux/macOS/native Windows direct evidence; WSL claim only if separately exercised.
16. **Final Spec 011 reconciliation** — map all 70 FR and 25 SC to exact canonical evidence, preserve historical failures/nonclaims, and close only after guarded landing/post-merge proof.

Tasks may split any slice further. They must not combine high-authority transport, persistence, process ownership, and controller behavior into one unreviewable first implementation PR.

---

# Explicit Rejections for the First Spec 011 Program

Unless a separately accepted amendment changes scope, reject:

- OS service installation/autostart;
- TCP/HTTP/WebSocket/public RPC;
- remote machines/SSH/cloud relay;
- generic daemon/service framework;
- Tokio solely for local IPC;
- generic RPC/IDL framework;
- persistent transcript database;
- semantic/vector memory;
- complete Herdr 104-method API;
- broad agent/integration parity;
- plugin runtime/marketplace;
- live owner binary replacement/handoff;
- self-update;
- provider/model gateway;
- automatic provider routing;
- browser runtime;
- generic filesystem/Git/SQL command methods;
- renderer-direct owner access;
- automatic Git merge/rebase/cherry-pick/push/PR/landing;
- PID reattachment;
- process enumeration as ownership proof;
- hidden same-user sandbox/isolation claims;
- bulk copying Herdr source.

---

# Verification Strategy

Each implementation slice must have deterministic unit/integration tests before promotion.

The program must ultimately include:

- pure domain/state-machine tests;
- migration upgrade and rollback/recovery fixtures;
- protocol framing/version/error tests;
- POSIX endpoint permission/peer fixtures;
- official native-Windows named-pipe ACL fixtures;
- detach/reattach exact-runtime tests;
- observer/controller race tests;
- owner/client crash tests;
- stale endpoint/generation/PID-reuse tests;
- high-output/replay/backpressure tests;
- forged terminal/agent protocol-shaped output tests;
- restart reconciliation proving `OWNERSHIP_LOST`;
- existing terminal lifecycle regression tests;
- existing Spec 003/006/007/008/009/010 regression suites;
- no-change evidence for verification/Git/human-decision authority;
- exact platform evidence for every released claim.

No single green unit test may stand in for real platform-security proof.

---

# Review and Landing Discipline

Every Tasks/implementation candidate must satisfy repository governance:

1. exact authorized slice only;
2. deterministic checks on exact head;
3. correctness/safety review;
4. Ponytail/YAGNI review;
5. fresh independent substantive review;
6. zero unresolved material findings/threads;
7. exact base/head/tree/scope/ruleset/mergeability reconciliation;
8. guarded normal merge with expected head;
9. merge tree/ordered-parent/signature verification;
10. every actually-triggered post-merge push workflow succeeds.

Failed candidates/runs remain historical evidence and must not be relabelled.

Moving a candidate after review invalidates candidate-bound acceptance evidence unless the governing review explicitly covers the successor.

---

# Plan Acceptance Gate

This Plan may land only if the exact final candidate proves:

- changed scope is exactly `specs/011-persistent-agent-runtime-private-local-control/plan.md` unless a separately justified governance correction is required;
- Spec 011 Entry and Spec are canonically closed and their post-merge quality runs are successful;
- all 70 FR and 25 SC have an implementation path in this Plan;
- persistent ownership is narrower than a general daemon/service framework;
- no public/network/remote/plugin/marketplace/update/handoff scope is introduced;
- OS-principal authentication and same-user nonclaim are explicit;
- Linux/macOS Unix-socket and native-Windows named-pipe security are distinct and directly qualified;
- no PID is process authority;
- owner generation, runtime namespace, canonical Session/workstream/provider/candidate/evidence identities remain separate;
- observer/controller authority is explicit and singular;
- protocol framing, versioning, bounds, replay/duplicate/slow-client behavior are defined;
- bounded in-memory replay is distinct from evidence and no transcript persistence is introduced;
- restart/owner-loss reconciliation fails closed;
- existing PTY/ConPTY lifecycle is reused rather than replaced;
- the renderer cannot connect directly or gain generic host authority;
- existing live Codex/Claude nonclaims remain unchanged;
- Herdr remains research evidence only and donor code is not admitted by this Plan;
- direct dependency candidates remain Tasks-gated;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/architecture review passes;
- Ponytail/YAGNI review challenges owner scope, transport, protocol, persistence, replay, dependency, and concurrency complexity;
- fresh independent substantive review challenges threat model, platform security, authority separation, failure semantics, resource bounds, and scope;
- zero unresolved material findings/review threads;
- immediate pre-landing Herdr freshness is reconciled without silently expanding scope;
- exact base/head/tree/scope/ruleset/mergeability is reconciled;
- guarded expected-head normal merge succeeds;
- merge tree/ordered parents/GitHub signature are verified;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical Plan landing may repository truth state:

```text
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS_AUTHORIZED=YES
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO

PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_LOCAL_CONTROL_IMPLEMENTATION_AUTHORIZED=NO
PUBLIC_RUNTIME_PROTOCOL_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_OWNER_HANDOFF_AUTHORIZED=NO
```

No production source, dependency, lockfile, migration, owner process, endpoint, or donor-code change may begin until a separate exact Tasks candidate is independently qualified and canonically accepted.
