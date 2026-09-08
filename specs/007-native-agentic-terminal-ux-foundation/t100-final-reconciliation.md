# T100 — Spec 007 Final Reconciliation Candidate

Status: `IN_QUALIFICATION`

Canonical base at candidate creation:

```text
BASE=e3e5d98c36bb4289e8e4aebcc2dbb07c97abaa98
BASE_TREE=0be307a19429261edb63a654cf5eb609a896387b
T099=CLOSED_CANONICAL
T100=AUTHORIZED_TO_START
```

This is the documentation/evidence-only closeout authorized by T100. It does not change production code, runtime behavior, dependencies, lockfiles, workflows, migrations, terminal ownership, provider/browser behavior, Git authority, or any later roadmap surface.

The final immutable T100 candidate SHA is intentionally not embedded in this file. PR metadata, exact-head CI, exact-head review, guarded landing, and post-merge Git identity bind the final candidate without creating a self-referential follow-up commit.

## 1. Canonical implementation chain

The Spec 007 first implementation program reconciles to the following canonical task landings. Candidate heads and trees are listed as historical exact-task identity; canonical closure is established by the corresponding guarded landing and post-merge verification.

| Task | PR | Exact accepted head | Accepted tree | Canonical merge | Post-merge push verification |
| --- | ---: | --- | --- | --- | --- |
| T087 | #106 | `93b5499c4782fe58e6597e31a8ffdcf74716bc1f` | `58922e1bdce4a7bf1733e93e75cfbf8362587d2f` | `43d93943570c8e733663189214584598a9b5fb33` | `quality` 34060741435 SUCCESS; `windows-terminal` 34060741353 SUCCESS |
| T088 | #107 | `2b6716ccda26ebca956ac89cd579beeb60bf1bca` | `395dfd146615d9070e2585266b5cd6352f0271b1` | `8ee80b6a35da320f9cc33c9bc734cc45c265b988` | `quality` 34062012706 SUCCESS; `windows-terminal` 34062012675 SUCCESS |
| T089 | #108 | `80a5bd6cb84191f4d2218a68f4605786ee1cce67` | `df62f10d268d4596f0ee42d346865903ba617b89` | `1f216402f2f281d80839b4125cde67eeeec1a233` | `quality` 34067369633 SUCCESS; `windows-terminal` 34067369635 SUCCESS |
| T090 | #109 | `8e5a41dad9044ae8a73ef77fb7fd6e280f077bca` | `0e8bbbd8a9f698e08ef1667e51e0b4117b9bc131` | `efc33abc24934a4a7e9615651dd98d5b7e0e1228` | `quality` 34069799576 SUCCESS; `windows-terminal` 34069799567 SUCCESS |
| T091 | #111 | `6ca840f2fa036aa077e8c081d2b26a36b0cc7791` | `3b52c637f4bc52741826403f67ee7b1827ed3ba1` | `6af87b211c1a012c252d03af9c4cba2e5dbb0de8` | `quality` 34145228610 SUCCESS; `windows-terminal` 34145228619 SUCCESS |
| T092 | #112 | `9f827678bdcc822910a1f49394fbbcc324c3f393` | `2a3de4d149d32b45899147abd14126f1f14878e1` | `821dc8362b78af473621e3ad2b9b1adc3735bb3f` | `quality` 34151619994 SUCCESS; `windows-terminal` 34151620032 SUCCESS |
| T093 | #113 | `c74519fef39d61db8ceae7cd7d8d31d57cbe1f01` | `ceeb08e55c4970cd2cb136cd2a5733c521a0e0ed` | `e56697830160ff4645a70236f8a3aa76f3c878b5` | `quality` 34154947978 SUCCESS; `windows-terminal` 34154948013 SUCCESS |
| T094 | #114 | `c716d27301d548e41f8bdccd4f326dab4f8b136a` | `a56da578cf840906120b5a3d02198aaa55a8448f` | `9d6300ff5566a40ba9a90247e164e0761a91fe65` | `quality` 34157352705 SUCCESS; `windows-terminal` 34157352717 SUCCESS |
| T095 | #115 | `251dd2728cc07eae48645989b4f7cac146b38d5e` | `fb0a58e67087904cbec3d3899718b15b6aea0e3f` | `f999ee521d6258b10fbd348299bb545fddd14ba5` | `quality` 34165618429 SUCCESS; `windows-terminal` 34165618431 SUCCESS |
| T096 | #117 | `57e6d25045ef61c4f868b33c1e5c10507d9cb79d` | `8704772323670365782cab92297a54d1c796188b` | `fc5d15c60115b302674de061f72119bc315f0320` | `quality` 34170292213 SUCCESS; `windows-terminal` 34170292209 SUCCESS |
| T097 | #119 | `92504b8d6243e41f7c6e0f649e51188429b9f217` | `a59190ca554740799b6212285abff89e8120aad3` | `b7cb7c23701b46c278d142e51321c76af4ca8ec4` | `quality` 34234249557 SUCCESS; `windows-terminal` 34234249761 SUCCESS |
| T098 | #120 | `bf821765ce6fe400e1eb95b86a194e158b6b7c62` | `fd58da53487bfae84a979aaa81ead424fcec3c65` | `df0d48992f37d430fa73271fee7936e118be612a` | `quality` 34251889285 SUCCESS; `windows-terminal` 34251889359 SUCCESS |
| T099 | #121 | `0875333b16c9c2c1838051c1bcafd6f6f56f8f1b` | `0be307a19429261edb63a654cf5eb609a896387b` | `e3e5d98c36bb4289e8e4aebcc2dbb07c97abaa98` | `quality` 34275498210 SUCCESS; `windows-terminal` 34275498328 SUCCESS |

The T095-to-T096 interval contains the forward-only cleanup commit `71c909f4ce7e0ac2fbe2bde50c27e66dc75f0045`, which removed an accidental T096 placeholder while restoring the exact accepted T095 tree. History is preserved; no rewrite or force-push is used.

Governance amendments used by this program remain bounded to their recorded authority. In particular, T087/T091 temporary lock-generation workflows were removed before final task qualification; T097 Amendment 003 authorized only the bounded benchmark/production-ready seam required for exact performance proof; T099 Amendment 004 widened only two test-only WSL host-observation budgets from 8 seconds to 15 seconds while leaving the Linux watchdog, timeout-injection values, and production WSL limits unchanged; and T099 Amendment 005 changed only the T060 fixture expectation to accept either proven termination or the already-existing fail-closed ownership-loss truth. The original T099 `release-candidate` failure on `c38686d2ddf749d5cbe5ed27c3918c77676c298c` remains material historical evidence and is not reclassified as a flake.

## 2. Dependency reconciliation

Spec 007 introduced only the task-local direct dependencies authorized by the Plan/Tasks:

| Task | Direct dependency | Exact version | Selected direct features | Provenance |
| --- | --- | --- | --- | --- |
| T087 | `ratatui` | `0.30.2` | defaults disabled; `crossterm_0_29` | checksum `3274ba0a2c5e1bcad2a2005d20f4dc59dad26b2eb0940fb094500dba4099d57d`; MIT; MSRV 1.88 |
| T087 | `crossterm` | `0.29.0` | accepted default event/backend family; `event-stream` and `osc52` absent | checksum `d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b`; MIT; MSRV 1.63 |
| T089 | `vt100` | `0.16.2` | synchronous parser path | checksum `054ff75fb8fa83e609e685106df4faeffdf3a735d3c74ebce97ec557d5d36fd9`; MIT; MSRV 1.70 |
| T091 | `ratatui-textarea` | `0.9.2` | defaults disabled; `crossterm` | checksum `3c78d5ba0f26f97baed69a4c479f268a31c7b5b89d68ab939842152e383d6e73`; MIT; MSRV 1.86 |

T089's newly resolved registry closure includes `vte 0.15.0` and `arrayvec 0.7.6` with recorded checksums. T091 dependency-generation evidence records 139 resolved packages, Cargo-recorded registry checksums, declared licenses, and MSRVs compatible with Winds Rust `1.97.1`. The final `Cargo.toml` contains exactly the expected Spec 007 direct additions and no `tui-term`, Alacritty runtime, Tokio/async runtime, webview shell, clipboard automation crate, fuzzy/semantic search dependency, provider/browser stack, daemon/service framework, LSP/editor framework, or plugin runtime.

Dependency-generation artifacts are provenance inputs only. Final task qualification was rerun on workflow-free exact heads, so generator success is never represented as final candidate CI.

## 3. Platform truth

`PROVEN_PLATFORM_BOUND` means only the named domain was directly exercised; no OS inherits proof from another.

- Linux: host TUI + Linux PTY workbench behavior directly exercised.
- macOS: host TUI + macOS PTY workbench behavior directly exercised.
- Native Windows: host TUI + ConPTY lifecycle directly exercised.
- Windows + Ubuntu WSL2: host/guest domain and mapped-workspace/path truth directly exercised separately from native Windows.
- Native-Windows terminal support is not promoted into unsupported native-Windows authoritative `winds verify` support.
- Platform-specific unavailable behavior remains unavailable/not claimed rather than inferred.

T096 is the canonical platform-integration task; later T097-T099 exact-head workflows continued to exercise applicable platform/regression paths without changing that evidence boundary.

## 4. Frozen performance and boundedness reconciliation

No FR-045..FR-053 threshold was relaxed.

Final T099 exact accepted head `0875333b16c9c2c1838051c1bcafd6f6f56f8f1b` / tree `0be307a19429261edb63a654cf5eb609a896387b` passed `t097-performance` run `34274620724`. That run rebuilt the exact candidate in the pinned Ubuntu 24.04 release environment and explicitly proved FR-045, bounded production output projection, FR-046 through FR-050 and FR-052, FR-051 idle CPU/RSS overhead, and FR-053 exact-candidate staleness semantics before assembling and uploading exact-candidate machine-readable artifact `10075321553` (`t097-performance-0875333b16c9c2c1838051c1bcafd6f6f56f8f1b`, digest `sha256:cab06edf67e7602694e4d32a349f92c80ff8d89fd8906483098989c49aed8294`). This is the final implementation-head performance provenance; the earlier T097 artifact remains historical only.

| Requirement | Status | Canonical disposition |
| --- | --- | --- |
| FR-045 | `PROVEN_PLATFORM_BOUND` | Cold/input-ready startup met `<=1500 ms p95` over the required release-profile campaign. |
| FR-046 | `PROVEN_PLATFORM_BOUND` | Workbench input-to-dispatch overhead met `<=16 ms p95` over >=1000 iterations. |
| FR-047 | `PROVEN_PLATFORM_BOUND` | 50-pane topology operation latency met `<=16 ms p95` over >=1000 operations. |
| FR-048 | `PROVEN_PLATFORM_BOUND` | Retained-history search met `<=100 ms p95` over >=200 searches on the required >=100,000-line corpus. |
| FR-049 | `PROVEN_PLATFORM_BOUND` | Live owned-terminal campaign processed 100,004 logical lines / 11,300,380 bytes and 1,000 in-campaign navigation samples without weakening the `<=100 ms p95` navigation budget. |
| FR-050 | `PROVEN_DETERMINISTIC` | Per-pane transcript remains bounded to 100,000 logical lines and 32 MiB, whichever is reached first, with visible truncation/eviction truth. |
| FR-051 | `PROVEN_PLATFORM_BOUND` | Ten-pane 60-second idle campaign recorded 0.2161% of one logical core and 4,026,368 bytes Winds RSS overhead. |
| FR-052 | `PROVEN_PLATFORM_BOUND` | 1,000 resize requests completed with final size verified from owned `TerminalSession::current_size`. |
| FR-053 | `PROVEN_GOVERNANCE_BOUNDARY` | Evidence records exact candidate/tree/environment/method and rejects older-candidate measurements after movement. |

T099 Amendment 004 changes no Spec performance threshold and no production cleanup timeout.

## 5. Functional requirement reconciliation

| Requirement | Status | Canonical evidence / disposition |
| --- | --- | --- |
| FR-001 | `PROVEN_DETERMINISTIC` | T090 retains accepted terminal/session ownership as live lifecycle authority. |
| FR-002 | `PROVEN_DETERMINISTIC` | T088 topology plus T090 ownership supports multiple panes without making pane identity process authority. |
| FR-003 | `PROVEN_DETERMINISTIC` | T088 uses opaque transient `PaneId`, distinct from canonical/native/process identities. |
| FR-004 | `PROVEN_DETERMINISTIC` | T088/T099 prove labels and titles remain presentation-only under duplicate/case/Unicode fixtures. |
| FR-005 | `PROVEN_DETERMINISTIC` | T088/T092/T099 preserve canonical identity through focus/layout churn. |
| FR-006 | `PROVEN_PLATFORM_BOUND` | T090/T096 exercise accepted close/interrupt/terminate/resize lifecycle paths per claimed domain and fail closed on unproven ownership. |
| FR-007 | `PROVEN_DETERMINISTIC` | T088/T090/T099 restart/restore fixtures never claim durable live-child reattachment. |
| FR-008 | `PROVEN_GOVERNANCE_BOUNDARY` | One-process architecture preserved; no daemon/socket/control server/IPC added. |
| FR-009 | `PROVEN_DETERMINISTIC` | T093 defines typed source presentation for user input, terminal output, repository evidence, warning/policy, and human-decision surfaces. |
| FR-010 | `PROVEN_DETERMINISTIC` | T093/T098 keep Agent-reported state separate from Winds-observed evidence. |
| FR-011 | `PROVEN_DETERMINISTIC` | T089/T093/T095/T099 prove terminal text/control data cannot self-promote to evidence or authority. |
| FR-012 | `PROVEN_DETERMINISTIC` | T093 uses truthful continuous-terminal fallback when attribution is ambiguous. |
| FR-013 | `PROVEN_DETERMINISTIC` | T089/T093 preserve terminal byte order/raw bytes while display decoding remains non-authoritative. |
| FR-014 | `PROVEN_DETERMINISTIC` | T094/T098 expose exact candidate/applicability with canonical verification evidence. |
| FR-015 | `PROVEN_DETERMINISTIC` | T091 shell mode dispatches explicitly and introduces no silent model/provider routing. |
| FR-016 | `PROVEN_DETERMINISTIC` | T091/T099 prove exactly-one-selected-owned-pane dispatch and no broadcast. |
| FR-017 | `PROVEN_DETERMINISTIC` | T091 covers Unicode and long input without silent truncation. |
| FR-018 | `PROVEN_DETERMINISTIC` | T091 preserves explicit multiline/paste semantics and rejects unsafe normalization. |
| FR-019 | `PROVEN_DETERMINISTIC` | T091 editor/history behavior preserves command text and remains non-canonical memory. |
| FR-020 | `PROVEN_DETERMINISTIC` | Unsupported editor/paste behavior fails explicitly rather than silently dropping/reordering bytes. |
| FR-021 | `PROVEN_DETERMINISTIC` | T088/T092 provide create/focus/split/resize/close without durable multiplexer ownership. |
| FR-022 | `PROVEN_DETERMINISTIC` | T092/T098 expose canonical workspace/session context independent of display labels. |
| FR-023 | `PROVEN_DETERMINISTIC` | Pane/session association uses stable canonical session identity. |
| FR-024 | `PROVEN_DETERMINISTIC` | T092 deterministic search returns explicit ambiguity rather than recency guessing. |
| FR-025 | `PROVEN_DETERMINISTIC` | T088/T090/T098 visibly distinguish live/exited/stopped/ownership-lost/error states. |
| FR-026 | `PROVEN_DETERMINISTIC` | Restored presentation metadata cannot establish live ownership. |
| FR-027 | `PROVEN_GOVERNANCE_BOUNDARY` | Pane visibility/context grants no Agent/delegate file/tool/execution authority. |
| FR-028 | `PROVEN_DETERMINISTIC` | T094/T098 keep IDLE/Agent DONE/VERIFIED/ACCEPTED semantics distinct. |
| FR-029 | `PROVEN_DETERMINISTIC` | Verification/evidence remains bound to exact candidate identity. |
| FR-030 | `PROVEN_DETERMINISTIC` | Candidate movement visibly stales prior evidence/review while preserving history. |
| FR-031 | `PROVEN_DETERMINISTIC` | Exit status alone never establishes VERIFIED/ACCEPTED. |
| FR-032 | `PROVEN_DETERMINISTIC` | Terminal/Agent success prose never establishes VERIFIED/ACCEPTED. |
| FR-033 | `PROVEN_DETERMINISTIC` | Explicit verification inspection reuses repository-native evidence semantics rather than a UI-local verifier. |
| FR-034 | `PROVEN_GOVERNANCE_BOUNDARY` | Spec 007 code/diff/evidence inspection remains read-only. |
| FR-035 | `PROVEN_GOVERNANCE_BOUNDARY` | No automatic winner, merge, rebase, cherry-pick, push, PR creation, or landing path introduced. |
| FR-036 | `PROVEN_DETERMINISTIC` | T093 retained-window search preserves source/workspace/session/pane context and explicit truncation truth. |
| FR-037 | `PROVEN_DETERMINISTIC` | Evidence-like transcript matches remain transcript data. |
| FR-038 | `PROVEN_DETERMINISTIC` | Bounded retention never mutates canonical evidence, approvals, or human decisions. |
| FR-039 | `PROVEN_GOVERNANCE_BOUNDARY` | No semantic-memory/vector/RAG/embedding/learned-retrieval architecture introduced. |
| FR-040 | `PROVEN_DETERMINISTIC` | T089/T095/T099 keep control sequences non-authoritative and incapable of direct host action. |
| FR-041 | `PROVEN_DETERMINISTIC` | OSC52 is fail-closed; no silent host clipboard write path exists. |
| FR-042 | `PROVEN_DETERMINISTIC` | T095 classifies hyperlink/file references as advisory/denied with no automatic open action. |
| FR-043 | `PROVEN_DETERMINISTIC` | Malformed/truncated/oversized/unsupported control data fails safely under T089/T095/T099. |
| FR-044 | `PROVEN_DETERMINISTIC` | Forged trusted-looking terminal output cannot change evidence/authority state. |
| FR-045 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-046 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-047 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-048 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-049 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-050 | `PROVEN_DETERMINISTIC` | See frozen performance reconciliation. |
| FR-051 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-052 | `PROVEN_PLATFORM_BOUND` | See frozen performance reconciliation. |
| FR-053 | `PROVEN_GOVERNANCE_BOUNDARY` | See frozen performance reconciliation. |
| FR-054 | `PROVEN_PLATFORM_BOUND` | T096 directly separates native Windows/ConPTY, WSL2, Linux, and macOS proof. |
| FR-055 | `PROVEN_PLATFORM_BOUND` | Native Windows and WSL2 remain separate path/execution domains. |
| FR-056 | `PROVEN_PLATFORM_BOUND` | Unsupported platform behavior is not inferred from another OS. |
| FR-057 | `PROVEN_DETERMINISTIC` | T092/T098 provide keyboard paths for core create/focus/navigation/search/inspection operations. |
| FR-058 | `PROVEN_DETERMINISTIC` | T098 uses explicit textual selected/lifecycle/evidence state, not color-only meaning. |
| FR-059 | `PROVEN_DETERMINISTIC` | T098 explicitly tests Unicode wide/combining selection/copy fixtures without identity corruption. |
| FR-060 | `PROVEN_PLATFORM_BOUND` | Accessibility/keyboard/parser/render claims are bounded to directly exercised workbench surfaces. |
| FR-061 | `PROVEN_GOVERNANCE_BOUNDARY` | No persistent owner, daemon, IPC/control protocol, remote execution, or mobile continuation added. |
| FR-062 | `PROVEN_GOVERNANCE_BOUNDARY` | No browser automation/profile/CDP runtime, provider mesh/SDK/auth brokerage, or model-routing authority added. |
| FR-063 | `PROVEN_GOVERNANCE_BOUNDARY` | No ACP dependency, MCP runtime, A2A, generic runtime/plugin framework, SDK, host, or marketplace added. |
| FR-064 | `PROVEN_GOVERNANCE_BOUNDARY` | No verified-learning activation, training/fine-tuning/RL, vector/RAG memory, or executable self-modification added. |
| FR-065 | `PROVEN_GOVERNANCE_BOUNDARY` | Spec 003 lifecycle and Spec 006 identity/evidence/authority invariants remain the accepted underlying boundaries and continue through regression gates. |
| FR-066 | `PROVEN_GOVERNANCE_BOUNDARY` | UI/parser/editor dependencies were selected only through exact task-local Plan/Tasks authority and dependency qualification. |

## 6. Success criterion reconciliation

| Criterion | Status | Canonical evidence / disposition |
| --- | --- | --- |
| SC-001 | `PROVEN_PLATFORM_BOUND` | T096 plus exact workbench lifecycle tests establish concurrent owned-pane operations on directly exercised domains; no durable ownership is inferred. |
| SC-002 | `PROVEN_DETERMINISTIC` | T088/T099 duplicate/case/Unicode rename/layout campaigns preserve canonical identity. |
| SC-003 | `PROVEN_DETERMINISTIC` | T089/T093/T095/T099 forged PASS/VERIFIED/ACCEPTED/evidence-like terminal data produces zero authority promotion. |
| SC-004 | `PROVEN_DETERMINISTIC` | T094/T098/T099 make candidate-A evidence stale/not-applicable after movement while retaining history. |
| SC-005 | `PROVEN_DETERMINISTIC` | T091/T099 prove exactly-one-pane shell dispatch, no provider/model invocation, and no silent broadcast. |
| SC-006 | `PROVEN_DETERMINISTIC` | T091 proves multiline/Unicode/long-input semantics without silent truncation or line-splitting drift. |
| SC-007 | `PROVEN_PLATFORM_BOUND` | Final T099 `t097-performance` run `34274620724` proves FR-045 on the pinned reference environment. |
| SC-008 | `PROVEN_PLATFORM_BOUND` | Final T099 `t097-performance` run `34274620724` proves FR-046 and FR-047. |
| SC-009 | `PROVEN_PLATFORM_BOUND` | Final T099 `t097-performance` run `34274620724` proves FR-048 and FR-049. |
| SC-010 | `PROVEN_DETERMINISTIC` | T089 plus final T099 `t097-performance` run `34274620724` prove FR-050 bounds and visible eviction without evidence mutation. |
| SC-011 | `PROVEN_PLATFORM_BOUND` | Final T099 `t097-performance` run `34274620724` proves FR-051 and FR-052. |
| SC-012 | `PROVEN_DETERMINISTIC` | T089/T095/T099 malformed/control campaigns cannot crash into privileged host actions. |
| SC-013 | `PROVEN_DETERMINISTIC` | T092/T098 prove keyboard reachability and non-color-only critical state. |
| SC-014 | `PROVEN_PLATFORM_BOUND` | T096 directly qualifies native Windows/ConPTY, WSL2, Linux, and macOS claims separately. |
| SC-015 | `PROVEN_DETERMINISTIC` | T088/T090/T099 restart/restore tests never claim durable live ownership. |
| SC-016 | `T100_FINAL_GATE` | T087-T099 canonical regressions are green; the T100 docs-only candidate still requires fresh exact-head repository/applicable workflows before closeout. |
| SC-017 | `T100_FINAL_GATE` | T087-T099 review gates are canonical; the T100 final exact docs-only candidate still requires author correctness/safety, Ponytail/YAGNI, fresh independent substantive review, and zero unresolved material findings. |
| SC-018 | `PROVEN_GOVERNANCE_BOUNDARY` | No daemon/IPC, remote/browser/provider mesh, ACP/MCP runtime, plugin system, learning subsystem, automatic winner, or automatic landing path was introduced. |

SC-016 and SC-017 are intentionally not pre-declared PASS inside this candidate artifact. They become satisfied only through exact-head T100 PR evidence and the guarded/post-merge closeout described below.

## 7. Spec 006 live-runtime nonclaims remain separate

Spec 007 does not upgrade or reinterpret the separately governed Spec 006 physical-runtime lanes:

```text
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

No historical one-shot authorization or historical runtime attempt is reused as current live PASS.

## 8. Negative-scope reconciliation

The accepted Spec 007 program introduced none of the following:

- persistent daemon, session owner, socket/RPC/IPC/control plane;
- remote execution or mobile continuation;
- browser automation/profile/CDP runtime;
- provider mesh/provider SDK/authentication brokerage or silent model routing;
- ACP/MCP/A2A runtime or generic tool/plugin framework;
- plugin host, marketplace, integration SDK;
- verified-learning activation, training/fine-tuning/RL, vector/RAG memory, executable self-modification;
- custom PTY or second terminal ownership runtime;
- custom full source editor or LSP/IDE replacement;
- automatic candidate winner, merge, rebase, cherry-pick, push, PR creation, or landing.

The workbench remains one-process, shell-compatible, and verification-native over previously accepted terminal/evidence authority seams.

## 9. Historical evidence discipline

Older-head CI, benchmark, and review evidence is retained as history only. Every task was requalified after candidate movement before its canonical landing. T100 does not represent an earlier task candidate as the current T100 candidate. Final frozen-performance claims are grounded in `t097-performance` run `34274620724` and artifact `10075321553` on final T099 implementation head `0875333b16c9c2c1838051c1bcafd6f6f56f8f1b`; T100 does not represent that implementation-head performance run as current CI for the moved docs-only T100 candidate, whose own exact-head qualification remains separate below.

## 10. T100 exact-candidate qualification boundary

This closeout is documentation/evidence-only. Focused implementation tests are N/A because T100 changes no implementation surface; repository/applicable regression workflows remain mandatory.

Before T100 may close canonically, the exact final candidate must satisfy all of the following:

- changed paths remain only T100-authorized documentation/evidence paths;
- no production source, Cargo/lockfile, workflow, migration, runtime, dependency, protocol, provider/browser, daemon/IPC, remote, learning, plugin, ACP/MCP, or automatic-landing behavior changes;
- exact-head repository `quality` succeeds;
- applicable exact-head platform/security/regression workflows succeed without treating older-head results as current;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review reaches the exact final head with zero unresolved material findings;
- zero unresolved material review threads;
- exact final main/base/head/tree/scope/ruleset/mergeability race reconciliation;
- expected-head guarded merge;
- landed canonical main/tree/parents verified;
- applicable post-merge push checks succeed on the landing commit.

Only after those gates are proven may repository truth state:

```text
T087..T100=CLOSED_CANONICAL
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS=CLOSED_CANONICAL
SPEC_007_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

Closing T100 authorizes no later Spec 007 phase and does not automatically authorize Spec 008, verified learning, durable runtime/daemon/IPC, remote execution, browser/provider orchestration, ACP/MCP, plugins, or any research-roadmap implementation.