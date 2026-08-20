# Implementation Plan: Agentic Terminal & Local Delegation Control Plane

## Summary

Build the smallest architecture that proves Spec 006 without turning Winds into a daemon, generic agent platform, remote control plane, or model gateway:

1. stable workspace -> workstream/task -> Winds-session identity independent of runtime-native IDs;
2. safe Codex/Claude discovery without install/auth/prompt/model side effects;
3. truthful `LIVE` / `RESUMED` / `RECONSTRUCTED` / `OWNERSHIP_LOST` continuity;
4. deterministic provenance-preserving context capsules;
5. one human-approved Planner -> Worker delegation with direct authority separate from delegation ceiling;
6. concrete official local Codex and Claude structured control paths before any generic runtime abstraction;
7. exact Git candidate binding to existing independent review and `winds verify` authority without automated landing.

The first program remains **single-process and local-first**. Reuse the current SQLite/WAL Store, system Git, execution ledger, process-ownership semantics, candidate verification records, and CLI. Do not add a persistent service, public IPC, network listener, MCP, remote execution, plugin marketplace, custom renderer, model gateway, SQL Studio, or LLM Observatory.

## Constitution Check

- runtime/model/tool prose remains `AGENT_REPORTED` until Winds observes it or a human decides it;
- no automatic winner, merge, rebase, cherry-pick, push, PR creation, force-clean, or primary-checkout mutation;
- Spec 006 is canonical at `c321d510463b207cd515ed391a47d4fb454fbe07`; this document is Plan only;
- canonical work/task/session identity survives runtime/native-session changes;
- Planner direct authority, delegation ceiling, Worker grant, human/team ceilings, and actual enforcement quality stay distinct;
- `RUNTIME != MODEL`, discovery != trust, and worktree/ACP root != sandbox;
- one Rust package/process; concrete Codex/Claude paths first; no generic JSON-RPC/plugin framework;
- each implementation slice requires exact-head deterministic gates plus independent review.

## Canonical Baseline

Planning base: `c321d510463b207cd515ed391a47d4fb454fbe07`.

Current facts:

- Constitution 1.1.0 and Spec 006 `spec.md` are canonical; Spec 003 is closed.
- Winds remains one Rust 2024 binary on Rust `1.97.1`.
- Direct dependencies are `libc`, exact `portable-pty 0.9.0`, exact `rusqlite 0.40.2` with bundled SQLite, `serde`, `serde_json`, and `sha2`.
- Existing source already provides workspace, execution, terminal, history, process-ownership, Store, system-Git, Windows/WSL, and CLI surfaces.
- `workspaces` already provides stable local workspace identity independent from mutable HEAD/dirty state.
- `executions` / `execution_events` already record lifecycle/source truth separately from candidate verification.
- candidate verification remains in `candidate_runs`, `events`, `evidence_reports`, and `promotions` with exact candidate OID/tree identity.
- `ExecutionStatus::OwnershipLost` already establishes fail-closed ownership semantics to reuse conceptually.

## External Runtime Decisions

### Codex: official App Server, not TUI scraping

Use the official local Codex App Server when a later Task authorizes live control.

Protocol facts that the implementation must preserve:

- bidirectional JSON-RPC-like messages framed as JSONL over stdio;
- required startup sequence: send `initialize`, require a successful response, send `initialized`, then and only then use `thread/start`, `thread/resume`, or other methods;
- durable threads support create/resume/fork/archive behavior;
- turns produce structured progress/item notifications;
- the server can issue approval requests and wait for allow/deny;
- a local client may launch App Server as a child with retained bidirectional stdio.

Plan decisions:

- revalidate exact discovered `codex` executable/version immediately before use;
- launch App Server as a directly owned child for the current Winds lifetime;
- implement the complete `initialize` -> successful response -> `initialized` handshake before any other request;
- use only minimal typed methods/events needed by accepted Tasks;
- bound frames, stdout/stderr, diagnostics, and malformed-message handling;
- use existing `serde`/`serde_json` unless implementation evidence proves a protocol dependency is necessary;
- persist Codex thread identity only as a runtime-native binding, never canonical Winds task/session identity;
- treat approval requests as mediation events, not OS-containment proof;
- after Winds/process loss, live ownership is lost even if a durable thread ID remains a future resume candidate.

Primary current source: `https://developers.openai.com/codex/app-server`.

### Claude Code: official local structured CLI first

Use documented local structured CLI behavior before inventing another protocol layer:

- `claude -p` / `--print`;
- `--output-format json|stream-json`;
- `--input-format stream-json` in print mode;
- exact `--resume <session-id>`;
- recency-based `--continue`;
- `--allowedTools`, `--disallowedTools`, `--permission-mode`;
- `--dangerously-skip-permissions` exists but is prohibited in the accepted Winds path.

Plan decisions:

- revalidate exact discovered `claude` executable/version before use;
- use structured print/stream mode, not TUI scraping;
- use exact `--resume <session-id>` only for a proven native binding;
- never use `--continue` as canonical Winds continuation;
- never use `--dangerously-skip-permissions`;
- keep the first live Claude role Planner/read-plan oriented unless a later exact Task proves stronger non-interactive mediation consistent with Winds authority semantics;
- label vendor restriction `AGENT_NATIVE_ENFORCED` or weaker unless stronger mediation is independently proven;
- if safe write/tool mediation would require excluded MCP or unsafe bypass, do not work around it—keep Claude Planner-only;
- persist native session ID only as a runtime-native binding;
- a new native session built from canonical Winds state is `RECONSTRUCTED`, never fake `RESUMED`.

Primary current source: `https://docs.anthropic.com/en/docs/claude-code/cli-usage`.

### ACP stays pinned but is not yet a dependency

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

This is a compatibility/provenance pin, not permission to add an unused crate. Codex and Claude already have official local structured surfaces, so the initial program SHOULD NOT add `agent-client-protocol` until a concrete accepted path requires ACP. That Task must repeat the exact Cargo graph/checksum/license/MSRV/platform audit against then-canonical main.

MCP remains separately gated and out of the first program.

## Architecture

### Keep one package/process

Do not create a Cargo workspace, daemon, background service, HTTP server, socket broker, plugin host, generic runtime registry, or arbitrary executable-template system.

Add concrete Agentic modules only as accepted Tasks need them. Likely proven seams are runtime discovery, concrete Codex control, concrete Claude control, deterministic context projection, and pure authority evaluation. If two runtimes do not need a shared trait, use a closed enum/match and concrete functions.

### Reuse existing workspace identity

Do not create a second repository/workspace registry. Existing `workspace_id` remains canonical local repository/worktree identity. Mutable Git state remains observation, not identity.

### Add canonical workstream/task and Winds-session identity

The first persistence slice should add the minimum forward-only schema:

**workstreams**
- stable opaque `workstream_id`;
- `workspace_id` FK -> existing `workspaces`;
- user-editable title/display text that is not identity;
- bounded canonical objective text;
- creation/update timestamps.

**winds_sessions**
- stable opaque `session_id`;
- `workstream_id` FK -> `workstreams`;
- user-editable display name;
- optional origin/fork session reference;
- creation/last-used timestamps.

`winds_sessions` SHOULD NOT duplicate `workspace_id` in the first schema. Its workspace is derived unambiguously through `workstream_id -> workstreams.workspace_id`. This removes the possibility of pairing a session with workspace A and a workstream owned by workspace B. If a future measured need justifies denormalizing workspace identity, that later migration must enforce the ownership relation transactionally/compositely rather than using two independent foreign keys.

**runtime_session_bindings**, only when a native-binding slice actually needs persistence:
- stable binding ID;
- `session_id` FK -> `winds_sessions`;
- concrete runtime kind (`CODEX` / `CLAUDE` initially);
- exact executable identity sufficient for stale-replacement detection on the claimed platform;
- observed runtime version/capability provenance;
- optional native thread/session ID;
- mapping/lifecycle state and observation time.

Consequences/invariants:

- a session belongs to exactly one canonical workstream and therefore exactly one workspace;
- cross-workspace session/workstream association is structurally impossible in the first schema;
- display names are never foreign keys or resume identities;
- runtime and model/provider identity stay separate;
- runtime-native session identity never becomes canonical Winds task/session identity.

### Context is a deterministic projection, not transcript memory

Do not persist arbitrary model context, hidden reasoning, or full vendor history as canonical state.

Canonical state stays bounded and typed enough for objective, human-decided constraints/decisions, explicit current work facts, selected file/path references, exact candidate/evidence references, and intentionally imported runtime/agent facts with provenance.

If core workstream fields are insufficient, add one bounded typed fact/reference table rather than a generic JSON memory/document store.

A context capsule is generated at use time and includes:

- schema/version;
- workspace/workstream/Winds-session IDs;
- source runtime/native binding where relevant;
- objective + selected constraints/decisions;
- selected work/file references;
- exact candidate/evidence references;
- per-fact provenance/authority;
- transferred, derived/reconstructed, omitted-by-policy/budget, and unavailable categories.

Serialization must define stable ordering, UTF-8 normalization policy, deterministic omission/truncation markers, no nondeterministic map iteration, and no secrets/full environment by default. Use existing `serde`, `serde_json`, and `sha2`; no vector DB, embeddings, retrieval service, or tokenizer is required.

### Winds sessions are not executions

A Winds session is durable organizational/continuity identity. An `execution` is requested/observed runtime activity.

When live agent execution lands:

- extend existing execution-kind vocabulary only for accepted actual Agent activity;
- add a typed child record linking an execution to Winds session/runtime binding/role;
- do not add nullable Agentic columns to terminal/shell rows;
- preserve terminal/shell semantics;
- runtime output remains source-labelled execution/history evidence, not candidate verification evidence.

### Candidate verification stays canonical and separate

Minimal bridge:

1. observe exact Git candidate identity with existing system-Git discipline;
2. bind workstream/session review state to exact candidate OID/tree;
3. reuse existing `winds verify` semantics;
4. reference resulting run/report rather than copying it as Agent truth;
5. candidate movement marks earlier review/evidence applicability stale while preserving history.

No Agentic status can make verification eligible merely because a model/reviewer says tests passed.

## Runtime Discovery

Discovery is read-only and fixture-first.

For Codex and Claude:

- search only explicit accepted executable sources for the platform;
- record exact executable identity/path plus safely observable version;
- query only documented non-agentic version/capability surfaces accepted by the Task;
- never auto-install/update/authenticate/accept terms/start an Agent/send a prompt;
- never duplicate credentials or persist environment secrets;
- represent auth readiness as unknown/unavailable when safe observation cannot establish it;
- retain capability provenance as catalog-declared, vendor-declared, Winds-locally-observed, or unavailable;
- revalidate launch-significant identity immediately before start/resume.

Discovery may remain ephemeral until persistence is needed to prove a native binding or continuity decision.

## Authority Architecture

### Pure policy evaluation before enforcement

Create a deterministic evaluator that consumes:

- requested operation/capability/resource;
- Planner direct authority;
- Planner delegation ceiling;
- explicit Worker grant;
- team policy if later introduced;
- human ceiling/decision;
- deny/ask/allow rules;
- content/resource binding identity where approval is content-bound.

It returns decision, reason, and required human action. It does not execute the operation.

### Protected policy plane

Canonical policy/trust state lives under Winds state-root/Store authority, not ordinary governed repository content.

Do not overclaim protection against the same OS user principal. If a runtime can directly reach a host resource outside Winds mediation, actual enforcement must be labelled `AGENT_NATIVE_ENFORCED`, `BEST_EFFORT_TRIPWIRE`, `OBSERVATION_ONLY`, or `UNAVAILABLE` as appropriate.

Reserve `WINDS_ENFORCED` for paths actually mediated by Winds.

### Human approval binding

Bind approval to deterministic normalized content, including as applicable:

- workstream/session IDs;
- Worker role/runtime request;
- workspace/worktree/root identity;
- requested capability set and resource/path scope;
- context capsule digest;
- budget fields if used;
- delegation ceiling + Worker grant;
- candidate/base identity if known.

Use existing `sha2` for local digests. Do not add PKI/signing merely for same-user approval without a separate threat model. Material normalized-content change invalidates approval and returns to ask/deny policy.

## First Live Runtime Paths

### Codex lifecycle

After fixture protocol tests pass:

1. revalidate exact `codex` executable/version;
2. spawn App Server as directly owned child with bounded stdio/stderr;
3. send `initialize`;
4. require a successful `initialize` response;
5. send `initialized`;
6. only then issue `thread/start`, exact `thread/resume`, `thread/fork`, or other accepted methods;
7. map structured notifications into source-labelled execution state;
8. route approval requests through Winds authority/human decision when the Task authorizes the operation;
9. on controlled close, reap only proven owned resources;
10. on ownership loss, retain durable thread identity only as a resume candidate while live ownership becomes lost.

Unknown/malformed/oversized messages fail or downgrade the affected capability; they never grant authority.

### Claude lifecycle

1. revalidate exact `claude` executable/version;
2. use documented structured print/stream mode;
3. use exact `--resume <session-id>` only for a revalidated binding;
4. never use `--continue` for canonical continuation;
5. never use `--dangerously-skip-permissions`;
6. first live Planner remains read/plan-oriented under the strongest documented exact-version agent-native restriction that can be proven;
7. label enforcement `AGENT_NATIVE_ENFORCED` or weaker unless stronger mediation is proven;
8. if write/tool mediation cannot be safely achieved without excluded MCP/unsafe bypass, do not implement a workaround;
9. persist native session ID only as a native binding;
10. new sessions from Winds canonical context are `RECONSTRUCTED`.

Heterogeneous runtimes do not need feature parity; capability truth wins over lowest-common-denominator abstraction.

## First Differentiated Walking Skeleton

After identity, discovery, context, and authority fixture slices are canonical:

1. human selects a Winds workspace/workstream;
2. Winds opens/continues a named Claude-backed Planner Winds session in an accepted read/plan-oriented mode;
3. Winds generates deterministic canonical context;
4. Planner returns a bounded Worker proposal, still `AGENT_REPORTED`;
5. Winds normalizes the contract and obtains explicit human approval/denial;
6. approved work uses a separate Worker Winds session and, when edit isolation is required, a separate exact Git worktree;
7. Codex App Server is the first structured Worker runtime;
8. Winds evaluates requested operations and labels actual enforcement truthfully;
9. Worker result remains agent-reported until Git/evidence is independently observed;
10. Winds observes exact candidate OID/tree and prepares independent-review context without builder persuasion as authority;
11. existing `winds verify` runs on the exact candidate;
12. candidate movement invalidates earlier review/check applicability;
13. human makes the final landing decision; Winds does not merge/push automatically.

If real-runtime prerequisites fail, fixture proof remains valid but the live claim is not made.

## Git / Worktree Strategy

Reuse existing system Git and workspace identity.

Fixture/Planner stages do not create extra worktrees merely to appear isolated.

When a Worker-editing Task lands:

- start from explicit exact base/candidate parent;
- create a Winds-owned Worker worktree using existing non-destructive Git primitives or the smallest compatible extension;
- bind Worker session/delegation to exact worktree root/common-dir identity;
- observe candidate OID/tree through system Git;
- retain failed/dirty worktree for recovery;
- never call the worktree an OS sandbox;
- do not automatically merge/remove/clean after Worker completion;
- keep final landing human-decided.

## Candidate / Review Staleness

Applicability is tied to exact candidate identity:

- `CURRENT`: applies to exact OID/tree;
- `STALE`: historical after movement;
- `UNAVAILABLE`: no accepted evidence/review exists.

Do not delete stale history.

Independent-review context includes exact candidate OID/tree/diff, applicable acceptance criteria, canonical constraints/human decisions, and accepted deterministic evidence references. Builder confidence/persuasion is not authority.

Reviewer output is review evidence; it does not directly mutate verification eligibility.

## Process Ownership / Restart Boundary

No persistent owner is added.

For App Server or another controlled child:

- exact retained child/process-scope object is live authority;
- persisted PID alone is insufficient after restart;
- Winds restart loses live process ownership;
- durable native thread/session ID may remain a resume candidate only;
- controlled cleanup acts only on proven owned resources;
- unproven cleanup/restart becomes `OWNERSHIP_LOST` or equivalent truthful state.

Do not retrofit terminal ownership machinery into a fake daemon.

## Persistence Sequence

Use forward-only migrations and the existing Store only.

### Identity first

Add `workstreams` + `winds_sessions` with the structural chain:

```text
workspaces(workspace_id)
  -> workstreams(workstream_id, workspace_id)
      -> winds_sessions(session_id, workstream_id)
```

This chain is the canonical workspace/workstream/session ownership invariant. No independent duplicate workspace FK is required in `winds_sessions`.

### Runtime bindings only when needed

Add `runtime_session_bindings(session_id, runtime_kind, executable_identity, observed_version/provenance, optional native_id, state, observed_time)` only when native persistence becomes necessary.

### Canonical facts only when context needs them

If core workstream fields are insufficient, add one bounded typed fact/reference table. Avoid generic memory storage.

### Delegation audit only before live delegation

Persist only normalized contract/audit facts needed to recover what the human approved, its digest, decision, authority request, and observed enforcement quality. No secret credentials.

### Agent execution child rows only with live runtime execution

Reuse `executions`; add a typed child linked to Winds session/runtime binding. Extend `ExecutionKind` only for accepted actual Agent execution kinds.

A conceptual table name in this Plan does not authorize creating it; Tasks define exact migration boundaries.

## Dependency Strategy

Fixture-first identity, Store, context, authority, candidate binding, and fake/runtime discovery require no new dependencies beyond standard library + existing `rusqlite`, `serde`, `serde_json`, `sha2`.

Do not add a fuzzy matcher, async runtime, HTTP client, JSON-RPC framework, policy engine, vector DB, tokenizer, embedding model, process supervisor, or plugin crate for those slices.

Codex: prefer bounded stdio + typed serde messages. Any generated protocol/dependency later requires exact Task-level provenance/YAGNI review.

Claude: local executable/structured CLI; no Anthropic Rust API SDK needed.

ACP: do not land SDK until an accepted concrete ACP path needs it; then repeat dependency/license/MSRV/platform audit.

## Platform Strategy

Progress from fixture truth to live claims:

1. Store/identity/context/authority fixtures on current quality platforms;
2. fake-executable discovery tests cross-platform where practical;
3. fake structured-runtime lifecycle tests;
4. real runtime proof only where reproducible evidence exists;
5. Windows/WSL real-runtime claims only after explicit evidence;
6. native-Windows authoritative `winds verify` stays separately unsupported unless changed by its own accepted Task.

Each Task declares the exact platform/execution-domain claim it proves.

## Implementation Phases After Tasks Are Canonical

### Phase 1 — Identity, fixture-only
Stable workstream/Winds-session identity and rename/continue/fork/new-task semantics. No Agent process.

### Phase 2 — Runtime discovery, fixture-only
Codex/Claude executable/version/capability provenance with fake executables. No prompt/model call.

### Phase 3 — Context, fixture-only
Minimal canonical facts/references plus deterministic capsule/digest/transfer report and import/compaction non-overwrite tests.

### Phase 4 — Authority, fixture-only
Pure direct-authority/delegation-ceiling/Worker-grant/deny-precedence/approval-digest/enforcement-quality tests.

### Phase 5 — Codex fake protocol then real connected proof
Fake App Server must cover the complete `initialize` response + `initialized` handshake, threads, notifications, bounds, and approvals before a separately named Task may launch real Codex/send a prompt.

### Phase 6 — Claude fake structured path then Planner proof
Fixture exact-resume/output/permission construction first. A separately named Task then may launch real Claude/send a prompt.

### Phase 7 — Cross-runtime handoff + one Planner -> Worker
One Planner, one Worker, one human-approved normalized contract, one Worker worktree if editing is required. No recursive fleet.

### Phase 8 — Exact candidate review + verification
Bind candidate, produce review-safe context, prove stale-on-movement, reuse deterministic verification authority, preserve human landing.

### Phase 9 — P2 findability
Only after P1 loop. Prefer simple deterministic matching before a new dependency.

### Phase 10 — Acceptance
Cross-platform/negative/fault tests, docs, correctness/safety, Ponytail, fresh exact-head independent review, final evidence reconciliation.

## Deterministic Test Strategy

### Identity
- rename preserves stable IDs/FKs/evidence refs;
- at least 20 sessions / 5 workstreams under one workspace;
- duplicate/case/Unicode display names do not collide with identity;
- a `winds_session` cannot be associated with a workstream from a second workspace because its only workspace path is through that workstream;
- invalid FK/unknown typed values fail closed;
- continue/fork/new-task relations are explicit.

### Continuity/runtime bindings
- exact native resume only for exact revalidated mapping;
- stale/missing/ambiguous mapping -> reconstructed/fail-closed, never false `RESUMED`;
- reused process/native IDs cannot become owned/live by coincidence;
- runtime replacement after discovery is detected.

### Context
- byte-for-byte deterministic capsule/digest for identical canonical input/policy;
- imported facts cannot overwrite human/observed facts;
- transferred/derived/omitted/unavailable categories explicit;
- cross-runtime fixture exposes unavailable private state;
- compaction never mutates canonical state.

### Authority/enforcement
- child-over-ceiling denied;
- Planner direct authority unaffected by broader delegation ceiling;
- repo/model/tool text cannot modify protected authority;
- explicit deny precedence;
- changed normalized approval content requires reapproval;
- bypassable direct host access cannot be labelled `WINDS_ENFORCED`.

### Codex protocol
- require `initialize` request -> successful response -> `initialized` notification before thread/other methods;
- pre-handshake methods rejected by fixture/client state;
- malformed/oversized JSONL bounded/fail-closed;
- approval request cannot self-authorize;
- App Server exit during handshake yields truthful failure;
- thread/native ID remains distinct from canonical session/task.

### Claude structured path
- exact resume ID accepted/rejected truthfully;
- `--continue` never used for canonical continuation;
- `--dangerously-skip-permissions` absent from accepted command construction;
- malformed/truncated output fails truthfully;
- agent-native restriction not mislabeled Winds/OS enforcement.

### Review/evidence
- candidate A evidence/review current;
- candidate B makes A stale while retaining history;
- reviewer context excludes builder confidence as authority;
- Agent `done/tests passed` cannot satisfy acceptance;
- existing verify/promote/recover regressions remain green when touched.

## Negative / Fault Cases

Relevant Tasks cover:

- corrupt/unknown workstream/session/native-binding DB values;
- cross-workspace workstream/session mismatch attempts;
- deleted/changed workspace identity after persistence;
- executable replaced after discovery;
- malformed/oversized version or runtime output;
- Codex App Server exit before initialize response or before `initialized` sequence completes;
- unknown protocol message;
- approval request after bound content/candidate changed;
- Claude resume rejected/reused;
- imported history with injected `allow`, tool syntax, or conflicting facts;
- context truncation with explicit omission markers;
- Planner proposal above delegation ceiling;
- Worker operation outside resource scope;
- editable repo policy contradicting protected Winds policy;
- runtime claims success while Git observation disagrees;
- runtime dies while persisted state looks active;
- candidate changes during review/check;
- dirty/failed Worker worktree retained;
- no blind PID/native-session attachment after restart.

## Repetition / Performance

Do not invent a costly live-model 100-cycle soak.

Use deterministic Store/identity/context/authority repetitions first, then bounded fake Codex/Claude lifecycle repetition. After live integrations stabilize, Tasks choose a bounded live repetition count from measured cost to detect child leaks, stale active records, nondeterministic capsule hashes, approval reuse, or evidence-staleness bugs.

Measure Store identity/session operations, context generation time/bytes, fake-runtime control overhead, and real runtime startup/handshake overhead where reproducible. Do not mix model inference latency into Winds local-control performance claims.

## Review Strategy

Correctness/safety review challenges identity conflation, cross-workspace ownership, false resume/live state, stale discovery, incomplete Codex handshake, imported-history escalation, policy self-escalation, approval replay, enforcement overclaim, worktree-as-sandbox overclaim, protocol bounds, process ownership, credential leakage, candidate/evidence staleness, and accidental verification-authority change.

Ponytail removes/rejects generic runtime/plugin traits before a third proven runtime, unused ACP, async runtime for two local children, custom JSON-RPC framework, second persistence/event systems, vector/RAG, hosted control plane, daemon/IPC, MCP, recursive fleet scheduler, custom sandbox manager, premature fuzzy-search dependency, custom renderer, and speculative SQL/LLM schemas.

At least one reviewer other than the authoring agent inspects each exact final acceptance-critical candidate; older-head reviews are stale.

## Plan-to-Tasks Gate

Accepting this Plan authorizes only creation/review of a separate `tasks.md`.

Tasks must:

- decompose phases into independently reviewable slices;
- keep identity/discovery/context/authority fixture-only work before real Agent execution;
- identify exact files/tests per slice without pre-authorizing unrelated files;
- name the **first Task explicitly authorized to start real Codex/send a prompt**;
- name the **first Task explicitly authorized to start real Claude/send a prompt**;
- require fresh provenance/dependency review before any new crate;
- keep ACP optional until a concrete runtime requires it;
- keep MCP/daemon/remote execution unauthorized;
- require exact-head CI/review/evidence gates before each canonical merge.

This Plan itself does **not** authorize implementation, migrations, dependency/lockfile changes, source/runtime code, starting Codex/Claude, prompts/model/provider calls, ACP crate landing, MCP, daemon/IPC, remote execution, or automatic landing.
