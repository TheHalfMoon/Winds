import { lazy, Suspense, useEffect, useMemo, useState } from "react";
import { terminalBridge } from "./bridge";
import "./terminal.css";

const TerminalRuntimeSurface = lazy(async () => {
  const module = await import("./TerminalRuntimeSurface");
  return { default: module.TerminalRuntimeSurface };
});

const activatedSessions = new Set<string>();

function IdleTerminalShell({
  canonicalSessionId,
  visible,
  onStart,
  loading = false,
  canonical = false,
  statusMessage,
}: {
  readonly canonicalSessionId: string;
  readonly visible: boolean;
  readonly onStart?: () => void;
  readonly loading?: boolean;
  readonly canonical?: boolean;
  readonly statusMessage?: string | null;
}) {
  const canStart = canonical && !loading;
  return (
    <section className="terminal-surface" hidden={!visible} aria-label={`Terminal for canonical Session ${canonicalSessionId}`}>
      <header className="terminal-trust-bar">
        <div>
          <strong>Terminal output · untrusted</strong>
          <span>Not verification evidence · OSC 52 blocked · no automatic links or file actions</span>
        </div>
        <span className="terminal-lifecycle" data-lifecycle="idle">No live terminal</span>
      </header>
      <div className="terminal-tools">
        <button type="button" className="text-button" disabled={!canStart} onClick={onStart}>Start terminal</button>
        <button type="button" className="text-button" disabled>Interrupt</button>
        <button type="button" className="text-button" disabled>Terminate</button>
        <button type="button" className="text-button" disabled>Close terminal</button>
        <label className="terminal-search">
          <span>Literal search</span>
          <input disabled placeholder="Start terminal to load renderer" />
        </label>
        <button type="button" className="text-button" disabled>Find next</button>
      </div>
      <div className="terminal-host" aria-label="Untrusted interactive terminal output" />
      <p className="terminal-status" role="status" aria-live="polite">
        {loading
          ? "Preparing terminal renderer…"
          : statusMessage ?? (canonical ? "Terminal is idle · Rust owns launch and lifecycle" : "Browser fixture · canonical terminal host unavailable")}
      </p>
    </section>
  );
}

export function TerminalSurface({
  canonicalSessionId,
  visible,
}: {
  readonly canonicalSessionId: string;
  readonly visible: boolean;
}) {
  const bridge = useMemo(() => terminalBridge(), []);
  const wasActivated = activatedSessions.has(canonicalSessionId);
  const [activated, setActivated] = useState(wasActivated);
  const [autoStart, setAutoStart] = useState(false);
  const [probeMessage, setProbeMessage] = useState<string | null>(null);

  useEffect(() => {
    if (activated || bridge.source !== "canonical") return;
    let cancelled = false;
    void bridge.status(canonicalSessionId)
      .then((status) => {
        if (cancelled || !status) return;
        activatedSessions.add(canonicalSessionId);
        setAutoStart(false);
        setActivated(true);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setProbeMessage(`Terminal status failed · ${error instanceof Error ? error.message : String(error)}`);
        }
      });
    return () => { cancelled = true; };
  }, [activated, bridge, canonicalSessionId]);

  if (!activated) {
    return (
      <IdleTerminalShell
        canonicalSessionId={canonicalSessionId}
        visible={visible}
        canonical={bridge.source === "canonical"}
        statusMessage={probeMessage}
        onStart={() => {
          activatedSessions.add(canonicalSessionId);
          setAutoStart(true);
          setActivated(true);
        }}
      />
    );
  }

  return (
    <Suspense fallback={<IdleTerminalShell canonicalSessionId={canonicalSessionId} visible={visible} canonical={bridge.source === "canonical"} loading />}>
      <TerminalRuntimeSurface canonicalSessionId={canonicalSessionId} visible={visible} autoStart={autoStart} />
    </Suspense>
  );
}
