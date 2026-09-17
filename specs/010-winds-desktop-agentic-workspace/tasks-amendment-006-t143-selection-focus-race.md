# Spec 010 Tasks Amendment 006 — T143 Selection/Workbench Focus Race Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation process, canonical Spec 010 T134 exact-Session/dual-Slot semantics, canonical T142A exact Session focus presentation, canonical T143 qualification authority, and the active Founder directive to continue without fabricating, suppressing, or rerunning away material evidence.

## Purpose

T143 exact-head performance qualification exposed a production correctness race in the already-closed desktop selection/focus path. The performance harness is not authorized to repair closed T134/T142A product behavior by itself. This amendment authorizes one bounded production repair so an exact Chat Session selection cannot be overwritten by stale Workbench focus propagation while asynchronous selection reconciliation is still in flight.

This amendment does not change any performance threshold, sample floor, runtime authority, terminal behavior, security boundary, accessibility requirement, or T144 human visual acceptance requirement.

## Material evidence

Forward-integrated T143 exact head `37c0b61db433f2900c9863544c930cec8562696f` produced first-attempt performance run `35169257376` and artifact `10475704463` with digest `sha256:991514b990b892177c2bcdd1c3bac61edc67f03bf226565bfe00ee7005a5367e`.

Native qualification passed every frozen native gate:

```text
cold launch p95 = 1057.8935 ms <= 1500 ms
idle CPU = 0.56447% <= 2%
renderer+host idle RSS max = 315.484 MiB <= 320 MiB
renderer present every idle sample = true
renderer+host process count >= 2 every idle sample = true
```

Renderer preflight was valid, and the isolated diagnostic probe completed all six alternating A/B selector changes. The authoritative campaign nevertheless failed immediately:

```text
phase = normal:selection
sampleIndex = 0
error = timed out waiting for committed UI state
```

The strengthened T143 predicate required the selected Session to be committed simultaneously in the controlled Chat selector, visible Chat identity, and the focused Workbench Slot. The failure therefore proves the existing product path can expose Chat selection before Workbench focus commits to the same exact Session.

## Root cause

`desktop/src/App.tsx` creates `focusDock` as a new function on every render and passes it to `DualSessionWorkspace` as `onFocusSession`.

`DualSessionWorkspace` has an effect that propagates the currently focused Slot back through `onFocusSession`. Because the callback identity changes when App renders a new external Chat selection, that effect can run again while the Workbench still reflects the previous focused Slot. It can therefore write the previous Session back into App state before the asynchronous selection-reconciliation effect commits the requested Session.

This is a product correctness race in callback/effect identity, not a T143 threshold failure and not evidence for weakening the committed-selection predicate.

## Exact repair authority after canonical landing

Only after this amendment is `CLOSED_CANONICAL` may a repair candidate modify:

- `desktop/src/App.tsx` solely to stabilize the selection/focus callback chain required by `DualSessionWorkspace`;
- existing desktop deterministic test files, or one narrowly named new desktop test file, solely to prove the race is closed;
- a bounded provenance note for the repair.

The preferred repair is to use React `useCallback` for `focusDock` and only the directly dependent callback chain (`openDockIntent`, `selectSession`, `selectProjectSession`) as required to keep callback identity stable across unrelated App renders. No new state machine, queue, dependency, timer, retry, sleep, or authority seam is authorized unless exact evidence proves the preferred repair insufficient.

## Required behavior

The final repair must prove all of the following:

- selecting Session B from Chat while Workbench currently focuses Session A cannot be reverted to A merely because App re-rendered;
- Workbench reconciliation still focuses the exact selected Session when that Session already occupies either visible Slot;
- Workbench-originated focus changes still propagate back to Chat/right-dock binding after actual Slot focus changes;
- no selection broadcast or cross-Session dispatch authority is introduced;
- no existing T134 dual-Slot identity, T136 right-dock binding, T138 navigation, T140 accessibility, T141 security, or T142A presentation contract is weakened;
- T143's `>=1000` selection sample floor and `<=50 ms p95` budget remain unchanged.

## Explicit non-authorization

This amendment does not authorize changes to:

- `desktop/src/dualSession/DualSessionWorkspace.tsx` unless the preferred App callback-stability repair is proven insufficient by new exact evidence;
- Session layout persistence semantics;
- terminal/runtime/provider execution authority;
- right-dock trust or binding semantics;
- any T143 threshold, sample count, timeout, process scope, or measurement definition;
- dependencies, lockfiles, schemas, migrations, runner images, or workflow retry policy;
- security or accessibility gates;
- T144 Founder human visual acceptance.

## Required repair evidence after amendment landing

The repair must move forward-only from then-current canonical `main` and requalify from scratch. Required evidence includes:

- changed production scope is exactly the bounded callback-identity repair above plus focused tests/provenance;
- `git diff --check` PASS;
- desktop format/typecheck/lint PASS;
- complete desktop tests PASS;
- production frontend build PASS;
- Rust fmt/clippy and applicable repository gates PASS;
- all actually-triggered exact-head workflows SUCCESS;
- focused deterministic regression proves Chat B selection cannot be overwritten by stale Workbench A propagation;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- guarded expected-head normal merge;
- verified merge tree, ordered parents, and GitHub signature;
- every actually-triggered post-merge workflow SUCCESS.

The failed T143 runs remain historical evidence and MUST NOT be rerun-to-green on their stale heads.

## Amendment acceptance gate

This governance-only amendment is not canonical merely because it exists. Its exact final candidate must satisfy:

- changed scope exactly this one amendment document;
- repository `quality` SUCCESS on Ubuntu and macOS;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- exact current main/base/head/tree/scope/mergeability reconciliation;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered applicable post-merge workflow SUCCESS.

Only after those gates may repository truth state:

```text
SPEC_010_TASKS_AMENDMENT_006=CLOSED_CANONICAL
T143_SELECTION_FOCUS_RACE_REPAIR=AUTHORIZED_BOUNDED_CALLBACK_IDENTITY_ONLY
T143_PERFORMANCE_THRESHOLDS=UNCHANGED
T143=BLOCKED_UNTIL_SELECTION_FOCUS_RACE_REPAIR_CLOSED_CANONICAL
T144=BLOCKED_UNTIL_T143_CLOSED_CANONICAL
```
