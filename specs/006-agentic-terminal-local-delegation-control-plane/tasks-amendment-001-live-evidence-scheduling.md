# Spec 006 Tasks Amendment 001 — Live-Evidence Scheduling

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Constitution 1.1.0 Governance deviation/amendment process and Founder governance decision recorded on PR #80 in issue comment `5551657392`.

## Purpose

Spec 006 intentionally requires real local Codex, Claude, and Worker execution evidence for selected acceptance claims. The repository implementation for those surfaces can be deterministically implemented, tested, reviewed, and landed even when the designated physical runtime is temporarily unavailable to the execution session performing repository work.

The original task graph couples repository implementation scheduling to external runtime availability. That coupling can indefinitely freeze later deterministic implementation work without increasing the truthfulness of the required live evidence. This amendment removes only that scheduling deadlock. It does not reduce, replace, synthesize, or waive any live-runtime evidence requirement.

## Precedence and Scope

When this amendment becomes canonical, it is part of the Spec 006 Tasks package and supersedes only the dependency-scheduling statements in `tasks.md` that conflict with this document for T079, T080, and the real-runtime portion of T082.

All other `tasks.md` requirements remain unchanged, including every safety boundary, authorized-path boundary, protocol pin, exact-output requirement, review requirement, evidence-integrity rule, and non-authorization statement.

If this amendment and `tasks.md` can be read consistently, both apply. If they conflict only on whether missing physical-runtime evidence blocks the next repository implementation slice, this amendment controls.

## Two Evidence Lanes

For T079, T080, and the real-runtime portion of T082, maintain two distinct evidence lanes.

### IMPLEMENTATION_LANE

The implementation lane requires, on the exact candidate:

- only task-authorized implementation and deterministic test surfaces;
- repository `quality` SUCCESS and every other applicable deterministic/platform CI gate;
- focused deterministic tests registered and demonstrably executed;
- author correctness/safety/evidence-integrity review;
- Ponytail/YAGNI review;
- fresh independent substantive review bound to the exact candidate, or an explicitly valid review stack whose final delta reaches it;
- zero unresolved material findings;
- exact changed-file/scope reconciliation;
- no unauthorized dependency, runtime, protocol, credential, access, or authority expansion;
- guarded expected-head landing;
- post-merge canonical main/tree verification before the next implementation slice begins.

A successfully landed implementation lane may satisfy the repository-implementation dependency for the next task.

### LIVE_EVIDENCE_LANE

The live-evidence lane remains exactly the real-runtime acceptance evidence already specified by the owning task.

No implementation-lane result, CI result, mock, fixture, generic Linux container, GitHub-hosted runner, historical receipt, Agent claim, newly manufactured authentication route, newly provisioned remote execution route, or reviewer statement may substitute for live evidence.

Missing access to an already-qualified designated runtime is recorded as:

```text
LIVE_EVIDENCE_DEFERRED_EXTERNAL
```

It is not recorded as PASS.

## T079 Scheduling Amendment

T079 remains the first task allowed to send a real Codex prompt under its existing safety and one-shot rules.

The T079 implementation may be landed after `T079_IMPLEMENTATION_LANE=PASS` even when the designated real local Codex runtime is unavailable.

After guarded landing and post-merge verification:

```text
T079_IMPLEMENTATION_LANE=LANDED
T079_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
```

That state satisfies the implementation-order dependency for starting the T080 implementation slice. It does not mean original T079 live acceptance is closed.

The exact T079 live contract remains unchanged. A live attempt still requires separately valid attempt-time authority and a qualifying governed runtime. Any candidate/head movement invalidates candidate-bound live authorization as already specified.

## T080 Scheduling Amendment

T080 remains the first task allowed to send a real Claude prompt under its existing safety boundary.

The T080 implementation may be developed and landed after the T079 implementation lane is canonically landed, without requiring T079 live PASS first.

If a qualifying real Claude runtime is unavailable, T080 must remain:

```text
T080_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T080_LIVE_PASS=NO
```

A landed T080 implementation lane satisfies the implementation-order dependency for T081. It does not establish real Claude execution, native-session provenance, or any runtime capability claim.

## T081

T081 remains deterministic contract/handoff work. It may begin only after the T080 implementation lane is canonically landed and post-merge verified.

T081 does not convert deferred T079 or T080 live evidence into PASS and must preserve `Agent completion != verification/acceptance`.

## T082 Scheduling Amendment

T082 contains both deterministic implementation and a real Codex Worker edit acceptance requirement.

Its deterministic implementation lane may be developed and landed after T081 is canonically landed.

If the qualifying governed Worker runtime is unavailable, record:

```text
T082_WORKER_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T082_WORKER_LIVE_PASS=NO
```

A landed T082 implementation lane satisfies the implementation-order dependency for T083. It does not establish that a real Worker edit occurred.

The original T082 rules remain unchanged: exact Winds-owned worktree identity, explicit approval, bounded authority, no primary-checkout mutation, no automatic merge/push/PR, dirty/failed state preservation, and Agent completion remaining `AGENT_REPORTED` until independent Git/evidence observation.

## T083–T085

T083, T084, and T085 continue in their existing dependency order using canonically landed implementation lanes as predecessor implementation dependencies.

They must preserve deferred live-evidence state explicitly and may not promote deferred or historical runtime evidence into current PASS.

Platform/runtime claims in T085 remain limited to actual evidence.

## T086 Final Reconciliation

T086 must reconcile both implementation and live-evidence lanes separately.

T086 may close the repository implementation program only when T070–T085 implementation requirements are canonically reconciled and all deterministic/review/landing gates pass.

T086 MUST NOT claim a live Codex, Claude, or Worker capability whose required live-evidence lane is still deferred.

If any live-evidence lane remains deferred, final state must distinguish repository implementation completion from live-capability acceptance, for example:

```text
SPEC_006_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
```

The original `SPEC_006_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL` marker may be used only for implementation-program closure and must be accompanied by explicit live-evidence state when any live lane is deferred.

A release, README, status page, or product claim may not represent a deferred live capability as proven. If final release acceptance requires those live capabilities, that release acceptance remains blocked until genuine evidence exists or a separately reviewed canonical scope amendment explicitly removes the capability from the release claim.

## Revised Implementation Dependency Chain

After this amendment is canonical:

```text
T078
 -> T079_IMPLEMENTATION_LANE
 -> T080_IMPLEMENTATION_LANE
 -> T081
 -> T082_IMPLEMENTATION_LANE
 -> T083
 -> T084
 -> T085
 -> T086
```

The live-evidence lanes remain separately open until genuinely satisfied:

```text
T079_LIVE_EVIDENCE_LANE
T080_LIVE_EVIDENCE_LANE
T082_WORKER_LIVE_EVIDENCE_LANE
```

## Explicit Non-Authorization

This amendment does not authorize:

- fabrication, inference, or reuse of stale live evidence;
- installing or updating Codex/Claude solely to manufacture acceptance evidence;
- credential read/copy/printing, login automation, terms acceptance, access/billing escalation, or synthetic authenticated homes;
- generic remote execution, a new remote-control path, MCP runtime, ACP v2, daemon/public IPC, plugin host, recursive fleet, or model gateway;
- primary-checkout mutation by an Agent;
- automatic winner selection, acceptance, merge, push, PR creation, force-clean, rebase, cherry-pick, or autonomous landing;
- weakening T079 one-shot semantics;
- treating deterministic CI as physical-runtime proof;
- treating an implementation-lane landing as a live-runtime PASS.

## Migration / Compatibility Impact

- Repository implementation scheduling changes only for the three explicitly identified real-runtime surfaces.
- Existing implementation, historical evidence, live receipts, and failed/consumed attempts remain historical and unchanged.
- Existing protocol/safety/security acceptance rules remain unchanged.
- Existing exact-head qualification becomes stale on any candidate that adds this amendment and must be rerun from scratch.
- Any previous T079 exact-head live authorization is invalid after candidate/head movement and must not be reused on the amended head.

## Acceptance of This Amendment

This amendment is not canonical merely because the Founder decision or this candidate file exists.

Before it can govern scheduling, the exact candidate containing it must satisfy the repository Standard Acceptance Gate, including deterministic CI, correctness/safety/evidence-integrity review, Ponytail/YAGNI review, fresh independent substantive exact-head review, zero unresolved material findings, exact scope reconciliation, guarded expected-head landing, and post-merge canonical verification.
