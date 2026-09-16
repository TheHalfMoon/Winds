import { useEffect, useMemo, useRef, useState } from "react";
import { leftDockBridge } from "../leftDock/bridge";
import type { BridgeLayoutPresentation, BridgeProject } from "../leftDock/types";
import { SessionSurface } from "../sessionSurface/SessionSurface";
import {
  closeSlot,
  defaultLayoutForProject,
  enableDual,
  layoutSaveRequest,
  reconcileLayout,
  replaceSlot,
  selectSessionForSlot,
  sessionForSlot,
  slotForSession,
  swapSlots,
  type SessionSlot,
} from "./model";
import "./dualSession.css";

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export interface DualSessionSelection {
  readonly workspaceId: string;
  readonly sessionId: string;
}

export interface DualSessionDockIntent extends DualSessionSelection {
  readonly surface: "files" | "changes";
}

export function DualSessionWorkspace({
  selection,
  onFocusSession,
  onDockIntent,
}: {
  readonly selection: DualSessionSelection | null;
  readonly onFocusSession?: (target: DualSessionSelection) => void;
  readonly onDockIntent?: (intent: DualSessionDockIntent) => void;
}) {
  const bridge = useMemo(() => leftDockBridge(), []);
  const [project, setProject] = useState<BridgeProject | null>(null);
  const [layout, setLayout] = useState<BridgeLayoutPresentation | null>(null);
  const [focusedSlot, setFocusedSlot] = useState<SessionSlot>("left");
  const [maximizedSlot, setMaximizedSlot] = useState<SessionSlot | null>(null);
  const [status, setStatus] = useState("Loading Session layout…");
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const pendingSelectionRef = useRef<DualSessionSelection | null>(null);
  const [selectionReplayNonce, setSelectionReplayNonce] = useState(0);

  const finishBusy = () => {
    busyRef.current = false;
    setBusy(false);
    if (pendingSelectionRef.current) setSelectionReplayNonce((value) => value + 1);
  };

  useEffect(() => {
    let cancelled = false;
    bridge.snapshot().then(async (snapshot) => {
      const nextProject = (selection
        ? snapshot.projects.find((candidate) => candidate.project.canonicalWorkspaceId === selection.workspaceId)
        : null)
        ?? snapshot.projects.find((candidate) => candidate.sessions.some((session) => !session.archived))
        ?? snapshot.projects[0]
        ?? null;
      if (!nextProject || cancelled) return;
      const saved = await bridge.loadLayout(nextProject.project.canonicalWorkspaceId);
      if (cancelled) return;
      setProject(nextProject);
      setLayout(reconcileLayout(saved, nextProject));
      setStatus(saved ? "Restored Session identities as presentation only · no runtime ownership" : "Default Session layout · not yet persisted");
    }).catch((error) => {
      if (!cancelled) setStatus(`Session layout unavailable · ${errorMessage(error)}`);
    });
    return () => { cancelled = true; };
  }, [bridge]);

  const persist = async (next: BridgeLayoutPresentation, message: string) => {
    if (busyRef.current || !project) return false;
    busyRef.current = true;
    setBusy(true);
    setStatus("Saving presentation layout…");
    try {
      const saved = await bridge.saveLayout(layoutSaveRequest(next));
      setLayout(reconcileLayout(saved, project));
      setStatus(`${message} · presentation only`);
      return true;
    } catch (error) {
      setStatus(`Layout update failed · ${errorMessage(error)}`);
      try {
        const fresh = await bridge.loadLayout(next.workspaceId);
        setLayout(reconcileLayout(fresh, project));
      } catch {
        // Preserve the last renderer state while the explicit failure remains visible.
      }
      return false;
    } finally {
      finishBusy();
    }
  };

  useEffect(() => {
    const requestedSelection = pendingSelectionRef.current ?? selection;
    if (!requestedSelection) return;
    if (busyRef.current) {
      pendingSelectionRef.current = requestedSelection;
      return;
    }
    pendingSelectionRef.current = null;
    busyRef.current = true;
    setBusy(true);
    let cancelled = false;
    const applySelection = async () => {
      try {
        const snapshot = await bridge.snapshot();
        const owner = snapshot.projects.find((candidate) => candidate.project.canonicalWorkspaceId === requestedSelection.workspaceId);
        if (!owner || cancelled) {
          if (!cancelled) setStatus(`Selected Session Project is unavailable · ${requestedSelection.workspaceId}`);
          return;
        }
        const targetExists = owner.sessions.some((session) => !session.archived && session.canonicalSessionId === requestedSelection.sessionId);
        if (!targetExists) {
          if (!cancelled) setStatus(`Selected canonical Session is unavailable · ${requestedSelection.sessionId}`);
          return;
        }
        const currentLayout = project?.project.canonicalWorkspaceId === owner.project.canonicalWorkspaceId && layout
          ? layout
          : reconcileLayout(await bridge.loadLayout(owner.project.canonicalWorkspaceId), owner);
        const occupiedSlot = slotForSession(currentLayout, requestedSelection.sessionId);
        if (occupiedSlot) {
          if (cancelled) return;
          setProject(owner);
          setLayout(currentLayout);
          setFocusedSlot(occupiedSlot);
          setMaximizedSlot(null);
          setStatus(`Focused exact canonical Session · ${requestedSelection.sessionId} · presentation only`);
          return;
        }
        let destinationSlot = focusedSlot;
        let next = selectSessionForSlot(currentLayout, owner, destinationSlot, requestedSelection.sessionId);
        if (!next && currentLayout.layoutMode === "SINGLE" && destinationSlot === "right") {
          destinationSlot = "left";
          next = selectSessionForSlot(currentLayout, owner, destinationSlot, requestedSelection.sessionId);
        }
        if (!next) {
          if (!cancelled) setStatus(`Selected Session already occupies the peer Slot · ${requestedSelection.sessionId}`);
          return;
        }
        if (cancelled) return;
        if (next.leftSessionId === currentLayout.leftSessionId && next.rightSessionId === currentLayout.rightSessionId) {
          setProject(owner);
          setLayout(currentLayout);
          setFocusedSlot(destinationSlot);
          setMaximizedSlot(null);
          setStatus(`Focused exact canonical Session · ${requestedSelection.sessionId}`);
          return;
        }
        setStatus(`Selecting exact canonical Session · ${requestedSelection.sessionId}`);
        const saved = await bridge.saveLayout(layoutSaveRequest(next));
        if (!cancelled) {
          setProject(owner);
          setLayout(reconcileLayout(saved, owner));
          setFocusedSlot(destinationSlot);
          setMaximizedSlot(null);
          setStatus(`Focused exact canonical Session · ${requestedSelection.sessionId} · presentation only`);
        }
      } catch (error) {
        if (!cancelled) setStatus(`Session selection failed · ${errorMessage(error)}`);
      } finally {
        finishBusy();
      }
    };
    void applySelection();
    return () => { cancelled = true; };
  }, [bridge, selection, selectionReplayNonce]);

  useEffect(() => {
    if (!project || !layout) return;
    const sessionId = focusedSlot === "left" ? layout.leftSessionId : layout.rightSessionId;
    if (!sessionId) return;
    onFocusSession?.({
      workspaceId: project.project.canonicalWorkspaceId,
      sessionId,
    });
  }, [focusedSlot, layout?.leftSessionId, layout?.rightSessionId, onFocusSession, project?.project.canonicalWorkspaceId]);

  if (!project || !layout) {
    return <div className="dual-session-empty" data-state="loading">{status}</div>;
  }

  const left = sessionForSlot(project, layout.leftSessionId, 0);
  const right = layout.layoutMode === "DUAL" ? sessionForSlot(project, layout.rightSessionId, 1) : null;
  const dual = layout.layoutMode === "DUAL" && Boolean(left && right);
  const sharedWorktree = dual && left?.canonicalWorkspaceId === right?.canonicalWorkspaceId;

  const operate = (next: BridgeLayoutPresentation | null, message: string) => {
    if (!next) {
      setStatus("Requested Session layout is unavailable");
      return;
    }
    void persist(next, message);
  };

  const renderSlot = (slot: SessionSlot, session: NonNullable<typeof left>, templateIndex: number) => (
    <section
      className="session-slot"
      data-slot={slot}
      data-focused={focusedSlot === slot ? "true" : "false"}
      data-maximized={maximizedSlot === slot ? "true" : "false"}
      key={session.canonicalSessionId}
      onFocusCapture={() => setFocusedSlot(slot)}
      onPointerDown={() => setFocusedSlot(slot)}
      aria-label={`${slot} Session Slot. ${session.displayName}.${focusedSlot === slot ? " Focused." : ""}`}
      aria-current={focusedSlot === slot ? "true" : undefined}
    >
      <div className="session-slot-controls">
        <label>
          <span>Replace {slot}</span>
          <select
            aria-label={`Replace ${slot} Session`}
            value={session.canonicalSessionId}
            disabled={busy}
            onChange={(event) => { setMaximizedSlot(null); operate(replaceSlot(layout, project, slot, event.currentTarget.value), `Replaced ${slot} Session only`); }}
          >
            {project.sessions.filter((candidate) => !candidate.archived).map((candidate) => (
              <option
                key={candidate.canonicalSessionId}
                value={candidate.canonicalSessionId}
                disabled={(slot === "left" ? layout.rightSessionId : layout.leftSessionId) === candidate.canonicalSessionId}
              >
                {candidate.displayName}
              </option>
            ))}
          </select>
        </label>
        <button type="button" className="text-button" disabled={busy} onClick={() => setMaximizedSlot(slot)}>Maximize</button>
        <button type="button" className="text-button" disabled={busy} onClick={() => { setFocusedSlot("left"); setMaximizedSlot(null); operate(closeSlot(layout, slot), `Closed ${slot} Slot only`); }}>Close</button>
      </div>
      <SessionSurface
        session={sessionForSlot(project, session.canonicalSessionId, templateIndex) ?? session}
        focused={focusedSlot === slot}
        mode="workbench"
        onDockIntent={(surface, workspaceId, sessionId) => onDockIntent?.({ surface, workspaceId, sessionId })}
      />
    </section>
  );

  const children = [];
  if (left) children.push(renderSlot("left", left, 0));
  if (dual && !maximizedSlot) {
    children.push(
      <input
        key="divider"
        className="session-divider"
        type="range"
        min="2500"
        max="7500"
        step="100"
        value={layout.splitBasisPoints}
        aria-label="Resize Session divider"
        disabled={busy}
        onChange={(event) => setLayout({ ...layout, splitBasisPoints: Number(event.currentTarget.value) })}
        onPointerUp={(event) => void persist({ ...layout, splitBasisPoints: Number(event.currentTarget.value) }, "Resized Session divider")}
        onKeyUp={(event) => void persist({ ...layout, splitBasisPoints: Number(event.currentTarget.value) }, "Resized Session divider")}
      />,
    );
  }
  if (right) children.push(renderSlot("right", right, 1));

  return (
    <section className="dual-session-workspace" aria-label="Dual Session view" data-surface="workbench">
      <header className="dual-session-toolbar">
        <div>
          <strong>{dual ? "Dual Session Workbench" : "Single Session Workbench"}</strong>
          <span>{status}</span>
        </div>
        <div className="dual-session-actions">
          {dual ? (
            <>
              <button type="button" className="text-button" disabled={busy || maximizedSlot !== null} onClick={() => operate(swapSlots(layout), "Swapped exact Session identities")}>Swap</button>
              <button type="button" className="text-button" disabled={maximizedSlot !== null} onClick={() => setFocusedSlot("left")}>Focus left</button>
              <button type="button" className="text-button" disabled={maximizedSlot !== null} onClick={() => setFocusedSlot("right")}>Focus right</button>
            </>
          ) : (
            <button type="button" className="text-button" disabled={busy} onClick={() => operate(enableDual(layout, project), "Opened second Session Slot")}>Open second Session</button>
          )}
          {maximizedSlot && <button type="button" className="text-button" onClick={() => setMaximizedSlot(null)}>Restore split</button>}
        </div>
      </header>

      {sharedWorktree && (
        <div className="shared-worktree-notice" role="status">
          Shared worktree · file changes may overlap, but every composer remains exact-target and never broadcasts.
        </div>
      )}
      {dual && (
        <div className="narrow-dual-fallback" role="status">
          Narrow fallback · dual identities are preserved; only the focused Slot is shown. Use Focus left/right to switch.
        </div>
      )}

      <div
        className="dual-session-layout"
        data-mode={dual ? "DUAL" : "SINGLE"}
        data-maximized={maximizedSlot ?? "none"}
        style={dual && !maximizedSlot ? { gridTemplateColumns: `${layout.splitBasisPoints}fr 12px ${10000 - layout.splitBasisPoints}fr` } : undefined}
      >
        {children}
      </div>
    </section>
  );
}
