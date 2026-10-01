import type { DesktopTopologyIdentity } from "../multiplexer/types";
import type { BridgeAgentObservation } from "../leftDock/types";

export type AgentDockSort = "family" | "recent" | "source";

export function agentDisplayName(
  observation: BridgeAgentObservation,
  aliases: Readonly<Record<string, string>>,
): string {
  const alias = aliases[observation.observationId]?.trim();
  return alias || observation.family.replaceAll("_", " ");
}

export function exactAgentPaneTarget(
  observation: BridgeAgentObservation,
): DesktopTopologyIdentity {
  return {
    multiplexerWorkspaceId: observation.multiplexerWorkspaceId,
    tabId: observation.tabId,
    paneId: observation.paneId,
  };
}

export function filterAndSortAgents(
  observations: readonly BridgeAgentObservation[],
  aliases: Readonly<Record<string, string>>,
  query: string,
  sort: AgentDockSort,
): readonly BridgeAgentObservation[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const filtered = observations.filter((observation) => {
    if (!normalizedQuery) return true;
    const haystack = [
      agentDisplayName(observation, aliases),
      observation.observationId,
      observation.family,
      observation.sourceClass,
      observation.confidenceClass,
      observation.freshness,
      observation.multiplexerWorkspaceId,
      observation.tabId,
      observation.paneId,
      observation.runtimeNamespaceId ?? "",
      observation.providerNativeSessionId ?? "",
      observation.structuredEvidenceSummary,
    ].join("\n").toLocaleLowerCase();
    return haystack.includes(normalizedQuery);
  });

  return [...filtered].sort((left, right) => {
    const stable = left.observationId.localeCompare(right.observationId);
    if (sort === "recent") {
      return right.observedUnixMs - left.observedUnixMs || stable;
    }
    if (sort === "source") {
      return left.sourceClass.localeCompare(right.sourceClass)
        || left.family.localeCompare(right.family)
        || stable;
    }
    return agentDisplayName(left, aliases).localeCompare(agentDisplayName(right, aliases))
      || left.family.localeCompare(right.family)
      || stable;
  });
}

export function groupAgentsByWorkspace(
  observations: readonly BridgeAgentObservation[],
): readonly (readonly [string, readonly BridgeAgentObservation[]])[] {
  const groups = new Map<string, BridgeAgentObservation[]>();
  for (const observation of observations) {
    const group = groups.get(observation.multiplexerWorkspaceId) ?? [];
    group.push(observation);
    groups.set(observation.multiplexerWorkspaceId, group);
  }
  return [...groups.entries()].sort(([left], [right]) => left.localeCompare(right));
}

export function detectionOnlyLabel(observation: BridgeAgentObservation): string {
  if (observation.runtimeNamespaceId) {
    return "Detection only · runtime observed · execution unproven";
  }
  return "Detection only · execution unproven";
}
