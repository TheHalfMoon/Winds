# Spec 011 Formal Entry Gate — Persistent Agent Runtime & Private Local Control

**Status:** Governance entry candidate. Specification-only authority if canonically accepted.

**Canonical Winds base at creation:** `c6965beb69f57ca7a7663dc4d2d4bf1af066d828`

**Canonical base tree:** `9fd1b0937bfda0bd854638288a752cf56585c3ec`

**Date:** 2026-09-18

## 1. Purpose

Spec 010's first desktop implementation program is canonically closed through T145. The exact Spec 010 closeout merge is `7ed50fb6b173c4853394a93b51691e54fa463349`, and its post-merge repository `quality` run `35330722928` succeeded on attempt 1.

The Founder-directed Herdr parity research refresh is also canonically landed at `c6965beb69f57ca7a7663dc4d2d4bf1af066d828`, tree `9fd1b0937bfda0bd854638288a752cf56585c3ec`. Its post-merge repository `quality` run `35334086855` succeeded on attempt 1.

The next formal product problem is the local ownership gap deliberately left open by the one-process architecture: a Winds-owned terminal or agent process currently cannot remain provably live merely because a desktop/TUI/CLI client exits or restarts. Persisted IDs, PIDs, presentation state, or historical records are intentionally insufficient to manufacture live ownership.

This entry gate may authorize only a separate **Spec 011 specification candidate** for a narrow persistent local runtime owner and private local control plane after this exact entry gate is independently qualified and canonically landed.

This entry gate does **not** authorize Plan, Tasks, implementation, dependencies, migrations, persistent process creation, daemon/service installation, IPC endpoints, public APIs, remote execution, plugins, marketplace behavior, live owner replacement, automatic Git actions, or donor-code copying.

## 2. Canonical inherited truth

The entry decision is bounded by:

- Winds Constitution 1.1.0;
- canonical Spec 003 Workspace Execution Spine;
- canonical Spec 006 Agentic Terminal & Local Delegation Control Plane;
- canonical Spec 007 Native Agentic Terminal UX Foundation;
- canonical Spec 008 Resumable Workflow & Decision Ledger;
- canonical Spec 009 Model Mesh & Explicit Multi-Provider Continuity;
- canonical Spec 010 Winds Desktop Agentic Workspace through T145;
- `docs/research/020-herdr-current-source-audit.md`;
- `docs/research/020-herdr-full-parity-program.md`;
- `docs/research/021-herdr-exhaustive-capability-ledger.md`;
- `docs/provenance/source-registry.md`;
- current accepted PTY/ConPTY ownership and `OWNERSHIP_LOST` semantics in the Rust core;
- current desktop terminal bridge and exact canonical Session/terminal bindings.

Inherited repository truth is:

```text
T127..T145=CLOSED_CANONICAL
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS=CLOSED_CANONICAL
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
HERDR_PARITY_RESEARCH_REFRESH=CLOSED_CANONICAL
CANONICAL_MAIN=c6965beb69f57ca7a7663dc4d2d4bf1af066d828

CURRENT_TERMINAL_LIVE_AUTHORITY=RETAINED_OWNED_CHILD_OR_PROCESS_SCOPE
PERSISTED_PID_IS_LIVE_AUTHORITY=NO
PERSISTED_TERMINAL_ROW_IS_LIVE_AUTHORITY=NO
DESKTOP_PRESENTATION_IS_LIVE_AUTHORITY=NO
CROSS_RESTART_LIVE_OWNERSHIP_PROVEN=NO
UNPROVEN_RESTART_OWNERSHIP=OWNERSHIP_LOST

SPEC_011_ENTRY_AUTHORIZED=NO
SPEC_011_PLAN_AUTHORIZED=NO
SPEC_011_TASKS_AUTHORIZED=NO
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_LOCAL_CONTROL_IMPLEMENTATION_AUTHORIZED=NO
PUBLIC_RUNTIME_PROTOCOL_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_OWNER_HANDOFF_AUTHORIZED=NO
```

## 3. Entry-time Herdr source reconciliation

The canonical research refresh pinned:

```text
RESEARCH_HERDR_PIN=da6bcd5969779bfe0396bcf89a8025d4375d611e
RESEARCH_HERDR_TREE=aece03633c003ba0fce01bc2564ad14e0fe9cac9
```

A fresh entry-time check on 2026-09-18 observed Herdr `master` at:

```text
ENTRY_HERDR_PIN=7df919d00e5bf8f6ed43a1781e81340cc0bbce8d
ENTRY_HERDR_TREE=647a010ad83d983943ecfe00f82b6a376f1bfbf1
COMMITS_SINCE_RESEARCH_PIN=1
REVERSE_DIVERGENCE=0
```

The exact one-commit delta is:

```text
7df919d00e5bf8f6ed43a1781e81340cc0bbce8d
fix: correct grok activity detection with custom or disabled osc signals (#4337)
```

The changed upstream paths are limited to:

- `distribution/agent-detection/grok.toml`;
- `scripts/agent_detection_manifest_check.py`;
- `src/detect/manifest/tests.rs`;
- `src/detect/manifests/grok.toml`.

Fresh entry-time source inspection confirms that the parity inventory counts remain:

```text
AGENT_FAMILIES=24
PUBLIC_SERIALIZED_API_METHODS=104
FROZEN_SERIALIZED_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INTEGRATIONS=1 (letta)
BASELINE_PARITY_LEDGER_ROWS=218
```

This upstream delta is material to later broad agent-detection parity, especially the proposed Spec 012 agent plane. It does not alter the persistent-owner/private-local-control problem admitted by this gate.

The formal Spec 011 candidate MUST bind its research references to `7df919d...` or a newer exact pin after another freshness check. If Herdr moves before this entry gate lands, the candidate MUST reconcile that movement rather than silently calling `7df919d...` current.

## 4. Founder-directed program position

The canonical Herdr parity program proposes this sequence:

```text
SPEC_011=PERSISTENT_AGENT_RUNTIME_AND_PRIVATE_LOCAL_CONTROL
SPEC_012=WORKSPACE_MULTIPLEXER_AND_AGENT_PLANE
SPEC_013=REMOTE_MACHINES_AND_THIN_CLIENTS
SPEC_014=INTEGRATION_PLUGIN_AND_MARKETPLACE_PLATFORM
SPEC_015=LIVE_HANDOFF_UPDATE_AND_FULL_PARITY_CLOSEOUT
```

This gate admits only the first specification stage.

If this gate closes canonically:

```text
SPEC_011_NAME=PERSISTENT_AGENT_RUNTIME_AND_PRIVATE_LOCAL_CONTROL
SPEC_011_FORMAL_SPEC_AUTHORIZED=YES
SPEC_011_PLAN_AUTHORIZED=NO
SPEC_011_TASKS_AUTHORIZED=NO
SPEC_011_IMPLEMENTATION_AUTHORIZED=NO
```

The parity roadmap remains a product/research target, not a blanket implementation grant.

## 5. Existing Winds seams the specification must preserve

Spec 011 is not permission to replace working Winds subsystems with a copied multiplexer.

The specification must begin from current Winds truth:

1. **PTY/ConPTY ownership already exists.** The Rust core owns live terminal lifecycle through retained child/process-scope objects and accepted `portable-pty` behavior. A presentation pane, PID, persisted row, or native agent session ID is not process authority.
2. **Restart reconciliation is already fail-closed.** When continuing ownership cannot be proven, persisted terminal/execution state becomes `OWNERSHIP_LOST` / process-state-unknown. No stale PID is blindly signaled or killed.
3. **Canonical identity already exists.** Workspace, Project presentation, Workstream/task, Winds Session, runtime identity, terminal identity, native provider session identity, Git candidate, and evidence identity are distinct.
4. **Desktop rendering is non-authoritative.** Tauri/WebView/React state cannot grant process, Git, verification, acceptance, credential, or landing authority.
5. **Verification authority already exists.** A long-lived owner may preserve process continuity but may not convert agent output, terminal output, process exit, or runtime status into verified/accepted candidate evidence.
6. **Local-first remains mandatory.** Persistent local work must not require a cloud control plane merely to survive a client exit.
7. **Git safety remains independent.** Persistent execution authority does not authorize merge, rebase, cherry-pick, push, PR creation, winner selection, primary-checkout mutation, or any broader Git policy.

The smallest viable Spec 011 architecture should move only ownership that genuinely must outlive a client into the persistent domain. Canonical data and privileged operations that do not require persistent process ownership should remain in existing Winds seams unless the specification proves otherwise.

## 6. Product thesis for the formal specification

The formal Spec 011 may define an implementation-independent local runtime model in which:

- one narrow Winds-owned local authority domain may outlive individual UI/CLI/TUI clients;
- Winds-created PTY/ConPTY/process ownership can survive client detach when the owner itself remains live;
- a client can detach and later reattach without pretending that persistence was reconstructed from a PID;
- multiple local clients may observe the same runtime truth;
- writable terminal/input/resize authority is explicit and singular or otherwise unambiguously leased;
- controller takeover is explicit, attributable, bounded, and never implied by focus alone;
- runtime namespaces are stable enough for reconnection but remain distinct from canonical Project/Session/workstream identity;
- durable metadata can describe topology and historical state without itself proving live ownership;
- bounded replay/history can reconstruct presentation without converting imported history into canonical evidence;
- native agent resume, live retained-process continuation, Winds reconstruction, and new process creation remain separately labelled;
- process exit, owner crash, endpoint loss, protocol mismatch, lease loss, and uncertain cleanup fail truthfully rather than becoming optimistic "connected" state.

The purpose is continuity with exact authority, not a general-purpose daemon framework.

## 7. Required independently testable user scenarios

The separate Spec 011 specification should define acceptance scenarios for at least:

### A. Detach without process loss

A user starts a Winds-owned terminal/agent process, detaches the presentation client, and the accepted persistent owner continues to hold the exact live ownership primitive. The child remains live because the owner remains live, not because a database row says so.

### B. Reattach to exact runtime truth

A new local client identifies the intended runtime namespace, authenticates to the private local owner, receives exact topology/lifecycle state, and reattaches. Similar display names, stale marker files, or reused OS PIDs cannot redirect it to another runtime silently.

### C. Multiple observers, explicit controller

More than one local client can observe one runtime without every observer acquiring input/resize/process-control authority. The active controller/lease is explicit. Takeover requires an explicit accepted action and produces deterministic prior-controller behavior.

### D. Owner crash / uncertain continuity

If the persistent owner crashes or disappears, clients and persisted state do not claim live ownership merely from old runtime metadata. Unprovable continuity becomes `OWNERSHIP_LOST`, `UNAVAILABLE`, or another specification-defined fail-closed state with process liveness separately unknown where appropriate.

### E. Protocol/version mismatch

A client and owner with incompatible local-control versions fail closed with an explicit compatibility error. Version mismatch does not trigger unsafe fallback, unversioned command execution, automatic downgrade, or endpoint replacement.

### F. Bounded replay

A reconnecting client can receive enough bounded history/state to render useful continuity. Replay has explicit limits, ordering, truncation semantics, and source labels. It cannot become candidate-verification evidence merely because the owner retained it.

### G. Native agent resume distinction

Where a runtime such as Codex/Claude/other supported agent exposes a native session identifier, Winds distinguishes native conversation resume from retained live process ownership and from Winds reconstruction.

### H. Clean stop and recovery

An explicit owner/runtime stop has bounded cleanup and deterministic final state. Failed or unproven cleanup does not report success, erase recovery material, or silently reuse an ambiguous endpoint.

## 8. Private local-control security contract the specification must define

The Constitution already requires an explicit threat model before any persistent owner or local control surface is implemented.

The formal specification MUST define measurable requirements for:

### Endpoint locality and ownership

- local-only transport by default;
- no TCP listener, HTTP server, WebSocket server, LAN listener, remote-origin command path, or cloud relay in Spec 011;
- Unix endpoint parent-directory/file ownership and permissions that prevent other local accounts from connecting or replacing the endpoint under the claimed threat model;
- Windows named-pipe or equivalent local transport with an explicit DACL/current-user/System policy rather than permissive defaults;
- safe stale-endpoint detection that distinguishes a live owner from stale path/marker state;
- removal/replacement only when endpoint ownership/identity is proven;
- collision-resistant runtime namespace/endpoint identity;
- no symlink/path replacement or marker-file ambiguity silently granting authority.

### Authentication and peer truth

"Local" is not sufficient authentication by itself.

The specification must define:

- what OS/process/user identity is trusted;
- how a client proves it belongs to the allowed local principal/domain;
- whether an additional per-install/per-runtime secret or challenge is required and how it is stored/rotated;
- how peer identity is bound to the exact runtime namespace;
- how stale credentials/session tokens fail;
- how logout/revocation/owner restart affect existing clients.

Spec 011 MUST NOT claim isolation from arbitrary malicious code already executing with the same effective OS user unless a stronger enforceable boundary is separately proven.

### Protocol lifecycle

The specification must define:

- explicit protocol/schema version;
- incompatible-version and downgrade behavior;
- request IDs / response correlation;
- duplicate/replay semantics for mutating requests;
- bounded frame/request/replay sizes;
- bounded queues and slow-client behavior;
- malformed-frame behavior;
- connection and request timeout semantics;
- deterministic owner shutdown/restart behavior;
- no raw terminal text or agent prose interpreted as control protocol.

### Observer/controller authority

The specification must define:

- read-only observation separately from write/control authority;
- one explicit controller lease or another equally unambiguous model;
- controller identity and lease lifecycle;
- explicit takeover/release behavior;
- disconnect/timeout behavior;
- race behavior for simultaneous takeover/input/resize/terminate;
- no implicit broadcast input across sessions or clients;
- no authority escalation merely because a client can observe a runtime.

### Secrets and history

The specification must define:

- no full process-environment persistence merely to resume;
- bounded transcript/history behavior;
- transcript/history default and retention policy;
- secret-sensitive data handling consistent with existing Winds limits;
- no claim of perfect secret detection;
- no privileged credential export through generic local-control methods.

## 9. Identity and authority invariants

The formal specification must preserve:

```text
PERSISTENT_RUNTIME_NAMESPACE != CANONICAL_WINDS_SESSION
PERSISTENT_RUNTIME_NAMESPACE != WORKSTREAM_OR_TASK
PERSISTENT_RUNTIME_NAMESPACE != GIT_CANDIDATE
PERSISTENT_RUNTIME_NAMESPACE != PROVIDER_NATIVE_SESSION

LIVE_PROCESS_OWNERSHIP != NATIVE_AGENT_RESUME
LIVE_PROCESS_OWNERSHIP != WINDS_RECONSTRUCTION
NATIVE_AGENT_RESUME != CANONICAL_TASK_CONTINUITY
IMPORTED_OR_REPLAYED_HISTORY != CANONICAL_EVIDENCE

OBSERVER_AUTHORITY != CONTROLLER_AUTHORITY
CLIENT_FOCUS != CONTROLLER_LEASE
CONTROLLER_AUTHORITY <= EXPLICIT_LOCAL_HUMAN_AUTHORITY
PERSISTENT_OWNER_AUTHORITY != GIT_LANDING_AUTHORITY

AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
PROCESS_EXIT != VERIFIED
PROCESS_EXIT != ACCEPTED
OWNER_LIVE != CANDIDATE_ELIGIBLE
```

No persistent owner may self-expand delegation ceilings, verification authority, acceptance authority, Git authority, filesystem/network sandbox claims, or provider credentials.

## 10. Herdr evidence admitted as research input, not implementation authority

The entry-time Herdr source demonstrates useful design evidence including:

- headless server ownership of PTYs/process state;
- named sessions;
- detach/reattach;
- multiple observing clients;
- explicit writable attach/takeover semantics;
- local socket/named-pipe paths;
- Unix permission hardening;
- Windows named-pipe security descriptors;
- stale endpoint identity/removal behavior;
- protocol-version mismatch handling;
- session restore/native agent resume;
- bounded client/server state transfer.

These observations justify specifying the product problem. They do not authorize copying Herdr's daemon, API enum, endpoint implementation, plugin infrastructure, remote stack, updater, or complete 104-method API.

Any later direct/adapted source reuse must be admitted by exact Tasks with source path, exact upstream revision, notice/license/vendor provenance, destination path, modification record, threat-model delta, tests, and removal/update path.

## 11. Explicit non-goals for Spec 011

Unless a later accepted amendment changes scope, the formal Spec 011 must not silently include:

- SSH transport, remote machines, remote execution, remote thin clients, or cloud relay;
- public RPC/HTTP/WebSocket/network protocol;
- generic external SDK/API promised for third-party clients;
- complete Herdr 104-method API parity;
- full workspace/tab/recursive-pane multiplexer parity;
- broad agent-detection parity or all 24 agent integrations;
- integration installer catalog expansion;
- plugin runtime, plugin hooks/actions/panes, or marketplace/catalog implementation;
- automatic model/provider routing;
- live owner binary replacement/handoff or self-update;
- generic service orchestration;
- browser automation/runtime;
- SQL/database studio;
- MCP/A2A or new ACP authority merely because a local control plane exists;
- OS sandbox claims from PTY/worktree/local endpoint isolation;
- automatic merge/rebase/cherry-pick/push/PR creation/landing;
- changing branch protection or repository governance;
- silently changing current platform support claims.

Those capabilities remain assigned to later separately governed work.

## 12. Specification quality bar

The formal Spec 011 must define measurable outcomes for at least:

- detach/reattach correctness and identity;
- exact owner/client/runtime namespace lifecycle;
- multiple observer behavior;
- controller lease/takeover races;
- owner crash and client crash;
- stale endpoint and replacement attacks;
- protocol version mismatch;
- malformed/oversized/duplicated/replayed messages;
- bounded history/replay and slow clients;
- terminal input/output/resize/exit continuity under detach/reattach;
- persistence without PID-as-authority regression;
- native agent resume distinction;
- resource/handle cleanup;
- idle owner resource bounds;
- high-output and multi-client stress;
- Unix/macOS and native-Windows local endpoint security only where directly exercised;
- zero verification/Git/acceptance authority escalation;
- upgrade/removal/recovery semantics sufficient to avoid stranded live work.

A durable owner is not accepted merely because a background process stays alive.

## 13. Entry acceptance gate

This entry candidate may land only if the exact final candidate proves:

- changed scope is exactly `docs/research/022-spec-011-entry-gate.md`;
- canonical base is the post-merge-qualified Herdr research refresh `c6965beb69f57ca7a7663dc4d2d4bf1af066d828`;
- the Herdr entry-time pin is rechecked immediately before final qualification;
- every Herdr movement since the canonical research pin is explicitly classified;
- no production source, dependency, lockfile, migration, workflow, IPC endpoint, process owner, service/daemon, runtime/provider, credential, remote, plugin, updater, or Git behavior changes;
- repository `quality` succeeds on the exact final head;
- correctness/safety/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review challenges whether the admitted product problem can be specified with a narrower persistent surface;
- fresh independent substantive review challenges threat boundaries, identity separation, non-goals, and specification-only authority;
- zero unresolved material findings/threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded expected-head normal merge;
- canonical merge tree, ordered parents, and GitHub signature are verified;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
HERDR_PARITY_RESEARCH_REFRESH=CLOSED_CANONICAL
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_FORMAL_SPEC_AUTHORIZED=YES
SPEC_011_PLAN_AUTHORIZED=NO
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

## 14. Entry effect

If this gate closes canonically, the next authorized repository unit is a separate:

`specs/011-persistent-agent-runtime-private-local-control/spec.md`

candidate only.

That specification must start from then-current canonical `main`, remain implementation-independent, define independently testable user scenarios and measurable outcomes, include an explicit threat model and authority vocabulary, reconcile the fresh Herdr pin against the relevant parity ledger rows, preserve all existing Winds nonclaims, and pass exact-candidate acceptance gates before any Plan is authored.

No Plan, Tasks, implementation, dependency, migration, service process, private endpoint, or donor-code copy is authorized merely because this entry gate lands.
