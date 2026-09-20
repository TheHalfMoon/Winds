# Spec 011 T160 — Native Platform Qualification

## Purpose

T160 directly qualifies the accepted Spec 011 persistent-runtime implementation on Linux, macOS, and native Windows without substituting evidence across operating-system domains.

T160 is qualification-only. It introduces no new runtime authority, transport, terminal backend, persistence schema, dependency, resource ceiling, protocol message, provider/model execution path, service installation, or remote-control surface.

## Canonical predecessor

T159 closed canonically at merge commit:

```text
9806deeba3ff493b173b8e4a2b90e000bac32860
```

Its merge tree is identical to the accepted T159 exact-head tree:

```text
56f5eec96b96a782faf6abc586616ee7ea6bc0dc
```

The ordered merge parents are the canonical T159 base followed by the accepted T159 candidate:

```text
2e1e81e96046ce6027702c4b92dfe22a7a3b653e
281673d8cbef0e2607efc60c67070e3e64cb8fae
```

All five actually-triggered T159 post-merge push workflows succeeded on that merge commit:

- `t159-performance` — run `35494672816`
- `quality` — run `35494672890`
- `t142-native-platform` — run `35494672777`
- `windows-terminal` — run `35494672819`
- `t141-desktop-security` — run `35494672783`

## Frozen T160 claim matrix

### Linux

Direct exact-candidate evidence must cover:

- POSIX Unix-socket parent/path ownership and mode;
- kernel peer-credential same-user proof and wrong-UID fail-closed fixture;
- OS entropy seam;
- PTY detach/reattach and detached exit;
- observer replay/backpressure;
- controller authority;
- owner crash/recovery;
- resource cleanup and reconnect/high-output campaign.

### macOS

Direct exact-candidate evidence must independently cover:

- POSIX Unix-socket parent/path behavior on macOS;
- macOS peer-identity mechanism;
- bounded path/fallback behavior;
- macOS entropy seam;
- PTY continuity;
- observer/controller semantics;
- lifecycle/recovery;
- resource cleanup.

Linux evidence cannot qualify macOS.

### Native Windows

Direct exact-candidate evidence must cover:

- explicit named-pipe security descriptor/DACL;
- current-user SID access;
- actual anonymous-principal denial fixture;
- entropy seam;
- same-generation collision and wrong-generation fail-closed behavior;
- ConPTY lifecycle;
- persistent detach/reattach, input/output, resize, exit, and stop;
- preserved fail-closed native-Windows interrupt limitation;
- observer/controller semantics;
- owner crash/recovery/reconnect;
- resource cleanup.

Linux/macOS evidence cannot qualify native Windows.

### WSL

Spec 011 does not introduce a Windows-host-to-WSL persistent-owner bridge.

The existing WSL execution-domain evidence does not automatically qualify a WSL persistent owner. T160 therefore freezes:

```text
SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED
WINDOWS_HOST_TO_WSL_OWNER_BRIDGE=NOT_INTRODUCED
```

This is a deliberate truthful nonclaim, not missing evidence relabelled as success.

## Exact workflow

The `t160-native-platform` workflow runs one direct job for each domain:

- `ubuntu-24.04` -> `linux`
- `macos-15` -> `macos`
- `windows-2025` -> `windows`

Every job is bound to the exact candidate SHA and runs:

1. Rust formatting and Clippy gates.
2. The complete `persistent_runtime::` test surface serially on that operating system.
3. Explicit assertions that the required domain-specific T149 or T150 security fixtures actually executed and passed.
4. A 100-cycle persistent-owner detach/exact-generation reattach churn campaign on that same operating system.
5. Direct PTY or ConPTY backend tests.
6. The T063 100-cycle terminal lifecycle soak.
7. The T063 active-close/resize guard.
8. Linux additionally reruns the release-profile T159 core resource campaign on the exact T160 candidate.
9. An exact-domain JSON evidence artifact containing commit/tree/runner/domain identity and explicit non-substitution/non-relaxation flags.

The matrix uses `fail-fast: false` so one platform cannot hide another platform's result.

## Evidence boundaries

The T160 workflow reuses accepted implementation tests rather than duplicating platform primitives.

The primary direct fixtures are:

- T149 for Linux/macOS private Unix transport, path/mode/peer identity, stale-path handling, and entropy.
- T150 for native-Windows named-pipe DACL/current-user/anonymous-denial/collision/entropy behavior.
- T152 for persistent terminal detach/reattach, detached exit, generation binding, native-Windows interrupt non-support, and the T160 100-cycle exact-generation reconnect churn.
- T153 for replay bounds and slow-client backpressure.
- T154 for controller authority and lease behavior.
- T157 for crash/restart/recovery truth.
- T063 for repeated terminal lifecycle cleanup and active-close/resize resource behavior.
- T159 core for Linux high-output, observer scale, runtime scale, and reconnect churn.

No fixture result from one operating system is used as direct evidence for another.

## Acceptance boundary

T160 closes only when one exact candidate has all of the following:

1. all three `t160-native-platform` matrix jobs successful;
2. exact candidate SHA/tree binding in each uploaded domain artifact;
3. Linux T149/T152/T153/T154/T157/T063/T159 evidence successful;
4. macOS T149/T152/T153/T154/T157/T063 evidence successful, including 100 persistent-owner reconnect cycles;
5. native-Windows T150/T152/T153/T154/T157/T063 and ConPTY evidence successful, including 100 persistent-owner reconnect cycles;
6. `SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED` remains explicit;
7. no threshold, security boundary, correctness gate, or authority surface weakened;
8. exact changed-file and Alibaba OpenCodeReview delegation/rule accounting reconciled;
9. zero unresolved material review findings;
10. exact base/head/tree/mergeability reconciliation immediately before landing;
11. guarded normal merge with exact expected-head binding;
12. merge tree/ordered parents/signature metadata reconciled;
13. every actually-triggered post-merge push workflow succeeds.

Only canonical T160 closure authorizes T161.
