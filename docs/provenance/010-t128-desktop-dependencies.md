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

The first directly claimed desktop CI host is GitHub-hosted `macos-15`. T128 does not preclaim an architecture: exact-head `desktop-quality` records `sw_vers` and `uname -m`, and final T128 evidence binds the host claim to the architecture actually observed by that run. Windows and Linux desktop product qualification remain owned by T142.

## Rust desktop host direct dependencies

| Package | Version | License | MSRV | Registry checksum |
| --- | --- | --- | --- | --- |
| `tauri` | `2.11.5` | Apache-2.0 OR MIT | `1.77.2` | `667b20e2726d572dea2de7370da16e188eb06008faf9a92fab7cdc46791190b5` |
| `tauri-build` | `2.6.3` | Apache-2.0 OR MIT | `1.77.2` | `bc9ce40b16101cb6ea63d3e221567affd1c3a9205f95d7bc574941a10636b632` |

Both resolve from the crates.io registry source recorded in `Cargo.lock`. The locked desktop Rust graph contains 432 package records on the current host resolution.

## JavaScript direct dependencies

| Package | Version | License | Purpose |
| --- | --- | --- | --- |
| `react` | `19.3.0` | MIT | inert renderer view model |
| `react-dom` | `19.3.0` | MIT | inert renderer DOM integration |

The following Plan candidates were reviewed but are deliberately **not admitted** by T128 because this slice does not use them:

| Deferred candidate | Plan version | License | Owning future task |
| --- | --- | --- | --- |
| `@tauri-apps/api` | `2.11.1` | Apache-2.0 OR MIT | first task that introduces a typed renderer bridge |
| `lucide-react` | `1.45.0` | ISC | T129 or later visual task if still necessary |
| `@xterm/xterm` | `6.0.0` | MIT | T135 terminal rendering |
| `@xterm/addon-fit` | `0.11.0` | MIT | T135 terminal fit behavior |

They are absent from `desktop/package.json` and `desktop/package-lock.json`. Their owning task must revalidate necessity, exact version, provenance, and security before adoption.

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

`package-lock.json` records exact resolved registry URLs and SHA-512 integrity values for all npm artifacts. The current lock contains 82 non-root package records.

## Direct npm integrity and engine evidence

All direct npm artifacts resolve from `https://registry.npmjs.org/`. The exact direct integrity values are:

```text
@tauri-apps/cli@2.11.4 sha512-R8xGtMpwyetawSqm9kYOuMmEqkhUbvcUy8n0aNXIxollKBLESUu5f4Fx+64hgASYm1H+jSWq6jCW6zqTnH6hqQ==
react@19.3.0 sha512-E8LUcbtBWt20bbl2YoHfx4ZDBdxVTfOKtCZn9cDSJ4l6/nuoApcpIBcj47t2wZoVX8g2ZHuMHbiShgCR1T5Sog==
react-dom@19.3.0 sha512-JDk8dgif51OjFoDE70+OT9ICyYr+69HlmihNwp1+Nsfbna3t5sIiCa9ZJktDmQ4/1b/rn26hIAR2uYXDMr5r0Q==
vite@8.3.0 sha512-lhZBVvEHefgE+HQZC9O7EBJgCU/nVzFNl7vkS4RE0APtWLP02/8QVIkQtzBxPquh7lq5/78NHipTj7ODQ6XuyQ==
@vitejs/plugin-react@6.1.1 sha512-yxLaQV9gkhS8ezJqCM6+ndU7mDY6gqAg75NQ+0IjwEI8IYOmQCgkRwHKVSfWXW076DsqMo0Dk+0FK1U+M5RgFw==
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

The T128 renderer imports React and React DOM only. The Tauri client API, xterm.js, addon-fit, and Lucide are not admitted into the T128 manifest or lockfile. Later owning tasks must qualify them from then-current registry and repository truth rather than inheriting a dormant dependency.

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

`desktop-quality` reproduces these gates on the first directly claimed desktop CI host (`macos-15`) using exact Node, npm, and Rust toolchains. It records `sw_vers` plus `uname -m` before toolchain installation. `desktop:build` invokes `tauri build --no-bundle --ci -- --locked`, so the release host build is required to use the committed Cargo lockfile.

## Local qualification observations

The local macOS arm64 environment produced the following observations before candidate commit:

```text
npm_ci=PASS
npm_audit_known_vulnerabilities=0
frontend_format=PASS
frontend_typecheck=PASS
frontend_lint=PASS
frontend_tests=20_passed_0_failed
vite_production_build=PASS
desktop_cargo_check=PASS
desktop_rust_format=PASS
desktop_rust_clippy=PASS
root_cli_build=PASS
```

The repaired local `npm run desktop:build` completed with exit code `0` and produced `desktop/src-tauri/target/release/winds-desktop-host`. Its command is `tauri build --no-bundle --ci -- --locked`, so the local release build was lock-constrained. Exact-head `desktop-quality` CI remains the authoritative T128 desktop-build gate.


## Initial candidate review history

The first T128 PR head was `96eaf701678f59b4ad26769c9f742c274d68a4d1` with tree `71f433a53b9fbbdec132d3558d809478d873d962`. Fresh independent review found three material issues: the Tauri release build did not pass Cargo `--locked`, the evidence preclaimed macOS arm64 while CI used `macos-latest` without architecture attestation, and four unused future runtime dependencies were admitted prematurely.

Those findings are preserved as material historical evidence. The first head's `desktop-quality` run `34722630080` and repository `quality` run `34722630082` both later completed `SUCCESS` on attempt 1, but they remain stale for acceptance because the reviewed candidate was materially defective. The repair is forward-only: the release build now passes `--locked`, CI uses `macos-15` and records its observed architecture, and all four unused future dependencies are removed from the T128 manifest and lockfile. All checks and reviews on the first head are historical only.

## Explicit nonclaims

T128 does not claim product Projects/Sessions, persistence, runtime launch, Codex or Claude execution, terminal rendering, Files/Changes/Evidence docks, command palette behavior, visual-system completion, final iconography, Windows/Linux desktop qualification, packaging/signing/notarization, or Founder visual acceptance.

T128 adds build infrastructure and an inert local desktop window only. T129 remains blocked until this exact dependency graph and shell land canonically and all actually-triggered post-merge gates pass.
