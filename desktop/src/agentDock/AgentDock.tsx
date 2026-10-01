import { useEffect, useMemo, useState } from "react";
import { topologyBridge } from "../multiplexer/bridge";
import type { BridgeAgentDockSnapshot, BridgeAgentObservation } from "../leftDock/types";
import {
  agentDisplayName,
  detectionOnlyLabel,
  exactAgentPaneTarget,
  filterAndSortAgents,
  groupAgentsByWorkspace,
  type AgentDockSort,
} from "./model";
import "./agentDock.css";

const MAX_ALIAS_LENGTH = 128;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function compactId(value: string): string {
  return value.length <= 12 ? value : `${value.slice(0, 6)}…${value.slice(-6)}`;
}

function AgentRow({
  observation,
  displayName,
  selected,
  focused,
  onView,
  onFocus,
}: {
  readonly observation: BridgeAgentObservation;
  readonly displayName: string;
  readonly selected: boolean;
  readonly focused: boolean;
  readonly onView: () => void;
  readonly onFocus: () => void;
}) {
  return (
    <div
      className="agent-row-shell"
      data-selected={selected ? "true" : "false"}
      data-focused={focused ? "true" : "false"}
    >
      <button
        type="button"
        className="agent-row"
        aria-current={selected ? "true" : undefined}
        onClick={onView}
      >
        <span className="agent-row-title">{displayName}</span>
        <span className="agent-row-meta">
          {observation.sourceClass} · {observation.freshness}
        </span>
        <span className="agent-row-binding">
          pane {compactId(observation.paneId)} · obs {compactId(observation.observationId)}
        </span>
      </button>
      <button
        type="button"
        className="agent-focus-action"
        onClick={onFocus}
        aria-label={`Focus exact pane for ${displayName}, observation ${observation.observationId}`}
      >
        {focused ? "Focused" : "Focus pane"}
      </button>
    </div>
  );
}

export function AgentDock({ snapshot }: { readonly snapshot: BridgeAgentDockSnapshot }) {
  const bridge = useMemo(() => topologyBridge(), []);
  const [aliases, setAliases] = useState<Record<string, string>>({});
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<AgentDockSort>("family");
  const [groupByWorkspace, setGroupByWorkspace] = useState(true);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [focusedId, setFocusedId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [status, setStatus] = useState("Agent observations are presentation-only.");

  const visible = useMemo(
    () => filterAndSortAgents(snapshot.observations, aliases, query, sort),
    [aliases, query, snapshot.observations, sort],
  );
  const groups = useMemo(
    () => groupByWorkspace
      ? groupAgentsByWorkspace(visible)
      : ([['ALL', visible] as const] as const),
    [groupByWorkspace, visible],
  );
  const selected = snapshot.observations.find((item) => item.observationId === selectedId) ?? null;

  useEffect(() => {
    if (selectedId && !snapshot.observations.some((item) => item.observationId === selectedId)) {
      setSelectedId(null);
      setRenameValue("");
    }
    if (focusedId && !snapshot.observations.some((item) => item.observationId === focusedId)) {
      setFocusedId(null);
    }
  }, [focusedId, selectedId, snapshot.observations]);

  async function focusObservation(observation: BridgeAgentObservation) {
    if (observation.freshness !== "CURRENT") {
      setStatus(`Focus refused · observation is ${observation.freshness}`);
      return;
    }
    setStatus("Resolving exact pane against current canonical topology…");
    try {
      const topology = await bridge.snapshot();
      const target = exactAgentPaneTarget(observation);
      const bound = await bridge.bindTarget(topology.topologyGeneration, target);
      if (
        bound.multiplexerWorkspaceId !== target.multiplexerWorkspaceId
        || bound.tabId !== target.tabId
        || bound.paneId !== target.paneId
      ) {
        throw new Error("trusted host returned a substituted pane identity");
      }
      setFocusedId(observation.observationId);
      setSelectedId(observation.observationId);
      setStatus(
        `Visible focus bound to pane ${observation.paneId} · presentation only · no controller/write authority`,
      );
    } catch (error) {
      setFocusedId(null);
      setStatus(`Focus unavailable · ${errorMessage(error)}`);
    }
  }

  function viewObservation(observation: BridgeAgentObservation) {
    setSelectedId(observation.observationId);
    setRenameValue(aliases[observation.observationId] ?? "");
    setStatus(`Viewing exact observation ${observation.observationId}`);
  }

  function saveAlias(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) return;
    const alias = renameValue.trim();
    if (alias.length > MAX_ALIAS_LENGTH) {
      setStatus(`Alias refused · maximum ${MAX_ALIAS_LENGTH} characters`);
      return;
    }
    setAliases((current) => {
      const next = { ...current };
      if (alias) next[selected.observationId] = alias;
      else delete next[selected.observationId];
      return next;
    });
    setStatus(
      alias
        ? `Display alias updated locally for ${selected.observationId} · immutable identity unchanged`
        : `Display alias cleared for ${selected.observationId} · immutable identity unchanged`,
    );
  }

  return (
    <section className="agent-dock" aria-label="Agent observations" data-authority={snapshot.authority}>
      <div className="agent-dock-heading">
        <div>
          <p className="section-kicker">Agent plane</p>
          <h2>Agents</h2>
        </div>
        <span className="agent-count">{snapshot.observations.length}</span>
      </div>

      {snapshot.availability !== "CURRENT" ? (
        <div className="agent-unavailable" role="status">
          <strong>Observation truth unavailable</strong>
          <span>{snapshot.unavailableReason ?? "OWNER_OBSERVATION_UNAVAILABLE"}</span>
          <span>Detection only · no provider or execution authority is inferred.</span>
        </div>
      ) : (
        <>
          <div className="agent-controls">
            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.currentTarget.value)}
              placeholder="Filter exact agent observations"
              aria-label="Filter exact agent observations"
            />
            <select value={sort} onChange={(event) => setSort(event.currentTarget.value as AgentDockSort)} aria-label="Sort agent observations">
              <option value="family">Family</option>
              <option value="recent">Most recent</option>
              <option value="source">Source</option>
            </select>
            <label className="agent-group-toggle">
              <input
                type="checkbox"
                checked={groupByWorkspace}
                onChange={(event) => setGroupByWorkspace(event.currentTarget.checked)}
              />
              Group by workspace
            </label>
          </div>

          <div className="agent-list" role="list">
            {groups.map(([workspaceId, observations]) => (
              <div className="agent-group" key={workspaceId} role="group" aria-label={workspaceId === "ALL" ? "All agent observations" : `Workspace ${workspaceId}`}>
                {workspaceId !== "ALL" && <div className="agent-group-label">Workspace · {compactId(workspaceId)}</div>}
                {observations.map((observation) => (
                  <div key={observation.observationId} role="listitem">
                    <AgentRow
                      observation={observation}
                      displayName={agentDisplayName(observation, aliases)}
                      selected={selectedId === observation.observationId}
                      focused={focusedId === observation.observationId}
                      onView={() => viewObservation(observation)}
                      onFocus={() => void focusObservation(observation)}
                    />
                  </div>
                ))}
              </div>
            ))}
            {visible.length === 0 && <p className="agent-empty">No matching current observations</p>}
          </div>
        </>
      )}

      {selected && (
        <div className="agent-detail" aria-label={`Agent observation ${selected.observationId}`}>
          <strong>{agentDisplayName(selected, aliases)}</strong>
          <span>{detectionOnlyLabel(selected)}</span>
          <dl>
            <dt>Observation</dt><dd>{selected.observationId}</dd>
            <dt>Family</dt><dd>{selected.family}</dd>
            <dt>Source</dt><dd>{selected.sourceClass}</dd>
            <dt>Confidence</dt><dd>{selected.confidenceClass}</dd>
            <dt>Freshness</dt><dd>{selected.freshness}</dd>
            <dt>Workspace</dt><dd>{selected.multiplexerWorkspaceId}</dd>
            <dt>Tab</dt><dd>{selected.tabId}</dd>
            <dt>Pane</dt><dd>{selected.paneId}</dd>
            <dt>Runtime</dt><dd>{selected.runtimeNamespaceId ?? "UNBOUND"}</dd>
            <dt>Owner generation</dt><dd>{selected.ownerGenerationId}</dd>
          </dl>
          <p className="agent-evidence">{selected.structuredEvidenceSummary}</p>
          <form className="agent-alias-form" onSubmit={saveAlias}>
            <label>
              <span>Display alias only</span>
              <input
                value={renameValue}
                maxLength={MAX_ALIAS_LENGTH}
                onChange={(event) => setRenameValue(event.currentTarget.value)}
                placeholder="Optional local alias"
              />
            </label>
            <button type="submit" className="quiet-action">Save alias</button>
          </form>
        </div>
      )}

      <div className="agent-status" role="status" aria-live="polite">{status}</div>
    </section>
  );
}
