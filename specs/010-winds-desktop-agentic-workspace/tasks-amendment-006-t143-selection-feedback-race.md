# Spec 010 Tasks Amendment 006 — T143 Qualification-Discovered Selection Feedback Race Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 010 T143 qualification authority, canonical T134 exact-session targeting and dual-session race requirements, canonical T141 adversarial focus-race requirements, canonical Amendments 004/005 qualification-discovered repair precedent, and the active Founder directive to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Material evidence

Forward-integrated exact head `37c0b61db433f2900c9863544c930cec8562696f` ran first-attempt T143 performance qualification as run `35169257376`. All non-T143 workflows on that exact head passed. T143 native qualification also passed every native gate, but the authoritative renderer campaign failed at `normal:selection`, sample index `0`, while waiting for the exact selection to commit across both Chat and the focused Workbench Slot.

Artifact `10475704463` (`sha256:991514b990b892177c2bcdd1c3bac61edc67f03bf226565bfe00ee7005a5367e`) retained a valid renderer preflight and a diagnostic probe in which six alternating Chat selector changes committed correctly. The stronger authoritative predicate nevertheless proved that Chat selection can commit while the Workbench still projects the previous focused Session.

Code inspection identifies the bounded production race: `App` recreates the `focusDock` callback on every render, while `DualSessionWorkspace` legitimately includes `onFocusSession` in its synchronization effect dependencies. A parent render caused by selecting Session B can therefore change callback identity before the Workbench selection effect has committed B, causing the Workbench focus effect to replay the previously focused Session A back into parent selection state.

## Authorized repair scope

This amendment authorizes one bounded forward-only repair and direct regression coverage:

- `desktop/src/App.tsx`: stabilize existing exact-session navigation/focus callbacks so callback identity changes do not manufacture a new focus event;
- `desktop/tests/dual-session-model.test.mjs` and/or another existing desktop deterministic test file: directly assert stable callback wiring / no stale focus replay seam;
- `docs/provenance/010-t143-performance.md`: record the qualification-discovered defect, exact repair identity, and post-repair qualification disposition.

No other production path is authorized by this amendment unless exact-head review proves it is mechanically required for the same callback-identity race and remains behavior-preserving outside that race.

## Non-negotiable boundaries

The repair MUST NOT:

- change exact Session targeting semantics, dual-session layout semantics, right-dock binding semantics, runtime authority, terminal authority, verification authority, or approval authority;
- weaken T143 thresholds, sample floors, committed-selection predicates, security gates, accessibility gates, or platform gates;
- add a dependency, command, IPC surface, generic dispatcher, background authority, or persistence path;
- suppress Workbench-to-parent focus synchronization when the focused Session genuinely changes;
- reinterpret either failed T143 head as qualified or rerun either failed head to green.

## Required proof

The repair successor must prove on its exact head:

1. deterministic frontend tests cover stable parent callback identity or an equivalent direct stale-focus replay invariant;
2. existing T134 dual-session race and exact-target tests remain green;
3. all ordinary repository workflows triggered by the repair PR are green;
4. after guarded merge, all workflows triggered by the merge commit are green;
5. T143 is forward-integrated onto the canonical repair merge and starts qualification from a new exact head;
6. the first T143 attempt on that new head must satisfy the unchanged full-workbench committed-selection predicate or remain failed evidence.

## Closure discipline

This amendment becomes canonical only after guarded expected-head merge and successful post-merge workflows. T143 remains blocked while the amendment or its authorized repair is not canonically closed. Closing this amendment authorizes only the bounded repair above; it does not close T143, authorize T144, or substitute for any later Founder visual acceptance.
