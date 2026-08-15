# Implementation Plan: Public Release Readiness

## Summary

Prepare the already-proven Winds 0.1 walking skeleton for a first public open-source release without changing product behavior or performing publication. The release-readiness slice is repository metadata, licensing/provenance, public contributor/security documentation, and a non-publishing release-candidate workflow that reproduces deterministic quality, the SC-001 soak, and platform artifact/checksum evidence on an exact commit.

The public transition itself remains a separate founder-controlled gate after this implementation is merged and independently reviewed.

## Constitution Check

- Evidence over claims: release notes and readiness claims bind to exact CI/soak/artifact evidence, not agent prose: PASS.
- Non-destructive Git safety: no product Git behavior changes; release dry-run does not tag, push, release, or mutate repository visibility: REQUIRED.
- Spec -> Plan -> Tasks precede implementation: PASS.
- Ponytail/YAGNI: no generic release framework, signing system, package publication, daemon, UI, adapter, or product abstraction: REQUIRED.
- Independent review before acceptance: REQUIRED.

## Current Baseline

Canonical base at specification start:

`e4c94ed98743f39bbc4da9ad3e9b8fa3cfb4b7ef`

Baseline facts verified before this plan:

- Walking-skeleton tasks T001-T023 are complete on `main`.
- Post-merge quality run #99 passed format, Clippy `-D warnings`, and tests on Ubuntu/macOS.
- No Git tags exist.
- No GitHub Releases exist.
- Repository visibility is private.
- GitHub reports no repository license.
- No root `LICENSE` file exists.
- `Cargo.toml` is `version = "0.0.0"`, `publish = false`, with no release license/repository/description metadata.
- Provenance ledger states no copied donor runtime code is currently present.

## Technical Context

- **Product code**: Rust 1.97.1, one unpublished package, system Git, SQLite WAL, existing safety model unchanged.
- **Distribution intent**: GitHub release artifacts for proven 0.1 platforms; crates.io publication remains disabled.
- **Licensing intent**: `MIT OR Apache-2.0`, subject to the audit task and explicit founder amendment if needed.
- **Release evidence**: exact commit, deterministic quality, SC-001 100-cycle soak, platform builds, SHA-256 manifest.
- **Publication boundary**: branch/PR implementation may prepare release assets but cannot make the repository public, create `v0.1.0`, create a GitHub Release, or publish a package.

## Delivery Architecture

No product architecture changes.

Repository-local release readiness consists of four small surfaces:

1. **Legal/provenance metadata** — license texts, package license metadata, and one audit document.
2. **Public-facing repository docs** — README, CONTRIBUTING, SECURITY, CHANGELOG/release notes as needed.
3. **Release-candidate automation** — one narrowly scoped/manual dry-run workflow using existing Cargo/GitHub primitives.
4. **Founder publication gate** — an explicitly unautomated decision/action after readiness evidence is accepted.

Do NOT introduce a release service, custom package manager, installer framework, generic artifact pipeline abstraction, signing/attestation stack, auto-versioning bot, changelog service, or multi-package workspace solely for this feature.

## Licensing and Provenance Strategy

### Source license

Use the standard Rust-friendly dual permissive expression:

`MIT OR Apache-2.0`

Rationale:

- commercial and open-source reuse remains permissive;
- Apache-2.0 supplies an explicit patent grant;
- MIT preserves broad ecosystem compatibility;
- current direct dependency/process-reference licensing is expected to be compatible;
- no copied donor runtime code is currently recorded.

The audit must still verify current direct/transitive dependency and donor reuse before licensing is accepted. If an incompatibility is found, stop and reconcile rather than weakening the ledger.

### Provenance

`docs/provenance/donors.md` remains the source ledger for material external influence. Add a release-facing license audit that records:

- package/dependency identity and lock pin where relevant;
- upstream license;
- Winds reuse mode;
- whether license text/notice obligations are triggered;
- any unresolved blocker.

Do not duplicate every Cargo.lock row manually if a deterministic machine-readable scan can prove the same information more reliably with less maintenance.

## Package and Version Metadata

Release candidate package metadata should move from internal placeholder `0.0.0` to `0.1.0` while preserving:

`publish = false`

Add only metadata justified by public consumption:

- `license = "MIT OR Apache-2.0"`
- repository URL
- README path
- concise description

Do not add crates.io categories/keywords/homepage/documentation fields unless they are needed by a real distribution surface.

## Public Documentation Strategy

### README

The README should become the front door for a fresh evaluator:

- what Winds is;
- what 0.1 actually does;
- trust/evidence model;
- supported environments;
- build/test quickstart;
- minimal verify/promote example using explicit existing refs;
- safety boundary and explicitly deferred scope;
- links to CONTRIBUTING, SECURITY, and provenance.

Avoid marketing claims not backed by current behavior.

### CONTRIBUTING

Reuse repository governance rather than invent a second process. Summarize:

Constitution -> Spec -> Plan -> Tasks -> Implement -> deterministic gate -> correctness/safety -> Ponytail -> independent review -> evidence reconciliation.

### SECURITY

State the current security boundary precisely. Provide a private reporting path supported by GitHub/repository configuration when one can be proven. Do not publish a personal address merely for convenience unless explicitly chosen. Do not promise response times that are not operationally staffed.

### CHANGELOG / release notes

Capture 0.1 behavior and deferred scope. Release notes must reference exact readiness evidence at the final release candidate, but exact run IDs should live in review/release metadata where appropriate rather than becoming stale source truth.

## Release-Candidate Workflow

One dry-run workflow, manual or narrowly release-scoped, must:

1. Check out an exact requested/current candidate commit with credentials persistence disabled.
2. Install pinned Rust 1.97.1.
3. Run existing quality commands with `--locked`.
4. Run the existing ignored SC-001 100-cycle soak.
5. Build release binaries only for proven supported targets.
6. Produce deterministic filenames containing Winds version and target identity.
7. Produce a SHA-256 checksum manifest.
8. Upload workflow artifacts only; do not create tags/releases/packages or alter repository visibility.

Use then-current official GitHub runner/target documentation when implementing target jobs. Do not guess macOS architecture labels from memory.

## Artifact Strategy

Minimum intended 0.1 distribution:

- Linux x86-64 binary for Linux and supported WSL2 Linux-domain usage.
- macOS arm64 binary when CI can prove the target using an official runner/build path.

Native Windows remains explicitly absent.

No installer, auto-updater, Homebrew formula, shell script installer, container image, crates.io package, or package-manager tap is required for this slice.

Artifact format should remain simple: executable file (optionally inside a minimal archive if GitHub artifact transport requires it) plus SHA-256 manifest. Avoid archive-format complexity unless necessary.

## Public-Repository Hygiene

Before publication, inspect tracked repository content for:

- secret/token patterns;
- private GitHub/API URLs or temporary signed download URLs;
- local absolute filesystem paths;
- personal machine/usernames where not intentionally attribution;
- stale “private/pre-alpha/internal-only” text that contradicts release state;
- unsupported security/platform claims;
- copied third-party code without provenance.

A finding blocks publication until reconciled. Do not auto-redact historical Git data in this feature; history-rewrite decisions require a separate explicit incident response if a real secret is found.

## Deterministic Gates

For implementation PRs:

1. Existing `quality` workflow on exact head.
2. Any focused checks for metadata/docs/workflow correctness.
3. Correctness/safety review.
4. Ponytail v4.9.0 simplicity review.
5. Independent reviewer pass.
6. Evidence reconciliation.

For the final release candidate dry run:

1. exact-head quality;
2. exact-head SC-001 100-cycle soak;
3. exact-head platform artifact build(s);
4. SHA-256 manifest verification;
5. public-hygiene scan/review;
6. zero publication-side effects.

## Task Sequencing

Implement in this order:

1. Canonical Spec 002 baseline.
2. License/provenance audit.
3. License files and package metadata.
4. Public-facing contributor/security/release documentation.
5. Release-candidate workflow and artifact/checksum evidence.
6. Public-hygiene review.
7. Correctness/safety + Ponytail + independent review.
8. Stop at founder publication gate.

The order intentionally resolves legal/provenance uncertainty before polishing distribution automation.

## Review Strategy

### Correctness/safety

Review for:

- inconsistent license/version metadata;
- copied-code/provenance omissions;
- release workflow write permissions or hidden publication side effects;
- unsupported platform/security claims;
- stale/private information leaks;
- artifacts not bound to the exact release commit/version.

### Ponytail

Delete any release framework/dependency/automation not strictly required for the first GitHub release candidate. Prefer shell/Cargo/GitHub Actions built-ins over new tools when they are sufficient.

### Independent review

At least one independent reviewer must inspect the exact final implementation head. Bot summaries, rate-limit responses, or review results bound only to older heads do not satisfy the gate.

## Publication Boundary

Even after this feature PR is accepted and merged:

- repository visibility remains private;
- no `v0.1.0` tag is created;
- no GitHub Release is created;
- no crate is published.

Those actions require a later explicit founder publication authorization naming the exact release commit. This boundary is part of the feature, not an administrative suggestion.
