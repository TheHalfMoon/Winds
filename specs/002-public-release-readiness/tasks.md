# Tasks: Public Release Readiness

This checklist records implementation/evidence truth for Spec 002. Checked items require repository evidence; nearby work does not imply completion. Public publication remains separately gated even after all implementation tasks are accepted.

## Phase 1 - Canonical Release-Readiness Baseline

- [x] **T024** Establish Spec 002 `spec.md`, `plan.md`, and `tasks.md` from canonical `main` after walking-skeleton T001-T023 completion. Define the non-negotiable publication boundary: no visibility change, `v0.1.0` tag, GitHub Release, or package publication without a separate explicit founder publication authorization.

## Phase 2 - License and Provenance Closure

- [x] **T025** Produce a release-facing licensing/provenance audit from the exact current dependency/provenance state. Exact-head probe `1516078d22ae64cd0a3b04a0d402351669846e70` used `cargo metadata --locked` and found 49/49 non-workspace packages with license metadata; ordinary quality also passed on Ubuntu/macOS. `docs/release/license-audit.md` records the grouped inventory, donor reuse modes, no identified copied donor runtime code, source-license compatibility, and the additional Unicode-3.0 notice obligation for `unicode-ident`. The temporary probe workflow is removed rather than retained as permanent CI.
- [x] **T026** Add the standard `MIT OR Apache-2.0` source license set and reconcile `docs/provenance/donors.md` / release audit with the chosen source-license truth. `LICENSE-MIT` uses the standard MIT terms with `Winds contributors`; `LICENSE-APACHE` is the standard Apache License 2.0 text. The provenance ledger explicitly distinguishes the Winds source license from unchanged third-party dependency/notice obligations.
- [ ] **T027** Reconcile public package metadata for the 0.1 release candidate: `version = "0.1.0"`, `license = "MIT OR Apache-2.0"`, repository, README, concise description, while preserving `publish = false`.

## Phase 3 - Public Project Surface

- [ ] **T028** Rewrite/extend `README.md` as a fresh-evaluator front door: product wedge, proven 0.1 behavior, trust model, supported environments, pinned build/test quickstart, minimal existing-ref verification/promotion example, safety boundary, deferred scope, and links to contributor/security/provenance docs.
- [ ] **T029** Add `CONTRIBUTING.md` that reuses rather than duplicates governance: Constitution -> Spec -> Plan -> Tasks -> Implement -> deterministic checks -> correctness/safety -> Ponytail -> independent review -> evidence reconciliation.
- [ ] **T030** Add `SECURITY.md` with the precise 0.1 security boundary and a proven private vulnerability-reporting path. Do not claim sandboxing or promise an unsupported response SLA.
- [ ] **T031** Add a concise `CHANGELOG.md` / 0.1 release-notes source that distinguishes shipped/proven behavior from explicitly deferred scope and avoids stale exact-run metadata in long-lived source text.

## Phase 4 - Non-Publishing Release Candidate

- [ ] **T032** Add one minimal release-candidate workflow that is manual or narrowly release-scoped, checks out an exact candidate SHA, uses read-only repository permissions, pinned Rust `1.97.1`, committed `Cargo.lock`, and performs zero tag/release/visibility/package mutations.
- [ ] **T033** Make the release-candidate workflow rerun existing deterministic quality and the SC-001 100-cycle soak on the exact candidate before accepting artifacts.
- [ ] **T034** Build only proven 0.1 distribution targets using then-current official runner/target support: Linux x86-64 and macOS arm64 when CI proves it. Do not emit or imply native Windows support.
- [ ] **T035** Produce version/target-identifying release-candidate artifact names plus a SHA-256 checksum manifest, and verify the manifest in CI without adding signing/attestation infrastructure.
- [ ] **T036** Execute the complete release-candidate dry run on one exact candidate commit and record reproducible evidence for quality, SC-001, target builds, checksums, and zero publication-side effects.

## Phase 5 - Public-Repository Hygiene and Review

- [ ] **T037** Perform a tracked-content public-hygiene audit for secrets/tokens, private or temporary URLs, local absolute paths, accidental personal-machine details, stale internal-only/release language, unsupported security/platform claims, and copied third-party code missing provenance. Treat real secret-history findings as a separate incident; do not rewrite history automatically.
- [ ] **T038** Run final correctness/safety review on the exact release-readiness implementation head, including license/version consistency, provenance, workflow permissions/side effects, artifact identity, public-doc truth, and publication boundary.
- [ ] **T039** Run Ponytail v4.9.0 final simplicity review and remove unjustified release tooling, dependencies, abstractions, installer/package-manager surfaces, or duplicate governance.
- [ ] **T040** Obtain and reconcile at least one independent reviewer pass on the exact final implementation head. A bot summary, skipped/rate-limited response, or older-head review is not sufficient.

## Founder Publication Gate

- [ ] **T041** **FOUNDER-CONTROLLED / NOT IMPLIED BY `go ahead` FOR IMPLEMENTATION** — after T025-T040 are accepted and merged, obtain a separate explicit founder publication authorization naming the exact release commit before changing repository visibility, creating `v0.1.0`, creating a GitHub Release, or publishing any package. Until then Winds remains private and unreleased.

## Explicitly Deferred Beyond This Feature

- crates.io publication and crate-distribution automation.
- native Windows artifacts.
- installer scripts, auto-updater, Homebrew formula/tap, container image, package-manager integrations.
- signing/attestation/SBOM infrastructure beyond the committed lockfile and SHA-256 manifest.
- agent adapters, `winds race`, UI/TUI/dashboard, daemon/runtime protocol, sandbox framework, MCP/A2A, Graphify/Jujutsu.
- automatic tag/release/version publication bots.
