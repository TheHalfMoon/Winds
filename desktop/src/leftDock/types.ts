export type BridgeRuntimeFamily = "codex" | "claude";
export type BridgeRuntimeState =
  | "requested"
  | "observed"
  | "mismatch"
  | "unknown"
  | "unavailable"
  | "stale"
  | "conflicting";
export type BridgeAttentionState =
  | "recovery_required"
  | "waiting_approval"
  | "blocked"
  | "retry_required"
  | "waiting_external"
  | "stale"
  | "unknown"
  | "none";

export interface BridgeAttentionItem {
  readonly workspaceId: string;
  readonly sessionId: string;
  readonly workflowRunId: string;
  readonly stageRunId: string;
  readonly stageKey: string;
  readonly state: BridgeAttentionState;
  readonly reason: string;
  readonly source: string;
  readonly authority: string;
  readonly candidateOid: string | null;
  readonly candidateTree: string | null;
  readonly approvalActionAvailable: boolean;
}

export interface BridgeAttentionSnapshot {
  readonly items: readonly BridgeAttentionItem[];
}

export interface BridgeRuntimeProjection {
  readonly state: BridgeRuntimeState;
  readonly requested: readonly BridgeRuntimeFamily[];
  readonly observed: readonly BridgeRuntimeFamily[];
}

export interface BridgeProjectSummary {
  readonly projectViewId: string;
  readonly displayName: string;
  readonly canonicalWorkspaceId: string;
  readonly canonicalRepoRoot: string;
  readonly canonicalGitCommonDir: string;
  readonly pinned: boolean;
  readonly presentationOrder: number;
  readonly collapsed: boolean;
  readonly presentationRevision: number | null;
  readonly sessionCount: number;
  readonly attentionCount: number;
  readonly attention: BridgeAttentionState;
  readonly searchInput: string;
}

export interface BridgeSessionSummary {
  readonly canonicalSessionId: string;
  readonly canonicalWorkstreamId: string;
  readonly canonicalWorkspaceId: string;
  readonly displayName: string;
  readonly displayAlias: string | null;
  readonly canonicalDisplayName: string;
  readonly pinned: boolean;
  readonly presentationOrder: number;
  readonly archived: boolean;
  readonly presentationRevision: number | null;
  readonly runtime: BridgeRuntimeProjection;
  readonly attention: BridgeAttentionState;
  readonly searchInput: string;
}

export interface BridgeWorkstream {
  readonly workstreamId: string;
  readonly displayName: string;
}

export interface BridgeProject {
  readonly project: BridgeProjectSummary;
  readonly sessions: readonly BridgeSessionSummary[];
  readonly availableWorkstreams: readonly BridgeWorkstream[];
}

export interface BridgeSnapshot {
  readonly projects: readonly BridgeProject[];
}

export interface ProjectPresentationRequest {
  readonly workspaceId: string;
  readonly displayName: string;
  readonly pinned: boolean;
  readonly sortOrder: number;
  readonly collapsed: boolean;
  readonly expectedRevision: number | null;
}

export interface SessionPresentationRequest {
  readonly sessionId: string;
  readonly displayAlias: string;
  readonly pinned: boolean;
  readonly sortOrder: number;
  readonly archived: boolean;
  readonly expectedRevision: number | null;
}

export interface CreateSessionRequest {
  readonly workspaceId: string;
  readonly workstreamId: string;
  readonly displayName: string;
}

export interface RenameSessionRequest {
  readonly sessionId: string;
  readonly displayName: string;
  readonly presentation: SessionPresentationRequest;
}

export type BridgeLayoutMode = "SINGLE" | "DUAL";

export interface BridgeLayoutPresentation {
  readonly workspaceId: string;
  readonly layoutMode: BridgeLayoutMode;
  readonly leftSessionId: string | null;
  readonly rightSessionId: string | null;
  readonly splitBasisPoints: number;
  readonly revision: number | null;
}

export interface LayoutPresentationRequest {
  readonly workspaceId: string;
  readonly layoutMode: BridgeLayoutMode;
  readonly leftSessionId: string | null;
  readonly rightSessionId: string | null;
  readonly splitBasisPoints: number;
  readonly expectedRevision: number | null;
}

export interface LeftDockBridge {
  readonly source: "canonical" | "fixture";
  snapshot(): Promise<BridgeSnapshot>;
  attentionSnapshot(): Promise<BridgeAttentionSnapshot>;
  updateProject(requests: readonly ProjectPresentationRequest[]): Promise<readonly BridgeProjectSummary[]>;
  createSession(request: CreateSessionRequest): Promise<BridgeSessionSummary>;
  renameSession(request: RenameSessionRequest): Promise<BridgeSessionSummary>;
  updateSession(requests: readonly SessionPresentationRequest[]): Promise<readonly BridgeSessionSummary[]>;
  loadLayout(workspaceId: string): Promise<BridgeLayoutPresentation | null>;
  saveLayout(request: LayoutPresentationRequest): Promise<BridgeLayoutPresentation>;
}
