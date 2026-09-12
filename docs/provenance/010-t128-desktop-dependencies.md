# Spec 010 T128 Desktop Dependency Qualification

Status: CANDIDATE EVIDENCE UNTIL T128 GUARDED LANDING

Canonical predecessor at qualification start:

```text
T127_MERGE=09d50fe185c061f23d21d4865f82b07a55736765
T128_TYPESCRIPT_AMENDMENT_MERGE=37129870df6257b0a50fdeac5a578aa1e4ca86cb
```

## Toolchain

```text
node=22.22.3
npm=10.9.8
rust=1.97.1
javascript_package_manager=npm
javascript_lockfile=desktop/package-lock.json
rust_lockfile=desktop/src-tauri/Cargo.lock
```

The first directly claimed desktop build host is macOS arm64. Windows and Linux desktop product qualification remain owned by T142.

## Rust desktop host direct dependencies

| Package | Version | License | MSRV | Registry checksum |
| --- | --- | --- | --- | --- |
| `tauri` | `2.11.5` | Apache-2.0 OR MIT | `1.77.2` | `667b20e2726d572dea2de7370da16e188eb06008faf9a92fab7cdc46791190b5` |
| `tauri-build` | `2.6.3` | Apache-2.0 OR MIT | `1.77.2` | `bc9ce40b16101cb6ea63d3e221567affd1c3a9205f95d7bc574941a10636b632` |

Both resolve from the crates.io registry source recorded in `Cargo.lock`. The locked desktop Rust graph contains 432 package records on the current host resolution.

## JavaScript direct dependencies

| Package | Version | License | Purpose |
| --- | --- | --- | --- |
| `@tauri-apps/api` | `2.11.1` | Apache-2.0 OR MIT | typed Tauri client API; not invoked in T128 |
| `react` | `19.3.0` | MIT | renderer view model |
| `react-dom` | `19.3.0` | MIT | renderer DOM integration |
| `@xterm/xterm` | `6.0.0` | MIT | terminal rendering reserved for T135; unused in T128 |
| `@xterm/addon-fit` | `0.11.0` | MIT | terminal fit helper reserved for T135; unused in T128 |
| `lucide-react` | `1.45.0` | ISC | generic UI icon grammar reserved for later visual tasks; unused in T128 |

## JavaScript direct development dependencies

| Package | Version | License | Purpose |
| --- | --- | --- | --- |
| `@tauri-apps/cli` | `2.11.4` | Apache-2.0 OR MIT | deterministic Tauri build CLI |
| `vite` | `8.3.0` | MIT | renderer build |
| `@vitejs/plugin-react` | `6.1.1` | MIT | React transform |
| `typescript` | `7.0.2` | Apache-2.0 | exact local TypeScript compiler |
| `@types/react` | `19.3.0` | MIT | React TSX declarations |
| `@types/react-dom` | `19.3.0` | MIT | React DOM declarations |
| `@types/node` | `22.20.2` | MIT | Node-facing Vite configuration declarations |

`package-lock.json` records exact resolved registry URLs and SHA-512 integrity values for all npm artifacts. The current lock contains 86 non-root package records.

## Direct npm integrity and engine evidence

All direct npm artifacts resolve from `https://registry.npmjs.org/`. The exact direct integrity values are:

```text
@tauri-apps/api@2.11.1 sha512-M2FPuYND2m+wh5hfW9ZpSdxMPdEJovPBWwoHJmwUpysTYNHaOkVFN419m/K0LIgjb/7KU2vBgsUepJWugQCvAA==
@tauri-apps/cli@2.11.4 sha512-R8xGtMpwyetawSqm9kYOuMmEqkhUbvcUy8n0aNXIxollKBLESUu5f4Fx+64hgASYm1H+jSWq6jCW6zqTnH6hqQ==
react@19.3.0 sha512-E8LUcbtBWt20bbl2YoHfx4ZDBdxVTfOKtCZn9cDSJ4l6/nuoApcpIBcj47t2wZoVX8g2ZHuMHbiShgCR1T5Sog==
react-dom@19.3.0 sha512-JDk8dgif51OjFoDE70+OT9ICyYr+69HlmihNwp1+Nsfbna3t5sIiCa9ZJktDmQ4/1b/rn26hIAR2uYXDMr5r0Q==
vite@8.3.0 sha512-lhZBVvEHefgE+HQZC9O7EBJgCU/nVzFNl7vkS4RE0APtWLP02/8QVIkQtzBxPquh7lq5/78NHipTj7ODQ6XuyQ==
@vitejs/plugin-react@6.1.1 sha512-yxLaQV9gkhS8ezJqCM6+ndU7mDY6gqAg75NQ+0IjwEI8IYOmQCgkRwHKVSfWXW076DsqMo0Dk+0FK1U+M5RgFw==
@xterm/xterm@6.0.0 sha512-TQwDdQGtwwDt+2cgKDLn0IRaSxYu1tSUjgKarSDkUM0ZNiSRXFpjxEsvc/Zgc5kq5omJ+V0a8/kIM2WD3sMOYg==
@xterm/addon-fit@0.11.0 sha512-jYcgT6xtVYhnhgxh3QgYDnnNMYTcf8ElbxxFzX0IZo+vabQqSPAjC3c1wJrKB5E19VwQei89QCiZZP86DCPF7g==
lucide-react@1.45.0 sha512-yH1ubCAduho9UR7oJhRXIQXogksRILBiTuZC4/bQIGeB9JOkxMlSuEHyyZpo1Z3S0yWJO2KTSUZbjiNvVxeOUw==
typescript@7.0.2 sha512-8FYau96o3NKOhbjKi/qNvG/W5jhzxkbdm5sj9AbZ/5T5sWqn3hJgLfGx27sRKZWTvyzCP8dLRBTf5tBTSRVUNA==
@types/react@19.3.0 sha512-N0rFCuH9YoxG9/m61l9MfpJKfmLOVU0em7ipIz6TRgSSkvReLB9vL85GB+yr8Bs5leqpvg96JSwF4ZS1s4viQg==
@types/react-dom@19.3.0 sha512-ZI7bU42mZXXKHn/qNLEw2IrbiINU7X5+vfgdixBHkCNpYWXjKgfQ/P+uyGb5CjOLB9UcnTeg3rylQtV2hym44Q==
@types/node@22.20.2 sha512-xlvWf4Vs9n1PEVYwP1n4vvG07M6y8WgvJ2t0vbrWTmijsIHp1cS+uJ2kMIRdY3nHZK0nCYKrPeD171+SzF4/zw==
```

Engine compatibility verified from registry metadata:

```text
vite@8.3.0 node="^20.19.0 || >=22.12.0"
@vitejs/plugin-react@6.1.1 node="^20.19.0 || >=22.12.0"
typescript@7.0.2 node=">=16.20.0"
@tauri-apps/cli@2.11.4 node=">=10"
selected_node=22.22.3
```

`22.22.3` satisfies every direct Node engine constraint above. Tauri and `tauri-build` both declare Rust `1.77.2` as MSRV; the repository-selected Rust `1.97.1` satisfies that requirement.

## Installation discipline

The canonical install command for frontend dependencies is:

```text
npm ci --ignore-scripts
```

No second JavaScript package manager is admitted. No postinstall script is required for the selected direct dependency surface. `npm audit` reported zero known vulnerabilities at T128 qualification time; that observation is time-bound and does not replace lockfile review.

The T128 renderer imports React only. It does not import the Tauri client API, xterm.js, addon-fit, or Lucide yet. Those selected direct dependencies are lock-qualified now so later owning tasks do not silently choose a different graph.

## T128 privilege boundary

The inert host has no `#[tauri::command]`, no invoke handler, no plugin registration, no filesystem or shell plugin, no product Store/Git/PTY/runtime call, and no capability file.

`tauri.conf.json` sets an empty capability list and bundles only local frontend assets. No `devUrl` is configured. The CSP denies network connections, frames, child contexts, objects, and form submission; scripts/styles/fonts default to the bundled application origin.

The temporary `desktop/src-tauri/icons/icon.png` exists only because `tauri::generate_context!()` requires an application icon at compile time. It is a Winds-owned T128 placeholder, SHA-256:

```text
a120239075497855c8475d941ae7d0934dfd670ac86d582b114e11cd2d57d212
```

It carries no final brand claim and is expected to be replaced only by an owning visual task.

## Frozen frontend gates

```text
npm run format:check
npm run typecheck
npm run lint
npm test
npm run frontend:build
npm run desktop:build
```

`desktop-quality` reproduces these gates on the first directly claimed desktop CI host (`macos-latest`) using exact Node, npm, and Rust toolchains.

## Local qualification observations

The local macOS arm64 environment produced the following observations before candidate commit:

```text
npm_ci=PASS
npm_audit_known_vulnerabilities=0
frontend_format=PASS
frontend_typecheck=PASS
frontend_lint=PASS
frontend_tests=17_passed_0_failed
vite_production_build=PASS
desktop_cargo_check=PASS
desktop_rust_format=PASS
desktop_rust_clippy=PASS
root_cli_build=PASS
```

A local `npm run desktop:build` invocation produced `desktop/src-tauri/target/release/winds-desktop-host`, a Mach-O arm64 release binary. The remote command wrapper timed out before returning the process exit status, so that local invocation is not counted as acceptance PASS. Exact-head `desktop-quality` CI is the authoritative T128 desktop-build gate.

## Explicit nonclaims

T128 does not claim product Projects/Sessions, persistence, runtime launch, Codex or Claude execution, terminal rendering, Files/Changes/Evidence docks, command palette behavior, visual-system completion, final iconography, Windows/Linux desktop qualification, packaging/signing/notarization, or Founder visual acceptance.

T128 adds build infrastructure and an inert local desktop window only. T129 remains blocked until this exact dependency graph and shell land canonically and all actually-triggered post-merge gates pass.
