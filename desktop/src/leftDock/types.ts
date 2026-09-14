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
  | "retry_required"
  | "waiting_external"
  | "stale"
  | "unknown"
  | "none";

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

export interface LeftDockBridge {
  readonly source: "canonical" | "fixture";
  snapshot(): Promise<BridgeSnapshot>;
  updateProject(requests: readonly ProjectPresentationRequest[]): Promise<readonly BridgeProjectSummary[]>;
  createSession(request: CreateSessionRequest): Promise<BridgeSessionSummary>;
  renameSession(request: RenameSessionRequest): Promise<BridgeSessionSummary>;
  updateSession(requests: readonly SessionPresentationRequest[]): Promise<readonly BridgeSessionSummary[]>;
}
