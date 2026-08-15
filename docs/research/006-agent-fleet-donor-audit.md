# Winds Agent Fleet — Open-Source Donor Audit

**Status:** Research dossier; no Agent Fleet product code authorized by this document.

**Audit date:** 2026-08-15

**Canonical Winds base:** `d9c43a26f4e920a8df9a89aee69d2eaf70e47037`

## 1. Purpose

Winds should become the single developer CLI/control plane from which a user can discover, launch, delegate to, resume, compare, review, verify, and account for many coding agents without collapsing those agents into one opaque model gateway.

The product distinction is not “run many agents.” Many tools can already do that. The distinction is:

> **Run every coding agent. Trust only the evidence.**

Winds must keep agent claims, caller intent, shell-reported telemetry, Winds-observed process/Git facts, deterministic verifier results, and explicit human decisions distinct.

This dossier identifies open-source implementations worth adopting, integrating, copying under provenance controls, or studying. It deliberately does **not** amend Spec 003 or authorize Agent Fleet implementation.

## 2. Non-negotiable architectural direction

### 2.1 Transport priority

For controlling an external coding agent, prefer in this order:

1. **ACP / standardized structured protocol**
2. **Vendor-native structured machine API or app server**
3. **Machine-readable CLI mode**
4. **Compatibility relay around a CLI**
5. **PTY/TUI interpretation only as a last-resort compatibility path**

Winds must not create a new public “Winds Agent Protocol” when ACP or an existing structured surface is sufficient.

### 2.2 Agent output is not verification evidence

A delegated agent may report success, tests passing, files changed, cost, or other facts. These reports are useful telemetry but are not authoritative Winds verification evidence. Winds must independently bind verification to exact workspace/base/candidate identity and run its own gates.

### 2.3 No Fleet code in T044

Spec 003 / T044 remains limited to the forward-only SQLite migration, workspace registry, common execution ledger, execution-scoped events, typed terminal-session records, domain/store types, and deterministic tests. Agent-specific tables, schedulers, routers, token/cost fields, ACP integration, and fleet UI are follow-on work.

## 3. Donor handling policy

Every donor is classified as one or more of:

- **ADOPT** — use a stable standard/library directly when it removes bespoke Winds code.
- **INTEGRATE** — invoke or speak to an external tool/protocol rather than copying implementation.
- **COPY-CANDIDATE** — permissively licensed implementation may be copied only after exact commit/path provenance and license obligations are recorded.
- **STUDY** — borrow design/testing ideas, not source code.
- **REJECT-IN-CORE** — useful project, but its architecture/dependency model conflicts with current Winds constraints.

No donor source may be copied into Winds merely because the repository is permissively licensed. Before any copy:

1. pin the exact donor repository commit;
2. identify exact copied paths/lines and their own license headers/notices;
3. verify transitive/generated/vendor code provenance;
4. record why direct adoption/integration or a smaller reimplementation is insufficient;
5. preserve required copyright/license notices;
6. add deterministic tests proving Winds semantics rather than donor semantics;
7. run correctness/safety and Ponytail review.

## 4. Tier 0 — foundational interoperability donors

### 4.1 Agent Client Protocol Rust SDK

Repository: `agentclientprotocol/rust-sdk`

Audit snapshot: `7d8291d42236023c683bfc52f13d27746cda59ea`

License: Apache-2.0.

Relevant surfaces:

- core `agent-client-protocol` client/agent roles and protocol types;
- connection builders and handlers;
- proxy/conductor model;
- HTTP/SSE/WebSocket transport crates;
- shared ACP test utilities;
- protocol-version and capability negotiation.

Decision: **ADOPT / DEEP STUDY**.

Winds should strongly prefer the official Rust SDK rather than implement ACP JSON-RPC framing, capability negotiation, session lifecycle, and compatibility logic itself. However, draft/unstable protocol-v2 and MCP-over-ACP features must not be treated as stable Winds contracts without explicit version pinning.

Important Winds rule: persist the negotiated protocol version and relevant capabilities with session metadata when that information matters to replay/debugging. Never infer wire compatibility from package version alone.

### 4.2 ACP Registry

Repository: `agentclientprotocol/registry`

Audit snapshot: `afc34d3d96ea1ee01f54cf3c8eed51bbd0862c54`

License: Apache-2.0 for the registry; individual listed agents retain their own licenses.

Relevant surfaces:

- machine-readable registry index;
- agent schema and distribution metadata;
- platform targets;
- authentication capability validation;
- automated version refresh.

Decision: **INTEGRATE / CACHE-SAFELY**.

Winds can use the registry as one discovery input, but must not equate registry presence with local installation, trust, compatibility, or permission to install. Local executable resolution and exact version discovery remain Winds-observed facts. Registry data should be cached with retrieval/source/version metadata and treated as advisory catalog data.

### 4.3 Codex ACP adapter

Repository: `agentclientprotocol/codex-acp`

Audit snapshot: `abf477689ecf1524d4ea83bfe1dae8cc8e6a0e34` (release commit 1.3.0).

License: Apache-2.0.

Relevant implementation ideas:

- starts Codex App Server instead of scraping the TUI;
- maps ACP requests/events to Codex app-server operations;
- exposes approval and sandbox modes;
- propagates shell/file-change/permission/MCP/terminal/reasoning/plan/review/token events;
- exposes subagents through namespaced metadata;
- regenerates Codex app-server types when the supported Codex version changes;
- produces Windows/macOS/Linux binaries.

Decision: **INTEGRATE FIRST; COPY ONLY SMALL GENERIC TEST/ADAPTER PATTERNS IF NEEDED**.

This is the preferred Codex interoperability path. Winds should not parse Codex terminal rendering when app-server/ACP is available. Version skew between Codex and the adapter must be surfaced explicitly rather than silently tolerated.

### 4.4 Claude Agent ACP adapter

Repository: `agentclientprotocol/claude-agent-acp`

License: Apache-2.0.

Relevant surfaces:

- Claude Agent SDK to ACP translation;
- tool permission requests;
- edit review;
- TODO/task updates;
- background/interactive terminals;
- session resume/fork/list/close support;
- nested subagent transcript metadata while retaining protocol-compatible fallback behavior.

Decision: **INTEGRATE FIRST / STUDY SESSION AND SUBAGENT MAPPING**.

Nested transcript extensions are useful but must remain capability-gated. Winds must preserve the distinction between standardized ACP data and vendor-namespaced metadata.

### 4.5 acpx

Repository: `openclaw/acpx`

License: MIT.

Relevant surfaces:

- lightweight headless ACP client;
- stateful ACP sessions;
- CLI composition intended for orchestrators/harnesses;
- explicit product stance against PTY scraping and adapter-specific glue.

Decision: **STUDY / POSSIBLE EXTERNAL FALLBACK TOOL**.

Winds is itself a Rust control plane, so embedding a Node CLI as the primary ACP layer would be unnecessary. The useful donor value is its UX, flow composition, failure handling, and minimal-client philosophy.

### 4.6 delegate-skills

Repository: `amElnagdy/delegate-skills`

License: MIT.

Relevant surfaces:

- one implementer CLI per relay;
- self-contained delegated briefs;
- common `delegate-relay.result.v1` result shape;
- touched-file discovery;
- exit/signal/OOM-aware reporting;
- session/resume handling where supported;
- read-only tripwires where the target CLI cannot enforce read-only itself;
- Windows process-tree handling;
- strict rule that relays do not commit and reviewer/orchestrator retains landing authority.

Decision: **COPY-CANDIDATE FOR SMALL RELAY/TEST PATTERNS; COMPATIBILITY FALLBACK, NOT CORE PROTOCOL**.

This is one of the best donors for heterogeneous CLI behavior and negative tests. Winds should preserve the “implementer does not own commit/promotion” principle. ACP/native structured transports should supersede relay logic whenever available.

### 4.7 Coder AgentAPI

Repository: `coder/agentapi`

License: MIT.

Relevant surfaces:

- common HTTP API over many coding agents;
- agent-specific launch/control adapters;
- compatibility when vendors lack stable SDKs.

Risk: some compatibility behavior parses/cleans TUI output and therefore can break when a vendor changes rendering.

Decision: **STUDY / FALLBACK REFERENCE**.

Do not make TUI cleaning a Winds primary control mechanism.

## 5. Tier 1 — workspace, worktree, scheduling, and lifecycle donors

### 5.1 Worktrunk

Repository: `max-sixty/worktrunk`

Implementation: Rust; first-class Linux/macOS/Windows distribution.

Relevant ideas:

- ergonomic worktree creation/switch/merge/remove;
- agent launch per worktree;
- project hook commands require approval on first run;
- changed project command requires renewed approval;
- bounded JSONL command log containing timestamp, command label, command, exit, and duration;
- Windows-specific command naming/packaging handling.

Decision: **DEEP STUDY / COPY-CANDIDATE FOR NARROW GIT-SAFETY OR APPROVAL TEST PATTERNS AFTER PINNING**.

Especially valuable for T045+ and later Agent Fleet workspace allocation. Winds should not automatically execute repository project hooks merely because Worktrunk supports them; Spec 003 explicitly requires no automatic project bootstrap.

### 5.2 agent-worktree

Repository: `nekocode/agent-worktree`

License: MIT. Implementation: Rust.

Relevant ideas:

- create worktree -> run agent -> inspect outcome -> keep/merge/cleanup;
- failures preserve the worktree;
- direct process execution rather than shell-string composition;
- Bash/Zsh/Fish/PowerShell support;
- explicit metadata under user-owned storage.

Decision: **DEEP STUDY**.

The “failure preserves workspace” behavior aligns closely with Winds recovery philosophy. Its merge UX must not be copied blindly: Winds promotion remains a separate verification/human-decision operation.

### 5.3 Gas Town

Repository: `gastownhall/gastown`

Relevant ideas:

- explicit worker taxonomy;
- persistent worker identity with ephemeral sessions;
- health monitoring and stuck-worker handling;
- queue/refinery verification stage;
- worktree allocation for workers;
- structured work decomposition and handoff;
- OTEL instrumentation.

Decision: **STUDY ONLY / REJECT-IN-CORE architecture**.

Do not copy its daemon-heavy hierarchy, tmux-centric orchestration, role mythology, or broad plugin system into early Winds. The useful concepts are scheduler state, worker health, queue backpressure, retry/escalation semantics, and separation of implementation from verification.

### 5.4 Beads

Repository: `gastownhall/beads`

License: MIT.

Relevant ideas:

- typed dependency graph;
- deterministic `ready` work detection;
- `blocks`, `related`, `parent-child`, and `discovered-from` relationships;
- JSON machine output;
- persistent task memory.

Decision: **STUDY DATA MODEL; DO NOT ADOPT DOLT FOR WINDS CORE**.

For a future Fleet scheduler, a small dependency-aware task graph may be justified. It should live in the existing Winds persistence model unless demonstrated scale/merge requirements demand a separate engine.

## 6. Tier 1 — usage, quota, cost, and model metadata donors

### 6.1 ccusage

Repository: `ryoppippi/ccusage`

License: MIT.

Relevant ideas:

- reads local usage data from many coding CLIs including Claude, Codex, OpenCode, Amp, Droid, Codebuff, Hermes, Pi, Goose, OpenClaw, Kilo, Kimi, Qwen, Copilot CLI, and Gemini;
- normalizes sessions/projects/time windows;
- cost calculation from version-pinned pricing data;
- supports offline/deterministic pricing snapshots.

Decision: **DEEP STUDY / POSSIBLE EXTERNAL INGESTION FIRST**.

Winds should prefer provider/agent-reported structured usage when available, use local-session parsers as explicitly sourced fallback telemetry, and never fabricate missing token counts. Historical cost must record pricing source/version; recomputing old executions using today’s price table would destroy auditability.

### 6.2 CodexBar

Repository: `steipete/CodexBar`

License: MIT.

Relevant ideas:

- dozens of provider IDs and explicit provider descriptors;
- ordered fetch strategies per provider: OAuth/API/CLI/local/web fallback;
- per-provider quota/reset windows;
- multiple accounts;
- local cost scan;
- status/incident polling;
- host APIs that isolate Keychain/browser/PTY/HTTP/token-cost concerns from provider-specific code;
- JSON CLI output and localhost-only server.

Decision: **DEEP STUDY; COPY-CANDIDATE ONLY FOR PROVIDER-NEUTRAL PARSING/TEST PATTERNS AFTER EXACT PATH AUDIT**.

Winds must not silently import browser cookies or credentials. Usage/quota collection needs explicit source labels and a secret-handling policy.

### 6.3 models.dev

Repository: `anomalyco/models.dev`

License: MIT.

Relevant ideas:

- provider-agnostic model metadata separated from provider-specific serving/pricing facts;
- model capabilities, context/output limits, modalities, release dates, weights/license metadata;
- provider-specific cost fields and overrides;
- generated machine-readable catalog/API.

Decision: **INTEGRATE / SNAPSHOT WITH PROVENANCE**.

For routing and cost estimation, Winds should snapshot the exact metadata/pricing revision used for a decision. Model metadata is advisory and must not override facts returned by the actual agent/provider.

## 7. Tier 1 — terminal and shell observability donors

### 7.1 Atuin

Repository: `atuinsh/atuin`

License: MIT. Implementation: Rust.

Relevant idea: shell-native command lifecycle hooks capture command text/cwd at pre-exec and exit code/duration at post-command across multiple shells.

Decision: **DEEP STUDY FOR T054**.

Winds should inject integration ephemerally into Winds-created sessions rather than editing persistent dotfiles. Shell hook facts remain `SHELL_REPORTED`, not `WINDS_OBSERVED`, unless independently proven.

### 7.2 VS Code shell integration

Repository: `microsoft/vscode` / shell integration scripts and docs.

License: MIT for source scripts.

Relevant ideas:

- OSC 633 prompt/preexec/completion/cwd markers;
- explicit command line reporting;
- optional nonce to reduce marker spoofing;
- per-shell injection logic;
- command detection falls back or disables itself when unreliable.

Decision: **DEEP STUDY / POSSIBLE NARROW COPY-CANDIDATE WITH ATTRIBUTION AFTER PATH AUDIT**.

The nonce model is particularly relevant to Spec 003’s requirement to test marker spoof/confusion. A nonce improves source confidence but does not turn shell-reported telemetry into authoritative process observation.

### 7.3 Microsoft Terminal / ConPTY

Repository: `microsoft/terminal`

License: MIT.

Relevant ideas:

- canonical Windows Console/ConPTY implementation;
- pseudoconsole lifecycle;
- VT input/output model;
- Windows process/terminal behavior.

Decision: **STUDY / PLATFORM REFERENCE**, not a source donor for a bespoke Winds terminal engine.

Winds should continue using a proven PTY/ConPTY abstraction rather than copying Windows Terminal internals.

### 7.4 WezTerm / portable-pty

Repository: `wezterm/wezterm`, `pty/` package lineage.

Decision already recorded by T043: `portable-pty 0.9.0` is the approved preferred dependency candidate, but it is not yet landed. The first runtime PR that uses it must pin the exact version, commit the resolved lockfile, and re-audit the actual transitive/license set.

Decision here: **NO CHANGE TO T043**.

## 8. Tier 1 — deterministic verification and review donors

### 8.1 ast-grep

Repository: `ast-grep/ast-grep`

Implementation: Rust; structural search/lint/rewrite using tree-sitter.

Decision: **INTEGRATE AS OPTIONAL VERIFIER / STUDY RULE MODEL**.

Useful for deterministic structural policy checks. It must not become a mandatory dependency merely to duplicate compiler/linter checks already provided by a project.

### 8.2 Difftastic

Repository: `Wilfred/difftastic`

License: MIT; vendored parsers have their own MIT/Apache licenses.

Decision: **INTEGRATE FOR HUMAN CANDIDATE COMPARISON; DO NOT USE AS SUCCESS AUTHORITY**.

A structural diff is excellent UX when comparing independent agent candidates, but “smaller/nicer diff” is not an automatic winner score.

### 8.3 reviewdog

Repository: `reviewdog/reviewdog`

License: MIT.

Relevant ideas: normalize many diagnostics into a common rich format, filter findings by diff, and preserve severity/rule/location/suggestion metadata.

Decision: **STUDY DIAGNOSTIC CONTRACT; POSSIBLE EXTERNAL INTEGRATION**.

Winds should likely define a small internal verifier-finding record inspired by SARIF/RDFormat rather than make reviewdog itself a core dependency.

### 8.4 Gitleaks

Repository: `gitleaks/gitleaks`

License: MIT for the scanner CLI.

Important licensing boundary: `gitleaks/gitleaks-action` is a separate product with a commercial license; do not treat the GitHub Action as MIT simply because the scanner is MIT.

Decision: **OPTIONAL EXTERNAL VERIFIER**.

No secret scanner can prove absence of secrets, so findings are evidence and a clean scan is not a confidentiality guarantee.

## 9. Tier 1 — sandbox and isolation donors

### 9.1 OpenSandbox

Repository: `opensandbox-group/OpenSandbox`

License: Apache-2.0.

Relevant surfaces:

- sandbox lifecycle APIs;
- Docker/Kubernetes runtime backends;
- in-sandbox command/filesystem execution daemon;
- ingress/egress controls;
- SDK/CLI/MCP client surfaces;
- explicit public protocol vs runtime provider boundaries.

Decision: **FUTURE EXTERNAL SANDBOX INTEGRATION / STUDY**, not a dependency for Spec 003 or first Agent Fleet slice.

Winds local worktrees are not sandboxes. If remote/strong isolation is later needed, OpenSandbox is a serious provider candidate, but adopting its control plane now would violate Ponytail and the current one-process architecture.

### 9.2 OpenAI Codex sandbox implementation

Repository: `openai/codex`

License: Apache-2.0. Predominantly Rust.

Relevant ideas:

- approval-policy modeling;
- platform-specific sandbox implementation;
- app-server structured control surface;
- Windows/Linux/macOS process behavior;
- explicit distinction between sandbox policy and approval policy.

Decision: **DEEP STUDY; DO NOT FORK CODE WHOLESALE**.

Winds can learn from Codex’s sandbox boundaries but must keep its own product claim precise: an external agent’s sandbox policy is not equivalent to Winds verification evidence or a guarantee that the user workspace is isolated.

## 10. Tier 2 — agent/client architecture references

### 10.1 Goose

Repository: `aaif-goose/goose`

License: Apache-2.0. Rust, cross-platform.

Relevant ideas:

- ACP providers;
- multiple model providers;
- MCP extension model;
- API/CLI/desktop surfaces;
- Windows/macOS/Linux portability.

Decision: **STUDY**.

Do not copy a provider/plugin framework into Winds. The useful evidence is that ACP can serve as a practical subscription-backed agent integration path.

### 10.2 OpenCode

Repository: `anomalyco/opencode`

License: MIT.

Relevant ideas:

- client/server separation;
- session/event API;
- provider-neutral model architecture;
- TUI product design;
- explicit security statement that permission UX is not sandbox isolation.

Decision: **STUDY**.

Winds should not adopt an always-on local server merely to imitate OpenCode. Its clear security boundary language is worth emulating.

### 10.3 Ratatui

Repository: `ratatui/ratatui`

License: MIT. Rust.

Decision: **LIKELY ADOPT LATER FOR WINDS TUI** if a rich terminal control surface is authorized.

Do not add it to T044 or backend-only slices.

## 11. Observability contract

### 11.1 OpenTelemetry Rust

Repository: `open-telemetry/opentelemetry-rust`

License: Apache-2.0.

Decision: **ADOPT LATER AT EXPORT BOUNDARY, NOT INTERNAL TRUTH MODEL**.

Winds’ internal execution ledger remains the product record. OTEL is an interoperability/export format, not the source of truth.

### 11.2 OpenTelemetry GenAI semantic conventions

Relevant current fields include input/output/cache/reasoning token usage, operation duration, time to first chunk, requested/response model, provider name, tool/workflow/agent semantics.

The conventions are still evolving. Decision: **VERSION-PIN AND MAP, DO NOT MIRROR BLINDLY INTO SQLITE COLUMNS**.

Spec 005 should define a versioned mapping from Winds typed LLM records to a pinned OTEL GenAI semantic-convention revision. If usage cannot be obtained efficiently/reliably, Winds should report `UNKNOWN`, consistent with the convention’s guidance not to emit invented token metrics.

## 12. Recommended Winds Agent Fleet architecture

```text
User / Winds TUI
       |
       v
Fleet Controller
  - task brief
  - explicit policy/budget
  - scheduler/queue
  - reviewer separation
       |
       +--------------------+---------------------+
       |                    |                     |
       v                    v                     v
  ACP transport      Native structured       CLI compatibility
  (preferred)        agent interfaces         relays
       |                    |                     |
       v                    v                     v
Codex/Claude/...       vendor app/API       long-tail agents
       |
       v
Per-agent execution workspace/worktree
       |
       v
Common Winds execution ledger
       |
       +--> agent-reported result/usage
       +--> Winds-observed process/Git facts
       +--> shell-reported telemetry
       +--> deterministic verifier findings
       |
       v
Candidate verification
       |
       v
Independent review
       |
       v
Explicit human promotion decision
```

## 13. Capability registry, not agent-specific product logic

A future Agent Fleet should discover capabilities such as:

- structured protocol: ACP/native/CLI;
- interactive vs headless;
- read-only enforcement level;
- workspace write support;
- session resume/fork;
- permission request support;
- model/effort selection;
- token usage availability;
- nested subagent telemetry;
- terminal/tool event availability;
- supported host platforms;
- exact executable/version/auth readiness.

The registry must distinguish **declared/catalog capabilities** from **locally observed capabilities**. Local launch-time reality wins.

Do not encode “Claude is best at architecture” or “Codex is best at implementation” as permanent product truth. Future routing may learn from the user’s own verified history, but routing reasons must remain inspectable and bounded by explicit budget/policy.

## 14. Scheduler principles borrowed from donors

A first Fleet scheduler should be much smaller than Gas Town:

- bounded concurrency;
- queued/running/waiting/succeeded/failed/cancelled state;
- dependency-aware ready work only when a real use case requires dependencies;
- exact workspace assignment;
- timeout/cancel semantics;
- retry only when policy allows and the failure class is retryable;
- preserve failed workspaces for inspection;
- no silent re-use of a dirty or ambiguous workspace;
- implementation and independent review are separate roles;
- no automatic winner selection.

No daemon is required for the first slice. If the controlling Winds process exits, recovery should describe persisted state truthfully instead of pretending running agents remain owned.

## 15. Usage, quota, and cost authority model

Future records should carry source/provenance for every usage/cost fact.

Suggested source classes:

- `PROVIDER_REPORTED`
- `AGENT_REPORTED`
- `LOCAL_LOG_PARSED`
- `WINDS_OBSERVED` (for wall-clock/process facts Winds directly measures)
- `DERIVED_FROM_PINNED_PRICING`
- `UNKNOWN`

Example: a provider-reported token count can be stored as provider-reported. Cost derived from that count and a pinned pricing snapshot is derived, not provider-reported. A subscription quota estimate from a browser/API fallback is a different source again.

## 16. Security boundaries

- A worktree is not a sandbox.
- Agent permission prompts are not OS isolation.
- ACP authentication does not make agent output authoritative verification evidence.
- A shell integration nonce can reduce marker spoofing but does not make shell-hook facts Winds-observed.
- Provider usage collectors may touch credentials/cookies/local logs; Winds must require explicit policy and minimize secret persistence.
- Agent discovery must not automatically install or execute catalog entries.
- Repository project hooks/config must not auto-run on workspace open.
- External sandbox providers must have their own capability and trust boundary rather than being hidden behind a generic “secure” boolean.

## 17. What Winds should not copy

- Gas Town daemon/Mayor/Witness/Deacon hierarchy.
- tmux as the core session architecture.
- a generic plugin runtime before a concrete product need.
- TUI scraping when structured protocol is available.
- browser-cookie usage collection by default.
- model/provider routing that cannot explain why a route was chosen.
- merge queues that can bypass Winds verification/human selection.
- whole terminal emulators/renderers in the Rust core.
- SQL/LLM fields in the current Spec 003 execution schema.
- OpenTelemetry attributes as direct database schema without a typed Winds contract.

## 18. Proposed follow-on research/Spec sequence

This dossier recommends, but does not itself authorize, a later specification tentatively named:

**Spec 006 — Agent Fleet & Delegation Control Plane**

Recommended ordering:

1. Finish and accept Spec 003 persistence/workspace/terminal spine.
2. Define the minimal Agent Fleet user stories and measurable outcomes.
3. Pin exact ACP Rust SDK/protocol revision.
4. Prove one ACP-controlled Codex session and one ACP-controlled Claude session in isolated workspaces.
5. Add a long-tail CLI compatibility adapter only after the structured path works.
6. Prove parallel isolated candidates with deterministic verification.
7. Add independent reviewer assignment.
8. Add usage/cost/quota telemetry only after execution identity is stable.
9. Add richer task dependency scheduling only when real concurrent workflows require it.
10. Add TUI fleet dashboard after backend records and lifecycle are proven.

## 19. Immediate implementation decision

**Do not start Agent Fleet code now.**

The immediate engineering path remains T044 -> T045... according to canonical Spec 003. This research changes the future design direction but does not broaden the active slice.

The most consequential conclusions to carry forward are:

1. ACP should be the preferred universal agent transport.
2. `delegate-skills` is a strong long-tail CLI compatibility and negative-test donor, not the core protocol.
3. Worktrunk/agent-worktree are strong worktree-safety donors.
4. ccusage/CodexBar/models.dev can eliminate large amounts of bespoke usage/quota/pricing discovery work.
5. Atuin/VS Code provide the strongest shell-lifecycle reference patterns for later T054.
6. Gas Town/Beads provide scheduler/task-graph ideas, but their heavy architecture should not be imported.
7. OpenSandbox is a future isolation provider, not a reason to turn Winds into a sandbox platform today.
8. Verification remains the Winds moat: many tools orchestrate agents; Winds must independently prove exact candidate results.

## 20. Source index

Primary repositories inspected in this research:

- https://github.com/agentclientprotocol/rust-sdk
- https://github.com/agentclientprotocol/registry
- https://github.com/agentclientprotocol/codex-acp
- https://github.com/agentclientprotocol/claude-agent-acp
- https://github.com/openclaw/acpx
- https://github.com/amElnagdy/delegate-skills
- https://github.com/coder/agentapi
- https://github.com/max-sixty/worktrunk
- https://github.com/nekocode/agent-worktree
- https://github.com/gastownhall/gastown
- https://github.com/gastownhall/beads
- https://github.com/ryoppippi/ccusage
- https://github.com/steipete/CodexBar
- https://github.com/anomalyco/models.dev
- https://github.com/atuinsh/atuin
- https://github.com/microsoft/vscode
- https://github.com/microsoft/terminal
- https://github.com/wezterm/wezterm
- https://github.com/ast-grep/ast-grep
- https://github.com/Wilfred/difftastic
- https://github.com/reviewdog/reviewdog
- https://github.com/gitleaks/gitleaks
- https://github.com/opensandbox-group/OpenSandbox
- https://github.com/openai/codex
- https://github.com/aaif-goose/goose
- https://github.com/anomalyco/opencode
- https://github.com/ratatui/ratatui
- https://github.com/open-telemetry/opentelemetry-rust
- https://github.com/open-telemetry/semantic-conventions-genai

Secondary projects should remain reference-only until separately audited and pinned.
