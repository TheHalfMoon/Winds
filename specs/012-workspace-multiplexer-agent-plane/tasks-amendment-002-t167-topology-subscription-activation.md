# Spec 012 Tasks Amendment 002 — T167 Topology Subscription Activation

Status: CANDIDATE

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

## Authority state before landing

```text
SPEC_012_TASKS_AMENDMENT_002=CANDIDATE
T166=CLOSED_CANONICAL_BY_MERGE_AND_POST_MERGE_VERIFICATION
T167=AUTHORIZED_BY_PREDECESSOR_BUT_BLOCKED_ON_PATH_CORRECTION
T168..T185=BLOCKED_BY_PREDECESSOR

T167_OWNER_TOPOLOGY_SUBSCRIPTION_ACTIVATION=NOT_YET_AUTHORIZED_PENDING_AMENDMENT_LANDING
PROTOCOL_V2_SCHEMA_CHANGE_AUTHORIZED=NO
AGENT_OBSERVATION_STREAM_ACTIVATION_AUTHORIZED=NO
ATTENTION_STREAM_ACTIVATION_AUTHORIZED=NO
WORKTREE_MUTATION_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

Canonical landing of this amendment authorizes only the narrow T167 owner-side topology-subscription activation described above. It does not authorize T168 or any later task.
