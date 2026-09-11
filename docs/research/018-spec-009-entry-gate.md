# Spec 009 Formal Entry Gate — Model Mesh & Explicit Multi-Provider Continuity

**Status:** Governance entry candidate. Specification-only authority if canonically accepted.

**Canonical base at creation:** `cf016cab5c2a5cd6a02d03387bd564f0be21989a`

**Date:** 2026-09-11

## 1. Purpose

Spec 008's first implementation program is canonically closed through T113. Its guarded merge is `cf016cab5c2a5cd6a02d03387bd564f0be21989a`, with post-merge `quality #1198` successful on attempt 1.

This entry gate reconciles the post-Spec-008 repository truth with the accepted future ordering and records the Founder decision for the next formal Spec Kit sequence.

This document does not create an implementation task. It may authorize only a separate Spec 009 specification candidate after this exact entry gate is independently qualified and canonically landed.

## 2. Canonical inputs

The entry decision is bounded by:

- Winds Constitution 1.1.0;
- canonical Spec 008 closeout through T113;
- `docs/research/014-loopforge-skillhone-roadmap-reconciliation.md`;
- `docs/research/015-selective-code-adoption-master-plan.md`;
- `docs/research/016-post-spec-007-roadmap-status-reconciliation.md`;
- `docs/research/017-spec-008-entry-gate.md`;
- research RFC #91, which remains non-canonical input;
- Draft PR #21, which remains a research archive rather than an acceptance surface;
- current accepted runtime/context implementation, including `src/agentic_runtime.rs` and `src/agentic_context.rs`, as repository truth rather than future authority.

The accepted post-Spec-008 truth is:

```text
T101..T113=CLOSED_CANONICAL
SPEC_008_ENTRY=CLOSED_CANONICAL
SPEC_008_SPEC=CLOSED_CANONICAL
SPEC_008_PLAN=CLOSED_CANONICAL
SPEC_008_TASKS=CLOSED_CANONICAL
SPEC_008_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
CANONICAL_MAIN=cf016cab5c2a5cd6a02d03387bd564f0be21989a
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Those live-runtime nonclaims remain unchanged. Model Mesh planning must not retroactively upgrade any earlier runtime proof level.

## 3. Founder decision

The Founder authorizes only the next canonical formal specification stage under the mandatory `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing` discipline.

The next formal specification is:

```text
SPEC_009_NAME=MODEL_MESH_AND_EXPLICIT_MULTI_PROVIDER_CONTINUITY
SPEC_009_FORMAL_SPEC_AUTHORIZED=YES_AFTER_THIS_ENTRY_LANDS
SPEC_009_PLAN_AUTHORIZED=NO
SPEC_009_TASKS_AUTHORIZED=NO
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
```

This ordinary project authority does not waive exact-head CI, correctness/safety review, Ponytail review, independent review, evidence reconciliation, guarded landing, source/dependency qualification, platform truth, or secret-handling boundaries.

The Founder's general instruction to continue does not itself authorize Spec 009 implementation. Canonical implementation authority may exist only after this entry lands and the separate Spec, Plan, and Tasks stages are each independently qualified and canonically accepted in dependency order.

## 4. Why Spec 009 is next

Canonical research already orders Model Mesh immediately after the Resumable Workflow & Decision Ledger, and `014-loopforge-skillhone-roadmap-reconciliation.md` states: `Only then author Model Mesh Spec 009`.

Spec 008 now supplies the missing canonical workflow target: provider/runtime continuity can bind to explicit `WorkflowRun`, `StageRun`, artifact freshness, decision lineage, and reconstruction truth instead of treating a provider transcript or native session as workflow authority.

The next missing layer is:

> **Explicit multi-runtime/provider/model identity and continuity without silent routing or authority expansion.**

## 5. Current repository baseline

The current accepted implementation already proves several inputs that Spec 009 must preserve rather than redesign:

- `RuntimeKind` is deliberately bounded to `Codex` and `Claude`;
- runtime discovery separates local observation, vendor declaration, catalog declaration, version state, capability state, and unknown authentication readiness;
- durable runtime/native bindings cannot manufacture live ownership after restart;
- native resume candidates are not equivalent to proven resume;
- canonical Winds session/workstream identity is distinct from native runtime identity;
- the accepted cross-runtime handoff is currently a deliberately narrow Claude-Planner -> Codex-Worker contract;
- context transfer is content-bound and authority-bounded;
- Spec 008 workflow continuation refuses to promote unproven runtime state into resumed workflow truth.

This means Spec 009 should generalize explicit continuity semantics only where required. It should not replace the existing runtime model with a generic plugin framework.

## 6. Proposed specification scope

The separate Spec 009 specification may define implementation-agnostic user scenarios, invariants, security/non-goals, and measurable acceptance criteria for:

- explicit separation of canonical Winds session/workflow identity, runtime/harness identity, provider identity, model identity, and provider-native session/turn identity;
- explicit user or policy-authorized target selection, with no silent provider/model switching;
- a direction-neutral continuity contract over the currently accepted Codex and Claude runtime families before broader runtime admission;
- exact provider/model/runtime provenance on handoff, execution observation, and returned agent material where such identity is locally observable;
- truthful `UNKNOWN`/`UNAVAILABLE` states when provider or model identity cannot be proven;
- capability declarations and local observations that cannot self-authorize execution;
- exact binding of provider/model continuity to canonical `WorkflowRun` / `StageRun`, candidate, artifact, evidence, and decision context;
- stale-state rules when runtime executable, runtime version, provider/model identity, native session identity, authority, workflow attempt, or candidate identity moves;
- explicit reconstruction and reassignment semantics when exact native continuation is unavailable;
- deterministic handoff context assembled from canonical structured state rather than provider-private memory or transcript replay;
- user-visible explanation of source runtime/provider/model, destination runtime/provider/model, what continuity was proven, and what was reconstructed or lost;
- secret-safe credential references and authentication-readiness truth without persisting raw credentials into canonical state, evidence, transcripts, or repository content;
- explicit unavailable/fallback behavior when a requested runtime/provider/model target cannot be used under the current authority ceiling;
- bounded observed usage/cost metadata only when supplied by a structured trustworthy observation; absence must remain unknown rather than estimated as fact;
- deterministic and adversarial qualification of identity drift, stale native sessions, ambiguous targets, unavailable capabilities, forged provider/model text, and cross-runtime authority escalation attempts.

The first implementation slice should prefer the already accepted Codex and Claude surfaces. Broader providers/runtimes must be admitted only from concrete user need and separately qualified capability, provenance, dependency, secret, and platform boundaries.

## 7. Required identity invariants

The specification should evaluate and, where accepted, formalize at least:

```text
WINDS_SESSION != RUNTIME_SESSION
RUNTIME != PROVIDER
PROVIDER != MODEL
MODEL_IDENTITY != MODEL_OUTPUT_TEXT
NATIVE_SESSION != WORKFLOW_RUN
NATIVE_TURN != STAGE_RUN
RUNTIME_DISCOVERY != EXECUTION_AUTHORITY
PROVIDER_AVAILABILITY != AUTHENTICATION_READINESS
MODEL_SELECTION != VERIFICATION_AUTHORITY
HANDOFF != NATIVE_RESUME
RECONSTRUCTED != RESUMED
PROVIDER_OR_MODEL_DRIFT => PRIOR_PROVIDER_BOUND_CONTINUITY_STALE
AGENT_REPORTED_MODEL_NAME != WINDS_OBSERVED_MODEL_IDENTITY
```

## 8. Explicit Spec 009 non-goals

Unless a later accepted amendment changes this boundary, Spec 009 does not authorize:

- automatic provider/model routing, learned routing, winner scoring, or silent fallback;
- generic plugin/provider marketplace, arbitrary adapter ABI, dynamic code loading, or user-installed runtime plugins;
- broad third-party provider fleet merely to claim provider count;
- a mandatory gateway such as LiteLLM or another central provider proxy;
- persistent background owner, daemon, server, socket, IPC, HTTP/SSE/WebSocket control plane, or public runtime protocol;
- remote execution, remote/mobile continuation, cloud/team control plane, or service orchestration;
- browser automation, browser profiles, CDP, screenshots as verification, or Browser Twin;
- learning, skill optimization/promotion, experiment-plane activation, training, fine-tuning, RL, or automatic policy mutation;
- MCP runtime expansion, A2A, or a generic tool/runtime protocol layer merely to support Model Mesh;
- credential acquisition, password/token scraping, automatic login, refresh-token brokerage, or raw-secret durable storage;
- provider/model output becoming verification or human-acceptance authority;
- automatic candidate selection, merge, rebase, cherry-pick, push, PR creation, or landing;
- weakening any accepted Spec 003/006/007/008 Git, evidence, authority, workflow, privacy, platform, recovery, or human-landing invariant.

## 9. Dependency, provenance, and secret boundary

This entry gate chooses no provider SDK, API client, gateway, credential library, or new dependency.

Any later Plan/Tasks unit that proposes a new runtime/provider integration or dependency must prove exact version/revision, license, provenance, supported platform/domain, secret boundary, network authority, failure semantics, removal path, and why the accepted existing runtime surfaces are insufficient.

Provider/model catalog declarations remain non-authoritative until locally observed at the proof level required by the accepted slice. Raw credentials must never become canonical workflow context or verification evidence.

## 10. Provisional future sequence

This is a governance ordering decision, not Tasks authorization for downstream specifications:

```text
SPEC_009  Model Mesh & explicit multi-provider continuity
    ↓
SPEC_010  Winds Continuum + durable local runtime owner
    ↓
SPEC_011  Verified Browser + browser evidence
    ↓
SPEC_012  Proof-Carrying Reality Branches / candidate comparison
    ↓
SPEC_013  Verified Learning + Experiment Plane
    ↓
SPEC_014  Remote/mobile/team continuation, only after local truth is strong
```

Later specifications remain non-authorized research ordering until each receives its own accepted entry/specification authority.

## 11. Entry acceptance gate

This entry gate may land only if the exact final candidate proves:

- changed scope is governance/research documentation only;
- canonical base is exactly descended from the complete Spec 008 T113 closeout;
- the stale historical PR #144/#145 paths are not treated as current authority;
- no production source, dependency, lockfile, migration, workflow-semantic implementation, runtime, provider, model, browser, daemon, IPC, learning, remote-execution, credential, or automatic-landing mutation;
- repository `quality` succeeds on the exact final head;
- correctness/governance/evidence-integrity author review passes;
- Ponytail/YAGNI review finds no premature abstraction, provider fleet, gateway, or protocol framework;
- a fresh independent reviewer challenges the exact candidate, sequence, current-runtime claims, identity model, secret boundary, and non-authorization claims;
- zero unresolved material findings/threads;
- final base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal landing;
- canonical merge identity/tree/ordered parents/signature verification;
- every actually triggered applicable post-merge push workflow succeeds.

Only after canonical landing may repository truth state:

```text
SPEC_008_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_009_FORMAL_SPEC_AUTHORIZED=YES
SPEC_009_PLAN_AUTHORIZED=NO
SPEC_009_TASKS_AUTHORIZED=NO
SPEC_009_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IPC_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
BROWSER_RUNTIME_AUTHORIZED=NO
LEARNING_IMPLEMENTATION_AUTHORIZED=NO
AUTOMATIC_ROUTING_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

## 12. Entry effect

If this entry gate closes canonically, the next authorized repository unit is a separate Spec 009 `spec.md` candidate only.

That specification must start from then-current canonical `main`, remain implementation-agnostic, preserve all historical failure/nonclaim evidence, and pass exact-candidate acceptance gates before any Spec 009 Plan may begin.