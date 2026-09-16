# Herdr Current Source Audit — 2026-09-16

**Status:** Research/provenance evidence only. No code admission or implementation authority.

## Exact source

- Repository: `https://github.com/herdrdev/herdr`
- Commit: `18061191fdc019498610aee81f0df93f6c2ebd31`
- Tree: `ab44d1c6939f1e9b5657512e831c6e50969941a2`
- Default branch observed: `master`
- Repository license observed: Apache-2.0
- Previous Winds Herdr research pin: `ef2674bab8a3b38984578473c1a80589ebcbb333`
- Commits from previous pin to current pin: 181

## Founder reuse statement

The Founder stated on 2026-09-16 that they have permission to copy and use all Herdr source code. Winds records that statement as `FOUNDER_PERMISSION_ASSERTION`; it does not waive Apache-2.0 notice obligations or third-party/vendored-source obligations.

## Inspected feature-bearing paths

Representative paths inspected at the exact pin include:

- persistent/session/runtime: `src/session.rs`, `src/persist.rs`, `src/server/**`, `src/client/terminal_sessions.rs`, `tests/detach_reattach.rs`, `tests/multi_client.rs`, `tests/server_headless.rs`;
- live handoff: `src/handoff_runtime.rs`, `src/server/handoff.rs`, `tests/live_handoff.rs`;
- local control/API: `src/ipc.rs`, `src/api/**`, `src/app/api/**`, `src/cli/**`;
- workspace/panes/layout: `src/workspace.rs`, `src/workspace/**`, `src/pane.rs`, `src/pane/**`, `src/layout.rs`;
- agents: `src/detect/**`, `src/pane/agent_detection.rs`, `src/app/agents.rs`, `src/agent_resume.rs`, `src/agent_view_eval.rs`;
- integrations: `src/integration/**`, `src/api/schema/integrations.rs`;
- plugins: `src/plugin_command.rs`, `src/plugin_paths.rs`, `src/app/api/plugins/**`, `src/persist/plugin_registry.rs`;
- remote: `src/remote.rs`, `src/remote/**`, `src/platform/remote_bridge.rs`, `tests/remote_attach.rs`, `tests/machine_api.rs`, `tests/machine_setup.rs`;
- worktrees: `src/worktree.rs`, `src/app/worktrees.rs`, `src/api/schema/worktrees.rs`;
- endpoint/multi-client transport: `src/client/endpoint/**`, `src/protocol/endpoint.rs`, `src/server/client_endpoint_control.rs`;
- rendering/performance: `src/render_prof.rs`, `src/render_signal.rs`, current surface-reuse/delta transport paths;
- update/install: `src/update.rs`, `distribution/**`.

## Agent detection inventory

The exact `Agent` enum contains 24 families:

1. Pi
2. Claude
3. Codex
4. Gemini
5. Cursor
6. Devin
7. Antigravity
8. Cline
9. Omp
10. Mastracode
11. OpenCode
12. GithubCopilot
13. Kimi
14. Kiro
15. Droid
16. Amp
17. Grok
18. Hermes
19. Kilo
20. Qodercli
21. Qwen
22. Letta
23. Maki
24. Muse

## Installable integration inventory

`IntegrationTarget::ALL` contains 17 targets:

1. Pi
2. Omp
3. Claude
4. Codex
5. Copilot
6. Devin
7. Droid
8. Kimi
9. Opencode
10. Kilo
11. Hermes
12. Qodercli
13. Qwen
14. Cursor
15. Mastracode
16. AntigravityCli
17. Grok

Detection coverage and installable integration coverage are intentionally not treated as identical.

## Structured surface observations

The inspected schemas/handlers include operations/events for workspace, pane, agent, integration, plugin, worktree, layout, session, server, and event-wait/subscription semantics. Representative method names include:

- `pane.read`, `pane.focus`, `pane.send_input`, `pane.send_text`, `pane.process_info`, `pane.wait_for_output`;
- `agent.explain`, `agent.prompt`, `agent.start`;
- `layout.set_split_ratio` plus export/apply paths;
- `events.subscribe`, `events.wait`;
- `integration.list` and install/uninstall request types;
- `plugin.action.list`, `plugin.action.invoke`, `plugin.pane.open`;
- worktree list/create/open/remove paths;
- server manifest/config/stop operations.

## Material delta since the prior Winds pin

The 181-commit delta includes material changes in at least:

- remote surface delta/incremental transport;
- saved SSH machine routing and multi-machine navigation;
- Windows remote host packaging/setup;
- live session/handoff robustness;
- Codex resume-session persistence;
- plugin continuity and focus events;
- agent state/session handling;
- worktree races/removal/navigation;
- Windows/SSH keyboard, mouse, paste, cwd, and process handling;
- client endpoint compatibility/reconnect behavior.

Therefore the old pin is insufficient as the sole implementation-time donor reference.

## License and vendor boundary

The root repository is Apache-2.0, but Winds must not infer that every vendored/generated/source-derived artifact is automatically reusable under only that root label.

Observed vendor metadata includes `vendor/libghostty-vt.vendor.json` pointing to upstream source commit `44f2a44df7e8c4a0c6df3f7d872ef3d7ead88e51`, plus separate portable-pty vendor/patch records. Every direct-copy slice touching vendor-derived code must audit upstream license, patch provenance, notices, and compatibility independently.

## Admission disposition

```text
HERDR_CURRENT_PIN=AUDITED_FOR_RESEARCH
HERDR_ROOT_LICENSE=APACHE-2.0
FOUNDER_PERMISSION_ASSERTION=RECORDED
FULL_PARITY_PRODUCT_TARGET=YES
DIRECT_COPY_AUTHORITY=NO_UNTIL_FORMAL_TASK_ADMISSION
THIRD_PARTY_VENDOR_BLANKET_ADMISSION=NO
```
