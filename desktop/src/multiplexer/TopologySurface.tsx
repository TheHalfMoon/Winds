import { useCallback, useEffect, useState } from "react";
import { topologyBridge } from "./bridge";
import type {
  DesktopTopologyBoundTarget,
  DesktopTopologyIdentity,
  DesktopTopologyLayoutNode,
  DesktopTopologySnapshot,
  DesktopTopologyTab,
  DesktopTopologyWorkspace,
} from "./types";
import "./topology.css";

function paneCount(node: DesktopTopologyLayoutNode): number {
  if (node.kind === "PANE") return 1;
  return paneCount(node.first) + paneCount(node.second);
}

interface LayoutProps {
  node: DesktopTopologyLayoutNode;
  workspace: DesktopTopologyWorkspace;
  tab: DesktopTopologyTab;
  bound: DesktopTopologyBoundTarget | null;
  bind: (target: DesktopTopologyIdentity) => Promise<void>;
}

function LayoutNode({ node, workspace, tab, bound, bind }: LayoutProps) {
  if (node.kind === "PANE") {
    const selected =
      bound?.multiplexerWorkspaceId === workspace.multiplexerWorkspaceId &&
      bound.tabId === tab.tabId &&
      bound.paneId === node.paneId;
    const focused = tab.focusedPaneId === node.paneId;
    const zoomed = tab.zoomedPaneId === node.paneId;
    return (
      <button
        type="button"
        className="multiplexer-pane"
        aria-current={selected ? "true" : undefined}
        aria-label={`Pane ${node.paneId}${focused ? ", focused" : ""}${zoomed ? ", zoomed" : ""}`}
        data-pane-id={node.paneId}
        data-focused={focused ? "true" : "false"}
        data-zoomed={zoomed ? "true" : "false"}
        onClick={() =>
          void bind({
            multiplexerWorkspaceId: workspace.multiplexerWorkspaceId,
            tabId: tab.tabId,
            paneId: node.paneId,
          })
        }
      >
        <span className="multiplexer-pane-label">Pane</span>
        <code>{node.paneId}</code>
      </button>
    );
  }

  const firstBasis = Math.min(9000, Math.max(1000, node.ratioBasisPoints));
  const secondBasis = 10000 - firstBasis;
  return (
    <div
      className="multiplexer-split"
      data-axis={node.axis}
      style={{
        gridTemplateColumns:
          node.axis === "HORIZONTAL"
            ? `${firstBasis}fr ${secondBasis}fr`
            : undefined,
        gridTemplateRows:
          node.axis === "VERTICAL"
            ? `${firstBasis}fr ${secondBasis}fr`
            : undefined,
      }}
      role="group"
      aria-label={`${node.axis.toLowerCase()} split`}
    >
      <LayoutNode
        node={node.first}
        workspace={workspace}
        tab={tab}
        bound={bound}
        bind={bind}
      />
      <LayoutNode
        node={node.second}
        workspace={workspace}
        tab={tab}
        bound={bound}
        bind={bind}
      />
    </div>
  );
}

export function TopologySurface() {
  const [snapshot, setSnapshot] = useState<DesktopTopologySnapshot | null>(null);
  const [bound, setBound] = useState<DesktopTopologyBoundTarget | null>(null);
  const [status, setStatus] = useState("Loading canonical topology…");

  const refresh = useCallback(async () => {
    try {
      const bridge = topologyBridge();
      const capability = await bridge.capability();
      if (
        !capability.trustedRustHost ||
        capability.rendererDirectOwnerAccess ||
        capability.rendererSuppliedOwnerGeneration ||
        capability.genericInvokeSurface
      ) {
        throw new Error("Desktop topology capability boundary is not trusted");
      }
      const next = await bridge.snapshot();
      setSnapshot(next);
      setBound((current) =>
        current?.topologyGeneration === next.topologyGeneration ? current : null,
      );
      setStatus(
        `Generation ${next.topologyGeneration} · ${next.workspaces.length} workspace${next.workspaces.length === 1 ? "" : "s"}`,
      );
    } catch (error) {
      setSnapshot(null);
      setBound(null);
      setStatus(
        error instanceof Error ? error.message : "Canonical topology unavailable",
      );
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const bind = useCallback(
    async (target: DesktopTopologyIdentity) => {
      if (!snapshot) return;
      try {
        const next = await topologyBridge().bindTarget(
          snapshot.topologyGeneration,
          target,
        );
        setBound(next);
        setStatus(`Bound exact target at generation ${next.topologyGeneration}`);
      } catch (error) {
        setBound(null);
        setStatus(
          error instanceof Error ? error.message : "Topology target became stale",
        );
        await refresh();
      }
    },
    [refresh, snapshot],
  );

  return (
    <section
      className="multiplexer-topology"
      aria-label="Canonical workspace topology"
      data-authority={snapshot?.authority ?? "UNAVAILABLE"}
    >
      <header className="multiplexer-topology-header">
        <div>
          <strong>Topology</strong>
          <span>{status}</span>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          aria-label="Refresh canonical topology"
        >
          Refresh
        </button>
      </header>
      {!snapshot ? (
        <p className="multiplexer-topology-empty" role="status">
          {status}
        </p>
      ) : snapshot.workspaces.length === 0 ? (
        <p className="multiplexer-topology-empty" role="status">
          No owner-authoritative workspace topology is currently available.
        </p>
      ) : (
        <div
          className="multiplexer-workspaces"
          role="group"
          aria-label="Workspace, tab, and pane topology"
        >
          {snapshot.workspaces.map((workspace) => {
            const boundToWorkspace =
              bound?.multiplexerWorkspaceId === workspace.multiplexerWorkspaceId;
            const activeTabId =
              boundToWorkspace && bound?.tabId
                ? bound.tabId
                : workspace.focusedTabId;

            return (
              <section
                key={workspace.multiplexerWorkspaceId}
                className="multiplexer-workspace"
                data-workspace-id={workspace.multiplexerWorkspaceId}
                aria-label={`Workspace ${workspace.displayLabel}`}
              >
                <button
                  type="button"
                  className="multiplexer-workspace-title"
                  aria-pressed={boundToWorkspace && !bound?.tabId}
                  onClick={() =>
                    void bind({
                      multiplexerWorkspaceId: workspace.multiplexerWorkspaceId,
                    })
                  }
                >
                  <span>{workspace.displayLabel}</span>
                  <code>{workspace.multiplexerWorkspaceId}</code>
                </button>
                <div
                  className="multiplexer-tabs"
                  role="group"
                  aria-label={`${workspace.displayLabel} tabs`}
                >
                  {workspace.tabs.map((tab) => {
                    const count = paneCount(tab.root);
                    return (
                      <button
                        key={tab.tabId}
                        type="button"
                        aria-pressed={activeTabId === tab.tabId}
                        data-tab-id={tab.tabId}
                        onClick={() =>
                          void bind({
                            multiplexerWorkspaceId:
                              workspace.multiplexerWorkspaceId,
                            tabId: tab.tabId,
                          })
                        }
                      >
                        <span>{tab.displayLabel}</span>
                        <small>
                          {count} pane{count === 1 ? "" : "s"}
                        </small>
                      </button>
                    );
                  })}
                </div>
                {workspace.tabs.map((tab) => (
                  <div
                    key={tab.tabId}
                    className="multiplexer-tab-layout"
                    hidden={activeTabId !== tab.tabId}
                  >
                    <LayoutNode
                      node={tab.root}
                      workspace={workspace}
                      tab={tab}
                      bound={bound}
                      bind={bind}
                    />
                  </div>
                ))}
              </section>
            );
          })}
        </div>
      )}
    </section>
  );
}
