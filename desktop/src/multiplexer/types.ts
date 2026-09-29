export type DesktopTopologyLayoutNode =
  | { kind: "PANE"; paneId: string; presentationEpoch: number }
  | {
      kind: "SPLIT";
      axis: "HORIZONTAL" | "VERTICAL";
      ratioBasisPoints: number;
      first: DesktopTopologyLayoutNode;
      second: DesktopTopologyLayoutNode;
    };

export interface DesktopTopologyTab {
  tabId: string;
  displayLabel: string;
  focused: boolean;
  focusedPaneId: string;
  zoomedPaneId: string | null;
  root: DesktopTopologyLayoutNode;
}

export interface DesktopTopologyWorkspace {
  multiplexerWorkspaceId: string;
  displayLabel: string;
  topologyGeneration: number;
  focusedTabId: string;
  tabs: DesktopTopologyTab[];
}

export interface DesktopTopologySnapshot {
  authority: "OWNER_AUTHORITATIVE_TOPOLOGY";
  topologyGeneration: number;
  workspaces: DesktopTopologyWorkspace[];
}

export interface DesktopTopologyCapability {
  authority: "OWNER_AUTHORITATIVE_TOPOLOGY";
  trustedRustHost: boolean;
  rendererDirectOwnerAccess: boolean;
  rendererSuppliedOwnerGeneration: boolean;
  controllingTtyRequired: boolean;
  terminalSurfaceCapable: boolean;
  genericInvokeSurface: boolean;
}

export interface DesktopTopologyBoundTarget {
  authority: "OWNER_AUTHORITATIVE_TOPOLOGY";
  topologyGeneration: number;
  multiplexerWorkspaceId: string;
  tabId: string | null;
  paneId: string | null;
}

export interface DesktopTopologyIdentity {
  multiplexerWorkspaceId: string;
  tabId?: string;
  paneId?: string;
}
