import type { RuntimeFamily } from "../runtime";
import type {
  BridgeAttentionState,
  BridgeProject,
  BridgeProjectSummary,
  BridgeSessionSummary,
  BridgeSnapshot,
  CreateSessionRequest,
  ProjectPresentationRequest,
  RenameSessionRequest,
  SessionPresentationRequest,
} from "./types";

export interface RuntimePresentation {
  readonly family: RuntimeFamily;
  readonly label: string;
  readonly proof: string;
}

export interface AttentionPresentation {
  readonly label: string;
  readonly actionable: boolean;
}

export type SearchResolution =
  | { readonly kind: "none" }
  | { readonly kind: "unique"; readonly sessionId: string }
  | { readonly kind: "ambiguous"; readonly sessionIds: readonly string[] };

const runtimeName = { codex: "Codex", claude: "Claude" } as const;
const attention: Record<BridgeAttentionState, AttentionPresentation> = {
  recovery_required: { label: "Recovery required", actionable: true },
  waiting_approval: { label: "Approval required", actionable: true },
  retry_required: { label: "Retry required", actionable: true },
  waiting_external: { label: "Waiting on external condition", actionable: true },
  stale: { label: "Refresh required", actionable: true },
  unknown: { label: "Attention state unknown", actionable: false },
  none: { label: "No material attention", actionable: false },
};

export function normalizedSearchText(value: string): string {
  return value.trim().replace(/[A-Z]/g, (character) => character.toLowerCase());
}

export function attentionView(state: BridgeAttentionState): AttentionPresentation {
  return attention[state];
}

function oneFamily(values: readonly (keyof typeof runtimeName)[]) {
  return values.length === 1 ? values[0] : null;
}

function families(values: readonly (keyof typeof runtimeName)[]): string {
  return values.length === 0 ? "none" : values.map((value) => runtimeName[value]).join(" + ");
}

export function runtimeView(session: BridgeSessionSummary): RuntimePresentation {
  const { state, requested, observed } = session.runtime;
  const requestedFamily = oneFamily(requested);
  const observedFamily = oneFamily(observed);
  if (state === "observed" && observedFamily) {
    return { family: observedFamily, label: `${runtimeName[observedFamily]} observed`, proof: "Winds-observed runtime identity" };
  }
  if (state === "requested" && requestedFamily) {
    return { family: requestedFamily, label: `${runtimeName[requestedFamily]} requested`, proof: "Requested only · not observed" };
  }
  if (state === "mismatch") {
    return { family: "conflicting", label: "Runtime mismatch", proof: `Requested ${families(requested)} · observed ${families(observed)}` };
  }
  if (state === "stale") {
    return { family: "stale", label: "Runtime identity stale", proof: `Historical observation: ${families(observed)}` };
  }
  if (state === "unavailable") {
    return { family: "unavailable", label: "Runtime unavailable", proof: `Historical observation: ${families(observed)}` };
  }
  if (state === "conflicting") {
    return { family: "conflicting", label: "Runtime identity conflicting", proof: `Requested ${families(requested)} · observed ${families(observed)}` };
  }
  return { family: "unknown", label: "Unknown runtime", proof: "No accepted runtime identity" };
}

export function filterProjects(snapshot: BridgeSnapshot, query: string): readonly BridgeProject[] {
  const needle = normalizedSearchText(query);
  if (!needle) return snapshot.projects;
  return snapshot.projects.flatMap((project) => {
    const projectMatches = normalizedSearchText(project.project.searchInput).includes(needle);
    const sessions = project.sessions.filter((session) => normalizedSearchText(session.searchInput).includes(needle));
    if (!projectMatches && sessions.length === 0) return [];
    return [{ ...project, sessions: projectMatches ? project.sessions : sessions }];
  });
}

export function reconcileSelectedSession(snapshot: BridgeSnapshot, selectedSessionId: string | null): string | null {
  if (!selectedSessionId) return null;
  return snapshot.projects.some((project) => project.sessions.some((session) => session.canonicalSessionId === selectedSessionId))
    ? selectedSessionId
    : null;
}

export function resolveSessionSearch(snapshot: BridgeSnapshot, query: string): SearchResolution {
  const raw = query.trim();
  const needle = normalizedSearchText(query);
  if (!needle) return { kind: "none" };
  const sessions = snapshot.projects.flatMap((project) => project.sessions);
  const canonical = sessions.filter((session) => session.canonicalSessionId === raw);
  if (canonical.length === 1) return { kind: "unique", sessionId: canonical[0].canonicalSessionId };
  if (sessions.some((session) => normalizedSearchText(session.canonicalSessionId) === needle)) return { kind: "none" };
  const exactAlias = sessions.filter((session) => normalizedSearchText(session.displayName) === needle);
  const candidates = exactAlias.length > 0
    ? exactAlias
    : sessions.filter((session) => normalizedSearchText(session.searchInput).includes(needle));
  if (candidates.length === 0) return { kind: "none" };
  if (candidates.length === 1) return { kind: "unique", sessionId: candidates[0].canonicalSessionId };
  return { kind: "ambiguous", sessionIds: candidates.map((session) => session.canonicalSessionId).sort() };
}

export function nextSessionFocusIndex(count: number, currentIndex: number, direction: 1 | -1): number | null {
  if (count <= 0) return null;
  if (currentIndex < 0 || currentIndex >= count) return direction > 0 ? 0 : count - 1;
  return (currentIndex + direction + count) % count;
}

export function projectUpdatePlan(
  project: BridgeProjectSummary,
  changes: Partial<Pick<BridgeProjectSummary, "pinned" | "presentationOrder" | "collapsed">>,
): ProjectPresentationRequest {
  return {
    workspaceId: project.canonicalWorkspaceId,
    displayName: project.displayName,
    pinned: changes.pinned ?? project.pinned,
    sortOrder: changes.presentationOrder ?? project.presentationOrder,
    collapsed: changes.collapsed ?? project.collapsed,
    expectedRevision: project.presentationRevision,
  };
}

export function sessionUpdatePlan(
  session: BridgeSessionSummary,
  changes: Partial<Pick<BridgeSessionSummary, "pinned" | "presentationOrder" | "archived">>,
): SessionPresentationRequest {
  return {
    sessionId: session.canonicalSessionId,
    displayAlias: session.displayAlias ?? session.canonicalDisplayName,
    pinned: changes.pinned ?? session.pinned,
    sortOrder: changes.presentationOrder ?? session.presentationOrder,
    archived: changes.archived ?? session.archived,
    expectedRevision: session.presentationRevision,
  };
}

export function sessionRenamePlan(session: BridgeSessionSummary, displayName: string): RenameSessionRequest {
  return {
    sessionId: session.canonicalSessionId,
    displayName,
    presentation: {
      sessionId: session.canonicalSessionId,
      displayAlias: displayName,
      pinned: session.pinned,
      sortOrder: session.presentationOrder,
      archived: session.archived,
      expectedRevision: session.presentationRevision,
    },
  };
}

export function createSessionPlan(project: BridgeProject, workstreamId: string, displayName: string): CreateSessionRequest {
  return { workspaceId: project.project.canonicalWorkspaceId, workstreamId, displayName };
}

function projectCompare(left: BridgeProjectSummary, right: BridgeProjectSummary): number {
  return left.presentationOrder - right.presentationOrder
    || normalizedSearchText(left.displayName).localeCompare(normalizedSearchText(right.displayName), "en")
    || left.canonicalWorkspaceId.localeCompare(right.canonicalWorkspaceId, "en");
}

function sessionCompare(left: BridgeSessionSummary, right: BridgeSessionSummary): number {
  return left.presentationOrder - right.presentationOrder
    || normalizedSearchText(left.displayName).localeCompare(normalizedSearchText(right.displayName), "en")
    || left.canonicalSessionId.localeCompare(right.canonicalSessionId, "en");
}

function swapAdjacent<T>(ordered: readonly T[], index: number, direction: 1 | -1): readonly T[] | null {
  const neighbor = index + direction;
  if (index < 0 || neighbor < 0 || neighbor >= ordered.length) return null;
  const result = [...ordered];
  [result[index], result[neighbor]] = [result[neighbor], result[index]];
  return result;
}

export function projectReorderPlan(
  projects: readonly BridgeProject[],
  workspaceId: string,
  direction: 1 | -1,
): readonly ProjectPresentationRequest[] | null {
  const current = projects.find((item) => item.project.canonicalWorkspaceId === workspaceId)?.project;
  if (!current) return null;
  const group = projects.map((item) => item.project).filter((item) => item.pinned === current.pinned).sort(projectCompare);
  const index = group.findIndex((item) => item.canonicalWorkspaceId === workspaceId);
  const moved = swapAdjacent(group, index, direction);
  if (!moved) return null;
  const neighbor = moved[index];
  if (current.presentationOrder !== neighbor.presentationOrder) {
    return [
      projectUpdatePlan(current, { presentationOrder: neighbor.presentationOrder }),
      projectUpdatePlan(neighbor, { presentationOrder: current.presentationOrder }),
    ];
  }
  return moved.map((item, order) => projectUpdatePlan(item, { presentationOrder: order * 10 }));
}

export function sessionReorderPlan(
  sessions: readonly BridgeSessionSummary[],
  sessionId: string,
  direction: 1 | -1,
): readonly SessionPresentationRequest[] | null {
  const current = sessions.find((item) => item.canonicalSessionId === sessionId);
  if (!current) return null;
  const group = sessions
    .filter((item) => item.archived === current.archived && item.pinned === current.pinned)
    .sort(sessionCompare);
  const index = group.findIndex((item) => item.canonicalSessionId === sessionId);
  const moved = swapAdjacent(group, index, direction);
  if (!moved) return null;
  const neighbor = moved[index];
  if (current.presentationOrder !== neighbor.presentationOrder) {
    return [
      sessionUpdatePlan(current, { presentationOrder: neighbor.presentationOrder }),
      sessionUpdatePlan(neighbor, { presentationOrder: current.presentationOrder }),
    ];
  }
  return moved.map((item, order) => sessionUpdatePlan(item, { presentationOrder: order * 10 }));
}
