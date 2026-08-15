# Security Policy

## Supported scope

Winds 0.1 is a bounded verification runtime. Security reports are welcome for vulnerabilities that can cause Winds itself to violate its documented invariants, including issues involving:

- mutation of the primary checkout during verification/promotion;
- unsafe ownership/deletion of Winds-managed paths;
- evidence corruption, substitution, or incorrect authority/identity binding;
- promotion of an unverified or stale candidate;
- command/process timeout or cleanup behavior that escapes the documented boundary;
- repository-state or Git-context handling that causes Winds to verify/promote a different snapshot than reported;
- vulnerabilities in Winds' packaged dependencies that materially affect the supported runtime.

Winds worktrees are **not security sandboxes**. Reports that only demonstrate behavior explicitly outside the 0.1 security claim—such as network access by a required check, access to the launching user's secrets, or arbitrary filesystem writes outside the candidate checkout—may still inform future hardening, but they are not evidence that the current sandbox promise was broken because no such promise exists.

## Supported versions

The first public release line is `0.1.x`. Until a public `0.1.0` release exists, the repository is a release candidate rather than a supported public release.

After release, security fixes are expected to target the current `0.1.x` line. This policy does not promise a fixed response or remediation SLA.

## Reporting a vulnerability

**Do not disclose vulnerability details in a public issue, discussion, or pull request.**

For the public Winds repository, use GitHub's private vulnerability reporting flow:

1. Open the repository's **Security** area.
2. Open **Advisories** / **Report a vulnerability**.
3. Submit the report privately with reproduction details and impact.

GitHub only permits repository owners/administrators to enable private vulnerability reporting after a repository is public. Winds' founder-controlled publication gate therefore requires private vulnerability reporting to be enabled as part of the public transition before the release is announced for external use.

If the **Report a vulnerability** action is unexpectedly unavailable on the public repository, do not post sensitive details. Open only a minimal public issue asking maintainers to establish a private contact channel, without including the vulnerability, proof of concept, affected paths, or exploit details.

Maintainers may use a draft GitHub repository security advisory to coordinate fixes privately.

## What to include

A useful report includes, when available:

- affected Winds version/commit;
- operating system and Git version;
- exact command/repository state needed to reproduce;
- observed behavior and expected invariant;
- minimal proof of concept that avoids unnecessary destructive effects;
- whether you believe credentials, source integrity, evidence integrity, or promotion integrity are at risk.

Please avoid accessing data you do not own, persisting beyond what is needed to demonstrate the issue, or publishing exploit details before coordinated disclosure.

## Disclosure

Winds maintainers will evaluate reports based on reproducibility, affected invariants, and user impact. Remediation and disclosure timing depend on the specific issue; this project does not promise an unsupported response-time SLA.
