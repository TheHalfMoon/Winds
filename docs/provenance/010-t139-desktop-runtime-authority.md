# T139 — Desktop Direct Runtime Launch Authority

Status: `CANDIDATE_UNAUTHORIZED`

Canonical base at candidate creation:

```text
BASE=d057db587f8c638b2be48d80dcdceae843f4cbe4
T138=CLOSED_CANONICAL
T139=AUTHORIZED_TO_START
```

## Decision

```text
DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED
DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

T139 does not launch Codex or Claude, send a prompt, read or persist credentials, alter provider configuration, install a runtime, select a model, or create a generic runtime dispatcher.

## Canonical evidence

The accepted Spec 006 closeout keeps the physical live-runtime lanes deferred: `T079_LIVE_PASS=NO`, `T080_LIVE_PASS=NO`, `T082_WORKER_LIVE_PASS=NO`, `REAL_CLAUDE_EXECUTION=NO`, and `REAL_CODEX_WORKER_EXECUTION=NO`.

Spec 010 repeats those nonclaims and requires requested runtime identity to remain distinct from Winds-observed identity. Its T139 authority gate explicitly forbids direct live Codex/Claude execution unless a separately accepted amendment authorizes it.

The merged desktop surface projects requested/observed runtime identity and canonical Session/workflow truth, but exposes no bounded Tauri command that grants desktop Codex or Claude launch/input authority. Existing Spec 006 runtime-specific code does not become desktop authority merely because it exists in the repository.

Terminal self-report, process titles, requested runtime labels, fixture events, and historical one-shot live attempts are insufficient to establish a current desktop launch seam.

## Product disposition

Canonical bridge-derived Sessions keep truthful runtime identity treatments, including `Codex requested`, `Claude requested`, observed, stale, unavailable, mismatch, and conflicting states.

Their product composer is disabled and explicitly presents `Direct launch unavailable`. No requested or observed runtime identity enables live desktop input. Fixture-only prompt recording remains limited to deterministic fixture surfaces used by tests and does not create a product launch path.

No launch button, runtime-start command, provider fallback, credential prompt, hidden terminal injection, or generic shell bridge is added by T139.

## Why no amendment is proposed here

The canonical T139 acceptance contract permits truthful unavailability when current authority is insufficient. The constitution's simplicity gate prefers not building an authority-sensitive execution surface without direct current evidence. A separate amendment would only be justified by a concrete, independently qualified live-runtime need and exact authority evidence; T139 does not manufacture that evidence.

## Acceptance disposition

- Exact Codex desktop launch authority: `UNAUTHORIZED`.
- Exact Claude desktop launch authority: `UNAUTHORIZED`.
- Historical live-runtime nonclaims: preserved.
- Requested versus observed runtime truth: preserved.
- Product UI: first-class unavailable path, not fake-connected.
- Credentials/configuration/provider/model state: untouched.
- New execution authority: none.

T139 therefore closes by proving and presenting the unavailable path rather than by claiming unproven runtime execution.
