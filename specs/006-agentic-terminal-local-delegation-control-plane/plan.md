# Implementation Plan: Agentic Terminal & Local Delegation Control Plane

## Summary

Build the smallest architecture that can prove Spec 006's differentiated loop without turning Winds into a daemon, generic agent platform, remote control plane, or model gateway:

1. preserve stable local workspace -> workstream/task -> Winds-session identity independent of runtime-native IDs;
2. discover Codex and Claude Code safely without install/auth/prompt/model side effects;
3. express truthful `LIVE` / `RESUMED` / `RECONSTRUCTED` / `OWNERSHIP_LOST` continuity;
4. generate deterministic provenance-preserving context capsules from canonical Winds state;
5. evaluate one human-approved Planner -> Worker delegation contract with direct authority separate from delegation ceiling;
6. connect first to the two real runtimes through their narrow official local structured surfaces rather than inventing a generic plugin system;
7. bind the resulting exact Git candidate to existing independent review and `winds verify` authority without changing verification semantics or automating landing.

The first implementation program remains **single-process and local-first**. It reuses the existing SQLite/WAL Store, system-Git discipline, execution ledger, ownership-loss semantics, and CLI process. It does not require a persistent service, public IPC, network listener, MCP, remote execution, custom renderer, model routing, SQL Studio, or LLM Observatory.

## Constitution Check

- **Evidence over claims**: runtime/model/tool prose remains `AGENT_REPORTED` unless Winds independently observes a fact or a human decides it. Existing candidate verification authority remains separate: REQUIRED.
- **Non-destructive Git safety**: no automatic winner, merge, rebase, cherry-pick, push, PR creation, force-clean, or primary-checkout mutation is introduced: REQUIRED.
- **Spec -> Plan -> Tasks before implementation**: Spec 006 is canonical at `c321d510463b207cd515ed391a47d4fb454fbe07`; this document is Plan only: PASS.
- **Canonical continuity**: Winds workspace/workstream/session identity must survive runtime/native-session changes and must not collapse native resume into task continuity: REQUIRED.
- **Authority ceilings**: Planner direct authority, delegation ceiling, Worker grant, team/human ceiling, and enforcement quality remain distinct: REQUIRED.
- **Runtime/capability truth**: runtime != model; declared != locally observed; discovery != trust; worktree/ACP root != sandbox: REQUIRED.
- **Ponytail/YAGNI**: keep one Rust package/process, reuse Store/Git/execution primitives, use two concrete runtime paths, no plugin framework, no generic JSON-RPC framework, no ACP dependency until an actual runtime path requires ACP: REQUIRED.
- **Independent review before acceptance**: each implementation slice must be exact-head reviewed after deterministic gates: REQUIRED.

## Canonical Baseline

Planning base:

`c321d510463b207cd515ed391a47d4fb454fbe07`

Canonical facts:

- Constitution 1.1.0 is canonical.
- Spec 003 is closed canonical.
- Spec 006 `spec.md` is closed canonical.
- Current crate is one Rust 2024 binary, `winds-control`, pinned to Rust `1.97.1`.
- Current direct dependencies are only `libc`, exact `portable-pty 0.9.0`, exact `rusqlite 0.40.2` with bundled SQLite, `serde`, `serde_json`, and `sha2`.
- Existing source already has concrete workspace, execution, terminal, history, process-ownership, Store, system-Git, Windows/WSL, and CLI surfaces.
- Existing `workspaces` persistence gives a stable local workspace identity keyed independently from mutable Git HEAD/dirty observations.
- Existing `executions`/`execution_events`/typed terminal and shell-command records already separate lifecycle/source truth from candidate verification tables.
- Existing candidate verification remains in `candidate_runs`, `events`, `evidence_reports`, and `promotions` with exact candidate OID/tree identity.
- `ExecutionStatus::OwnershipLost` and fail-closed process-ownership behavior already exist and must be reused conceptually rather than reinvented for agent sessions.

## Current External Control-Surface Decisions

These decisions were rechecked against official primary sources during planning because the live interfaces moved after early research.

### Codex: use App Server, not terminal scraping

OpenAI's current Codex App Server is the preferred first concrete connected-runtime surface.

Relevant current facts from OpenAI's official App Server engineering documentation:

- Codex CLI, IDE, app, and other clients share the same Codex harness.
- App Server exposes a client-facing bidirectional JSON-RPC-like protocol framed as JSONL over stdio.
- The client sends `initialize` before other methods.
- durable Codex threads can be created, resumed, forked, and archived;
- one user turn produces structured progress/item notifications;
- the server can issue approval requests and pause until the client answers allow/deny;
- local clients launch a child App Server and keep a bidirectional stdio channel open.

Plan decision:

- later implementation should launch the exact locally discovered `codex` executable in its official App Server mode as a directly owned child for the current Winds lifetime;
- use only the minimal typed methods/events needed by accepted tasks;
- do not scrape TUI escape sequences or terminal text as the primary control plane;
- do not add a generic JSON-RPC dependency merely to handle this vendor-specific JSONL protocol; `serde`/`serde_json` already exist and are sufficient for a deliberately narrow typed envelope unless implementation evidence proves otherwise;
- App Server process ownership ends with the current Winds process under this specification; cross-Winds-restart live ownership is not claimed;
- durable Codex `thread` identity may be persisted as a runtime-native binding, but never becomes the canonical Winds task/session identity;
- approval requests are useful mediation events, not automatic proof of OS containment.

Primary source:
`https://openai.com/index/unlocking-the-codex-harness/`

### Claude Code: use the official local structured CLI surface first

Anthropic's current Claude Code CLI supports a narrow local structured automation path:

- `claude -p` / `--print` for non-interactive execution;
- `--output-format json` or `stream-json`;
- `--input-format stream-json` for structured input in print mode;
- `--resume <session-id>` for a specific native session and `--continue` for recent-directory continuation;
- explicit `--allowedTools`, `--disallowedTools`, and `--permission-mode` controls;
- `--dangerously-skip-permissions` exists and MUST NOT be used by Winds' accepted path.

Plan decision:

- use an exact discovered local Claude Code executable and structured print/stream mode for the first integration;
- persist a proven Claude native session ID only as a runtime-native binding;
- never use `--continue` as canonical continuity because "most recent" is ambiguous by Winds rules; Winds continuation may use only an exact `--resume <session-id>` mapping after revalidation, otherwise reconstruct a new native session;
- first Claude live role should remain **Planner/read-oriented** (for example using the vendor's plan-oriented permission mode where applicable) until a later exact task proves a non-interactive write/tool mediation path consistent with Winds authority semantics;
- do not use `--dangerously-skip-permissions`;
- do not require Anthropic's cloud Managed Agents API, remote sandboxes, vaults, or hosted sessions for the local-first Spec 006 path;
- do not add a Rust Anthropic SDK merely to control local Claude Code.

Primary source:
`https://docs.anthropic.com/en/docs/claude-code/cli-usage`

### ACP: keep the canonical pin, defer the crate until a concrete runtime needs it

Canonical governance already pins:

```text
ACP_WIRE_PROTOCOL=1
ACP_SCHEMA=schema-v1.20.0
ACP_SCHEMA_COMMIT=5e89c71497fe07dd4ae633c181a17224f4a8956d
ACP_RUST_SDK=2.0.0
ACP_RUST_SDK_COMMIT=ce023279824149008659dd8f4b8b70266a7e8210
UNSTABLE_PROTOCOL_V2=DISABLED
UNSTABLE_MCP_OVER_ACP=DISABLED
```

The pin is a compatibility/provenance gate, not a mandate to create an unused dependency.

Because the first two real targets have official vendor-native structured local surfaces, the initial implementation program SHOULD NOT land `agent-client-protocol` until a concrete accepted task selects an ACP-speaking runtime/path. If that happens, the task must perform the previously required exact Cargo graph/checksum/license/MSRV/platform audit before adding the crate.

This is intentionally stricter YAGNI than "pin then immediately depend."

### MCP remains out

Neither the first Codex App Server integration nor the first Claude Code structured path requires MCP to prove canonical continuity and one bounded delegation. MCP remains separately gated. Do not use Claude Code's `--permission-prompt-tool` MCP path to work around authority design in the first slice.

## Architecture

### Keep one Rust package and one process

Do not create a Cargo workspace, daemon, background service, local HTTP server, socket broker, or plugin host.

Retain the existing top-level structure and add only concrete agentic modules as tasks require them. The intended shape is conceptually:

```text
src/
  main.rs                  existing CLI dispatch
  cli_workspace.rs         existing workspace/execution CLI
  domain.rs                existing shared evidence/execution types
  store.rs                 existing single SQLite/WAL Store
  git.rs + git/*            existing Git/workspace/terminal primitives
  execution.rs             existing owned terminal execution lifecycle
  agentic.rs               new narrow Spec 006 coordination module when implementation begins
  agentic/
    runtime.rs             concrete runtime identity/discovery + tiny shared envelopes if justified
    codex.rs               Codex App Server path only
    claude.rs              Claude Code structured CLI path only
    context.rs             deterministic canonical context projection
    authority.rs           pure policy/delegation evaluation
```

This is a planning shape, not permission to create every file at once. A task should create a submodule only when that slice has concrete behavior/tests. If two concrete runtime modules do not need a shared trait, use an enum/match and concrete functions instead of introducing a generic `AgentRuntime` plugin abstraction.

### Reuse existing workspace identity

Do not invent a second repository/workspace registry.

The existing `workspaces` row remains the canonical local repository/worktree identity for Spec 006. Agentic state references `workspace_id`.

Mutable Git state is still observed via existing system-Git logic and must not become part of workspace identity.

### Add canonical workstream/task and Winds-session identity separately

The first persistence slice should add the minimum forward-only schema needed to represent the Spec distinction:

- one canonical **workstream/task** record under a workspace;
- many stable **Winds sessions** under a workstream;
- optional exact **runtime-native session bindings** under a Winds session.

Conceptual fields:

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

**runtime_session_bindings**
- stable binding id;
- Winds `session_id`;
- concrete runtime kind (`CODEX` / `CLAUDE` in first implementation; fail-closed on unknown values);
- exact executable identity sufficient for stale-replacement detection under the implementing platform task;
- observed runtime version/capability provenance;
- optional native session/thread ID;
- lifecycle/mapping state and observation time.

Do not store model/provider identity in the same column as runtime identity. When model identity is observed later, keep it separately source-labelled.

No display name may be used as a foreign key or resume identity.

### Keep context as a deterministic projection, not a second transcript database

Do not persist arbitrary model context, hidden reasoning, or full vendor history as the canonical task state.

Canonical work state should be small and typed enough to preserve:

- objective;
- human-decided constraints/decisions;
- explicit current work-state facts;
- exact file/path references where selected;
- exact candidate/evidence references;
- imported/agent facts with provenance when the user intentionally carries them forward.

If a task needs persistence beyond the core workstream fields, use one bounded append-only canonical fact/reference table rather than a generic JSON memory store. Each record must carry a narrow fact kind plus source/authority classification.

A **context capsule** is generated at use time as a deterministic serialization/projection of canonical records and selected inputs. It is not the canonical store itself.

The capsule should contain:

- schema/version;
- workspace/workstream/Winds-session stable IDs;
- source runtime/native session mapping when relevant;
- objective and selected constraints/decisions;
- selected file/path/work-state references;
- exact candidate/evidence references when applicable;
- per-fact provenance/authority;
- explicit categories for transferred, derived/reconstructed, omitted-by-policy/budget, and unavailable state.

A compact model-facing view may be derived from the capsule, but compaction cannot mutate canonical records.

### Reuse the execution ledger for runtime work; do not make canonical sessions executions

A Winds session is a durable organizational/continuity identity. An `execution` is an observed/requested runtime activity. They are not the same object.

When live agent execution later lands:

- extend the existing execution-kind typed vocabulary only as needed for actual agent activity;
- add a typed child record linking an execution to a Winds session/runtime binding and role;
- do not add nullable agent columns to every existing terminal/shell record;
- do not change existing terminal/shell execution semantics;
- agent turn/output remains execution/history evidence with source labels, not candidate verification evidence.

This reuses the ledger's lifecycle/source/time model and avoids a parallel generic event bus.

### Keep candidate verification canonical and separate

Do not copy or reinterpret `candidate_runs`, `evidence_reports`, or promotion logic into an agentic subsystem.

Agentic work may record that a Winds workstream/session is currently associated with an exact candidate OID/tree/worktree observation. Acceptance still proceeds through existing verification primitives.

The bridge should be minimal:

1. observe exact Git candidate identity through existing system-Git discipline;
2. bind session/workstream review state to that exact OID/tree;
3. invoke/use existing `winds verify` semantics for deterministic authority;
4. record references to the resulting run/report, not duplicate its report as agent truth;
5. if candidate identity changes, mark prior review/verification applicability stale while retaining history.

No agentic status may write an existing verification row into `ELIGIBLE` merely because a model/reviewer says tests passed.

### Runtime discovery is read-only and revalidated at use time

Discovery should be its own bounded pure/read-only slice before agent execution.

For each supported runtime:

- search only explicit accepted executable sources for the platform;
- record the exact executable path/identity and safely observable version;
- query only documented non-agentic local capability/version surfaces accepted by that task;
- never auto-install/update/authenticate/accept terms/start an agent/send a prompt;
- never duplicate credentials or persist environment secrets;
- represent auth readiness as `UNKNOWN/UNAVAILABLE` when safe observation cannot establish it;
- represent each capability with provenance: catalog-declared, vendor-declared, locally observed, or unavailable;
- revalidate launch-significant executable identity/version immediately before starting/resuming a runtime.

Discovery results can initially be ephemeral. Persist only what is required to prove a runtime/native-session binding or audit a continuity decision.

### Concrete runtime dispatch before generic interfaces

Use a closed first-slice enum such as conceptual `RuntimeKind::{Codex, Claude}` with explicit match-based dispatch.

Do not build:

- runtime registries;
- dynamically loaded plugins;
- generic provider configuration schemas;
- arbitrary executable templates;
- a "capabilities JSON" plugin contract;
- marketplace/install machinery.

If a third real runtime later creates repeated mechanics, refactor only the common proven seam after evidence exists.

### Codex connected-session lifecycle

Later Codex tasks should follow this bounded lifecycle:

1. revalidate exact discovered `codex` executable/version;
2. spawn official App Server as a directly owned child with bounded stdio/stderr handling;
3. complete the required initialization handshake;
4. query/use only the narrow protocol capabilities needed by the task;
5. create an exact native thread or resume/fork an exact known thread after binding validation;
6. map thread/turn/item notifications into source-labelled execution state;
7. route server approval requests through the Winds authority evaluator/human decision path when the accepted task requires it;
8. on controlled close, reap only the directly owned child/process scope according to existing ownership discipline;
9. on Winds/process loss, mark current live ownership lost while retaining durable native thread identity only as a future resume candidate;
10. never report a reconnected native thread as `LIVE`; it is `RESUMED` only after exact mapping/use succeeds.

Bound every reader/frame/diagnostic buffer. A malformed/oversized/unknown message must fail or downgrade the affected capability; it must not grant authority.

### Claude connected-session lifecycle

The first Claude path should be deliberately narrower than Codex App Server because the local control surfaces differ.

1. revalidate exact discovered `claude` executable/version;
2. use documented structured print/stream mode, not TUI scraping;
3. use exact native `--resume <session-id>` only when Winds has an exact revalidated binding;
4. do not use `--continue` to satisfy Winds canonical continuation because recency is not identity;
5. do not use `--dangerously-skip-permissions`;
6. for the first live Planner path, restrict the role to read/planning behavior under the strongest documented local agent-native restriction that can be proven for the exact version;
7. label that restriction `AGENT_NATIVE_ENFORCED` or weaker unless Winds/OS mediation independently proves stronger enforcement;
8. if a non-interactive write/tool permission path cannot be mediated without MCP or unsafe bypass flags, do not work around it—keep Claude Planner-only in the first walking skeleton and let the Codex Worker path prove bounded editing;
9. persist the returned native Claude session ID only as a native binding, not canonical task identity;
10. reconstructed sessions use canonical context and are labelled `RECONSTRUCTED`, never fake `RESUMED`.

This asymmetry is intentional: runtime support is capability-driven, not forced into lowest-common-denominator semantics.

### First differentiated live walking skeleton

After fixture-only identity/context/authority slices are accepted, the smallest useful live demonstration should be:

1. human selects an existing Winds workspace/workstream;
2. Winds opens/continues a named **Planner** Winds session backed by Claude Code in an exact locally supported read/plan-oriented mode;
3. Winds generates a deterministic canonical context capsule for the Planner;
4. Planner returns a bounded Worker proposal; that proposal is still `AGENT_REPORTED`;
5. Winds presents a normalized delegation contract and the human explicitly approves or denies it;
6. approved work opens/uses a separate Worker Winds session and, when the implementing task requires edit isolation, a separate exact Git worktree;
7. Codex App Server is used as the first structured Worker runtime because its client/server approval flow is explicitly designed for embedding;
8. Winds policy evaluates requested operations and labels actual enforcement quality truthfully;
9. Worker result remains agent-reported until Winds independently observes the Git candidate and deterministic evidence;
10. Winds observes exact candidate OID/tree and prepares independent-review context that excludes builder persuasion/confidence as authority;
11. deterministic `winds verify` executes against the exact candidate using existing verification semantics;
12. candidate movement invalidates prior review/check applicability;
13. human makes the final landing decision; Winds does not merge/push automatically.

If any real-runtime prerequisite fails, fixture-only proof remains valid but the live claim is not made.

## Authority Architecture

### Separate policy evaluation from enforcement

Create a small deterministic authority evaluator before any live adapter can request a write/tool operation.

The evaluator consumes only typed inputs:

- requested operation/capability/resource identity;
- Planner direct authority;
- Planner delegation ceiling;
- explicit Worker grant;
- team policy if introduced by the accepted task;
- human ceiling/decision;
- applicable explicit deny/ask/allow rules;
- content/resource binding identity where approval is content-bound.

It returns a decision plus the reason and required human action. It does **not** execute the operation.

This pure evaluator is testable without a model, runtime, ACP, MCP, network, or subprocess.

### Protected policy plane

Canonical policy/trust records must live under the Winds state root / Store authority domain, not as ordinary governed repository files.

A repository may contain advisory/project-requested policy text later, but it cannot override protected human policy merely because an agent can edit the repository.

OS filesystem permissions may harden local state where available, but do not overclaim security against the same user principal. If the actor/runtime can directly reach the host resource outside Winds mediation, represent the stronger ceiling as policy intent and the actual enforcement as `AGENT_NATIVE_ENFORCED`, `BEST_EFFORT_TRIPWIRE`, `OBSERVATION_ONLY`, or `UNAVAILABLE` as appropriate.

`WINDS_ENFORCED` is reserved for operations whose relevant path is actually mediated by Winds in the accepted implementation.

### Human approval binding

The first delegation approval should bind to deterministic normalized content, not an unstructured chat sentence.

At minimum bind:

- workstream/session IDs;
- Worker role/runtime request;
- workspace/worktree/root identity;
- requested operation/capability set;
- resource/path scope;
- relevant context capsule digest;
- applicable budget fields if that task uses budgets;
- delegation ceiling/Worker grant;
- candidate/base identity where already known.

Use an existing cryptographic primitive (`sha2`) for a canonical digest when an implementing task needs content-bound approval. Do not add a signing/PKI dependency for local same-user approval without a demonstrated threat model.

A material normalized-content change invalidates the approval and returns to ask/deny according to policy.

## Context Architecture

### Source classifications

Do not overload Spec 003's `FactSource` blindly if its vocabulary becomes semantically wrong for Agentic context.

Plan for an agentic authority/source vocabulary that can represent at least:

- `HUMAN_DECIDED`;
- `WINDS_OBSERVED`;
- `AGENT_REPORTED`;
- `IMPORTED_HISTORY`;
- `VENDOR_DECLARED` / `CATALOG_DECLARED` / `WINDS_LOCALLY_OBSERVED` for capability claims where those are the correct dimension.

Keep **fact authority/source** separate from **capability provenance** when they answer different questions. Do not make one giant enum just to reuse a column.

### Capsule determinism

Canonical serialization rules must be explicit before a live model sees a capsule:

- stable key/record ordering;
- normalized UTF-8 text policy;
- explicit schema version;
- deterministic omission/truncation markers;
- no timestamps inside a digest unless the timestamp is itself canonical input;
- no absolute temporary paths unless required and source-labelled;
- no nondeterministic HashMap iteration;
- no secrets/full environment by default.

Use existing `serde`/`serde_json` and `sha2`; no vector DB, embedding model, retrieval service, or tokenization dependency is required for the first capsule.

### Imported/native history

Do not import entire vendor histories automatically.

When a later task needs native history:

- import only through a documented exact runtime surface;
- bound bytes/items;
- retain runtime/session/source identity;
- treat text/tool claims as imported data;
- never import inaccessible private reasoning by assumption;
- never let imported history overwrite human/observed canonical facts;
- make omission/unavailable state visible in the transfer report.

## Persistence Plan

Use forward-only SQLite migrations and the existing `Store`; do not create another database.

Sequence persistence by proven need rather than one giant schema migration:

### Identity migration (first persistence task)

Add only:

- `workstreams`;
- `winds_sessions`;
- `runtime_session_bindings` if exact native binding is part of that same accepted slice; otherwise defer it one task.

Required invariants:

- foreign keys to existing `workspaces`;
- stable IDs independent of display names;
- fail-closed typed reads;
- deterministic rename semantics;
- no model/provider credential/history blobs.

### Canonical fact/reference persistence (only when context slice needs it)

Add one bounded typed table for canonical work facts/references if core workstream fields are insufficient. Avoid a generic memory/document store.

### Delegation persistence (only when live/auditability requires it)

Before live delegation, add a typed immutable/append-oriented contract/audit record sufficient to recover what the human approved, its digest, decision, requested authority, and enforcement-quality observation.

Do not store secret credentials inside delegation records.

### Agent execution child records (only when live runtime execution lands)

Reuse `executions` and add a typed child table linked to Winds session/runtime binding. Extend `ExecutionKind` only with actual accepted agent execution kinds.

No table is authorized merely because this Plan names it; Tasks decide the exact migration boundaries after Plan acceptance.

## CLI / UX Proof Surface

Keep the existing single `winds` binary and simple dispatch style.

The backend must be provable before rich TUI/GUI work. Exact command spelling is a Tasks decision, but the minimal eventual CLI capabilities are:

- list/select workspace/workstream/Winds sessions;
- create/rename a session without changing task identity;
- explicitly continue/fork/new-task;
- inspect Codex/Claude discovery/capability provenance;
- render context-transfer summary/digest;
- present one delegation proposal and explicit human decision;
- inspect exact runtime/native binding and continuity state;
- bind/inspect exact candidate and stale review/evidence state;
- invoke/reuse existing verify path for exact candidate proof.

Do not add a large nested CLI framework or fuzzy-search dependency until the P2 task proves the existing/simple parsing cannot satisfy it.

## Git and Worktree Strategy

Reuse existing system-Git code and registration identity.

For fixture-only and Planner-only phases, do not create extra worktrees merely to look isolated.

When the Worker editing task lands:

- start from an exact explicit base/candidate parent;
- create a Winds-owned Worker worktree using existing non-destructive Git primitives or the smallest extension consistent with them;
- bind Worker session/delegation to exact worktree root/common-dir identity;
- observe candidate OID/tree through system Git;
- retain failed/dirty worktree for recovery;
- do not call it an OS sandbox;
- do not automatically merge/remove/clean it after Worker completion;
- keep final human landing outside the agent loop.

## Candidate / Review Staleness Model

Applicability is a pure relation between evidence/review and exact candidate identity.

The implementing design should model at least:

- `CURRENT` / applicable to exact OID/tree;
- `STALE` / historical after candidate movement;
- `UNAVAILABLE` / no accepted evidence/review exists.

Do not delete old evidence when it becomes stale.

Independent-review context should be generated from:

- exact candidate OID/tree/diff;
- Spec 006 acceptance criteria relevant to the task;
- canonical constraints/human decisions;
- accepted deterministic evidence references that already apply;
- no builder confidence/persuasion as authority.

A reviewer output remains a review claim until its accepted review process classifies the finding; it does not itself mutate candidate/evidence state.

## Process Ownership and Restart Boundary

Spec 006 first program does not introduce a long-lived owner.

For Codex App Server or any local controlled child:

- reuse the existing ownership lesson: exact retained child/process-scope object is the live authority;
- a persisted PID is never sufficient after restart;
- Winds restart loses live ownership;
- durable native session/thread ID may remain a resume candidate but is not proof of a live process;
- controlled cleanup acts only on proven owned resources;
- unproven cleanup/restart becomes `OWNERSHIP_LOST` or equivalent truthful state.

Do not retrofit the terminal ownership-lease implementation into a fake daemon. Persistent ownership is a later specification/gate.

## Security and Trust Boundary

The Plan assumes a local developer workstation and does not claim isolation from the same OS user principal.

Threats the first program MUST address:

- repository/config/history text attempting to grant itself authority;
- Planner/Worker prose requesting escalation;
- stale native session IDs or executable replacement;
- approval replay after content/resource/candidate change;
- runtime/tool events claiming success without Winds observation;
- oversized/malformed structured streams;
- imported-history prompt injection;
- agent modifying policy-like files inside the worktree;
- direct-host runtime capabilities that bypass Winds mediation;
- candidate movement while checks/review are running;
- secrets/full environments accidentally entering context or persistence.

Claims the first program MUST NOT make:

- worktree = sandbox;
- agent-native deny = OS sandbox;
- Winds policy intent = Winds enforcement when bypass exists;
- ACP root = sandbox;
- native session resume = canonical context preservation;
- model/tool output = verification evidence;
- a local CLI invocation = cryptographically authenticated human identity.

## Dependency Strategy

### No new dependency in fixture-first core

Identity, Store, deterministic context, authority evaluation, candidate binding, and read-only runtime discovery can be built with the standard library plus already accepted `rusqlite`, `serde`, `serde_json`, and `sha2`.

Do not add a fuzzy matcher, async runtime, HTTP client, JSON-RPC framework, process supervisor, policy engine, vector DB, tokenizer, embedding model, or plugin crate for these slices.

### Codex path

Prefer direct bounded stdio + typed serde messages over a new protocol framework. If the official protocol/code generation becomes necessary for safe version compatibility, the implementing task must justify the smallest dependency/codegen footprint with exact provenance and Ponytail review.

### Claude path

Use the local executable/structured CLI surface. No Anthropic Rust SDK/API dependency is required for local Claude Code control.

### ACP

Do not land the Rust SDK until a concrete ACP path is selected. When selected, repeat exact dependency/license/MSRV/platform audit against the then-canonical tree rather than relying only on the governance pin.

## Platform Strategy

Do not turn current terminal support into an unsupported Agentic claim automatically.

Each task must declare its execution-domain support.

Recommended progression:

1. fixture-only domain/Store/context/authority tests on existing quality platforms;
2. read-only runtime discovery tests using fake executables cross-platform where practical;
3. first real local structured runtime proof on the platform(s) with deterministic CI/fixture support available for that runtime;
4. Windows/WSL real-runtime claims only after explicit Windows/WSL evidence exists;
5. native-Windows authoritative `winds verify` remains separately unsupported unless that pre-existing authority changes through its own accepted task.

Do not hide platform gaps behind a cross-platform enum.

## Implementation Phases After Tasks Are Canonical

This Plan does not create Tasks, but it freezes the dependency order that Tasks should refine.

### Phase 1 - Canonical identity substrate (fixture-only)

Prove stable workstream/Winds-session identity and rename/continue/fork/new-task semantics in Store/domain tests. No agent process.

### Phase 2 - Runtime discovery and capability provenance (fixture-only)

Discover Codex/Claude executable/version/capability facts with fake executables and no agent task execution. Revalidate stale replacement. No prompts/model calls.

### Phase 3 - Deterministic context capsule and transfer report (fixture-only)

Persist only the necessary canonical facts/references, serialize deterministic capsules, prove compaction/import provenance/non-overwrite rules. No agent process.

### Phase 4 - Authority/delegation evaluator (fixture-only)

Pure deterministic policy tests for direct authority, delegation ceiling, child grant, deny precedence, content-bound approval digest, enforcement-quality reporting, and protected policy plane. No model/runtime required.

### Phase 5 - Codex connected-session proof

Land only the minimal local App Server control path and exact dependency/protocol handling required by the accepted task. Prove create/resume/fork/approval-event behavior with fixture/fake server first, then a real exact runtime proof where CI/environment permits.

### Phase 6 - Claude Planner + cross-runtime handoff proof

Use structured Claude Code local mode and exact native resume only. Keep Planner authority read/plan-oriented unless stronger mediation is independently proven. Prove Claude -> canonical capsule -> Codex continuity without claiming private-state transfer.

### Phase 7 - Single human-approved Planner -> Worker walking skeleton

One Planner, one Worker, one approved contract, one exact Worker worktree when editing is required. No recursive fleet.

### Phase 8 - Exact candidate independent review + verification integration

Bind exact candidate, generate review-safe context, prove stale-on-movement, invoke/reuse existing deterministic verification authority, and preserve explicit human landing decision.

### Phase 9 - P2 findability only after the P1 loop is proven

Add fuzzy/session/file/context selection only if the accepted P1 loop shows a real UX need. Prefer deterministic simple matching before a new dependency.

### Phase 10 - Acceptance

Docs, complete cross-platform/negative/fault tests, correctness/safety review, Ponytail, fresh independent exact-head review, and final evidence reconciliation.

## Deterministic Test Strategy Mapped to Spec Success Criteria

### SC-001 / SC-002 - identity

- rename fixtures prove IDs/FKs/evidence refs unchanged;
- at least 20 sessions / 5 workstreams under one workspace;
- duplicate/case/Unicode display names do not collide with identity;
- fail-closed invalid FK/unknown typed vocabulary reads.

### SC-003 / SC-004 - continuity

- explicit continue/fork/new semantics;
- exact native resume success;
- stale/missing/ambiguous native mapping -> reconstructed/fail-closed;
- reused native/process ID cannot become owned/live by coincidence.

### SC-005 / SC-006 - context

- byte-for-byte deterministic capsule for same canonical inputs/policy;
- deterministic digest;
- imported facts cannot overwrite human/observed facts;
- transferred/derived/omitted/unavailable categories;
- cross-runtime fixture with explicit unavailable private state.

### SC-007 - discovery

- fake Codex/Claude executables prove version/capability observation only;
- no install/update/auth/terms/prompt/model call;
- executable replacement between discovery/use detected.

### SC-008 / SC-009 - authority/enforcement

- child-over-ceiling denied;
- Planner direct authority unchanged by broader delegation ceiling;
- policy-file/repository prompt injection cannot escalate;
- deny precedence;
- changed content/resource digest requires reapproval;
- bypassable runtime host access cannot be labelled `WINDS_ENFORCED`.

### SC-010 / SC-011 - review/evidence

- exact candidate A evidence/review current;
- mutation to candidate B makes A stale while retaining history;
- reviewer capsule excludes builder confidence as authority;
- agent `done/tests passed` cannot satisfy acceptance.

### SC-012 / SC-013 - walking skeleton scope

- exactly one Planner -> one Worker path;
- no recursive delegation dependency;
- no daemon/public network/MCP/remote/plugin/renderer/SQL/LLM-obs dependency needed for the accepted proof.

### SC-014 - verification regression

Every implementation slice that touches candidate/review integration must run the full applicable pre-existing verify/promote/recover regression suite. Agentic execution/history must not alter verification eligibility semantics.

## Negative and Fault Testing

Tasks must include relevant tests from this list before the corresponding claim is accepted:

- corrupt/unknown workstream/session/native-binding DB values;
- session rename racing with selection/use;
- deleted workspace/worktree after session persistence;
- executable replaced after discovery;
- runtime version output malformed/oversized/non-UTF-8 where supported;
- Codex App Server exits before/after initialize;
- malformed/oversized JSONL frame;
- unknown App Server notification/request;
- approval request arrives after content/candidate binding changed;
- Claude resume ID missing/rejected/reused;
- structured Claude output truncated/malformed;
- imported history includes prompt-injected `allow`, tool syntax, or conflicting canonical facts;
- capsule size bound/truncation preserves canonical state and explicit omission markers;
- Planner proposes Worker grant above ceiling;
- Worker event requests outside approved resource scope;
- protected Winds policy differs from editable repo policy text;
- child/runtime claims operation succeeded while Git observation disproves it;
- runtime dies while DB says active -> truthful interrupted/ownership-lost mapping;
- candidate changes during reviewer/check collection -> stale applicability;
- dirty/failed Worker worktree retained for recovery;
- no blind PID/session attachment after Winds restart.

## Soak / Repetition

Do not invent a large live-agent soak before fixture semantics stabilize.

Required progression:

1. deterministic Store/identity/context/authority repetitions first;
2. bounded fake-runtime protocol lifecycle repetition for Codex/Claude adapters;
3. only after stable live integration, repeat the single Planner -> Worker fixture/walking skeleton enough to detect leaked owned child processes, stale active rows, nondeterministic capsule hashes, approval reuse, or evidence-staleness bugs.

The exact cycle count belongs in Tasks after measuring runtime cost; unlike Spec 003 PTY lifecycle, a network/model-dependent 100-cycle live-agent soak would be expensive and nondeterministic and is not justified as a default acceptance gate.

## Performance Measurement Plan

Spec 006 intentionally did not invent latency targets.

During implementation Tasks, measure separately:

- local Store identity/session list/create/rename latency;
- context capsule generation time/bytes;
- fake-runtime discovery/control overhead;
- real runtime startup/handshake overhead where a deterministic environment permits it.

Do not mix model inference latency into Winds local-control performance claims. Model/provider latency is external and belongs to later observability work if/when authorized.

## Review Strategy

### Correctness / safety

Review every relevant slice for:

- canonical task/session/native-ID conflation;
- false live/resume state;
- runtime replacement/stale discovery;
- imported-history trust escalation;
- policy self-escalation;
- content-bound approval replay;
- enforcement-quality overclaim;
- worktree-as-sandbox overclaim;
- protocol framing/output bounds;
- process ownership/reaping mistakes;
- credential/environment leakage;
- candidate/review/evidence staleness errors;
- accidental change to verify/promote/recover authority.

### Ponytail

Challenge and remove:

- generic runtime/provider/plugin traits before three concrete runtimes prove a common seam;
- unused ACP dependency;
- async runtime merely for two local child processes;
- custom JSON-RPC framework;
- second persistence/event system;
- vector/embedding/RAG subsystem for context capsules;
- hosted control plane;
- daemon/IPC;
- MCP;
- recursive fleet scheduler;
- custom sandbox manager;
- broad fuzzy-search dependency before P2 need;
- custom renderer;
- speculative SQL/LLM schemas.

### Independent review

At least one reviewer other than the authoring agent must inspect the exact final implementation candidate for each acceptance-critical slice. Reviews bound only to older heads are stale.

For delegation/security slices, independent review must explicitly challenge:

- whether a stated ceiling is actually enforceable;
- whether same-user direct host access bypasses Winds;
- whether repo/model text can influence protected authority;
- whether candidate/review staleness is fail-closed.

## Plan-to-Tasks Gate

Acceptance of this Plan authorizes creation/review of a separate `tasks.md` only.

Before Tasks can authorize implementation they must:

- decompose the phases above into independently reviewable slices;
- identify exact files/tests expected per slice without pre-authorizing unrelated files;
- keep fixture-only identity/discovery/context/authority work before real agent execution;
- require a fresh dependency/provenance gate before any new crate lands;
- explicitly identify the first task that is allowed to start a real Codex/Claude process/send a prompt;
- keep MCP/daemon/remote execution unauthorized;
- require expected-head and exact-candidate CI/review evidence before each canonical merge.

This Plan itself does **not** authorize:

- `tasks.md` implementation claims before Plan merge;
- `Cargo.toml` / `Cargo.lock` mutation;
- migrations;
- source/runtime code;
- starting Codex/Claude;
- prompt/model/provider calls;
- ACP crate landing;
- MCP;
- daemon/IPC;
- remote execution;
- automatic landing.
