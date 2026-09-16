import { useState, type FormEvent, type KeyboardEvent } from "react";
import { TerminalSurface } from "../terminal/TerminalSurface";
import { RuntimeMark } from "../components/RuntimeMark";
import {
  composerAvailability,
  createFixtureSubmission,
  eventCanClaimTrustedState,
  eventSourceLabel,
  eventTrust,
  shouldSubmitComposerShortcut,
} from "./model";
import type { SessionSurfaceFixture, SessionWorkEvent } from "./types";
import "./sessionSurface.css";

function WorkEvent({
  event,
  onDockIntent,
}: {
  readonly event: SessionWorkEvent;
  readonly onDockIntent: (event: SessionWorkEvent) => void;
}) {
  const trust = eventTrust(event);
  return (
    <article className="stream-event session-work-event" data-kind={event.kind} data-trust={trust}>
      <div className="stream-rail" aria-hidden="true" />
      <div className="stream-event-body">
        <header className="stream-event-header">
          <span className="stream-label">{event.title}</span>
          <span className="event-source">
            {eventSourceLabel(event)} · {eventCanClaimTrustedState(event) ? "trusted source" : "not trusted evidence"}
          </span>
        </header>
        {event.kind === "command_result" ? (
          <pre className="command-block"><code>{event.body}</code></pre>
        ) : (
          <p>{event.body}</p>
        )}
        {event.meta && <p className="event-meta">{event.meta}</p>}
        {event.detail && (
          <details className="work-event-details">
            <summary>Tool detail</summary>
            <dl>
              <div><dt>Target</dt><dd>{event.detail.target}</dd></div>
              <div><dt>Status</dt><dd>{event.detail.status}</dd></div>
              <div><dt>Source</dt><dd>{event.detail.source}</dd></div>
              {event.detail.failure && <div><dt>Failure</dt><dd>{event.detail.failure}</dd></div>}
            </dl>
          </details>
        )}
        {event.dockIntent && (
          <button type="button" className="text-button future-dock-action" onClick={() => onDockIntent(event)}>
            Open {event.dockIntent === "changes" ? "Changes" : "Files"} · T136 binding
          </button>
        )}
      </div>
    </article>
  );
}

export function SessionSurface({
  session,
  focused = false,
  onDockIntent,
  mode = "conversation",
}: {
  readonly session: SessionSurfaceFixture;
  readonly focused?: boolean;
  readonly onDockIntent?: (surface: "files" | "changes", workspaceId: string, sessionId: string) => void;
  readonly mode?: "conversation" | "workbench";
}) {
  const [events, setEvents] = useState<readonly SessionWorkEvent[]>(session.events);
  const [surface, setSurface] = useState<"work_stream" | "terminal">(mode === "workbench" ? "terminal" : "work_stream");
  const [draft, setDraft] = useState("");
  const [sequence, setSequence] = useState(1);
  const availability = composerAvailability(session);
  const [status, setStatus] = useState(availability.label);

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const submission = createFixtureSubmission(session, draft, sequence);
    if (!submission) {
      setStatus(availability.enabled ? "Enter a prompt before recording the fixture event" : availability.label);
      return;
    }
    setEvents((current) => [...current, submission.event]);
    setDraft("");
    setSequence((current) => current + 1);
    setStatus(`Fixture prompt recorded for ${submission.targetSessionId} · not dispatched`);
  };

  const handleComposerKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (!shouldSubmitComposerShortcut({
      isComposing: event.nativeEvent.isComposing,
      key: event.key,
      metaKey: event.metaKey,
      ctrlKey: event.ctrlKey,
    })) return;
    event.preventDefault();
    event.currentTarget.form?.requestSubmit();
  };

  const handleDockIntent = (event: SessionWorkEvent) => {
    if (!event.dockIntent) return;
    onDockIntent?.(event.dockIntent, session.canonicalWorkspaceId, session.canonicalSessionId);
    setStatus(
      `Opened ${event.dockIntent === "changes" ? "Changes" : "Files"} for exact Session · ${session.canonicalSessionId}`,
    );
  };

  return (
    <section
      className="session-pane session-surface"
      data-primary={focused ? "true" : "false"}
      data-runtime-proof={session.runtime.proofState}
      data-composer-mode={session.composerMode}
      aria-label={`${session.displayName} ${mode === "workbench" ? "Workbench" : "Session work surface"}. ${session.runtime.label}. ${session.runtime.proof}. ${session.lifecycle}.`}
    >
      <header className="session-header">
        <div className="session-identity">
          <RuntimeMark runtime={session.runtime.family} label={session.runtime.label} />
          <div>
            <h2>{session.displayName}</h2>
            <p>{session.canonicalSessionId} · {session.worktreeContext}</p>
          </div>
        </div>
        <div className="session-header-actions">
          <span className="status-chip">{session.lifecycle}</span>
          {session.composerMode === "unavailable" && (
            <span className="status-chip" data-tone="muted">Direct launch unavailable</span>
          )}
        </div>
      </header>

      <div className="stream-toolbar" aria-label="Session stream controls">
        <div className="segmented-control" role="group" aria-label="Session surface view">
          <button
            type="button"
            aria-pressed={surface === "work_stream"}
            data-active={surface === "work_stream" ? "true" : "false"}
            onClick={() => setSurface("work_stream")}
            onKeyDown={(event) => {
              if (event.key === "ArrowRight" || event.key === "ArrowDown" || event.key === "End") {
                event.preventDefault();
                setSurface("terminal");
                event.currentTarget.nextElementSibling instanceof HTMLElement && event.currentTarget.nextElementSibling.focus();
              }
            }}
          >
            {mode === "workbench" ? "Activity" : "Work Stream"}
          </button>
          <button
            type="button"
            aria-pressed={surface === "terminal"}
            data-active={surface === "terminal" ? "true" : "false"}
            onClick={() => setSurface("terminal")}
            onKeyDown={(event) => {
              if (event.key === "ArrowLeft" || event.key === "ArrowUp" || event.key === "Home") {
                event.preventDefault();
                setSurface("work_stream");
                event.currentTarget.previousElementSibling instanceof HTMLElement && event.currentTarget.previousElementSibling.focus();
              }
            }}
          >
            Terminal
          </button>
        </div>
        <span className="fixture-label">{session.runtime.proof}</span>
      </div>

      <div className="work-stream" hidden={surface !== "work_stream"} tabIndex={surface === "work_stream" ? 0 : -1} aria-label={`${session.displayName} Work Stream`}>
        <div className="stream-day"><span>User-visible work events</span></div>
        {session.surfaceState === "loading" ? (
          <div className="work-stream-state" data-state="loading">Loading fixture work events…</div>
        ) : session.surfaceState === "error" ? (
          <div className="work-stream-state" data-state="error">Fixture surface error · runtime identity remains explicit</div>
        ) : events.length === 0 ? (
          <div className="empty-state" data-state="empty">
            <strong>No work events yet</strong>
            <span>This fixture cannot imply runtime launch, input, or verification evidence.</span>
          </div>
        ) : (
          events.map((event) => <WorkEvent event={event} onDockIntent={handleDockIntent} key={event.id} />)
        )}
      </div>

      <TerminalSurface canonicalSessionId={session.canonicalSessionId} visible={surface === "terminal"} />

      <form className="composer" hidden={surface !== "work_stream" || mode === "workbench"} aria-label={`${session.displayName} composer`} onSubmit={handleSubmit}>
        <div className="composer-context">
          <span className="context-pill">Target · {session.displayName}</span>
          <span className="context-pill">Session · {session.canonicalSessionId}</span>
        </div>
        <textarea
          aria-label={`Prompt fixture for ${session.displayName}`}
          placeholder={availability.enabled ? "Record a local fixture prompt…" : "Direct runtime launch unavailable"}
          rows={3}
          value={draft}
          disabled={!availability.enabled}
          onChange={(event) => setDraft(event.currentTarget.value)}
          onKeyDown={handleComposerKeyDown}
        />
        <div className="composer-actions">
          <div className="composer-tools">
            <button type="button" className="icon-button" aria-label="Attach context unavailable until authorized" disabled>＋</button>
            <span className="composer-hint">User-visible output only</span>
          </div>
          <div className="composer-submit">
            <span className="composer-hint">⌘↵ record fixture</span>
            <button type="submit" className="primary-action" disabled={!availability.enabled || draft.trim().length === 0}>
              Record fixture prompt
            </button>
          </div>
        </div>
        <p className="composer-status" role="status" aria-live="polite">{status}</p>
      </form>
    </section>
  );
}
