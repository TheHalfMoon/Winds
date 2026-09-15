# T140 — Desktop Accessibility Acceptance

Status: `CANDIDATE`

Canonical base at candidate creation:

```text
BASE=26f557bf07deaf667730e711ba988b8438179567
T139=CLOSED_CANONICAL
T140=AUTHORIZED_TO_START
```

## Scope

T140 changes only desktop presentation, accessibility semantics, deterministic appearance fixtures, tests, and its exact-candidate qualification workflow. It adds no Rust authority, runtime execution path, filesystem/Git/shell dispatcher, provider, credential, or dependency.

## Acceptance coverage

- WCAG 2.2 AA normal-text contrast is calculated from exact dark, light, and high-contrast tokens against all primary surfaces.
- System appearance is represented by leaving `data-theme` unset and honoring `prefers-color-scheme`.
- Compact density, reduced motion, 125%/200% scaling, and narrow layout have deterministic fixtures.
- Right-dock tabs use a roving-tab contract with ArrowLeft/ArrowRight/Home/End navigation and tab/panel relationships.
- Session Work Stream/Terminal selection exposes semantic selected state and paired keyboard/pointer routes.
- Project, Session, runtime, attention, and focused-slot truth receive screen-reader meaningful labels without changing canonical identity.
- Command palette search exposes combobox/listbox/active-option semantics while retaining explicit Session disambiguation.
- Composer shortcut behavior is exercised through a pure behavior helper: IME composition cannot submit; plain Enter remains multiline; Cmd/Ctrl+Enter is the only fixture shortcut.
- The T140 workflow runs the same deterministic accessibility suite and locked Tauri build on Ubuntu, macOS, and native Windows exact candidates.

## Boundaries

T140 does not claim direct VoiceOver, Narrator, or Orca automation. It establishes semantic accessibility contracts and cross-platform deterministic behavior. Native host/input/focus/high-DPI claims remain subject to T142 direct platform qualification.

T140 does not authorize direct Codex or Claude launch; T139's `UNAUTHORIZED` decisions remain unchanged.
