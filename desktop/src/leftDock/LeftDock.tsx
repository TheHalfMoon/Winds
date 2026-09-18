import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { RuntimeMark } from "../components/RuntimeMark";
import { leftDockBridge } from "./bridge";
import {
  attentionView,
  createSessionPlan,
  filterProjects,
  nextSessionFocusIndex,
  projectReorderPlan,
  projectRenderedExpanded,
  projectUpdatePlan,
  reconcileSelectedSession,
  resolveSessionSearch,
  runtimeView,
  sessionRenamePlan,
  sessionReorderPlan,
  sessionUpdatePlan,
} from "./model";
import type { BridgeProject, BridgeSessionSummary, BridgeSnapshot, LeftDockBridge } from "./types";

const emptySnapshot: BridgeSnapshot = { projects: [] };

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function sessionFocusKey(sessionId: string, action: string): string {
  return `session:${sessionId}:${action}`;
}

function projectFocusKey(workspaceId: string, action: string): string {
  return `project:${workspaceId}:${action}`;
}

function SessionRow({
  session,
  selected,
  busy,
  writable,
  canMoveUp,
  canMoveDown,
  onSelect,
  onRename,
  onPin,
  onMove,
  onFocusKey,
  onRestoreFocusKey,
}: {
  readonly session: BridgeSessionSummary;
  readonly selected: boolean;
  readonly busy: boolean;
  readonly writable: boolean;
  readonly canMoveUp: boolean;
  readonly canMoveDown: boolean;
  readonly onSelect: () => void;
  readonly onRename: (displayName: string) => Promise<boolean>;
  readonly onPin: () => void;
  readonly onMove: (direction: 1 | -1) => void;
  readonly onFocusKey: (key: string) => void;
  readonly onRestoreFocusKey: (key: string) => void;
}) {
  const runtime = runtimeView(session);
  const attention = attentionView(session.attention);
  const [editing, setEditing] = useState(false);
  const [renameValue, setRenameValue] = useState(session.displayName);
  const mutationDisabled = busy || !writable;

  useEffect(() => {
    if (!editing) setRenameValue(session.displayName);
  }, [editing, session.displayName]);

  const renameTriggerKey = sessionFocusKey(session.canonicalSessionId, "rename");

  function closeRenameEditor() {
    onRestoreFocusKey(renameTriggerKey);
    setEditing(false);
  }

  async function submitRename(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const displayName = renameValue.trim();
    if (!displayName || displayName === session.displayName) {
      closeRenameEditor();
      return;
    }
    if (await onRename(displayName)) closeRenameEditor();
  }

  return (
    <div className="session-row-shell" data-selected={selected ? "true" : "false"} data-archived={session.archived ? "true" : "false"}>
      <button
        type="button"
        className="session-row"
        data-session-id={session.canonicalSessionId}
        data-focus-key={sessionFocusKey(session.canonicalSessionId, "select")}
        data-active={selected ? "true" : "false"}
        data-attention={attention.actionable ? "true" : "false"}
        aria-current={selected ? "page" : undefined}
        aria-label={`${session.displayName}. ${runtime.label}. ${runtime.proof}. ${session.archived ? "Archived Session" : "Active Session"}. ${attention.label}.`}
        onClick={onSelect}
        onFocus={() => onFocusKey(sessionFocusKey(session.canonicalSessionId, "select"))}
      >
        <RuntimeMark runtime={runtime.family} label={`${runtime.label}. ${runtime.proof}`} compact />
        <span className="session-row-copy">
          <span className="session-row-title">{session.displayName}</span>
          <span className="session-row-meta">{runtime.label} · {runtime.proof}</span>
          <span className="session-lifecycle-text">{session.archived ? "Archived Session" : "Active Session"}</span>
          {session.attention !== "none" && <span className="session-attention-text">{attention.label}</span>}
        </span>
      </button>
      <div className="row-actions" aria-label={`Presentation actions for canonical Session ${session.canonicalSessionId}`}>
        <button type="button" className="mini-action" data-focus-key={sessionFocusKey(session.canonicalSessionId, "pin")} disabled={mutationDisabled} onClick={onPin} aria-label={`${session.pinned ? "Unpin" : "Pin"} canonical Session ${session.canonicalSessionId}`}>{session.pinned ? "★" : "☆"}</button>
        <button type="button" className="mini-action" data-focus-key={sessionFocusKey(session.canonicalSessionId, "move-up")} disabled={mutationDisabled || !canMoveUp} onClick={() => onMove(-1)} aria-label={`Move canonical Session ${session.canonicalSessionId} up`}>↑</button>
        <button type="button" className="mini-action" data-focus-key={sessionFocusKey(session.canonicalSessionId, "move-down")} disabled={mutationDisabled || !canMoveDown} onClick={() => onMove(1)} aria-label={`Move canonical Session ${session.canonicalSessionId} down`}>↓</button>
        <button type="button" className="mini-action" data-focus-key={renameTriggerKey} disabled={mutationDisabled} onClick={() => setEditing((value) => !value)} aria-label={`Rename canonical Session ${session.canonicalSessionId}`}>✎</button>
      </div>
      {editing && writable && (
        <form className="inline-session-form" onSubmit={(event) => void submitRename(event)}>
          <label>
            <span>Session display name</span>
            <input data-focus-key={sessionFocusKey(session.canonicalSessionId, "rename-input")} value={renameValue} disabled={busy} onChange={(event) => setRenameValue(event.currentTarget.value)} aria-label={`Rename canonical Session ${session.canonicalSessionId}`} />
          </label>
          <div className="inline-form-actions">
            <button type="submit" className="quiet-action" disabled={busy || !renameValue.trim()}>Save name</button>
            <button type="button" className="quiet-action" disabled={busy} onClick={closeRenameEditor}>Cancel</button>
          </div>
        </form>
      )}
    </div>
  );
}

export function LeftDock({
  onSelectSession,
  requestedSessionId,
}: {
  readonly onSelectSession?: (workspaceId: string, sessionId: string) => void;
  readonly requestedSessionId?: string | null;
}) {
  const bridge = useMemo<LeftDockBridge>(() => leftDockBridge(), []);
  const writable = bridge.source === "canonical";
  const [snapshot, setSnapshot] = useState<BridgeSnapshot>(emptySnapshot);
  const [query, setQuery] = useState("");
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const focusedControlKeyRef = useRef<string | null>(null);
  const [focusRestoreRevision, setFocusRestoreRevision] = useState(0);
  const [status, setStatus] = useState("Loading Projects and Sessions…");
  const [busy, setBusy] = useState(false);
  const [creatingWorkspaceId, setCreatingWorkspaceId] = useState<string | null>(null);
  const [createName, setCreateName] = useState("");
  const [createWorkstreamId, setCreateWorkstreamId] = useState("");
  const dockRef = useRef<HTMLElement>(null);
  const navRef = useRef<HTMLElement>(null);
  const busyRef = useRef(false);

  const rememberFocusKey = useCallback((key: string | null) => {
    focusedControlKeyRef.current = key;
  }, []);

  const restoreFocusKey = useCallback((key: string) => {
    focusedControlKeyRef.current = key;
    setFocusRestoreRevision((revision) => revision + 1);
  }, []);

  const refresh = useCallback(async () => {
    const next = await bridge.snapshot();
    setSnapshot(next);
    setSelectedSessionId((selected) => reconcileSelectedSession(next, selected));
    setStatus(bridge.source === "canonical" ? "Canonical snapshot · refresh to observe external changes" : "Fixture · read-only · no authority");
  }, [bridge]);

  useEffect(() => {
    void refresh().catch((error) => setStatus(`Load failed · ${errorMessage(error)}`));
  }, [refresh]);

  useEffect(() => {
    if (requestedSessionId !== undefined) setSelectedSessionId(requestedSessionId);
  }, [requestedSessionId]);

  useEffect(() => {
    const focusedControlKey = focusedControlKeyRef.current;
    if (!focusedControlKey || busy) return;
    const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    if (active?.dataset.focusKey === focusedControlKey) return;
    const target = Array.from(dockRef.current?.querySelectorAll<HTMLElement>("[data-focus-key]") ?? [])
      .find((element) => !element.closest("[hidden]") && element.dataset.focusKey === focusedControlKey);
    if (target) target.focus();
  }, [busy, focusRestoreRevision, snapshot]);

  const mutate = useCallback(async (operation: () => Promise<unknown>): Promise<boolean> => {
    if (!writable) {
      setStatus("Fixture mode is read-only · no canonical authority");
      return false;
    }
    if (busyRef.current) {
      setStatus("Update already in progress · wait for the canonical snapshot refresh");
      return false;
    }
    busyRef.current = true;
    setBusy(true);
    try {
      await operation();
      await refresh();
      return true;
    } catch (error) {
      const message = errorMessage(error);
      try {
        await refresh();
        setStatus(`Update failed · ${message} · canonical snapshot refreshed`);
      } catch (refreshError) {
        setStatus(`Update failed · ${message} · refresh failed · ${errorMessage(refreshError)}`);
      }
      return false;
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }, [refresh, writable]);

  const visibleProjects = useMemo(() => filterProjects(snapshot, query), [snapshot, query]);

  const updateProject = useCallback((project: BridgeProject, changes: Parameters<typeof projectUpdatePlan>[1]) => {
    void mutate(() => bridge.updateProject([projectUpdatePlan(project.project, changes)]));
  }, [bridge, mutate]);

  const moveProject = useCallback((workspaceId: string, direction: 1 | -1) => {
    const plan = projectReorderPlan(snapshot.projects, workspaceId, direction);
    if (!plan) return;
    void mutate(() => bridge.updateProject(plan));
  }, [bridge, mutate, snapshot.projects]);

  const updateSession = useCallback((session: BridgeSessionSummary, changes: Parameters<typeof sessionUpdatePlan>[1]) => {
    void mutate(() => bridge.updateSession([sessionUpdatePlan(session, changes)]));
  }, [bridge, mutate]);

  const moveSession = useCallback((project: BridgeProject, sessionId: string, direction: 1 | -1) => {
    const plan = sessionReorderPlan(project.sessions, sessionId, direction);
    if (!plan) return;
    void mutate(() => bridge.updateSession(plan));
  }, [bridge, mutate]);

  const beginCreateSession = useCallback((project: BridgeProject) => {
    if (!writable) {
      setStatus("Fixture mode is read-only · no canonical authority");
      return;
    }
    if (project.availableWorkstreams.length === 0) {
      setStatus("Session creation requires a canonical Workstream in this Project");
      return;
    }
    setCreatingWorkspaceId(project.project.canonicalWorkspaceId);
    setCreateName("");
    setCreateWorkstreamId(project.availableWorkstreams.length === 1 ? project.availableWorkstreams[0].workstreamId : "");
  }, [writable]);

  const closeCreateSession = useCallback((workspaceId: string) => {
    restoreFocusKey(projectFocusKey(workspaceId, "create-session"));
    setCreatingWorkspaceId(null);
  }, [restoreFocusKey]);

  const submitCreateSession = useCallback(async (project: BridgeProject) => {
    const displayName = createName.trim();
    if (!displayName || !createWorkstreamId) {
      setStatus("Choose one canonical Workstream and enter a Session name");
      return;
    }
    const success = await mutate(() => bridge.createSession(createSessionPlan(project, createWorkstreamId, displayName)));
    if (success) {
      closeCreateSession(project.project.canonicalWorkspaceId);
      setCreateName("");
      setCreateWorkstreamId("");
    }
  }, [bridge, closeCreateSession, createName, createWorkstreamId, mutate]);

  const renameSession = useCallback((session: BridgeSessionSummary, displayName: string) => (
    mutate(() => bridge.renameSession(sessionRenamePlan(session, displayName)))
  ), [bridge, mutate]);

  const resolveSearch = useCallback(() => {
    const resolution = resolveSessionSearch(snapshot, query);
    if (resolution.kind === "unique") {
      setSelectedSessionId(resolution.sessionId);
      const owner = snapshot.projects.find((project) => project.sessions.some((session) => session.canonicalSessionId === resolution.sessionId));
      if (owner) onSelectSession?.(owner.project.canonicalWorkspaceId, resolution.sessionId);
      setStatus(`Selected canonical Session · ${resolution.sessionId}`);
    } else if (resolution.kind === "ambiguous") {
      setStatus(`Ambiguous Session search · ${resolution.sessionIds.join(", ")}`);
    } else {
      setStatus(query.trim() ? "No exact Session resolution" : "Enter a Session ID or alias to resolve");
    }
  }, [onSelectSession, query, snapshot]);

  const onNavKeyDown = useCallback((event: React.KeyboardEvent<HTMLElement>) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    const rows = Array.from(navRef.current?.querySelectorAll<HTMLButtonElement>("button.session-row") ?? [])
      .filter((row) => !row.closest("[hidden]"));
    if (rows.length === 0) return;
    const activeIndex = rows.findIndex((row) => row === document.activeElement);
    const nextIndex = nextSessionFocusIndex(rows.length, activeIndex, event.key === "ArrowDown" ? 1 : -1);
    if (nextIndex === null) return;
    event.preventDefault();
    rows[nextIndex].focus();
  }, []);

  const searching = query.trim().length > 0;
  const renderProjectGroups = useCallback((projects: readonly BridgeProject[], renderQuery: string) => (
    <>
      {projects.map((project) => {
        const current = project.project;
        const mutationDisabled = busy || !writable;
        const creating = creatingWorkspaceId === current.canonicalWorkspaceId;
        const projectAttention = attentionView(current.attention);
        const renderedExpanded = projectRenderedExpanded(project, renderQuery);
        const upPlan = projectReorderPlan(snapshot.projects, current.canonicalWorkspaceId, -1);
        const downPlan = projectReorderPlan(snapshot.projects, current.canonicalWorkspaceId, 1);
        return (
          <section className="project-group" key={current.canonicalWorkspaceId} data-open={renderedExpanded ? "true" : "false"}>
            <div className="project-row-shell">
              <button type="button" className="project-row" data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "toggle")} aria-expanded={renderedExpanded} aria-label={`${current.displayName} Project. ${current.sessionCount} Sessions. ${current.attentionCount} need attention.`} disabled={mutationDisabled} onClick={() => updateProject(project, { collapsed: !current.collapsed })}>
                <span className="disclosure" aria-hidden="true">{renderedExpanded ? "⌄" : "›"}</span>
                <span className="project-copy"><span className="project-name">{current.displayName}</span><span className="project-meta">{current.canonicalWorkspaceId}</span></span>
                <span className="project-count">{current.sessionCount}</span>
                {!renderedExpanded && current.attentionCount > 0 && <span className="project-attention-summary">{current.attentionCount} need attention · {projectAttention.label}</span>}
              </button>
              <div className="row-actions" aria-label={`Presentation actions for canonical Project ${current.canonicalWorkspaceId}`}>
                <button type="button" className="mini-action" data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "pin")} disabled={mutationDisabled} onClick={() => updateProject(project, { pinned: !current.pinned })} aria-label={`${current.pinned ? "Unpin" : "Pin"} canonical Project ${current.canonicalWorkspaceId}`}>{current.pinned ? "★" : "☆"}</button>
                <button type="button" className="mini-action" data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "move-up")} disabled={mutationDisabled || !upPlan} onClick={() => moveProject(current.canonicalWorkspaceId, -1)} aria-label={`Move canonical Project ${current.canonicalWorkspaceId} up`}>↑</button>
                <button type="button" className="mini-action" data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "move-down")} disabled={mutationDisabled || !downPlan} onClick={() => moveProject(current.canonicalWorkspaceId, 1)} aria-label={`Move canonical Project ${current.canonicalWorkspaceId} down`}>↓</button>
                <button type="button" className="mini-action" data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "create-session")} disabled={mutationDisabled} onClick={() => beginCreateSession(project)} aria-label={`Create Session in canonical Project ${current.canonicalWorkspaceId}`}>＋</button>
              </div>
            </div>
            {creating && writable && (
              <form className="inline-session-form create-session-form" onSubmit={(event) => { event.preventDefault(); void submitCreateSession(project); }}>
                <label><span>Session name</span><input data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "create-name")} value={createName} disabled={busy} onChange={(event) => setCreateName(event.currentTarget.value)} aria-label={`Session name for canonical Project ${current.canonicalWorkspaceId}`} /></label>
                <label><span>Canonical Workstream</span><select data-focus-key={projectFocusKey(current.canonicalWorkspaceId, "create-workstream")} value={createWorkstreamId} disabled={busy} onChange={(event) => setCreateWorkstreamId(event.currentTarget.value)} aria-label={`Canonical Workstream for Project ${current.canonicalWorkspaceId}`}>
                  {project.availableWorkstreams.length > 1 && <option value="" disabled>Select one Workstream explicitly</option>}
                  {project.availableWorkstreams.map((workstream) => <option key={workstream.workstreamId} value={workstream.workstreamId}>{workstream.displayName} · {workstream.workstreamId}</option>)}
                </select></label>
                <div className="inline-form-actions"><button type="submit" className="quiet-action" disabled={busy || !createName.trim() || !createWorkstreamId}>Create Session</button><button type="button" className="quiet-action" disabled={busy} onClick={() => closeCreateSession(current.canonicalWorkspaceId)}>Cancel</button></div>
              </form>
            )}
            {renderedExpanded && <div className="session-list" aria-label={`${current.displayName} Sessions`}>
              {project.sessions.map((session) => <SessionRow key={session.canonicalSessionId} session={session} selected={selectedSessionId === session.canonicalSessionId} busy={busy} writable={writable} canMoveUp={sessionReorderPlan(project.sessions, session.canonicalSessionId, -1) !== null} canMoveDown={sessionReorderPlan(project.sessions, session.canonicalSessionId, 1) !== null} onSelect={() => { if (session.archived) { setStatus(`Archived Session is not an active workspace target · ${session.canonicalSessionId}`); return; } setSelectedSessionId(session.canonicalSessionId); onSelectSession?.(current.canonicalWorkspaceId, session.canonicalSessionId); }} onRename={(displayName) => renameSession(session, displayName)} onPin={() => updateSession(session, { pinned: !session.pinned })} onMove={(direction) => moveSession(project, session.canonicalSessionId, direction)} onFocusKey={rememberFocusKey} onRestoreFocusKey={restoreFocusKey} />)}
              {project.sessions.length === 0 && <p className="session-empty">No Sessions in this Project</p>}
            </div>}
          </section>
        );
      })}
    </>
  ), [
    beginCreateSession, busy, closeCreateSession, createName, createWorkstreamId, creatingWorkspaceId, moveProject,
    moveSession, onSelectSession, rememberFocusKey, renameSession, restoreFocusKey,
    selectedSessionId, snapshot.projects, submitCreateSession, updateProject, updateSession, writable,
  ]);
  const browseProjectGroups = useMemo(
    () => renderProjectGroups(snapshot.projects, ""),
    [renderProjectGroups, snapshot.projects],
  );
  const searchProjectGroups = useMemo(
    () => searching ? renderProjectGroups(visibleProjects, query) : null,
    [query, renderProjectGroups, searching, visibleProjects],
  );

  return (
    <aside ref={dockRef} className="left-dock" aria-label="Projects and Sessions" data-source={bridge.source} onFocusCapture={(event) => {
      const target = (event.target as HTMLElement).closest<HTMLElement>("[data-focus-key]");
      rememberFocusKey(target?.dataset.focusKey ?? null);
    }}>
      <div className="dock-heading-row">
        <div><p className="section-kicker">Workspace</p><h2>Projects</h2></div>
        <span className="projection-source">{bridge.source === "canonical" ? "Canonical snapshot" : "Fixture · read-only"}</span>
      </div>
      <div className="left-dock-search">
        <span aria-hidden="true">⌕</span>
        <input type="search" value={query} onChange={(event) => setQuery(event.currentTarget.value)} onKeyDown={(event) => { if (event.key === "Enter") resolveSearch(); }} aria-label="Search Projects and Sessions" placeholder="Search Projects and Sessions" />
      </div>
      <nav ref={navRef} className="project-list" aria-label="Project navigation" onKeyDown={onNavKeyDown}>
        <div data-project-browse hidden={searching}>
          {browseProjectGroups}
        </div>
        {searching && <div data-project-search-results>
          {searchProjectGroups}
        </div>}
        {(searching ? visibleProjects.length === 0 : snapshot.projects.length === 0) && <p className="session-empty">No matching Projects or Sessions</p>}
      </nav>
      <div className="dock-foot">
        <div className="projection-status" role="status" aria-live="polite">{status}</div>
        <button type="button" className="quiet-action" data-focus-key="dock:refresh" onClick={() => {
          void refresh().catch((error) => setStatus(`Refresh failed · ${errorMessage(error)}`));
        }} disabled={busy}><span aria-hidden="true">↻</span>Refresh canonical snapshot</button>
      </div>
    </aside>
  );
}
