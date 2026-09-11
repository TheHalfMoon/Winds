# T110 Workbench/TUI Necessity Decision

Status: `TUI_NOT_REQUIRED_FIRST_SLICE`

## Exact canonical basis

```text
T109_MERGE=d13e2931cfb39cbc0f48b8602904873c47cd350a
T109_TREE=51766359c9493fba1ff0f9786f340720ebc2d103
T109_POST_MERGE_QUALITY=SUCCESS_ATTEMPT_1
T109_POST_MERGE_WINDOWS_TERMINAL=SUCCESS_ATTEMPT_1
```

T110 evaluates whether any current Spec 008 P1 operator requirement remains impossible through the qualified workflow domain, Store, projection, and CLI seams after T109 canonical closure. It does not evaluate whether a TUI would be more convenient or visually preferable.

The canonical Plan requires CLI-first exposure and permits a later read-only Workbench/TUI projection only if the qualified CLI is demonstrably insufficient for a current requirement. T110 authorizes no production/runtime edit by default.

## Decision rule

A TUI is necessary only if a current Spec 008 requirement needs operator-visible workflow truth that cannot be obtained or exercised through the canonically landed T101-T109 seams without creating a second authority path.

Presentation preference, discoverability preference, reduced typing, visual grouping, dashboards, or future product ergonomics do not satisfy this necessity threshold.

## P1 reconciliation

| P1 capability | Canonical first-slice seam | T110 result |
| --- | --- | --- |
| Return to exact workflow/stage truth | `workflow open` plus `workflow status` | SATISFIED_WITHOUT_TUI |
| Inspect exact candidate/artifact freshness | T103 persisted baseline truth projected by `workflow status` / `reviewer-handoff`, with optional exact `--candidate-oid` + `--candidate-tree` context | SATISFIED_WITHOUT_TUI |
| See bounded retry/no-progress/recovery truth | `record-failure`, `retry`, `recover`, and status projection retry fields | SATISFIED_WITHOUT_TUI |
| Inspect append-only decisions | `decision-record` and `decision-list`, with source/authority and candidate/evidence applicability preserved | SATISFIED_WITHOUT_TUI |
| Distinguish resume/reconstruction/ownership loss | `resume-preview` plus actor/reconstruction projection input where canonical binding truth exists | SATISFIED_WITHOUT_TUI |
| Produce reviewer-independent handoff | `reviewer-handoff` from T107 canonical projections | SATISFIED_WITHOUT_TUI |
| Preserve redaction/completeness truth | T108 durable-state rules surfaced through status/handoff decision and reconstruction content-state projections | SATISFIED_WITHOUT_TUI |
| Explain current state and blockers | `status`, `resume-preview`, and `why-blocked` | SATISFIED_WITHOUT_TUI |

The Plan's explicit first user capabilities are all represented by the landed T109 command surface: create/open, prepare/start, bounded lifecycle movement, status, resume preview, why-blocked, decision record/query, reviewer handoff, and explicit retry/recovery.

T103/T104 canonical Store state includes artifact-baseline and actor/reconstruction records that the T107/T109 projections inspect. T109 does not create a generic CLI mutation command for every underlying canonical record type, and the Plan does not require such a command as a first-slice user capability. A future proven need for a new mutation surface would require its own authority decision; a read-only TUI cannot be used to manufacture that authority.

## Necessity test

No current P1 requirement requires spatial layout, pane state, mouse interaction, persistent UI state, or a Workbench-only observation source. The required operator truths are structured and already available through deterministic CLI JSON projections or the qualified Store/domain seams beneath them.

A Workbench/TUI could improve convenience, grouping, or discoverability, but those are presentation preferences. They do not prove that a current canonical requirement is impossible through the landed CLI/projection surface.

Adding a TUI now would also create avoidable scope: another presentation integration, another place to police stale identity and authority labels, and a risk of accidentally persisting or mutating presentation-derived truth. None of that is required to satisfy the current first slice.

## Authority and nonclaims

```text
TUI_REQUIRED_FOR_CURRENT_P1=NO
TUI_NOT_REQUIRED_FIRST_SLICE=YES
T110_PRODUCTION_SOURCE_CHANGE=NO
T110_RUNTIME_CHANGE=NO
T110_NEW_DEPENDENCY=NO
T110_NEW_PERSISTENCE=NO
T110_NEW_MUTATION_PATH=NO
T110_SECOND_WORKFLOW_AUTHORITY=NO
```

This decision does not prohibit a future read-only Workbench presentation. It states only that such integration is not currently necessary and therefore is not authorized by T110. Any later concrete requirement must first be proven against then-current canonical CLI/projection capability and receive an explicit Tasks amendment defining the exact source slice.

## Acceptance reconciliation

- Every P1 operator capability is reconciled against the canonically landed T101-T109 domain/Store/projection/CLI seams.
- Presentation preference alone is explicitly rejected as a necessity signal.
- No production/runtime source, durable UI state, dependency, mutation path, or second workflow authority is introduced.
- No concrete unmet current Spec 008 requirement was found that requires Workbench/TUI integration.
- The default/no-need result therefore remains `TUI_NOT_REQUIRED_FIRST_SLICE`.

## Decision

```text
T110_DECISION=TUI_NOT_REQUIRED_FIRST_SLICE
T110_UNMET_CURRENT_SPEC_008_TUI_REQUIREMENT=NONE_PROVEN
T110_TASKS_AMENDMENT_REQUIRED=NO
T111_AUTHORITY=BLOCKED_UNTIL_T110_CANONICAL_LANDING
```

T110 may close only through the repository Standard Acceptance Gate on this exact documentation-only candidate. Until guarded landing and post-merge verification complete, this document is candidate evidence and does not authorize T111.
