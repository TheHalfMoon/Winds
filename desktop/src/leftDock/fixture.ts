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
  async snapshot() { return fixture; },
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
