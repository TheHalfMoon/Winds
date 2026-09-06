# Spec 007 Formal Entry Gate — Native Agentic Terminal UX Foundation

**Status:** Governance entry candidate. Specification-only authority if canonically accepted.

**Canonical base at creation:** `b918665d5167bc648a9087825280db81e0f69f97`

**Date:** 2026-09-06

## 1. Purpose

Spec 006's first implementation program is canonically closed. This entry gate reconciles the post-Spec-006 research programs against that exact repository state and records the Founder decision for the next formal Spec Kit sequence.

This document does not itself create an implementation task. It may authorize only a separate Spec 007 specification candidate after this entry gate is independently qualified and canonically landed.

## 2. Canonical inputs

The entry decision is bounded by:

- Winds Constitution 1.1.0;
- canonical Spec 006 closeout through T086;
- `docs/research/010-verified-learning-loop-roadmap.md`;
- `docs/research/011-herdr-parity-and-beyond-roadmap.md`;
- `docs/research/012-agentic-era-terminal-north-star.md`;
- research RFC #91, which remains non-canonical input rather than implementation authority.

The accepted Spec 006 closeout truth remains:

```text
T070..T086=CLOSED_CANONICAL
SPEC_006_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_006_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Historical bounded T079 attempts remain historical. No live-runtime nonclaim is upgraded by this entry decision.

## 3. Founder decision

The Founder authorizes the next canonical planning sequence to proceed through the repository's mandatory `Constitution -> Spec -> Plan -> Tasks -> Implement -> Verify -> Review -> Human landing` discipline.

The next formal specification is:

```text
SPEC_007_NAME=NATIVE_AGENTIC_TERMINAL_UX_FOUNDATION
SPEC_007_FORMAL_SPEC_AUTHORIZED=YES_AFTER_THIS_ENTRY_LANDS
SPEC_007_PLAN_AUTHORIZED=NO
SPEC_007_TASKS_AUTHORIZED=NO
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
```

This decision is ordinary project authority only. It does not waive exact-head CI, correctness/safety review, Ponytail review, independent review, evidence reconciliation, or guarded landing.

## 4. Post-Spec-006 roadmap reconciliation

The North Star requires an explicit accepted governance decision before `N0`–`N8` may reorder or bypass the research-only `L0`–`L3` and `H0` sequencing. This entry gate makes that dependency decision explicitly.

### 4.1 Verified Learning `L0`–`L3`

The learning roadmap remains valuable but is not a prerequisite for the first terminal-workbench specification.

Decision:

```text
L0_L3_STATUS=DEFERRED_SEPARATE_FUTURE_FORMAL_PROGRAM
L0_L3_BLOCKS_SPEC_007=NO
LEARNING_IMPLEMENTATION_AUTHORIZED=NO
```

Rationale:

- Spec 007 requires no learned behavior, training, routing, skill promotion, or self-modification;
- exact-candidate evidence and review authority already exist without a learning subsystem;
- forcing a learning plane before basic daily terminal UX would add architecture unrelated to the first user need;
- the learning roadmap itself states that downstream phases are not required merely because they appear in research sequencing.

A future learning program must still enter through its own accepted Spec Kit sequence and protected-evaluation gates.

### 4.2 Herdr `H0`

The H0 reconciliation/threat-model entry is satisfied for Spec 007 by an explicit negative architecture decision:

```text
SPEC_007_PERSISTENT_OWNER=NOT_AUTHORIZED
SPEC_007_DAEMON=NOT_AUTHORIZED
SPEC_007_LOCAL_IPC=NOT_AUTHORIZED
SPEC_007_REMOTE_CONTROL=NOT_AUTHORIZED
CURRENT_ONE_PROCESS_ARCHITECTURE=PRESERVED
```

Spec 007 must not require cross-process ownership or a private/public control protocol. Persistent ownership, detach/reattach across owner restart, authenticated local control, and remote continuity remain a later formal specification with an explicit threat model.

### 4.3 North Star `N0` / `N1`

Spec 007 formalizes the smallest useful portion of the North Star:

- `N0`: decide the product/UI kernel boundaries needed for the first workbench without adopting a daemon or IPC;
- `N1`: define the first beautiful terminal-workbench foundation while preserving existing verification authority.

Framework/dependency selection is not made by this entry gate. The Spec 007 Plan must choose the smallest architecture only after the accepted specification freezes requirements, and any new UI/rendering dependency requires exact-version/license/MSRV/platform/provenance review before Tasks may rely on it.

## 5. Spec 007 specification scope

The specification may define user scenarios and measurable acceptance criteria for:

- a daily-driver terminal workbench built on the existing PTY/ConPTY/WSL execution truth;
- typed, source-labelled terminal interaction records or blocks without converting terminal output into verification evidence;
- a high-quality universal command input/editor for shell-first use;
- tabs/panes/workspace/session navigation within the current non-daemon process model;
- keyboard-first workflows, search/history, copy/collapse/navigation, and attention/accessibility semantics;
- code/diff/evidence inspection surfaces that preserve `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`;
- explicit `DONE != VERIFIED` presentation;
- preservation and discoverability of `winds verify` and exact-candidate evidence;
- measurable startup, input-latency, large-output, idle-resource, and interaction budgets;
- Linux/macOS/Windows/WSL claims only where directly exercised by deterministic evidence.

The specification must remain implementation-agnostic. It may state product requirements for a native/rich workbench but must not select a framework merely because research references use one.

## 6. Explicit Spec 007 non-goals

Unless a later accepted amendment changes this boundary, Spec 007 does not authorize:

- persistent owner, daemon, server, socket, IPC, HTTP/SSE/WebSocket control plane;
- remote execution or mobile/remote continuation;
- browser automation, browser profiles, CDP, screenshots as verification, or Browser Twin;
- provider mesh, provider SDKs, automatic model routing, new provider authentication, or credential brokerage;
- MCP runtime, ACP dependency landing, A2A, or generic runtime/plugin frameworks;
- recursive/multi-worker fleets beyond the already accepted bounded Spec 006 semantics;
- learning, VerifiedExperience activation, skill mutation/promotion, experiment plane, training, fine-tuning, or RL;
- vector/RAG memory, semantic memory engine, or transcript-as-canonical-memory behavior;
- plugin host, marketplace, integration SDK, or supply-chain execution surface;
- automatic candidate selection, merge, rebase, cherry-pick, push, PR creation, or landing;
- weakening any Spec 003/006 Git, evidence, authority, platform, privacy, or recovery invariant.

## 7. Provisional future sequence

This is a governance ordering decision, not Tasks authorization for downstream specifications:

```text
SPEC_007  Native Agentic Terminal UX Foundation
    ↓
SPEC_008  Model Mesh / explicit multi-provider session continuity
    ↓
SPEC_009  Winds Continuum + durable local runtime owner
    ↓
SPEC_010  Verified Browser + browser evidence
    ↓
SPEC_011  Proof-Carrying Reality Branches / candidate comparison
    ↓
SPEC_012  Remote/mobile/team continuation, only after local truth is strong
```

The Herdr H1–H12 and North-Star N2–N8 research phases are inputs to those later specifications, not independently authorized task ladders. Verified Learning L0–L3 remains a separate future formal program and may be scheduled only by a later accepted governance decision based on measured need.

## 8. Entry acceptance gate

This entry gate may land only if the exact final candidate proves:

- changed scope is governance/research documentation only;
- canonical base still descends from the Spec 006 T086 closeout;
- no production source, dependency, lockfile, migration, workflow-semantic, runtime, provider, browser, daemon, IPC, or authority mutation;
- repository `quality` succeeds on the exact final head;
- correctness/governance/evidence-integrity review passes;
- Ponytail/YAGNI review finds no unjustified scope;
- an independent reviewer challenges the exact candidate and roadmap reconciliation;
- zero unresolved material findings/threads;
- final base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head landing;
- post-merge canonical `main`/tree verification.

Only after canonical landing may repository truth state:

```text
SPEC_006=CLOSED_CANONICAL_IMPLEMENTATION_PROGRAM
SPEC_007_FORMAL_SPEC_AUTHORIZED=YES
SPEC_007_PLAN_AUTHORIZED=NO
SPEC_007_TASKS_AUTHORIZED=NO
SPEC_007_IMPLEMENTATION_AUTHORIZED=NO
PERSISTENT_OWNER_IPC_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
BROWSER_RUNTIME_AUTHORIZED=NO
PROVIDER_MESH_IMPLEMENTATION_AUTHORIZED=NO
LEARNING_IMPLEMENTATION_AUTHORIZED=NO
```
