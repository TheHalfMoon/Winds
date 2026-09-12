# T125 — Usage/Cost Observation Necessity and Source Qualification

Status: `IN_QUALIFICATION`

Canonical base at candidate creation:

```text
BASE=10d1b9e9cbd4ded56cefecf32f4a04d03d623846
BASE_TREE=56a317a093f5374ea9c7a0d6a46331fe9ace97f5
T124=CLOSED_CANONICAL
T125=AUTHORIZED_TO_START
```

Decision:

```text
MODEL_MESH_USAGE_OBSERVATION=UNKNOWN
T125_OBSERVATION_ADAPTER_SELECTED=NO
T125_PRODUCTION_SOURCE_CHANGE=NO
T125_TEST_MODULE_REQUIRED=NO
T125_NEW_COLLECTION_PATH=NO
T125_NEW_PERSISTENCE=NO
T125_NEW_DEPENDENCY=NO
T125_PROVIDER_API_OR_SDK=NO
T125_BILLING_OR_PRICING_SOURCE=NO
T125_EXTERNAL_TELEMETRY=NO
```

This is the truthful T125 closeout candidate unless exact-head review or qualification proves that a production-qualified already-authorized structured local usage source was missed. T125 does not create a source merely to avoid `UNKNOWN`.

## 1. Qualification rule

The canonical Spec 009 Plan requires all of the following before an observation adapter is permitted:

- the source already exists in an authorized local runtime path;
- the observation is structured rather than inferred from prose;
- schema and provenance can be qualified explicitly;
- the observation can be bounded, source-labelled, scope-bound, freshness-aware, and secret-safe;
- no billing API, pricing database, provider SDK, telemetry SaaS, gateway, or guessed public price is added;
- usage/cost never becomes routing, winner selection, verification, human acceptance, execution, or Git authority.

The Plan also states explicitly that the existing Codex test protocol includes structured token-usage fields but those tests do not by themselves authorize a new production usage/cost persistence path.

## 2. Canonical source inspection

The canonical codebase contains only the following relevant Model Mesh usage surface:

```text
src/model_mesh.rs
ModelMeshUsageCostProjection::{Unknown}
project_model_mesh_target(...).usage_cost = Unknown
project_model_mesh_continuity(...).usage_cost = Unknown
```

There is no observed/qualified variant, no Model Mesh usage event type, and no usage/cost persistence schema in migration 0011.

The Codex protocol code recognizes `thread/tokenUsage/updated` only in the bounded T079 test protocol surface. The parser validates exact object keys, exact thread/turn identity, unsigned integer token counters, and a bounded model-context-window shape. The `CodexProtocolClient` contains no production usage-observation field, Model Mesh provenance binding, durable usage record, or production adapter from that notification into Model Mesh.

The T079-specific client state and connected-proof helpers are test-governed. They do not establish a generally admitted production runtime observation source. The separately governed live-runtime state also remains:

```text
T079_LIVE_PASS=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Therefore T079 token-usage fixture acceptance cannot be promoted into T125 production observation authority.

## 3. Rejected non-sources

The following are not qualified observation sources and are not adapted:

- source-agent or model prose containing token counts, cost, `PASS`, `VERIFIED`, or similar labels;
- public model pricing tables or current advertised prices;
- stale local price files;
- provider billing APIs;
- provider SDK callbacks;
- external telemetry or observability SaaS;
- terminal transcripts or arbitrary provider responses;
- command-history byte quotas or unrelated local storage-usage counters;
- shell telemetry events unrelated to provider/model token usage;
- estimates derived from prompt length, output length, characters, bytes, or heuristics.

None may be relabelled observed usage/cost.

## 4. Requirement disposition

| Requirement | T125 disposition |
| --- | --- |
| FR-071 | No qualified source exists, so no usage/cost metadata is retained. The conditional retention path remains unactivated. |
| FR-072 | SATISFIED: missing usage/cost remains `UNKNOWN`; no prose, guessed token count, or price estimate is presented as observed fact. |
| FR-073 | SATISFIED: `UNKNOWN` usage/cost creates zero verification, acceptance, routing, winner-selection, execution, or Git authority. |
| FR-074 | SATISFIED: no billing API, pricing database, telemetry service, external observability platform, or provider SDK is added. |
| FR-080 | SATISFIED for usage/cost labels: agent-generated cost/usage prose remains non-authoritative. |
| SC-016 | The no-qualified-source branch is proven by the existing `UNKNOWN` projection and this source-qualification decision. A structured-present production adapter is not claimed or manufactured. |
| SC-017 | SATISFIED: usage/cost state provides zero automatic routing, winner-selection, verification, or human-acceptance authority. |

## 5. Why no T125 implementation or focused test is added

T125 authorizes `src/model_mesh.rs`, an existing read-only runtime observation adapter, `src/t125_model_mesh_usage_tests.rs`, and test registration only if a suitable already-authorized source is selected.

No such source passed qualification. Adding an adapter or focused production-observation test without a qualified source would manufacture a data path that canonical governance explicitly forbids.

Existing T120 and T123 tests already prove the current truthful behavior:

- Model Mesh target and continuity projections serialize usage/cost as `UNKNOWN`;
- agent-reported cost/winner/authority labels do not promote any Model Mesh outcome;
- usage/cost does not affect target resolution or reviewer authority.

T125 therefore remains documentation-only.

## 6. Frozen schema and secret boundary

`migrations/0011_model_mesh_continuity.sql` remains immutable with accepted SHA-256:

```text
9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d
```

No credential, token, password, cookie, key, provider-private state, or arbitrary provider response is introduced into durable Model Mesh state.

## 7. T125 acceptance gate

This candidate may close T125 only after:

```text
MODEL_MESH_USAGE_OBSERVATION=UNKNOWN
QUALIFIED_STRUCTURED_LOCAL_SOURCE=NONE_FOUND
PRODUCTION_SOURCE_CHANGE=NO
FOCUSED_T125_TEST=NOT_REQUIRED
NEW_COLLECTION_PATH=NO
MIGRATION_0011_CHANGE=NO
NEW_DEPENDENCY=NO
BILLING_API=NO
PRICING_DATABASE=NO
PROVIDER_SDK=NO
EXTERNAL_TELEMETRY=NO
EXACT_HEAD_QUALITY=REQUIRED
AUTHOR_REVIEW=REQUIRED
PONYTAIL_YAGNI_REVIEW=REQUIRED
FRESH_INDEPENDENT_SUBSTANTIVE_REVIEW=REQUIRED
UNRESOLVED_MATERIAL_FINDINGS=0_REQUIRED
GUARDED_NORMAL_LANDING=REQUIRED
POST_MERGE_PUSH_VERIFICATION=REQUIRED
```

Only after those gates are proven may repository truth state:

```text
T125=CLOSED_CANONICAL
T126=AUTHORIZED
```
