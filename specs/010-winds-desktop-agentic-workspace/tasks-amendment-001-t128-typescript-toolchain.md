# Spec 010 Tasks Amendment 001 — T128 TypeScript Toolchain Qualification

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: canonical Spec 010 entry/spec/plan/tasks, canonical T127 closure, the Standard Acceptance Gate requiring frontend typecheck/lint/test/build commands, and the Founder directive to continue the authorized desktop program without silently adding unqualified dependencies.

## Purpose

T128 is authorized to qualify the selected desktop dependency graph and land an inert Tauri/React shell. Its accepted task contract selects React 19.3 and Vite 8 and requires deterministic frontend typechecking.

The selected direct-version block, however, does not name a TypeScript compiler or React declaration packages. Implementing a TSX renderer while claiming a real `typecheck` gate would therefore require either an undeclared toolchain dependency or a misleading non-typechecking command.

This amendment closes only that contract gap. It does not change the desktop architecture, product behavior, runtime authority, or task ordering.

## Canonical predecessor state

This amendment is valid only while repository truth continues to prove:

```text
T127=CLOSED_CANONICAL
T127_MERGE=09d50fe185c061f23d21d4865f82b07a55736765
T128=AUTHORIZED
T128_IMPLEMENTATION_LANDED=NO
```
## Exact additional dependency authority

T128 may add the following exact **development-only** direct packages to `desktop/package.json` and its committed `package-lock.json`:

```text
typescript=7.0.2
@types/react=19.3.0
@types/react-dom=19.3.0
@types/node=22.20.2
```

These packages exist only to make TSX/configuration typechecking deterministic under the already-selected Node/Vite/React toolchain.

They do not authorize additional runtime JavaScript, renderer state frameworks, component systems, CSS systems, test frameworks, linters, routers, query clients, editor frameworks, animation frameworks, or Tauri plugins.

`@types/node` is authorized only because Vite's Node-facing configuration surface declares `@types/node` compatibility and the exact selected version satisfies that range. It grants no Node runtime inside the Tauri application renderer.

## Required deterministic frontend gates

T128 must define and document canonical npm scripts for:

- formatting/checking authored frontend text without adding a formatter framework;
- TypeScript typechecking with `tsc` and no emit;
- lint-equivalent static checking using only already-authorized compiler/build surfaces unless a later amendment separately authorizes a dedicated linter;
- deterministic frontend tests using only already-authorized platform/runtime primitives unless a later task proves a test framework necessary;
- Vite production build;
- Tauri release-mode host build on directly claimed host(s).
The lint-equivalent gate may intentionally reuse the TypeScript compiler with stricter/no-pretty output in T128; this amendment does not authorize installing ESLint, Biome, Oxlint, or another lint framework merely to make the command name distinct.

The test gate may use Node's built-in `node:test` for inert-shell/configuration assertions. A React/component test framework is not required or authorized by this amendment.

All commands must run through the committed npm dependency graph. A globally installed `tsc`, package-manager shim, or uncommitted tool is insufficient acceptance evidence.

## Provenance requirements

T128 dependency provenance must record for every selected direct npm package and Rust crate, including the four packages added here:

- exact version;
- registry/source URL or canonical package identity;
- declared license;
- package integrity/checksum available from the package manager/registry;
- engine or MSRV requirement when declared;
- direct runtime versus development-only classification;
- why the dependency is necessary in the owning task.

The committed lockfiles remain the canonical transitive graph evidence. No third-party source or asset may be copied into Winds under this amendment.

## Security and authority invariants

Adding compiler/type packages must not change the renderer trust boundary. T128 still requires:

- local bundled content only for the privileged window;
- no remote capability URLs;
- no shell/filesystem plugin;
- no generic Tauri command dispatcher;
- no Winds Store/Git/PTTY/runtime mutation from the renderer;
- no Node runtime privilege in the production renderer;
- no credential access or persistence;
- no new public IPC/control surface.
## Explicit non-authorization

This amendment does not authorize:

- changing React, Vite, Tauri, xterm.js, or Lucide versions already selected by T128;
- adding a second JavaScript package manager;
- adding a renderer runtime dependency beyond the already-selected T128 list;
- adding a dedicated lint/format/test framework;
- changing Rust core behavior, CLI/TUI behavior, schema, migrations, runtime/provider execution, Git behavior, terminal ownership, or Model Mesh behavior;
- weakening CSP or Tauri ACL/capability boundaries;
- remote content, remote-origin capabilities, shell/filesystem plugins, generic invoke, or browser-like navigation;
- beginning T129 product visual-system work before T128 closes canonically.

## Amendment acceptance gate

This amendment is governance-only and becomes canonical only after:

- changed scope exactly this amendment document;
- `git diff --check` PASS;
- repository `quality` SUCCESS on the exact amendment head;
- author correctness/safety/governance review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review with zero material findings;
- zero unresolved review threads;
- exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge push workflow succeeds.

No CI result or review from a superseded head may qualify a successor.
## T128 continuation after landing

After this amendment is canonical, T128 may proceed from then-current canonical `main` using the original T128 paths plus the exact four development-only packages above.

T128 must preserve this amendment as provenance rather than silently folding the packages into history. The final T145 reconciliation must record both the original selected stack and this toolchain amendment.

Only after this amendment lands and post-merge verification completes may repository truth state:

```text
SPEC_010_TASKS_AMENDMENT_001=CLOSED_CANONICAL
T128_TYPESCRIPT_TOOLCHAIN_EXTENSION=AUTHORIZED
T128=AUTHORIZED
T129=BLOCKED_PENDING_T128
```

Landing this amendment does not itself close T128 and does not authorize T129.