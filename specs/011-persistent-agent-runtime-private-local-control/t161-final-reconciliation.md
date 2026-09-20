# T161 Final Reconciliation — Spec 011 Persistent Agent Runtime and Private Local Control

## Status and closure rule

- Canonical implementation-bearing state through T160 is merge `255f563ca43aeac51439e7e3dd5b65ab57513a7a`, tree `eee06b04c431602c441d27a7089fc2a852cf39b6`.
- T146..T160 are `CLOSED_CANONICAL` under the governance, merge, exact-head, and post-merge evidence ledger below.
- This T161 candidate is documentation/evidence only: checked-state reconciliation in `tasks.md` plus this artifact. It changes no production source, dependency, lockfile, workflow, migration, schema, runtime/provider authority, endpoint, security boundary, terminal behavior, desktop behavior, or Git authority.
- The checked T161 task records the final candidate state; it is not itself proof of canonical closure.
- T161 becomes `CLOSED_CANONICAL` only after this exact documentation candidate passes final review/quality gates, lands by guarded normal merge with an exact expected-head guard, and every actually-triggered post-merge push workflow succeeds.
- Exact T161 HEAD/TREE, final author/Ponytail/independent reviews, CI, merge identity, and post-merge runs belong in the PR/merge trail to avoid a self-referential candidate hash.
- Closing T161 authorizes no Spec 012 implementation automatically.

## 1. Canonical Spec 011 governance chain

| Governance unit | PR | Canonical merge | Exact post-merge verification |
| --- | ---: | --- | --- |
| Spec 011 Entry | #225 | `4ba393ea68bdd49477a633bc4f7e2bd072d287ff` | `quality` 35336085572 — SUCCESS |
| Spec 011 Specification | #226 | `5734dbb67becf386fe1e1340c2d585fcdf9f6c13` | `quality` 35337765623 — SUCCESS |
| Spec 011 Plan | #227 | `f36189becfb3f6322ffe3ff5b72ef7ba5e373a49` | `quality` 35342365164 — SUCCESS |
| Spec 011 Tasks | #228 | `a90575133bfccd8e2879f45930d50497e50aa504` | `quality` 35410249311 — SUCCESS |

Canonical Tasks acceptance authorized only T146. Every later implementation unit remained dependency-blocked until its predecessor closed.

## 2. T146–T160 canonical implementation and repair ledger

Every canonical row below is a normal merge on the accepted history. The post-merge column counts the workflows actually triggered by the exact merge; every listed set completed SUCCESS. Repairs are retained as separate forward-only authority and do not rewrite the original task history.

| Unit | PR | Canonical merge | Actually-triggered post-merge push verification |
| --- | ---: | --- | --- |
| T146 | #229 | `ab781dae92d7a6c1cc9a44cba9a4fd26da68e3be` | 8/8 SUCCESS |
| T147 | #230 | `78982a5e66dc3c97c7c0f41fd5f1e03436d857c1` | 7/7 SUCCESS |
| T148 | #231 | `7dc4be58d7b785a74a0c1380e9dc086f7a1ff438` | 4/4 SUCCESS |
| T148 protocol repair | #232 | `faea00ab6165f23f27275a493604dcc071ee3388` | 4/4 SUCCESS |
| T149 | #233 | `a62a197ebe00629edbeb0d5653bca463d22f2b2f` | 4/4 SUCCESS |
| T150 | #234 | `fb98fa1c6c5f5ff13a443397bf89fbf6f26e8933` | 13/13 SUCCESS |
| T151 | #235 | `699f6243fbea3396be8d6a4d77ec80652b777b38` | 8/8 SUCCESS |
| T152 | #236 | `08ef042c1771e2fb0b57139a49f76a8bb09dea31` | 4/4 SUCCESS |
| T153 | #237 | `ba20c1f26ce0ef024636a54a9676d3e83619ba9a` | 4/4 SUCCESS |
| T154 | #238 | `76f63f39f5dedea9e042ca6f56daccda8754fe5f` | 4/4 SUCCESS |
| T155 | #239 | `76d5e95e3c84b966a0bbb69f77a37f12d420ec78` | 4/4 SUCCESS |
| T156 | #240 | `637662e8868ac18a729aef85fc5f905929c18b2e` | 5/5 SUCCESS |
| T157 | #241 | `45d5d7e98b2fc2b49f2a6ffe53fabab3446520a6` | 7/7 SUCCESS |
| T158 | #242 | `68fe2b1b2ccbc0a87e788ca14565c04fe9cb8cc1` | 4/4 SUCCESS |
| T152 byte-bound repair | #244 | `2e1e81e96046ce6027702c4b92dfe22a7a3b653e` | 4/4 SUCCESS |
| T159 | #243 | `9806deeba3ff493b173b8e4a2b90e000bac32860` | 5/5 SUCCESS |
| T160 | #245 | `255f563ca43aeac51439e7e3dd5b65ab57513a7a` | 2/2 SUCCESS |

Key final qualification runs:

- T159 post-merge: `t159-performance` 35494672816, `quality` 35494672890, `t142-native-platform` 35494672777, `windows-terminal` 35494672819, `t141-desktop-security` 35494672783 — all SUCCESS.
- T160 exact-head: `quality` 35496245262 and `t160-native-platform` 35496245265 — SUCCESS.
- T160 post-merge: `quality` 35498690714 and `t160-native-platform` 35498690627 — SUCCESS.
- T160 post-merge native-platform artifacts are retained for Linux, macOS, and native Windows and are bound to exact canonical merge `255f563c...`.

No pre-final head is substituted for the accepted canonical merge.

## 3. Historical failure and supersession truth

Historical failures remain failures and remain inspectable.

- T158 qualification exposed a pre-existing T152 direct-output pressure failure. The later exact T159 Ubuntu campaign independently reproduced the same `OutputGap`; this evidence superseded the earlier environmental hypothesis.
- PR #244 repaired the actual product defect by replacing read-fragment-count admission with the same existing 256 KiB byte-count ceiling. No queue ceiling increased and fail-closed overflow behavior remained.
- T159 retained superseded campaign failures for burst scheduling, deliberately slow observer pressure, and a marker-echo test-harness defect.
- Exact candidate `52861dc986fff9850f78e9fafeb9cf93dcb76b82` remains a real T159 failure: run 35493719090 observed only 99,199 logical lines because PTY input echo could satisfy the completion marker.
- Final T159 head `281673d8cbef0e2607efc60c67070e3e64cb8fae` fixed only that harness defect and preserved all runtime/resource/security ceilings.
- `docs/provenance/011-t159-performance.md`, PR #243, and PR #244 remain the detailed failure/repair ledger. Successful successors do not rewrite those records.

## 4. Canonical implementation identity and authority boundaries

The following identities remain deliberately separate:

```text
RUNTIME_NAMESPACE_ID != RUNTIME_ALIAS
OWNER_GENERATION_ID != OS_PID
RUNTIME_NAMESPACE_ID != CANONICAL_WINDS_SESSION
RUNTIME_NAMESPACE_ID != WORKFLOW_OR_TASK_ID
RUNTIME_NAMESPACE_ID != PROVIDER_NATIVE_SESSION_ID
RUNTIME_NAMESPACE_ID != PROJECT_PRESENTATION_ID
RUNTIME_NAMESPACE_ID != GIT_CANDIDATE_OR_EVIDENCE_ID
AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
REPLAYED_OUTPUT != CANONICAL_EVIDENCE
PROCESS_EXIT != VERIFIED
```

Accepted authority remains:

- OS principal restriction first;
- exact protocol version/generation/correlation next;
- observer authority read-only;
- at most one explicit controller lease per runtime;
- all consequential actions exact-target only;
- zero multi-runtime input/control broadcast;
- renderer/WebView remains indirect through the trusted Rust host seam;
- terminal/agent/replayed text remains untrusted data;
- persistent ownership grants no Git merge/rebase/cherry-pick/push/PR/landing authority and no verification/human-acceptance authority.

No PID-as-authority, PID reconstruction, process enumeration recovery, or blind process signaling entered the accepted persistent-runtime implementation.

## 5. Persistence and migration inventory

Canonical migration inventory ends at 0013. There is no Spec 011 migration above 0013.

| Migration | Canonical blob | Size | Reconciliation |
| --- | --- | ---: | --- |
| `0011_model_mesh_continuity.sql` | `2ba3e3eb1e21e49780966829dc433d7b94577284` | 20,238 bytes | byte-identical at T146, T147, and T160 |
| `0012_desktop_presentation.sql` | `c4b38a229273f1617e1a2cff8f96d1b899e39faa` | 4,660 bytes | byte-identical at T146, T147, and T160 |
| `0013_persistent_runtime_owner.sql` | `58ec274234849712fa89f1e423cbc12db9be8379` | 6,680 bytes | introduced by T147 and byte-identical at T160 |

Thus Spec 011 did not mutate frozen migrations 0011/0012 after entry and did not drift migration 0013 after T147 acceptance.

Migration 0013 stores bounded runtime/owner metadata needed for fail-closed reconciliation. It does not turn persisted PID/endpoint material into live ownership proof and does not persist terminal output, full environments, ordinary secrets, credentials, tokens, or raw control frames as authority.

## 6. Dependency and provenance inventory

Spec 011 directly admitted one Windows-specific dependency edge:

```toml
[target.'cfg(windows)'.dependencies]
windows-sys = { version = "=0.61.2", default-features = false, features = [
    "Win32_Security_Authorization",
    "Win32_Security_Cryptography",
    "Win32_Storage_FileSystem",
    "Win32_System_IO",
    "Win32_System_Pipes",
    "Win32_System_Threading",
] }
```

T150 provenance records:

- `windows-sys 0.61.2`;
- lockfile checksum `ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc`;
- license `MIT OR Apache-2.0`;
- crate MSRV Rust 1.71, below Winds Rust 1.97.1;
- `default-features = false`;
- the exact six direct Win32 child feature modules above;
- generated parent features are implied rather than broadly admitted;
- the crate already existed transitively and direct T150 admission added the Winds root edge;
- removal path is explicit.

Existing `portable-pty = 0.9.0` remains the accepted PTY/ConPTY lineage; Spec 011 reuses the accepted terminal backend instead of adding a second terminal backend.

Herdr remains research-only/unadmitted for this program:

- `docs/provenance/source-registry.md` marks the canonical Herdr source as future donor / not implementation-admitted;
- `docs/provenance/spec-011-t146-domain-entropy.md` explicitly states no Herdr source is copied or adapted;
- the canonical T160 implementation contains no `herdr` reference under `src/`;
- the Spec 011 Herdr donor-code ledger is therefore empty;
- founder permission does not override exact Tasks admission, provenance, license, security, test, and removal-path gates.

## 7. Platform, performance, and resource qualification

### Linux

T160 directly exercised the Linux domain on Ubuntu 24.04 with:

- private POSIX Unix socket path/ownership/mode;
- kernel peer credential same-user proof and wrong-UID fail-closed fixture;
- OS entropy;
- persistent PTY detach/reattach;
- observer/controller behavior;
- owner crash/recovery;
- replay/backpressure;
- 100 persistent-owner reconnect cycles;
- 100-cycle terminal lifecycle/resource cleanup;
- the full release-profile T159 core campaign.

### macOS

T160 directly exercised macOS 15 independently from Linux with:

- macOS Unix socket path/fallback/peer behavior;
- macOS entropy;
- PTY continuity;
- controller/replay/recovery;
- 100 persistent-owner reconnect cycles;
- 100-cycle terminal lifecycle/resource cleanup.

Linux evidence is not substituted for macOS.

### Native Windows

T160 directly exercised Windows Server 2025 with:

- named-pipe security descriptor/effective DACL;
- current-user SID access;
- actual anonymous-principal denial fixture;
- generation-bound collision/fail-closed behavior;
- Windows entropy;
- ConPTY lifecycle/input/output/resize/exit/stop;
- preserved fail-closed native-Windows interrupt limitation;
- controller/replay/recovery;
- 100 persistent-owner reconnect cycles;
- 100-cycle terminal lifecycle/resource cleanup.

Linux/macOS/WSL evidence is not substituted for native Windows.

### WSL

Spec 011 introduced no Windows-host-to-WSL persistent-owner bridge. Existing WSL execution/path evidence is not relabelled as persistent-owner proof.

```text
SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED
WINDOWS_HOST_TO_WSL_OWNER_BRIDGE=NOT_INTRODUCED
```

### T159 canonical engineering bounds

Final accepted T159 evidence proved:

- output >= 10 MiB and >= 100,000 logical lines; final campaigns observed 13,027,082 bytes / 100,200 lines;
- 500 reconnect cycles;
- 8 concurrent observers;
- 32 runtime namespaces;
- aggregate replay <= 64 MiB;
- per-runtime replay <= 8 MiB;
- per-runtime replay events <= 10,000;
- per-observer queued outbound payload <= 4 MiB;
- reattach/snapshot <= 100 ms p95;
- cached observer attach <= 100 ms p95;
- controller input dispatch <= 10 ms p95;
- resize dispatch <= 25 ms p95;
- independent idle owner CPU <= 1% p95 and RSS <= 64 MiB;
- FD/storage bounds remained stable;
- no threshold was relaxed and no correctness/security check was disabled.

These are engineering qualification ceilings, not marketing claims.

## 8. Evidence key

| Key | Canonical evidence |
| --- | --- |
| E146 | T146 domain/identity/event contracts; `docs/provenance/spec-011-t146-domain-entropy.md`; PR #229 / merge `ab781dae...`. |
| E147 | Migration 0013, persisted metadata truth, fail-closed prior-generation reconciliation; PR #230 / merge `78982a5e...`. |
| E148 | Bounded protocol v1 framing/messages/correlation/replay rejection plus forward repair PR #232; merges `7dc4be58...` and `faea00ab...`. |
| E149 | Linux/macOS private Unix transport, path ownership/mode, peer identity, stale-path defenses, entropy; PR #233 / merge `a62a197e...`. |
| E150 | Native-Windows named pipe DACL/SID/anonymous denial/collision/entropy and qualified `windows-sys`; PR #234 / merge `fb98fa1c...`. |
| E151 | Same-binary owner mode, singleton generation/readiness/idle shutdown; PR #235 / merge `699f6243...`. |
| E152 | Persistent terminal ownership, exact-generation detach/reattach, detached exit, stale-generation rejection, Windows interrupt nonclaim; PR #236 plus byte-bound repair PR #244. |
| E153 | Read-only observers, bounded memory replay, explicit gaps, slow-reader behavior; PR #237 / merge `ba20c1f2...`. |
| E154 | One explicit controller lease, deterministic takeover/release/expiry/races, zero implicit promotion; PR #238 / merge `76f63f39...`. |
| E155 | Shared Rust local-control client, exact endpoint/generation handling, bounded projection seam; PR #239 / merge `76d5e95e...`. |
| E156 | Truthful continuity/ownership/controller/replay/mismatch presentation states; PR #240 / merge `637662e8...`. |
| E157 | Owner/runtime stop/crash/restart/recovery, corruption fail-closed behavior, outcome uncertainty, no blind reconstruction; PR #241 / merge `45d5d7e9...`. |
| E158 | Adversarial protocol/endpoint/renderer/authority/secret campaign and negative-scope proof; PR #242 / merge `68fe2b1b...`. |
| E159 | Resource/performance/high-output/observer/runtime/reconnect qualification; PR #243 / merge `9806deeb...`; retained failure ledger `docs/provenance/011-t159-performance.md`. |
| E160 | Direct Linux/macOS/native-Windows platform qualification, 100 reconnects/platform, T063 soak, WSL nonclaim; PR #245 / merge `255f563c...`. |

## 9. Functional requirement reconciliation — FR-001 through FR-070

| Requirement | Classification | Exact evidence / preserved boundary |
| --- | --- | --- |
| FR-001 | PROVEN_DOMAIN | E146 — immutable runtime namespace identity independent of aliases/PID/provider/session/presentation identities. |
| FR-002 | PROVEN_PERSISTENCE_TRUTH | E147 — persisted metadata alone never proves live ownership. |
| FR-003 | PROVEN_OWNERSHIP | E152 — live-process claims derive from owner-held ownership; PID equality is not authority. |
| FR-004 | PROVEN_RECOVERY_TRUTH | E147 + E157 — unproven continuation converges to ownership lost/unknown, never optimistic live ownership. |
| FR-005 | PROVEN_DOMAIN | E146 — process liveness and Winds ownership are separate facts. |
| FR-006 | PROVEN_TERMINAL | E152 — presentation detach alone does not terminate the owned child. |
| FR-007 | PROVEN_CLIENT | E155 — reattach is exact runtime namespace + owner generation; aliases/stale endpoints cannot redirect control. |
| FR-008 | PROVEN_GENERATION | E151 + E157 — owner generation prevents stale replacement-owner continuity. |
| FR-009 | PROVEN_IDENTITY_SEPARATION | E146 — runtime identity remains separate from Session/workflow/provider/Project/Git evidence identities. |
| FR-010 | PROVEN_PRESENTATION_TRUTH | E156 — provider-native resume remains distinct from retained live-process ownership. |
| FR-011 | PROVEN_PRESENTATION_TRUTH | E156 — Winds reconstruction/reassignment remains distinct from native resume and retained ownership. |
| FR-012 | PROVEN_ATTRIBUTION | E146 + E156 — exposed continuity states preserve source/proof class. |
| FR-013 | PROVEN_OBSERVER_BOUNDARY | E153 — multiple observers are read-only for every consequential mutation. |
| FR-014 | PROVEN_CONTROLLER | E154 — control authority is explicit, attributable, bounded, and per-runtime. |
| FR-015 | PROVEN_CONTROLLER | E154 — focus/visibility/order/recent-observer status never grants controller authority. |
| FR-016 | PROVEN_CONTROLLER | E154 — takeover conflict resolution and prior-controller disposition are deterministic. |
| FR-017 | PROVEN_CONTROLLER | E154 — disconnect/timeout/restart/expiry behavior is explicit. |
| FR-018 | PROVEN_CONTROLLER_RACE | E154 — races serialize without multiple controllers or cross-runtime action. |
| FR-019 | PROVEN_ZERO_BROADCAST | E154 — every control action targets one runtime; multi-runtime input/control broadcast remains prohibited. |
| FR-020 | PROVEN_PROTOCOL | E148 — mutating requests carry exact target and request identity. |
| FR-021 | PROVEN_PROTOCOL | E148 — local-control protocol is explicitly versioned. |
| FR-022 | PROVEN_PROTOCOL | E148 — incompatible versions fail closed; no unsafe downgrade. |
| FR-023 | PROVEN_CORRELATION | E148 + E155 — responses remain bound to exact runtime/request/generation. |
| FR-024 | PROVEN_REPLAY_REJECTION | E148 + E157 — duplicate/replayed consequential requests follow explicit rejection/idempotency truth. |
| FR-025 | PROVEN_BOUNDS | E148 + E153 + E159 — frames, queues, replay, and retained history have hard tested bounds. |
| FR-026 | PROVEN_ADVERSARIAL_BOUND | E148 + E158 — malformed/oversized control data fails closed within bounded resources. |
| FR-027 | PROVEN_BACKPRESSURE | E153 + E159 — slow/non-reading clients receive explicit bounded gap/disconnect semantics. |
| FR-028 | PROVEN_UNTRUSTED_OUTPUT | E148 + E158 — terminal/agent output cannot become protocol, lifecycle truth, approval, evidence, or host action. |
| FR-029 | PROVEN_LOCAL_ONLY | E149 + E150 + E158 — no TCP/HTTP/WebSocket/LAN/cloud/remote-origin control surface. |
| FR-030 | PROVEN_PLATFORM_SECURITY | E149 + E150 + E160 — each released platform directly proves its OS-principal endpoint policy. |
| FR-031 | PROVEN_ENDPOINT_SECURITY | E149 + E150 + E157 — stale/path/symlink/name ambiguity fails closed. |
| FR-032 | PROVEN_ENDPOINT_SECURITY | E149 + E150 + E157 — endpoint state is not destructively replaced merely because a path/name exists. |
| FR-033 | PROVEN_DOMAIN_ENTROPY | E146 — runtime/generation identities are collision-resistant and exact-bound. |
| FR-034 | PROVEN_PEER_IDENTITY | E149 + E150 — trusted local principal and kernel/SID proof are explicit. |
| FR-035 | PROVEN_DESIGN_BOUNDARY | E146 + E149 + E150 — OS principal/generation is the accepted boundary; no ordinary persisted secret/token is introduced as hidden authority. |
| FR-036 | TRUTHFUL_NONCLAIM | E149 + E150 + E158 — same-effective-user malicious-code isolation is not claimed. |
| FR-037 | PROVEN_NEGATIVE_SCOPE | E158 — remote clients/SSH/cloud relay/mobile/network control remain absent. |
| FR-038 | PROVEN_RENDERER_BOUNDARY | E155 + E158 — WebView/renderer content has no generic owner-control access. |
| FR-039 | PROVEN_TYPED_BRIDGE | E155 — desktop integration remains a typed allowlisted Rust seam, not a generic dispatcher. |
| FR-040 | PROVEN_GIT_BOUNDARY | E158 — persistent ownership grants no merge/rebase/cherry-pick/push/PR/landing authority. |
| FR-041 | PROVEN_VERIFICATION_BOUNDARY | E158 — persistent ownership grants no verification or human-acceptance authority. |
| FR-042 | PROVEN_TRUTH_BOUNDARY | E156 + E158 — process completion is not candidate verification/acceptance. |
| FR-043 | PROVEN_SOURCE_DISTINCTION | E156 + E158 — agent-reported, Winds-observed, and human-decided states remain distinct. |
| FR-044 | PROVEN_REPLAY | E153 — replay preserves ordering/source identity and explicit gap/truncation semantics. |
| FR-045 | PROVEN_EVIDENCE_BOUNDARY | E153 + E158 — replayed output/history never becomes canonical evidence by retention. |
| FR-046 | PROVEN_BOUNDS | E153 — replay/history defaults, hard bounds, and eviction behavior are explicit. |
| FR-047 | PROVEN_SECRET_BOUNDARY | E147 + E158 — full process environments and secret values are not durably retained for reattach. |
| FR-048 | PROVEN_CLIENT_SECURITY | E155 + E158 — generic history/control methods do not expose raw privileged credentials by default. |
| FR-049 | PROVEN_RECOVERY | E147 + E157 — owner restart explains uncertainty without manufacturing live ownership. |
| FR-050 | PROVEN_CORRUPTION_FAIL_CLOSED | E147 + E157 — partial/corrupt persistence fails closed without silent ambiguous reinitialization. |
| FR-051 | PROVEN_STOP | E157 — explicit runtime stop has deterministic owned-resource cleanup/final-state semantics. |
| FR-052 | PROVEN_UNCERTAIN_CLEANUP | E157 — unproven termination/reaping remains unresolved/ownership-lost rather than clean success. |
| FR-053 | PROVEN_IDEMPOTENT_TERMINAL_STATE | E157 — repeated stop/close remains deterministic for terminal runtimes. |
| FR-054 | PROVEN_OWNER_SHUTDOWN | E157 — shutdown behavior/result is exposed per runtime independently. |
| FR-055 | PROVEN_PLATFORM_TERMINAL | E152 + E160 — accepted POSIX PTY/native-Windows ConPTY lifecycle behavior is preserved. |
| FR-056 | TRUTHFUL_WINDOWS_LIMITATION | E152 + E160 — native-Windows interrupt remains fail-closed; no unsupported interrupt claim. |
| FR-057 | PROVEN_DOMAIN_SEPARATION | E150 + E160 — native Windows and WSL claims remain separate domains. |
| FR-058 | PROVEN_DIRECT_PLATFORM_EVIDENCE | E160 — released endpoint/detach/controller/recovery claims are exercised directly per claimed platform. |
| FR-059 | PROVEN_RESOURCE_BOUND | E159 — idle CPU/RSS/FD/storage ceilings directly measured. |
| FR-060 | PROVEN_STRESS_BOUND | E159 — high-output + multi-observer stress preserves exact targeting and bounded queues. |
| FR-061 | PROVEN_RECONNECT_BOUND | E159 — reconnect churn preserves generation/controller/client/runtime integrity without unbounded leak. |
| FR-062 | PROVEN_EVENT_ATTRIBUTION | E146 — lifecycle event schema retains runtime/generation/order/proof/controller identity and excludes secret material. |
| FR-063 | PROVEN_IDENTITY_TARGETING | E146 + E155 — aliases remain presentation helpers, never consequential authority. |
| FR-064 | PROVEN_DISAMBIGUATION | E155 — ambiguous user-facing runtime lookup does not silently choose consequential identity. |
| FR-065 | PROVEN_LOCAL_INDEPENDENCE | E151 + E158 + T161 inventory — ordinary local continuity requires no cloud account/control plane. |
| FR-066 | PROVEN_NEGATIVE_SCOPE | E158 — no generic plugin/runtime extension authority entered through the protocol. |
| FR-067 | PROVEN_SCOPE_BOUNDARY | E146 + T161 inventory — complete Herdr parity was not required for this first persistent-runtime program. |
| FR-068 | PROVEN_PROVENANCE_BOUNDARY | E146 + T161 inventory — no direct/adapted Herdr source slice was admitted or copied. |
| FR-069 | PROVEN_HISTORICAL_TRUTH | T161 failure ledger — failed/rejected/superseded evidence and inherited nonclaims remain inspectable and unrelabelled. |
| FR-070 | PROVEN_PRESENTATION_TRUTH | E156 — ownership loss, protocol mismatch, controller/replay/generation mismatch remain explicit states. |

Reconciliation result:

```text
FR_RECONCILED=70/70
FR_UNMAPPED=0
FR_FALSELY_UPGRADED_NONCLAIMS=0
```

## 10. Success criterion reconciliation — SC-001 through SC-025

| Criterion | Classification | Exact evidence / preserved boundary |
| --- | --- | --- |
| SC-001 | PROVEN_CONTINUITY | E152 — long-running owned terminal survives complete client detach and exact-generation reattach without PID reconstruction. |
| SC-002 | PROVEN_PERSISTENCE_TRUTH | E147 + E157 — persisted runtime with no proven owner never reports live-owned. |
| SC-003 | PROVEN_STALE_STATE_SECURITY | E157 + E158 — stale PID/endpoint fixtures cannot redirect reattach or control an unrelated process. |
| SC-004 | PROVEN_OBSERVER_CONTROLLER | E153 + E154 — multiple observers receive state while mutations require exact controller authority. |
| SC-005 | PROVEN_CONTROLLER_RACE | E154 — simultaneous takeover produces exactly one deterministic controller outcome. |
| SC-006 | PROVEN_NO_IMPLICIT_PROMOTION | E154 — disconnect/expiry never silently promotes an observer. |
| SC-007 | PROVEN_PROTOCOL_VERSION | E148 — version mismatch fails explicitly with zero consequential operation/downgrade. |
| SC-008 | PROVEN_REPLAY_REJECTION | E148 — duplicate/replayed mutating requests cannot repeat consequential action outside specified semantics. |
| SC-009 | PROVEN_ADVERSARIAL_BOUND | E158 — malformed/oversized campaigns remain bounded and cannot expand authority/crash owner. |
| SC-010 | PROVEN_BACKPRESSURE | E153 + E159 — slow client behavior is bounded and explicit. |
| SC-011 | PROVEN_REPLAY_GAP | E153 — replay beyond ceiling reconnects with explicit gap/truncation and ordering truth. |
| SC-012 | PROVEN_UNTRUSTED_OUTPUT | E158 — forged VERIFIED/approval/protocol/runtime/host-action output cannot mutate trusted state. |
| SC-013 | PROVEN_CRASH_RECOVERY | E157 — owner crash converges fail-closed without blind PID signal/kill and preserves uncertainty. |
| SC-014 | PROVEN_CONTINUITY_CLASS | E156 — provider resume, retained live continuation, reconstruction, and fresh process are observably distinct. |
| SC-015 | PROVEN_CLEANUP_TRUTH | E157 — clean success is reported only when termination/reaping/finalization is known. |
| SC-016 | PROVEN_PLATFORM_SECURITY | E149 + E150 + E160 — wrong-principal and stale/replaced endpoint fixtures directly exercise every claimed platform policy. |
| SC-017 | PROVEN_NATIVE_WINDOWS | E150 + E160 — Windows endpoint/controller/detach-reconnect claims are native-Windows evidence, not inference. |
| SC-018 | TRUTHFUL_WSL_BOUNDARY | E160 — WSL remains distinct; Spec 011 persistent owner inside WSL is explicitly NOT_CLAIMED rather than inferred. |
| SC-019 | PROVEN_RESOURCE_BOUND | E159 — idle CPU/RSS/FD/storage remain within frozen ceilings. |
| SC-020 | PROVEN_HIGH_OUTPUT_BOUND | E159 — high output + observers preserve controller/target truth and bounded owner resources. |
| SC-021 | PROVEN_RECONNECT_BOUND | E154 + E157 + E159 + E160 — client disconnect/revocation, recovery uncertainty, reconnect churn, and repeated lifecycle campaigns preserve generation/controller/client/runtime bounds without ambiguous authority or unbounded records. |
| SC-022 | PROVEN_RENDERER_SECURITY | E158 — WebView/terminal/agent content cannot invoke generic control, create controller authority, or manufacture lifecycle/evidence truth. |
| SC-023 | PROVEN_NEGATIVE_SCOPE | E158 + T161 inventory — no public network control, remote execution, plugin runtime, marketplace, live binary handoff, automatic Git landing, or same-user sandbox claim entered. |
| SC-024 | T161_FINAL_GATE | Canonical implementation evidence through T160 is complete; fresh exact T161 author/Ponytail/independent review, quality, guarded landing, and post-merge proof remain required. |
| SC-025 | T161_FINAL_GATE | This artifact reconciles all 70 FR and 25 SC and preserves inherited truth/nonclaims; canonical closeout occurs only after exact T161 landing/post-merge proof. |

Reconciliation result:

```text
SC_RECONCILED=25/25
SC_UNMAPPED=0
SC_PREMATURELY_CLOSED_FINAL_GATES=0
```

## 11. Inherited provider/runtime nonclaims and negative scope

Historical provider/runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED
DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED
```

Persistent ownership does not upgrade provider/model truth. A live owned shell/process is not proof of real Codex/Claude execution, provider-native session continuation, verification, human acceptance, or model identity.

Additional explicit boundaries:

- same-effective-user malicious-code isolation is not claimed;
- private locality is not represented as an OS sandbox;
- no public TCP/UDP/HTTP/HTTPS/WebSocket/QUIC/SSH/LAN/cloud-relay owner control surface;
- no remote/mobile client control plane;
- no generic plugin runtime, marketplace, generic method dispatcher, or provider router;
- no updater/service installation authority;
- no live binary handoff;
- no automatic Git merge/rebase/cherry-pick/push/PR/landing authority;
- no generic WebView/renderer access to the owner endpoint;
- no second terminal backend;
- no complete Herdr parity or Herdr source adoption;
- no hidden cloud-account/control-plane requirement for ordinary local continuity;
- no WSL persistent-owner claim and no Windows-host-to-WSL owner bridge.

## 12. T161 exact-candidate review and landing gates

The final T161 candidate must remain limited to:

- `specs/011-persistent-agent-runtime-private-local-control/tasks.md` checked-state reconciliation;
- `specs/011-persistent-agent-runtime-private-local-control/t161-final-reconciliation.md`.

Required exact-head gates:

- repository `quality` succeeds on the exact final T161 head;
- changed scope remains documentation/governance only;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS with no unnecessary abstraction/scope;
- fresh independent substantive review covers all FR/SC, canonical merge evidence, migrations/dependencies/nonclaims, and yields zero material findings;
- zero unresolved material review threads;
- Alibaba OpenCodeReview delegation accounting is reconciled truthfully; Markdown exclusion is not relabelled as an LLM review;
- immediately before landing: current `main`, PR base/head/tree, ahead/behind, changed paths, rulesets, mergeability, and exact-head CI are reconciled;
- if `main` moves, integrate only by normal forward merge and restart exact-head qualification/review; never rebase or force-push;
- guarded normal merge uses exact `expected_head_sha`;
- merge tree equals accepted T161 candidate tree and ordered parents are reconciled;
- GitHub signature metadata is verified;
- every actually-triggered post-merge push workflow succeeds.

The documentation-only T161 candidate must not masquerade as a new implementation build; implementation-bearing qualification remains bound to canonical T160 tree `eee06b04c431602c441d27a7089fc2a852cf39b6`.

## 13. Closure state machine

Current pre-landing truth:

```text
T146..T160=CLOSED_CANONICAL
T161=IN_QUALIFICATION

SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=CLOSED_CANONICAL
SPEC_011_FIRST_PERSISTENT_RUNTIME_PROGRAM=NOT_YET_CLOSED

SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED
WINDOWS_HOST_TO_WSL_OWNER_BRIDGE=NOT_INTRODUCED
SUCCESSOR_IMPLEMENTATION_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO
```

Only after exact T161 guarded landing and successful post-merge verification may repository truth state:

```text
T146..T161=CLOSED_CANONICAL

SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=CLOSED_CANONICAL
SPEC_011_FIRST_PERSISTENT_RUNTIME_PROGRAM=CLOSED_CANONICAL

SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED
WINDOWS_HOST_TO_WSL_OWNER_BRIDGE=NOT_INTRODUCED
SUCCESSOR_IMPLEMENTATION_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO
```

Closing T161 closes the first Spec 011 persistent-runtime program only. Any Spec 012 implementation requires its own canonical Entry -> Specification -> Plan -> Tasks authority chain.
