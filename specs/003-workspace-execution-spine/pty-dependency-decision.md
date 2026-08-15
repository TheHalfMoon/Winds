# T043 PTY Dependency and Provenance Decision

**Decision date**: 2026-08-15

**Canonical feature**: Spec 003 — Workspace Execution Spine

**Decision**: **ACCEPT `portable-pty` 0.9.0 as the preferred direct dependency for the first PTY/ConPTY implementation slice, but do not land the crate until the first runtime slice actually uses it.**

This is a dependency/provenance decision, not a claim that terminal behavior is implemented or that native Windows/WSL support is proven.

## Accepted candidate

| Field | Decision evidence |
|---|---|
| Crate | `portable-pty` |
| Exact package version to request | `=0.9.0` |
| Upstream repository | `wezterm/wezterm` |
| Published-source VCS commit | `f8921727a11b9f8b073e8c24821d72fd41283500` |
| Upstream path | `pty/` |
| License | MIT |
| Default features | none |
| Reuse mode | direct dependency when terminal code first lands; no copied/adapted donor runtime code approved by T043 |
| Current Winds state | approved candidate only; not yet present in `Cargo.toml` or `Cargo.lock` |

Primary package/source evidence:

- https://docs.rs/crate/portable-pty/0.9.0
- https://docs.rs/crate/portable-pty/0.9.0/source/Cargo.toml.orig
- https://docs.rs/crate/portable-pty/0.9.0/source/.cargo_vcs_info.json
- https://docs.rs/crate/portable-pty/0.9.0/source/LICENSE.md

## Why this candidate fits Spec 003

The 0.9.0 public API supplies the concrete primitives Spec 003 needs without requiring a daemon, multiplexer, terminal renderer, or async runtime:

- native cross-platform PTY system selection;
- PTY master/slave allocation;
- child spawn through a command builder;
- master reader/writer handles;
- terminal resize/query;
- child `try_wait` / `wait` and process identity while the child handle is owned;
- a kill handle while Winds still owns the corresponding process/session capability.

The design is synchronous/blocking. That is acceptable for the first Winds slice because Winds can place blocking PTY reads behind bounded owned threads without introducing Tokio solely to service terminal I/O. T050/T051 remain responsible for proving actual lifecycle behavior and race handling.

T043 does **not** authorize reconstructing process ownership from `process_id()` after restart. Spec 003 remains authoritative: persisted PID alone is not identity, and lost ownership becomes `OWNERSHIP_LOST` with no blind signal/kill.

## Dependency footprint audit

`portable-pty` 0.9.0 has no default features. Its published normal direct dependencies are:

- `anyhow 1.0`
- `downcast-rs 1.0`
- `filedescriptor 0.8.3`
- `libc 0.2`
- `log 0.4`
- `nix 0.28` with `term` and `fs`
- `serial2 0.2`
- `shell-words 1.1`

Optional-only dependencies are `serde` and `serde_derive`; Winds does not need the `serde_support` feature for the initial PTY slice.

Windows additionally declares:

- `bitflags 1.3`
- `lazy_static 1.4`
- `shared_library 0.1`
- `winapi 0.3` with console/handle/file/named-pipe/synchronization features
- `winreg 0.10`

Published dev dependencies (`smol`, `futures`) are not required by downstream Winds runtime use.

### Footprint concern: mandatory serial support

`serial2 0.2` is a normal, non-optional dependency in `portable-pty` 0.9.0 even though Winds does not currently need serial TTY support. Its published lock graph includes platform support such as `cfg-if`, `libc`, and `winapi`. This is accepted as a bounded cost for using the mature WezTerm PTY implementation, but it is a known Ponytail pressure point.

The exact **Winds-resolved transitive graph** cannot truthfully be fixed before the crate is inserted into Winds' own manifest and `Cargo.lock`; Cargo version unification and target selection affect that graph. Therefore the runtime landing PR MUST:

1. request exactly `portable-pty = "=0.9.0"`;
2. commit the resulting `Cargo.lock`;
3. inspect the actual resolved direct/transitive additions;
4. rerun the dependency/license audit for those exact locked versions;
5. remove/reconsider `portable-pty` if the resolved footprint or license set materially violates the Spec 003 simplicity/security boundary.

This landing condition is part of the T043 decision; T043 does not pretend the future lockfile already exists.

## Rust 1.97.1 compatibility audit

The published crate uses Rust edition 2018 and declares no `rust-version` / MSRV field. Therefore upstream metadata does not provide an exact MSRV claim.

The crate predates Winds' pinned Rust 1.97.1 toolchain and was successfully published/documented on stable Rust-era tooling. Rust's stable-language compatibility model is designed so previously stable source continues to compile on later stable releases, absent exceptional compiler/soundness breakage. This makes 1.97.1 a reasonable compatibility target, but it is **not treated as execution proof**.

The first PR that actually lands `portable-pty` MUST compile/clippy/test the exact locked dependency graph under Winds' pinned Rust 1.97.1. Until then the decision is `COMPATIBILITY_EXPECTED / RUNTIME_PROOF_PENDING` rather than a false claim of compiler execution.

Rust stability reference:

- https://doc.rust-lang.org/edition-guide/editions/index.html

## Platform behavior audit

### Linux / macOS

The crate exposes the Unix PTY implementation needed for allocation, resize, owned reader/writer access, spawning, and child lifecycle. T050 must still prove Winds-specific resource ownership, interrupt/close behavior, bounded streams, and no leaked directly owned child in controlled lifecycle tests.

### Windows

`native_pty_system()` selects the crate's ConPTY implementation on Windows and the published package carries Windows console/handle/named-pipe dependencies. This is sufficient for a dependency decision, **not** a Winds support claim.

A material risk was found: `portable-pty-psmux` exists specifically because its maintainers need newer ConPTY creation flags (`PSEUDOCONSOLE_RESIZE_QUIRK`, `WIN32_INPUT_MODE`, and `PASSTHROUGH_MODE`) that upstream `portable-pty` 0.9.0 does not expose. Winds will not pre-emptively take that fork. T051 must test Winds' actual Windows behavior first; only demonstrated failures may justify a narrowly reviewed alternative or upstream patch.

Reference:

- https://docs.rs/crate/portable-pty-psmux/0.9.6/source/README.md

### WSL

WSL selection/path mapping is outside the PTY crate's responsibility. Spec 003 uses Microsoft's supported `wsl.exe` surface for WSL discovery/launch. T052/T062 remain responsible for real WSL integration evidence.

## Alternatives considered

### `xpty` 0.3.6 — REJECT for first slice

Pros:

- explicitly declares `rust-version = "1.70"`;
- moves `serial2` behind an optional `serial` feature;
- modernizes dependency versions and error typing;
- provides Linux/macOS/Windows CI in its own project.

Why not now:

- it is a young fork of `portable-pty` 0.9.0 rather than the source used by WezTerm;
- its own README describes async support and better ConPTY control as planned improvements;
- Winds currently needs mature bounded PTY mechanics more than a newer fork surface.

Reference:

- https://docs.rs/crate/xpty/0.3.6/source/README.md
- https://docs.rs/crate/xpty/0.3.6/source/Cargo.toml.orig

### `rust-pty` 0.5.0 — REJECT for first slice

It offers a cross-platform Unix/ConPTY abstraction with first-class async I/O, but its model is Tokio-based. Adding an async runtime solely for PTY I/O would expand Winds' runtime model before a measured need exists.

Reference:

- https://docs.rs/rust-pty/0.5.0/rust_pty/

### `portable-pty-psmux` 0.9.6 — REJECT pending demonstrated need

This fork is valuable risk evidence for modern Windows ConPTY behavior, but adopting a fork before Winds demonstrates that the extra flags are required would violate Ponytail/YAGNI. Keep it as a fallback reference for T051.

Reference:

- https://docs.rs/crate/portable-pty-psmux/0.9.6/source/README.md

### Unix-only PTY crates — REJECT

`ptyprocess`, `pty-process`, and `pty` can cover Unix PTY behavior but do not satisfy Spec 003's single dependency direction for native Windows ConPTY. Using separate unrelated Unix/Windows libraries would increase integration and behavior divergence without a proven benefit.

## License / notice decision

`portable-pty` 0.9.0 is MIT licensed. If/when the dependency lands:

- preserve its upstream license/notice requirements in release dependency notices;
- record the exact locked package set in the release license audit;
- do not imply that Winds' dual `MIT OR Apache-2.0` license relicenses the dependency;
- no copied/adapted WezTerm code is approved by this decision.

## Final T043 verdict

**ACCEPT `portable-pty = "=0.9.0"` as the first implementation dependency candidate.**

The accepted boundary is intentionally narrow:

- dependency, not copied code;
- no features initially;
- no daemon/multiplexer/renderer adoption;
- no PID-based restart ownership;
- no Windows support claim until T051 evidence;
- no WSL support claim until T052/T062 evidence;
- actual Rust 1.97.1 compile and exact Winds lockfile/license graph are mandatory at dependency landing.

If those landing gates fail, T043's candidate decision must be reopened rather than patched around silently.
