# Winds External Source Registry

**Status:** provenance/index only; this document does **not** authorize implementation, copying, dependency admission, protocol adoption, or scope expansion.

**Registry date:** 2026-08-18

**Canonical base at creation:** `3ba8df1857407b07de06aca1389c70b41ce58b00`

## Purpose

This registry preserves external sources supplied by the founder and sources discovered while researching founder-requested Winds donor areas so that useful references are not lost between chats.

It is intentionally broader than `docs/provenance/donors.md`:

- `source-registry.md` records **candidate/reference provenance**.
- `donors.md` records projects that materially shape Winds or have reached an approved dependency/process/donor status.
- `docs/research/006-agent-fleet-donor-audit.md` contains deeper technical evaluation for many Agent Fleet candidates.

A source appearing here is **not admitted**. Before copying/adapting code or adding a dependency, Winds must still pin the exact upstream commit/path, verify license/notice obligations, classify reuse mode, justify why integration or smaller Winds-authored code is insufficient, add deterministic tests, and pass correctness/safety + Ponytail + independent review.

## Provenance classes

- **FOUNDER_SUPPLIED** — URL was explicitly supplied by the founder in Winds or adjacent agent/developer-tooling research.
- **RESEARCH_DISCOVERED** — URL was found while researching a founder-requested Winds capability/source family.
- **DERIVED_CANONICAL** — canonical repository/source was resolved from a founder-supplied website/account.
- **UNADMITTED** — reference only; no runtime/code/dependency authority.

## Founder-supplied Winds / adjacent developer-tooling sources

| Source | URL | Provenance | Winds relevance | Admission state |
|---|---|---|---|---|
| delegate-skills | https://github.com/amElnagdy/delegate-skills | FOUNDER_SUPPLIED | Heterogeneous CLI delegation, relay/result contracts, touched-file discovery, process-tree/negative-test ideas | Already tracked as future donor in `donors.md`; no runtime dependency |
| Graphify | https://github.com/Graphify-Labs/graphify | FOUNDER_SUPPLIED | Possible future code-graph/context reference | UNADMITTED; explicitly outside current Spec 003 runtime |
| AFFiNE | https://github.com/toeverything/affine | FOUNDER_SUPPLIED | Possible future workspace/knowledge UX reference | UNADMITTED; not current runtime architecture |
| Continue | https://continue.dev | FOUNDER_SUPPLIED | Coding-agent / IDE integration reference | UNADMITTED |
| Augment Code | https://www.augmentcode.com | FOUNDER_SUPPLIED | Coding-agent / context/reference product | UNADMITTED |
| Qodo | https://www.qodo.ai | FOUNDER_SUPPLIED | Independent code-review evidence source | External reviewer; not Winds product truth |
| Qodo App | https://app.qodo.ai/home | FOUNDER_SUPPLIED | Review workflow UI/service | External reviewer; not runtime dependency |
| Graphite | https://graphite.com | FOUNDER_SUPPLIED | PR/review/developer-workflow reference | UNADMITTED product/process reference |
| Graphite App | https://app.graphite.com/get-started | FOUNDER_SUPPLIED | PR/review workflow | UNADMITTED product/process reference |
| Cubic | https://www.cubic.dev | FOUNDER_SUPPLIED | Independent code-review evidence source | External reviewer; not Winds product truth |
| Greptile | https://www.greptile.com | FOUNDER_SUPPLIED | Independent code-review / codebase-context reference | External reviewer/reference; not runtime dependency |
| Fern | https://buildwithfern.com/?utm_campaign=buildWith&utm_medium=docs&utm_source=docs.cohere.com | FOUNDER_SUPPLIED | API/documentation tooling reference | UNADMITTED |
| JetBrains | https://www.jetbrains.com | FOUNDER_SUPPLIED | IDE/developer-tooling reference | UNADMITTED |
| Agentica | https://github.com/wrtnlabs/agentica | FOUNDER_SUPPLIED | Agent framework / orchestration reference | UNADMITTED |
| LLM Space | https://github.com/deer-flow/llm-space | FOUNDER_SUPPLIED | Agent workspace/orchestration reference | UNADMITTED |
| DeerFlow | https://github.com/bytedance/deer-flow | FOUNDER_SUPPLIED | Orchestration, subagents, sandbox/memory/skills reference | UNADMITTED; no wholesale architecture transplant |
| OpenSandbox | https://github.com/opensandbox-group/OpenSandbox | FOUNDER_SUPPLIED | Future isolated-execution/sandbox lifecycle reference | UNADMITTED; broad sandboxing remains outside Spec 003 |
| LlamaCoder | https://github.com/nutlope/llamacoder | FOUNDER_SUPPLIED | Possible future UI/preview/evaluation reference | UNADMITTED |
| TencentDB Agent Memory | https://github.com/TencentCloud/TencentDB-Agent-Memory | FOUNDER_SUPPLIED | Future governed agent memory/skills/provenance/ACL reference | UNADMITTED |
| Pi | https://github.com/earendil-works/pi | FOUNDER_SUPPLIED | Agent CLI/runtime reference | UNADMITTED; no Pi transplant in Spec 003 |
| OpenJarvis | https://openjarvis.stanford.edu | FOUNDER_SUPPLIED | Agent/runtime research reference | UNADMITTED |
| OpenWhispr | https://openwhispr.com | FOUNDER_SUPPLIED | Voice/input tooling reference | UNADMITTED / peripheral |
| OpenPipe | https://openpipe.ai | FOUNDER_SUPPLIED | Model/evaluation/observability reference | UNADMITTED |
| Herdr website | https://herdr.dev | FOUNDER_SUPPLIED | Agent terminal workspace/runtime reference | UNADMITTED; daemon/socket/plugin/persistent-session architecture remains outside Spec 003 |
| nicobailon GitHub account | https://github.com/nicobailon | FOUNDER_SUPPLIED | Requested Pi extension ecosystem audit | Account-level source; individual repositories listed below |

## Herdr — derived canonical source

Founder supplied `https://herdr.dev`. The associated canonical repository resolved during research is:

- https://github.com/herdrdev/herdr — **DERIVED_CANONICAL / UNADMITTED**.

Observed research facts at registry creation: Rust project; package metadata identifies Herdr as a terminal workspace manager for AI coding agents; inspected package metadata reported Apache-2.0 and `portable-pty = 0.9.0`. Its server/socket/API/plugin/persistent-session design is useful future donor evidence but is intentionally **not** imported into Spec 003.

## nicobailon Pi ecosystem snapshot

Founder supplied `https://github.com/nicobailon` and requested all Pi extensions be checked. A current GitHub repository search on 2026-08-18 returned the following 28 Pi-related repositories. These are preserved as **RESEARCH_DISCOVERED / UNADMITTED** references; inclusion does not mean they have been license-audited or approved.

- https://github.com/nicobailon/pi-subagents
- https://github.com/nicobailon/pi-mcp-adapter
- https://github.com/nicobailon/pi-web-access
- https://github.com/nicobailon/pi-intercom
- https://github.com/nicobailon/pi-messenger
- https://github.com/nicobailon/pi-powerline-footer
- https://github.com/nicobailon/pi-annotate
- https://github.com/nicobailon/pi-interactive-shell
- https://github.com/nicobailon/pi-boomerang
- https://github.com/nicobailon/pi-interview-tool
- https://github.com/nicobailon/pi-design-deck
- https://github.com/nicobailon/pi-rewind-hook
- https://github.com/nicobailon/pi-coordination
- https://github.com/nicobailon/pi-prompt-template-model
- https://github.com/nicobailon/pi-review-loop
- https://github.com/nicobailon/pi-skill-palette
- https://github.com/nicobailon/pi-discord
- https://github.com/nicobailon/pi-model-switch
- https://github.com/nicobailon/pi-side-chat
- https://github.com/nicobailon/pi-foreground-chains
- https://github.com/nicobailon/pi-custom-compaction
- https://github.com/nicobailon/mcp-to-pi-tools
- https://github.com/nicobailon/pi-mono
- https://github.com/nicobailon/pi-autoresearch
- https://github.com/nicobailon/pi-subagent-enhanced
- https://github.com/nicobailon/pi-memory-workbench
- https://github.com/nicobailon/pi-extensions
- https://github.com/nicobailon/pi-tool-display

The same broad search also returned `nicobailon/picpaster`; it is intentionally excluded from the Pi-related registry because the name match alone is insufficient evidence that it belongs to the Pi agent-extension ecosystem.

## Research-discovered sources from founder-requested Winds donor work

These were surfaced while executing the founder's request to search open-source code useful for Winds. They are preserved separately so assistant-discovered sources are never confused with founder-supplied URLs.

### Agent interoperability and control

- https://github.com/agentclientprotocol/rust-sdk
- https://github.com/agentclientprotocol/registry
- https://github.com/agentclientprotocol/codex-acp
- https://github.com/agentclientprotocol/claude-agent-acp
- https://github.com/openclaw/acpx
- https://github.com/coder/agentapi
- https://github.com/asheshgoplani/agent-deck
- https://github.com/awslabs/cli-agent-orchestrator

### Worktree / scheduling / multi-agent lifecycle

- https://github.com/max-sixty/worktrunk
- https://github.com/nekocode/agent-worktree
- https://github.com/gastownhall/gastown
- https://github.com/gastownhall/beads
- https://github.com/workstream-labs/workstreams

### Usage / model / terminal / verification references already discussed in the Agent Fleet donor audit

- https://github.com/ryoppippi/ccusage
- https://github.com/steipete/CodexBar
- https://github.com/anomalyco/models.dev
- https://github.com/atuinsh/atuin
- https://github.com/microsoft/vscode
- https://github.com/microsoft/terminal
- https://github.com/wezterm/wezterm
- https://github.com/ast-grep/ast-grep
- https://github.com/Wilfred/difftastic

For detailed decisions and any pinned snapshots already researched, see `docs/research/006-agent-fleet-donor-audit.md`. This registry does not supersede that audit.

## Existing accepted process/dependency provenance

The following are already canonical in `docs/provenance/donors.md` and are listed here only for registry completeness:

- https://github.com/github/spec-kit — Spec Kit process reference.
- https://github.com/DietrichGebert/ponytail — Ponytail review/process reference.
- https://github.com/HKUDS/DeepCode — review-methodology reference.
- https://github.com/wezterm/wezterm — `portable-pty` lineage; exact accepted dependency details live in `donors.md` and the lock audit.
- https://github.com/retep998/winapi-rs — exact license-text provenance for accepted target crates.

## Scope firewall

This registry must never be used as evidence that a feature is authorized.

In particular, as of the canonical base recorded above:

- T061 is closed/canonical.
- T062 real Windows+WSL2 integration evidence is the next unchecked Spec 003 platform task.
- Herdr/Pi/Agent Fleet/daemon/server/socket/public protocol/plugin/provider/MCP/ACP/A2A work is **not** silently authorized by this registry.
- Graphify/code-graph, broad sandbox frameworks, agent memory systems, UI frameworks, and provider/model routing remain follow-on work unless a future spec explicitly admits them.

## Maintenance rule

When the founder supplies a new external source relevant to Winds, append it here with:

1. exact URL as supplied;
2. provenance class;
3. why it may matter to Winds;
4. admission state;
5. exact commit/license/path only after an audit is actually performed.

Do not silently convert a source-registry entry into an approved donor or runtime dependency.
