# Implementation Plan: Workspace Multiplexer & Agent Plane

**Branch**: plan/012-workspace-multiplexer-agent-plane  
**Created**: 2026-09-20  
**Status**: Plan candidate only. Tasks, implementation, dependency adoption, lockfile mutation, migrations, source reuse, production protocol changes, agent execution, worktree mutation, remote execution, plugins, marketplace, updater/distribution changes, and automatic Git landing are NOT authorized by this file alone.

## Canonical Baseline

This Plan begins from the canonically accepted Spec 012 specification:

~~~text
SPEC_012_ENTRY_MERGE=ec214544d94d90b23fe4a0156d0040a2faf0622c
SPEC_012_ENTRY_POST_MERGE_QUALITY=35513773204 SUCCESS

SPEC_012_SPEC_MERGE=27dd3e88255abd1184c8ca1e54cd671f20e058fd
SPEC_012_SPEC_TREE=ec0d9a525502bbb0609f3fe38a3f86871f3b92c9
SPEC_012_SPEC_POST_MERGE_QUALITY=35517252780 SUCCESS

SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN_AUTHORIZED=YES
SPEC_012_TASKS_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO
~~~

The accepted specification contains 92 functional requirements and 30 success criteria. This Plan must make each requirement implementable without weakening existing Winds identity, runtime, evidence, Git, process, platform, privacy, accessibility, or human-decision truth.

Inherited boundaries remain binding:

~~~text
PUBLIC_RUNTIME_PROTOCOL_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_BINARY_HANDOFF_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
HERDR_DIRECT_COPY_AUTHORIZED=NO
HERDR_ADAPTED_COPY_AUTHORIZED=NO
HERDR_TEST_PORT_AUTHORIZED=NO
~~~

## Plan-Time Herdr Research Recheck

The current Plan-stage upstream observation chain is:

~~~text
HERDR_PLAN_WORKTREE_HEAD=6c62707a3a43427a34fb418ce20c834a88089ba1
HERDR_PLAN_WORKTREE_TREE=0940b3d41853316fcd861b5902ed7a4ebd155066
HERDR_PLAN_WORKTREE_PARENT=74505861e40c48e070e711bd4de662a17b0939b3
HERDR_PLAN_WORKTREE_GITHUB_VERIFICATION=VERIFIED_VALID
HERDR_PLAN_WORKTREE_MESSAGE=fix: preserve explicit worktree workspace membership (#4301)

HERDR_PLAN_EVENT_HEAD=65927cef1c735169ba07677311ec9d6d7435c9f3
HERDR_PLAN_EVENT_TREE=516897019bb382bbdc4c0a8dba83758a9d8d1fef
HERDR_PLAN_EVENT_PARENT=6c62707a3a43427a34fb418ce20c834a88089ba1
HERDR_PLAN_EVENT_GITHUB_VERIFICATION=VERIFIED_VALID
HERDR_PLAN_EVENT_MESSAGE=fix: drain event subscriptions and report history loss (#4225)

HERDR_PLAN_RESEARCH_HEAD=f10df75c8a5f1b5e5f90689c3a8ceb6758366e05
HERDR_PLAN_RESEARCH_TREE=3fa0fa2a9f48cf1cf1f0820cfab94abda63b8549
HERDR_PLAN_RESEARCH_PARENT=65927cef1c735169ba07677311ec9d6d7435c9f3
HERDR_PLAN_RESEARCH_GITHUB_VERIFICATION=VERIFIED_VALID
HERDR_PLAN_RESEARCH_GITHUB_VERIFICATION_REASON=valid
HERDR_PLAN_RESEARCH_GITHUB_VERIFIED_AT=2026-09-20T15:12:12Z
HERDR_PLAN_RESEARCH_MESSAGE=fix: reject terminal-less attach before starting a session (#4395)
~~~

The first post-Spec delta changes only src/app/api/worktrees.rs and preserves explicit worktree-to-workspace membership rather than allowing inferred discovery refresh to silently rewrite explicit membership.

The next upstream delta changes event subscription/history behavior and documentation. It drains retained lifecycle/agent-status batches, detects when a subscriber has fallen behind bounded retained history, reports explicit events_lost state, closes only the affected subscription, and requires resubscription plus authoritative snapshot recovery rather than silently continuing with missing events. It also documents that snapshot/event ordering must not be guessed across an unproven boundary.

The latest upstream delta changes autodetect attach preflight. It rejects a terminal-less interactive attach before socket lookup, session-directory creation, or daemon startup. The useful design lesson is fail-before-side-effects when the selected client surface cannot satisfy the capability required by the requested operation. This is not authority to copy Herdr startup behavior or to require a TTY for Desktop.

Plan consequences:

- Winds must model explicit repository/worktree membership as durable user-owned metadata.
- Discovery may update observations but MUST NOT silently remove an explicit membership relation.
- A worktree that becomes temporarily unavailable is stale/unavailable, not automatically forgotten.
- Explicit membership remains separate from repository trust and from current filesystem existence.
- A lost/gapped multiplexer, agent-observation, or attention event stream marks the client projection stale.
- Recovery requires a fresh authoritative snapshot plus a fresh subscription boundary.
- Buffered events MUST NOT be blindly replayed over a newer snapshot unless an exact Winds-owned generation/sequence relation proves they are still applicable.
- TopologyGeneration remains the strongest ordering primitive for topology mutations; discontinuity requires resnapshot.
- Agent/attention streams must expose explicit gap/loss state rather than silently skipping retained history.
- Any operation that requires an interactive terminal surface or geometry MUST preflight that exact client capability before creating multiplexer state, starting/binding a runtime, creating persistence records, or starting an owner solely for that request.
- TUI/CLI terminal capability and Desktop terminal-surface capability are distinct; absence of a controlling TTY MUST NOT be generalized into a Desktop failure when the trusted Desktop surface provides the required geometry/input capability.
- All upstream changes are design evidence only. No source reuse is admitted.

---

## Planning Principle

The smallest architecture that satisfies Spec 012 is:

> add one Winds-owned multiplexer domain inside the already accepted persistent owner, reuse existing terminal/runtime/Git/store seams, expose only closed typed local operations, and keep presentation state separate from runtime authority and canonical evidence.

The Plan deliberately avoids a second daemon, second terminal backend, public API, generic command bus, plugin host, remote plane, terminal-text inference engine, or automatic Git workflow.

~~~text
MULTIPLEXER_AUTHORITY
  = LIVE_TOPOLOGY_SERIALIZATION
  + EXACT_TARGET_NAVIGATION
  + PRESENTATION_STATE
  + STRUCTURED_AGENT_OBSERVATION
  + EXPLICIT_WORKTREE_MEMBERSHIP
  + TYPED_LOCAL_ACTIONS

MULTIPLEXER_AUTHORITY
  != PROCESS_OWNERSHIP_AUTHORITY
  != PROVIDER_EXECUTION_AUTHORITY
  != VERIFICATION_AUTHORITY
  != HUMAN_ACCEPTANCE
  != GIT_LANDING_AUTHORITY
  != REMOTE_AUTHORITY
  != PLUGIN_AUTHORITY
~~~

---

# Architecture Decisions

## AD-012-01 — Extend the existing persistent owner; do not create a second owner

The Spec 011 persistent owner remains the sole live local authority process.

Spec 012 topology, agent-observation, attention, and local workspace state are hosted as a bounded domain inside that owner. The owner serializes accepted topology mutations and publishes bounded projections/events to clients.

Do not create:

- a second daemon;
- a second socket/named-pipe server;
- a browser/WebView listener;
- a second terminal-process registry;
- a second SQLite authority;
- a public workspace API server.

Desktop, TUI, and CLI must continue through the shared trusted Rust local-control client seam.

Why:

- one owner generation avoids split-brain topology authority;
- existing observer/controller semantics remain reusable for runtime-affecting actions;
- owner crash truth already fails closed;
- local principal authentication and transport security are already qualified.

## AD-012-02 — Multiplexer identities are new opaque domains

Introduce explicit Winds-owned identity types:

~~~text
MultiplexerWorkspaceId
TabId
PaneId
TopologyGeneration
LayoutTemplateId
AgentObservationId
~~~

These identities MUST NOT reuse:

- canonical Git Workspace ID;
- Project ID;
- Session/Workstream ID;
- RuntimeNamespaceId;
- terminal ID;
- provider-native session ID;
- OS PID;
- path;
- label;
- visual ordinal;
- focus state.

IDs are generated through the already accepted Spec 011 OS-entropy seam. Tasks MUST reuse that seam rather than adding UUID/random dependencies.

Pane ID lifetime is independent of topology generation. A surviving pane retains its PaneId across split ratios, moves, swaps, zoom, focus changes, tab reorder, workspace rename, and other topology mutations.

A replacement pane MUST receive a new PaneId.

## AD-012-03 — Canonical Git workspace identity remains separate

Current src/workspace.rs defines a stable canonical Git workspace identity from canonical worktree root plus Git common directory.

Spec 012 does not replace it.

Use this terminology in implementation:

~~~text
MultiplexerWorkspaceId = UI/runtime topology grouping identity
GitWorkspaceId         = existing store Workspace ID bound to a concrete Git worktree
RepositoryIdentity     = canonical Git common directory identity
~~~

A multiplexer workspace may explicitly contain zero or more GitWorkspaceId memberships.

A pane may bind one GitWorkspaceId for cwd/worktree context, but that does not change the parent MultiplexerWorkspaceId.

This prevents current desktop use of workspaceId as Project/Git identity from silently becoming multiplexer topology authority.

## AD-012-04 — One immutable-ID topology tree with generation-based optimistic validation

Each MultiplexerWorkspace owns ordered tabs.

Each tab owns one recursive topology tree:

~~~text
LayoutNodeV1
  = Pane(PaneId)
  | Split {
      axis: Horizontal | Vertical,
      ratio_basis_points: u16,
      first: Box<LayoutNodeV1>,
      second: Box<LayoutNodeV1>
    }
~~~

Plan constraints:

- split ratio uses integer basis points, not floating point;
- accepted range is 1000..9000 basis points unless a later exact Task tightens it;
- every mutation starts from an exact TopologyGeneration;
- every successful topology mutation increments the generation exactly once;
- stale generation returns an explicit stale-topology error;
- mutation is never retargeted by current focus, ordinal, label, or geometry;
- focus and zoom are explicit state, not identity;
- directional navigation is derived deterministically from current geometry with stable tie-breaking.

Topology mutation uses compare-and-apply semantics inside the owner authority loop. No distributed lock or generic transactional framework is needed.

## AD-012-05 — Separate live topology snapshots from reusable layout templates

Two serialization shapes are required.

### TopologySnapshotV1

Used for owner restart/presentation restoration.

It may contain:

- MultiplexerWorkspaceId;
- TabId;
- PaneId;
- topology generation;
- alias/title/presentation metadata;
- active tab/focused pane;
- optional GitWorkspaceId references;
- optional RuntimeNamespaceId references as references only;
- optional AgentObservationId references as presentation links.

It MUST NOT preserve:

- controller lease authority;
- owner generation as reusable authority;
- OS PID;
- raw process handle;
- bearer credentials;
- verification/acceptance status as mutable authority;
- Git landing authority.

### LayoutTemplateV1

Used for save/export/apply.

It contains structure and presentation defaults only.

It MUST omit:

- MultiplexerWorkspaceId;
- TabId;
- PaneId;
- RuntimeNamespaceId;
- terminal ID;
- provider-native session ID;
- controller authority;
- agent observation identity;
- Git candidate/evidence IDs;
- secrets.

Applying a template generates fresh workspace/tab/pane IDs and creates no process.

This makes saved layouts genuine templates rather than disguised live-state resurrection.

## AD-012-06 — Migration 0014 is the planned persistence boundary

If Tasks confirm this Plan, the next schema migration is planned as:

migrations/0014_workspace_multiplexer.sql

No schema mutation is authorized by this Plan alone.

The smallest planned persistence model is:

### multiplexer_workspaces

One row per MultiplexerWorkspaceId containing:

- immutable workspace ID;
- mutable alias;
- topology format version;
- topology generation;
- bounded TopologySnapshotV1 JSON;
- created/updated timestamps.

The JSON snapshot avoids premature relational decomposition of recursive pane trees.

### multiplexer_layout_templates

One row per LayoutTemplateId containing:

- immutable template ID;
- mutable name;
- format version;
- bounded LayoutTemplateV1 JSON;
- created/updated timestamps.

### multiplexer_worktree_memberships

Explicit relation between:

- MultiplexerWorkspaceId;
- existing GitWorkspaceId;
- membership source = EXPLICIT_USER | CREATED_BY_WINDS | IMPORTED_EXPLICIT;
- created/last-confirmed timestamps;
- stale/unavailable observation state.

Discovery is not membership authority.

### repository_trust

Explicit accepted repository trust keyed by canonical RepositoryIdentity.

Trust MUST NOT be inferred from:

- opening a terminal;
- focusing a pane;
- detecting an agent;
- displaying a repository;
- imported layout content;
- terminal/agent text.

Tasks may reduce table count if they can preserve all constraints with a smaller schema. They MUST NOT collapse GitWorkspaceId into MultiplexerWorkspaceId.

## AD-012-07 — Current in-memory owner state is authoritative while the owner generation is live

While the owner generation is live:

- the owner topology domain is the live authority;
- SQLite is durable recovery/presentation metadata;
- clients never directly mutate multiplexer tables;
- multiple clients observe the same topology generation;
- a client disconnect does not transfer or mutate topology.

On owner restart:

1. load persisted TopologySnapshotV1;
2. create a new owner generation;
3. reconstruct presentation topology;
4. reconcile each RuntimeNamespaceId against canonical Spec 011 truth;
5. mark unprovable runtime bindings OWNERSHIP_LOST/unavailable rather than pretending reattachment;
6. retain pane identity only where the persisted pane itself is restored as presentation metadata;
7. never restore controller leases.

## AD-012-08 — Protocol v2 extends the same private local-control protocol

Spec 012 requires additional local-control message families. Do not create a second protocol.

The first Spec 012 implementation program plans to bump the private protocol from version 1 to version 2.

Reasons:

- new message kinds materially extend the protocol contract;
- an old v1 owner and new v2 client must not silently interpret partial capability;
- exact version failure is safer than compatibility guessing.

No automatic downgrade is allowed.

A v2 client encountering a live v1 owner reports explicit protocol mismatch/upgrade-required state. It MUST NOT kill, replace, or hand off the old owner automatically.

If a live v1 owner still owns runtimes, the v2 client enters BLOCKED_LEGACY_OWNER state for owner-backed features. It MUST NOT start a second owner, reclaim PIDs, migrate controller leases, or pretend the runtimes were upgraded. Recovery is manual/explicit: continue with a compatible v1 client, or explicitly quiesce/stop the old owner and runtimes through a compatible path before a fresh v2 owner starts. Seamless live owner upgrade is a non-goal.

Planned v2 message families are closed enums, not arbitrary method names:

~~~text
LIST_MULTIPLEXER_WORKSPACES
MULTIPLEXER_SNAPSHOT
REQUEST_MULTIPLEXER_WRITE
RELEASE_MULTIPLEXER_WRITE
MULTIPLEXER_WRITE_STATE
APPLY_TOPOLOGY_OPERATION
MULTIPLEXER_EVENT

LIST_AGENT_OBSERVATIONS
AGENT_OBSERVATION_SNAPSHOT
AGENT_OBSERVATION_EVENT

LIST_WORKTREES
APPLY_WORKTREE_OPERATION
WORKTREE_OPERATION_RESULT

ATTENTION_SNAPSHOT
ATTENTION_EVENT
~~~

APPLY_TOPOLOGY_OPERATION and APPLY_WORKTREE_OPERATION use closed Rust enums with variant-specific validated payloads.

No string-dispatched generic RPC method registry is allowed.

## AD-012-09 — Topology write authority and runtime controller authority are separate

Every multiplexer connection starts read-only for topology.

Observer-safe operations include:

- list/snapshot/read;
- search/filter;
- explain;
- subscribe;
- presentation-only inspection.

A client that needs consequential topology mutation MUST explicitly request MultiplexerWrite authority after local-principal authentication. The owner records the granted connection mode. There is no automatic observer-to-writer promotion based on focus, UI visibility, or first mutation attempt.

MultiplexerWrite is not exclusive: multiple authenticated interactive clients may hold it, while TopologyGeneration compare-and-apply semantics serialize conflicting mutations. It is also not durable across reconnect or owner generation.

Topology-only mutations require:

- authenticated local client;
- explicit MultiplexerWrite authority;
- exact immutable target;
- current TopologyGeneration;
- valid closed operation schema;
- accepted user/client intent.

Examples:

- rename workspace/tab;
- reorder tab;
- split an unbound pane;
- move/swap unbound presentation topology;
- focus pane;
- resize split geometry;
- zoom pane;
- apply layout template;
- pane.clear presentation epoch.

These operations do not grant runtime controller authority.

Runtime-affecting operations still require the existing Spec 011 RuntimeNamespaceId controller lease in addition to any required MultiplexerWrite authority.

Examples:

- terminal input;
- send keys/text;
- resize actual PTY/ConPTY;
- interrupt;
- stop/terminate;
- any prompt transport to a live agent pane;
- closing a pane when the selected close policy also stops its bound runtime.

A topology mutation cannot smuggle a runtime mutation. MultiplexerWrite never implies Runtime Controller, and Runtime Controller never implies MultiplexerWrite.

## AD-012-10 — Pane split/close are explicit about process authority

Splitting a pane creates a new unbound PaneId by default.

It does not:

- start a shell;
- launch an agent;
- clone a runtime;
- inherit controller authority;
- inherit provider-native identity.

A later explicit runtime-start action may bind a runtime namespace to that pane under separately authorized Tasks.

Pane close has two distinct policies:

~~~text
DETACH_VIEW
STOP_RUNTIME_THEN_CLOSE
~~~

DETACH_VIEW removes the PaneId from topology after exact binding validation and leaves any bound RuntimeNamespaceId alive and discoverable in runtime inventory. It requires MultiplexerWrite but not Runtime Controller because it does not mutate the process.

STOP_RUNTIME_THEN_CLOSE requires both MultiplexerWrite and the exact Runtime Controller lease. The pane is removed only after the stop operation reaches a truthful terminal disposition. If stop is failed, outcome-unknown, or ownership-lost, the close must not be reported as a clean process stop.

There is no implicit terminate-on-close policy and no automatic orphan cleanup.

This keeps layout editing reversible and prevents split/close/navigation actions from gaining hidden execution authority.

## AD-012-11 — Reuse existing PTY/ConPTY runtime ownership and replay

A bound pane points to an existing RuntimeNamespaceId managed by the Spec 011 owner.

Do not add:

- a second PTY implementation;
- pane-owned child handles;
- pane-owned PID authority;
- duplicate transcript persistence.

Input/resize/interrupt/stop flow through the existing runtime controller path.

The multiplexer stores only presentation/runtime references and derives runtime truth from the owner.

## AD-012-12 — Pane clear is a presentation event, not child input

pane.clear means:

- clear rendered screen and pane scrollback presentation for that PaneId;
- advance a pane-local presentation epoch;
- publish a typed clear marker so all observers converge;
- retain canonical runtime lifecycle truth;
- send no bytes to the child;
- create no verification/evidence event;
- create no process state transition.

The owner replay layer may discard replay presentation older than the clear marker for that pane view, subject to existing bounded replay truth.

Ctrl-L or shell-specific clear commands remain ordinary user input and are not equivalent to pane.clear.

## AD-012-13 — Reuse existing terminal rendering dependencies

No new terminal-rendering dependency is selected by this Plan.

Current accepted direct dependencies remain sufficient planning inputs:

- crossterm 0.29.0;
- ratatui 0.30.2;
- ratatui-textarea 0.9.2;
- vt100 0.16.2;
- portable-pty 0.9.0.

TUI/workbench should reuse existing rendering/input infrastructure.

Desktop must reuse the existing trusted Tauri Rust host and existing terminal bridge/surface. A later Task may refactor that surface but may not add a new terminal framework merely for parity.

## AD-012-14 — Scrollback and rendering must not duplicate owner replay per observer

The existing owner replay budget remains canonical for runtime output history.

A pane view must reference bounded runtime replay rather than retaining an unbounded duplicate transcript per client.

Plan ceilings:

~~~text
existing per-runtime replay ceiling: <= 8 MiB
existing aggregate owner replay ceiling: <= 64 MiB
pane-local rendered/parsed working buffer: <= 2 MiB beyond referenced replay
graphics payload retained per pane: <= 8 MiB
notification pending queue: <= 128 items
~~~

Tasks must prove the exact implementation does not multiply the 8 MiB runtime replay budget by number of observers.

## AD-012-15 — The 24-agent catalog is static, compile-time, and data-only

The first Spec 012 program uses a Winds-owned static detector catalog for all 24 required families.

Each catalog entry may contain data such as:

- canonical family enum;
- accepted executable basenames;
- accepted structured launch metadata patterns;
- provider-native structured identifiers where independently available;
- platform applicability;
- detector revision.

The catalog contains no executable plugin code.

No hot-reload system, downloaded manifest, dynamic library, script hook, plugin runtime, marketplace, or remote detector feed is planned.

A future manifest refresh system requires a separate amendment if static compiled rules prove insufficient.

## AD-012-16 — Agent observation sources are ordered by trust and never terminal prose

AgentObservationV1 records:

~~~text
observation_id: AgentObservationId
family
source_class
confidence_class
freshness
multiplexer_workspace_id: MultiplexerWorkspaceId
git_workspace_id: GitWorkspaceId optional
tab_id: TabId
pane_id: PaneId
runtime_namespace_id: RuntimeNamespaceId optional
provider_native_session_id optional
observed_at / owner generation
structured evidence summary
~~~

The two workspace domains are never overloaded:

- `multiplexer_workspace_id` is required because the observation is presented inside one exact multiplexer topology;
- `git_workspace_id` is optional worktree/repository context only when independently known;
- neither field may be inferred from the other;
- a detector or client MUST NOT use a generic `workspace_id` field whose domain is implicit.

Source classes, strongest first:

1. WINDS_LAUNCH_METADATA — exact executable/arguments from a Winds-owned launch already authorized elsewhere;
2. OWNED_PROCESS_METADATA — structured process/runtime metadata available from an owned runtime without parsing terminal prose;
3. PROVIDER_STRUCTURED_METADATA — independently validated provider-native structured metadata;
4. USER_DECLARED_PRESENTATION — explicit human label; never promoted to provider-execution proof.

Terminal text is not a detection source.

Unknown/ambiguous/stale/unavailable remain first-class outcomes.

## AD-012-17 — Detection support does not authorize agent launch

All 24 families may be detected/presented.

The first Spec 012 program does NOT admit generic agent.start.

Inherited provider execution authorization remains unchanged.

This Plan allows later Tasks to evaluate only these narrow agent actions:

### agent.prompt

May be admitted only as a controller-only exact-pane terminal-input operation against an already live, currently detected agent observation.

It is not a provider API call and does not prove provider/model identity.

### agent.send_keys

Same authority as exact pane input. Requires current PaneId + RuntimeNamespaceId + controller lease.

### agent.wait

Observer-safe wait over structured Winds events such as:

- lifecycle transition;
- explicit accepted attention state;
- process exit;
- observation freshness change.

It MUST NOT parse "done", "completed", prompts, or approval-looking terminal text.

### agent.start

Rejected from the first Spec 012 program.

A future agent-start seam requires an exact separately governed authority decision because detection is not execution authorization.

## AD-012-18 — Needs You uses structured state only

The initial Needs You aggregator may consume only trusted structured sources already owned by Winds, such as:

- canonical workflow/session attention state;
- explicit controller conflict or takeover requirement;
- OUTCOME_UNKNOWN requiring human reconciliation;
- explicit typed runtime failure requiring a user choice;
- explicit provider-structured attention event from a separately accepted adapter;
- worktree action requiring explicit trust or destructive confirmation.

It MUST NOT infer Needs You from:

- terminal prose;
- ANSI title;
- agent markdown;
- model confidence;
- regexes over output;
- detected question marks;
- phrases such as "approve", "done", or "need input".

Every item carries exact source and immutable binding.

## AD-012-19 — Attention deduplication uses stable event identity

A trusted attention item key is derived from:

~~~text
source_domain
source_event_id or deterministic source sequence
exact target identity
attention kind
owner generation where applicable
~~~

Display grouping may coalesce equivalent items, but canonical source records remain attributable.

Stale events never rebind to newly focused panes or sessions.

## AD-012-20 — Command palette is a closed typed registry

The desktop command palette remains command-first but not shell-first.

The first Spec 012 program uses a compile-time Winds action registry.

Allowed categories include:

- navigate workspace/tab/pane;
- focus exact Session/pane/agent;
- apply safe topology operation;
- open bounded Winds surface;
- refresh;
- open/trust/create/open/remove worktree through exact typed workflows;
- pane clear;
- bounded agent prompt/send-keys only if separately admitted by exact Tasks.

User-defined arbitrary shell commands are not part of the first program.

Unknown action IDs fail closed.

Search ranking affects presentation only; selection binds the exact action target snapshot before execution.

## AD-012-21 — Worktree discovery reuses current safe Git authority

System Git remains the authority.

Reuse and extend the existing Rust Git/workspace seams:

- Repo::open;
- inspect_existing_workspace;
- canonical worktree root;
- canonical Git common directory;
- read-only porcelain/plumbing helpers;
- external state-root safety;
- existing store GitWorkspaceId records.

Do not introduce libgit2/JGit or a second Git implementation.

Worktree discovery should use machine-readable Git output, planned as git worktree list --porcelain -z or the smallest equivalent directly proven by Tasks.

Human-formatted Git output must not be parsed.

## AD-012-22 — Repository trust is explicit and repository-scoped

Before consequential worktree mutation, the repository must have an explicit accepted trust record bound to canonical RepositoryIdentity.

Trust is not branch trust, path-prefix trust, pane trust, or agent trust.

Every mutation reopens/revalidates the repository through system Git and confirms the canonical Git common directory still matches the trusted identity.

If the path now resolves to another repository identity, trust does not transfer.

The admitted same-user non-isolation statement remains unchanged: arbitrary malicious same-user code is outside the claimed isolation boundary.

## AD-012-23 — Explicit worktree membership survives discovery drift

The post-Spec Herdr 6c62707a research finding is adopted as a design constraint, not source code.

Winds distinguishes:

~~~text
DISCOVERED_WORKTREE
EXPLICIT_MEMBER
CREATED_BY_WINDS_MEMBER
STALE_EXPLICIT_MEMBER
REMOVED_EXPLICIT_MEMBER
~~~

Rules:

- discovery may add/update observations;
- discovery failure cannot erase EXPLICIT_MEMBER;
- unavailable explicit members become STALE_EXPLICIT_MEMBER;
- only explicit accepted removal or exact proven repository deletion may remove membership;
- worktree path reuse by a different GitWorkspaceId cannot inherit the old membership;
- UI grouping uses explicit membership first, discovery only as observation.

## AD-012-24 — Worktree create is exact and non-destructive by default

Planned initial create behavior:

1. require trusted RepositoryIdentity;
2. resolve user-selected base ref to an exact commit OID using read-only Git;
3. choose/validate destination under the configured worktree root policy;
4. reject existing destination;
5. reject symlink/reparse/path traversal ambiguity;
6. create a detached worktree at the exact OID by default;
7. optionally create a new local branch only when the user explicitly supplies a new branch name;
8. never use --force;
9. re-inspect/register the resulting GitWorkspaceId;
10. record CREATED_BY_WINDS_MEMBER only after exact post-create identity succeeds.

Existing-branch checkout semantics may be admitted only if Tasks prove safe handling for branches already checked out elsewhere.

No remote fetch/pull is implicit.

## AD-012-25 — Worktree open/adopt is observation plus explicit membership

Opening an existing worktree reuses inspect_existing_workspace.

The operation:

- proves Git identity;
- records current branch/head/dirty observation;
- requires explicit repository trust before consequential follow-up;
- adds explicit membership only after accepted user action.

Opening alone performs no checkout, branch mutation, fetch, pull, merge, push, or PR action.

## AD-012-26 — Worktree removal never force-removes

Removal requirements:

- exact GitWorkspaceId and canonical path;
- trusted RepositoryIdentity;
- current identity revalidation;
- clean state unless a separately accepted destructive amendment exists;
- no unmerged/untracked loss;
- not the canonical main worktree when Git forbids it;
- no active Winds runtime/pane binding unless explicitly stopped/unbound first;
- system Git worktree remove without --force;
- no fs::remove_dir_all fallback;
- post-operation git worktree list proof;
- explicit membership removed only after exact success.

Failure retains the record as stale/recovery-required.

## AD-012-27 — Worktree root policy is explicit and reversible

The default worktree root is a user-owned directory outside the repository working tree and outside the Winds state root.

Tasks must freeze exact platform resolution.

The configured root:

- must be absolute/canonical;
- must reject traversal;
- must reject ambiguous symlink/reparse substitutions;
- must not be a repository state directory;
- must be changeable without moving existing worktrees;
- creates no automatic cleanup obligation for old roots.

## AD-012-28 — No Git landing authority enters the multiplexer

No Spec 012 operation may perform:

- merge;
- rebase;
- cherry-pick;
- push;
- PR creation;
- PR approval;
- branch-protection change;
- winner selection;
- automatic landing.

A pane/agent asking for these actions remains untrusted text.

Existing Winds verification/Git governance remains authoritative.

## AD-012-29 — Event queues reuse existing bounded client backpressure

Spec 012 does not create another unbounded event system.

Multiplexer/agent/attention events share the existing local-control connection discipline.

Plan ceilings per client:

~~~text
all queued local-control payload combined: <= 4 MiB
multiplexer topology event backlog: <= 1024 events
agent observation event backlog: <= 1024 events
attention event backlog: <= 512 events
~~~

When a presentation event is dropped or retained history is no longer available, clients receive an explicit gap/events-lost state and MUST mark the affected projection stale.

Recovery is:

1. establish a fresh subscription boundary;
2. fetch a fresh authoritative snapshot;
3. replace the stale cached projection;
4. continue from newly observed events.

Buffered events are not unconditionally replayed over the fresh snapshot. A topology event may be applied only when its exact resulting TopologyGeneration is the next valid generation for the cached workspace. Any generation discontinuity requires another resnapshot.

Agent-observation and attention streams use the same fail-explicit principle: sequence/history loss invalidates the cached stream projection and requires a fresh snapshot. Authority state is re-queried; it is never reconstructed from missing presentation events.

## AD-012-30 — Snapshot size is bounded by bytes and counts

A single encoded `MULTIPLEXER_SNAPSHOT` frame MUST fit within 256 KiB total, including the 4-byte length prefix and the complete serialized protocol envelope.

For the first v2 program:

~~~text
MAX_MULTIPLEXER_SNAPSHOT_FRAME_BYTES = 262144
MAX_MULTIPLEXER_SNAPSHOT_JSON_BYTES  = 262140
~~~

The snapshot encoder MUST measure the exact final encoded frame, not estimate object size from counts.

Variable-length serialized fields are bounded before persistence or publication:

~~~text
opaque ID textual encoding: <= 96 UTF-8 bytes
workspace alias: <= 128 UTF-8 bytes
tab alias: <= 128 UTF-8 bytes
pane label: <= 256 UTF-8 bytes
pane title: <= 256 UTF-8 bytes
layout-template name: <= 128 UTF-8 bytes
agent presentation label in multiplexer projection: <= 128 UTF-8 bytes
closed presentation-status/detail text: <= 512 UTF-8 bytes per field
~~~

Topology/presentation persistence MUST use closed typed fields. The authoritative multiplexer snapshot MUST NOT contain an unbounded arbitrary metadata map.

The first program also plans explicit count ceilings:

~~~text
multiplexer workspaces per owner: <= 32
tabs per workspace: <= 32
panes per tab: <= 64
aggregate panes across owner: <= 256
agent observations retained as current: <= 512
saved layout templates: <= 128
explicit worktree memberships: <= 256
~~~

Count ceilings and byte ceilings are independent upper bounds. A state may be below every count ceiling and still exceed the serialized-byte budget.

Before accepting any mutation that changes authoritative snapshot content, the owner MUST encode the candidate resulting snapshot through the exact protocol-v2 serializer and enforce the total-frame limit. If the encoded frame would exceed 256 KiB:

- reject the mutation with a typed `SNAPSHOT_LIMIT_EXCEEDED` outcome;
- do not increment TopologyGeneration;
- do not persist the oversized candidate as accepted state;
- do not publish partial/truncated authoritative topology;
- preserve the previous accepted topology unchanged.

Reads of an already accepted topology are never silently truncated. If startup encounters legacy/corrupt persisted state whose exact encoded snapshot exceeds the bound, quarantine that multiplexer workspace as unavailable/recovery-required under the corrupt-record policy rather than emitting an oversized or partial snapshot.

Agent-observation and attention snapshots are separate bounded protocol projections. Their Tasks must define their own exact encoded-frame byte budgets within the same 256 KiB protocol-frame ceiling; their 512/current-event count limits alone are not treated as byte-size proof.

These are first-program resource ceilings, not product marketing claims.

Tasks may tighten them. Relaxation requires measured evidence and a Plan amendment if it breaks the frame/resource model.

## AD-012-31 — Directional navigation uses geometry with deterministic tie-breaking

For directional focus:

1. candidates must lie in the requested half-plane;
2. primary-axis distance is minimized;
3. overlap on the orthogonal axis is preferred;
4. orthogonal distance is next;
5. stable PaneId lexical byte order is the final deterministic tie-break.

Tasks may improve the exact metric but MUST preserve determinism and stable tie-breaking.

Current focus is a presentation input, not action authority.

## AD-012-32 — Accessibility is part of the topology contract

Primary actions require:

- keyboard path;
- visible focus;
- pointer path where pointer hardware exists;
- non-color state distinction;
- reduced-motion compatibility;
- high-contrast compatibility;
- scaled-text tolerance.

Desktop and TUI may render differently but must preserve the same identity/authority semantics.

No animated topology transition may delay or obscure the authoritative accepted state.

## AD-012-33 — Clipboard, links, graphics, notifications stay local and bounded

The first program may expose local-only behavior through existing trusted host seams.

Rules:

- no remote bridge;
- no auto-open from terminal text;
- links require scheme validation and explicit action;
- clipboard write requires explicit user action/policy;
- graphics are presentation only;
- notifications/toasts are bounded and suppressible;
- titles/graphics/notifications cannot generate trusted status.

No new clipboard/graphics dependency is selected by this Plan.

## AD-012-34 — No new direct dependency is planned

The first program should use current direct dependencies and standard library behavior.

Current relevant dependencies include:

- crossterm 0.29.0;
- libc 0.2.189;
- portable-pty 0.9.0;
- ratatui 0.30.2;
- ratatui-textarea 0.9.2;
- rusqlite 0.40.2;
- serde / serde_json;
- sha2 0.10;
- vt100 0.16.2;
- windows-sys 0.61.2 on Windows.

No Tokio, generic RPC framework, plugin framework, terminal framework, Git library, UUID crate, random crate, database, search engine, or manifest engine is planned.

Any new dependency requires an exact Tasks-stage necessity/provenance gate.

## AD-012-35 — Herdr remains research/test-design evidence only

No Herdr source is admitted by this Plan.

No direct copy, adapted copy, or test port is authorized.

Herdr is used to challenge omissions in:

- topology operations;
- navigation/focus behavior;
- terminal UX edge cases;
- detector family coverage;
- workspace agent presentation;
- worktree membership behavior;
- resource/backpressure cases.

Any later donor proposal requires a separate exact source-path/revision/license/provenance/security/removal analysis and explicit Tasks authorization.

---

## AD-012-36 — Required client capability is preflighted before consequential side effects

A requested operation must identify the client capability it actually needs before any side effect.

Examples:

- TUI/CLI interactive attach that requires terminal rows/columns must prove usable terminal geometry first;
- Desktop pane binding must prove the trusted Desktop terminal surface is initialized and can provide required geometry/input semantics;
- a noninteractive observer/list/snapshot operation does not require terminal geometry merely because another client surface does.

If the required capability is unavailable, Winds fails with a typed capability-unavailable outcome before:

- creating a MultiplexerWorkspaceId, TabId, or PaneId solely for the failed request;
- starting or binding a RuntimeNamespaceId;
- creating a session/runtime persistence record;
- creating filesystem state;
- starting the persistent owner solely to service that unusable interactive request;
- changing TopologyGeneration.

Capability preflight is not provider detection and does not grant process authority.

A later failure after capability preflight still follows the operation's normal transactional/outcome-unknown rules; this decision only prevents known-invalid interactive requests from creating avoidable partial state.

---

# Protocol v2 Planning Model

## Handshake

The existing private local-control handshake remains.

Spec 012 requires:

- minimum_protocol_version = 2;
- maximum_protocol_version = 2 for first v2 clients;
- exact owner generation;
- explicit mismatch against v1;
- no downgrade;
- no automatic old-owner kill.

## Topology request binding

Every topology mutation request includes:

~~~text
multiplexer_workspace_id
expected_topology_generation
exact target ID(s)
closed operation variant
request sequence
owner generation
~~~

The response includes:

~~~text
accepted topology generation
exact changed target IDs
typed outcome
snapshot digest or bounded changed projection
~~~

If expected generation is stale, no mutation occurs.

## Worktree request binding

Every consequential worktree operation includes:

~~~text
repository_identity
trust_record_revision or equivalent exact trust snapshot
git_workspace_id when existing
exact canonical destination/path where applicable
exact base commit OID for create
operation variant
request sequence
owner generation
~~~

No operation accepts a raw shell command.

## Agent operation binding

Any admitted prompt/send-keys action includes:

~~~text
agent_observation_id
pane_id
runtime_namespace_id
owner_generation_id
controller connection identity
observation freshness token
bounded input payload
~~~

Any mismatch fails closed.

---

# Persistence and Recovery Model

## Live owner state

The owner holds the current topology model in memory and serializes mutations.

SQLite persistence happens through accepted Rust store methods after mutation validation.

A persistence failure after an in-memory mutation must not be silently reported as durable success. Tasks must choose one of:

- commit persistence before publishing accepted generation; or
- mark the operation failed/outcome-unknown and force resnapshot/recovery.

The final Tasks design must be atomic enough that clients never receive a durable-success claim for state not recoverable after owner restart.

## Restart reconstruction

On restart:

- load persisted topology;
- validate format version and all IDs;
- reject malformed/cyclic/duplicate topology;
- create new owner generation;
- reconcile runtime references;
- preserve explicit worktree memberships;
- mark missing worktrees stale;
- never recreate processes automatically;
- never recreate agent observations as fresh without new evidence;
- never recreate Needs You solely from old display text.

## Corrupt topology record

A corrupt multiplexer snapshot must not prevent Winds core verification from starting.

Plan behavior:

- quarantine/report the multiplexer workspace record as unavailable;
- preserve the raw record for recovery if practical;
- do not auto-delete;
- do not infer replacement IDs;
- continue unrelated workspaces/runtimes where safe.

## Saved template versioning

LayoutTemplateV1 includes an explicit format version.

Unknown versions fail with an unsupported-layout error.

No silent structural downgrade is allowed.

---

# Agent Detection Qualification Plan

Every one of the 24 families receives deterministic fixtures covering:

- positive structured match;
- near-match negative;
- ambiguous executable/name;
- stale observation;
- replaced pane/runtime;
- unsupported platform where applicable;
- user-declared label that must not promote detection;
- terminal prose containing the family name that must not promote detection.

Required families:

~~~text
Pi
Claude
Codex
Gemini
Cursor
Devin
Antigravity
Cline
Omp
Mastracode
OpenCode
GithubCopilot
Kimi
Kiro
Droid
Amp
Grok
Hermes
Kilo
Qodercli
Qwen
Letta
Maki
Muse
~~~

A family may be RELEASED_DETECTION_ONLY if no execution seam is admitted.

The UI must state that distinction.

---

# Needs You and Attention Plan

Planned attention kinds are closed and typed, approximately:

~~~text
CONTROLLER_REQUIRED
CONTROLLER_CONFLICT
OUTCOME_UNKNOWN
RUNTIME_OWNERSHIP_LOST
EXPLICIT_AGENT_ATTENTION
WORKTREE_TRUST_REQUIRED
WORKTREE_DESTRUCTIVE_CONFIRMATION
WORKTREE_STALE_MEMBERSHIP
PROTOCOL_UPGRADE_REQUIRED
~~~

Tasks may reduce this list.

There is no FREEFORM_AGENT_MESSAGE attention kind.

Attention rendering always includes source labels and exact binding.

---

# Worktree Safety Plan

## Discovery

Use read-only system Git.

Discovery records:

- GitWorkspaceId;
- canonical worktree root;
- canonical Git common directory;
- HEAD OID if available;
- branch/detached state;
- dirty state;
- locked/prunable metadata where directly available;
- observation timestamp.

Discovery does not mutate membership.

## Trust

Trust is an explicit accepted action stored against canonical RepositoryIdentity with a monotonic trust-record revision or equivalent exact snapshot.

A repository may be observed but untrusted.

Untrusted repositories can be displayed read-only but cannot create/remove worktrees.

No separate opaque trust identity is required unless Tasks prove a concrete need; repository identity plus exact trust record revision is the default simpler model.

## Create

No force flag.

No implicit remote fetch.

No implicit branch overwrite.

Destination must be empty/nonexistent and policy-valid.

## Remove

No force flag.

Dirty state blocks.

Active runtime bindings block until explicitly handled.

Failure leaves recovery state visible.

## Concurrent create/remove

Owner serializes worktree mutations by RepositoryIdentity.

Different repositories may proceed independently only if Tasks prove the implementation remains bounded and simpler than global serialization.

The first implementation may globally serialize worktree mutations for simplicity.

---

# Threat Model Implementation Plan

## Stale topology

Tests cover:

- stale generation;
- closed pane ID;
- replacement pane in same visual location;
- renamed/reordered tabs;
- delayed focus;
- duplicate aliases;
- replayed mutation request.

No retargeting is permitted.

## Cross-target terminal input

Deterministic barriers exercise:

- focus changes while input request is delayed;
- controller takeover during send-keys;
- pane swap during prompt;
- stale agent observation during prompt;
- duplicate request sequence.

Exactly one immutable runtime target is allowed.

## Forged terminal/agent content

Fixtures inject:

- VERIFIED;
- ACCEPTED;
- Needs You;
- provider/model names;
- JSON-looking protocol frames;
- command-looking text;
- clickable link-looking text;
- title escape sequences.

None may mutate trusted state.

## Detector ambiguity

Fixtures cover executable basename collision, wrapper scripts, renamed binaries, shell aliases, stale process metadata, and user labels.

Ambiguous remains ambiguous.

## Worktree path substitution

Tests cover:

- symlink path;
- Windows reparse/junction behavior;
- nested repository confusion;
- Git common-dir change;
- destination reuse;
- stale explicit membership;
- deleted/recreated path;
- dirty worktree;
- current main worktree;
- repository trust mismatch.

## Resource exhaustion

Campaigns cover:

- maximum topology;
- repeated split/close churn;
- many current agent observations;
- high output;
- slow observers;
- notification spam;
- oversized layout JSON;
- oversized clipboard/graphics request;
- repeated worktree discovery refresh.

---

# Platform Plan

## Linux

Directly qualify:

- existing Unix private owner transport remains intact under protocol v2;
- terminal input/mouse/focus behavior;
- CJK/IME where environment supports direct automation;
- symlink/path safety;
- worktree create/open/remove;
- topology/resource campaigns.

## macOS

Qualify separately:

- protocol v2 over accepted Unix transport;
- terminal keyboard/mouse/focus;
- IME/text composition where directly exercisable;
- canonical paths/symlinks;
- worktree operations;
- resource cleanup.

Linux evidence does not substitute.

## Native Windows

Directly qualify:

- protocol v2 over accepted named pipe;
- DACL behavior remains unchanged;
- ConPTY pane input/resize/lifecycle;
- mouse/keyboard/cursor behavior;
- reparse/junction path safety;
- worktree create/open/remove;
- path-length/case behavior;
- no WSL inference.

## WSL2

WSL remains a separate Linux execution domain.

No native-Windows ↔ WSL multiplexer bridge is introduced.

A WSL claim requires direct WSL evidence.

---

# Performance and Resource Budgets

Initial engineering targets:

~~~text
owner multiplexer idle CPU increment over Spec 011 baseline: <= 0.25% one logical core p95
owner multiplexer idle RSS increment: <= 24 MiB
local multiplexer snapshot p95: <= 100 ms
single topology mutation owner-side p95: <= 20 ms
directional focus resolution p95 at 256 panes: <= 10 ms
client projection update after accepted local mutation p95: <= 100 ms
agent observation refresh for 256 panes p95: <= 250 ms
worktree read-only discovery for 64 worktrees p95: <= 500 ms excluding pathological filesystem latency
saved layout apply for 256-pane template p95: <= 100 ms before any explicit runtime starts
aggregate panes: <= 256
current agent observations: <= 512
saved templates: <= 128
explicit worktree memberships: <= 256
control frame: <= 256 KiB
all queued client payload: <= 4 MiB
~~~

These are qualification ceilings, not marketing claims.

No optimization may weaken exact target binding, stale-generation rejection, trust checks, runtime controller checks, or evidence truth.

---

# Dependency and Provenance Gates

Before adopting any dependency or donor slice, the owning Task must prove:

- exact version/revision/checksum;
- license/notices;
- Rust 1.97.1/MSRV compatibility where applicable;
- frontend runtime compatibility where applicable;
- platform support;
- transitive impact;
- relevant security advisories;
- why standard library/current dependency is insufficient;
- smallest feature set;
- update/removal path.

No dependency is admitted by this Plan.

For Herdr, root Apache-2.0 does not authorize bulk copying or third-party/vendor material transitively.

---

# Implementation-Boundary Map

Planned Winds-owned seams, exact filenames deferred to Tasks:

~~~text
multiplexer/
  domain          IDs, topology, generation, focus, typed operations
  persistence     0014 mappings + snapshot/template validation
  service         owner-side serialized topology authority
  protocol        protocol-v2 typed payloads and bindings
  projection      bounded client snapshots/events
  agent_catalog   static 24-family detection rules
  agent_state     observations/freshness/source truth
  attention       structured Needs You aggregation
  worktree        trust, discovery, explicit membership, safe mutation
  navigation      deterministic directional/neighbor/edge rules
~~~

Existing modules remain authoritative for:

- persistent owner/private transport: src/persistent_runtime/**;
- PTY/ConPTY lifecycle and controller authority;
- Git process execution and Repo identity: src/git.rs + workspace modules;
- canonical Git workspace identity: src/workspace.rs;
- SQLite Store/migrations;
- canonical Project/Session/Workstream identity;
- verification/evidence/human-decision truth;
- Tauri trusted Rust host;
- desktop left/right dock bridges;
- existing command palette presentation;
- TUI/workbench rendering.

No duplicate Git, terminal, verification, or database authority is planned.

---

# Proposed Dependency-Ordered Implementation Slices

Tasks should decompose the implementation into small reviewable units approximately as follows.

1. **Domain freeze** — MultiplexerWorkspaceId/TabId/PaneId/TopologyGeneration/LayoutTemplateId, topology tree, immutable-ID invariants, closed error vocabulary. No persistence/protocol/process changes.

2. **Topology state machine** — create/rename/reorder/focus/split/swap/move/resize/zoom/close using expected-generation semantics and deterministic navigation. Pure tests only.

3. **Persistence format** — TopologySnapshotV1/LayoutTemplateV1 validators and migration 0014 with no owner protocol change yet.

4. **Owner multiplexer service** — integrate topology state machine into existing owner authority loop; no client protocol exposure beyond in-process tests.

5. **Protocol v2 core** — exact version bump, mismatch behavior, bounded multiplexer snapshot/events, no runtime mutation additions yet.

6. **Shared Rust client projection** — list/snapshot/observe topology through existing client seam; no renderer-direct connection.

7. **TUI topology projection** — workspace/tab/pane presentation, keyboard/pointer-equivalent terminal controls where applicable, deterministic focus. No agent/worktree mutation yet.

8. **Desktop topology projection** — Tauri typed host commands/bridge, recursive panes/tabs, accessibility, no generic invoke surface.

9. **Pane/runtime binding** — exact PaneId ↔ RuntimeNamespaceId reference, controller-only runtime actions, pane clear presentation epoch, stale-binding tests.

10. **Terminal UX hardening** — scrollback/search/selection/copy/link/local graphics/notification behavior within resource/security bounds.

11. **Static 24-family detector catalog** — data model and positive/negative/ambiguous/stale fixtures; no execution.

12. **Agent observation service** — source/freshness/binding lifecycle and bounded projections/events.

13. **Agent dock/presentation** — compact list/filter/sort/view/focus/rename presentation labels, no launch.

14. **Structured Needs You** — trusted source aggregation/deduplication/stale semantics, no terminal prose inference.

15. **Narrow agent prompt/send-keys/wait evaluation** — admit only exact-pane controller transport and observer structured wait if all authority tests pass; agent.start remains prohibited.

16. **Repository trust + discovery** — read-only Git worktree inventory, explicit trust records, stale explicit memberships; no create/remove.

17. **Worktree create/open** — exact OID, safe destination, detached default, optional explicit new branch, no force/fetch/landing.

18. **Worktree remove/recovery** — clean-only, no force, runtime-binding guards, exact post-remove proof, stale recovery state.

19. **Command palette typed registry** — Winds-defined exact actions only; no arbitrary user shell command registry.

20. **Adversarial identity/security** — stale topology, cross-target input, forged terminal/agent content, detector ambiguity, path substitution, malformed protocol v2.

21. **Stress/performance** — maximum topology, high output, slow clients, churn, detector scale, worktree discovery scale, resource ceilings.

22. **Platform qualification** — Linux/macOS/native Windows direct evidence; WSL only if separately exercised.

23. **Parity/requirements reconciliation** — map all 92 FR, 30 SC, and exact 142 Spec 012 ledger rows to evidence or explicit accepted non-applicability.

24. **Final Spec 012 implementation closeout** — only after all separately authorized Tasks land and exact evidence proves no scope leakage.

Tasks may split these slices further. They MUST NOT combine protocol versioning, migration, owner integration, agent detection, worktree mutation, and UI into one first implementation PR.

---

# Explicit Rejections for the First Spec 012 Program

Reject unless a separately accepted amendment changes scope:

- second owner/daemon/service;
- public TCP/HTTP/WebSocket/RPC;
- SSH/remote machines/cloud relay;
- generic RPC/method dispatcher;
- arbitrary shell command palette;
- downloaded detector manifests;
- detector hot reload;
- plugin runtime/marketplace;
- generic integration installer;
- provider/model router;
- agent.start generic execution;
- terminal-text agent detection;
- terminal-text Needs You inference;
- transcript persistence/search database;
- second terminal backend;
- libgit2 or alternate Git authority;
- git worktree --force;
- filesystem recursive-delete fallback for worktrees;
- implicit fetch/pull;
- automatic merge/rebase/cherry-pick/push/PR/landing;
- live owner binary handoff/updater;
- same-user sandbox claims;
- bulk Herdr copying.

---

# Verification Strategy

Every Tasks/implementation slice requires deterministic exact-head evidence.

The final program must include:

- identity collision/reuse tests;
- topology generation/stale request tests;
- deterministic navigation tests;
- recursive split/move/swap/resize/zoom tests;
- persistence corruption/version tests;
- layout template authority-stripping tests;
- protocol v1/v2 mismatch tests;
- protocol-v2 malformed/replay/binding tests;
- owner restart presentation/runtime reconciliation;
- multi-client topology convergence;
- exact PaneId ↔ RuntimeNamespaceId controller tests;
- pane clear presentation-marker tests;
- high-output/scrollback bounds;
- link/clipboard/graphics/notification adversarial tests;
- all 24 agent detector positive/negative/ambiguous/stale suites;
- terminal-prose false-positive tests;
- Needs You source/dedup/stale tests;
- exact agent prompt/send-keys target race tests if admitted;
- repository trust tests;
- worktree discovery/create/open/remove tests;
- dirty worktree and no-force tests;
- explicit membership preservation under discovery drift;
- symlink/reparse/path reuse tests;
- large-topology/resource campaigns;
- accessibility tests;
- native platform evidence;
- regression suites for Specs 001-011;
- explicit no-change evidence for verification/human/Git landing authority.

No fixture-only proof may be relabelled as native/provider execution proof.

---

# Spec 012 Requirement Coverage Map

The Plan covers the specification groups as follows:

| Spec group | Plan decisions |
| --- | --- |
| FR-001..FR-015 | AD-012-02 through AD-012-06, AD-012-31 |
| FR-016..FR-030 | AD-012-11 through AD-012-14, AD-012-33 |
| FR-031..FR-047 | AD-012-15 through AD-012-17 |
| FR-048..FR-054 | AD-012-18 through AD-012-19 |
| FR-055..FR-064 | AD-012-08 through AD-012-10, AD-012-20 |
| FR-065..FR-074 | AD-012-21 through AD-012-28 |
| FR-075..FR-084 | AD-012-29 through AD-012-33 |
| FR-085..FR-092 | Platform Plan, dependency/provenance gates, explicit rejections |
| SC-001..SC-006 | topology/identity/persistence slices |
| SC-007..SC-011 | 24-family detection qualification |
| SC-012..SC-016 | attention/exact-target/protocol tests |
| SC-017..SC-018 | worktree trust/mutation tests |
| SC-019..SC-027 | presentation/security/resource/platform campaigns |
| SC-028..SC-030 | final parity, inherited-truth, review/landing reconciliation |

All exact 142 ledger rows assigned to Spec 012 remain an omission-prevention checklist at Tasks and final reconciliation. They are not source-admission authority.

---

# Review and Landing Discipline

Every Plan/Tasks/implementation candidate must satisfy repository governance:

1. exact authorized scope only;
2. deterministic quality on exact head;
3. author correctness/safety/architecture review;
4. Ponytail/YAGNI review;
5. fresh independent substantive review;
6. zero unresolved material findings/threads;
7. exact base/head/tree/scope/ruleset/mergeability reconciliation;
8. immediate Herdr freshness recheck when the unit depends on Herdr research;
9. guarded normal merge using exact expected head;
10. merge tree/ordered-parent/GitHub-signature verification;
11. every actually-triggered post-merge push workflow succeeds.

A review of an earlier head is not acceptance evidence for a moved candidate unless the review explicitly covers the successor range.

Rate-limit, billing-limit, neutral check, and unavailable reviewer states must remain labelled exactly as such.

---

# Plan Acceptance Gate

This Plan may land only if the exact final candidate proves:

- changed scope is exactly specs/012-workspace-multiplexer-agent-plane/plan.md unless a separately justified governance-only correction is required;
- Spec 012 Entry and Specification are canonically closed;
- post-Spec merge quality 35517252780 is successful;
- all 92 FR and 30 SC have an implementation path;
- the exact 142 Spec 012 ledger rows remain accounted for;
- the existing Spec 011 owner remains the only live local owner;
- MultiplexerWorkspaceId/TabId/PaneId are separate from Git Workspace/Project/Session/runtime/provider/candidate/evidence identities;
- PaneId is not scoped to topology generation and is never reused for replacement panes;
- topology mutations use exact immutable IDs plus expected generation;
- saved layout templates strip live authority and generate fresh IDs;
- migration 0014 is narrow, justified, and Tasks-gated;
- protocol v2 extends the existing private protocol rather than creating another server;
- v1/v2 mismatch fails explicitly with no silent downgrade;
- topology-only actions cannot acquire runtime controller authority;
- PTY/ConPTY ownership remains in Spec 011 runtime control;
- pane.clear is presentation-only and does not send child input;
- scrollback/event memory remains bounded;
- all 24 agent families use a static data-only detection catalog;
- terminal prose is not detection or Needs You authority;
- agent.start remains prohibited in the first program;
- any prompt/send-keys action is exact-pane controller-only if later Tasks admit it;
- worktree operations reuse system Git and existing Git identity;
- repository trust is explicit;
- explicit worktree membership survives discovery drift;
- worktree create/remove never uses --force;
- no merge/rebase/cherry-pick/push/PR/landing authority is introduced;
- no remote/plugin/marketplace/updater scope is introduced;
- no new dependency is admitted by the Plan;
- Herdr remains research evidence only with no source admission;
- current Herdr plan-stage head is freshly rechecked immediately before landing and any material movement is reconciled;
- interactive client/surface capability is preflighted before topology/runtime/persistence side effects for operations that require that capability, without imposing TTY requirements on Desktop;
- repository quality succeeds on the exact final head;
- author correctness/safety/architecture review passes;
- Ponytail/YAGNI review challenges owner reuse, protocol v2, migration 0014, topology persistence, agent catalog, worktree membership/trust, resource ceilings, and any unnecessary abstraction;
- fresh independent substantive review challenges identity lifetime, stale-generation semantics, cross-target dispatch, agent detection truth, worktree safety, protocol upgrade behavior, failure recovery, resource bounds, and platform claims;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability is reconciled immediately before landing;
- guarded expected-head normal merge succeeds;
- merge tree/ordered parents/GitHub verification are reconciled;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical Plan landing may repository truth state:

~~~text
SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN=CLOSED_CANONICAL
SPEC_012_TASKS_AUTHORIZED=YES
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO

MULTIPLEXER_IMPLEMENTATION_AUTHORIZED=NO
PROTOCOL_V2_IMPLEMENTATION_AUTHORIZED=NO
MIGRATION_0014_AUTHORIZED=NO
AGENT_START_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
~~~

No production source, dependency, lockfile, migration, protocol, runtime behavior, detector, worktree mutation, desktop/TUI behavior, or donor-code change may begin until a separate exact Tasks candidate is independently qualified and canonically accepted.
