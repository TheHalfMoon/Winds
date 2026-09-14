import { invoke, isTauri } from "@tauri-apps/api/core";
import { fixtureLeftDockBridge } from "./fixture";
import type { LeftDockBridge } from "./types";

const canonicalLeftDockBridge: LeftDockBridge = {
  source: "canonical",
  snapshot: () => invoke("left_dock_snapshot"),
  updateProject: (request) => invoke("left_dock_update_project", { request }),
  createSession: (request) => invoke("left_dock_create_session", { request }),
  renameSession: (request) => invoke("left_dock_rename_session", { request }),
  updateSession: (request) => invoke("left_dock_update_session", { request }),
};

export function leftDockBridge(): LeftDockBridge {
  return isTauri() ? canonicalLeftDockBridge : fixtureLeftDockBridge;
}
