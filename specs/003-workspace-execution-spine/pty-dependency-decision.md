# T043 PTY Dependency and Provenance Decision

**Decision date**: 2026-08-15

**Canonical feature**: Spec 003 — Workspace Execution Spine

**Historical T043 decision**: **ACCEPT `portable-pty` 0.9.0 as the preferred direct dependency for the first PTY/ConPTY implementation slice, with landing deferred until the first runtime slice that actually used it.**

**Current status**: **ACCEPTED, LANDED, LOCK-AUDITED, AND PLATFORM-PROVEN FOR THE ACCEPTED SPEC 003 WORKSPACE-TERMINAL SURFACE.**

This document preserves the original T043 dependency/provenance reasoning while reconciling it with the runtime evidence that landed afterward. T043 itself was a dependency decision; T050-T052 and T061-T062 supplied the implementation/platform proof.

## Accepted dependency

| Field | Decision / current evidence |
|---|---|
| Crate | `portable-pty` |
| Exact package version | `=0.9.0` |
| Upstream repository | `wezterm/wezterm` |
| Published-source VCS commit | `f8921727a11b9f8b073e8c24821d72fd41283500` |
| Upstream path | `pty/` |
| License | MIT |
| Default features | none |
| Reuse mode | direct dependency; no copied/adapted donor runtime code approved by T043 |
| Current Winds state | landed by T050 with exact pin and committed lockfile; exact locked dependency/license audit recorded in `docs/provenance/portable-pty-0.9.0-lock-audit.md` |

Primary package/source evidence used by the original decision:

- https://docs.rs/crate/portable-pty/0.9.0
- https://docs.rs/crate/portable-pty/0.9.0/source/Cargo.toml.orig
- https://docs.rs/crate/portable-pty/0.9.0/source/.cargo_vcs_info.json
- https://docs.rs/crate/portable-pty/0.9.0/source/LICENSE.md

## Why this dependency fits Spec 003

The 0.9.0 public API supplies the concrete primitives Spec 003 needs without requiring a daemon, multiplexer, terminal renderer, or async runtime:

- native cross-platform PTY system selection;
- PTY master/slave allocation;
- child spawn through a command builder;
- master reader/writer handles;
- terminal resize/query;
- child `try_wait` / `wait` and process identity while the child handle is owned;
- a kill handle while Winds still owns the corresponding process/session capability.

The design is synchronous/blocking. Winds uses bounded owned-thread/lifecycle machinery rather than introducing Tokio solely to service terminal I/O.

Nothing in the dependency changes Spec 003 restart authority: a persisted PID is not process identity, and lost ownership becomes `OWNERSHIP_LOST` with no blind signal/kill.

## Landing gates and their disposition

T043 required the first runtime PR that used the crate to:

1. request exactly `portable-pty = "=0.9.0"`;
2. commit the Winds-resolved `Cargo.lock`;
3. inspect the actual resolved direct/transitive additions;
4. rerun the dependency/license audit for those exact locked versions;
5. compile/clippy/test the exact graph under Winds' pinned Rust toolchain;
6. reopen the dependency decision rather than silently work around a material footprint/license failure.

**Those landing gates were satisfied by T050.** PR #23 landed the exact pin and lockfile, passed the pinned Rust 1.97.1 quality/release gates, and recorded the exact locked transitive/license audit in `docs/provenance/portable-pty-0.9.0-lock-audit.md`. The decision therefore no longer has `RUNTIME_PROOF_PENDING` status.

## Dependency-footprint audit

The original published-package audit identified normal dependencies including `anyhow`, `downcast-rs`, `filedescriptor`, `libc`, `log`, `nix`, `serial2`, and `shell-words`, plus Windows support dependencies. T050's exact lock audit supersedes any attempt to infer the final Winds graph from published metadata alone; the committed `Cargo.lock` and the lock-audit document are the canonical resolved-graph evidence.

### Mandatory serial-support pressure

`serial2` was identified at T043 as a non-optional footprint cost even though Winds does not need serial TTY support. That Ponytail pressure was accepted as the bounded cost of using the mature WezTerm PTY implementation. T067 later re-challenged the final direct-dependency surface and found no justified dependency removal or replacement.

## Rust 1.97.1 compatibility

At T043, upstream metadata provided no exact MSRV proof, so compatibility was only expected. That uncertainty is now resolved for the Winds use case: T050 and subsequent quality/platform gates compiled, linted, and tested the locked graph under Winds' pinned Rust 1.97.1 toolchain.

This is Winds execution evidence for the accepted graph; it is not a claim about every possible `portable-pty` consumer or feature combination.

## Platform evidence after landing

### Linux / macOS

T050 proved the accepted Unix PTY lifecycle: allocation, canonical cwd, one output consumer, input/output, resize/current-size, owned-child observation/termination/reaping, and ownership-scoped foreground-process-group interrupt behavior.

### Native Windows

T051 proved the accepted `portable-pty` ConPTY path on native Windows for create/input/output/resize/exit/terminate/close/reap. The platform evidence did **not** prove a safe ownership-scoped ConPTY interrupt primitive, so native-Windows `interrupt()` remains explicitly fail-closed rather than falling back to process-global console signaling. T061 later broadened official-Windows touched-surface evidence.

The historical `portable-pty-psmux` risk remains useful reference material, but Winds did not pre-emptively adopt that fork because accepted native-Windows behavior was proven without it.

### WSL

WSL identity, path mapping, and distribution selection are intentionally outside the PTY crate. T052 implemented the explicit WSL launch/mapping boundary, and T062 supplied real Windows Server 2025 + Ubuntu WSL2 integration evidence. That evidence does not convert `portable-pty` into the authority for WSL identity or Git equivalence.

## Alternatives considered by T043

### `xpty` 0.3.6 — REJECTED for first slice

It offered a newer fork surface and optional serial support, but at decision time Winds preferred the mature WezTerm-derived implementation and did not have a demonstrated reason to switch. No later Spec 003 evidence has required reopening that choice.

### `rust-pty` 0.5.0 — REJECTED for first slice

Its Tokio-oriented model would have expanded Winds' runtime model before a measured need existed.

### `portable-pty-psmux` 0.9.6 — RETAINED AS REFERENCE ONLY

Its extra ConPTY flags remain useful risk evidence, but the accepted Winds native-Windows slice did not demonstrate a need to adopt the fork.

### Unix-only PTY crates — REJECTED

Separate unrelated Unix/Windows libraries would increase platform divergence without a proven benefit for the accepted cross-platform slice.

## License / notice status

`portable-pty` 0.9.0 is MIT licensed and is now a landed dependency. Winds therefore:

- preserves the dependency's upstream license/notice requirements in release dependency notices;
- records the exact locked package set in the release/license audit;
- does not imply that Winds' `MIT OR Apache-2.0` project license relicenses the dependency;
- has not approved copied/adapted WezTerm runtime code through this decision.

T050 also reconciled the two exact `winapi-*-pc-windows-gnu 0.4.0` package tuples required by the locked graph through the fail-closed release license collector and provenance records.

## Current final verdict

**`portable-pty = "=0.9.0"` is ACCEPTED AND LANDED for the Spec 003 workspace-terminal implementation.**

The accepted boundary remains narrow:

- direct dependency, not copied donor code;
- exact version pin and committed lockfile;
- no daemon, multiplexer, renderer, public runtime protocol, or plugin/provider framework;
- no PID-based restart ownership;
- native-Windows workspace/terminal support only to the behavior actually proven by T051/T061;
- WSL support only to the behavior actually proven by T052/T062;
- no implication that native-Windows authoritative `winds verify` required-check execution is supported.

Any future dependency switch or broader runtime claim requires its own evidence rather than treating the historical T043 candidate wording as current implementation truth.
