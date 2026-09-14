import type { RuntimeFamily } from "../runtime";

export type SessionRuntimeProofState =
  | "observed"
  | "requested"
  | "mismatch"
  | "unknown"
  | "unavailable"
  | "stale"
  | "conflicting";

export type WorkEventKind =
  | "user_prompt"
  | "agent_response"
  | "tool_action"
  | "command_result"
  | "file_change"
  | "test_check"
  | "approval_attention"
  | "error"
  | "completion";

export type WorkEventSource = "user" | "agent" | "winds" | "human" | "fixture";
export type WorkEventTrust =
  | "user_supplied"
  | "agent_reported"
  | "winds_observed"
  | "human_decided"
  | "fixture_only";

export interface SessionRuntimePresentation {
  readonly family: RuntimeFamily;
  readonly proofState: SessionRuntimeProofState;
  readonly label: string;
  readonly proof: string;
}

export interface WorkEventDetail {
  readonly target: string;
  readonly status: string;
  readonly source: string;
  readonly failure?: string;
}

export interface SessionWorkEvent {
  readonly id: string;
  readonly kind: WorkEventKind;
  readonly title: string;
  readonly body: string;
  readonly source: WorkEventSource;
  readonly meta?: string;
  readonly detail?: WorkEventDetail;
  readonly dockIntent?: "files" | "changes";
}

export type SessionSurfaceState = "ready" | "loading" | "error" | "empty";
export type ComposerMode = "fixture_only" | "unavailable";

export interface SessionSurfaceFixture {
  readonly canonicalSessionId: string;
  readonly canonicalWorkstreamId: string;
  readonly canonicalWorkspaceId: string;
  readonly displayName: string;
  readonly lifecycle: string;
  readonly worktreeContext: string;
  readonly runtime: SessionRuntimePresentation;
  readonly surfaceState: SessionSurfaceState;
  readonly composerMode: ComposerMode;
  readonly events: readonly SessionWorkEvent[];
}

export interface ComposerSubmission {
  readonly targetSessionId: string;
  readonly delivery: "fixture_only";
  readonly event: SessionWorkEvent;
}
