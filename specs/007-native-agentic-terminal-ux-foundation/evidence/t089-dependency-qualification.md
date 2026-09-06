# T089 Dependency Qualification Evidence

Status: `CANDIDATE_INPUT_ONLY`

This note records source, checksum, license, MSRV, and capability qualification for the exact dependency authority granted by Spec 007 T089. It is input evidence only. Final T089 qualification still requires fresh exact-head CI, focused tests, scope reconciliation, and review on the final candidate.

## Authorized direct dependency

```toml
vt100 = "=0.16.2"
```

No other new direct dependency is authorized or added by T089.

## Published package metadata

The published/upstream `vt100` 0.16.2 manifest declares:

```text
name=vt100
version=0.16.2
license=MIT
rust_version=1.70
dependencies=itoa 1.0.15, unicode-width 0.2.1, vte 0.15.0
```

The published/upstream `vte` 0.15.0 manifest declares:

```text
name=vte
version=0.15.0
license=Apache-2.0 OR MIT
rust_version=1.62.1
default_features=std
```

With only the default `std` feature active, its selected non-optional dependencies are `arrayvec` and `memchr`. Its optional `ansi` feature is not required by `vt100` and is not selected by the T089 direct dependency declaration.

The resolved `arrayvec` 0.7.6 manifest declares:

```text
name=arrayvec
version=0.7.6
license=MIT OR Apache-2.0
rust_version=1.51
default_features=std
```

Its `borsh`, `serde`, and `zeroize` dependencies are optional and are not selected by the T089 path.

All declared external MSRVs above are below the Winds pinned Rust `1.97.1` toolchain.

## Candidate lock graph

The T089 candidate lock records the following new registry packages:

```text
arrayvec=0.7.6
arrayvec_checksum=7c02d123df017efcdfbd739ef81735b36c5ba83ec3c59c80a9d7ecc718f92e50

vt100=0.16.2
vt100_checksum=054ff75fb8fa83e609e685106df4faeffdf3a735d3c74ebce97ec557d5d36fd9

vte=0.15.0
vte_checksum=a5924018406ce0063cd67f8e008104968b74b563ee1b85dde3ed1f7cb87d3dbd

source=registry+https://github.com/rust-lang/crates.io-index
```

The same crate archive checksums are independently visible in public FreeBSD ports provenance for these crate versions.

The existing Winds lock already provides compatible selected versions for the remaining `vt100`/`vte` requirements:

```text
itoa=1.0.18
unicode-width=0.2.2
memchr=2.8.3
```

Therefore the candidate graph adds exactly three registry packages and one direct root dependency. Fresh `cargo --locked` CI remains the authoritative check that the committed lock matches Cargo's resolver result.

## Capability and feature boundary

The T089 dependency path introduces no network/provider/browser client, Tokio/async runtime, clipboard automation crate, daemon/service framework, PTY implementation, editor framework, Git mutation library, or generalized plugin/runtime layer.

`vt100` is used only through the published synchronous parser seam:

```text
Parser::new_with_callbacks(...)
Parser::process(...)
```

Terminal-originated callbacks are treated as untrusted presentation requests. The Winds-owned callback implementation records bounded saturating counters only. It does not retain callback payloads and does not open URLs/files, write or read the host clipboard, execute commands, make network requests, mutate Git/repository state, change evidence authority, or accept terminal-originated resize requests as host authority.

Screen resizing occurs only through the explicit Winds pane-size update seam.

## T089 boundedness boundary

The candidate uses `vt100` with zero library scrollback and maintains the Winds-owned transcript separately. Transcript retention is bounded at the Spec FR-050 ceiling:

```text
MAX_TRANSCRIPT_LINES=100000
MAX_TRANSCRIPT_BYTES=33554432
```

Eviction/truncation is explicit state. Terminal strings such as `PASS`, `VERIFIED`, `ACCEPTED`, forged JSON, OSC data, hyperlinks, titles, and clipboard requests remain terminal presentation data only.

## Evidence-integrity boundary

- This file does not claim final T089 qualification.
- Public source metadata and checksums are dependency-provenance inputs, not exact-head CI evidence.
- The committed lock must pass fresh `--locked` CI on the exact final candidate.
- Focused T089 tests must be registered and executed on the exact final candidate.
- Any HEAD/TREE movement invalidates candidate-bound CI and review evidence.
- Platform claims remain limited to directly exercised domains.
