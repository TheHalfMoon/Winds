# T122 Workbench Model Mesh Necessity Decision

Status: `WORKBENCH_MODEL_MESH_NECESSARY=NO`

## Exact canonical basis

```text
T121_FINAL_CANDIDATE=6091d847207cabce3723815e62e3c85ae563f1a8
T121_FINAL_TREE=926d2526cccc711febafa5f1ad87e919a55fc4cf
T121_MERGE=9951e1c460815b818cefc6a7ae67fa1e0a7949fe
T121_MERGE_PARENTS=[2851cb3cb76754fa1033ccd83f58d706284d947e,6091d847207cabce3723815e62e3c85ae563f1a8]
T121_MERGE_SIGNATURE=VERIFIED_VALID
T121_EXACT_HEAD_QUALITY=SUCCESS_ATTEMPT_1
T121_EXACT_HEAD_WINDOWS_TERMINAL=SUCCESS_ATTEMPT_1
T121_EXACT_HEAD_RELEASE_CANDIDATE=SUCCESS_ATTEMPT_1
T121_EXACT_HEAD_T097_PERFORMANCE=SUCCESS_ATTEMPT_1
T121_POST_MERGE_QUALITY=SUCCESS_ATTEMPT_1
T121_POST_MERGE_WINDOWS_TERMINAL=SUCCESS_ATTEMPT_1
T121_UNRESOLVED_REVIEW_THREADS=0
```

T122 evaluates only whether a read-only Workbench Model Mesh rendering is necessary after canonical T121 CLI qualification. It does not evaluate whether a Workbench presentation would be more convenient, attractive, discoverable, or eventually useful.

The canonical Plan requires the Model Mesh layer to be exposed first through narrow existing-style surfaces and states that Workbench/TUI may later render the same read-only projections. The recommended sequence selects Workbench rendering only if it remains necessary after CLI qualification.

## Decision rule

A Workbench Model Mesh surface is necessary only if a current Spec 009 operator requirement cannot be satisfied through the canonically landed Model Mesh domain, persistence, projection, and T121 CLI seams without creating a second authority path.

Presentation preference, reduced typing, visual grouping, pane proximity, dashboard ergonomics, or future discoverability do not satisfy this necessity threshold.

## Required first-user capability reconciliation

| Canonical Plan capability | Canonical T121 surface | T122 result |
| --- | --- | --- |
| Inspect admitted runtime target availability without executing | `model-mesh availability` | `SATISFIED_WITHOUT_WORKBENCH` |
| Submit or record an explicit stage-bound target request | `model-mesh request` through the existing exact content-bound HUMAN persistence path | `SATISFIED_WITHOUT_WORKBENCH` |
| Inspect target resolution and why blocked | `model-mesh status` and `model-mesh why-blocked` | `SATISFIED_WITHOUT_WORKBENCH` |
| Inspect source-labelled requested, declared, observed, and agent-reported identity truth | `model-mesh status` plus canonical stored identity-claim projection | `SATISFIED_WITHOUT_WORKBENCH` |
| Inspect continuity class, drift/staleness, material context loss, and source/destination binding | `model-mesh continuity` plus canonical continuity projection fields | `SATISFIED_WITHOUT_WORKBENCH` |
| Preserve an earlier failed/unavailable request while recording a later explicit alternate request | append-only `model-mesh request` behavior; no historical rewrite | `SATISFIED_WITHOUT_WORKBENCH` |
| Generate reviewer continuity context without source-agent persuasion authority | `model-mesh reviewer` | `SATISFIED_WITHOUT_WORKBENCH` |

No required first-user capability depends on pane geometry, persistent UI state, pointer interaction, Workbench-only storage, or a Workbench-only observation source.

## Spec 009 operator-truth reconciliation

### User Story 8 / FR-068 through FR-070

The required operator surface is a concise non-mutating projection of requested target, observed/source-labelled identity truth, workflow/stage binding, continuity result, drift/staleness, material context loss, authority ceiling, and blocking reason.

Canonical T121 provides the required inspection split:

- `availability` inspects durable local actor/runtime target truth and explicitly reports that live runtime discovery, provider execution, and credential operations were not performed;
- `status` projects the exact stored request, source-labelled identity claims, durable runtime binding, resolution state, drift fields, authentication readiness, blocker list, policy path, execution-authority nonclaim, and verification/acceptance/landing nonclaims;
- `why-blocked` preserves the status projection and identifies a primary blocker without adding mutation or authority;
- `continuity` projects append-only continuity history and explicitly reports that history was not rewritten;
- `reviewer` projects exact target/continuity context while keeping current candidate/evidence freshness, verification, and human acceptance unavailable or unknown when not supplied.

T121 inspection actions require an existing initialized canonical Store, use the qualified read-only immutable SQLite inspection path, reject active SQLite sidecars, validate canonical workflow and Model Mesh schema definitions, validate bounded StageRun lineage, and apply SQL-level bounded result sentinels before referenced records are materialized. The final independent review found no inspection mutation or authority-expansion path.

A Workbench rendering would consume the same already-qualified projections. It would not make any currently required canonical truth newly observable.

### FR-069 negative-state visibility

The current Model Mesh domain/projection seam distinguishes the required negative-state categories. The T121 CLI preserves explicit blocker and unavailable/unknown truth rather than silently substituting another target or manufacturing current live authority.

The absence of a Workbench rendering does not collapse or hide those canonical states; the deterministic JSON projection remains operator-facing and inspectable.

### FR-071 through FR-074 usage/cost truth

T122 does not need a Workbench surface to satisfy optional usage/cost visibility. T125 separately owns the decision whether an already-authorized structured local usage source can be qualified. Until then, usage/cost remains `UNKNOWN` where no qualified source exists.

A Workbench surface cannot create a trustworthy observation source and cannot be used to justify a billing API, pricing database, provider SDK, external telemetry service, guessed token count, or guessed price.

## Spec 007 preservation

No current Spec 009 requirement requires extending the existing Spec 007 Workbench lifecycle, navigation, keyboard, rendering, accessibility, or platform surfaces.

Selecting a new Workbench rendering now would create additional presentation integration that would itself need to preserve:

- canonical identity versus display identity;
- source-labelled truth versus terminal/model prose;
- stale versus current evidence applicability;
- lifecycle and host-safety boundaries;
- keyboard/findability/accessibility behavior for every newly claimed UI operation;
- directly exercised platform claims only.

Those obligations are valid if a future concrete operator gap proves a Workbench surface necessary. They are not evidence that such a surface is currently necessary.

## YAGNI and authority boundary

Adding Workbench behavior would duplicate an already-qualified read-only projection surface without closing a current requirement gap. It would also introduce another place where presentation state could accidentally be confused with canonical target, continuity, authority, verification, or acceptance truth.

T122 therefore selects the smaller canonical result: record the evidence-backed no-need decision and make no Workbench source change.

```text
WORKBENCH_MODEL_MESH_NECESSARY=NO
T122_PRODUCTION_SOURCE_CHANGE=NO
T122_WORKBENCH_SOURCE_CHANGE=NO
T122_MAIN_RS_CHANGE=NO
T122_FOCUSED_IMPLEMENTATION_TEST=NOT_APPLICABLE_DOCUMENTATION_ONLY
T122_RUNTIME_CHANGE=NO
T122_PROVIDER_EXECUTION=NO
T122_CREDENTIAL_OPERATION=NO
T122_NEW_DEPENDENCY=NO
T122_NEW_PERSISTENCE=NO
T122_NEW_MUTATION_PATH=NO
T122_SECOND_MODEL_MESH_AUTHORITY=NO
T122_MIGRATION_0011_CHANGE=NO
```

## Live-runtime and platform nonclaims

T122 changes no live-runtime or platform truth. In particular, it does not convert deterministic persistence/projection evidence into real provider execution or native resume proof.

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

No new native Windows, WSL2, Linux, or macOS Model Mesh behavior is claimed by this documentation-only decision.

## Historical evidence discipline

The historical T121 `quality` attempt-1 failure remains material evidence. It was resolved through the canonical Spec 009 Tasks Amendment 001 plus authorized terminal fixture repair, not by relabelling or erasing the failed attempt. T122 does not alter that history.

All predecessor T121 independent-review findings were repaired forward-only before the final accepted T121 head. T122 does not weaken or reinterpret those findings.

## Acceptance reconciliation

- T121 is canonically closed and post-merge verified before this necessity decision.
- Every current Spec 009 first-user/operator capability is reconciled against a landed T121 CLI/projection seam.
- User Story 8 and FR-068 through FR-070 require operator-facing non-mutating truth, not a Workbench-specific presentation.
- FR-071 through FR-074 do not create a Workbench requirement; optional usage/cost source qualification remains owned by T125.
- No concrete unmet current Spec 009 requirement was found that requires Workbench rendering.
- No Workbench source, production source, runtime, provider execution, dependency, persistence, migration, or mutation path is introduced.
- Existing Spec 007 Workbench lifecycle, host-safety, navigation, accessibility, and platform truth remains unchanged.
- The evidence-backed result is therefore `WORKBENCH_MODEL_MESH_NECESSARY=NO`.

## Decision

```text
T122_DECISION=WORKBENCH_MODEL_MESH_NECESSARY_NO
T122_UNMET_CURRENT_SPEC_009_WORKBENCH_REQUIREMENT=NONE_PROVEN
T122_TASKS_AMENDMENT_REQUIRED=NO
T123_AUTHORITY=BLOCKED_UNTIL_T122_CANONICAL_LANDING
```

T122 may close only through the repository Standard Acceptance Gate on this exact documentation-only candidate. Until guarded landing and post-merge verification complete, this document is candidate evidence and does not authorize T123.
