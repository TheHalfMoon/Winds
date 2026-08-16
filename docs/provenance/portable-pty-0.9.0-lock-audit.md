# portable-pty 0.9.0 — T050 Locked Dependency Audit

**Status:** T050 dependency-landing evidence. This document does not authorize T051+ behavior.

**Winds slice:** Spec 003 / T050 — Unix PTY terminal lifecycle

**Direct dependency:** `portable-pty = "=0.9.0"`

**Upstream source decision:** `wezterm/wezterm` commit `f8921727a11b9f8b073e8c24821d72fd41283500`, path `pty/`, MIT. The dependency candidate was accepted by T043 before landing.

## 1. Lock generation

The T050 branch removed the pre-existing lockfile in an isolated GitHub Actions job, ran `cargo generate-lockfile` under pinned Rust `1.97.1`, and committed Cargo's generated result without hand-editing package entries.

- Cargo-generated lock commit: `ffa48fe2fba6c90bd1bc3578d1c8812a85cd7840`
- `portable-pty` locked version: `0.9.0`
- `portable-pty` crates.io checksum: `b4a596a2b3d2752d94f51fac2d4a96737b8705dddd311a32b9af47211f08671e`
- Total packages in the resulting Winds lockfile: 69
- The temporary lock-generation workflow was removed before the implementation candidate was reviewed; it is not part of the final T050 diff.

The exact direct/build host graph observed on Ubuntu was:

```text
portable-pty v0.9.0
├── anyhow v1.0.104
├── bitflags v1.3.2
├── downcast-rs v1.2.1
├── filedescriptor v0.8.3
│   └── thiserror v1.0.69
├── libc v0.2.189
├── log v0.4.33
├── nix v0.28.0
│   ├── bitflags v2.13.1
│   ├── cfg-if v1.0.4
│   ├── libc v0.2.189
│   └── cfg_aliases v0.1.1 (build)
├── serial2 v0.2.38
│   ├── cfg-if v1.0.4
│   └── libc v0.2.189
└── shell-words v1.1.1
```

The lockfile also contains the target-specific packages selected by `portable-pty` for non-Linux targets, including the Windows support packages listed below. Their presence is intentionally audited now even though T050 product code is Unix-only; T051 is responsible for native-Windows runtime proof.

## 2. Exact new locked packages and declared licenses

A read-only GitHub Actions audit ran `cargo metadata --locked --format-version 1` under Rust `1.97.1` and required every newly introduced package/version to be present. Audit run: `31950989434`.

| Package | Version | Cargo metadata license | Repository |
|---|---:|---|---|
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | `dtolnay/anyhow` |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 | `bitflags/bitflags` |
| `cfg_aliases` | 0.1.1 | MIT | `katharostech/cfg_aliases` |
| `downcast-rs` | 1.2.1 | MIT/Apache-2.0 | `marcianx/downcast-rs` |
| `filedescriptor` | 0.8.3 | MIT | `wezterm/wezterm` |
| `lazy_static` | 1.5.0 | MIT OR Apache-2.0 | `rust-lang-nursery/lazy-static.rs` |
| `log` | 0.4.33 | MIT OR Apache-2.0 | `rust-lang/log` |
| `nix` | 0.28.0 | MIT | `nix-rust/nix` |
| `portable-pty` | 0.9.0 | MIT | `wezterm/wezterm` |
| `serial2` | 0.2.38 | BSD-2-Clause OR Apache-2.0 | `de-vri-es/serial2-rs` |
| `shared_library` | 0.1.9 | Apache-2.0/MIT | `tomaka/shared_library` |
| `shell-words` | 1.1.1 | MIT/Apache-2.0 | `tmiasko/shell-words` |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | `dtolnay/thiserror` |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | `dtolnay/thiserror` |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | `retep998/winapi-rs` |
| `winapi-i686-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | `retep998/winapi-rs` |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | `retep998/winapi-rs` |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | `microsoft/windows-rs` |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | `microsoft/windows-rs` |
| `winreg` | 0.10.1 | MIT | `gentoo90/winreg-rs` |

No newly introduced package declares GPL, LGPL, AGPL, SSPL, or another copyleft/source-available license in Cargo metadata. Package metadata is evidence about the declared package license; it does not replace preservation of applicable copyright/license notices in redistributed artifacts.

## 3. Redistribution / notice posture

T050 copies no donor source into Winds. `portable-pty` is consumed as an exact crates.io dependency. The MIT/BSD/Apache family licenses above are compatible with the Winds dependency posture, subject to their normal notice/attribution obligations.

Before a public binary/package distribution process is declared complete, that distribution process must preserve applicable third-party notices for the actual shipped dependency set. This audit does not claim that Cargo metadata alone satisfies redistribution notice obligations.

## 4. Runtime-source facts used by T050

The accepted `portable-pty 0.9.0` Unix implementation:

- calls `setsid()` in the spawned child before attaching the PTY;
- uses `TIOCSCTTY` to make the PTY the child's controlling terminal;
- exposes `MasterPty::process_group_leader()`, implemented on Unix with `tcgetpgrp`;
- retains a concrete `Child` handle with `process_id`, `try_wait`, `wait`, and `kill` behavior.

T050 uses those facts narrowly for live, in-process ownership checks. In particular, interrupt signaling derives the current foreground process group from the exact owned PTY and verifies through `getsid` that it still belongs to the session led by the exact retained child handle before sending `SIGINT`. A persisted PID is never used as process authority.

## 5. Scope and known follow-ons

This audit does not prove or authorize:

- native Windows/ConPTY runtime support — T051;
- WSL launch or path mapping — T052;
- restart attachment or durable process ownership — T053;
- command telemetry — T054;
- a terminal renderer, daemon, public IPC/protocol, plugin system, MCP/ACP, or Agent Fleet.

The dependency remains accepted only while exact-head compilation/tests and the Spec 003 correctness/safety gates pass. If a later target exposes a dependency incompatibility or an unacceptable license/provenance condition, the T043 decision must be reopened rather than silently bypassed.
