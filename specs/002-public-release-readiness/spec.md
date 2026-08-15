# Feature Specification: Public Release Readiness

**Feature Branch**: `spec/002-public-release-readiness`

**Created**: 2026-08-15

**Status**: Authorized for release-readiness implementation; public publication remains separately gated

**Input**: Prepare the completed Winds 0.1 verification walking skeleton for a trustworthy first public open-source release without adding product behavior, weakening safety claims, or publishing/tagging the repository before an explicit founder publication decision.

## User Scenarios & Testing

### User Story 1 - Understand the License and Provenance (Priority: P1)

A prospective user, contributor, or company evaluating Winds can determine what license applies to Winds, which external projects materially influenced it, and whether copied donor runtime code is present.

**Why this priority**: A public repository without clear licensing and provenance is not safely reusable, regardless of code quality.

**Independent Test**: Inspect the repository from a fresh checkout and confirm the root licensing files, package metadata, and provenance ledger agree; every current dependency/process donor has a recorded license/reuse mode; no copied runtime donor code is claimed unless exact provenance is recorded.

**Acceptance Scenarios**:

1. **Given** the release candidate tree, **When** a user inspects licensing, **Then** Winds has an explicit permissive open-source license choice with consistent package/repository metadata.
2. **Given** the release candidate tree, **When** provenance is inspected, **Then** current donor/dependency relationships are documented and no unresolved copied-code provenance remains.
3. **Given** an unresolved or incompatible licensing/provenance item, **When** release readiness is evaluated, **Then** publication is blocked rather than guessed through.

### User Story 2 - Evaluate and Build Winds from a Fresh Checkout (Priority: P1)

A technical evaluator can understand the product boundary, supported environments, build Winds with the pinned toolchain and lockfile, and run the documented first verification flow without relying on private project knowledge.

**Why this priority**: The first public experience must reproduce the same bounded product truth already proven internally.

**Independent Test**: From a fresh supported-environment checkout, follow only repository documentation to build with `--locked`, run deterministic tests, and identify the exact supported/unsupported safety boundaries.

**Acceptance Scenarios**:

1. **Given** a fresh checkout, **When** the documented build steps are followed, **Then** Winds builds using the pinned Rust toolchain and committed lockfile.
2. **Given** the public README, **When** a user evaluates safety claims, **Then** worktree isolation is not described as OS/network/secret sandboxing and no automatic merge/winner behavior is implied.
3. **Given** the first release metadata, **When** a user checks the version, repository, license, and description, **Then** those fields consistently describe the same `0.1.0` release candidate while crate publication remains disabled unless separately authorized.

### User Story 3 - Contribute or Report a Vulnerability Safely (Priority: P2)

A contributor can discover the expected development/review workflow, and a security reporter can identify a responsible disclosure path without opening sensitive vulnerability details in a public issue by default.

**Why this priority**: Public visibility creates contributor and security intake immediately.

**Independent Test**: Inspect the repository root and confirm contributor/security documents are discoverable, align with `AGENTS.md` and the constitution, and do not promise unsupported SLAs or security properties.

**Acceptance Scenarios**:

1. **Given** a contributor, **When** they read `CONTRIBUTING.md`, **Then** the Spec Kit -> deterministic checks -> correctness/safety -> Ponytail -> independent review sequence is explicit.
2. **Given** a security reporter, **When** they read `SECURITY.md`, **Then** supported scope and a private reporting path are stated without inventing an unsupported response SLA.

### User Story 4 - Reproduce a Release Candidate Without Publishing It (Priority: P2)

A maintainer can run one explicit release-candidate workflow against an exact commit and obtain deterministic build/test/soak evidence and distributable candidate artifacts/checksums without creating a Git tag, GitHub Release, changing repository visibility, or publishing a crate.

**Why this priority**: Publication should be a small human-controlled final action, not the first time release mechanics are exercised.

**Independent Test**: Run the release-candidate workflow on an exact commit and verify quality, SC-001 soak, platform build targets, artifact identity, and checksums while repository/tag/release state remains unchanged.

**Acceptance Scenarios**:

1. **Given** an exact release-candidate commit, **When** the dry-run workflow executes, **Then** deterministic quality and the required pre-release soak pass before candidate artifacts are accepted.
2. **Given** generated artifacts, **When** they are inspected, **Then** filenames identify platform/version, checksums are emitted, and no unsupported platform is represented as supported.
3. **Given** the dry run, **When** GitHub repository state is inspected, **Then** no release/tag/visibility/package publication mutation occurred.

### User Story 5 - Publish Only After an Explicit Human Decision (Priority: P3)

After all release-readiness evidence is reviewed, the founder may separately authorize the public transition and `v0.1.0` publication.

**Why this priority**: Publication is externally visible and difficult to retract cleanly; it must remain distinct from implementation acceptance.

**Independent Test**: Before explicit publication authorization, verify the repository is still private and no `v0.1.0` tag or GitHub Release exists.

**Acceptance Scenarios**:

1. **Given** all readiness tasks except publication are complete, **When** no separate publication authorization exists, **Then** Winds remains private and untagged/unreleased.
2. **Given** an explicit founder publication decision in a later gate, **When** publication is performed, **Then** it must target the exact reviewed release commit and must not silently add product scope.

### Edge Cases

- GitHub detects no license or a different license than package metadata.
- A donor record says “reference only” but copied runtime code is later discovered.
- Release metadata says `0.1.0` while artifacts or docs still say `0.0.0`/pre-alpha inconsistently.
- The release candidate builds on Linux but not the declared macOS target.
- A release workflow has write permissions or implicitly creates a tag/release.
- A security policy implies sandboxing or a response SLA that Winds does not provide.
- A README installation path depends on unpublished crates.io distribution.
- Checksums or artifact names do not bind clearly to version/platform.
- Public-transition documentation accidentally exposes secrets, internal tokens, private URLs, or personal/local paths.

## Requirements

### Functional Requirements

- **FR-001**: Release-readiness work MUST NOT change repository visibility, create/push a release tag, create a GitHub Release, publish a crate/package, or otherwise perform the public release. Those actions require a separate explicit founder publication authorization after this feature's evidence is complete.
- **FR-002**: Winds MUST complete a repository licensing/provenance audit before public publication. Any copied/adapted donor runtime code MUST have exact source path, commit, license, modification, and update-strategy provenance; unresolved/incompatible provenance blocks publication.
- **FR-003**: The intended source license for Winds 0.1 MUST be `MIT OR Apache-2.0`, represented by standard license texts and consistent package metadata, unless an explicit founder decision amends this requirement before the licensing task is accepted.
- **FR-004**: `Cargo.toml` release-candidate metadata MUST identify version `0.1.0`, repository, README, description, and `MIT OR Apache-2.0` license consistently. `publish = false` MUST remain in 0.1 unless crate publication is separately authorized.
- **FR-005**: Public-facing documentation MUST describe Winds as an independent verification runtime and MUST preserve the proven 0.1 trust/safety boundary: no OS/network/secret sandbox claim, no automatic winner selection, and no product behavior that merges/rebases/cherry-picks/pushes/opens PRs.
- **FR-006**: The repository MUST provide discoverable contributor guidance aligned with the constitution and `AGENTS.md`, including the Spec Kit sequence and mandatory deterministic/correctness/Ponytail/independent-review gates.
- **FR-007**: The repository MUST provide a security policy describing supported 0.1 security scope and a private vulnerability-reporting path. It MUST NOT promise an unproven response SLA or unsupported isolation properties.
- **FR-008**: Release-facing notes/changelog MUST distinguish proven 0.1 behavior from explicitly deferred scope and MUST bind release claims to reproducible evidence rather than reviewer/agent prose alone.
- **FR-009**: A release-candidate workflow MUST be explicit/manual or narrowly release-scoped, use read-only repository permissions for the dry run, check out an exact candidate commit, use the pinned Rust toolchain and committed lockfile, and MUST NOT mutate tags/releases/visibility/packages.
- **FR-010**: The release-candidate gate MUST rerun deterministic quality and the SC-001 100-cycle soak on the exact candidate commit before artifact acceptance.
- **FR-011**: Candidate distribution artifacts MUST cover only currently supported release environments that can be proven in CI. The minimum intended targets are Linux x86-64 (also usable inside supported WSL2 Linux-domain deployments) and macOS arm64; unsupported native Windows artifacts MUST NOT be implied.
- **FR-012**: Candidate artifacts MUST have deterministic, version/platform-identifying names and SHA-256 checksums. Signing/attestation infrastructure is not required for 0.1 and MUST NOT be added solely for speculative future needs.
- **FR-013**: Public-repository hygiene MUST be checked for accidental secrets, private URLs/tokens, personal/local filesystem paths, stale internal-only instructions, and misleading release/version language before publication.
- **FR-014**: The final publication step MUST remain an explicit human-controlled gate after deterministic CI, release-candidate dry run, correctness/safety review, Ponytail review, and at least one independent reviewer pass.
- **FR-015**: This feature MUST NOT add a daemon, plugin system, public runtime protocol, agent adapter, UI, sandbox framework, generic release framework, or other product behavior. Release tooling should remain repository-local and minimal.

### Key Artifacts

- **License set**: standard MIT and Apache-2.0 texts plus matching package metadata.
- **Provenance/license audit**: release-facing record of donor/dependency reuse and unresolved blockers.
- **Public project docs**: README, CONTRIBUTING, SECURITY, and release notes/changelog as justified by tasks.
- **Release-candidate workflow**: non-publishing exact-head build/test/soak/artifact evidence.
- **Release candidate artifacts**: platform/version-named binaries plus SHA-256 checksum manifest.
- **Publication gate**: explicit founder decision separate from implementation merge.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A fresh supported-environment checkout can build/test Winds using only documented commands, pinned Rust `1.97.1`, committed `Cargo.lock`, and `--locked` dependency resolution.
- **SC-002**: Licensing/provenance review has zero unresolved copied-code or incompatible-license blockers before public publication.
- **SC-003**: The exact release-candidate commit passes quality on Ubuntu/macOS and the SC-001 100-cycle verification soak before artifacts are accepted.
- **SC-004**: Release-candidate dry run produces SHA-256-identified artifacts only for proven supported targets and performs zero tag/release/visibility/package mutations.
- **SC-005**: Public-facing docs contain zero known claims of OS/network/secret sandboxing, automatic winner selection, automatic downstream Git integration, or native Windows support for 0.1.
- **SC-006**: Before a separate founder publication authorization, repository visibility remains private and `v0.1.0` tag/GitHub Release remain absent.

## Assumptions

- Winds 0.1 remains a GitHub-distributed, unpublished Rust package (`publish = false`) unless a later explicit decision adds crates.io publication.
- The walking-skeleton behavior already merged on `main` is the complete 0.1 product scope; this feature prepares that behavior for external consumption rather than adding features.
- Existing dependency/process-reference licensing is expected to permit a permissive Winds license because no copied donor runtime code is currently recorded; the audit task must verify rather than merely assume this.
- GitHub-hosted runner/target labels may change; the implementation task must use then-current official runner/target documentation rather than freezing an unverified label in this specification.
- Public publication may require repository-setting changes outside code review; those remain part of the separately authorized founder publication gate.
