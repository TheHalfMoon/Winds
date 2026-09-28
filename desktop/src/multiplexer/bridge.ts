import { invoke } from "@tauri-apps/api/core";
import type {
  DesktopTopologyBoundTarget,
  DesktopTopologyCapability,
  DesktopTopologyIdentity,
  DesktopTopologySnapshot,
} from "./types";

const CAPABILITY_COMMAND = "multiplexer_topology_capability";
const SNAPSHOT_COMMAND = "multiplexer_topology_snapshot";
const BIND_TARGET_COMMAND = "multiplexer_topology_bind_target";

export async function loadTopologyCapability(): Promise<DesktopTopologyCapability> {
  return invoke<DesktopTopologyCapability>(CAPABILITY_COMMAND);
}

export async function loadTopologySnapshot(
  expectedOwnerGenerationId: string | null,
): Promise<DesktopTopologySnapshot> {
  return invoke<DesktopTopologySnapshot>(SNAPSHOT_COMMAND, {
    request: { expectedOwnerGenerationId },
  });
}

export async function bindTopologyTarget(
  expectedOwnerGenerationId: string | null,
  topologyGeneration: number,
  target: DesktopTopologyIdentity,
): Promise<DesktopTopologyBoundTarget> {
  return invoke<DesktopTopologyBoundTarget>(BIND_TARGET_COMMAND, {
    request: {
      expectedOwnerGenerationId,
      expectedTopologyGeneration: topologyGeneration,
      multiplexerWorkspaceId: target.multiplexerWorkspaceId,
      tabId: target.tabId ?? null,
      paneId: target.paneId ?? null,
    },
  });
}

export const topologyCommandIds = Object.freeze([
  CAPABILITY_COMMAND,
  SNAPSHOT_COMMAND,
  BIND_TARGET_COMMAND,
] as const);
