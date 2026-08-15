# Research: Workspace Execution Spine

**Date**: 2026-08-15

**Scope**: Primary-source architecture/donor research for Spec 003. All projects listed here are reference-only unless a later task records an exact dependency/code-reuse decision with commit/version/license provenance.

## Decision Summary

Winds should not try to become “the best terminal, SQL client, and LLM platform” by merging three unrelated applications. The reusable advantage is a precise workspace/execution spine:

- exact repository/workspace identity;
- explicit execution domain;
- real terminal/session control;
- one local lifecycle/timing/authority ledger;
- typed domain records for shell now, SQL and LLM later.

The strongest donor strategy is selective:

- PTY/session mechanics from mature Rust terminal projects;
- WSL behavior from Microsoft documentation;
- shell lifecycle concepts from Atuin;
- environment trust/discovery concepts from mise;
- SQL UX/client concepts from Harlequin/usql with Rust parsing from sqlparser-rs;
- LLM metric semantics from OpenTelemetry GenAI, with Langfuse/LiteLLM as observability/cost feature references.

Do not wholesale-copy multiplexer, daemon, plugin, SQL adapter, or LLM gateway architectures into Spec 003.

## Terminal and Workspace References

### WezTerm / `portable-pty`

Primary source: https://github.com/wezterm/wezterm

Observed role:

- cross-platform terminal emulator/multiplexer implemented predominantly in Rust;
- upstream ecosystem contains the `portable-pty` crate used to allocate and control pseudoterminals across platforms;
- demonstrates mature handling of terminal lifecycle, PTY I/O, resizing, and platform differences.

Winds decision:

- **candidate runtime dependency for PTY/ConPTY only**;
- do not adopt the WezTerm GUI, multiplexer, Lua configuration, daemon/mux architecture, or unpublished internal terminal model;
- T043 must verify the exact current `portable-pty` release/source, MSRV compatibility, license, Windows behavior, and dependency footprint before adoption;
- if code is copied/adapted rather than consumed as a crate, exact upstream path/commit/license/modification provenance becomes mandatory.

Why not hand-write PTY first:

- PTY lifecycle has platform-specific correctness traps around raw mode, process ownership, controlling terminals, Windows pseudoconsoles, resize, and exit/reaping;
- a proven dependency is likely smaller and safer than inventing a new portability layer.

### Zed terminal implementation

Primary source: https://github.com/zed-industries/zed/tree/main/crates/terminal_view

Observed design notes:

- terminal implementation separates terminal/PTY event-loop concerns from integration into the UI framework;
- TTY creation is fallible and modeled explicitly before UI subscription;
- Zed work on persistent sessions illustrates that process/PTY ownership and terminal-state replay become substantially more complex when sessions outlive the main application process.

Winds decision:

- **reference only**;
- copy the separation principle: prove backend PTY/session ownership before binding it to the eventual UI;
- defer persistent live sessions across Winds restarts because they imply an external process/daemon plus a reconnection/state protocol that Spec 003 does not need.

### Zellij

Primary source: https://github.com/zellij-org/zellij

Observed capabilities relevant to Winds:

- Rust terminal workspace/multiplexer;
- explicit sessions, panes, shell/cwd configuration, session serialization/resurrection, CLI session control, and programmatic pane actions;
- recent releases expose rich CLI control and session metadata.

Winds decision:

- **UX/session-lifecycle reference only**;
- useful concepts: session identity, explicit switch/attach semantics, running-command metadata, resize/input/close control, session-manager organization;
- do not adopt Zellij's server/client protocol, web client, WASM plugin system, layout manager, collaboration model, or multiplexer breadth in Spec 003;
- Winds UI may later learn from pane/session organization after the execution spine is stable.

### Ghostty / `libghostty`

Primary source: https://github.com/ghostty-org/ghostty

Observed capability:

- Ghostty exposes `libghostty`/`libghostty-vt` as an embeddable terminal-state/VT library across macOS, Linux, Windows, and WebAssembly;
- upstream explicitly describes the API as usable but still in flux/not yet version-stable.

Winds decision:

- **defer**;
- do not add a terminal renderer/VT parser to Spec 003;
- revisit when the graphical terminal view is specified, comparing then-current `libghostty`, xterm.js, and other embedding choices against UI stack, API stability, performance, accessibility, and license/provenance requirements.

## Windows / WSL

### Microsoft WSL command surface

Primary source: https://learn.microsoft.com/windows/wsl/basic-commands

Relevant supported behavior:

- `wsl --list --verbose` / related list forms expose installed distributions and WSL version/state information;
- `wsl --distribution <Distribution Name>` selects a specific distribution;
- WSL commands can be invoked as `wsl.exe` from Linux contexts when crossing back to Windows;
- WSL provides explicit status/version/update/terminate interfaces.

Winds decision:

- treat Microsoft WSL CLI behavior as normative;
- discover distributions through supported CLI output rather than Windows registry reverse engineering;
- represent Windows and every WSL distribution as distinct execution domains;
- validate effective cwd/repository identity after launch instead of trusting string-only path conversion;
- do not use WSL terminate/unregister/destructive lifecycle operations as ordinary Winds workspace behavior.

## Shell History / Lifecycle

### Atuin

Primary sources:

- https://github.com/atuinsh/atuin
- https://github.com/atuinsh/atuin/blob/main/docs/docs/guide/shell-integration.md

Observed model:

- Atuin records shell history with SQLite and adds command context;
- its shell integration uses lifecycle hooks: pre-exec records command/timestamp/cwd and post-command records exit code/duration;
- hook availability depends on interactive shell startup/integration.

Winds decision:

- **borrow the lifecycle concept, not the product**;
- command-level execution records must come from a reliable shell integration or explicit Winds command, never from guessing raw terminal keystrokes;
- inject integration ephemerally into Winds-created shells rather than editing `.bashrc`, `.zshrc`, PowerShell profiles, etc.;
- session-level terminal control must still work when command hooks are unavailable.

## Environment Organization / Trust

### mise

Primary sources:

- https://github.com/jdx/mise
- https://github.com/jdx/mise-docs/blob/main/configuration.md

Observed model:

- project-local tool versions, environment variables, and tasks can be described through hierarchical configuration;
- configuration can affect command execution and therefore has a trust boundary;
- current mise work includes explicit handling for untrusted config/worktrees.

Winds decision:

- **reference only** for environment inventory/trust UX;
- detect common environment/tool manifests without running them on open/clone;
- later environment activation should be an explicit trusted action, ideally using the user's installed environment manager rather than reimplementing hundreds of tool installers;
- do not create a Winds package/runtime manager in Spec 003.

## SQL Follow-On Research

SQL is explicitly requested as a first-class future Winds surface. It is intentionally separated from Spec 003 implementation so connection/query behavior does not distort terminal/workspace primitives.

### Harlequin

Primary source: https://github.com/tconbeer/harlequin

Observed strengths:

- terminal-native SQL IDE rather than a thin query runner;
- editor + data catalog + query execution + export workflow;
- DuckDB/SQLite built-in support plus adapters for additional databases;
- configurable profiles and database-specific connection behavior;
- Postgres adapter exposes manual transaction mode.

Winds use:

- **UX/reference donor** for the future SQL Studio: query editor flow, catalog/schema navigation, profile UX, transaction visibility, result export;
- do not copy its Python plugin architecture into Rust Winds.

### usql

Primary source: https://github.com/xo/usql

Observed strengths:

- one CLI across many SQL/NoSQL databases;
- psql-like commands, syntax highlighting, context completion, multiple database support, and cross-database copying.

Winds use:

- **behavior/reference donor** for universal connection UX and interactive database workflows;
- future Winds SQL should not claim universal dialect support until each driver/dialect is actually tested.

### Apache DataFusion `sqlparser-rs`

Primary source: https://github.com/apache/datafusion-sqlparser-rs

Observed strengths:

- Rust SQL lexer/parser with ANSI and multiple dialect support;
- designed as a foundation for query engines and SQL analysis;
- visitor/AST support can enable statement classification and editor intelligence.

Winds use:

- **future Rust dependency candidate** for dialect-aware parsing, classification, formatting/intelligence support;
- parser classification must never override database-server truth;
- write-risk gating must fail conservatively when syntax/dialect cannot be classified reliably.

### SQL Studio quality target

The later Spec 004 should include:

- secret-safe connection profiles;
- Postgres first unless evidence supports a broader first slice;
- schema/catalog browser;
- strong context completion;
- multi-query tabs/history linked to workspace;
- explicit auto/manual transactions;
- visible read/write classification and confirmation for destructive statements;
- client/server timing, rows affected/returned, cancellation;
- EXPLAIN/EXPLAIN ANALYZE visualization/artifacts where supported;
- bounded result persistence/export;
- SQLite/DuckDB convenience where useful without confusing embedded/local databases with production connections.

## LLM Follow-On Research

### OpenTelemetry GenAI semantic conventions

Primary sources:

- https://github.com/open-telemetry/semantic-conventions-genai
- https://github.com/open-telemetry/semantic-conventions-genai/blob/main/docs/gen-ai/gen-ai-metrics.md

Observed semantics relevant to Winds:

- provider/model and operation identity;
- input/output token usage;
- cache creation/read input tokens;
- reasoning output tokens when applicable;
- client operation duration;
- time-to-first-chunk/token style streaming metrics are being developed;
- conventions are evolving and must be version-pinned when adopted.

Winds decision:

- **normative semantic reference for future LLM observability**;
- use provider-reported usage when available; do not invent token counts when the provider does not expose them unless an explicitly labeled offline estimator is enabled;
- maintain requested model separately from actual response model when providers return both;
- exact OTel convention/schema version must be pinned in Spec 005.

### Langfuse

Primary source: https://github.com/langfuse/langfuse

Observed strengths:

- open-source LLM observability, traces, sessions, metrics, evaluations, prompt management, datasets/playground;
- tracks model calls plus retrieval/tool/agent logic and supports token/cost analysis;
- session grouping/replay is especially relevant to a workspace execution timeline.

Winds use:

- **product/UX reference only**;
- Winds should natively show local execution traces without requiring Langfuse;
- later add export/interoperability rather than recreating every hosted analytics/evaluation feature in the core workspace.

### LiteLLM

Primary source: https://github.com/BerriAI/litellm

Observed strengths:

- unified model/provider gateway across many LLM APIs;
- cost tracking, budgets, routing/load balancing, guardrails, and observability callbacks;
- model-cost metadata supports input/output/cache/reasoning and other modality-specific pricing dimensions.

Winds use:

- **gateway/cost-accounting reference only** for Spec 005;
- do not add a Python proxy or universal provider abstraction in Spec 003;
- future cost records must include the pricing source/version because model pricing changes independently of historical executions;
- routing/budget policy should be layered after raw provider/model/token/time/cost observations are correct.

### LLM Observatory quality target

The later Spec 005 should make every model call inspectable at workspace/session level:

- provider;
- requested and actual model;
- request/response identity;
- input/output/cache/reasoning tokens;
- start/end/total latency and time-to-first-token/chunk where available;
- retry/rate-limit/error state;
- tool calls and subagent spans when present;
- exact monetary cost or `UNKNOWN`, with pricing source/version;
- context-window utilization and cache effectiveness;
- per-task/session/workspace aggregate time/tokens/cost;
- privacy controls for prompt/response/tool payload persistence;
- optional OpenTelemetry-compatible export.

## Explicit Non-Adoptions for Spec 003

Do not add solely because donors have them:

- Zellij/WezTerm daemon or multiplexer protocols;
- Zellij WASM plugin system;
- Ghostty renderer/VT library;
- web/remote terminal sharing;
- environment package manager;
- Harlequin Python adapter system;
- universal SQL driver matrix;
- SQL parser dependency before SQL Spec 004;
- Langfuse/ClickHouse stack;
- LiteLLM proxy/gateway;
- OpenTelemetry SDK/exporter dependency before a concrete export requirement;
- MCP/A2A or generic agent runtime.

## Provenance Rule

This research document records conceptual influence only. Before any external runtime code is copied, adapted, vendored, or added as a dependency, the implementation task must record:

- exact upstream repository/package;
- exact release/version/commit;
- upstream license;
- reuse mode (`dependency`, `adapted code`, `copied code`, or `reference only`);
- exact Winds paths affected;
- required notices/attribution;
- update strategy.

No donor's popularity or architecture is authority for Winds correctness; every adopted primitive must still satisfy Winds tests, safety boundaries, Ponytail review, and independent review.
