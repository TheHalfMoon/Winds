import type { BridgeLayoutPresentation, BridgeSnapshot, LeftDockBridge } from "./types";

const fixture: BridgeSnapshot = {
  projects: [
    {
      project: {
        projectViewId: "fixture-winds",
        displayName: "Winds",
        canonicalWorkspaceId: "fixture-winds",
        canonicalRepoRoot: "/fixture/Winds",
        canonicalGitCommonDir: "/fixture/Winds/.git",
        pinned: true,
        presentationOrder: 0,
        collapsed: false,
        presentationRevision: 1,
        sessionCount: 3,
        attentionCount: 1,
        attention: "waiting_approval",
        searchInput: "winds\nfixture-winds\n/fixture/winds",
      },
      availableWorkstreams: [{ workstreamId: "fixture-workstream", displayName: "Desktop" }],
      sessions: [
        {
          canonicalSessionId: "fixture-session-a",
          canonicalWorkstreamId: "fixture-workstream",
          canonicalWorkspaceId: "fixture-winds",
          displayName: "Quiet Current",
          displayAlias: "Quiet Current",
          canonicalDisplayName: "Session A",
          pinned: true,
          presentationOrder: 0,
          archived: false,
          presentationRevision: 1,
          runtime: { state: "observed", requested: ["codex"], observed: ["codex"] },
          attention: "none",
          searchInput: "quiet current\nsession a\nfixture-session-a\nfixture-winds",
        },
        {
          canonicalSessionId: "fixture-session-b",
          canonicalWorkstreamId: "fixture-workstream",
          canonicalWorkspaceId: "fixture-winds",
          displayName: "Runtime bridge review",
          displayAlias: null,
          canonicalDisplayName: "Runtime bridge review",
          pinned: false,
          presentationOrder: 10,
          archived: false,
          presentationRevision: null,
          runtime: { state: "requested", requested: ["claude"], observed: [] },
          attention: "waiting_approval",
          searchInput: "runtime bridge review\nfixture-session-b\nfixture-winds",
        },
        {
          canonicalSessionId: "fixture-session-c",
          canonicalWorkstreamId: "fixture-workstream",
          canonicalWorkspaceId: "fixture-winds",
          displayName: "Unknown runtime",
          displayAlias: null,
          canonicalDisplayName: "Unknown runtime",
          pinned: false,
          presentationOrder: 20,
          archived: false,
          presentationRevision: null,
          runtime: { state: "unknown", requested: [], observed: [] },
          attention: "none",
          searchInput: "unknown runtime\nfixture-session-c\nfixture-winds",
        },
      ],
    },
  ],
};

const T143_PROJECT_COUNT = 100;
const T143_SESSIONS_PER_PROJECT = 10;
let t143PerformanceFixture: BridgeSnapshot | null = null;

function buildT143PerformanceFixture(): BridgeSnapshot {
  if (t143PerformanceFixture) return t143PerformanceFixture;
  const projects = Array.from({ length: T143_PROJECT_COUNT }, (_, projectIndex) => {
    const projectId = `t143-project-${String(projectIndex).padStart(3, "0")}`;
    const sessions = Array.from({ length: T143_SESSIONS_PER_PROJECT }, (_, sessionIndex) => {
      const sessionId = `${projectId}-session-${String(sessionIndex).padStart(2, "0")}`;
      const runtime = sessionIndex % 2 === 0
        ? { state: "observed" as const, requested: ["codex" as const], observed: ["codex" as const] }
        : { state: "requested" as const, requested: ["claude" as const], observed: [] };
      return {
        canonicalSessionId: sessionId,
        canonicalWorkstreamId: `${projectId}-workstream`,
        canonicalWorkspaceId: projectId,
        displayName: `Performance Session ${projectIndex}-${sessionIndex}`,
        displayAlias: null,
        canonicalDisplayName: `Performance Session ${projectIndex}-${sessionIndex}`,
        pinned: sessionIndex === 0,
        presentationOrder: sessionIndex * 10,
        archived: false,
        presentationRevision: null,
        runtime,
        attention: "none" as const,
        searchInput: `performance session ${projectIndex} ${sessionIndex}\n${sessionId}\n${projectId}`,
      };
    });
    return {
      project: {
        projectViewId: projectId,
        displayName: `Performance Project ${projectIndex}`,
        canonicalWorkspaceId: projectId,
        canonicalRepoRoot: `/fixture/performance/${projectId}`,
        canonicalGitCommonDir: `/fixture/performance/${projectId}/.git`,
        pinned: projectIndex === 0,
        presentationOrder: (projectIndex + 1) * 10,
        collapsed: false,
        presentationRevision: null,
        sessionCount: sessions.length,
        attentionCount: 0,
        attention: "none" as const,
        searchInput: `performance project ${projectIndex}\n${projectId}`,
      },
      availableWorkstreams: [{ workstreamId: `${projectId}-workstream`, displayName: "Performance" }],
      sessions,
    };
  });
  t143PerformanceFixture = { projects: [...fixture.projects, ...projects] };
  return t143PerformanceFixture;
}

function activeFixture(): BridgeSnapshot {
  if (typeof window === "undefined" || import.meta.env.VITE_WINDS_T143_BENCHMARK !== "1") return fixture;
  return new URLSearchParams(window.location.search).get("perf") === "t143-large"
    ? buildT143PerformanceFixture()
    : fixture;
}

let fixtureLayout: BridgeLayoutPresentation = {
  workspaceId: "fixture-winds",
  layoutMode: "DUAL",
  leftSessionId: "fixture-session-a",
  rightSessionId: "fixture-session-b",
  splitBasisPoints: 5000,
  revision: 1,
};

function readOnly(): Promise<never> {
  return Promise.reject(new Error("Fixture mode is read-only · no canonical authority"));
}

export const fixtureLeftDockBridge: LeftDockBridge = {
  source: "fixture",
  async snapshot() { return activeFixture(); },
  async attentionSnapshot() {
    return {
      items: [{
        workspaceId: "fixture-winds",
        sessionId: "fixture-session-b",
        workflowRunId: "fixture-workflow",
        stageRunId: "fixture-stage",
        stageKey: "review",
        state: "waiting_approval",
        reason: "fixture canonical approval gate",
        source: "WINDS_OBSERVED",
        authority: "WINDS_POLICY",
        candidateOid: "fixture-candidate",
        candidateTree: "fixture-candidate-tree",
        approvalActionAvailable: false,
      }],
    };
  },
  updateProject: readOnly,
  createSession: readOnly,
  renameSession: readOnly,
  updateSession: readOnly,
  async loadLayout(workspaceId) {
    return workspaceId === fixtureLayout.workspaceId ? fixtureLayout : null;
  },
  async saveLayout(request) {
    if (request.workspaceId !== "fixture-winds") throw new Error("Fixture layout Project is unavailable");
    const sessionIds = new Set(fixture.projects[0].sessions.map((session) => session.canonicalSessionId));
    if (request.leftSessionId && !sessionIds.has(request.leftSessionId)) throw new Error("Fixture left Session is unavailable");
    if (request.rightSessionId && !sessionIds.has(request.rightSessionId)) throw new Error("Fixture right Session is unavailable");
    if (request.layoutMode === "DUAL" && (!request.leftSessionId || !request.rightSessionId || request.leftSessionId === request.rightSessionId)) {
      throw new Error("Fixture dual layout requires two distinct Sessions");
    }
    if (request.expectedRevision !== fixtureLayout.revision) throw new Error("Fixture layout lost revision/update race");
    fixtureLayout = {
      workspaceId: request.workspaceId,
      layoutMode: request.layoutMode,
      leftSessionId: request.leftSessionId,
      rightSessionId: request.layoutMode === "SINGLE" ? null : request.rightSessionId,
      splitBasisPoints: request.splitBasisPoints,
      revision: (fixtureLayout.revision ?? 0) + 1,
    };
    return fixtureLayout;
  },
};
