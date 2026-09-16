import { invoke, isTauri } from "@tauri-apps/api/core";
import { fixtureLeftDockBridge } from "./fixture";
import type { LeftDockBridge } from "./types";

const canonicalLeftDockBridge: LeftDockBridge = {
  source: "canonical",
  snapshot: () => invoke("left_dock_snapshot"),
  attentionSnapshot: () => invoke("left_dock_attention_snapshot"),
  updateProject: (updates) => invoke("left_dock_update_project", { request: { updates } }),
  createSession: (request) => invoke("left_dock_create_session", { request }),
  renameSession: (request) => invoke("left_dock_rename_session", { request }),
  updateSession: (updates) => invoke("left_dock_update_session", { request: { updates } }),
  loadLayout: (workspaceId) => invoke("workspace_load_layout", { workspaceId }),
  saveLayout: (request) => invoke("workspace_save_layout", { request }),
};

export function leftDockBridge(): LeftDockBridge {
  if (import.meta.env.VITE_WINDS_T143_BENCHMARK === "1") return fixtureLeftDockBridge;
  return isTauri() ? canonicalLeftDockBridge : fixtureLeftDockBridge;
}
