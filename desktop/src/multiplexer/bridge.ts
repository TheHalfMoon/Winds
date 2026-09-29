import { invoke, isTauri } from "@tauri-apps/api/core";
import type {
  DesktopTopologyBoundTarget,
  DesktopTopologyCapability,
  DesktopTopologyIdentity,
  DesktopTopologySnapshot,
} from "./types";

export interface TopologyBridge {
  source: "canonical" | "fixture";
  capability(): Promise<DesktopTopologyCapability>;
  snapshot(): Promise<DesktopTopologySnapshot>;
  bindTarget(
    topologyGeneration: number,
    target: DesktopTopologyIdentity,
  ): Promise<DesktopTopologyBoundTarget>;
}

const canonicalTopologyBridge: TopologyBridge = {
  source: "canonical",
  capability: () => invoke("multiplexer_topology_capability"),
  snapshot: () => invoke("multiplexer_topology_snapshot", { request: {} }),
  bindTarget: (topologyGeneration, target) =>
    invoke("multiplexer_topology_bind_target", {
      request: {
        expectedTopologyGeneration: topologyGeneration,
        multiplexerWorkspaceId: target.multiplexerWorkspaceId,
        tabId: target.tabId ?? null,
        paneId: target.paneId ?? null,
      },
    }),
};

function unavailable(): Promise<never> {
  return Promise.reject(
    new Error(
      "Canonical multiplexer topology is unavailable outside the trusted Rust host",
    ),
  );
}

// The renderer never invents, caches, or approximates owner topology. Without the
// trusted host every topology read fails closed instead of presenting a fixture as
// owner-authoritative truth.
const fixtureTopologyBridge: TopologyBridge = {
  source: "fixture",
  capability: unavailable,
  snapshot: unavailable,
  bindTarget: unavailable,
};

export function topologyBridge(): TopologyBridge {
  if (
    import.meta.env.VITE_WINDS_T143_BENCHMARK === "1" ||
    import.meta.env.VITE_WINDS_T143_NATIVE_READY === "1"
  ) {
    return fixtureTopologyBridge;
  }
  return isTauri() ? canonicalTopologyBridge : fixtureTopologyBridge;
}
