# T087 Dependency Qualification Evidence

Status: `PROVENANCE_INPUT_ONLY`

This note records deterministic dependency provenance produced by the canonically authorized temporary T087 lock-generation workflow. It is input evidence only. Final T087 qualification must run again after the temporary workflow is removed and the final candidate HEAD/TREE is fixed.

## Generator provenance

```text
BASE_MAIN=59bd1c32cc549977c27d13ca3b3a32d4e94e604f
GENERATOR_HEAD=e23ff7c9512508417320e18678e4def51e555f15
GENERATOR_RUN=34054835399
GENERATOR_JOB=101544764766
GENERATOR_ARTIFACT_ID=9995675894
GENERATOR_ARTIFACT_DIGEST=sha256:ffa4010d3366e54c60cbe5c3acfe27535b02aea7ec70248b085ced8b8388b2da
GENERATED_CARGO_LOCK_SHA256=493353efb097eea27494921bbbae19538783d36f5580c7a581c05ab15370ede9
GENERATED_CARGO_LOCK_GIT_BLOB=8cb37540414ba86d347c0c900bcb5b794e165281
RUST_TOOLCHAIN=1.97.1
```

The generator run and job completed successfully on the exact generator HEAD. The artifact contained the Cargo-generated `Cargo.lock`, full `cargo metadata`, a package/license/MSRV summary, and `cargo tree -e features` output. The committed `Cargo.lock` Git blob is byte-identical to the generated artifact blob.

## Direct dependency decision

```toml
crossterm = "=0.29.0"
ratatui = { version = "=0.30.2", default-features = false, features = ["crossterm_0_29"] }
```

- T087 adds no other direct dependency.
- Ratatui defaults are disabled.
- Only the Ratatui Crossterm 0.29 backend feature is requested directly.
- Ratatui 0.30.2 resolves through `ratatui-crossterm 0.1.2`; that adapter enables Crossterm 0.29 upstream defaults.
- The selected Crossterm defaults observed in the generated feature tree are `bracketed-paste`, `derive-more`, `events`, and `windows`.
- Neither `event-stream` nor `osc52` is selected.

## Prohibited-capability inspection

Exact negative search over the generated `cargo tree -e features` output produced the following result:

| Surface | Present |
| --- | --- |
| `event-stream` | NO |
| `osc52` | NO |
| `tokio` | NO |
| `async-std` | NO |
| `smol` | NO |
| `reqwest` | NO |
| `hyper` | NO |
| `curl` | NO |
| `arboard` | NO |
| `copypasta` | NO |
| `tui-term` | NO |
| `alacritty` | NO |
| `webview` | NO |
| `wry` | NO |
| `tao` | NO |

No Tokio/async executor, network/provider/browser client, clipboard automation crate, daemon/service framework, `tui-term`, or Alacritty terminal runtime was present in the generated feature tree.

## Version, source, checksum, license, and MSRV findings

The generated lock resolves the authorized direct versions exactly:

```text
ratatui=0.30.2
ratatui_checksum=3274ba0a2c5e1bcad2a2005d20f4dc59dad26b2eb0940fb094500dba4099d57d
crossterm=0.29.0
crossterm_checksum=d8b9f2e4c67f833b660cdb0a3523065869fb35570177239812ed4c905aeff87b
source=registry+https://github.com/rust-lang/crates.io-index
```

Generated Cargo metadata reports:

```text
ratatui_license=MIT
ratatui_rust_version=1.88.0
crossterm_license=MIT
crossterm_rust_version=1.63.0
```

The full graph's maximum declared `rust_version` is the local `winds-control` package at `1.97.1`; the highest declared external dependency MSRV observed is `1.88.0`. Neither exceeds the pinned Winds toolchain `1.97.1`.

The dependency closure metadata inspected for T087 contained permissive SPDX expressions including MIT, Apache-2.0, BSD-2-Clause, Zlib, BSL-1.0, Unlicense, Unicode-3.0, and Apache-2.0 with LLVM exception combinations. No GPL, AGPL, LGPL, SSPL, Commons-Clause, CDDL, EPL, or MPL expression was observed in the inspected closure. This is engineering provenance, not legal advice.

## Evidence-integrity boundary

- This file does not claim final candidate qualification.
- Generator success is not reused as final-head CI evidence.
- Any final candidate HEAD/TREE movement invalidates candidate-bound qualification and review evidence.
- The temporary lock-generation workflow must be absent from the final T087 base-to-head diff before final qualification starts.
- Ubuntu, macOS, and native-Windows build/test claims require fresh exact-head platform evidence under the canonical T087 acceptance gate.
