# Feature Specification: Persistent Agent Runtime & Private Local Control

**Feature Branch**: `spec/011-persistent-agent-runtime-private-local-control`  
**Created**: 2026-09-18  
**Status**: Specification candidate only. Plan, Tasks, dependencies, migrations, persistent-owner implementation, private local-control implementation, service installation, remote execution, plugin runtime, marketplace, live owner handoff, and product-source changes are NOT authorized by this file alone.  
**Input**: Canonical Spec 011 Entry Gate for the Founder-directed Herdr parity program. The immediate product problem is truthful local continuity: Winds-owned terminal/agent processes may outlive an individual client only when a narrow Winds-owned runtime authority actually retains ownership, and local clients must reattach through an authenticated, versioned, least-authority private control surface without converting persisted metadata, PIDs, terminal text, or native provider session IDs into process authority.

## Product Thesis

Winds needs a durable local execution substrate, but not a general daemon framework.

The purpose of Spec 011 is to define the smallest local runtime contract that lets approved Winds-owned terminal/agent work continue when a desktop/TUI/CLI client detaches, then lets later local clients reattach to the **same proven owner and runtime namespace** without weakening the evidence, Git, verification, human-decision, credential, or platform boundaries already accepted in Specs 003–010.

```text
PERSISTENT_OWNER_LIVE != CHILD_LIVE_UNLESS_OWNERSHIP_IS_PROVEN
PERSISTED_PID != PROCESS_IDENTITY
PERSISTED_ROW != LIVE_OWNERSHIP
RUNTIME_NAMESPACE != CANONICAL_WINDS_SESSION
RUNTIME_NAMESPACE != WORKSTREAM_OR_TASK
RUNTIME_NAMESPACE != PROVIDER_NATIVE_SESSION
RUNTIME_NAMESPACE != GIT_CANDIDATE
CLIENT_FOCUS != CONTROLLER_AUTHORITY
OBSERVER_AUTHORITY != CONTROLLER_AUTHORITY
LIVE_PROCESS_OWNERSHIP != NATIVE_AGENT_RESUME
LIVE_PROCESS_OWNERSHIP != WINDS_RECONSTRUCTION
REPLAYED_HISTORY != CANONICAL_EVIDENCE
PROCESS_EXIT != VERIFIED != ACCEPTED
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
```

Spec 011 defines product behavior, trust boundaries, failure semantics, and measurable outcomes. It does not choose a transport library, async runtime, process supervisor framework, service manager, IPC codec, database migration, authentication library, or donor-code slice.

## Canonical Baseline

Spec 011 begins only after the canonical Entry Gate landed as:

```text
ENTRY_MERGE=4ba393ea68bdd49477a633bc4f7e2bd072d287ff
ENTRY_TREE=6335db9bb9f872ea6b996450646139ebba6af54b
POST_ENTRY_QUALITY=35336085572 SUCCESS ATTEMPT_1
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_FORMAL_SPEC_AUTHORIZED=YES
SPEC_011_PLAN_AUTHORIZED=NO
SPEC_011_TASKS_AUTHORIZED=NO
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_LOCAL_CONTROL_IMPLEMENTATION_AUTHORIZED=NO
```

Accepted Winds behavior that this Spec MUST preserve includes:

- real PTY ownership on POSIX and ConPTY-equivalent ownership on accepted native Windows paths;
- exact terminal/session identity for create/input/output/resize/exit/terminate/close;
- retained-child/process-scope cleanup only where Winds can prove ownership;
- explicit fail-closed native-Windows interrupt semantics where ownership-scoped interrupt is not proven;
- restart reconciliation to `OWNERSHIP_LOST` when continuing ownership cannot be proven;
- no blind PID lookup/signal/kill as recovery authority;
- canonical Project/Session/workstream/runtime/provider/model/candidate/evidence identities remaining distinct;
- desktop/WebView/TUI presentation remaining non-authoritative;
- verification and human acceptance remaining separate from process/runtime status;
- local-first operation without a mandatory cloud control plane;
- existing Git safety and exact-candidate governance remaining independent from execution continuity.

## Entry-Time Herdr Research Pin

The Entry Gate reconciled Herdr at:

```text
HERDR_ENTRY_PIN=7df919d00e5bf8f6ed43a1781e81340cc0bbce8d
HERDR_ENTRY_TREE=647a010ad83d983943ecfe00f82b6a376f1bfbf1
AGENT_FAMILIES=24
PUBLIC_SERIALIZED_API_METHODS=104
FROZEN_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INTEGRATIONS=1
BASELINE_PARITY_LEDGER_ROWS=218
```

The one-commit drift from the canonical research refresh affected Grok agent-detection manifest/test material, not the persistent-owner/private-local-control problem. Herdr remains research/design evidence only. No source is admitted for copying by this Spec.

---

# User Scenarios & Testing

## User Story 1 — Detach Without Losing a Proven Live Process (Priority: P1)

A developer starts a Winds-owned terminal or admitted agent process, then closes or disconnects the current presentation client. The work continues only because the accepted persistent owner still holds the live ownership primitive. The user later sees truthful continuity rather than a reconstructed illusion based on a PID or remembered session row.

**Independent Test**: Start a long-running PTY-backed process under the future accepted owner, detach every presentation client, prove output/liveness continues under retained ownership, then reattach and prove the exact same runtime namespace and owned process are observed.

**Acceptance Scenarios**:

1. **Given** a live Winds-owned process and an attached client, **When** the client detaches cleanly, **Then** the process may continue only if the persistent owner remains live and retains exact ownership.
2. **Given** a client exits unexpectedly, **When** the owner remains healthy, **Then** the owner does not terminate the child merely because the client disappeared unless explicit lifecycle policy requires it.
3. **Given** persisted metadata but no provable owner, **When** a later client starts, **Then** Winds MUST NOT claim the process is still live.
4. **Given** an owner whose child exited while no client was attached, **When** a client reconnects, **Then** Winds reports the observed final lifecycle state without inventing continued liveness.

## User Story 2 — Reattach to the Exact Runtime Namespace (Priority: P1)

A developer chooses a remembered Winds runtime and reconnects to the exact live owner/runtime namespace. Similar display names, stale endpoint files, reused OS PIDs, stale credentials, or provider-native session IDs cannot silently redirect control to another runtime.

**Independent Test**: Create multiple similar runtime namespaces, stale metadata, endpoint collisions, and PID reuse fixtures. Reattach must either bind to the exact intended live runtime identity or fail closed with an explicit reason.

**Acceptance Scenarios**:

1. **Given** two runtimes with identical display aliases, **When** the user selects one, **Then** the request binds to immutable runtime identity rather than alias.
2. **Given** stale endpoint metadata for a dead owner, **When** a client attempts reattach, **Then** stale state cannot impersonate a live owner.
3. **Given** a reused OS PID, **When** persisted metadata references the old process, **Then** PID equality alone cannot prove identity or authorize control.
4. **Given** exact live owner identity but incompatible protocol version, **When** a client connects, **Then** reattach fails explicitly rather than downgrading silently.

## User Story 3 — Multiple Observers, One Explicit Controller (Priority: P1)

Several local clients may observe one runtime, but observation never grants input, resize, interrupt, terminate, or other control authority. Writable authority is explicit, attributable, and race-safe.

**Independent Test**: Attach multiple observer clients and one controller, then exercise simultaneous input, resize, takeover, disconnect, lease expiry, and terminate races. At no point may two clients accidentally become implicit controllers.

**Acceptance Scenarios**:

1. **Given** two observers and one controller, **When** an observer sends a mutating action, **Then** the request is rejected without affecting runtime state.
2. **Given** an explicit controller, **When** another eligible client requests takeover, **Then** prior-controller disposition is deterministic and attributable.
3. **Given** the controller disconnects, **When** no valid successor exists, **Then** observer clients do not inherit write authority automatically.
4. **Given** simultaneous takeover requests, **When** the owner resolves them, **Then** exactly one accepted controller outcome exists under the specified ordering/lease rules.

## User Story 4 — Owner Crash and Ownership Loss Are Truthful (Priority: P1)

If the persistent owner crashes, is killed, is replaced unexpectedly, or becomes unreachable, Winds must not infer live child ownership from historical metadata. Process liveness and Winds ownership may become unknown independently.

**Independent Test**: Crash/kill the owner at controlled points while child processes are running, with and without reconnectable OS state. Verify every client and persisted record converges to a fail-closed ownership state and no blind signal/kill occurs.

**Acceptance Scenarios**:

1. **Given** a vanished owner and persisted runtime state, **When** a client opens Winds, **Then** the runtime cannot be labelled live-owned until ownership is independently proven.
2. **Given** a process may still exist after owner loss, **When** Winds cannot prove ownership, **Then** it reports ownership lost/process state unknown rather than attaching by PID.
3. **Given** owner recovery fails, **When** cleanup is uncertain, **Then** Winds preserves recovery evidence and does not report successful cleanup.
4. **Given** a restarted owner with a new identity, **When** stale clients reconnect, **Then** they cannot silently treat the new owner as the prior ownership generation.

## User Story 5 — Private Local Control Fails Closed (Priority: P1)

A local client communicates with the persistent owner through a private, versioned, authenticated control surface scoped to the current machine and accepted local principal. The surface is not a public SDK/server and cannot be reached through remote origins merely because a transport exists.

**Independent Test**: Exercise wrong local user/principal, stale credentials, endpoint replacement, malformed/oversized messages, unsupported protocol version, duplicate/replayed mutating requests, slow clients, and forged terminal content. Each must fail within explicit bounds without expanding authority.

**Acceptance Scenarios**:

1. **Given** a client outside the accepted local principal boundary, **When** it attempts connection, **Then** it is rejected before privileged runtime state is exposed.
2. **Given** incompatible protocol versions, **When** the client connects, **Then** the mismatch is explicit and no unsafe compatibility fallback occurs.
3. **Given** duplicated/replayed mutating request identity, **When** the owner receives it, **Then** behavior follows deterministic idempotency/rejection semantics rather than repeating consequential action blindly.
4. **Given** terminal/agent output that resembles a control frame, **When** rendered or replayed, **Then** it remains data and cannot become protocol input.

## User Story 6 — Bounded Replay Restores Context, Not Authority (Priority: P1)

A reconnecting client receives enough recent output/events/topology to render useful continuity without requiring an unbounded transcript and without treating retained history as verification evidence.

**Independent Test**: Produce output beyond replay limits, detach, reconnect, and prove ordering/truncation/source metadata are explicit. A replayed `VERIFIED` or approval-looking string cannot become trusted Winds state.

**Acceptance Scenarios**:

1. **Given** history larger than the replay bound, **When** a client reconnects, **Then** truncation is explicit and deterministic.
2. **Given** replay gaps or evicted material, **When** presented, **Then** Winds does not visually imply complete history.
3. **Given** replayed agent/terminal claims about tests or approvals, **When** displayed, **Then** they remain source-labelled historical output.
4. **Given** a slow observer, **When** its queue exceeds defined limits, **Then** owner memory remains bounded and the client receives explicit loss/backpressure/disconnect semantics.

## User Story 7 — Native Agent Resume Is Not Live Process Continuity (Priority: P1)

When an admitted runtime exposes a provider-native conversation/session resume mechanism, Winds keeps that semantic separate from retained live process ownership and from Winds reconstruction.

**Independent Test**: Exercise fixtures for retained live process, provider-native resume after process loss, Winds reconstruction, and fresh process creation. Each must produce a distinct continuity classification.

**Acceptance Scenarios**:

1. **Given** retained child ownership, **When** the client reconnects, **Then** continuity may be classified as live retained-process continuation.
2. **Given** no live owned process but a valid provider-native session resume, **When** resumed later, **Then** it is labelled native resume, not live ownership.
3. **Given** only canonical Winds workflow/context state, **When** a new process is created, **Then** it is reconstruction/reassignment, not resume.
4. **Given** an agent claims it resumed, **When** no accepted runtime evidence supports that claim, **Then** Winds does not upgrade continuity class.

## User Story 8 — Explicit Stop, Cleanup, and Recovery (Priority: P1)

The user can stop a runtime/owner intentionally, and Winds produces bounded, truthful cleanup/recovery state. Failure to terminate or reap an owned process is not reported as success.

**Independent Test**: Exercise clean stop, stuck child, client disconnect during stop, owner crash during stop, partial metadata write, and repeated stop request.

**Acceptance Scenarios**:

1. **Given** a healthy owned runtime, **When** explicit stop succeeds, **Then** final lifecycle and cleanup evidence are deterministic.
2. **Given** cleanup times out or ownership becomes uncertain, **When** stop completes procedurally, **Then** Winds reports unresolved/ownership-lost state rather than `STOPPED` success.
3. **Given** a repeated stop request, **When** the runtime is already terminal, **Then** the outcome is safe and deterministic.
4. **Given** owner shutdown with multiple runtimes, **When** policy requires preservation or termination, **Then** each runtime receives an independently attributable lifecycle result.

## User Story 9 — Local Endpoint Security Is Platform-Truthful (Priority: P1)

On every platform where Spec 011 support is claimed, the private endpoint must enforce the accepted local-principal boundary using platform-appropriate primitives. Winds does not infer Windows security from Unix permissions or vice versa.

**Independent Test**: Directly exercise endpoint creation/ownership/permissions or ACL policy, stale endpoint replacement, wrong-principal access, and path/namespace collision on each claimed platform domain.

**Acceptance Scenarios**:

1. **Given** a POSIX-class claimed platform, **When** endpoint state is created, **Then** parent/path ownership and access policy match the explicit threat model.
2. **Given** native Windows support is claimed, **When** endpoint state is created, **Then** effective DACL/principal restrictions are directly proven on native Windows.
3. **Given** WSL support is discussed, **When** tested, **Then** WSL and native Windows endpoint/security claims remain distinct.
4. **Given** malicious code already running as the same effective OS user, **When** considering the threat model, **Then** Winds MUST NOT claim isolation unless a stronger separately proven boundary exists.

## User Story 10 — Persistent Ownership Remains Lightweight and Local (Priority: P2)

The owner can remain idle without becoming a resource-heavy orchestration service. A user with no active persistent work should not pay unbounded CPU, memory, file-handle, or disk-history cost.

**Independent Test**: Measure idle owner resource use, many idle runtimes/observers, high-output sessions, reconnect churn, and bounded-history storage under Plan-defined ceilings.

---

# Threat Model

Spec 011 protects process-control authority, exact runtime identity, local-control confidentiality/integrity, canonical Winds identities, lifecycle truth, and bounded retained history.

Adversarial or failure conditions that the Plan/Tasks MUST qualify include:

- a different local OS user/principal attempting to discover or control the endpoint;
- stale or attacker-created endpoint/path/marker replacement;
- symlink/reparse/path substitution where applicable;
- guessed runtime namespace/endpoint identifiers;
- stale, stolen, replayed, or cross-runtime credentials/tokens where such credentials exist;
- a compromised or malicious presentation client that has only observer authority;
- malformed/oversized/duplicate/replayed control frames;
- slow-reader/backpressure/resource-exhaustion behavior;
- concurrent controller takeover/input/resize/interrupt/terminate races;
- owner crash, client crash, partial persistence, abrupt machine shutdown, and endpoint leftovers;
- forged terminal/agent content shaped like trusted protocol/events/approvals/evidence;
- PID reuse and stale persisted lifecycle rows;
- incompatible protocol/client versions and downgrade attempts;
- accidental exposure through public network listeners, browser/WebView origins, or remote relays.

Explicit nonclaim:

> Spec 011 does not promise isolation from arbitrary malicious code already executing with the same effective OS user/principal unless the later Plan selects and directly proves an additional enforceable security boundary.

---

# Functional Requirements

- **FR-001**: Every persistent runtime namespace MUST have an immutable Winds-generated identity independent of aliases, OS PID, terminal ID, provider-native session ID, Project, Session display name, or workstream name.
- **FR-002**: Persisted runtime metadata MUST NOT by itself prove a live owner or live child process.
- **FR-003**: A live-process claim MUST derive from an accepted owner-held ownership primitive or another Plan-defined proof with equivalent strength; PID equality alone is insufficient.
- **FR-004**: When continuing ownership cannot be proven, the runtime MUST fail closed to `OWNERSHIP_LOST`, `UNAVAILABLE`, or another explicitly specified non-live-owned state rather than remain `RUNNING`.
- **FR-005**: Process liveness and Winds ownership MUST be representable separately when one is known and the other is not.
- **FR-006**: Client detach MUST NOT terminate an owned child solely because the presentation connection ended unless explicit lifecycle policy for that runtime says otherwise.
- **FR-007**: Reattach MUST bind to exact runtime namespace and owner generation; aliases and stale endpoint markers MUST NOT redirect control.
- **FR-008**: Owner generation/restart identity MUST prevent stale clients from silently treating a replacement owner as the prior live owner.
- **FR-009**: Runtime namespace identity MUST remain distinct from canonical Winds Session, Workstream/Task, provider-native session, Project presentation identity, and Git candidate/evidence identity.
- **FR-010**: Native provider session resume MUST remain semantically distinct from retained live-process ownership.
- **FR-011**: Winds reconstruction/reassignment MUST remain distinct from both native provider resume and retained live-process continuation.
- **FR-012**: Every exposed continuity state MUST identify its source/proof class sufficiently for UI/CLI surfaces to preserve those distinctions.
- **FR-013**: More than one local client MAY observe a runtime, but observer authority MUST exclude input, resize, interrupt, terminate, controller transfer, privileged configuration, and equivalent mutations.
- **FR-014**: Write/control authority MUST be explicit, attributable, bounded, and unambiguous for each runtime.
- **FR-015**: Client focus, visibility, connection order, or being the most recent observer MUST NOT implicitly grant controller authority.
- **FR-016**: Controller acquisition/takeover MUST have deterministic conflict resolution and prior-controller disposition.
- **FR-017**: Controller disconnect, timeout, owner restart, and lease expiration MUST have explicit deterministic behavior.
- **FR-018**: Simultaneous takeover/input/resize/interrupt/terminate races MUST NOT produce multiple implicit controllers or cross-runtime action.
- **FR-019**: Input and control actions MUST target exactly one runtime by default; no connection, Project membership, or visual grouping may create implicit broadcast.
- **FR-020**: Every mutating request MUST carry an exact target identity and request identity sufficient for deterministic correlation and replay handling.
- **FR-021**: The private local-control protocol MUST be explicitly versioned.
- **FR-022**: Incompatible versions MUST fail closed with an explicit error; silent unsafe downgrade or unversioned fallback is prohibited.
- **FR-023**: Request/response correlation MUST prevent a response from being silently attributed to the wrong runtime, request, or owner generation.
- **FR-024**: Duplicate/replayed mutating requests MUST follow explicit idempotency/rejection semantics appropriate to the operation.
- **FR-025**: Messages/frames, queues, replay windows, and retained history MUST have Plan-defined hard bounds.
- **FR-026**: Malformed or oversized control data MUST fail within bounded resource use and MUST NOT crash the owner or expand authority.
- **FR-027**: Slow or non-reading clients MUST have explicit backpressure, drop, truncation, or disconnect semantics that keep owner resource use bounded.
- **FR-028**: Terminal/agent output MUST remain untrusted data and MUST NOT be interpreted as control protocol, trusted lifecycle truth, approval, evidence, or host action.
- **FR-029**: The private control surface MUST remain local-only in Spec 011; public TCP/HTTP/WebSocket/LAN/cloud/remote-origin command paths are prohibited.
- **FR-030**: Each claimed platform MUST enforce an explicit local-principal endpoint-access policy using platform-appropriate primitives directly proven on that platform.
- **FR-031**: Endpoint parent/path/namespace creation MUST resist stale-state replacement and path/symlink/reparse ambiguity under the accepted threat model.
- **FR-032**: An endpoint MUST NOT be removed or replaced merely because a path exists; Winds MUST distinguish a live owner from stale state before destructive replacement.
- **FR-033**: Endpoint/runtime namespace identifiers MUST be sufficiently collision-resistant or otherwise uniquely bound to exact runtime identity.
- **FR-034**: Client authentication/peer validation MUST define which local OS/process/user principal is trusted and how that fact is proven.
- **FR-035**: If an additional local secret/challenge/token is used, its creation, storage, rotation, expiry, revocation, and cross-runtime binding MUST be specified by Plan/Tasks and MUST NOT be logged or persisted as ordinary history.
- **FR-036**: Same-effective-user malicious-code isolation MUST NOT be claimed unless a stronger enforceable boundary is separately selected and proven.
- **FR-037**: Remote clients, SSH transport, cloud relay, mobile clients, or network control MUST NOT be admitted through the Spec 011 private-local-control surface.
- **FR-038**: Browser/WebView content MUST NOT receive generic access to the owner control plane merely because the desktop application uses a WebView.
- **FR-039**: Any desktop bridge to persistent-owner actions MUST preserve existing typed allowlisted authority and MUST NOT become a generic command dispatcher.
- **FR-040**: Persistent ownership MUST NOT grant Git merge/rebase/cherry-pick/push/PR/landing authority.
- **FR-041**: Persistent ownership MUST NOT grant verification or human-acceptance authority.
- **FR-042**: Process exit/completion MUST NOT be represented as candidate verification or acceptance.
- **FR-043**: Owner/runtime status MUST retain `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED` distinctions wherever agent/provider material is shown.
- **FR-044**: Bounded replay MUST preserve ordering/source identity and expose truncation/gap semantics explicitly.
- **FR-045**: Replayed output/history MUST NOT become canonical evidence merely because the persistent owner retained it.
- **FR-046**: History retention MUST have an explicit default, maximum bound, and deletion/eviction behavior selected later by Plan/Tasks.
- **FR-047**: Full process environments and secret values MUST NOT be durably retained merely to enable reattach.
- **FR-048**: Generic history/control methods MUST NOT expose raw privileged credentials by default.
- **FR-049**: Owner crash/restart MUST preserve enough state to explain uncertainty without manufacturing live ownership.
- **FR-050**: Partial persistence/corruption MUST fail closed and MUST preserve recoverable evidence rather than silently reinitialize ambiguous live state.
- **FR-051**: Explicit runtime stop MUST produce deterministic cleanup/final-state semantics for resources Winds can prove it owns.
- **FR-052**: If cleanup/termination/reaping cannot be proven, Winds MUST retain an unresolved/ownership-lost truth rather than report clean success.
- **FR-053**: Repeated stop/close requests MUST be safe and deterministic for already-terminal runtimes.
- **FR-054**: An owner shutdown policy MUST define per-runtime preservation/termination behavior and expose the result for each runtime independently.
- **FR-055**: Spec 011 MUST preserve accepted POSIX PTY and native-Windows ConPTY lifecycle behavior rather than weakening it for persistence.
- **FR-056**: Native Windows interrupt MUST remain fail-closed unless a later accepted implementation proves an ownership-scoped interrupt primitive.
- **FR-057**: Native Windows and WSL security/runtime claims MUST remain separate directly exercised domains.
- **FR-058**: Platform support MUST be claimed only where exact candidate evidence directly exercises endpoint security, detach/reattach, controller behavior, and lifecycle recovery on that platform.
- **FR-059**: Idle persistent-owner CPU, memory, handle/file-descriptor, and storage use MUST have Plan-defined measurable ceilings.
- **FR-060**: High-output and multi-client stress MUST preserve exact target/controller ownership and bounded memory/queue behavior.
- **FR-061**: Reconnect churn MUST not leak runtime/controller/client records or create ambiguous ownership generations.
- **FR-062**: Owner/client lifecycle events MUST expose enough source/identity information for presentation surfaces to remain truthful without leaking secret material.
- **FR-063**: Aliases MAY be user-friendly but MUST never replace immutable runtime identity for consequential actions.
- **FR-064**: Runtime lookup/search MUST fail or require explicit disambiguation when user-facing names are ambiguous.
- **FR-065**: Spec 011 MUST NOT require a cloud account/control plane for ordinary local continuity.
- **FR-066**: Spec 011 MUST NOT introduce generic plugin/runtime extension authority as a side effect of defining the local-control protocol.
- **FR-067**: Spec 011 MUST NOT require complete Herdr API parity; only the smallest persistent-owner/local-control semantics are in scope.
- **FR-068**: Spec 011 MUST NOT authorize direct/adapted Herdr source reuse; every copied/adapted slice requires later exact Tasks admission and provenance/license/security evidence.
- **FR-069**: Spec 011 MUST preserve all inherited historical failed evidence and nonclaims rather than relabelling them as resolved by the presence of a persistent owner.
- **FR-070**: Presentation clients MUST surface ownership loss, protocol incompatibility, controller status, replay truncation, and stale/runtime-generation mismatch as explicit states rather than optimistic generic `connected` state.

---

# Success Criteria

- **SC-001**: A long-running owned terminal/process survives complete detachment of all presentation clients for a Plan-defined interval and is later reattached to the exact same proven owner/runtime namespace without PID-based reconstruction.
- **SC-002**: A persisted runtime record with no provable live owner never renders or reports as live-owned.
- **SC-003**: PID-reuse and stale-endpoint fixtures cannot redirect reattach or terminate/control an unrelated process.
- **SC-004**: Two or more observers can receive runtime state while every mutating observer request is rejected until explicit controller authority exists.
- **SC-005**: Simultaneous controller-takeover races produce exactly one deterministic controller outcome and no duplicated input/resize/terminate action.
- **SC-006**: Controller disconnect/expiry never silently promotes an observer to write authority.
- **SC-007**: Protocol-version mismatch fails explicitly with zero consequential operation and zero silent downgrade.
- **SC-008**: Duplicate/replayed mutating request fixtures never repeat a consequential action contrary to the operation's specified replay semantics.
- **SC-009**: Malformed/oversized-message campaigns complete within bounded memory/time and cannot crash the owner or expand authority.
- **SC-010**: A slow-client campaign proves bounded queue/memory behavior and explicit drop/truncation/disconnect semantics.
- **SC-011**: History beyond the replay ceiling reconnects with explicit truncation/gap markers and correct source ordering.
- **SC-012**: Replayed output containing forged `VERIFIED`, approval, protocol-like JSON, runtime labels, or host actions cannot mutate trusted Winds state.
- **SC-013**: Owner-crash fixtures converge to fail-closed ownership truth without blind PID signal/kill and preserve uncertainty/recovery evidence.
- **SC-014**: Native provider resume, retained live-process continuation, Winds reconstruction, and fresh-process creation are observably distinct continuity classes.
- **SC-015**: Explicit stop/cleanup fixtures prove clean success only when termination/reaping/finalization is actually known; uncertain cleanup remains unresolved/ownership-lost.
- **SC-016**: Wrong-principal and stale/replaced-endpoint fixtures fail under the directly exercised local endpoint security policy for every claimed platform.
- **SC-017**: Native-Windows endpoint/controller/detach-reconnect claims are proven on native Windows rather than inferred from Linux/macOS or WSL.
- **SC-018**: WSL claims, if any, remain distinct from native-Windows claims and are directly exercised.
- **SC-019**: Persistent-owner idle CPU/RSS/handle and retained-history storage campaigns remain within Plan-defined bounds.
- **SC-020**: High-output plus multiple-observer stress preserves exact controller/input targeting and bounded owner resource use.
- **SC-021**: Reconnect churn and repeated client crashes produce zero ambiguous owner generations and no unbounded leaked client/controller/runtime records.
- **SC-022**: Security fixtures prove WebView/terminal/agent content cannot invoke generic owner control, create controller authority, or manufacture trusted lifecycle/evidence state.
- **SC-023**: The exact implementation program introduces no public network control, remote execution, plugin runtime, marketplace, live binary handoff, automatic Git landing, or same-user sandbox claim unless separately amended and governed.
- **SC-024**: The final exact candidate passes repository quality, applicable platform/security/stress workflows, author correctness/safety review, Ponytail/YAGNI review, fresh independent substantive review, and has zero unresolved material findings/threads.
- **SC-025**: Final Spec 011 closeout reconciles every FR/SC to exact evidence and preserves inherited Specs 003–010 truth/nonclaims without relabelling failed or unavailable proof.

---

# Explicit Non-Goals

Spec 011 does NOT by itself authorize or require:

- SSH transport, remote machines, remote execution, mobile/thin clients, cloud relay, or team control plane;
- public TCP/HTTP/WebSocket/RPC server or a promised third-party SDK/API;
- complete Herdr 104-method API parity;
- full workspace/tab/recursive-pane multiplexer parity;
- all 24 agent detections/integrations or the broader Agent Plane assigned to Spec 012;
- integration installer expansion, generic plugin runtime, hooks/actions/managed panes, marketplace/catalog, or dynamic code loading;
- automatic provider/model routing, winner scoring, or learned routing;
- live owner binary replacement/handoff, self-update, or remote-host update;
- browser automation/runtime, Browser Twin, CDP, SQL Studio, MCP/A2A expansion, or generic service orchestration;
- automatic merge/rebase/cherry-pick/push/PR creation/landing or branch-protection changes;
- a claim that PTY/worktree/local-endpoint isolation is an OS sandbox;
- isolation from arbitrary malicious code already executing as the same effective OS user without a stronger separately proven boundary;
- bulk-copying Herdr or another donor repository;
- a transport, serialization, async runtime, service manager, authentication library, storage engine, or database migration merely because a research source uses one.

---

# Plan-Stage Decisions Required

The later Plan MUST decide, with exact evidence rather than preference:

1. the smallest persistent-owner process/domain shape and lifecycle model;
2. whether owner startup is on-demand, login/session scoped, application-managed, or another bounded model;
3. local transport primitive(s) per claimed platform;
4. endpoint namespace/path generation and stale-endpoint recovery;
5. peer authentication and whether an additional local secret/challenge is necessary;
6. protocol framing/serialization/versioning/request-id/replay strategy;
7. observer/controller lease model, takeover semantics, and race ordering;
8. how existing `TerminalSession`/PTY/ConPTY ownership moves into the persistent domain without weakening cleanup;
9. durable metadata schema and whether any migration is actually necessary;
10. bounded output/history/replay storage, truncation, retention, and deletion policy;
11. owner crash/restart reconciliation and owner-generation identity;
12. native provider session-resume integration boundaries;
13. client bridge/API shape for CLI/TUI/Desktop without creating a generic dispatcher;
14. process startup environment handling and secret-minimization policy;
15. exact platform qualification matrix for Linux/macOS/native Windows/WSL where claimed;
16. performance/resource/stress thresholds for idle owner, high output, multiple clients, and reconnect churn;
17. packaging/install/uninstall/removal behavior sufficient to avoid stranded live work;
18. whether any Herdr slice is genuinely smaller/safer to adapt than a Winds-native implementation; default is no source admission until Tasks prove it.

Every dependency or donor-code proposal MUST record exact version/revision, license/notices, vendor/third-party provenance, security/authority impact, supported platform domain, removal/update path, and why existing accepted code is insufficient.

---

# Specification Acceptance Gate

This Spec may land only if its exact final candidate proves:

- canonical base is the post-merge-qualified Spec 011 Entry Gate or a governance-only forward descendant;
- changed scope is exactly `specs/011-persistent-agent-runtime-private-local-control/spec.md` unless a tightly coupled governance-only correction is separately justified;
- requirements remain implementation-independent and do not select a transport/runtime/service framework through specification wording;
- persistent owner authority is limited to the smallest local runtime continuity problem;
- observer/controller, owner/runtime namespace, native-agent-resume, reconstruction, canonical Session/workstream, and Git/evidence identities remain explicitly distinct;
- the threat model covers wrong local principal, stale/replaced endpoint, protocol version/replay/malformed input, controller races, owner/client crashes, forged content, PID reuse, and same-user nonclaim;
- no production source, dependency, lockfile, migration, workflow, owner process, IPC endpoint, remote execution, plugin runtime, marketplace, updater/handoff, browser, credential broker, automatic routing, or automatic landing behavior changes;
- Herdr freshness is rechecked and any material movement is reconciled rather than silently inherited;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review challenges daemon-framework creep, public-protocol creep, unnecessary persistence, generic API expansion, and premature donor-code adoption;
- fresh independent substantive review challenges testability, ownership proof, threat model, local-principal assumptions, race semantics, failure truth, platform claims, and non-goals;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal landing;
- merge tree/ordered parents/signature are verified;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN_AUTHORIZED=YES
SPEC_011_TASKS_AUTHORIZED=NO
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_LOCAL_CONTROL_IMPLEMENTATION_AUTHORIZED=NO
PUBLIC_RUNTIME_PROTOCOL_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_OWNER_HANDOFF_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

No source implementation may begin until a separate Plan and Tasks sequence independently qualifies and lands.
