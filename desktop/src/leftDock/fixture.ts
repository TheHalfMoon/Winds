import type { BridgeSnapshot, LeftDockBridge } from "./types";

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
};
