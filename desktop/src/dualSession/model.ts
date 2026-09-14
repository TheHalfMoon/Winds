import type {
  BridgeLayoutPresentation,
  BridgeProject,
  BridgeSessionSummary,
  LayoutPresentationRequest,
} from "../leftDock/types";
import type { SessionSurfaceFixture } from "../sessionSurface/types";

export type SessionSlot = "left" | "right";

function availableSessions(project: BridgeProject): readonly BridgeSessionSummary[] {
  return project.sessions.filter((session) => !session.archived);
}

export function defaultLayoutForProject(project: BridgeProject): BridgeLayoutPresentation {
  const sessions = availableSessions(project);
  const leftSessionId = sessions[0]?.canonicalSessionId ?? null;
  const rightSessionId = sessions[1]?.canonicalSessionId ?? null;
  return {
    workspaceId: project.project.canonicalWorkspaceId,
    layoutMode: rightSessionId ? "DUAL" : "SINGLE",
    leftSessionId,
    rightSessionId,
    splitBasisPoints: 5000,
    revision: null,
  };
}

export function reconcileLayout(
  layout: BridgeLayoutPresentation | null,
  project: BridgeProject,
): BridgeLayoutPresentation {
  const fallback = defaultLayoutForProject(project);
  if (!layout || layout.workspaceId !== project.project.canonicalWorkspaceId) return fallback;
  const ids = new Set(availableSessions(project).map((session) => session.canonicalSessionId));
  const leftSessionId = layout.leftSessionId && ids.has(layout.leftSessionId) ? layout.leftSessionId : fallback.leftSessionId;
  const remaining = availableSessions(project).find((session) => session.canonicalSessionId !== leftSessionId)?.canonicalSessionId ?? null;
  const savedRight = layout.rightSessionId && ids.has(layout.rightSessionId) && layout.rightSessionId !== leftSessionId
    ? layout.rightSessionId
    : null;
  const rightSessionId = layout.layoutMode === "DUAL" ? (savedRight ?? remaining) : null;
  return {
    ...layout,
    layoutMode: rightSessionId ? layout.layoutMode : "SINGLE",
    leftSessionId,
    rightSessionId,
    splitBasisPoints: Math.min(7500, Math.max(2500, layout.splitBasisPoints)),
  };
}

export function swapSlots(layout: BridgeLayoutPresentation): BridgeLayoutPresentation {
  if (layout.layoutMode !== "DUAL" || !layout.leftSessionId || !layout.rightSessionId) return layout;
  return { ...layout, leftSessionId: layout.rightSessionId, rightSessionId: layout.leftSessionId };
}

export function replaceSlot(
  layout: BridgeLayoutPresentation,
  project: BridgeProject,
  slot: SessionSlot,
  replacementSessionId: string,
): BridgeLayoutPresentation | null {
  if (!availableSessions(project).some((session) => session.canonicalSessionId === replacementSessionId)) return null;
  const peer = slot === "left" ? layout.rightSessionId : layout.leftSessionId;
  if (peer === replacementSessionId) return null;
  if (slot === "right" && layout.layoutMode !== "DUAL") return null;
  return slot === "left"
    ? { ...layout, leftSessionId: replacementSessionId }
    : { ...layout, rightSessionId: replacementSessionId };
}

export function slotForSession(layout: BridgeLayoutPresentation, sessionId: string): SessionSlot | null {
  if (layout.leftSessionId === sessionId) return "left";
  if (layout.layoutMode === "DUAL" && layout.rightSessionId === sessionId) return "right";
  return null;
}

export function selectSessionForSlot(
  layout: BridgeLayoutPresentation,
  project: BridgeProject,
  slot: SessionSlot,
  sessionId: string,
): BridgeLayoutPresentation | null {
  if (layout.layoutMode === "SINGLE" && slot === "right") return null;
  return replaceSlot(layout, project, slot, sessionId);
}

export function closeSlot(layout: BridgeLayoutPresentation, slot: SessionSlot): BridgeLayoutPresentation {
  if (slot === "right" || layout.layoutMode !== "DUAL") {
    return { ...layout, layoutMode: "SINGLE", rightSessionId: null, leftSessionId: slot === "left" ? null : layout.leftSessionId };
  }
  return { ...layout, layoutMode: "SINGLE", leftSessionId: layout.rightSessionId, rightSessionId: null };
}

export function enableDual(layout: BridgeLayoutPresentation, project: BridgeProject): BridgeLayoutPresentation | null {
  if (layout.layoutMode === "DUAL" && layout.leftSessionId && layout.rightSessionId) return layout;
  const sessions = availableSessions(project);
  const leftSessionId = layout.leftSessionId ?? sessions[0]?.canonicalSessionId ?? null;
  const peer = sessions.find((session) => session.canonicalSessionId !== leftSessionId);
  if (!leftSessionId || !peer) return null;
  return { ...layout, layoutMode: "DUAL", leftSessionId, rightSessionId: peer.canonicalSessionId };
}

export function layoutSaveRequest(layout: BridgeLayoutPresentation): LayoutPresentationRequest {
  return {
    workspaceId: layout.workspaceId,
    layoutMode: layout.layoutMode,
    leftSessionId: layout.leftSessionId,
    rightSessionId: layout.layoutMode === "DUAL" ? layout.rightSessionId : null,
    splitBasisPoints: layout.splitBasisPoints,
    expectedRevision: layout.revision,
  };
}

export function sessionForSlot(
  project: BridgeProject,
  sessionId: string | null,
  templateIndex: number,
): SessionSurfaceFixture | null {
  const session = project.sessions.find((candidate) => candidate.canonicalSessionId === sessionId);
  if (!session) return null;
  const { state, requested, observed } = session.runtime;
  const observedFamily = observed.length === 1 ? observed[0] : null;
  const requestedFamily = requested.length === 1 ? requested[0] : null;
  const family = state === "observed" && observedFamily
    ? observedFamily
    : state === "requested" && requestedFamily
      ? requestedFamily
      : state === "stale"
        ? "stale"
        : state === "unavailable"
          ? "unavailable"
          : state === "unknown"
            ? "unknown"
            : "conflicting";
  const label = state === "observed" && observedFamily
    ? `${observedFamily === "codex" ? "Codex" : "Claude"} observed`
    : state === "requested" && requestedFamily
      ? `${requestedFamily === "codex" ? "Codex" : "Claude"} requested`
      : state === "mismatch" ? "Runtime mismatch" : `Runtime ${state}`;
  const proof = state === "observed"
    ? "Winds-observed runtime identity"
    : state === "requested"
      ? "Requested only · not observed"
      : "No current runtime ownership is inferred from layout presentation";
  const unavailable = ["unknown", "unavailable", "conflicting", "mismatch"].includes(state);
  const attention = session.attention === "none" ? "Ready presentation" : session.attention.replaceAll("_", " ");
  return {
    canonicalSessionId: session.canonicalSessionId,
    canonicalWorkstreamId: session.canonicalWorkstreamId,
    canonicalWorkspaceId: session.canonicalWorkspaceId,
    displayName: session.displayName,
    lifecycle: attention,
    worktreeContext: project.project.canonicalRepoRoot,
    runtime: { family, proofState: state, label, proof },
    surfaceState: unavailable ? "empty" : "ready",
    composerMode: unavailable ? "unavailable" : "fixture_only",
    events: [{
      id: `${session.canonicalSessionId}-layout-${templateIndex}`,
      kind: "agent_response",
      title: "Session presentation",
      body: "This T134 event is presentation-only and does not imply runtime input, ownership, or dispatch.",
      source: "fixture",
      meta: "layout fixture · not verification evidence",
    }],
  };
}
