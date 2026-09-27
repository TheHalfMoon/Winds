# Spec 012 Tasks Amendment 002 — T167 Topology Subscription Activation

Status: CLOSED_CANONICAL

Authority basis: Winds Constitution 1.1.0, canonical Spec 012 Plan event-loss/recovery decisions, canonical T166 protocol-v2 contract, and canonical T167 purpose/acceptance requirements.

## Material contradiction

Canonical T166 deliberately froze the complete first-program protocol-v2 vocabulary while requiring not-yet-authorized later-domain operations to return typed `UNSUPPORTED_OPERATION` with no side effect. Canonical T167 is the first task that requires the shared Rust client to list, snapshot, and **subscribe** to topology through that already-frozen v2 contract, then recover its bounded projection after gaps.

The accepted T167 path list permits only `src/persistent_runtime/client.rs`, new `src/multiplexer/projection.rs`, and focused `src/t167_multiplexer_client_tests.rs`. However, canonical main after T166 still routes `ProtocolPayload::SubscribeMultiplexerEvents` through `inactive_v2_domain_response(...)` in `src/persistent_runtime/owner.rs`. A production T167 client therefore cannot receive an accepted topology subscription boundary or topology events without changing an owner-side path that the current T167 path list omits.

This is a narrow activation-path omission. It is not authority to alter protocol-v2 message kinds or schemas, add a second endpoint/server, activate agent-observation or attention streams, add a dependency, expand runtime-controller authority, or introduce T168+ UI behavior.

## Narrow correction

T167 additionally permits `src/persistent_runtime/owner.rs` **only** to activate the already-frozen T166 topology subscription contract needed by T167. The activation is limited to:

- accept `SubscribeMultiplexerEventsV2` only when `stream == MultiplexerSubscriptionStreamV2::Topology`;
- return the already-frozen `MultiplexerEventSubscriptionAckV2` with an exact topology-generation boundary and the exact optional workspace filter supplied by the request;
- retain only bounded per-connection topology-subscription presentation state inside the existing owner/session lifetime;
- emit only already-frozen `MultiplexerEventV2` topology events for accepted topology changes relevant to that subscription;
- preserve exact owner-generation, connection, request/correlation, workspace-filter, and topology-generation binding;
- clear subscription state on ordinary disconnect/reconnect rather than reconstructing authority from cached presentation;
- preserve the existing bounded owner outbound queue and fail closed on slow-client/gap conditions so the T167 client must resubscribe and obtain an authoritative snapshot;
- preserve `MultiplexerWrite` and Runtime Controller authority as separate explicit authorities; subscribing never grants either one.

The correction does **not** authorize:

- any protocol-v2 message-kind, payload-schema, framing, cursor, or frame-ceiling change;
- activation of `AgentObservations` or `Attention` subscription streams;
- worktree handlers or Git mutation;
- a second owner, listener, endpoint, network/public RPC surface, or transport change;
- renderer/WebView direct owner access;
- detector execution, provider execution, `agent.start`, command-palette authority, T168+ UI, or new dependency;
- arbitrary replay of buffered topology events across a discontinuity.

If implementation proves that safe topology subscription requires any additional production path or authority beyond this exact correction, T167 must stop and obtain a separate accepted amendment rather than expanding scope implicitly.

## Required qualification

The corrected T167 candidate must prove:

1. a read-only client can list and snapshot topology without requesting `MultiplexerWrite`;
2. topology subscribe returns the exact current generation boundary and cannot activate non-topology streams;
3. an accepted next-generation topology mutation produces an exactly bound event and converges with an authoritative snapshot;
4. a generation discontinuity or explicit `HistoryGap` marks cached projection stale and requires fresh subscribe + authoritative snapshot before continuation;
5. workspace-filtered subscriptions cannot receive a different workspace event as trusted projection input;
6. reconnect/owner-generation movement clears subscription/cache authority and cannot reuse stale presentation as live truth;
7. malformed/stale/wrong-connection/wrong-generation subscription/event material fails closed;
8. subscription never grants Runtime Controller or `MultiplexerWrite` authority;
9. existing Spec 011 runtime-control behavior and T166 protocol-v2 wire schema remain unchanged;
10. all applicable exact-head repository and native-platform checks pass.

Candidate movement invalidates stale exact-candidate evidence. This governance amendment is canonical only after all of the following complete on the exact final head:

- repository quality;
- author correctness/safety/governance/evidence-integrity review;
- Ponytail/YAGNI review;
- **genuine Jev review** bound to the exact candidate; if Jev cannot actually execute, the amendment remains blocked and no PASS may be inferred or fabricated;
- **Alibaba Open Code Review exact-head delegation/rule accounting**; if the Markdown path is classified `unsupported_ext`, that truthful tool result is recorded and the document receives explicit manual exact-head review, but the OCR invocation itself must actually run;
- fresh independent exact-head review with zero unresolved material findings/threads;
- immediate pre-landing identity/scope/mergeability reconciliation;
- guarded expected-head normal merge;
- successful post-merge verification.

No unavailable reviewer or tool is relabelled as PASS, and no predecessor Jev/OCR result qualifies a moved candidate.

## Canonical closeout

```text
AMENDMENT_PR=259
BASE=7fb9761874e03608fc7d47a9174dd9c27f4a0537
HEAD=7213c69c86bcebf81154fe9bbe113656abe46db9
HEAD_TREE=8c52c91774bf1fc80277f4058b54c2c628100781
MERGE=c9c85e75fa066049f62a7f3ab2ccfcc70bf98e6d
MERGE_TREE=8c52c91774bf1fc80277f4058b54c2c628100781
MERGE_PARENT_1=7fb9761874e03608fc7d47a9174dd9c27f4a0537
MERGE_PARENT_2=7213c69c86bcebf81154fe9bbe113656abe46db9
MERGE_GITHUB_VERIFICATION=VERIFIED_VALID
PR_QUALITY_RUN=36254396434 SUCCESS
POST_MERGE_PUSH_RUN=36255400257 SUCCESS
CHANGED_PATHS=1
UNRESOLVED_MATERIAL_THREADS=0
JEV=TypeSafe Jev pin 31f89602797fb7bea007f8a480bf368bf564954e exact range 7fb97618..7213c69c expected_hunks=1 reviewed_hunks=1 findings=0 blocking_findings=0 coverage=COMPLETE status=PASSED executor_run=TheHalfMoon/Cotra 36255268563 job 108440735965
JEV_IN_REPO_ATTEMPT=Winds run 36254996057 Jev step FAILED with TYPESAFE_API_KEY_NOT_CONFIGURED and is superseded by the pinned review-only executor above; it is never relabelled as PASS
ALIBABA_OPEN_CODE_REVIEW=v1.12.9 exact-range preview on Winds run 36254996057 job 108439976387 reviewable_count=0 excluded_count=1 exclude_reason=unsupported_ext for the sole Markdown path, followed by explicit manual exact-head Markdown review
```

The amendment was canonically landed by merge `c9c85e75fa066049f62a7f3ab2ccfcc70bf98e6d`, and the activated contract was then implemented and closed by T167 below.

## Canonical T167 closeout

```text
T167_PR=260
BASE=c9c85e75fa066049f62a7f3ab2ccfcc70bf98e6d
HEAD=5d71621d99dc2e75a5fe25c393e9e2b8b4b2a5e0
HEAD_TREE=cbd1d30e619e3248bdbdaf796573afd1683a0fb7
MERGE_BASE=c9c85e75fa066049f62a7f3ab2ccfcc70bf98e6d
AHEAD_BY=11
BEHIND_BY=0
CHANGED_PATHS=4 src/multiplexer/projection.rs src/persistent_runtime/client.rs src/persistent_runtime/owner.rs src/t167_multiplexer_client_tests.rs
PR_EXACT_HEAD_RUNS=quality 36295258131, t159-performance 36295258127, t160-native-platform 36295258146, t141-desktop-security 36295258075, t142-native-platform 36295258193, windows-terminal 36295258148, release-candidate 36295258145; all SUCCESS
JEV=TypeSafe Jev pin 31f89602797fb7bea007f8a480bf368bf564954e exact range c9c85e75..5d71621d expected_hunks=21 reviewed_hunks=21 findings=0 blocking_findings=0 coverage=COMPLETE status=PASSED executor_run=TheHalfMoon/Cotra 36295313567 job 108552961724
ALIBABA_OPEN_CODE_REVIEW=v1.12.9 delegation accounting on Winds run 36295289244 job 108552897352 reviewable_count=4 excluded_count=0 with resolved rules for every changed Rust path
INDEPENDENT_EXACT_HEAD_REVIEW=https://github.com/TheHalfMoon/Winds/pull/260#pullrequestreview-5328876532 material_findings=0
UNRESOLVED_REVIEW_THREADS=0
MERGE=5b43efd05b9199a1a34ccb1528ae200fdcce79d4
MERGE_TREE=cbd1d30e619e3248bdbdaf796573afd1683a0fb7
MERGE_PARENT_1=c9c85e75fa066049f62a7f3ab2ccfcc70bf98e6d
MERGE_PARENT_2=5d71621d99dc2e75a5fe25c393e9e2b8b4b2a5e0
MERGE_GITHUB_VERIFICATION=VERIFIED_VALID
POST_MERGE_PUSH_RUNS_ON_5b43efd=quality 36303066333, t159-performance 36303066311, t160-native-platform 36303066318, t141-desktop-security 36303066331, t142-native-platform 36303066324, windows-terminal 36303066327; six actually-triggered workflows, all SUCCESS
POST_MERGE_FAILURES=0
```

No post-merge workflow beyond the six actually triggered by the canonical merge was required, and none of the six failed.

## Authority state

```text
SPEC_012_TASKS_AMENDMENT_002=CLOSED_CANONICAL
T166=CLOSED_CANONICAL_BY_MERGE_AND_POST_MERGE_VERIFICATION
T167=CLOSED_CANONICAL_BY_MERGE_AND_POST_MERGE_VERIFICATION
T168=AUTHORIZED
T169..T185=BLOCKED_BY_PREDECESSOR

T167_OWNER_TOPOLOGY_SUBSCRIPTION_ACTIVATION=LANDED_CANONICAL
PROTOCOL_V2_SCHEMA_CHANGE_AUTHORIZED=NO
AGENT_OBSERVATION_STREAM_ACTIVATION_AUTHORIZED=NO
ATTENTION_STREAM_ACTIVATION_AUTHORIZED=NO
WORKTREE_MUTATION_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

Canonical landing of this amendment authorized only the narrow T167 owner-side topology-subscription activation described above, and that activation is now landed. T168 is authorized solely by canonical T167 closeout under the Tasks dependency order; this amendment grants no protocol-v2 schema change, no non-topology stream activation, and no authority beyond the T168 task text.
