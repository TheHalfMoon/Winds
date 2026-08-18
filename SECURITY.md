# Security Policy

## Supported scope

The current public release line is `0.1.x`, beginning with `v0.1.0`. Current `main` also contains accepted-but-unreleased Spec 003 workspace-terminal implementation slices. Security reports are welcome for vulnerabilities that can cause Winds itself to violate the documented invariants of either surface.

For the authoritative verification path, relevant issues include:

- mutation of the primary checkout during verification/promotion;
- unsafe ownership/deletion of Winds-managed candidate paths;
- evidence corruption, substitution, or incorrect authority/identity binding;
- promotion of an unverified or stale candidate;
- command/process timeout or cleanup behavior that escapes the documented verification boundary;
- repository-state or Git-context handling that causes Winds to verify/promote a different snapshot than reported;
- vulnerabilities in Winds' packaged dependencies that materially affect the supported runtime.

For the accepted Spec 003 workspace-terminal surface on current `main`, relevant issues also include:

- starting a terminal or explicit command against a different canonical workspace, execution domain, executable/profile, or cwd than Winds reports;
- PTY/ConPTY lifecycle bugs that make Winds falsely report a directly owned session as exited, interrupted, closed, or still owned;
- restart/recovery behavior that trusts a stale PID, blindly signals an unproven process, or fabricates continuing process ownership;
- native-Windows/WSL mapping or Git-identity errors that silently claim workspace equivalence when the accepted checks do not prove it;
- SQLite partial-transition behavior that leaves falsely-live/falsely-final execution records or incorrectly elevates source/authority;
- command/history/transcript persistence that violates documented bounds, disable controls, sanitization rules, or the no-full-environment-persistence boundary;
- shell or process output being misclassified as authoritative Winds verification evidence;
- workspace execution/history becoming candidate eligibility or promotion authority without going through the accepted verification path.

## Explicit non-sandbox boundary

Winds verification worktrees are **not security sandboxes**. Workspace terminals and explicit `winds run` commands likewise execute with the permissions of the user who launched Winds.

Reports that only demonstrate behavior explicitly outside these security claims—such as network access by a required check or workspace command, access to secrets already available to the launching user, or arbitrary filesystem writes permitted to that process—may still inform future hardening, but they are not evidence that Winds broke an OS/network/secret isolation promise because no such promise exists.

PTY/ConPTY ownership is lifecycle ownership for resources Winds can prove it owns. It is not proof that Winds confines every descendant process, filesystem effect, network connection, or credential reachable by the launched process.

See [`specs/003-workspace-execution-spine/terminal-trust-boundary.md`](specs/003-workspace-execution-spine/terminal-trust-boundary.md) for the detailed workspace-terminal trust boundary.

## Platform boundary

The platform claim differs by surface:

- authoritative `winds verify` / `winds promote` required-check execution is supported on Linux and macOS, and on WSL2 when Winds/Git/repository/check execution all live inside the Linux domain;
- native Windows authoritative required-check execution remains intentionally unsupported and fails closed before verification/promotion mutation;
- Spec 003 workspace-terminal behavior has accepted Linux/macOS PTY, native-Windows ConPTY touched-surface, and real Windows+Ubuntu WSL2 integration evidence.

A native-Windows workspace-terminal bug is therefore in scope even though native-Windows verification is not currently a supported claim.

## Supported versions

The current public release line is `0.1.x`. Winds `v0.1.0` is the first supported public release.

Current `main` is a development branch rather than a released `0.2.x` line. Reports against unreleased workspace-terminal behavior are welcome, but they should identify the exact affected commit so the report is bound to a reproducible snapshot.

Security fixes are expected to target the current supported release line and/or current development head as appropriate to the affected surface. This policy does not promise a fixed response or remediation SLA.

## Reporting a vulnerability

**Do not disclose vulnerability details in a public issue, discussion, or pull request.**

For the public Winds repository, use GitHub's private vulnerability reporting flow:

1. Open the repository's **Security** area.
2. Open **Advisories** / **Report a vulnerability**.
3. Submit the report privately with reproduction details and impact.

If the **Report a vulnerability** action is unexpectedly unavailable, do not post sensitive details. Open only a minimal public issue asking maintainers to establish a private contact channel, without including the vulnerability, proof of concept, affected paths, or exploit details.

Maintainers may use a draft GitHub repository security advisory to coordinate fixes privately.

## What to include

A useful report includes, when available:

- affected Winds release or exact commit;
- operating system, architecture, and Git version;
- whether the issue affects authoritative verification, workspace-terminal execution, or both;
- exact command/repository/workspace state needed to reproduce;
- observed behavior and the documented invariant you believe was violated;
- minimal proof of concept that avoids unnecessary destructive effects;
- whether you believe credentials, source integrity, process ownership, execution-history integrity, evidence integrity, or promotion integrity are at risk;
- for Windows/WSL issues, the Windows version, selected distribution, relevant path/domain mapping, and whether the mismatch was surfaced or silently misreported.

Please avoid accessing data you do not own, persisting beyond what is needed to demonstrate the issue, or publishing exploit details before coordinated disclosure.

## Disclosure

Winds maintainers will evaluate reports based on reproducibility, affected invariants, and user impact. Remediation and disclosure timing depend on the specific issue; this project does not promise an unsupported response-time SLA.
