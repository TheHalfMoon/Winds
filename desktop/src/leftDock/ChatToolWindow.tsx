import { useEffect, useMemo, useState } from "react";
import { RuntimeMark } from "../components/RuntimeMark";
import { sessionForSlot } from "../dualSession/model";
import type { DualSessionSelection } from "../dualSession/DualSessionWorkspace";
import { composerAvailability, eventSourceLabel, eventTrust } from "../sessionSurface/model";
import { leftDockBridge } from "./bridge";
import type { BridgeProject, BridgeSnapshot } from "./types";

const emptySnapshot: BridgeSnapshot = { projects: [] };

function resolveTarget(snapshot: BridgeSnapshot, selection: DualSessionSelection | null) {
  if (selection) {
    const project = snapshot.projects.find((candidate) => candidate.project.canonicalWorkspaceId === selection.workspaceId);
    const session = project?.sessions.find((candidate) => candidate.canonicalSessionId === selection.sessionId && !candidate.archived);
    if (project && session) return { project, sessionId: session.canonicalSessionId };
  }
  for (const project of snapshot.projects) {
    const session = project.sessions.find((candidate) => !candidate.archived);
    if (session) return { project, sessionId: session.canonicalSessionId };
  }
  return null;
}

function sessionOptions(snapshot: BridgeSnapshot) {
  return snapshot.projects.flatMap((project) => project.sessions
    .filter((session) => !session.archived)
    .map((session) => ({
      key: `${project.project.canonicalWorkspaceId}\u0000${session.canonicalSessionId}`,
      workspaceId: project.project.canonicalWorkspaceId,
      sessionId: session.canonicalSessionId,
      label: `${session.displayName} · ${project.project.displayName}`,
    })));
}

export function ChatToolWindow({
  selection,
  onSelectSession,
}: {
  readonly selection: DualSessionSelection | null;
  readonly onSelectSession: (workspaceId: string, sessionId: string) => void;
}) {
  const bridge = useMemo(() => leftDockBridge(), []);
  const [snapshot, setSnapshot] = useState<BridgeSnapshot>(emptySnapshot);
  const [status, setStatus] = useState("Loading exact Session identity…");

  useEffect(() => {
    let cancelled = false;
    bridge.snapshot().then((next) => {
      if (cancelled) return;
      setSnapshot(next);
      setStatus(bridge.source === "canonical" ? "Canonical Session projection" : "Fixture projection · no dispatch authority");
    }).catch((error: unknown) => {
      if (!cancelled) setStatus(`Chat projection unavailable · ${error instanceof Error ? error.message : String(error)}`);
    });
    return () => { cancelled = true; };
  }, [bridge, selection?.workspaceId, selection?.sessionId]);

  const target = useMemo(() => resolveTarget(snapshot, selection), [selection, snapshot]);
  const options = useMemo(() => sessionOptions(snapshot), [snapshot]);
  const surface = target ? sessionForSlot(target.project as BridgeProject, target.sessionId, 0) : null;
  const availability = surface ? composerAvailability(surface) : null;
  const selectedKey = target ? `${target.project.project.canonicalWorkspaceId}\u0000${target.sessionId}` : "";

  return (
    <aside className="chat-tool-window" aria-label="Chat tool window" data-source={bridge.source}>
      <header className="chat-tool-header">
        <div><p className="section-kicker">Left tool window</p><h2>Chat</h2></div>
        <span className="projection-source">{bridge.source === "canonical" ? "Exact target" : "Fixture"}</span>
      </header>
      {surface ? (
        <>
          <div className="chat-session-selector">
            <label htmlFor="chat-session-target">Session</label>
            <select id="chat-session-target" value={selectedKey} onChange={(event) => {
              const option = options.find((candidate) => candidate.key === event.currentTarget.value);
              if (option) onSelectSession(option.workspaceId, option.sessionId);
            }}>
              {options.map((option) => <option key={option.key} value={option.key}>{option.label}</option>)}
            </select>
          </div>
          <div className="chat-session-identity">
            <RuntimeMark runtime={surface.runtime.family} label={`${surface.runtime.label}. ${surface.runtime.proof}`} compact />
            <div><strong>{surface.displayName}</strong><span>{surface.canonicalSessionId}</span></div>
          </div>
          <div className="chat-stream" aria-label={`${surface.displayName} conversation and work stream`}>
            <p className="chat-stream-boundary">Exact Session projection · presentation does not grant runtime, dispatch, or verification authority.</p>
            {surface.events.length === 0 ? <p className="chat-stream-empty">No projected work events for this Session.</p> : surface.events.map((event) => (
              <article className="chat-event" key={event.id} data-kind={event.kind} data-trust={eventTrust(event)}>
                <header><strong>{event.title}</strong><span>{eventSourceLabel(event)}</span></header>
                <p>{event.body}</p>
                {event.meta && <small>{event.meta}</small>}
              </article>
            ))}
          </div>
          <form className="chat-composer" aria-label={`${surface.displayName} Chat composer`} onSubmit={(event) => event.preventDefault()}>
            <textarea rows={3} aria-label={`Prompt for exact Session ${surface.canonicalSessionId}`} placeholder={availability?.enabled ? "Type for this exact Session…" : "Direct runtime input unavailable"} disabled={!availability?.enabled} />
            <div className="chat-composer-meta"><span>Target · {surface.canonicalSessionId}</span><button type="submit" className="primary-action" disabled={!availability?.enabled}>Send</button></div>
            <p role="status">{availability?.label ?? "Composer unavailable"}</p>
          </form>
        </>
      ) : (
        <div className="chat-tool-empty" data-state="empty"><strong>No active Session</strong><span>Select or create an active canonical Session in Projects.</span></div>
      )}
      <footer className="chat-tool-status" role="status" aria-live="polite">{status}</footer>
    </aside>
  );
}
