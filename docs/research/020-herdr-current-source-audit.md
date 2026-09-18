# Herdr Current Source Audit — 2026-09-18

**Status:** Research/provenance evidence only. No code admission or implementation authority.

## Exact source

- Repository: `https://github.com/herdrdev/herdr`
- Default branch observed: `master`
- Commit: `da6bcd5969779bfe0396bcf89a8025d4375d611e`
- Tree: `aece03633c003ba0fce01bc2564ad14e0fe9cac9`
- Package version observed: `0.9.1`
- Root repository license observed: Apache-2.0
- Immediately previous Winds audit pin: `18061191fdc019498610aee81f0df93f6c2ebd31`
- Previous audit tree: `ab44d1c6939f1e9b5657512e831c6e50969941a2`
- Commits from previous audit pin to current pin: **21**
- Earlier September research pin: `ef2674bab8a3b38984578473c1a80589ebcbb333`
- Current Winds canonical base for this research refresh: `7ed50fb6b173c4853394a93b51691e54fa463349`

## Founder reuse statement

The Founder previously directed Winds to reach full Herdr feature parity and stated that they have permission to copy/use Herdr source code. Winds records that statement as `FOUNDER_PERMISSION_ASSERTION`. It does not waive Apache-2.0 notice obligations, copyright obligations, or independent provenance/license review for vendored, generated, patched, or third-party material.

## Fresh inventory at the exact pin

### Agent detection families

`src/detect/mod.rs` declares `Agent::ALL: [Self; 24]`. The families are:

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

### Integration targets

The stable serialized client/API integration enum remains frozen at **17** targets in `src/api/schema/integrations.rs`:

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

In addition, the current source exposes **Letta** as an experimental **CLI-only installable integration** through `EXPERIMENTAL_INTEGRATION_TARGET_LABELS = &["letta"]`. Herdr intentionally keeps Letta outside the frozen generation-1 `IntegrationTarget` enum until the planned registry replaces the enum-keyed surface.

Therefore the research inventory must distinguish:

```text
FROZEN_SERIALIZED_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INSTALLABLE_TARGETS=1
TOTAL_OBSERVED_INSTALLABLE_TARGET_LABELS=18
EXPERIMENTAL_TARGET=letta
```

The 2026-09-16 draft PR #212 counted the 17 frozen enum targets but did not separately enumerate the already-present experimental Letta install path. That draft remains historical evidence and is superseded by this audit.

### Public structured API

`src/api/schema.rs` still exposes exactly **104 serialized public methods**. Internal `serde(skip)` graphics stream-control variants are implementation detail and are not counted as public serialized methods.

The public surface spans server/config/manifest control, workspace, worktree, tab, agent, pane, layout, events/waiting, integration, and plugin operations. The exact method ledger is preserved in `docs/research/021-herdr-exhaustive-capability-ledger.md`.

## 21-commit delta from the previous audit

The current pin is 21 commits ahead of `18061191...` with no reverse divergence. The exact commit series includes material changes in these product/qualification domains:

- local usability when remote machines stall;
- cancellation of remote handoffs when local agent navigation wins;
- exact session-name confirmation before deletion;
- worktree forced-removal behavior for submodules;
- protected-preview -> stable release promotion and release metadata for v0.9.1;
- OpenCode `tui.json` integration registration;
- saved-machine connection metadata caching;
- native-Windows VT/mouse/input recovery and terminal restoration;
- Windows default pane shell preference for `pwsh`;
- local Windows input gauntlet coverage and symlink-test qualification.

This delta does not change the 24-agent or 104-public-method counts, and the frozen integration enum remains 17. It is nevertheless material to later remote, Windows, integration, worktree, release/update, and platform qualification specifications.

## Feature-bearing source families inspected

The current and immediately preceding exact pins establish the following source families as material successor references:

- persistent/session/runtime: `src/session.rs`, persistence/server/client ownership paths, detach/reattach and multi-client tests;
- local control/API: `src/api/**`, `src/server/**`, protocol/socket paths, CLI surfaces;
- workspace/pane/layout: `src/workspace/**`, `src/pane.rs`, layout and client-shell paths;
- agents: `src/detect/**`, agent state/resume/view and integration authority paths;
- integrations: `src/integration/**`, `src/api/schema/integrations.rs`;
- plugins: plugin command/path/registry/API surfaces;
- remote/multi-machine: `src/remote/**`, `src/client/endpoint/**`, saved-machine and remote bridge paths;
- worktrees: `src/worktree.rs`, workspace Git paths, worktree API;
- input/platform: `src/input/**`, `src/platform/windows/**`, terminal modes and Windows input gauntlet scripts;
- update/release/install: `src/update.rs`, `distribution/**`, release/preview workflows and scripts;
- rendering/performance: render profiling/signaling and incremental surface delivery paths.

## License and vendor boundary

The root repository is Apache-2.0, but Winds must not infer that every vendored/generated/source-derived artifact is automatically reusable under only that root label.

Direct or adapted reuse must independently bind exact Herdr source paths and audit any underlying vendor/third-party origin, including portable-pty patches, libghostty-derived material, embedded integration assets, generated artifacts, and platform-specific bundled runtime components.

## Admission disposition

```text
HERDR_CURRENT_PIN=AUDITED_FOR_RESEARCH
HERDR_ROOT_LICENSE=APACHE-2.0
FOUNDER_PERMISSION_ASSERTION=RECORDED
FULL_PARITY_PRODUCT_TARGET=YES
AGENT_FAMILIES=24
PUBLIC_SERIALIZED_API_METHODS=104
FROZEN_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INTEGRATIONS=1
DIRECT_COPY_AUTHORITY=NO_UNTIL_FORMAL_TASK_ADMISSION
THIRD_PARTY_VENDOR_BLANKET_ADMISSION=NO
SPEC_011_ENTRY_AUTHORIZED=NO
PERSISTENT_OWNER_IMPLEMENTATION_AUTHORIZED=NO
PRIVATE_IPC_IMPLEMENTATION_AUTHORIZED=NO
REMOTE_EXECUTION_IMPLEMENTATION_AUTHORIZED=NO
PLUGIN_RUNTIME_IMPLEMENTATION_AUTHORIZED=NO
```

This audit is sufficiently fresh for a successor **entry-gate candidate** at the time recorded. The entry gate must still re-check the upstream head immediately before its own final qualification; material upstream movement requires reconciliation rather than silently inheriting this snapshot.
