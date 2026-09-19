# Spec 011 T152 persistent terminal ownership provenance

## Scope

T152 moves the already-qualified `TerminalSession` PTY/ConPTY ownership primitive under the persistent owner for bounded shell runtimes. It does not add a second terminal backend, provider execution, remote transport, plugin behavior, service-manager behavior, or PID-based recovery.

## Ownership boundary

- The persistent owner owns the exact `TerminalSession`, its child handle, PTY/ConPTY master, writer, and one output consumer.
- A presentation attachment contains only the exact runtime namespace and owner generation. Dropping every attachment does not drop the owner-held terminal primitive.
- Reattach succeeds only when both the runtime namespace and expected owner generation match the live owner-held registry entry.
- Persisted `LIVE_OWNED` metadata remains historical metadata only. It cannot reconstruct a process after owner loss.
- Runtime aliases, terminal text, process titles, and PIDs are never ownership authority.

## Output-drain boundary

The accepted terminal backend requires its output reader to remain actively drained while the child is live. T152 therefore runs one owner-held output pump per live runtime.

The T152 pump is deliberately bounded and non-replay:

- read chunk: 4 KiB;
- transient queue: 64 chunks;
- maximum queued payload: 256 KiB;
- a full transient queue is drained without creating an unbounded transcript;
- no durable terminal output is introduced;
- no T152 claim promises output replay after detach.

T153 remains solely responsible for bounded replay, explicit history-gap/truncation semantics, observer delivery, aggregate replay budgets, and slow-client protection.

## Lifecycle truth

- Start persists `LIVE_OWNED / RUNNING / AVAILABLE / RETAINED_LIVE_PROCESS` only after the owner holds the exact terminal primitive.
- Natural process exit while detached is observed through the retained child handle and is persisted as `UNOWNED / EXITED` with `PROCESS_STATE_OBSERVED`.
- Explicit close/terminate uses only the retained `TerminalSession` ownership primitive and persists `RUNTIME_STOPPED` after proven child exit.
- Owner idle-exit suppression follows the count of live owner-held runtimes, not persisted rows or presentation clients.
- Owner generation mismatch fails closed; no PID reconstruction path exists.

## Platform continuity

T152 reuses the existing portable-pty 0.9.0 platform backend and preserves its already-qualified semantics:

- POSIX PTY input/output/resize/current-size/exit/terminate/close;
- ownership-scoped POSIX interrupt as already accepted;
- native-Windows ConPTY input/output/resize/exit/terminate/close;
- native-Windows interrupt remains explicitly unsupported and fail-closed.

No new terminal dependency or platform backend is introduced.

## Acceptance evidence

The focused T152 fixtures prove:

- a long-running owner-held shell survives complete presentation detachment;
- exact runtime namespace, owner generation, and terminal-session identity survive reattach;
- resize remains delegated to the existing terminal primitive;
- process exit while fully detached is observed and persisted;
- stale owner generation cannot redirect a live runtime;
- terminate/reap acts only through the retained owner-held terminal primitive;
- the T152 layer contains no PID reconstruction, provider/model launch, or second PTY backend.

Local exact-head qualification is recorded separately on the pull request. Official exact-head GitHub CI, including the native-Windows terminal gate, remains required before acceptance or merge.

## Review policy

Alibaba OpenCodeReview delegation mode is the independent review/accounting mechanism for this slice. CodeRabbit, Qodo, and Cubic are not qualification evidence.
