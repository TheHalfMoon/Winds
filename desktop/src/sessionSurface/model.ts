import type {
  ComposerSubmission,
  SessionSurfaceFixture,
  SessionWorkEvent,
  WorkEventSource,
  WorkEventTrust,
} from "./types";

const trustBySource: Record<WorkEventSource, WorkEventTrust> = {
  user: "user_supplied",
  agent: "agent_reported",
  winds: "winds_observed",
  human: "human_decided",
  fixture: "fixture_only",
};

const sourceLabelBySource: Record<WorkEventSource, string> = {
  user: "User supplied",
  agent: "Agent reported",
  winds: "Winds observed",
  human: "Human decided",
  fixture: "Fixture only",
};

export function eventTrust(event: SessionWorkEvent): WorkEventTrust {
  return trustBySource[event.source];
}

export function eventSourceLabel(event: SessionWorkEvent): string {
  return sourceLabelBySource[event.source];
}

export function eventCanClaimTrustedState(event: SessionWorkEvent): boolean {
  return event.source === "winds" || event.source === "human";
}

export function composerAvailability(session: SessionSurfaceFixture): {
  readonly enabled: boolean;
  readonly label: string;
} {
  if (session.composerMode === "fixture_only") {
    return {
      enabled: true,
      label: "Fixture only · prompts are recorded locally and never dispatched",
    };
  }
  return {
    enabled: false,
    label: "Direct runtime input unavailable · canonical desktop launch authority is not established",
  };
}

export function createFixtureSubmission(
  session: SessionSurfaceFixture,
  draft: string,
  sequence: number,
): ComposerSubmission | null {
  const body = draft.trim();
  if (session.composerMode !== "fixture_only" || body.length === 0) return null;
  return {
    targetSessionId: session.canonicalSessionId,
    delivery: "fixture_only",
    event: {
      id: `${session.canonicalSessionId}-fixture-prompt-${sequence}`,
      kind: "user_prompt",
      title: "You",
      body,
      source: "user",
      meta: "Local fixture · not dispatched",
    },
  };
}
