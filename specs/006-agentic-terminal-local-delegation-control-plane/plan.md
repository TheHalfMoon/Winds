# Implementation Plan: Agentic Terminal & Local Delegation Control Plane

## Summary

Build the smallest architecture that can prove Spec 006's differentiated loop without turning Winds into a daemon, generic agent platform, remote control plane, or model gateway:

1. preserve stable workspace -> workstream/task -> Winds-session identity independently from runtime-native IDs;
2. discover Codex and Claude Code safely without install/auth/prompt/model side effects;
3. express truthful `LIVE` / `RESUMED` / `RECONSTRUCTED` / `OWNERSHIP_LOST` continuity;
4. generate deterministic provenance-preserving context capsules from canonical Winds state;
5. evaluate one human-approved Planner -> Worker delegation with direct authority separate from delegation ceiling;
6. connect to Codex and Claude through their narrow official local structured surfaces, not a generic plugin system;
7. bind exact Git candidates to existing independent review and `winds verify` authority without changing verification semantics or automating landing.

The first implementation program remains **single-process and local-first**. It reuses the existing SQLite/WAL Store, system-Git discipline, execution ledger, ownership-loss semantics, candidate verification records, and CLI process.

It does **not** require a persistent service, public IPC, network listener, MCP, remote execution, generic runtime/plugin framework, custom renderer, model routing, SQL Studio, or LLM Observatory.

## Constitution Check

- **Evidence over claims**: runtime/model/tool prose remains `AGENT_REPORTED` unless Winds independently observes a fact or a human decides it. Existing candidate verification authority remains separate: REQUIRED.
- **Non-destructive Git safety**: no automatic winner, merge, rebase, cherry-pick, push, PR creation, force-clean, or primary-checkout mutation: REQUIRED.
- **Spec -> Plan -> Tasks before implementation**: Spec 006 is canonical at `c321d510463b207cd515ed391a47d4fb454fbe07`; this file is Plan only: PASS.
- **Canonical continuity**: Winds workspace/workstream/session identity survives runtime/native-session changes and does not collapse native resume into task continuity: REQUIRED.
- **Authority ceilings**: Planner direct authority, delegation ceiling, Worker grant, team/human ceiling, and enforcement quality remain distinct: REQUIRED.
- **Runtime/capability truth**: runtime != model; declared != locally observed; discovery != trust; worktree/ACP root != sandbox: REQUIRED.
- **Ponytail/YAGNI**: one Rust package/process; reuse Store/Git/execution primitives; two concrete runtime paths first; no generic JSON-RPC framework; no ACP dependency until a concrete runtime requires ACP: REQUIRED.
- **Independent review**: every implementation slice requires exact-head deterministic gates and independent review before acceptance: REQUIRED.

## Canonical Baseline

Planning base:

`c321d510463b207cd515ed391a47d4fb454fbe07`

Current repository facts:

- Constitution 1.1.0 is canonical.
- Spec 003 is closed canonical.
- Spec 006 `spec.md` is closed canonical.
- Winds remains one Rust 2024 binary, pinned to Rust `1.97.1`.
- Current direct dependencies are `libc`, exact `portable-pty 0.9.0`, exact `rusqlite 0.40.2` with bundled SQLite, `serde`, `serde_json`, and `sha2`.
- Existing source already has workspace, execution, terminal, command-history, process-ownership, Store, system-Git, Windows/WSL, and CLI surfaces.
- Existing `workspaces` persistence gives stable local workspace identity independent from mutable Git HEAD/dirty state.
- Existing `executions` and `execution_events` already separate lifecycle/source truth from candidate verification tables.
- Existing candidate verification remains in `candidate_runs`, `events`, `evidence_reports`, and `promotions` with exact candidate OID/tree identity.
- `ExecutionStatus::OwnershipLost` and fail-closed process ownership already exist and should be reused conceptually for agent-runtime ownership truth.

## External Runtime Decisions

These decisions were rechecked against current official primary sources during planning.

### Codex: official App Server, not TUI scraping

Use the official local Codex App Server when a later task reaches live control.

Current relevant protocol facts:

- App Server is a client-facing bidirectional JSON-RPC-like protocol framed as JSONL over stdio.
- The required startup sequence is explicit: send the `initialize` request, receive a successful `initialize` response, send the `initialized` notification, and only then send `thread/start`, `thread/resume`, or other protocol methods.
- durable Codex threads support create/resume/fork/archive behavior;
- turns emit structured progress/item notifications;
- the server can issue approval requests and wait for allow/deny;
- a local client may launch App Server as a child and retain bidirectional stdio.

Plan decisions:

- revalidate the exact discovered `codex` executable/version immediately before use;
- launch App Server as a directly owned child for the current Winds lifetime;
- implement the complete `initialize` -> successful response -> `initialized` notification handshake before any other request;
- use only the minimal typed methods/events required by accepted Tasks;
- bound line/frame, stdout/stderr, diagnostics, and malformed-message handling;
- do not scrape TUI escape sequences/text as the primary control plane;
- do not add a generic JSON-RPC dependency by default; existing `serde`/`serde_json` should handle the narrow vendor envelope unless implementation evidence proves otherwise;
- persist Codex thread identity only as a runtime-native binding, never canonical Winds task/session identity;
- treat approval requests as mediation events, not proof of OS containment;
- after Winds/process loss, live ownership is lost even if a durable thread ID remains a future resume candidate.

Primary current source: `https://developers.openai.com/codex/app-server`

### Claude Code: official local structured CLI first

Use Claude Code's documented local structured CLI surface before inventing another protocol layer.

Relevant current capabilities include:

- `claude -p` / `--print`;
- `--output-format json` or `stream-json`;
- `--input-format stream-json` in print mode;
- exact `--resume <session-id>`;
- recency-based `--continue`;
- `--allowedTools`, `--disallowedTools`, and `--permission-mode`;
- `--dangerously-skip-permissions`.

Plan decisions:

- revalidate the exact discovered `claude` executable/version before use;
- use structured print/stream mode, not TUI scraping;
- use only exact `--resume <session-id>` for a proven native binding;
- never use recency-based `--continue` as Winds canonical continuation;
- never use `--dangerously-skip-permissions`;
- keep the first live Claude role Planner/read-plan oriented unless a later exact task proves stronger non-interactive mediation consistent with Winds authority semantics;
- label vendor restriction `AGENT_NATIVE_ENFORCED` or weaker unless Winds/OS mediation independently proves stronger enforcement;
- if safe write/tool mediation would require excluded MCP or unsafe bypass flags, do not work around the boundary—keep Claude Planner-only in the first walking skeleton;
- persist returned native session ID only as a runtime-native binding;
- a new native session created from canonical Winds context is `RECONSTRUCTED`, not fake `RESUMED`.

Primary current source: `https://docs.anthropic.com/en/docs/claude-code/cli-usage`

### ACP remains pinned but not yet a dependency

Canonical governance pins:

```text
ACP_WIRE_PROTOCOL=1
ACP_SCHEMA=schema-v1.20.0
ACP_SCHEMA_COMMIT=5e89c71497fe07dd4ae633c181a17224f4a8956d
ACP_RUST_SDK=2.0.0
ACP_RUST_SDK_COMMIT=ce023279824149008659dd8f4b8b70266a7e8210
UNSTABLE_PROTOCOL_V2=DISABLED
UNSTABLE_MCP_OVER_ACP=DISABLED
```

This is a compatibility/provenance pin, not a mandate to add an unused crate.

Because Codex and Claude have official vendor-native structured local surfaces, the first implementation program SHOULD NOT add `agent-client-protocol` until a concrete accepted runtime path requires ACP. That task must then repeat the exact Cargo graph/checksum/license/MSRV/platform audit against the then-canonical tree.

### MCP remains out

The first Codex and Claude paths do not require MCP to prove canonical continuity or one bounded delegation. Do not use MCP or Claude's MCP-related permission-prompt path to bypass the authority design.

## Architecture

### Keep one Rust package and process

Do not create a Cargo workspace, daemon, background service, local HTTP server, socket broker, or plugin host.

Retain the current top-level structure. Add concrete Agentic modules only when accepted Tasks need them. Conceptually, the likely seams are:

- runtime identity/discovery;
- concrete Codex control;
- concrete Claude control;
- deterministic context projection;
- pure authority/delegation evaluation.

Do not create all modules up front. If Codex and Claude do not need a shared trait, use a closed enum/match plus concrete functions instead of a generic `AgentRuntime` framework.

### Reuse existing workspace identity

Do not create a second repository/workspace registry. Existing `workspace_id` remains the canonical local repository/worktree identity. Mutable HEAD/branch/dirty observations remain observations, not identity.

### Add workstream/task and Winds-session identity separately

The first persistence slice should add only the minimum forward-only schema required by the Spec:

**workstreams**
- stable opaque `workstream_id`;
- `workspace_id` FK;
- user-editable display/title text that is not identity;
- bounded canonical objective text;
- creation/update timestamps.

**winds_sessions**
- stable opaque `session_id`;
- `workspace_id` and `workstream_id` FKs;
- user-editable display name;
- optional origin/fork session reference;
- creation/last-used timestamps.

**runtime_session_bindings**, only when the native-binding slice actually needs persistence:
- stable binding ID;
- Winds `session_id`;
- concrete runtime kind (`CODEX` / `CLAUDE` initially);
- exact executable identity sufficient for stale-replacement detection for the claimed platform;
- observed runtime version/capability provenance;
- optional native session/thread ID;
- mapping/lifecycle state and observation time.

No display name may be a foreign key or resume identity. Runtime and model/provider identity remain separate fields/concepts.

### Keep context a deterministic projection, not a transcript database

Do not persist arbitrary model context, hidden reasoning, or full vendor history as canonical work state.

Canonical state should remain bounded and typed enough to retain:

- objective;
- human-decided constraints/decisions;
- explicit current work facts;
- selected file/path references;
- exact candidate/evidence references;
- intentionally imported agent/runtime facts with provenance.

If more persistence is needed than core workstream fields, add one bounded typed fact/reference table rather than a generic JSON memory/document store.

A context capsule is generated at use time from canonical state and selected inputs. It should include:

- schema/version;
- workspace/workstream/Winds-session IDs;
- source runtime/native binding when relevant;
- objective and selected constraints/decisions;
- selected work/file references;
- exact candidate/evidence references where applicable;
- per-fact provenance/authority;
- explicit transferred, derived/reconstructed, omitted-by-policy/budget, and unavailable categories.

Canonical serialization must define stable ordering, UTF-8 normalization policy, schema version, deterministic omission/truncation markers, no nondeterministic map iteration, and no secrets/full environment by default.

Use existing `serde`, `serde_json`, and `sha2`. No vector DB, embeddings, retrieval service, or tokenizer is required for the first capsule.

### Winds sessions are not executions

A Winds session is durable organizational/continuity identity. An `execution` is requested/observed runtime activity.

When live agent execution later lands:

- extend existing execution-kind vocabulary only for actual accepted agent activity;
- add a typed child record linking an execution to Winds session/runtime binding/role;
- do not add nullable Agentic columns to terminal/shell records;
- keep terminal/shell semantics unchanged;
- runtime output remains source-labelled execution/history evidence, not candidate verification evidence.

### Keep candidate verification canonical and separate

Do not copy or reinterpret candidate verification into an Agentic subsystem.

The minimal bridge is:

1. observe exact Git candidate identity with existing system-Git discipline;
2. bind workstream/session review state to exact candidate OID/tree;
3. reuse existing `winds verify` semantics for deterministic authority;
4. reference resulting run/report instead of duplicating it as Agent truth;
5. if candidate identity changes, mark earlier review/evidence applicability stale while retaining history.

No Agentic status can make existing verification `ELIGIBLE` merely because a model/reviewer says tests passed.

## Runtime Discovery

Discovery is a read-only fixture-first slice.

For Codex and Claude:

- search only explicit accepted executable sources for the platform;
- record exact executable path/identity and safely observable version;
- query only documented non-agentic local capability/version surfaces accepted by the task;
- never auto-install/update/authenticate/accept terms/start an agent/send a prompt;
- never duplicate credentials or persist environment secrets;
- represent authentication readiness as unknown/unavailable when safe observation cannot establish it;
- represent capability provenance as catalog-declared, vendor-declared, Winds-locally-observed, or unavailable;
- revalidate launch-significant identity immediately before start/resume.

Discovery may remain ephemeral until persistence is required to prove a native-session binding or continuity decision.

## Authority Architecture

### Separate policy evaluation from enforcement

Create a small pure deterministic evaluator before live adapters may request write/tool operations.

Inputs:

- requested operation/capability/resource;
- Planner direct authority;
- Planner delegation ceiling;
- explicit Worker grant;
- team policy if later introduced;
- human ceiling/decision;
- explicit deny/ask/allow rules;
- content/resource binding identity when approval is content-bound.

Output: decision, reason, and required human action. The evaluator does not execute anything.

### Protected policy plane

Canonical policy/trust records live under the Winds state-root/Store authority domain, not as ordinary governed repository content.

Do not overclaim protection against the same OS user principal. If a runtime can directly reach a host resource outside Winds mediation, represent actual enforcement as `AGENT_NATIVE_ENFORCED`, `BEST_EFFORT_TRIPWIRE`, `OBSERVATION_ONLY`, or `UNAVAILABLE` as appropriate.

Reserve `WINDS_ENFORCED` for operations whose relevant path Winds actually mediates in the accepted implementation.

### Human approval binding

Bind approvals to deterministic normalized content rather than unstructured chat prose.

At minimum, when relevant, bind:

- workstream/session IDs;
- Worker role/runtime request;
- workspace/worktree/root identity;
- requested operation/capability set;
- resource/path scope;
- context capsule digest;
- budget fields if used;
- delegation ceiling and Worker grant;
- candidate/base identity if already known.

Use existing `sha2` for local canonical digests when needed. Do not add PKI/signing merely for same-user local approval without a separate threat model.

Material normalized-content change invalidates approval and returns to ask/deny according to policy.

## First Live Runtime Paths

### Codex connected-session lifecycle

After fixture protocol tests pass:

1. revalidate exact `codex` executable/version;
2. spawn App Server as directly owned child with bounded stdio/stderr;
3. send `initialize`;
4. require a successful `initialize` response;
5. send `initialized` notification;
6. only then issue `thread/start`, exact `thread/resume`, `thread/fork`, or later accepted requests;
7. map structured notifications/events into source-labelled execution state;
8. route approval requests through Winds authority/human decision when that Task authorizes the operation;
9. on controlled close, reap only proven owned resources;
10. on ownership loss, mark live ownership lost while retaining durable thread identity only as a resume candidate.

Unknown/malformed/oversized messages fail or downgrade the affected capability; they never grant authority.

### Claude connected-session lifecycle

The first Claude path remains deliberately narrower:

1. revalidate exact `claude` executable/version;
2. use documented structured print/stream mode;
3. use exact native `--resume <session-id>` only for a revalidated binding;
4. never use `--continue` to satisfy canonical Winds continuation;
5. never use `--dangerously-skip-permissions`;
6. first live Planner path remains read/plan-oriented under the strongest documented exact-version agent-native restriction that can be proven;
7. label enforcement `AGENT_NATIVE_ENFORCED` or weaker unless stronger mediation is proven;
8. if write/tool mediation cannot be safely achieved without excluded MCP/unsafe bypass, do not implement the workaround;
9. persist native session ID only as a native binding;
10. new sessions built from Winds canonical context are `RECONSTRUCTED`.

Heterogeneous runtimes do not need feature parity. Capability truth is more important than a lowest-common-denominator abstraction.

## First Differentiated Walking Skeleton

After identity, discovery, context, and authority fixture slices are canonical:

1. human selects a Winds workspace/workstream;
2. Winds opens/continues a named Planner Winds session backed by Claude Code in an accepted read/plan-oriented mode;
3. Winds generates deterministic canonical context for the Planner;
4. Planner returns a bounded Worker proposal, still `AGENT_REPORTED`;
5. Winds normalizes the delegation contract and obtains explicit human approval/denial;
6. approved work uses a separate Worker Winds session and, when edit isolation is required, a separate exact Git worktree;
7. Codex App Server is the first structured Worker runtime;
8. Winds evaluates requested operations and labels actual enforcement quality truthfully;
9. Worker result remains agent-reported until Winds independently observes the Git candidate/evidence;
10. Winds observes exact candidate OID/tree and prepares independent-review context without builder persuasion/confidence as authority;
11. existing `winds verify` executes against the exact candidate;
12. candidate movement invalidates prior review/check applicability;
13. human makes the final landing decision; Winds does not merge/push automatically.

If any real-runtime prerequisite fails, fixture-only proof remains valid but the corresponding live claim is not made.

## Git and Worktree Strategy

Reuse existing system-Git and workspace identity.

Fixture-only and Planner-only stages do not create extra worktrees merely to appear isolated.

When a Worker editing Task lands:

- start from an explicit exact base/candidate parent;
- create a Winds-owned Worker worktree with existing non-destructive Git primitives or the smallest compatible extension;
- bind Worker session/delegation to exact worktree root/common-dir identity;
- observe candidate OID/tree through system Git;
- retain failed/dirty worktree for recovery;
- never call the worktree an OS sandbox;
- do not automatically merge/remove/clean after Worker completion;
- keep final landing human-decided.

## Candidate / Review Staleness

Applicability is tied to exact candidate identity, not branch/display name.

Model at least:

- `CURRENT`: applies to exact OID/tree;
- `STALE`: historical after candidate movement;
- `UNAVAILABLE`: no accepted evidence/review exists.

Do not delete historical evidence when stale.

Independent-review context should include exact candidate OID/tree/diff, applicable acceptance criteria, canonical constraints/human decisions, and accepted deterministic evidence references. Builder confidence/persuasion is not authority.

Reviewer output is review evidence; it does not itself mutate verification eligibility.

## Process Ownership / Restart Boundary

Spec 006 first program does not add a long-lived owner.

For App Server or any controlled child:

- exact retained child/process-scope object is live authority;
- persisted PID alone is never enough after restart;
- Winds restart loses live process ownership;
- durable native session/thread ID may remain a resume candidate but is not proof of a live process;
- controlled cleanup acts only on proven owned resources;
- unproven cleanup/restart becomes `OWNERSHIP_LOST` or equivalent truthful state.

Do not retrofit existing terminal ownership machinery into a fake daemon. Persistent ownership remains a later gated specification.

## Persistence Sequence

Use forward-only SQLite migrations and the existing Store. Do not create another database.

### Identity persistence first

Add only `workstreams` and `winds_sessions`. Add `runtime_session_bindings` only when the native-binding slice actually requires persistence.

Required invariants:

- FKs to existing `workspaces`;
- stable IDs independent from display names;
- fail-closed typed reads;
- deterministic rename semantics;
- no model/provider credentials or history blobs.

### Canonical fact/reference persistence only when context needs it

If workstream fields are insufficient, add one bounded typed canonical fact/reference table. Avoid a generic memory store.

### Delegation persistence only before live auditability requires it

Persist only the normalized contract/audit facts needed to recover what the human approved, its digest, decision, authority request, and enforcement-quality observation. No secret credentials.

### Agent execution child records only when live runtime execution lands

Reuse `executions` and add a typed child record linked to Winds session/runtime binding. Extend `ExecutionKind` only with actual accepted kinds.

Naming a table in this Plan does not authorize its creation; Tasks decide exact migration boundaries.

## Dependency Strategy

### Fixture-first core: no new dependencies

Identity, Store, deterministic context, authority evaluation, candidate binding, and fake/runtime discovery can use the standard library plus accepted `rusqlite`, `serde`, `serde_json`, and `sha2`.

Do not add a fuzzy matcher, async runtime, HTTP client, JSON-RPC framework, policy engine, vector DB, tokenizer, embedding model, process supervisor, or plugin crate for those slices.

### Codex

Prefer bounded stdio plus typed serde messages. If official generated protocol types later become necessary for safe compatibility, the exact implementing Task must justify the smallest footprint with provenance and Ponytail review.

### Claude

Use the local executable/structured CLI. No Anthropic Rust API SDK is needed for local Claude Code control.

### ACP

Do not land the Rust SDK until an accepted concrete ACP path needs it; then repeat exact dependency/license/MSRV/platform audit.

## Platform Strategy

Do not turn existing terminal support into an Agentic platform claim automatically.

Recommended progression:

1. fixture-only domain/Store/context/authority tests on current quality platforms;
2. fake-executable runtime discovery tests cross-platform where practical;
3. fake structured-runtime protocol lifecycle tests;
4. real runtime proof only on platforms with reproducible evidence;
5. Windows/WSL real-runtime claims only after explicit Windows/WSL evidence;
6. native-Windows authoritative `winds verify` remains separately unsupported unless changed by its own accepted task.

Each Task declares the platform/execution-domain claim it actually proves.

## Implementation Phases After Tasks Are Canonical

This Plan does not create Tasks, but freezes their dependency order.

### Phase 1 — Canonical identity, fixture-only

Stable workstream/Winds-session identity plus rename/continue/fork/new-task semantics. No Agent process.

### Phase 2 — Runtime discovery/capability provenance, fixture-only

Codex and Claude exact executable/version/capability discovery using fake executables. No prompts/model calls.

### Phase 3 — Context capsule/transfer report, fixture-only

Minimal canonical facts/references, deterministic serialization/digest, compaction/import provenance/non-overwrite tests. No Agent process.

### Phase 4 — Authority/delegation evaluator, fixture-only

Pure tests for direct authority, delegation ceiling, Worker grant, deny precedence, approval digest, enforcement-quality reporting, and protected policy plane. No Agent process.

### Phase 5 — Codex fake protocol then connected-session proof

First implement a bounded fake App Server covering the complete `initialize` response + `initialized` handshake, threads, notifications, malformed/bounded framing, and approvals. Only then authorize a real exact Codex process in a separately named Task.

### Phase 6 — Claude fake structured path then Planner proof

Prove exact resume/output semantics with fixtures first. Then a separate Task may authorize a real Claude Planner/read-plan path.

### Phase 7 — Cross-runtime handoff and one approved Planner -> Worker

One Planner, one Worker, one normalized human-approved contract, one Worker worktree when editing is required. No recursive fleet.

### Phase 8 — Exact candidate review + verification integration

Bind exact candidate, generate review-safe context, prove stale-on-movement, reuse existing deterministic verification authority, preserve explicit human landing.

### Phase 9 — P2 findability only after P1

Add session/file/context selection only if the P1 loop proves need. Prefer deterministic simple matching before a new dependency.

### Phase 10 — Acceptance

Cross-platform/negative/fault tests, docs, correctness/safety, Ponytail, fresh exact-head independent review, evidence reconciliation.

## Deterministic Test Strategy

Tasks must map the Spec success criteria to deterministic proof.

### Identity

- rename leaves stable IDs/FKs/evidence refs unchanged;
- at least 20 sessions / 5 workstreams under one workspace;
- duplicate/case/Unicode display names do not collide with identity;
- invalid FK/unknown typed vocabulary reads fail closed;
- continue/fork/new-task relationships are explicit and deterministic.

### Continuity/runtime bindings

- exact native resume succeeds only for exact revalidated mapping;
- stale/missing/ambiguous mapping -> reconstructed/fail-closed, never false `RESUMED`;
- reused process/native IDs cannot become owned/live by coincidence;
- runtime replacement after discovery is detected before use.

### Context

- byte-for-byte deterministic capsule/digest for identical canonical input/policy;
- imported facts cannot overwrite human/observed facts;
- transferred/derived/omitted/unavailable categories remain explicit;
- cross-runtime fixture exposes unavailable provider-private state;
- bounded compaction never mutates canonical state.

### Authority/enforcement

- child-over-ceiling denied;
- Planner direct authority does not expand because delegation ceiling is broader;
- repo/model/tool text cannot modify protected authority;
- explicit deny precedence;
- changed normalized approval content requires reapproval;
- bypassable direct host access cannot be labelled `WINDS_ENFORCED`.

### Codex protocol

- require `initialize` request -> successful response -> `initialized` notification before any thread/other method;
- server rejects or fixture flags pre-handshake requests;
- malformed/oversized JSONL bounded and fail-closed;
- approval request cannot self-authorize;
- App Server exit before/during handshake produces truthful failure;
- thread/native ID is distinct from Winds canonical session/task identity.

### Claude structured path

- exact resume ID accepted/rejected truthfully;
- `--continue` is never used for canonical continuation;
- `--dangerously-skip-permissions` is absent from accepted command construction;
- malformed/truncated structured output fails truthfully;
- agent-native plan/read restriction is not mislabeled as Winds/OS enforcement.

### Review/evidence

- candidate A evidence/review current;
- mutation to candidate B makes A stale while retaining history;
- reviewer context excludes builder confidence as authority;
- Agent `done/tests passed` cannot satisfy acceptance;
- existing verify/promote/recover regressions remain green when touched.

## Negative / Fault Cases

Relevant Tasks must cover:

- corrupt/unknown workstream/session/native-binding DB values;
- deleted or changed workspace identity after persistence;
- executable replaced after discovery;
- malformed/oversized version or structured runtime output;
- Codex App Server exits before initialize response or before `initialized` acknowledgement path completes;
- unknown protocol notification/request;
- approval request after bound content/candidate changed;
- Claude resume rejected/reused;
- imported history containing injected `allow`/tool syntax/conflicting facts;
- context truncation with explicit omission markers;
- Planner proposing Worker grant above ceiling;
- Worker operation outside resource scope;
- editable repo policy contradicting protected Winds policy;
- runtime claims success while Git observation disagrees;
- runtime dies while persisted state looks active;
- candidate changes while review/check is running;
- dirty/failed Worker worktree retained;
- no blind PID/native-session attachment after Winds restart.

## Repetition / Performance

Do not invent a costly live-model 100-cycle soak.

Progression:

1. deterministic Store/identity/context/authority repetition;
2. bounded fake Codex/Claude lifecycle repetition;
3. after live integrations stabilize, a bounded repetition count chosen in Tasks from measured cost to detect child leaks, stale active records, nondeterministic capsule hashes, approval reuse, or evidence-staleness bugs.

Measure separately:

- Store identity/session list/create/rename latency;
- context generation time/bytes;
- fake-runtime discovery/control overhead;
- real runtime startup/handshake overhead where reproducible.

Do not mix model inference latency into Winds local-control performance claims.

## Review Strategy

### Correctness / safety

Challenge:

- task/session/native-ID conflation;
- false live/resume state;
- stale runtime discovery;
- incomplete Codex initialization handshake;
- imported-history trust escalation;
- policy self-escalation;
- approval replay after content change;
- enforcement-quality overclaim;
- worktree-as-sandbox overclaim;
- protocol/output bounds;
- process ownership/reaping;
- credential/environment leakage;
- candidate/review/evidence staleness;
- accidental verify/promote/recover authority change.

### Ponytail

Remove/reject unless real evidence requires them:

- generic runtime/provider/plugin traits before a third runtime proves a common seam;
- unused ACP dependency;
- async runtime merely for two local child processes;
- custom JSON-RPC framework;
- second persistence/event system;
- vector/embedding/RAG subsystem;
- hosted control plane;
- daemon/IPC;
- MCP;
- recursive fleet scheduler;
- custom sandbox manager;
- broad fuzzy-search dependency before P2 need;
- custom renderer;
- speculative SQL/LLM schemas.

### Independent review

At least one reviewer other than the authoring agent must inspect each exact final acceptance-critical candidate. Reviews bound to older heads are stale.

Delegation/security review must explicitly challenge whether stated ceilings are actually enforceable, whether same-user host access bypasses Winds, whether repo/model text can influence protected policy, and whether candidate/review staleness is fail-closed.

## Plan-to-Tasks Gate

Acceptance of this Plan authorizes creation/review of a separate `tasks.md` only.

Before Tasks can authorize implementation they must:

- decompose the phases above into independently reviewable slices;
- keep fixture-only identity/discovery/context/authority work before real Agent execution;
- identify exact files/tests expected per slice without pre-authorizing unrelated files;
- name the **first Task explicitly authorized to start a real Codex process/send a prompt**;
- name the **first Task explicitly authorized to start a real Claude process/send a prompt**;
- require fresh provenance/dependency review before any new crate;
- keep ACP optional until a concrete runtime requires it;
- keep MCP/daemon/remote execution unauthorized;
- preserve exact-head CI/review/evidence gates before each canonical merge.

This Plan itself does **not** authorize:

- implementation claims before Tasks merge;
- `Cargo.toml` / `Cargo.lock` mutation;
- migrations;
- source/runtime code;
- starting Codex/Claude;
- prompts/model/provider calls;
- ACP crate landing;
- MCP;
- daemon/IPC;
- remote execution;
- automatic landing.
