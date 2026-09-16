import { invoke, isTauri } from "@tauri-apps/api/core";
import { fileFixtures } from "../fixtures/workspace";
import type {
  FilePreviewResponse, RightDockArtifactsResponse, RightDockBinding, RightDockBridge,
  RightDockChangesResponse, RightDockContextResponse, RightDockEvidenceResponse,
  RightDockFilesResponse, RightDockTarget,
} from "./types";

function fixtureBinding(target: RightDockTarget): RightDockBinding {
  const key = `${target.workspaceId}:${target.sessionId}`;
  const digest = Array.from(key).reduce((value, character) => ((value * 33) ^ character.charCodeAt(0)) >>> 0, 5381).toString(16).padStart(8, "0");
  return {
    workspaceId: target.workspaceId,
    sessionId: target.sessionId,
    worktreeRoot: "/fixture/Winds",
    gitCommonDir: "/fixture/Winds/.git",
    workflowRunId: null,
    stageRunId: null,
    candidateOid: null,
    candidateTree: null,
    headOid: "fixture-head",
    treeOid: "fixture-tree",
    worktreeStateSha256: "fixture-worktree-state",
    bindingDigest: `fixture-${digest}`,
  };
}

const fixtureBridge: RightDockBridge = {
  source: "fixture",
  bind: async (target) => fixtureBinding(target),
  files: async (binding): Promise<RightDockFilesResponse> => ({
    binding,
    entries: fileFixtures.filter((entry) => entry.kind === "file").map((entry) => ({ path: entry.name, kind: "file" })),
    truncated: false,
  }),
  preview: async (binding, path): Promise<FilePreviewResponse> => ({
    binding, path, state: "text", byteLen: 38, content: `Fixture preview for ${path}\nNo host authority.`,
  }),
  changes: async (binding): Promise<RightDockChangesResponse> => ({
    binding,
    entries: [{ path: "desktop/src/App.tsx", status: " M" }],
    diff: "Fixture diff only · not verification evidence",
    diffLossy: false,
  }),
  evidence: async (binding): Promise<RightDockEvidenceResponse> => ({ binding, entries: [] }),
  context: async (binding): Promise<RightDockContextResponse> => ({
    binding,
    facts: [
      { key: "session", value: binding.sessionId, source: "FIXTURE_ONLY", authority: "PRESENTATION_ONLY" },
      { key: "candidate_oid", value: binding.candidateOid ?? "unavailable", source: "FIXTURE_ONLY", authority: "PRESENTATION_ONLY" },
    ],
  }),
  artifacts: async (binding): Promise<RightDockArtifactsResponse> => ({ binding, entries: [] }),
};

const canonicalBridge: RightDockBridge = {
  source: "canonical",
  bind: (request) => invoke("right_dock_bind", { request }),
  files: (binding) => invoke("right_dock_files", { request: { binding } }),
  preview: (binding, path) => invoke("right_dock_preview_file", { request: { binding, path } }),
  changes: (binding) => invoke("right_dock_changes", { request: { binding } }),
  evidence: (binding) => invoke("right_dock_evidence", { request: { binding } }),
  context: (binding) => invoke("right_dock_context", { request: { binding } }),
  artifacts: (binding) => invoke("right_dock_artifacts", { request: { binding } }),
};

export function rightDockBridge(): RightDockBridge {
  if (
    import.meta.env.VITE_WINDS_T143_BENCHMARK === "1"
    || import.meta.env.VITE_WINDS_T143_NATIVE_READY === "1"
  ) return fixtureBridge;
  return isTauri() ? canonicalBridge : fixtureBridge;
}
