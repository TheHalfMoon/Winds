# Herdr Exhaustive Capability Ledger — Required Winds Parity Scope

**Status:** Research/provenance evidence only. This ledger does not itself authorize implementation.

**Herdr exact pin:** `herdrdev/herdr@da6bcd5969779bfe0396bcf89a8025d4375d611e`
**Herdr tree:** `aece03633c003ba0fce01bc2564ad14e0fe9cac9`
**Observed root license:** Apache-2.0
**Founder direction:** all Herdr capabilities are required parity scope for Winds; direct source reuse is permitted by Founder assertion, subject to per-slice provenance/notice/vendor review.

## Completeness contract

This file is the omission-prevention ledger for the Founder-directed Herdr parity program. A later implementation program may improve, redesign, or replace a Herdr mechanism, but it may not silently drop a capability recorded here. Full parity closeout requires every row to reach one explicit disposition: `PROVEN_WINDS_PARITY`, `PROVEN_WINDS_SUPERSET`, or `FOUNDER_ACCEPTED_NOT_APPLICABLE`. Research-only, deferred, partial, or undocumented rows do not satisfy closeout.

The ledger is source-derived from four surfaces at the exact pin: (1) the complete 104-method public serialized socket/API enum, (2) all 24 compiled agent-detection families, (3) all 17 frozen serialized integration targets **plus one experimental CLI-only Letta installable target**, and (4) feature-bearing runtime/client/config/plugin/test modules that expose product behavior beyond the public API. The explicit rows below therefore total **218** baseline capability entries: 24 agent rows + 18 integration rows + 104 public API rows + 72 additional runtime/UI/transport/config/plugin/packaging rows. A fresh exact-pin reconciliation is required before each successor Spec entry and again before final closeout.

## Agent detection families — all required

| ID | Herdr capability | Source evidence | Winds target | Required disposition |
|---|---|---|---|---|
| A01 | Detect and classify `Pi` agent sessions | `src/detect/manifests/pi.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A02 | Detect and classify `Claude` agent sessions | `src/detect/manifests/claude.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A03 | Detect and classify `Codex` agent sessions | `src/detect/manifests/codex.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A04 | Detect and classify `Gemini` agent sessions | `src/detect/manifests/gemini.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A05 | Detect and classify `Cursor` agent sessions | `src/detect/manifests/cursor.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A06 | Detect and classify `Devin` agent sessions | `src/detect/manifests/devin.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A07 | Detect and classify `Antigravity` agent sessions | `src/detect/manifests/antigravity.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A08 | Detect and classify `Cline` agent sessions | `src/detect/manifests/cline.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A09 | Detect and classify `Omp` agent sessions | `src/detect/manifests/omp.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A10 | Detect and classify `Mastracode` agent sessions | `src/detect/manifests/mastracode.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A11 | Detect and classify `OpenCode` agent sessions | `src/detect/manifests/opencode.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A12 | Detect and classify `GithubCopilot` agent sessions | `src/detect/manifests/github-copilot.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A13 | Detect and classify `Kimi` agent sessions | `src/detect/manifests/kimi.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A14 | Detect and classify `Kiro` agent sessions | `src/detect/manifests/kiro.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A15 | Detect and classify `Droid` agent sessions | `src/detect/manifests/droid.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A16 | Detect and classify `Amp` agent sessions | `src/detect/manifests/amp.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A17 | Detect and classify `Grok` agent sessions | `src/detect/manifests/grok.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A18 | Detect and classify `Hermes` agent sessions | `src/detect/manifests/hermes.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A19 | Detect and classify `Kilo` agent sessions | `src/detect/manifests/kilo.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A20 | Detect and classify `Qodercli` agent sessions | `src/detect/manifests/qodercli.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A21 | Detect and classify `Qwen` agent sessions | `src/detect/manifests/qwen.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A22 | Detect and classify `Letta` agent sessions | `src/detect/manifests/letta.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A23 | Detect and classify `Maki` agent sessions | `src/detect/manifests/maki.toml`; `src/detect/mod.rs` | Spec 012 | Required |
| A24 | Detect and classify `Muse` agent sessions | `src/detect/manifests/muse.toml`; `src/detect/mod.rs` | Spec 012 | Required |

## Installable integration targets — all required

The frozen generation-1 serialized `IntegrationTarget::ALL` enum contains I01–I17. I18 records the separately installable experimental CLI-only Letta path, which is intentionally outside that frozen enum. Both are parity scope.

| ID | Herdr capability | Source evidence | Winds target | Required disposition |
|---|---|---|---|---|
| I01 | Install/uninstall/status integration for `Pi` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I02 | Install/uninstall/status integration for `Omp` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I03 | Install/uninstall/status integration for `Claude` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I04 | Install/uninstall/status integration for `Codex` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I05 | Install/uninstall/status integration for `Copilot` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I06 | Install/uninstall/status integration for `Devin` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I07 | Install/uninstall/status integration for `Droid` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I08 | Install/uninstall/status integration for `Kimi` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I09 | Install/uninstall/status integration for `Opencode` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I10 | Install/uninstall/status integration for `Kilo` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I11 | Install/uninstall/status integration for `Hermes` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I12 | Install/uninstall/status integration for `Qodercli` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I13 | Install/uninstall/status integration for `Qwen` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I14 | Install/uninstall/status integration for `Cursor` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I15 | Install/uninstall/status integration for `Mastracode` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I16 | Install/uninstall/status integration for `AntigravityCli` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I17 | Install/uninstall/status integration for `Grok` | `src/api/schema/integrations.rs`; `src/integration/**` | Spec 014 | Required |
| I18 | Experimental CLI-only install/uninstall/status integration for `Letta` outside the frozen generation-1 enum | `src/integration/mod.rs`; `src/cli/integration.rs`; `src/integration/actions.rs`; `src/integration/registry.rs` | Spec 014 | Required |

## 2026-09-18 exact-pin delta note

The immediately preceding audit pin `18061191fdc019498610aee81f0df93f6c2ebd31` is 21 commits behind this ledger pin with no reverse divergence. The delta materially affects remote/local fallback behavior, exact session-deletion targeting, worktree removal with submodules, v0.9.1 release promotion/metadata, OpenCode registration handling, saved-machine connection metadata, native-Windows terminal/input recovery, default Windows shell selection, and Windows input qualification.

The delta does **not** change the 24-agent count, the 104 serialized public-method count, or the 17-member frozen integration enum. The Letta CLI-only experimental integration row is called out explicitly here because the prior research draft did not distinguish it from the frozen target count even though the experimental path already existed.

## Public socket/API capabilities — complete method enum

All 104 serialized methods in `src/api/schema.rs` are individually tracked below. Internal `serde(skip)` stream-control variants are implementation detail and are covered by the graphics/runtime rows in the non-API section.

| ID | Herdr API method | Source evidence | Winds target | Reuse strategy candidate |
|---|---|---|---|---|
| M001 | `ping` | `src/api/schema.rs` | Spec 011 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M002 | `server.stop` | `src/api/schema.rs` | Spec 011 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M003 | `server.live_handoff` | `src/api/schema.rs` | Spec 015 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M004 | `server.reload_config` | `src/api/schema.rs` | Spec 011 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M005 | `server.agent_manifests` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M006 | `server.reload_agent_manifests` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M007 | `notification.show` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M008 | `product_announcement.dismiss` | `src/api/schema.rs` | Spec 015 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M009 | `release_notes.dismiss` | `src/api/schema.rs` | Spec 015 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M010 | `command.invoke` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M011 | `client.window_title.set` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M012 | `client.window_title.clear` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M013 | `client_shell.surface.set` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M014 | `session.snapshot` | `src/api/schema.rs` | Spec 011 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M015 | `workspace.create` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M016 | `workspace.list` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M017 | `workspace.get` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M018 | `workspace.focus` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M019 | `workspace.rename` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M020 | `workspace.move` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M021 | `workspace.move_block` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M022 | `workspace.report_metadata` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M023 | `workspace.close` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M024 | `worktree.list` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M025 | `worktree.create` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M026 | `worktree.open` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M027 | `worktree.remove` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M028 | `tab.create` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M029 | `tab.list` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M030 | `tab.get` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M031 | `tab.focus` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M032 | `tab.rename` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M033 | `tab.move` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M034 | `tab.close` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M035 | `agent.list` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M036 | `agent.get` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M037 | `agent.read` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M038 | `agent.explain` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M039 | `agent.send_keys` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M040 | `agent.rename` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M041 | `agent.view.set` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M042 | `agent.view.clear` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M043 | `agent.focus` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M044 | `agent.start` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M045 | `agent.prompt` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M046 | `agent.wait` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M047 | `pane.split` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M048 | `pane.swap` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M049 | `pane.move` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M050 | `pane.zoom` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M051 | `pane.layout` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M052 | `pane.process_info` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M053 | `layout.export` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M054 | `layout.apply` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M055 | `layout.set_split_ratio` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M056 | `pane.neighbor` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M057 | `pane.edges` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M058 | `pane.focus_direction` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M059 | `pane.resize` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M060 | `pane.scroll` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M061 | `pane.edit_scrollback` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M062 | `pane.selection.read` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M063 | `pane.copy_motion` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M064 | `pane.copy_search` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M065 | `pane.list` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M066 | `pane.current` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M067 | `pane.get` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M068 | `pane.focus` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M069 | `pane.input.set` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M070 | `pane.link.activate` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M071 | `pane.link.resolve` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M072 | `pane.rename` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M073 | `pane.send_text` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M074 | `pane.send_keys` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M075 | `pane.send_input` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M076 | `pane.read` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M077 | `pane.graphics.set` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M078 | `pane.graphics.clear` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M079 | `pane.graphics.info` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M080 | `pane.graphics.stream` | `src/api/schema.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| M081 | `pane.report_agent` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M082 | `pane.report_agent_session` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M083 | `pane.report_metadata` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M084 | `pane.clear_agent_authority` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M085 | `pane.release_agent` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M086 | `pane.close` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M087 | `popup.close` | `src/api/schema.rs` | Spec 012 | TEST_PORT / WINDS_NATIVE_REIMPLEMENTATION candidate |
| M088 | `events.subscribe` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M089 | `events.wait` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M090 | `pane.wait_for_output` | `src/api/schema.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| M091 | `integration.list` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M092 | `integration.install` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M093 | `integration.uninstall` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M094 | `plugin.link` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M095 | `plugin.list` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M096 | `plugin.unlink` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M097 | `plugin.enable` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M098 | `plugin.disable` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M099 | `plugin.action.list` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M100 | `plugin.action.invoke` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M101 | `plugin.log.list` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M102 | `plugin.pane.open` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M103 | `plugin.pane.focus` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| M104 | `plugin.pane.close` | `src/api/schema.rs` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |

## Runtime, UI, transport, configuration, plugin and packaging capabilities

These rows cover product behavior visible in feature-bearing modules/config/tests that is not fully represented by a single public method.

| ID | Herdr capability | Source evidence | Winds target | Reuse strategy candidate |
|---|---|---|---|---|
| R001 | Persistent named sessions | `src/session.rs; src/persist/**; tests/detach_reattach.rs` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R002 | Headless server lifecycle and socket ownership | `src/server/headless/**; src/server/socket_paths.rs; tests/server_headless.rs` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R003 | Detach / reattach | `tests/detach_reattach.rs; src/client/attach.rs` | Spec 011 | TEST_PORT + adapted runtime candidate |
| R004 | Multi-client attachment | `tests/multi_client.rs; src/server/clients.rs` | Spec 011 | TEST_PORT + adapted runtime candidate |
| R005 | Session restore and agent resume | `src/persist/restore.rs; src/agent_resume.rs; config session.resume_agents_on_restore` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R006 | Private IPC and wire protocol | `src/ipc.rs; src/protocol/**; src/api/**` | Spec 011 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R007 | Input lease / takeover semantics | `src/input/lease.rs; src/server/terminal_attach.rs` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R008 | Workspace/tab/pane multiplexer | `src/workspace/**; src/layout.rs; src/pane/**; src/ui/panes.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R009 | Pane split/swap/move/resize/zoom/focus/neighbors/edges | `src/api/schema.rs; src/app/api/panes.rs` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R010 | Layout export/apply/split ratios | `src/layout.rs; layout.* API methods` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R011 | Scrollback read/edit and copy mode | `src/terminal/history_read.rs; src/copy_mode.rs; src/client/shell/copy_mode.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R012 | Selection and word selection | `src/selection.rs; src/client/shell/word_selection.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R013 | Link hover/resolve/activate | `src/client/shell/link_hover.rs; pane.link.* API methods` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R014 | Kitty graphics and direct graphics streaming | `src/kitty_graphics/**; src/client/direct_graphics.rs; pane.graphics.*` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R015 | Terminal modes/effects/theme/title | `src/terminal_modes.rs; src/terminal_effects.rs; src/terminal_theme.rs; src/terminal/title.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R016 | Clipboard forwarding and clipboard images | `src/client/clipboard_forwarding.rs; src/client/clipboard_images.rs; src/server/clipboard_image.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R017 | Notifications and sound | `src/client/notifications.rs; src/server/notifications.rs; src/sound.rs; src/config/sound.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R018 | Agent detection manifests and hot reload | `src/detect/**; server.agent_manifests; server.reload_agent_manifests` | Spec 012 | DIRECT/ADAPTED_COPY + TEST_PORT candidate |
| R019 | Agent state, prompt, wait, attach, start, rename, view | `src/app/agents.rs; src/app/agent_view.rs; agent.* API methods` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R020 | Agent sidebar, sorting, labels and status indicators | `src/client/shell/agent_sidebar.rs; src/config/sidebar.rs; config AgentPanelSortConfig` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R021 | Custom keybindings and indexed navigation | `src/config/keybinds.rs; src/input/keybindings.rs; KeysConfig` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R022 | Command palette/custom commands | `src/app/custom_commands.rs; command.invoke` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R023 | Context menus and overlays | `src/client/shell/context_menu.rs; src/client/shell/overlays.rs; src/client/shell/overlay_input.rs` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R024 | Settings/preferences overlay | `src/client/shell/settings.rs; settings_overlay.rs; preferences.rs` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R025 | Theme system with auto dark/light switching and custom colors | `src/config/theme.rs; src/app/theme_sync.rs` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R026 | Sidebar modes, widths, collapse behavior and mobile threshold | `src/config/model.rs; src/client/shell/sidebar.rs; mobile.rs` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R027 | Tab bar position/right-side metadata/status | `src/config/tab_bar.rs; src/app/tab_bar_status.rs; src/ui/tab_surface.rs` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R028 | Window-title templating and remote title control | `src/config/window_title.rs; src/app/window_title.rs; client.window_title.*` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R029 | Mouse capture/right-click routing/copy-on-select/scroll tuning | `UiConfig; src/client/shell/mouse.rs; src/input/mouse.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R030 | CJK IME cursor/input-source behavior | `ExperimentalConfig; src/client/shell/input_source.rs; src/input/**` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R031 | Saved SSH machine catalog | `src/client/endpoint/catalog.rs; src/cli/spec/machine.rs` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R032 | SSH remote attach and explicit remote sessions | `src/remote/**; tests/remote_attach.rs` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R033 | Remote keybinding modes | `src/remote/args.rs; --remote-keybindings` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R034 | Remote API bridge | `src/remote.rs; src/platform/remote_bridge.rs` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R035 | Multi-machine endpoint navigation | `src/client/endpoint/**; src/client/shell/endpoints.rs; aggregate_navigation.rs` | Spec 013 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R036 | Endpoint heartbeat/health and reconnect supervision | `src/client/endpoint/health.rs; supervisor.rs` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R037 | Remote clipboard/image forwarding | `src/platform/remote_bridge.rs; clipboard modules` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R038 | Remote surface incremental patch/reuse | `src/protocol/surface_delta/**; surface_reuse.rs; client/shell/surface_patch.rs` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R039 | Git worktree discovery/create/open/remove | `src/workspace/git/**; src/worktree.rs; worktree.* API methods` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R040 | Repository trust gate for worktree operations | `CLI worktree --trust-repository; src/worktree.rs` | Spec 012 | WINDS_NATIVE_AUTHORITY + TEST_PORT candidate |
| R041 | Built-in integration install/uninstall/status | `src/integration/**; integration.* API methods` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| R042 | Plugin local link/unlink/enable/disable/list | `src/persist/plugin_registry.rs; plugin.* API methods` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| R043 | GitHub plugin install with pinned source metadata | `src/cli/plugin.rs; PluginSourceInfo` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| R044 | Plugin build and startup commands | `PluginManifestBuild; PluginManifestStartup; src/plugin_command.rs` | Spec 014 | ADAPTED_COPY + THREAT_MODEL + TEST_PORT candidate |
| R045 | Plugin actions with scoped contexts | `PluginManifestAction; PluginActionContext; plugin.action.*` | Spec 014 | ADAPTED_COPY + THREAT_MODEL + TEST_PORT candidate |
| R046 | Plugin event hooks | `PluginManifestEventHook; src/plugin_command.rs` | Spec 014 | ADAPTED_COPY + THREAT_MODEL + TEST_PORT candidate |
| R047 | Plugin-owned panes: overlay/popup/split/tab/zoomed | `PluginManifestPane; PluginPanePlacement; plugin.pane.*` | Spec 014 | ADAPTED_COPY + THREAT_MODEL + TEST_PORT candidate |
| R048 | Plugin link handlers | `PluginManifestLinkHandler; clicked_url/link_handler_id context` | Spec 014 | ADAPTED_COPY + THREAT_MODEL + TEST_PORT candidate |
| R049 | Plugin command logs/status/stdout/stderr | `PluginCommandLogInfo; plugin.log.list` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| R050 | Plugin platform gating | `PluginPlatform Linux/macOS/Windows` | Spec 014 | ADAPTED_COPY + TEST_PORT candidate |
| R051 | Updater with stable/preview channels | `src/update.rs; config UpdateConfig; CLI update/channel` | Spec 015 | ADAPTED_COPY + TEST_PORT candidate |
| R052 | Live handoff for update/remote attach | `src/handoff_runtime.rs; src/server/handoff.rs; tests/live_handoff.rs` | Spec 015 | ADAPTED_COPY + TEST_PORT candidate |
| R053 | Release notes and product announcements | `src/release_notes.rs; src/product_announcements.rs; UI modules` | Spec 015 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R054 | Installers for Unix/PowerShell/CMD and Windows packaging | `distribution/install.*; scripts/package_windows_conpty.*` | Spec 015 | ADAPTED_COPY + TEST_PORT candidate |
| R055 | Cross-platform Linux/macOS/Windows runtime paths | `src/platform/**; scripts/windows_*` | Spec 015 | TEST_PORT + WINDS_NATIVE_ADAPTER candidate |
| R056 | Render profiling and scale benchmarks | `src/render_prof.rs; src/server/render_scale_benchmark.rs` | Spec 015 | TEST_PORT candidate |
| R057 | Incremental render/surface delivery | `src/render_signal.rs; src/server/render_stream.rs; protocol surface delta/reuse` | Spec 015 | ADAPTED_COPY + TEST_PORT candidate |

| R058 | First-run onboarding surface | `src/ui/onboarding.rs`; `Config.onboarding` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R059 | Config validation/reload with applied/partial/failed diagnostics | `src/config/**`; `ConfigReloadReport`; `server.reload_config` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R060 | Terminal default shell, login/non-login mode and new-CWD policy | `TerminalConfig`; `src/client/terminal_setup.rs` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R061 | Headless terminal geometry while no client is attached | `ServerConfig.headless_cols/headless_rows`; `src/server/headless/**` | Spec 011 | ADAPTED_COPY + TEST_PORT candidate |
| R062 | Bounded per-pane scrollback memory configuration | `AdvancedConfig.scrollback_limit_bytes`; terminal history modules | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R063 | Workspace close confirmation and tab/workspace naming prompts | `UiConfig.confirm_close`; `prompt_new_tab_name`; `prompt_new_workspace_name` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R064 | Pane borders, outer borders, scrollbars, gaps and agent border labels | `UiConfig.pane_borders`; `pane_outer_borders`; `pane_scrollbars`; `pane_gaps`; `show_agent_labels_on_pane_borders` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R065 | Accent-color and status-indicator customization | `UiConfig.accent`; `StatusIndicatorStyle` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R066 | Visual toast delivery, delay and placement policies | `ToastConfig`; `HerdrToastConfig`; `ClipboardToastConfig` | Spec 012 | WINDS_NATIVE_UI + TEST_PORT candidate |
| R067 | Worktree root-directory configuration | `WorktreesConfig.directory` | Spec 012 | WINDS_NATIVE_ADAPTER + TEST_PORT candidate |
| R068 | Managed SSH config keepalive/private connection reuse | `RemoteConfig.manage_ssh_config`; `src/remote/**` | Spec 013 | ADAPTED_COPY + TEST_PORT candidate |
| R069 | Optional nested-Herdr execution | `ExperimentalConfig.allow_nested` | Spec 013 | THREAT_MODEL + WINDS_NATIVE_REIMPLEMENTATION candidate |
| R070 | Optional persisted pane screen history | `ExperimentalConfig.pane_history`; `src/persist/**` | Spec 011 | ADAPTED_COPY + PRIVACY_REVIEW + TEST_PORT candidate |
| R071 | Host redraw-on-focus and host-cursor policy | `UiConfig.redraw_on_focus_gained`; `HostCursorModeConfig` | Spec 012 | ADAPTED_COPY + TEST_PORT candidate |
| R072 | Update version/manifest checks | `UpdateConfig.version_check`; `UpdateConfig.manifest_check`; `src/update.rs` | Spec 015 | ADAPTED_COPY + TEST_PORT candidate |

### Plugin marketplace terminology

At this Herdr pin, source evidence proves GitHub-backed plugin installation plus local linking, registry management, actions, events, panes, link handlers and logs. A distinct curated marketplace/catalog service is not established by the inspected source. Winds may intentionally build a marketplace as a parity-plus surface, but it must not be cited as an already-proven Herdr capability without newer source evidence.

## Admission and copy rules

1. Founder permission permits direct Herdr source reuse, but every admitted copy must bind exact source path, exact Herdr commit/tree, reuse mode, retained copyright/NOTICE obligations, third-party/vendor origin, local modifications, tests, security/authority delta, and removal/update path.
2. Root Apache-2.0 status is not treated as a blanket license assertion for vendored or third-party material. `vendor/libghostty-vt.*`, portable-pty patch records, generated assets, embedded integration assets, and any other derived dependency are reviewed independently before copying.
3. `DIRECT_COPY` is preferred only when the Herdr boundary already matches Winds architecture and authority. Otherwise use `ADAPTED_COPY`, `TEST_PORT`, or `WINDS_NATIVE_REIMPLEMENTATION`.
4. Herdr behavior never overrides Winds invariants: exact Project/Session identity, `AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED`, least authority, no raw terminal text as trusted control, explicit human approval for consequential authority expansion, deterministic evidence, and no fabricated runtime/provider state.
5. Remote, plugin, updater, installer, and persistent-owner surfaces require their own threat models and exact-head qualification before implementation/landing.

## Closeout rule

`HERDR_FULL_PARITY=PROVEN` is forbidden until all **218 baseline rows** in this refreshed ledger have evidence-backed final dispositions, the current Herdr source has been re-audited for feature drift, all admitted direct/adapted copies have provenance/license ledgers, and Spec 015 parity closeout reconciles current Winds behavior against both this pin and any newer Founder-accepted Herdr pin.
