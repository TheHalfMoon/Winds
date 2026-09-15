# T142 Native Platform Qualification

## Scope

T142 directly qualifies the exact-candidate platform claims below for:
- macOS 15;
- Ubuntu 24.04 Linux;
- native Windows 2025;
- Ubuntu WSL2 as a Windows-hosted but distinct execution and path domain.

This task adds qualification evidence only. It does not add product authority, runtime launch capability, persistence semantics, or dependencies.

## Exact evidence contract

Each native host must:
- check out and verify the exact candidate SHA;
- run the deterministic renderer/accessibility suite;
- exercise the native Workbench terminal input, focus, resize, and ownership path;
- build the locked release-like Tauri desktop host;
- record the exact candidate commit and tree;
- record the release binary SHA-256;
- record runner OS/architecture and the platform WebView version;
- retain the JSON record as an exact-candidate GitHub artifact.
## Platform-specific claims

### Linux

Direct evidence uses Ubuntu 24.04, native Unix PTY integration, T096 Workbench terminal input/focus/resize, WebKitGTK version discovery, and the locked Tauri build.

### macOS

Direct evidence uses macOS 15, native Unix PTY integration, T096 Workbench terminal input/focus/resize, WKWebView framework version discovery, and the locked Tauri build.

### Native Windows

Direct evidence uses Windows 2025, native ConPTY integration, T096 Workbench terminal input/focus/resize, WebView2 runtime version discovery, and the locked Tauri build.

### WSL2

WSL2 is not treated as native Windows or Linux substitution. The existing `windows-terminal` workflow provisions a real Ubuntu WSL2 distribution and proves exact candidate, mapped workspace identity, production prepare/launch, fallback non-equivalence, and T096 host/guest domain and path truth.

T142 changes are included in that workflow's path triggers so the real WSL2 proof runs on the same candidate.
## Appearance and input boundary

The renderer/accessibility suite is exercised on every native host. It directly preserves the keyboard/pointer and IME composition contracts introduced by T140 together with dark/light/system/high-contrast, reduced-motion, narrow-layout, and 200% scaling semantics.

These are deterministic application contracts executed on each host; they are not relabelled as OS-level Tauri-window focus, GUI input injection, or live theme-transition evidence.

## Explicit nonclaims

- `OS_LEVEL_TAURI_WINDOW_FOCUS_AUTOMATION=NOT_CLAIMED`
- `OS_LEVEL_IME_INJECTION_AUTOMATION=NOT_CLAIMED`
- `LIVE_NATIVE_SYSTEM_THEME_TRANSITION_AUTOMATION=NOT_CLAIMED`
- `PLATFORM_BUILD_SUBSTITUTES_FOR_T144_HUMAN_VISUAL_ACCEPTANCE=NO`

The absence of those claims is product-safe: Winds does not present a platform-specific live-IME or live-theme-transition guarantee beyond the directly exercised contracts.

## Closure rule

T142 may close only after the exact-head workflow succeeds on all three native host families, the real Windows+WSL2 workflow succeeds on the same exact candidate, retained artifacts are identity-bound, independent review has zero material findings, and guarded landing plus post-merge push verification succeed.
